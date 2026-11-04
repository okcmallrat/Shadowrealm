//! Original session spellbooks and companion spell-selection decisions.
//! JSON describes verified references; it contains no extracted spellbook payload.
use crate::{avoidance::State, dialogue_data::Address, party::Runner, RomError};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    initialization: Address,
    record_size: usize,
    records: Vec<Address>,
    classification_routine: Address,
    attack_ids: Vec<u8>,
    healing_id: u8,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/combat/spellbooks.json");
    pub fn parse(json: &str) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid spellbook JSON"))?;
        if c.schema_version != 1
            || c.initialization.0 != 0x566a2
            || c.record_size != 34
            || c.classification_routine.0 != 0x13c74
            || c.attack_ids != [0, 1, 7, 8]
            || c.healing_id != 11
            || c.records.iter().map(|a| a.0).collect::<Vec<_>>()
                != [0x166ad8, 0x166afa, 0x166b1c, 0x166b3e, 0x166b60]
        {
            return Err(RomError("spellbook JSON outside verified scope"));
        }
        Ok(c)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spellbooks {
    pub records: [[u8; 34]; 5],
    attack_ids: [u8; 4],
    healing_id: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spell {
    pub id: u8,
    pub force: u8,
}
impl Spellbooks {
    /// $566a2 copies all 170 bytes, including padding and unused slots.
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        let c = Catalog::parse(Catalog::JSON)?;
        let mut records = [[0; 34]; 5];
        for (record, at) in records.iter_mut().zip(c.records) {
            record.copy_from_slice(
                rom.get(at.0..at.0 + 34)
                    .ok_or(RomError("spellbook source outside ROM"))?,
            );
            Self::entries(record)?;
        }
        Ok(Self {
            records,
            attack_ids: c.attack_ids.try_into().unwrap(),
            healing_id: c.healing_id,
        })
    }
    fn entries(record: &[u8; 34]) -> Result<Vec<Spell>, RomError> {
        let last = u16::from_be_bytes([record[0], record[1]]);
        if last >= 16 {
            return Err(RomError("spellbook count outside record"));
        }
        Ok(record[2..2 + (usize::from(last) + 1) * 2]
            .chunks_exact(2)
            .map(|p| Spell {
                id: p[0],
                force: p[1],
            })
            .collect())
    }
    pub fn known(&self, index: u16) -> Result<Vec<Spell>, RomError> {
        Self::entries(
            self.records
                .get(usize::from(index))
                .ok_or(RomError("actor spellbook index outside session"))?,
        )
    }
    /// $13c74 retains attack order/duplicates and the last healing force.
    pub fn candidates(&self, index: u16) -> Result<(Vec<Spell>, Option<Spell>), RomError> {
        let entries = Self::entries(
            self.records
                .get(usize::from(index))
                .ok_or(RomError("actor spellbook index outside session"))?,
        )?;
        let attacks = entries
            .iter()
            .copied()
            .filter(|s| self.attack_ids.contains(&s.id))
            .collect();
        let heal = entries
            .iter()
            .copied()
            .filter(|s| s.id == self.healing_id)
            .last();
        Ok((attacks, heal))
    }
}
fn byte(rom: &[u8], at: usize) -> Result<u8, RomError> {
    rom.get(at)
        .copied()
        .ok_or(RomError("spell drain table outside ROM"))
}
/// $10660, including byte arithmetic, first focus from slots 6..0, and rounding.
pub fn estimated_drain(rom: &[u8], a: &Runner, s: Spell) -> Result<u16, RomError> {
    drain_with_force(rom, a, s, s.force as i8 as i16 as u16)
}
pub(crate) fn cast_drain(rom: &[u8], a: &Runner, s: Spell) -> Result<u16, RomError> {
    drain_with_force(rom, a, s, u16::from(s.force))
}
fn drain_with_force(rom: &[u8], a: &Runner, s: Spell, force: u16) -> Result<u16, RomError> {
    if s.id >= 13 {
        return Err(RomError("spell drain ID outside verified table"));
    }
    let mut magic = a.raw[0x80];
    if let Some(slot) = (0..7).rev().find(|slot| a.raw[0x66 + slot] == 0x18) {
        magic = magic.wrapping_add(a.raw[0x6e + slot]);
    }
    let injury = (u32::from(magic) * u32::from(a.raw[0x4d]) * 25 / 100) as u16;
    let retained = u16::from(magic).wrapping_sub(injury);
    let pool = (retained & 0xff00) | u16::from((retained as u8).wrapping_add(a.raw[0x7c]));
    let mut difficulty = (force >> 1).wrapping_add(force & 1);
    difficulty = (difficulty & 0xff00)
        | u16::from(
            (difficulty as u8).wrapping_add(byte(rom, 0xd4ea0 + usize::from(s.id) * 48 + 0x26)?),
        );
    if (difficulty.wrapping_sub(2) as i16) < 0 {
        difficulty = 2;
    }
    let category = byte(rom, 0xd4ea0 + usize::from(s.id) * 48 + 0x24)?;
    if category >= 4 {
        return Err(RomError("spell drain category outside table"));
    }
    let base = byte(rom, 0x166ad4 + usize::from(category))?.wrapping_neg();
    let difference = 7u16.wrapping_sub(difficulty);
    let percent = if 7i16 > (difficulty as i16) {
        difference.wrapping_mul(20)
    } else {
        10
    };
    let product = u32::from(pool) * u32::from(percent);
    let mut reduced = (product / 100) as u16;
    // DIVU must not overflow its 16-bit quotient.
    if product / 100 > 65535 {
        return Err(RomError("spell drain division outside model"));
    }
    if product % 100 >= 50 {
        reduced = reduced.wrapping_add(1);
    }
    // $106d0 sign-extends the 16-bit quotient before DIVU #2. A negative
    // quotient overflows that unsigned division and leaves D6 unchanged.
    let divided = if (reduced as i16) < 0 {
        reduced
    } else {
        reduced / 2
    };
    Ok(u16::from(base).wrapping_sub(divided))
}
pub fn unsafe_drain(rom: &[u8], a: &Runner, s: Spell) -> Result<bool, RomError> {
    if a.raw[0x9c] == 1 {
        return Ok(true);
    }
    let d = estimated_drain(rom, a, s)?;
    Ok((d as i16) >= 0 && d as u8 >= a.raw[0x9c])
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    KeepEquipped,
    Spell(Spell),
    InventoryWeapon,
}
/// $13c20 checks party health in slot order; unavailable companions are skipped.
pub fn healing_target(party: &[Runner]) -> Option<u8> {
    (0..3u16)
        .find(|slot| {
            party.iter().any(|a| {
                a.word(0x3e) == *slot && (*slot == 0 || a.raw[0x10] & 1 != 0) && a.raw[0x4c] < 4
            })
        })
        .map(|slot| slot as u8)
}
/// $13a92..$13d0a, stopping before equipment mutation. Caller stages source/RNG
/// when the resulting spell action is not yet supported by the native engine.
pub fn select(
    rom: &[u8],
    books: &Spellbooks,
    a: &Runner,
    party: &[Runner],
    rng: &mut State,
) -> Result<(Selection, Option<u8>), RomError> {
    if (a.raw[0x9b] as i8) >= 2 {
        return Ok((Selection::KeepEquipped, None));
    }
    if (a.raw[0x56] as i8) >= 0 && (a.raw[0x9c] as i8) < 5 {
        return Ok((Selection::KeepEquipped, None));
    }
    let (attacks, heal) = books.candidates(a.word(0xa0))?;
    let injured = healing_target(party);
    if (a.raw[0x56] as i8) < 0 {
        let reconsider = match a.raw[0x57] {
            1 | 2 | 8 | 9 => heal.is_none() || injured.is_none(),
            12 => injured.is_some(),
            _ => false,
        };
        if reconsider {
            let current = Spell {
                id: a.raw[0x57].wrapping_sub(1),
                force: a.raw[0x9d],
            };
            return Ok((
                if unsafe_drain(rom, a, current)? {
                    Selection::InventoryWeapon
                } else {
                    Selection::KeepEquipped
                },
                injured,
            ));
        }
    }
    if (a.raw[0x9c] as i8) < 5 {
        return Ok((Selection::KeepEquipped, None));
    }
    if let Some(heal) = heal {
        if !unsafe_drain(rom, a, heal)? && injured.is_some() {
            return Ok((Selection::Spell(heal), injured));
        }
    }
    if attacks.is_empty() {
        return Ok((Selection::KeepEquipped, injured));
    }
    let spell = attacks[usize::from(rng.sample(attacks.len() as u16))];
    Ok((
        if unsafe_drain(rom, a, spell)? {
            Selection::InventoryWeapon
        } else {
            Selection::Spell(spell)
        },
        injured,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_rejects_unknown_fields_versions_and_changed_classification() {
        assert!(Catalog::parse(Catalog::JSON).is_ok());
        for json in [
            Catalog::JSON.replace("\"schema_version\": 1", "\"schema_version\": 2"),
            Catalog::JSON.replace("[0, 1, 7, 8]", "[0, 1, 7, 9]"),
            Catalog::JSON.replacen('{', "{\"invented_rule\":true,", 1),
            Catalog::JSON.replace("0x166ad8", "0x166ad9"),
        ] {
            assert!(Catalog::parse(&json).is_err());
        }
    }
    #[test]
    fn bounded_record_count_uses_the_original_inclusive_last_index() {
        let mut record = [0xff; 34];
        record[..6].copy_from_slice(&[0, 1, 0, 2, 11, 3]);
        assert_eq!(
            Spellbooks::entries(&record).unwrap(),
            vec![Spell { id: 0, force: 2 }, Spell { id: 11, force: 3 }]
        );
        record[..2].copy_from_slice(&16u16.to_be_bytes());
        assert!(Spellbooks::entries(&record).is_err());
    }
}

#[cfg(test)]
mod availability_tests {
    use super::*;
    #[test]
    fn inactive_pool_slots_are_not_injured_party_members() {
        let mut party: Vec<_> = (0..3)
            .map(|slot| {
                let mut a = Runner {
                    raw: [0; 256],
                    template: 0,
                    clock: Default::default(),
                };
                a.set_word(0x3e, slot);
                a
            })
            .collect();
        party[0].raw[0x4c] = 10;
        assert_eq!(healing_target(&party), None);
        party[2].raw[0x10] = 1;
        assert_eq!(healing_target(&party), Some(2));
        party[1].raw[0x10] = 1;
        assert_eq!(healing_target(&party), Some(1));
    }
}
