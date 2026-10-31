//! Map-data operations recovered from ROM $3032, $3084, $313a, $1610, $3fd8.
//! Field names describe observed consumers; unknown descriptor bytes stay raw.
use crate::RomError;
pub const MAP_COUNT: usize = 56;
pub const DESCRIPTOR_TABLE: usize = 0x3232;
pub const GRID_COPY_BYTES: usize = 0x1c20;
fn bytes(data: &[u8], offset: usize, count: usize) -> Result<&[u8], RomError> {
    let end = offset
        .checked_add(count)
        .ok_or(RomError("map offset overflow"))?;
    data.get(offset..end)
        .ok_or(RomError("map data outside ROM"))
}
fn word(data: &[u8], offset: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(
        bytes(data, offset, 2)?.try_into().unwrap(),
    ))
}
fn long(data: &[u8], offset: usize) -> Result<u32, RomError> {
    Ok(u32::from_be_bytes(
        bytes(data, offset, 4)?.try_into().unwrap(),
    ))
}
pub struct MapDescriptor {
    pub id: usize,
    pub address: usize,
    pub raw: [u8; 96],
}
impl MapDescriptor {
    pub fn read(rom: &[u8], id: usize) -> Result<Self, RomError> {
        if id >= MAP_COUNT {
            return Err(RomError("map index must be below 56"));
        }
        if rom.get(0x180..0x18e) != Some(b"GM MK-1352 -00".as_slice()) {
            return Err(RomError("map offsets require Shadowrun GM MK-1352 -00"));
        }
        let address = long(rom, DESCRIPTOR_TABLE + id * 4)? as usize;
        Ok(Self {
            id,
            address,
            raw: bytes(rom, address, 96)?.try_into().unwrap(),
        })
    }
    pub fn grid_pointer(&self, layer: usize) -> Result<usize, RomError> {
        let offset = match layer {
            0 => 0x14,
            1 => 0x1c,
            _ => return Err(RomError("layer must be 0 or 1")),
        };
        Ok(long(&self.raw, offset)? as usize)
    }
    pub fn tile_pointer(&self, layer: usize) -> Result<u32, RomError> {
        let offset = match layer {
            0 => 0x18,
            1 => 0x20,
            _ => return Err(RomError("layer must be 0 or 1")),
        };
        long(&self.raw, offset)
    }
    pub fn row_stride_bytes(&self) -> u16 {
        word(&self.raw, 0x24).unwrap()
    }
    /// Exactly the fixed-size copy performed by $313a. This can include bytes
    /// beyond the logical grid. Logical height has not been established.
    pub fn grid_copy<'a>(&self, rom: &'a [u8], layer: usize) -> Result<&'a [u8], RomError> {
        bytes(rom, self.grid_pointer(layer)?, GRID_COPY_BYTES)
    }
    /// $3fd8: byte offset of a block ID plus byte offset within its 4x4 tiles.
    /// Coordinate inputs are pixel units after the renderer's fixed-point shift.
    pub fn offsets(&self, x: u16, y: u16) -> (u16, u16) {
        let grid = (y >> 5)
            .wrapping_mul(self.row_stride_bytes())
            .wrapping_add((x >> 5).wrapping_mul(2));
        let tile = (((y >> 3) & 3) * 4 + ((x >> 3) & 3)) * 2;
        (grid, tile)
    }
}
/// $3032 uses signed comparisons; the resulting flag lives at $fffc0e.
/// Its broader gameplay meaning remains unconfirmed.
pub fn special_map_flag(id: u16) -> bool {
    matches!(id, 6..=9) || (44..50).contains(&(id as i16))
}
/// Decode $1610's byte stream. The first word is output length minus one.
/// Control bits are read MSB first: 0 selects a literal; 1 reads a command
/// byte whose high bit selects RLE (0) or LZ (1).
/// Returns decoded bytes and the number of source bytes consumed.
pub fn decompress(source: &[u8]) -> Result<(Vec<u8>, usize), RomError> {
    let length = word(source, 0)? as usize + 1;
    let mut out = Vec::with_capacity(length);
    let mut pos = 2;
    let mut flags = 0u8;
    let mut bits = 0;
    while out.len() < length {
        if bits == 0 {
            flags = bytes(source, pos, 1)?[0];
            pos += 1;
            bits = 8;
        }
        let compressed = flags & 0x80 != 0;
        flags <<= 1;
        bits -= 1;
        if !compressed {
            out.push(bytes(source, pos, 1)?[0]);
            pos += 1;
            continue;
        }
        let command = bytes(source, pos, 1)?[0];
        pos += 1;
        let count = if command & 0x80 == 0 {
            (command & 0x7f) as usize + 3
        } else {
            (command & 15) as usize + 3
        };
        if count > length - out.len() {
            return Err(RomError("compressed token exceeds declared length"));
        }
        let value = bytes(source, pos, 1)?[0];
        pos += 1;
        if command & 0x80 == 0 {
            out.resize(out.len() + count, value);
        } else {
            let distance = ((value as usize) << 3) | ((command as usize >> 4) & 7);
            if distance == 0 || distance > out.len() {
                return Err(RomError("invalid compressed back-reference"));
            }
            for _ in 0..count {
                let byte = out[out.len() - distance];
                out.push(byte);
            }
        }
    }
    Ok((out, pos))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn literal_and_rle_decode() {
        assert_eq!(
            decompress(&[0, 3, 0x40, b'A', 0, b'B']).unwrap(),
            (b"ABBB".to_vec(), 6)
        );
    }
    #[test]
    fn overlapping_lz_copy() {
        // AB followed by four bytes copied from distance two.
        assert_eq!(
            decompress(&[0, 5, 0x20, b'A', b'B', 0xa1, 0]).unwrap().0,
            b"ABABAB"
        );
    }
    #[test]
    fn malformed_streams_are_rejected() {
        for stream in [
            &[0][..],
            &[0, 0, 0x80, 0x80, 0],
            &[0, 0, 0x80, 0, 4],
            &[0, 1, 0, b'A'],
        ] {
            assert!(decompress(stream).is_err());
        }
    }
    #[test]
    fn map_flag_boundaries() {
        for id in 0..=65535u16 {
            assert_eq!(
                special_map_flag(id),
                [6, 7, 8, 9, 44, 45, 46, 47, 48, 49].contains(&id)
            );
        }
    }
    #[test]
    fn block_and_tile_offsets() {
        let mut raw = [0; 96];
        raw[0x24..0x26].copy_from_slice(&120u16.to_be_bytes());
        let m = MapDescriptor {
            id: 0,
            address: 0,
            raw,
        };
        assert_eq!(m.offsets(0, 0), (0, 0));
        assert_eq!(m.offsets(31, 31), (0, 30));
        assert_eq!(m.offsets(40, 48), (122, 18));
    }
}
