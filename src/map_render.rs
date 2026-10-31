//! Static map-plane reconstruction from ROM setup script $1406 and renderer $3d18.
//! Excludes sprites, the HUD, animated transfers, palette fades and raster effects.
use crate::{
    maps::{decompress, MapDescriptor},
    RomError,
};
fn slice(b: &[u8], p: usize, n: usize) -> Result<&[u8], RomError> {
    b.get(p..p.checked_add(n).ok_or(RomError("render offset overflow"))?)
        .ok_or(RomError("render data outside source"))
}
fn word(b: &[u8], p: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(slice(b, p, 2)?.try_into().unwrap()))
}
fn long(b: &[u8], p: usize) -> Result<u32, RomError> {
    Ok(u32::from_be_bytes(slice(b, p, 4)?.try_into().unwrap()))
}
pub fn rgb(color: u16) -> [u8; 3] {
    [
        ((((color >> 1) & 7) * 255) / 7) as u8,
        ((((color >> 5) & 7) * 255) / 7) as u8,
        ((((color >> 9) & 7) * 255) / 7) as u8,
    ]
}
/// Genesis tile word: index 0..10, H/V flips 11/12, palette 13..14, priority 15.
fn sample(vram: &[u8], attr: u16, x: usize, y: usize) -> Result<u8, RomError> {
    let x = if attr & 0x800 != 0 { 7 - x } else { x };
    let y = if attr & 0x1000 != 0 { 7 - y } else { y };
    let byte = *vram
        .get((attr as usize & 0x7ff) * 32 + y * 4 + x / 2)
        .ok_or(RomError("tile outside VRAM"))?;
    Ok(if x & 1 == 0 { byte >> 4 } else { byte & 15 })
}
fn composite(samples: [(u16, u8); 2], palette: &[u16; 64], backdrop: usize) -> [u8; 3] {
    let mut color = palette[backdrop];
    // Layer 0 is plane B, layer 1 is plane A. A wins ties; index zero transparent.
    for high in [false, true] {
        for (attr, index) in samples {
            if index != 0 && (attr & 0x8000 != 0) == high {
                color = palette[((attr as usize >> 13) & 3) * 16 + index as usize];
            }
        }
    }
    rgb(color)
}
#[derive(Clone)]
pub struct MapScene {
    descriptor: MapDescriptor,
    grids: [Vec<u8>; 2],
    definitions: [Vec<u8>; 2],
    vram: Vec<u8>,
    pub palette: [u16; 64],
    backdrop: usize,
}
impl MapScene {
    pub fn load(rom: &[u8], id: usize) -> Result<Self, RomError> {
        let descriptor = MapDescriptor::read(rom, id)?;
        let setup = long(&descriptor.raw, 0x34)? as usize;
        let registers = long(rom, setup)? as usize;
        // These known scripts map layer 0 to B at E000 and layer 1 to A at C000.
        if word(rom, registers)? != 0xc000 || word(rom, registers + 2)? != 0xe000 {
            return Err(RomError("unsupported map plane configuration"));
        }
        let mut backdrop = 0;
        for i in 0..15 {
            let r = word(rom, registers + 8 + i * 2)?;
            if r >> 8 == 0x87 {
                backdrop = r as usize & 63;
            }
        }
        let pal = long(rom, setup + 4)? as usize;
        let mut palette = [0; 64];
        for (i, c) in palette.iter_mut().enumerate() {
            *c = word(rom, pal + i * 2)?;
        }
        // $2c72 calls $566c2 after $1406: replace target banks 1..3.
        for (i, color) in palette.iter_mut().enumerate().skip(16) {
            *color = word(rom, 0x18ac56 + (i - 16) * 2)?;
        }
        let mut vram = vec![0; 65536];
        let mut cursor = setup + 16;
        let mut terminated = false;
        for _ in 0..256 {
            let source = long(rom, cursor)?;
            cursor += 4;
            if source == 0 {
                terminated = true;
                break;
            }
            let destination = long(rom, cursor)? as usize;
            let size = long(rom, cursor + 4)? as usize;
            cursor += 8;
            if source & 1 != 0 {
                let (decoded, _) = decompress(slice(
                    rom,
                    (source & !1) as usize,
                    rom.len()
                        .checked_sub((source & !1) as usize)
                        .ok_or(RomError("invalid graphics source"))?,
                )?)?;
                let end = destination
                    .checked_add(decoded.len())
                    .ok_or(RomError("VRAM range overflow"))?;
                vram.get_mut(destination..end)
                    .ok_or(RomError("graphics outside VRAM"))?
                    .copy_from_slice(&decoded);
            } else {
                let data = slice(rom, source as usize, size)?;
                let end = destination
                    .checked_add(size)
                    .ok_or(RomError("VRAM range overflow"))?;
                vram.get_mut(destination..end)
                    .ok_or(RomError("graphics outside VRAM"))?
                    .copy_from_slice(data);
            }
        }
        if !terminated {
            return Err(RomError("unterminated graphics load list"));
        }
        // All current map scripts have an empty secondary tilemap initialization list.
        if long(rom, cursor)? != 0 {
            return Err(RomError("unsupported secondary setup list"));
        }
        let grids = [
            descriptor.grid_copy(rom, 0)?.to_vec(),
            descriptor.grid_copy(rom, 1)?.to_vec(),
        ];
        let first = descriptor.tile_pointer(0)?;
        let definitions = if first & 1 != 0 {
            let start = (first & !1) as usize;
            let (data, _) = decompress(
                rom.get(start..)
                    .ok_or(RomError("definitions outside ROM"))?,
            )?;
            [data.clone(), data]
        } else {
            // Only retain the referenced definition span, rather than guessing table length.
            let mut layers = [Vec::new(), Vec::new()];
            for layer in 0..2 {
                let max = grids[layer]
                    .chunks_exact(2)
                    .map(|w| u16::from_be_bytes([w[0], w[1]]) as usize)
                    .max()
                    .unwrap_or(0);
                let start = descriptor.tile_pointer(layer)? as usize;
                // Copy only available ROM data; later sampling checks individual accesses.
                let length = ((max + 1) * 32).min(rom.len().saturating_sub(start));
                layers[layer] = slice(rom, start, length)?.to_vec();
            }
            layers
        };
        Ok(Self {
            descriptor,
            grids,
            definitions,
            vram,
            palette,
            backdrop,
        })
    }
    #[cfg(test)]
    pub(crate) fn sprite_fixture(vram: Vec<u8>) -> Self {
        Self {
            descriptor: MapDescriptor {
                id: 0,
                address: 0,
                raw: [0; 96],
            },
            grids: [vec![], vec![]],
            definitions: [vec![], vec![]],
            vram,
            palette: [0; 64],
            backdrop: 0,
        }
    }
    /// Collision setup can write outside its raster in the original ROM. Preserve
    /// those writes when they alias copied map grids, without indexing unsafely.
    pub(crate) fn apply_collision_overflow(
        &mut self,
        writes: &[(i16, u8)],
    ) -> Result<(), RomError> {
        for (offset, value) in writes {
            let address = 0xff1700i64 + i64::from(*offset);
            let mut applied = false;
            for (layer, base) in [(0, 0xff2700i64), (1, 0xff4700i64)] {
                if address >= base && address < base + self.grids[layer].len() as i64 {
                    self.grids[layer][(address - base) as usize] = *value;
                    applied = true;
                }
            }
            if !applied {
                return Err(RomError(
                    "collision setup writes into an unmodeled RAM region",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn definition_word(&self, block: u16, tile: usize) -> Result<u16, RomError> {
        word(&self.definitions[0], block as usize * 32 + tile * 2)
    }
    pub fn vram(&self) -> &[u8] {
        &self.vram
    }
    pub(crate) fn transfer(
        &mut self,
        rom: &[u8],
        source: usize,
        destination: usize,
        length: usize,
    ) -> Result<(), RomError> {
        let payload = slice(rom, source, length)?;
        let end = destination
            .checked_add(length)
            .ok_or(RomError("VRAM transfer overflow"))?;
        self.vram
            .get_mut(destination..end)
            .ok_or(RomError("transfer outside VRAM"))?
            .copy_from_slice(payload);
        Ok(())
    }
    pub(crate) fn tile_pixel(&self, tile: u16, x: usize, y: usize) -> Result<u8, RomError> {
        sample(&self.vram, tile, x, y)
    }
    pub(crate) fn frame_pixel(
        &self,
        x: u16,
        y: u16,
        screen_x: usize,
        screen_y: usize,
        window: Option<u16>,
        sprite: Option<(u16, u8)>,
    ) -> Result<[u8; 3], RomError> {
        // The hardware keeps fetching B beneath the window, including the next
        // linear grid row when the crop extends beyond the visible map columns.
        let b = if window.is_some() {
            self.attribute_linear(0, x, y)?
        } else {
            self.attribute(0, x, y)?
        };
        let a = if let Some(attr) = window {
            attr
        } else {
            self.attribute(1, x, y)?
        };
        let bp = sample(&self.vram, b, x as usize & 7, y as usize & 7)?;
        let ap = if window.is_some() {
            sample(&self.vram, a, screen_x & 7, screen_y & 7)?
        } else {
            sample(&self.vram, a, x as usize & 7, y as usize & 7)?
        };
        let mut color = self.palette[self.backdrop];
        // VDP priority: B low, A/window low, sprite low, B high, A high, sprite high.
        for high in [false, true] {
            for (attr, index) in [(b, bp), (a, ap), sprite.unwrap_or((0, 0))] {
                if index != 0 && (attr & 0x8000 != 0) == high {
                    color = self.palette[((attr as usize >> 13) & 3) * 16 + index as usize];
                }
            }
        }
        Ok(rgb(color))
    }
    fn attribute(&self, layer: usize, x: u16, y: u16) -> Result<u16, RomError> {
        if (x as usize >> 5) * 2 >= self.descriptor.row_stride_bytes() as usize {
            return Err(RomError("crop exceeds map row stride"));
        }
        self.attribute_linear(layer, x, y)
    }
    fn attribute_linear(&self, layer: usize, x: u16, y: u16) -> Result<u16, RomError> {
        let (grid, inside) = self.descriptor.offsets(x, y);
        let block = word(&self.grids[layer], grid as usize)? as usize;
        let attr = word(&self.definitions[layer], block * 32 + inside as usize)?;
        // $3dae XORs priority on layer 1 (ffdf58 = $8000).
        Ok(attr ^ if layer == 1 { 0x8000 } else { 0 })
    }
    pub fn render(&self, x: u16, y: u16, width: usize, height: usize) -> Result<Vec<u8>, RomError> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || x as usize + width > 65536
            || y as usize + height > 65536
        {
            return Err(RomError("invalid render dimensions"));
        }
        // Do not allow 68000 word-offset wrap to silently select an unrelated grid row.
        let last_y = y as usize + height - 1;
        let last_x = x as usize + width - 1;
        if (last_y >> 5) * self.descriptor.row_stride_bytes() as usize + (last_x >> 5) * 2 + 2
            > self.grids[0].len()
        {
            return Err(RomError("crop exceeds copied grid buffer"));
        }
        let mut out = Vec::with_capacity(width * height * 3);
        for row in 0..height {
            for col in 0..width {
                let px = x + col as u16;
                let py = y + row as u16;
                let mut samples = [(0, 0); 2];
                for (layer, item) in samples.iter_mut().enumerate() {
                    let attr = self.attribute(layer, px, py)?;
                    *item = (
                        attr,
                        sample(&self.vram, attr, px as usize & 7, py as usize & 7)?,
                    );
                }
                out.extend_from_slice(&composite(samples, &self.palette, self.backdrop));
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tile_flips_and_nibbles() {
        let mut v = vec![0; 64];
        v[32] = 0x12;
        v[63] = 0x34;
        assert_eq!(sample(&v, 1, 0, 0).unwrap(), 1);
        assert_eq!(sample(&v, 1, 1, 0).unwrap(), 2);
        assert_eq!(sample(&v, 0x1801, 0, 0).unwrap(), 4);
        assert_eq!(sample(&v, 0x1801, 1, 0).unwrap(), 3);
        assert_eq!(sample(&v, 0x801, 7, 0).unwrap(), 1);
        assert_eq!(sample(&v, 0x1001, 7, 0).unwrap(), 4);
    }
    #[test]
    fn priority_transparency_and_palette_banks() {
        let mut p = [0; 64];
        p[1] = 0xe;
        p[17] = 0xe0;
        p[33] = 0xe00;
        assert_eq!(composite([(0, 1), (0x2000, 1)], &p, 0), [0, 255, 0]);
        assert_eq!(composite([(0x8000, 1), (0x2000, 1)], &p, 0), [255, 0, 0]);
        assert_eq!(composite([(0x8000, 1), (0xc000, 1)], &p, 0), [0, 0, 255]);
        assert_eq!(composite([(0, 1), (0xc000, 0)], &p, 0), [255, 0, 0]);
        assert_eq!(composite([(0, 0), (0, 0)], &p, 1), [255, 0, 0]);
    }
    #[test]
    fn channel_expansion() {
        assert_eq!(rgb(0xeee), [255; 3]);
        assert_eq!(rgb(0), [0; 3]);
    }
}
