//! Small, embedded JSON pilot. It records verified ROM references and three
//! translated actions, not copied text or a general-purpose quest interpreter.
use crate::RomError;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Address(pub usize);
impl TryFrom<String> for Address {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        usize::from_str_radix(s.strip_prefix("0x").ok_or("address needs 0x prefix")?, 16)
            .map(Self)
            .map_err(|_| "invalid hexadecimal address")
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceRecord {
    pub text: Address,
    pub destination: Address,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeRecord {
    pub address: Address,
    pub text: Address,
    pub choices: Vec<ChoiceRecord>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    RunnerKnown,
    Contacts,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub field: Field,
    pub bit: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRecord {
    pub id: usize,
    pub routine: Address,
    pub next: Address,
    pub effects: Vec<Effect>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conversation {
    pub runner_index: u8,
    pub opinion: i8,
    pub root: Address,
    pub nodes: Vec<NodeRecord>,
    pub actions: Vec<ActionRecord>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub scope: String,
    pub entrance: NodeRecord,
    pub trent: Conversation,
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Knowledge {
    pub runner_known: u16,
    pub contacts: u32,
}
impl Knowledge {
    pub fn apply(&mut self, effect: &Effect) {
        match effect.field {
            Field::RunnerKnown => self.runner_known |= 1u16 << effect.bit,
            Field::Contacts => self.contacts |= 1u32 << effect.bit,
        }
    }
}
pub fn long(rom: &[u8], p: usize) -> Result<usize, RomError> {
    let end = p
        .checked_add(4)
        .ok_or(RomError("dialogue address overflow"))?;
    Ok(u32::from_be_bytes(
        rom.get(p..end)
            .ok_or(RomError("dialogue reference outside ROM"))?
            .try_into()
            .unwrap(),
    ) as usize)
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/dialogue/space-needle.json");
    pub fn parse(json: &str) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid dialogue JSON"))?;
        if c.schema_version != 1
            || c.trent.runner_index != 2
            || c.trent.opinion != 0
            || c.trent.nodes.len() > 128
            || c.scope.is_empty()
        {
            return Err(RomError("unsupported dialogue JSON scope/version"));
        }
        let mut nodes = HashSet::new();
        let mut actions = HashSet::new();
        for n in std::iter::once(&c.entrance).chain(c.trent.nodes.iter()) {
            if n.address.0 < 250 || !nodes.insert(n.address.0) || n.choices.len() > 3 {
                return Err(RomError("invalid/duplicate dialogue node"));
            }
        }
        for action in &c.trent.actions {
            if action.id >= 250 || !actions.insert(action.id) || c.node(action.next.0).is_none() {
                return Err(RomError("invalid dialogue action"));
            }
            for e in &action.effects {
                if e.bit
                    >= match e.field {
                        Field::RunnerKnown => 16,
                        Field::Contacts => 32,
                    }
                {
                    return Err(RomError("dialogue flag bit outside field"));
                }
            }
        }
        if c.node(c.trent.root.0).is_none() {
            return Err(RomError("missing conversation root"));
        }
        for n in &c.trent.nodes {
            for choice in &n.choices {
                if choice.destination.0 >= 250 && c.node(choice.destination.0).is_none() {
                    return Err(RomError("dangling dialogue node reference"));
                }
            }
        }
        Ok(c)
    }
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        let c = Self::parse(Self::JSON)?;
        c.validate(rom)?;
        Ok(c)
    }
    pub fn node(&self, address: usize) -> Option<&NodeRecord> {
        self.trent.nodes.iter().find(|n| n.address.0 == address)
    }
    pub fn action(&self, id: usize) -> Option<&ActionRecord> {
        self.trent.actions.iter().find(|a| a.id == id)
    }
    pub fn can_follow(&self, destination: usize) -> bool {
        if destination < 250 {
            return self.action(destination).is_some();
        }
        self.node(destination).is_some_and(|n| {
            n.choices.is_empty()
                || n.choices
                    .iter()
                    .any(|c| c.destination.0 >= 250 || self.action(c.destination.0).is_some())
        })
    }
    pub fn validate(&self, rom: &[u8]) -> Result<(), RomError> {
        for n in std::iter::once(&self.entrance).chain(self.trent.nodes.iter()) {
            let p = n.address.0;
            if long(rom, p)? != n.text.0 {
                return Err(RomError("JSON dialogue body differs from ROM"));
            }
            if n.choices.is_empty() {
                if long(rom, p + 4)? != u32::MAX as usize {
                    return Err(RomError("JSON exposition differs from ROM"));
                }
            } else {
                for (i, c) in n.choices.iter().enumerate() {
                    if long(rom, p + 4 + i * 4)? != c.text.0
                        || long(rom, p + 16 + i * 4)? != c.destination.0
                    {
                        return Err(RomError("JSON dialogue choice differs from ROM"));
                    }
                }
                if n.choices.len() < 3 && long(rom, p + 4 + n.choices.len() * 4)? != 0 {
                    return Err(RomError("JSON dialogue omits ROM choices"));
                }
            }
        }
        if long(rom, 0xfe640 + usize::from(self.trent.runner_index) * 16)? != self.trent.root.0 {
            return Err(RomError("JSON recruitment root differs from ROM"));
        }
        for a in &self.trent.actions {
            if long(rom, 0x12ccb4 + a.id * 4)? != a.routine.0 {
                return Err(RomError("JSON action source differs from ROM"));
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schema_and_references_fail_closed() {
        let json = Catalog::JSON;
        assert!(
            Catalog::parse(&json.replace("\"schema_version\": 1", "\"schema_version\": 2"))
                .is_err()
        );
        assert!(Catalog::parse(&json.replace("\"bit\": 12", "\"bit\": 32")).is_err());
        assert!(Catalog::parse(&json.replacen("0x0feba0", "0x123456", 1)).is_err());
        let c = Catalog::parse(json).unwrap();
        assert!(c.validate(&[]).is_err());
        assert!(!c.can_follow(3));
        assert!(!c.can_follow(0xfecfc));
        assert!(c.can_follow(190));
        assert!(Catalog::parse(&json.replace("\"contacts\"", "\"untraced_operation\"")).is_err());
    }
}
