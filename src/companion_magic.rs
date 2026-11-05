//! Verified register residue passed from the prior actor to companion conversion.
use crate::{
    collision::{Collision, Contact},
    dialogue_data::Address,
    party::Runner,
    RomError,
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    dispatcher: Address,
    companion_call: Address,
    companion_scheduler: Address,
    selection: Address,
    conversion: Address,
    collision: Address,
    pixel_shift: u8,
    fixed_handlers: Vec<Address>,
    pixel_handlers: Vec<Address>,
    capacity_table: Address,
    capacity_stride: usize,
    capacity_offset: usize,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/combat/companion-magic.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self =
            serde_json::from_str(json).map_err(|_| RomError("invalid companion-magic JSON"))?;
        if c.schema_version != 1
            || c.dispatcher.0 != 0xaaa0
            || c.companion_call.0 != 0xac8e
            || c.companion_scheduler.0 != 0x138b8
            || c.selection.0 != 0x13a92
            || c.conversion.0 != 0x125fa
            || c.collision.0 != 0xe7c8
            || c.pixel_shift != 4
            || c.fixed_handlers.iter().map(|a| a.0).collect::<Vec<_>>() != [0xb574]
            || c.pixel_handlers.iter().map(|a| a.0).collect::<Vec<_>>()
                != [
                    0xb45c, 0xb3ee, 0xb07e, 0xb38a, 0xafe0, 0xafe2, 0xb0d4, 0xb10c,
                ]
            || c.capacity_table.0 != 0xd4540
            || c.capacity_stride != 48
            || c.capacity_offset != 36
        {
            return Err(RomError("companion-magic context outside verified scope"));
        }
        if rom.get(c.companion_call.0..c.companion_call.0 + 6)
            != Some(&[0x4e, 0xb9, 0, 1, 0x38, 0xb8])
            || rom.get(c.collision.0 + 12..c.collision.0 + 14) != Some(&[0xe8, 0x4d])
            || rom.get(c.conversion.0..c.conversion.0 + 4) != Some(&[0xca, 0xfc, 0, 0x30])
        {
            return Err(RomError("companion-magic caller differs from ROM"));
        }
        Ok(c)
    }
}
/// D5.w survives $aaa0..$aac6 into the next active actor. Walking restores its
/// fixed-point candidate after $b48c; idle/action poses leave $e7c8's pixel X.
/// Trigger scripts, temporary contact handlers and special walking banks defer.
pub fn after_actor(collision: &Collision, actor: &Runner) -> Option<u16> {
    if actor.raw[0x10] & 1 == 0
        || actor.bank() >= 13
        || actor.raw[0x9e] & 0x20 != 0
        || actor.raw[0xb0] & 1 != 0
        || matches!(
            collision.contact(actor.position()).ok()?,
            Contact::Trigger(_)
        )
    {
        return None;
    }
    match crate::combat_runtime::handler(actor) {
        0xb574 => Some(actor.position().0),
        0xb45c | 0xb3ee | 0xb07e | 0xb38a | 0xafe0 | 0xafe2 | 0xb0d4 | 0xb10c => {
            Some(actor.position().0 >> 4)
        }
        _ => None,
    }
}
