//! Baseline uninsured solo recovery. Insurance, mission and companion archives defer.
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
}
impl Record {
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        let r: Self = serde_json::from_str(include_str!("../data/dialogue/clinic.json"))
            .map_err(|_| RomError("invalid clinic JSON"))?;
        if r.schema_version != 1
            || r.scope.is_empty()
            || r.routine.0 != 0xc64c
            || r.node.0 != 0xc25bc
            || r.text.0 != 0xc23d7
            || r.target_map != 2
            || r.spawn.0 != 0x2b24
        {
            return Err(RomError("clinic record outside verified scope"));
        }
        if crate::dialogue_data::long(rom, r.node.0)? != r.text.0
            || crate::dialogue_data::long(rom, r.node.0 + 4)? != 0xffffffff
        {
            return Err(RomError("clinic node differs from ROM"));
        }
        Ok(r)
    }
}
pub struct Recovery {
    pub actor: Runner,
    pub balance: u32,
    pub fee: u32,
}
/// $c726..$c7c0 normal uninsured core, after the solo $c802 archive save.
/// The returned state is staged; callers prepare passage/destination before committing.
pub fn prepare(party: &[Runner], balance: u32, flags: u32, mode: u8) -> Option<Recovery> {
    if party.len() != 1
        || party[0].template != 0
        || party[0].word(0x3e) != 0
        || party[0].raw[0x4c] != 0
        || flags != 0
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
    })
}
