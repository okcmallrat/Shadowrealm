//! Bounded expository text support. General branching conversation scripts are
//! deliberately not treated as plain strings or automatically bypassed.
use crate::{maps::decompress, RomError};
pub struct Dialogue {
    pub node: usize,
    pub text_address: usize,
    pub text: String,
    pub font: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn branching_and_control_scripts_are_not_bypassed() {
        let mut rom = vec![0; 32];
        rom[0..4].copy_from_slice(&16u32.to_be_bytes());
        assert!(Dialogue::exposition(&rom, 0)
            .err()
            .unwrap()
            .0
            .contains("branching"));
        rom[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
        rom[16] = 0x81;
        assert!(Dialogue::exposition(&rom, 0)
            .err()
            .unwrap()
            .0
            .contains("control"));
        rom[16..].fill(b'A');
        assert!(Dialogue::exposition(&rom, 0)
            .err()
            .unwrap()
            .0
            .contains("unterminated"));
    }
}
fn long(b: &[u8], p: usize) -> Result<u32, RomError> {
    let a = b
        .get(p..p + 4)
        .ok_or(RomError("dialogue pointer outside ROM"))?;
    Ok(u32::from_be_bytes(a.try_into().unwrap()))
}
impl Dialogue {
    pub fn exposition(rom: &[u8], node: usize) -> Result<Self, RomError> {
        let text_address = long(rom, node)? as usize;
        if long(rom, node + 4)? != u32::MAX {
            return Err(RomError(
                "dialogue node requires unimplemented branching script",
            ));
        }
        let mut text = String::new();
        let mut ended = false;
        for byte in rom
            .get(text_address..)
            .ok_or(RomError("text outside ROM"))?
            .iter()
            .take(8192)
        {
            match *byte {
                0xff => {
                    ended = true;
                    break;
                }
                0x80 => text.push('\n'),
                32..=126 => text.push(*byte as char),
                _ => return Err(RomError("unsupported dialogue control code")),
            }
        }
        if !ended {
            return Err(RomError("unterminated dialogue text"));
        }
        let (font, _) = decompress(
            rom.get(0xc1064..)
                .ok_or(RomError("dialogue font missing"))?,
        )?;
        Ok(Self {
            node,
            text_address,
            text,
            font,
        })
    }
    fn draw(
        &self,
        rom: &[u8],
        pixels: &mut [u8],
        text: &str,
        x: usize,
        y: usize,
    ) -> Result<(), RomError> {
        for (i, c) in text.bytes().enumerate() {
            if !(32..=126).contains(&c) {
                return Err(RomError("non-ASCII dialogue glyph"));
            }
            let mut index = usize::from(c - 32);
            if index >= 0x3d {
                index -= 4;
            }
            let p = 0xc1320 + index * 2;
            let b = rom.get(p..p + 2).ok_or(RomError("font map missing"))?;
            let attr = u16::from_be_bytes([b[0], b[1]]);
            let tile = (attr & 0x7ff)
                .checked_sub(0x5ab)
                .ok_or(RomError("font glyph outside decoded range"))?
                as usize;
            for row in 0..8 {
                for col in 0..8 {
                    let px = if attr & 0x800 != 0 { 7 - col } else { col };
                    let py = if attr & 0x1000 != 0 { 7 - row } else { row };
                    let byte = *self
                        .font
                        .get(tile * 32 + py * 4 + px / 2)
                        .ok_or(RomError("font tile missing"))?;
                    let value = if px & 1 == 0 { byte >> 4 } else { byte & 15 };
                    if value != 0 && x + i * 8 + col < 320 && y + row < 224 {
                        let at = ((y + row) * 320 + x + i * 8 + col) * 3;
                        pixels[at..at + 3].fill(240);
                    }
                }
            }
        }
        Ok(())
    }
    /// Host presentation using the recovered original glyphs and exact text.
    /// This is not yet the original full-screen portrait/conversation layout.
    pub fn render(&self, rom: &[u8], pixels: &mut [u8]) -> Result<(), RomError> {
        if pixels.len() != 320 * 224 * 3 {
            return Err(RomError("dialogue frame dimensions invalid"));
        }
        for value in pixels.iter_mut() {
            *value /= 3;
        }
        for y in 30..222 {
            for x in 4..316 {
                let at = (y * 320 + x) * 3;
                pixels[at..at + 3].fill(if x == 4 || x == 315 || y == 30 || y == 221 {
                    100
                } else {
                    8
                });
            }
        }
        self.draw(rom, pixels, "LOCATION", 12, 38)?;
        let mut lines = Vec::new();
        for paragraph in self.text.split('\n') {
            let mut line = String::new();
            for word in paragraph.split_whitespace() {
                if word.len() > 37 {
                    return Err(RomError("dialogue word exceeds panel width"));
                }
                if !line.is_empty() && line.len() + 1 + word.len() > 37 {
                    lines.push(std::mem::take(&mut line));
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
            lines.push(line);
        }
        if lines.len() > 15 {
            return Err(RomError("exposition needs unimplemented text paging"));
        }
        for (i, line) in lines.iter().enumerate() {
            self.draw(rom, pixels, line, 12, 54 + i * 10)?;
        }
        self.draw(rom, pixels, "ENTER / Z: CONTINUE", 12, 208)
    }
}
