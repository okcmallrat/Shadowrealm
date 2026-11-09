//! Normal Joshua medical recovery, including compact companion archives and insurance.
//! Mission-specific callbacks and general clinic services remain outside this subset.
use crate::{dialogue_data::Address, party::Runner, RomError};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub schema_version: u8,
    pub scope: String,
    pub routine: Address,
    pub node: Address,
    pub text: Address,
    pub target_map: usize,
    pub spawn: Address,
    pub insured: InsuredRecord,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsuredRecord {
    pub node: Address,
    pub text: Address,
    pub more_text: Address,
    pub continuation: Address,
    pub continuation_text: Address,
    pub target_map: usize,
    pub spawn: Address,
}
impl Record {
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        let r: Self = serde_json::from_str(include_str!("../data/dialogue/clinic.json"))
            .map_err(|_| RomError("invalid clinic JSON"))?;
        if r.schema_version != 2
            || r.scope.is_empty()
            || r.routine.0 != 0xc64c
            || r.node.0 != 0xc25bc
            || r.text.0 != 0xc23d7
            || r.target_map != 2
            || r.spawn.0 != 0x2b24
            || r.insured.node.0 != 0xc25a0
            || r.insured.text.0 != 0xc22c4
            || r.insured.more_text.0 != 0xfe5bf
            || r.insured.continuation.0 != 0xc25b4
            || r.insured.continuation_text.0 != 0xc238f
            || r.insured.target_map != 0
            || r.insured.spawn.0 != 0x2aea
        {
            return Err(RomError("clinic record outside verified scope"));
        }
        if crate::dialogue_data::long(rom, r.node.0)? != r.text.0
            || crate::dialogue_data::long(rom, r.node.0 + 4)? != 0xffffffff
        {
            return Err(RomError("clinic node differs from ROM"));
        }
        for (at, value) in [
            (r.insured.node.0, r.insured.text.0),
            (r.insured.node.0 + 4, r.insured.more_text.0),
            (r.insured.node.0 + 8, 0),
            (r.insured.node.0 + 12, 0),
            (r.insured.node.0 + 16, r.insured.continuation.0),
            (r.insured.continuation.0, r.insured.continuation_text.0),
            (r.insured.continuation.0 + 4, 0xffffffff),
        ] {
            if crate::dialogue_data::long(rom, at)? != value {
                return Err(RomError("insured clinic node differs from ROM"));
            }
        }
        Ok(r)
    }
}
pub struct Recovery {
    pub actor: Runner,
    pub balance: u32,
    pub fee: u32,
    pub insured: bool,
}
/// $c726..$c7c0 normal core; caller must stage $c716/$c802 companion archives.
/// The returned state is staged; callers prepare passage/destination before committing.
pub fn prepare(party: &[Runner], balance: u32, flags: u32, mode: u8) -> Option<Recovery> {
    if party.is_empty()
        || party.len() > 3
        || party.iter().skip(1).any(|r| !(3..13).contains(&r.template))
        || party[0].template > 2
        || party[0].word(0x3e) != 0
        || (party[0].raw[0x4c] != 0 && party[0].raw[0x9c] != 0)
        || flags & !1 != 0
        || mode != 0
    {
        return None;
    }
    let mut actor = party[0].clone();
    actor.raw[0x10] = (actor.raw[0x10] & !0x50) | 1;
    actor.raw[0x11] &= !0x40;
    actor.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    if (actor.raw[0x4c].wrapping_sub(6) as i8) < 0 {
        actor.raw[0x4c] = 6;
    }
    if (actor.raw[0x9c].wrapping_sub(8) as i8) < 0 {
        actor.raw[0x9c] = 8;
    }
    let fee = (balance >> 3).min(5000);
    Some(Recovery {
        actor,
        balance: balance - fee,
        fee,
        insured: flags & 1 != 0,
    })
}
