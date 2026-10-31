//! Exterior seam routing at $3168 and spawn application at $2b78.
//! Scripted interior doors are a separate path and remain unsupported.
use crate::RomError;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionPlan {
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
