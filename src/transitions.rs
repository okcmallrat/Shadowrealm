//! Exterior seam routing and the bounded map 0 / Space Needle round trip.
//! Other scripted doors remain unsupported.
use crate::RomError;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    Exterior,
    Clinic,
    EnterSpaceNeedle,
    EnterJumpHouse,
    ReturnToExterior,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExteriorReturn {
    pub map_id: usize,
    pub descriptor: [u8; 96],
    pub camera_fixed: (u16, u16),
    pub positions: [(u16, u16); 3],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionPlan {
    pub kind: TransitionKind,
    pub exterior_return: Option<ExteriorReturn>,
    pub source_map: usize,
    pub target_map: usize,
    pub spawn_address: usize,
    pub camera_fixed: (u16, u16),
    pub positions: [(u16, u16); 3],
}
fn word(rom: &[u8], p: usize) -> Result<u16, RomError> {
    let b = rom
        .get(p..p + 2)
        .ok_or(RomError("transition spawn outside ROM"))?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}
impl TransitionPlan {
    /// $2a32/$2a44/$2b78 medical arrival, driven by the bounded JSON reference.
    pub fn clinic(
        rom: &[u8],
        source: usize,
        record: &crate::clinic::Record,
        insured: bool,
    ) -> Result<Self, RomError> {
        let spawn = if insured {
            record.insured.spawn.0
        } else {
            record.spawn.0
        };
        let mut positions = [(0, 0); 3];
        for (i, p) in positions.iter_mut().enumerate() {
            *p = (word(rom, spawn + 4 + i * 4)?, word(rom, spawn + 6 + i * 4)?);
        }
        Ok(Self {
            kind: TransitionKind::Clinic,
            exterior_return: None,
            source_map: source,
            target_map: if insured {
                record.insured.target_map
            } else {
                record.target_map
            },
            spawn_address: spawn,
            camera_fixed: (word(rom, spawn)? & 0xff80, word(rom, spawn + 2)? & 0xff80),
            positions,
        })
    }

    pub fn space_needle(
        rom: &[u8],
        map: &crate::maps::MapDescriptor,
        trigger: u8,
        camera_fixed: (u16, u16),
        player: (u16, u16),
    ) -> Result<Option<Self>, RomError> {
        Self::space_needle_from_contact(
            rom,
            map,
            trigger,
            trigger.rotate_left(3) & 3,
            camera_fixed,
            player,
        )
    }
    /// $bc36 derives return orientation from the first negative footprint sample.
    pub fn space_needle_from_contact(
        rom: &[u8],
        map: &crate::maps::MapDescriptor,
        trigger: u8,
        orientation: u8,
        camera_fixed: (u16, u16),
        player: (u16, u16),
    ) -> Result<Option<Self>, RomError> {
        if map.id != 0 || trigger != 0xc6 {
            return Ok(None);
        }
        Self::enter_location(
            rom,
            map,
            trigger,
            orientation,
            camera_fixed,
            player,
            10,
            TransitionKind::EnterSpaceNeedle,
        )
    }
    /// Verified Redmond (map 2), Jump House door C4, shared bar map 13.
    pub fn jump_house_from_contact(
        rom: &[u8],
        map: &crate::maps::MapDescriptor,
        trigger: u8,
        orientation: u8,
        camera_fixed: (u16, u16),
        player: (u16, u16),
    ) -> Result<Option<Self>, RomError> {
        if map.id != 2 || trigger != 0xc4 {
            return Ok(None);
        }
        Self::enter_location(
            rom,
            map,
            trigger,
            orientation,
            camera_fixed,
            player,
            13,
            TransitionKind::EnterJumpHouse,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn enter_location(
        rom: &[u8],
        map: &crate::maps::MapDescriptor,
        trigger: u8,
        orientation: u8,
        camera_fixed: (u16, u16),
        player: (u16, u16),
        expected: usize,
        kind: TransitionKind,
    ) -> Result<Option<Self>, RomError> {
        let table = u32::from_be_bytes(map.raw[0x4e..0x52].try_into().unwrap()) as usize;
        let target = *rom
            .get(table + usize::from(trigger & 31))
            .ok_or(RomError("location route outside ROM"))? as usize;
        if target != expected {
            return Err(RomError("verified interior target differs from traced ROM"));
        }
        let descriptor = crate::maps::MapDescriptor::read(rom, target)?;
        let mut positions = [(0, 0); 3];
        let mut return_positions = [(0, 0); 3];
        // $bc36 chooses orientation from the first negative footprint code.
        let orientation = usize::from(orientation & 3);
        for slot in 0..3 {
            positions[slot] = (
                word(&descriptor.raw, 0x42 + slot * 4)?,
                word(&descriptor.raw, 0x44 + slot * 4)?,
            );
            let base = 0xbd34 + orientation * 12 + slot * 4;
            return_positions[slot] = (
                player.0.wrapping_add(word(rom, base)?),
                player.1.wrapping_add(word(rom, base + 2)?),
            );
        }
        let mut saved_descriptor = map.raw;
        saved_descriptor[..2].copy_from_slice(&camera_fixed.0.to_be_bytes());
        saved_descriptor[2..4].copy_from_slice(&camera_fixed.1.to_be_bytes());
        let saved = ExteriorReturn {
            map_id: map.id,
            descriptor: saved_descriptor,
            camera_fixed,
            positions: return_positions,
        };
        Ok(Some(Self {
            kind,
            exterior_return: Some(saved),
            source_map: map.id,
            target_map: target,
            spawn_address: descriptor.address,
            camera_fixed: (
                word(&descriptor.raw, 0)? & 0xff80,
                word(&descriptor.raw, 2)? & 0xff80,
            ),
            positions,
        }))
    }
    pub fn leave_interior(map: usize, trigger: u8, saved: ExteriorReturn) -> Option<Self> {
        if !((map == 10 && saved.map_id == 0) || (map == 13 && saved.map_id == 2))
            || trigger != 0x80
        {
            return None;
        }
        Some(Self {
            kind: TransitionKind::ReturnToExterior,
            exterior_return: Some(saved),
            source_map: map,
            target_map: saved.map_id,
            spawn_address: 0,
            camera_fixed: saved.camera_fixed,
            positions: saved.positions,
        })
    }
    /// The $3168 decision searches on low trigger bits, then uses the original
    /// fixed-point camera X to distinguish map 0's west/east exits.
    pub fn exterior(
        rom: &[u8],
        map: usize,
        trigger: u8,
        camera_fixed_x: u16,
    ) -> Result<Option<Self>, RomError> {
        if trigger & 31 != 0x10 || ![0, 3, 5].contains(&map) {
            return Ok(None);
        }
        let (target_map, spawn_address) = match map {
            0 if camera_fixed_x >= 0x1400 => (5, 0x3212),
            0 => (3, 0x3222),
            5 => (0, 0x31f2),
            _ => (0, 0x3202),
        };
        let camera_fixed = (
            word(rom, spawn_address)? & 0xff80,
            word(rom, spawn_address + 2)? & 0xff80,
        );
        let mut positions = [(0, 0); 3];
        for (i, position) in positions.iter_mut().enumerate() {
            *position = (
                word(rom, spawn_address + 4 + i * 4)?,
                word(rom, spawn_address + 6 + i * 4)?,
            );
        }
        Ok(Some(Self {
            kind: TransitionKind::Exterior,
            exterior_return: None,
            source_map: map,
            target_map,
            spawn_address,
            camera_fixed,
            positions,
        }))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionStage {
    FadeOut,
    Warmup,
    FadeIn,
}
#[derive(Debug, Clone, Copy)]
pub struct ActiveTransition {
    pub plan: TransitionPlan,
    pub stage: TransitionStage,
    pub ticks: u8,
    pub target_palette: [u16; 64],
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_boundaries_and_spawn_word_mask() {
        let mut rom = vec![0; 0x3232];
        rom[0x3222..0x3224].copy_from_slice(&0x6cf0u16.to_be_bytes());
        let west = TransitionPlan::exterior(&rom, 0, 0xb0, 0x13ff)
            .unwrap()
            .unwrap();
        assert_eq!(west.target_map, 3);
        assert_eq!(west.camera_fixed.0, 0x6c80);
        assert_eq!(
            TransitionPlan::exterior(&rom, 0, 0xf0, 0x1400)
                .unwrap()
                .unwrap()
                .target_map,
            5
        );
        assert_eq!(
            TransitionPlan::exterior(&rom, 5, 0xb0, 0)
                .unwrap()
                .unwrap()
                .spawn_address,
            0x31f2
        );
        assert_eq!(
            TransitionPlan::exterior(&rom, 3, 0xf0, 0)
                .unwrap()
                .unwrap()
                .spawn_address,
            0x3202
        );
        assert!(TransitionPlan::exterior(&rom, 0, 0xc0, 0)
            .unwrap()
            .is_none());
        assert!(TransitionPlan::exterior(&rom, 10, 0xb0, 0)
            .unwrap()
            .is_none());
    }
    #[test]
    fn incomplete_spawn_data_is_rejected() {
        assert!(TransitionPlan::exterior(&[], 0, 0xb0, 0).is_err());
    }
}
