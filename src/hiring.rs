//! First-hire Trent subset of $21226/$21326/$21332/$42e6. This does not implement
//! job completion, dismissal, rehire archives, or companion AI.
use crate::{
    dialogue_data::{long, HiringRecord},
    party::Runner,
    RomError,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quote {
    pub short_term: u32,
    pub permanent: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contract {
    SingleRun,
    Permanent,
}
impl Contract {
    pub fn flag(self) -> u8 {
        match self {
            Self::SingleRun => 0,
            Self::Permanent => 255,
        }
    }
}
pub enum Outcome {
    FullParty,
    InsufficientFunds,
    Hired { member: Box<Runner>, balance: u32 },
}
fn word(rom: &[u8], p: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(
        rom.get(p..p + 2)
            .ok_or(RomError("hiring fee outside ROM"))?
            .try_into()
            .unwrap(),
    ))
}
pub fn initial_balance(rom: &[u8]) -> Result<u32, RomError> {
    if rom.get(0x7fc..0x7fe) != Some(&[0x23, 0xfc]) || long(rom, 0x802)? != 0xfffbfe {
        return Err(RomError(
            "initial currency instruction differs from traced ROM",
        ));
    }
    Ok(long(rom, 0x7fe)? as u32)
}
pub fn quote(
    rom: &[u8],
    record: &HiringRecord,
    leader: &Runner,
    opinion: i8,
) -> Result<Quote, RomError> {
    if !(-1..=4).contains(&opinion) {
        return Err(RomError("unsupported runner opinion"));
    }
    let multiplier = u32::from(
        *rom.get(record.opinion_multipliers.0 + (opinion + 1) as usize)
            .ok_or(RomError("opinion fee multiplier missing"))?,
    );
    let base = u32::from(word(rom, record.fee_record.0)?) * multiplier / 10;
    let rating = leader.raw[0x7a]
        .wrapping_add(leader.raw[0x7a])
        .wrapping_add(leader.raw[0x8b])
        .wrapping_add(leader.raw[0x8c])
        >> 2;
    let discount = u32::from(rating) * 5 * (base / 100);
    Ok(Quote {
        short_term: base.wrapping_sub(discount),
        permanent: (u32::from(word(rom, record.fee_record.0 + 2)?) * multiplier / 10)
            .wrapping_sub(discount * 10),
    })
}
pub fn prepare(
    rom: &[u8],
    party: &[Runner],
    npc: &Runner,
    balance: u32,
    quote: Quote,
    contract: Contract,
) -> Result<Outcome, RomError> {
    if party.len() >= 3 {
        return Ok(Outcome::FullParty);
    }
    if party.is_empty() || party.iter().any(|r| r.template == 5) {
        return Err(RomError("unsupported duplicate/empty recruitment context"));
    }
    let price = match contract {
        Contract::SingleRun => quote.short_term,
        Contract::Permanent => quote.permanent,
    };
    // CMP.L/BLE uses signed comparison; SUB.L wraps. Do not silently clamp ROM arithmetic.
    if (price as i32) > (balance as i32) {
        return Ok(Outcome::InsufficientFunds);
    }
    let mut member = Runner::load(rom, 5, party.len())?;
    member.set_position(npc.position().0, npc.position().1);
    member.raw[0x4c] = 10;
    member.raw[0x9c] = 10;
    member.raw[0x8f] = contract.flag();
    member.set_word(0xca, 30); // existing native exploration animation rate
    Ok(Outcome::Hired {
        member: Box::new(member),
        balance: balance.wrapping_sub(price),
    })
}
pub fn recruit_indices(party: &[Runner]) -> [u8; 2] {
    let mut indices = [255; 2];
    for (slot, runner) in party.iter().skip(1).enumerate().take(2) {
        if runner.template >= 3 && runner.template < 13 {
            indices[slot] = (runner.template - 2) as u8;
        }
    }
    indices
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::AnimationClock;
    #[test]
    fn byte_wrapping_and_unsupported_opinion_are_preserved() {
        let catalog =
            crate::dialogue_data::Catalog::parse(crate::dialogue_data::Catalog::JSON).unwrap();
        let h = &catalog.trent.hiring;
        let mut rom = vec![0; h.opinion_multipliers.0 + 6];
        rom[h.opinion_multipliers.0..].copy_from_slice(&[30, 20, 15, 10, 10, 9]);
        rom[h.fee_record.0..h.fee_record.0 + 4].copy_from_slice(&[0, 150, 5, 220]);
        let mut leader = Runner {
            raw: [0; 256],
            template: 0,
            clock: AnimationClock::default(),
        };
        leader.raw[0x7a] = 1;
        leader.raw[0x8b] = 1;
        leader.raw[0x8c] = 1;
        assert_eq!(
            quote(&rom, h, &leader, 0).unwrap(),
            Quote {
                short_term: 285,
                permanent: 2850
            }
        );
        leader.raw[0x7a] = 128;
        leader.raw[0x8b] = 0;
        leader.raw[0x8c] = 0;
        assert_eq!(quote(&rom, h, &leader, 0).unwrap().short_term, 300);
        assert!(quote(&rom, h, &leader, 5).is_err());
    }
}
