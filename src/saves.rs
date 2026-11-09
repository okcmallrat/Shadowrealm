//! Original save image layout, bounded native file envelope and atomic slot I/O.
//! Transient gameplay reconstruction is staged by Game, never deserialized live.
use crate::{dialogue_data::Address, RomError};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub const SLOT_BYTES: usize = 2048;
pub const GLOBAL_OFFSET: usize = 0x3d6;
pub const GLOBAL_BYTES: usize = 0x248;
pub const COORDINATES: usize = GLOBAL_OFFSET + GLOBAL_BYTES;
pub const CHECKSUM: usize = 0x630;
pub const MAX_FILE_BYTES: u64 = 32768;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct References {
    schema_version: u8,
    scope: String,
    gate: Address,
    writer: Address,
    reader: Address,
    checksum: Address,
    header: Address,
    records: usize,
    record_bytes: usize,
    global_offset: usize,
    global_bytes: usize,
    checksum_offset: usize,
    slot_bytes: usize,
}
pub fn validate_references(rom: &[u8]) -> Result<(), RomError> {
    let r: References = serde_json::from_str(include_str!("../data/saves/references.json"))
        .map_err(|_| RomError("invalid save reference JSON"))?;
    if r.schema_version != 1
        || r.scope.is_empty()
        || [r.gate.0, r.writer.0, r.reader.0, r.checksum.0, r.header.0]
            != [0xefb4, 0x596c4, 0x594e0, 0x59786, 0x59972]
        || [
            r.records,
            r.record_bytes,
            r.global_offset,
            r.global_bytes,
            r.checksum_offset,
            r.slot_bytes,
        ] != [13, 74, GLOBAL_OFFSET, GLOBAL_BYTES, CHECKSUM, SLOT_BYTES]
        || rom.get(r.header.0 + 4..r.header.0 + 8) != Some(b"GAME")
    {
        return Err(RomError("save references outside verified scope"));
    }
    Ok(())
}
pub fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
pub fn long(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}
pub fn put_word(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
}
pub fn put_long(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_be_bytes());
}
/// $59786 sums exactly $61c bytes after the twenty-byte header.
pub fn original_checksum(image: &[u8; SLOT_BYTES]) -> u32 {
    image[20..CHECKSUM].iter().map(|&b| u32::from(b)).sum()
}
/// $596c4/$59798: compact archives, raw global block, map, camera, positions.
pub fn image(
    rom: &[u8],
    records: &[[u8; 74]; 13],
    global: &[u8; GLOBAL_BYTES],
    map: u16,
    camera: (u16, u16),
    positions: [(u16, u16); 3],
) -> Result<[u8; SLOT_BYTES], RomError> {
    validate_references(rom)?;
    let mut image = [0; SLOT_BYTES];
    image[..20].copy_from_slice(&rom[0x59972..0x59986]);
    for (i, r) in records.iter().enumerate() {
        image[20 + i * 74..20 + (i + 1) * 74].copy_from_slice(r);
    }
    image[GLOBAL_OFFSET..COORDINATES].copy_from_slice(global);
    put_word(&mut image, COORDINATES, map);
    put_word(&mut image, COORDINATES + 2, camera.0);
    put_word(&mut image, COORDINATES + 4, camera.1);
    for (slot, (x, y)) in positions.into_iter().enumerate() {
        put_word(&mut image, COORDINATES + 6 + slot * 4, x);
        put_word(&mut image, COORDINATES + 8 + slot * 4, y);
    }
    let checksum = original_checksum(&image);
    put_long(&mut image, CHECKSUM, checksum);
    Ok(image)
}
pub fn validate_image(bytes: &[u8]) -> Result<[u8; SLOT_BYTES], RomError> {
    let image: [u8; SLOT_BYTES] = bytes
        .try_into()
        .map_err(|_| RomError("save image length differs from original slot"))?;
    if &image[4..8] != b"GAME" || long(&image, CHECKSUM) != original_checksum(&image) {
        return Err(RomError("save image signature or checksum is invalid"));
    }
    Ok(image)
}
/// Host envelope CRC, distinct from the original additive checksum.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    !crc
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Native {
    pub hero_template: usize,
    pub tick: u64,
    pub rng: u16,
    pub job_base_pay: u16,
    pub job_accepted: bool,
    pub completed_jobs: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    format: String,
    version: u8,
    rom_bytes: usize,
    rom_product: String,
    rom_checksum: u16,
    pub image: Vec<u8>,
    pub native: Native,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload: Payload,
    crc32: u32,
}
impl Payload {
    pub fn new(image: [u8; SLOT_BYTES], native: Native) -> Self {
        Self {
            format: "shadowrealm-native-save".into(),
            version: 1,
            rom_bytes: 0x200000,
            rom_product: "GM MK-1352 -00".into(),
            rom_checksum: 0x6baf,
            image: image.to_vec(),
            native,
        }
    }
    pub fn validate(&self) -> Result<[u8; SLOT_BYTES], RomError> {
        if self.format != "shadowrealm-native-save"
            || self.version != 1
            || self.rom_bytes != 0x200000
            || self.rom_product != "GM MK-1352 -00"
            || self.rom_checksum != 0x6baf
            || self.native.hero_template > 2
        {
            return Err(RomError("unsupported save format, version or ROM revision"));
        }
        validate_image(&self.image)
    }
    pub fn encode(self) -> Result<Vec<u8>, RomError> {
        self.validate()?;
        let bytes =
            serde_json::to_vec(&self).map_err(|_| RomError("unable to encode save payload"))?;
        let output = serde_json::to_vec_pretty(&Envelope {
            payload: self,
            crc32: crc32(&bytes),
        })
        .map_err(|_| RomError("unable to encode save file"))?;
        if output.len() as u64 > MAX_FILE_BYTES {
            return Err(RomError("save file exceeds size bound"));
        }
        Ok(output)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, RomError> {
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(RomError("save file exceeds size bound"));
        }
        let envelope: Envelope =
            serde_json::from_slice(bytes).map_err(|_| RomError("invalid native save JSON"))?;
        let canonical = serde_json::to_vec(&envelope.payload)
            .map_err(|_| RomError("unable to validate save file"))?;
        if crc32(&canonical) != envelope.crc32 {
            return Err(RomError("native save checksum is invalid"));
        }
        envelope.payload.validate()?;
        Ok(envelope.payload)
    }
}
pub fn slot_path(directory: &Path, slot: u8) -> io::Result<PathBuf> {
    if !(1..=2).contains(&slot) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "save slot must be 1 or 2",
        ));
    }
    Ok(directory.join(format!("slot-{slot}.json")))
}
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
}
pub fn read_bounded(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save file is too large",
        ));
    }
    let mut bytes = vec![];
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save file is too large",
        ));
    }
    Ok(bytes)
}
fn stage_file(path: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "save directory missing"))?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "save filename missing"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let mut temp_name = std::ffi::OsString::from(".");
    temp_name.push(name);
    temp_name.push(format!(".{}.{stamp}.tmp", std::process::id()));
    let temp = parent.join(temp_name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()
    })();
    drop(file);
    if let Err(error) = result {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    Ok(temp)
}
/// Same-directory temporary files and replacement; the previous valid primary
/// becomes .bak. An invalid old primary cannot overwrite a valid backup.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    write_atomic_checked(path, bytes, |data| Payload::decode(data).is_ok())
}
pub fn write_atomic_checked(
    path: &Path,
    bytes: &[u8],
    valid: impl Fn(&[u8]) -> bool,
) -> io::Result<()> {
    if !valid(bytes) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save validation failed",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "save directory missing"))?;
    fs::create_dir_all(parent)?;
    let temp = stage_file(path, bytes)?;
    let result = (|| {
        match read_bounded(path) {
            Ok(old) if valid(&old) => {
                let backup = backup_path(path);
                let staged = stage_file(&backup, &old)?;
                if let Err(error) = fs::rename(&staged, &backup) {
                    let _ = fs::remove_file(staged);
                    return Err(error);
                }
            }
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::InvalidData
                ) => {}
            Err(error) => return Err(error),
        }
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crc_matches_standard_check_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf43926);
        assert_eq!(crc32(b""), 0);
    }
    #[test]
    fn malformed_envelopes_are_bounded_and_rejected() {
        assert!(Payload::decode(b"{}").is_err());
        assert!(Payload::decode(&vec![b' '; MAX_FILE_BYTES as usize + 1]).is_err());
        assert!(slot_path(Path::new("saves"), 0).is_err());
        assert!(validate_image(&[0; SLOT_BYTES]).is_err());
    }
}
