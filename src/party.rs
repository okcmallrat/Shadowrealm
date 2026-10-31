//! Persistent party state from the ROM templates used by $42b0/$4350.
use crate::{
    actors::{AnimationClock, Sequence},
    hud::Hud,
    map_render::MapScene,
    RomError,
};
fn range(b: &[u8], p: usize, n: usize) -> Result<&[u8], RomError> {
    b.get(p..p.checked_add(n).ok_or(RomError("party offset overflow"))?)
        .ok_or(RomError("party data outside ROM"))
}
fn word(b: &[u8], p: usize) -> u16 {
    u16::from_be_bytes([b[p], b[p + 1]])
}
fn long(b: &[u8], p: usize) -> u32 {
    u32::from_be_bytes(b[p..p + 4].try_into().unwrap())
}
#[derive(Clone)]
pub struct Runner {
    pub raw: [u8; 256],
    pub template: usize,
    pub clock: AnimationClock,
}
impl Runner {
    pub fn load(rom: &[u8], template: usize, slot: usize) -> Result<Self, RomError> {
        if template >= 13 || slot >= 3 {
            return Err(RomError("invalid runner template or slot"));
        }
        let mut raw: [u8; 256] = range(rom, 0x7291a + template * 256, 256)?
            .try_into()
            .unwrap();
        raw[0x3e..0x40].copy_from_slice(&(slot as u16).to_be_bytes());
        // $42b0/$4350: original party VRAM slots and transfer destinations.
        let base = [1u16, 0x19, 0x31][slot];
        raw[0x20..0x22].copy_from_slice(&base.to_be_bytes());
        raw[0x26..0x28].copy_from_slice(&(base * 32).to_be_bytes());
        Ok(Self {
            raw,
            template,
            clock: AnimationClock::default(),
        })
    }
    pub fn word(&self, p: usize) -> u16 {
        word(&self.raw, p)
    }
    pub fn set_word(&mut self, p: usize, value: u16) {
        self.raw[p..p + 2].copy_from_slice(&value.to_be_bytes());
    }
    pub fn position(&self) -> (u16, u16) {
        (self.word(0), self.word(4))
    }
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.set_word(0, x);
        self.set_word(4, y);
    }
    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.raw[0x58..0x64])
            .trim()
            .to_owned()
    }
    pub fn bank(&self) -> usize {
        self.word(0xe) as usize
    }
    pub fn state(&self) -> usize {
        (self.word(0x16) & 0x7fff) as usize / 4
    }
    pub fn palette(&self) -> u16 {
        self.word(0x2c)
    }
    pub fn health(&self) -> u8 {
        self.raw[0x4c].min(10)
    }
    pub fn sequence(&self, rom: &[u8]) -> Result<Sequence, RomError> {
        Sequence::read(rom, self.bank(), self.state())
    }
    pub fn update_hud(
        &self,
        rom: &[u8],
        scene: &mut MapScene,
        hud: &mut Hud,
        slot: usize,
    ) -> Result<(), RomError> {
        let palette = if self.palette() == 0 {
            0x2000
        } else {
            self.palette()
        };
        let portrait = long(&self.raw, 0x50) as usize + self.word(0x54) as usize;
        hud.portrait(rom, scene, slot, portrait, palette)?;
        hud.health(rom, slot, self.health())?;
        let item = self.raw[0x57];
        let signed_item = self.raw[0x56] as i8;
        let source = if item == 0 {
            match self.word(0x90) & 12 {
                8 | 12 => 0xd1878,
                4 => 0xd186c,
                _ => 0xd1860,
            }
        } else {
            let table = if signed_item < 0 { 0xd4ea0 } else { 0xd4540 };
            let entry = table + (item as usize - 1) * 48;
            long(range(rom, entry + 26, 4)?, 0) as usize
        };
        let icon_palette =
            if item == 0 || (signed_item < 0 && (self.raw[0x9b] as i8) < 2 && item < 6) {
                0x2000
            } else if item < 15 {
                0x6000
            } else {
                0x2000
            };
        hud.equipment(rom, scene, slot, source, icon_palette)?;
        let resource = if item == 0 || signed_item < 0 || signed_item == 127 {
            0
        } else if (13..16).contains(&item) {
            let i = self.raw[0x56] as usize;
            *self
                .raw
                .get(0x6e + i)
                .ok_or(RomError("invalid item resource index"))?
        } else {
            let max = range(rom, 0xd4540 + (item as usize - 1) * 48 + 36, 1)?[0];
            self.raw[0x4e].min(max)
        };
        hud.resource(rom, slot, resource)?;
        Ok(())
    }
    /// Direction change at $b530; signed modular angle difference.
    pub fn turn_toward(&mut self, target: u16) {
        let current = self.word(0x2e);
        let delta = current.wrapping_sub(target);
        let next = if delta == 0 {
            target
        } else if (delta as i16) < 0 {
            if delta >= 0xf800 {
                target
            } else {
                current.wrapping_add(0x800)
            }
        } else if delta < 0x800 {
            target
        } else {
            current.wrapping_sub(0x800)
        };
        self.set_word(0x2e, next);
        self.set_word(0x34, target);
    }
    pub fn select_motion(&mut self, moving: bool) {
        let mut angle = self.word(0x2e).rotate_left(3);
        // Idle $b464 rounds on N; moving $b6cc uses the truncated octant.
        if !moving && (angle as i16) < 0 {
            angle = angle.wrapping_add(1);
        }
        let state = (angle & 7) + if moving { 8 } else { 0 };
        self.set_word(0x16, state * 4);
    }
    pub fn tick_animation(&mut self, rom: &[u8]) -> Result<(), RomError> {
        let seq = self.sequence(rom)?;
        self.clock.tick(
            self.word(0x1c),
            seq.increment(self.word(0xca), self.clock.frame),
            seq.frames.len() as u16,
            seq.terminal,
        )?;
        self.set_word(0x1a, self.clock.phase);
        self.set_word(0x1e, self.clock.frame);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn turning_uses_short_modular_path() {
        let mut r = Runner {
            raw: [0; 256],
            template: 0,
            clock: AnimationClock::default(),
        };
        r.set_word(0x2e, 0xf000);
        r.turn_toward(0x1000);
        assert_eq!(r.word(0x2e), 0xf800);
        r.turn_toward(0x1000);
        assert_eq!(r.word(0x2e), 0);
    }
    #[test]
    fn facing_selects_idle_and_walking_octants() {
        let mut r = Runner {
            raw: [0; 256],
            template: 0,
            clock: AnimationClock::default(),
        };
        for i in 0..8 {
            r.set_word(0x2e, i * 0x2000);
            r.select_motion(false);
            assert_eq!(r.state(), i as usize);
            r.select_motion(true);
            assert_eq!(r.state(), i as usize + 8);
        }
    }
}
