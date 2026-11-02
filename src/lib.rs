pub mod actors;
pub mod archives;
pub mod attack_roll;
pub mod avoidance;
pub mod boot;
pub mod clinic;
pub mod collision;
pub mod combat;
pub mod combat_runtime;
pub mod damage;
pub mod dialogue;
pub mod dialogue_data;
pub mod encounters;
pub mod environment;
pub mod followers;
pub mod frame;
pub mod game;
pub mod hiring;
pub mod hud;
pub mod input;
pub mod map_render;
pub mod maps;
pub mod npc;
pub mod npc_combat;
pub mod party;
pub mod projectiles;
pub mod scene;
pub mod transitions;
pub mod video;
pub mod weapons;
// ROM inspection primitives for a native Shadowrun reconstruction.
use std::fmt;
#[derive(Debug, PartialEq)]
pub struct RomError(pub &'static str);
impl fmt::Display for RomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for RomError {}
pub struct Rom<'a> {
    bytes: &'a [u8],
}
impl<'a> Rom<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, RomError> {
        if bytes.len() < 0x200 {
            return Err(RomError("ROM is shorter than its header"));
        }
        if bytes.len() % 2 != 0 {
            return Err(RomError("ROM has an incomplete word"));
        }
        if &bytes[0x100..0x104] != b"SEGA" {
            return Err(RomError("expected a raw big-endian Genesis ROM"));
        }
        Ok(Self { bytes })
    }
    pub fn text(&self, start: usize, end: usize) -> Result<String, RomError> {
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(RomError("text range outside ROM"))?;
        Ok(String::from_utf8_lossy(bytes).trim().to_owned())
    }
    pub fn vector(&self, index: usize) -> Result<u32, RomError> {
        if index >= 64 {
            return Err(RomError("vector index must be below 64"));
        }
        let offset = index * 4;
        Ok(u32::from_be_bytes(
            self.bytes[offset..offset + 4].try_into().unwrap(),
        ))
    }
    pub fn stored_checksum(&self) -> u16 {
        u16::from_be_bytes([self.bytes[0x18e], self.bytes[0x18f]])
    }
    pub fn checksum(&self) -> u16 {
        self.bytes[0x200..].chunks_exact(2).fold(0u16, |sum, w| {
            sum.wrapping_add(u16::from_be_bytes([w[0], w[1]]))
        })
    }
    /// Decode one uncompressed Genesis 8x8, 4-bit tile. Palette assignment is unknown.
    pub fn tile(&self, offset: usize) -> Result<[u8; 64], RomError> {
        let end = offset
            .checked_add(32)
            .ok_or(RomError("tile offset overflow"))?;
        let data = self
            .bytes
            .get(offset..end)
            .ok_or(RomError("tile outside ROM"))?;
        let mut pixels = [0; 64];
        for (i, byte) in data.iter().enumerate() {
            pixels[i * 2] = byte >> 4;
            pixels[i * 2 + 1] = byte & 15;
        }
        Ok(pixels)
    }
    pub fn strings(&self, minimum: usize) -> Vec<(usize, String)> {
        let mut result = Vec::new();
        let mut start = 0;
        for end in 0..=self.bytes.len() {
            if end == self.bytes.len() || !(32..=126).contains(&self.bytes[end]) {
                if end - start >= minimum {
                    result.push((
                        start,
                        String::from_utf8_lossy(&self.bytes[start..end]).into_owned(),
                    ));
                }
                start = end + 1;
            }
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Vec<u8> {
        let mut b = vec![0; 0x240];
        b[0x100..0x104].copy_from_slice(b"SEGA");
        b
    }
    #[test]
    fn rejects_bad_headers() {
        assert!(Rom::parse(&[]).is_err());
        assert!(Rom::parse(&vec![0; 512]).is_err());
        let mut b = sample();
        b.push(0);
        assert!(Rom::parse(&b).is_err());
    }
    #[test]
    fn checksum_wraps_big_endian_words() {
        let mut b = sample();
        b[0x200..0x204].copy_from_slice(&[255, 255, 0, 2]);
        assert_eq!(Rom::parse(&b).unwrap().checksum(), 1);
    }
    #[test]
    fn vectors_are_big_endian_and_bounded() {
        let mut b = sample();
        b[4..8].copy_from_slice(&[0, 0, 2, 54]);
        let r = Rom::parse(&b).unwrap();
        assert_eq!(r.vector(1).unwrap(), 0x236);
        assert!(r.vector(64).is_err());
    }
    #[test]
    fn tile_nibbles_and_bounds() {
        let mut b = sample();
        b[512] = 0xab;
        let r = Rom::parse(&b).unwrap();
        assert_eq!(&r.tile(512).unwrap()[..2], &[10, 11]);
        assert!(r.tile(usize::MAX).is_err());
        assert!(r.tile(b.len() - 31).is_err());
    }
    #[test]
    fn strings_include_final_run() {
        let mut b = sample();
        b.extend_from_slice(b"HELLO");
        b.push(b'!');
        let r = Rom::parse(&b).unwrap();
        assert!(r.strings(6).contains(&(0x240, "HELLO!".into())));
    }
}
