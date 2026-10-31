//! Native translation of the reset routine's decision at ROM $236..$246.
//! Hardware reads are explicit inputs; this does not emulate the Genesis bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetPath {
    InitializeHardware,
    Resume,
}
/// The original tests a longword at $A10008, then a word at $A1000C.
/// Either nonzero value bypasses initialization and branches to $330.
pub fn reset_path(io_a10008: u32, io_a1000c: u16) -> ResetPath {
    if io_a10008 != 0 || io_a1000c != 0 {
        ResetPath::Resume
    } else {
        ResetPath::InitializeHardware
    }
}
/// The version nibble read at $A10001 controls the TMSS signature write.
/// The write is the big-endian longword "SEGA" to $A14000.
pub fn tmss_signature(hardware_version: u8) -> Option<[u8; 4]> {
    if hardware_version & 0x0f != 0 {
        Some(*b"SEGA")
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reset_decisions_match_branch_conditions() {
        assert_eq!(reset_path(0, 0), ResetPath::InitializeHardware);
        assert_eq!(reset_path(1, 0), ResetPath::Resume);
        assert_eq!(reset_path(0, 1), ResetPath::Resume);
        assert_eq!(reset_path(u32::MAX, u16::MAX), ResetPath::Resume);
    }
    #[test]
    fn tmss_uses_low_nibble_only() {
        for version in 0..=255u8 {
            assert_eq!(tmss_signature(version).is_some(), version & 15 != 0);
        }
        assert_eq!(tmss_signature(1), Some(*b"SEGA"));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionDecision {
    Continue,
    MismatchScreen,
    Halt,
}
/// ROM $330..$36a. The header is matched as bytes, not a numeric region mask.
/// The zero entry in the ROM's J/0/U/E table branches to the infinite loop.
pub fn region_decision(hardware_version: u8, header: &[u8; 16]) -> RegionDecision {
    let region = [b'J', 0, b'U', b'E'][(hardware_version >> 6) as usize];
    if region == 0 {
        RegionDecision::Halt
    } else if header.contains(&region) {
        RegionDecision::Continue
    } else {
        RegionDecision::MismatchScreen
    }
}
#[cfg(test)]
mod region_tests {
    use super::*;
    #[test]
    fn us_rom_checks_all_four_hardware_regions() {
        let header = *b"U               ";
        assert_eq!(region_decision(0, &header), RegionDecision::MismatchScreen);
        assert_eq!(region_decision(0x40, &header), RegionDecision::Halt);
        assert_eq!(region_decision(0x80, &header), RegionDecision::Continue);
        assert_eq!(
            region_decision(0xc0, &header),
            RegionDecision::MismatchScreen
        );
        assert_eq!(region_decision(0xbf, &header), RegionDecision::Continue);
    }
}
