//! Bounded exposition and a verified Trent conversation subset. Unsupported
//! actions are unavailable; JSON does not authorize arbitrary script execution.
use crate::{
    dialogue_data::{Catalog, Knowledge, NodeRecord},
    maps::decompress,
    RomError,
};
pub struct Choice {
    pub text_address: usize,
    pub text: String,
    pub destination: usize,
    pub available: bool,
}
pub struct Dialogue {
    pub node: usize,
    pub text_address: usize,
    pub text: String,
    pub font: Vec<u8>,
    pub choices: Vec<Choice>,
    pub selected: usize,
    pub page: usize,
    pub caption: &'static str,
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
        Self::build(rom, node, text_address)
    }
    fn build(rom: &[u8], node: usize, text_address: usize) -> Result<Self, RomError> {
        let text = Self::read_text(rom, text_address)?;
        let (font, _) = decompress(
            rom.get(0xc1064..)
                .ok_or(RomError("dialogue font missing"))?,
        )?;
        Ok(Self {
            node,
            text_address,
            text,
            font,
            choices: vec![],
            selected: 0,
            page: 0,
            caption: "LOCATION",
        })
    }
    pub fn from_record(
        rom: &[u8],
        record: &NodeRecord,
        catalog: &Catalog,
    ) -> Result<Self, RomError> {
        let mut result = Self::build(rom, record.address.0, record.text.0)?;
        result.caption = "TRENT";
        for c in &record.choices {
            result.choices.push(Choice {
                text_address: c.text.0,
                text: Self::read_text(rom, c.text.0)?,
                destination: c.destination.0,
                available: catalog.can_follow(c.destination.0),
            });
        }
        Ok(result)
    }
    /// Prepare the next page before committing any recovered flag side effects.
    /// Unknown actions (including hiring) leave both conversation and state intact.
    pub fn follow(
        &self,
        rom: &[u8],
        catalog: &Catalog,
        slot: usize,
        knowledge: &mut Knowledge,
    ) -> Result<Option<Self>, RomError> {
        let Some(choice) = self.choices.get(slot).filter(|c| c.available) else {
            return Ok(None);
        };
        let action = catalog.action(choice.destination);
        let destination = action.map(|a| a.next.0).unwrap_or(choice.destination);
        let next = Self::from_record(
            rom,
            catalog
                .node(destination)
                .ok_or(RomError("dialogue destination unavailable"))?,
            catalog,
        )?;
        if let Some(action) = action {
            for effect in &action.effects {
                knowledge.apply(effect);
            }
        }
        Ok(Some(next))
    }
    pub fn read_text(rom: &[u8], text_address: usize) -> Result<String, RomError> {
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
        Ok(text)
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
        self.draw(rom, pixels, self.caption, 12, 38)?;
        let lines = Self::wrap(&self.text, 37)?;
        let page_height = if self.choices.is_empty() { 14 } else { 8 };
        let start = self.page.min((lines.len().max(1) - 1) / page_height) * page_height;
        for (i, line) in lines.iter().skip(start).take(page_height).enumerate() {
            self.draw(
                rom,
                pixels,
                line,
                12,
                54 + i * if self.choices.is_empty() { 10 } else { 9 },
            )?;
        }
        let mut y = 140;
        for (i, choice) in self.choices.iter().enumerate() {
            let marker = if self.selected == i { ">" } else { " " };
            let available = if choice.available { "" } else { "*" };
            let text = format!("{marker}{}{available} {}", i + 1, choice.text.trim());
            for line in Self::wrap(&text, 37)? {
                self.draw(rom, pixels, &line, 12, y)?;
                y += 9;
            }
        }
        if lines.len() > page_height {
            self.draw(
                rom,
                pixels,
                &format!(
                    "PAGE {}/{}  LEFT/RIGHT",
                    start / page_height + 1,
                    lines.len().div_ceil(page_height)
                ),
                12,
                if self.choices.is_empty() { 198 } else { 128 },
            )?;
        }
        let hint = if self.choices.is_empty() {
            "ENTER / Z: CONTINUE"
        } else if !self.choices[self.selected].available {
            "* UNAVAILABLE IN THIS BUILD"
        } else {
            "UP/DOWN: CHOOSE  ENTER: SELECT"
        };
        self.draw(rom, pixels, hint, 12, 208)
    }
    pub fn change_page(&mut self, delta: isize) -> Result<(), RomError> {
        let count = Self::wrap(&self.text, 37)?
            .len()
            .max(1)
            .div_ceil(if self.choices.is_empty() { 14 } else { 8 });
        self.page = self.page.saturating_add_signed(delta).min(count - 1);
        Ok(())
    }
    fn wrap(text: &str, width: usize) -> Result<Vec<String>, RomError> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let mut line = String::new();
            for word in paragraph.split_whitespace() {
                if word.len() > width {
                    return Err(RomError("dialogue word exceeds panel width"));
                }
                if !line.is_empty() && line.len() + 1 + word.len() > width {
                    lines.push(std::mem::take(&mut line));
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
            lines.push(line);
        }
        Ok(lines)
    }
}
