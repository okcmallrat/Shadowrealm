//! Fixed window layout from $334c, with explicit party state for dynamic cells.
//! This reconstructs the ROM window; it does not invent names or game statistics.
use crate::{map_render::MapScene, RomError};
fn word(rom: &[u8], p: usize) -> Result<u16, RomError> {
    let b = rom
        .get(p..p.checked_add(2).ok_or(RomError("HUD offset overflow"))?)
        .ok_or(RomError("HUD data outside ROM"))?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}
pub struct Hud {
    cells: Vec<u16>,
}
impl Hud {
    pub fn load(rom: &[u8], scene: &mut MapScene) -> Result<Self, RomError> {
        scene.transfer(rom, 0x6a5be, 0xdac0, 0x4c0)?;
        let mut hud = Self {
            cells: vec![0; 64 * 32],
        };
        // $1232 preserves A1: each call rereads the same source template.
        let mut source = 0x6ace2;
        hud.copy(rom, &mut source, 0xf044, 6, 2, 0x6000)?;
        source = 0x6ace2;
        hud.copy(rom, &mut source, 0xfd44, 6, 2, 0x6000)?;
        for destination in [0xf144, 0xf544, 0xf944] {
            source = 0x6aa7e;
            hud.copy(rom, &mut source, destination, 6, 8, 0x6000)?;
        }
        Ok(hud)
    }
    fn copy(
        &mut self,
        rom: &[u8],
        source: &mut usize,
        destination: usize,
        width: usize,
        height: usize,
        flags: u16,
    ) -> Result<(), RomError> {
        for y in 0..height {
            for x in 0..width {
                let index = (destination - 0xf000) / 2 + y * 64 + x;
                *self
                    .cells
                    .get_mut(index)
                    .ok_or(RomError("HUD layout outside window"))? = word(rom, *source)? | flags;
                *source += 2;
            }
        }
        Ok(())
    }
    /// $3674's health meter table; party slot 0..2, level 0..10.
    /// Frontend supplies levels explicitly; this does not read a live entity.
    pub fn health(&mut self, rom: &[u8], slot: usize, level: u8) -> Result<(), RomError> {
        if slot >= 3 || level > 10 {
            return Err(RomError("invalid HUD health input"));
        }
        let mut source = 0x6ab3e + level as usize * 8;
        self.copy(rom, &mut source, 0xf1c4 + slot * 0x400, 1, 4, 0x6000)
    }
    /// Portrait tile indices are a 4x4 crop of an 8-wide portrait definition.
    /// Mirrors the transfer loop at $3404; no actor-to-portrait mapping is assumed.
    pub fn portrait(
        &mut self,
        rom: &[u8],
        scene: &mut MapScene,
        slot: usize,
        source: usize,
        palette: u16,
    ) -> Result<(), RomError> {
        if slot >= 3 {
            return Err(RomError("invalid portrait slot"));
        }
        let mut tile = 0x694 + slot * 22;
        for y in 0..4 {
            for x in 0..4 {
                let id = word(
                    rom,
                    source
                        .checked_add((y * 8 + x) * 2)
                        .ok_or(RomError("portrait offset overflow"))?,
                )? as usize;
                scene.transfer(rom, 0x6acfa + id * 32, tile * 32, 32)?;
                self.cells[(0xf1c6 - 0xf000) / 2 + slot * 512 + y * 64 + x] =
                    tile as u16 | 0x8000 | palette;
                tile += 1;
            }
        }
        Ok(())
    }
    /// Equipment icon transfer and 3x2 tile layout at $34c8/$35f0.
    /// Source is the six-word icon definition chosen by the entity/item logic.
    pub fn equipment(
        &mut self,
        rom: &[u8],
        scene: &mut MapScene,
        slot: usize,
        source: usize,
        palette: u16,
    ) -> Result<(), RomError> {
        if slot >= 3 {
            return Err(RomError("invalid equipment slot"));
        }
        for index in 0..6 {
            let id = word(
                rom,
                source
                    .checked_add(index * 2)
                    .ok_or(RomError("icon offset overflow"))?,
            )? as usize;
            let tile = 0x6a4 + slot * 22 + index;
            scene.transfer(rom, 0xcf97c + id * 32, tile * 32, 32)?;
            self.cells[(0xf448 - 0xf000) / 2 + slot * 512 + (index / 3) * 64 + index % 3] =
                tile as u16 | 0x8000 | palette;
        }
        Ok(())
    }
    /// Two-cell item resource meter used by $36fc. The caller resolves the item
    /// and clamps its current resource level before selecting the table entry.
    pub fn resource(&mut self, rom: &[u8], slot: usize, level: u8) -> Result<(), RomError> {
        if slot >= 3 || level > 38 {
            return Err(RomError("invalid HUD resource input"));
        }
        let mut source = 0x6ac46 + level as usize * 4;
        self.copy(rom, &mut source, 0xf4c4 + slot * 0x400, 2, 1, 0x6000)
    }
    /// Active-party highlight at $39f2 reuses the selected panel's template.
    pub fn select(&mut self, rom: &[u8], slot: usize) -> Result<(), RomError> {
        if slot >= 3 {
            return Err(RomError("invalid selected party slot"));
        }
        let mut source = 0x6aade;
        self.copy(rom, &mut source, 0xf144 + slot * 0x400, 6, 8, 0x6000)
    }
    /// Secondary condition meter from $37ec, with selected-party palette variant.
    pub fn condition(
        &mut self,
        rom: &[u8],
        slot: usize,
        level: u8,
        selected: bool,
    ) -> Result<(), RomError> {
        if slot >= 3 || level > 10 {
            return Err(RomError("invalid HUD condition level"));
        }
        let mut source = if selected { 0x6abee } else { 0x6ab96 } + level as usize * 8;
        self.copy(rom, &mut source, 0xf3c6 + slot * 0x400, 4, 1, 0x6000)
    }
    pub fn words(&self) -> &[u16] {
        &self.cells
    }
    pub fn attribute(&self, x: usize, y: usize) -> Option<u16> {
        // Map register $833c: F000 window. $9191/$929f: right of x=272.
        if (272..320).contains(&x) && y < 224 {
            Some(self.cells[(y / 8) * 64 + x / 8])
        } else {
            None
        }
    }
}
