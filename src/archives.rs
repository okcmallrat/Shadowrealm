//! In-memory compact runner records ($59798/$597ee/$598b0) and hiring opinion.
//! This is the original record layout, not disk save/load or a quest archive.
use crate::{party::Runner, RomError};

pub const RECORD_SIZE: usize = 74;
const RANGES: [(usize, usize); 10] = [
    (0x0e, 2),
    (0x2c, 2),
    (0x4c, 4),
    (0x56, 2),
    (0x58, 55),
    (0x9a, 3),
    (0x90, 2),
    (0xa0, 2),
    (0x8f, 1),
    (0x9d, 1),
];
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Archives {
    pub records: [[u8; RECORD_SIZE]; 11],
    /// $fffc12..$fffc1b, one signed byte per hireable runner.
    pub opinions: [u8; 10],
}
impl Archives {
    /// Host development-session initialization uses the supplied runner templates,
    /// just as the prior first-hire fixtures did; this is not original new-game boot.
    pub fn load(rom: &[u8], party: &[Runner]) -> Result<Self, RomError> {
        let mut result = Self {
            records: [[0; RECORD_SIZE]; 11],
            opinions: [0; 10],
        };
        result.records[0] = pack(&Runner::load(rom, 0, 0)?);
        for index in 1..11 {
            result.records[index] = pack(&Runner::load(rom, index + 2, 1)?);
        }
        result.snapshot(party)?;
        Ok(result)
    }
    pub fn index(runner: &Runner) -> Result<usize, RomError> {
        match runner.template {
            0..=2 => Ok(0),
            3..=12 => Ok(runner.template - 2),
            _ => Err(RomError("runner outside compact archive")),
        }
    }
    pub fn snapshot(&mut self, party: &[Runner]) -> Result<(), RomError> {
        for runner in party {
            self.records[Self::index(runner)?] = pack(runner);
        }
        Ok(())
    }
    pub fn restore(&self, runner: &mut Runner) -> Result<(), RomError> {
        let record = &self.records[Self::index(runner)?];
        let mut at = 0;
        for (offset, size) in RANGES {
            runner.raw[offset..offset + size].copy_from_slice(&record[at..at + size]);
            at += size;
        }
        Ok(())
    }
    pub fn opinion(&self, runner_index: u8) -> i8 {
        self.opinions[usize::from(runner_index)] as i8
    }
    /// $c8b0: byte addition, then N-test. Negative results become $ff, including
    /// wrapped positive inputs; this deliberately isn't signed saturating arithmetic.
    pub fn penalize(&mut self, archive_index: usize) -> Result<(), RomError> {
        let opinion = self
            .opinions
            .get_mut(archive_index.wrapping_sub(1))
            .ok_or(RomError("companion opinion index outside archive"))?;
        let value = opinion.wrapping_sub(2);
        *opinion = if (value as i8) < 0 { 255 } else { value };
        Ok(())
    }
    /// $c716..$c8a2: zero companion health BEFORE snapshotting, then remove and
    /// penalize every active companion. Caller stages assets before committing.
    pub fn after_defeat(&self, party: &[Runner]) -> Result<Self, RomError> {
        let mut party = party.to_vec();
        for runner in party.iter_mut().skip(1) {
            runner.raw[0x4c] = 0;
        }
        let mut next = self.clone();
        next.snapshot(&party)?;
        for runner in party.iter().skip(1).filter(|r| r.raw[0x10] & 1 != 0) {
            next.penalize(Self::index(runner)?)?;
        }
        Ok(next)
    }
}
pub fn pack(runner: &Runner) -> [u8; RECORD_SIZE] {
    let mut record = [0; RECORD_SIZE];
    let mut at = 0;
    for (offset, size) in RANGES {
        record[at..at + size].copy_from_slice(&runner.raw[offset..offset + size]);
        at += size;
    }
    record
}
