//! Verified scripted visit selector; no substitute for its real spawn outcome.
use crate::{
    avoidance::State,
    dialogue_data::{long, Address},
    RomError,
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    scope: String,
    action: usize,
    routine: Address,
    group: Address,
    prototypes: Vec<Address>,
    normal: Address,
    ambush: Address,
    room_fee_instruction: Address,
    room_fee: u8,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/encounters/stoker.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self =
            serde_json::from_str(json).map_err(|_| RomError("invalid Stoker encounter JSON"))?;
        if c.schema_version != 1
            || c.scope.is_empty()
            || c.action != 233
            || [
                c.routine.0,
                c.group.0,
                c.normal.0,
                c.ambush.0,
                c.room_fee_instruction.0,
            ] != [0x249c4, 0x1d6e2a, 0xf4ab4, 0xf4acc, 0x1f73c]
            || c.room_fee != 20
            || c.prototypes.iter().map(|p| p.0).collect::<Vec<_>>()
                != [0x1d66a4, 0x1d665a, 0x1d665a]
        {
            return Err(RomError("Stoker encounter outside verified scope"));
        }
        if long(rom, 0x12ccb4 + c.action * 4)? != c.routine.0
            || rom.get(c.room_fee_instruction.0..c.room_fee_instruction.0 + 8)
                != Some(&[0x13, 0xfc, 0, 20, 0, 0xff, 0xf1, 0x0d])
        {
            return Err(RomError("Stoker action/fee differs from ROM"));
        }
        for (i, p) in c.prototypes.iter().enumerate() {
            if long(rom, c.group.0 + i * 4)? != p.0 {
                return Err(RomError("Stoker group differs from ROM"));
            }
        }
        if long(rom, c.group.0 + c.prototypes.len() * 4)? != 0 {
            return Err(RomError("Stoker group is incomplete"));
        }
        Ok(c)
    }
    pub fn roll(&self, rng: &mut State) -> bool {
        rng.sample(100) >= 80
    }
    pub fn node(&self, count: usize) -> usize {
        if count == 0 {
            self.normal.0
        } else {
            self.ambush.0
        }
    }
    pub fn fee(&self) -> u8 {
        self.room_fee
    }
}
