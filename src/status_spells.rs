//! Instant spell initializers, visible-actor status clocks and original tile transforms.
use crate::{dialogue_data::Address, map_render::MapScene, party::Runner, RomError};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Invisibility,
    Rockskin,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub item: u8,
    pub kind: Kind,
    initializer: Address,
    handler: Address,
    timer_routine: Address,
    graphics_routine: Address,
    flag: u8,
    timer: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    duration_factor: u8,
    tick_mask: u16,
    rockskin_colors: Address,
    pub statuses: Vec<Status>,
}
fn word(rom: &[u8], at: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(
        rom.get(at..at + 2)
            .ok_or(RomError("status word outside ROM"))?
            .try_into()
            .unwrap(),
    ))
}
fn long(rom: &[u8], at: usize) -> Result<usize, RomError> {
    Ok(u32::from_be_bytes(
        rom.get(at..at + 4)
            .ok_or(RomError("status pointer outside ROM"))?
            .try_into()
            .unwrap(),
    ) as usize)
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/combat/status-spells.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self =
            serde_json::from_str(json).map_err(|_| RomError("invalid status-spell JSON"))?;
        if c.schema_version != 1
            || c.duration_factor != 8
            || c.tick_mask != 63
            || c.rockskin_colors.0 != 0x3edc
            || c.statuses.len() != 2
        {
            return Err(RomError("status-spell catalog outside verified scope"));
        }
        for (s, expected) in c.statuses.iter().zip([
            (4, Kind::Invisibility, 0x4dca, 0x4e38, 0x3efe, 0x80, 0x40),
            (11, Kind::Rockskin, 0x4dfe, 0x4e64, 0x3e86, 0x20, 0x41),
        ]) {
            if (
                s.item,
                s.kind,
                s.initializer.0,
                s.timer_routine.0,
                s.graphics_routine.0,
                s.flag,
                s.timer,
            ) != expected
                || s.handler.0 != 0x49a0
                || long(rom, 0x4964 + usize::from(s.item) * 4)? != s.initializer.0
                || long(rom, 0x490a + usize::from(s.item) * 4)? != s.handler.0
            {
                return Err(RomError("status-spell reference differs from ROM"));
            }
        }
        rom.get(c.rockskin_colors.0..c.rockskin_colors.0 + 16)
            .ok_or(RomError("status colors outside ROM"))?;
        Ok(c)
    }
    pub fn status(&self, item: u8) -> Option<&Status> {
        self.statuses.iter().find(|s| s.item == item)
    }
    /// $4dca/$4dfe clear B2 immediately, but do not release their graphics slot.
    pub fn initialize(&self, target: &mut Runner, item: u8, successes: u8) -> Result<(), RomError> {
        let s = self
            .status(item)
            .ok_or(RomError("unsupported instant status"))?;
        target.raw[0x11] |= s.flag;
        target.raw[s.timer] = successes.wrapping_mul(self.duration_factor);
        target.raw[0xb2] = 0;
        Ok(())
    }
}

/// $46f8/$4706: status clocks run only for active, visible actors. Invisibility
/// holds for the original $9e bit-6 exception. Zero expires on the next decrement.
pub fn tick(actors: &mut [Runner], phase: u16) {
    if phase & 63 != 0 {
        return;
    }
    for a in actors
        .iter_mut()
        .filter(|a| a.raw[0x10] & 1 != 0 && a.word(0xc) != 0)
    {
        for (flag, timer, held) in [(0x20, 0x41, false), (0x80, 0x40, a.raw[0x9e] & 0x40 != 0)] {
            if a.raw[0x11] & flag == 0 || held {
                continue;
            }
            a.raw[timer] = a.raw[timer].wrapping_sub(1);
            if (a.raw[timer] as i8) < 0 {
                a.raw[0x11] &= !flag;
            }
        }
    }
}

/// $3e86/$3efe tile bytes before the original transfer. The transform uses the
/// first sprite's column-major footprint. Rockskin wins when both flags are set.
pub fn graphics(
    rom: &[u8],
    scene: &MapScene,
    frame: usize,
    a: &Runner,
    camera: (u16, u16),
) -> Result<Vec<u8>, RomError> {
    if word(rom, frame)? != 0x8002 {
        return Err(RomError("status graphics require dynamic frame"));
    }
    let source = long(rom, frame + 2)?;
    let size = usize::from(word(rom, frame + 6)?);
    let original = rom
        .get(source..source + size)
        .ok_or(RomError("status graphics outside ROM"))?;
    if a.raw[0x11] & 0x20 != 0 {
        let colors = rom
            .get(0x3edc..0x3edc + 16)
            .ok_or(RomError("status colors outside ROM"))?;
        return Ok(original
            .iter()
            .map(|b| (colors[usize::from(b >> 4)] << 4) | colors[usize::from(b & 15)])
            .collect());
    }
    if a.raw[0x11] & 0x80 == 0 {
        return Ok(original.to_vec());
    }
    let dimensions = word(rom, frame + 12)?;
    let columns = (dimensions >> 2) + 1;
    let rows = (dimensions & 3) + 1;
    if usize::from(columns) * usize::from(rows) * 32 != size {
        return Err(RomError("invisibility frame footprint outside model"));
    }
    // The original reconstructs these from projected words and camera >> 4,
    // retaining the rounded projection and the actor's vertical displacement.
    let (px, py) = crate::actors::project_origin(
        a.position(),
        (camera.0.wrapping_mul(16), camera.1.wrapping_mul(16)),
        a.word(8) as i16,
    );
    let left = word(rom, frame + 8)?
        .wrapping_add(px)
        .wrapping_sub(128)
        .wrapping_add(camera.0);
    let top = word(rom, frame + 10)?
        .wrapping_add(py)
        .wrapping_sub(128)
        .wrapping_add(camera.1);
    let mut result = Vec::with_capacity(size);
    for x in 0..columns {
        for y in 0..rows {
            let tile = scene.status_background_tile(
                rom,
                left.wrapping_add(x * 8),
                top.wrapping_add(y * 8),
            )?;
            for b in tile {
                let mask = original[result.len()];
                result.push(
                    b & (if mask & 0xf0 != 0 { 0xf0 } else { 0 }
                        | if mask & 15 != 0 { 15 } else { 0 }),
                );
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_catalog_rejects_unknown_fields_and_unverified_versions() {
        assert!(Catalog::parse(
            &Catalog::JSON.replace("\"schema_version\": 1", "\"schema_version\": 2"),
            &[]
        )
        .is_err());
        assert!(Catalog::parse(&Catalog::JSON.replacen('{', "{\"unknown\":1,", 1), &[]).is_err());
    }
}
