//! Original companion equipment conversion and post-cast drain application.
//! Casting/effect scheduling remains separate; no action is fabricated here.
use crate::{
    party::Runner,
    spellbooks::{self, Selection, Spell},
    weapons, RomError,
};
fn byte(rom: &[u8], at: usize) -> Result<u8, RomError> {
    rom.get(at)
        .copied()
        .ok_or(RomError("equipment table outside ROM"))
}
fn capacity(rom: &[u8], item: u8) -> Result<u8, RomError> {
    if !(1..=12).contains(&item) {
        return Err(RomError("equipment capacity ID outside model"));
    }
    byte(rom, 0xd4540 + usize::from(item - 1) * 48 + 0x24)
}
/// $13b7a/$125fa: D5 is an explicit incoming register, not inferred from $57.
/// The original tests the full packed DIVU result shifted by eight when rounding.
pub fn equip_spell(
    rom: &[u8],
    actor: &mut Runner,
    spell: Spell,
    capacity_index: u16,
) -> Result<(), RomError> {
    if spell.id >= 13 {
        return Err(RomError("equipment spell ID outside model"));
    }
    let mut a = actor.clone();
    if a.raw[0x57] != 0 && (a.raw[0x57] as i8) < 13 {
        let cap = byte(rom, 0xd4540 + usize::from(capacity_index) * 48 + 0x24)?;
        let product = u32::from(a.raw[0x4e]) * 100;
        let (q, r) = if cap == 0 {
            (product, 0)
        } else {
            (product / u32::from(cap), product % u32::from(cap))
        };
        a.raw[0x4e] = q.wrapping_add(u32::from(r != 0 || q >= 256)) as u8;
    }
    a.raw[0x56] = 255;
    a.raw[0x57] = spell.id + 1;
    a.raw[0x9d] = spell.force;
    a.set_word(0xdc, weapons::cooldown(&a, 0));
    *actor = a;
    Ok(())
}
/// $13af2/$13466/$36fc: first inventory entry below 13 (including empty zero).
pub fn equip_inventory(rom: &[u8], actor: &mut Runner) -> Result<(), RomError> {
    let mut a = actor.clone();
    let found = (0..8).find(|slot| a.raw[0x66 + slot] < 13);
    let item = found.map_or(0, |slot| a.raw[0x66 + slot]);
    a.raw[0x57] = item;
    let mut increment = 0u8;
    if item == 0 {
        a.raw[0x9d] = 0;
        // $13478 is an absolute memory read, not an immediate $7f constant.
        a.raw[0x56] = byte(rom, 0x7f)?;
    } else {
        a.raw[0x56] = found.unwrap() as u8;
        if a.raw[0x4e] != 0 {
            let product = u16::from(a.raw[0x4e]) * u16::from(capacity(rom, item)?);
            a.raw[0x4e] = 0;
            a.raw[0x9d] = 0;
            increment = (product / 100 + u16::from(product % 100 >= 50)) as u8;
        }
    }
    if a.raw[0x10] & 1 != 0 && (a.raw[0x56] as i8) >= 0 && a.raw[0x56] != 127 {
        let old = a.raw[0x4e];
        let sum = old.wrapping_add(increment);
        let positive = (old as i8 as i16) + (increment as i8 as i16) > 0;
        a.raw[0x4e] = if positive { sum } else { 0 };
        if positive {
            let cap = capacity(rom, item)?;
            if (cap.wrapping_sub(sum) as i8) < 0 {
                a.raw[0x4e] = cap;
            }
        }
    }
    *actor = a;
    Ok(())
}
pub fn apply_selection(
    rom: &[u8],
    actor: &mut Runner,
    selection: Selection,
    capacity_index: u16,
) -> Result<(), RomError> {
    match selection {
        Selection::KeepEquipped => Ok(()),
        Selection::Spell(s) => equip_spell(rom, actor, s, capacity_index),
        Selection::InventoryWeapon => equip_inventory(rom, actor),
    }
}
pub(crate) fn add_health(a: &mut Runner, amount: u8) {
    let old = a.raw[0x4c];
    let sum = old.wrapping_add(amount);
    a.raw[0x4c] = if a.raw[0x10] & 1 != 0 && (old as i8 as i16) + (amount as i8 as i16) > 0 {
        if (sum as i8) > 10 {
            10
        } else {
            sum
        }
    } else {
        0
    };
}
pub(crate) fn add_condition(a: &mut Runner, amount: u8) {
    if a.raw[0x10] & 1 == 0 {
        a.raw[0x9c] = 0;
        return;
    }
    let old = a.raw[0x9c];
    let result = old.wrapping_add(amount);
    if (old as i8 as i16) + (amount as i8 as i16) > 0 {
        a.raw[0x9c] = if (result as i8) > 10 { 10 } else { result };
    } else {
        add_health(a, result);
        a.raw[0x9c] = 0;
    }
}
/// $10422..$1047c after a successful spell effect. Drain foci are consumed in
/// slots 6..0; condition overflow spills into health through $37ec/$3674.
pub fn apply_drain(rom: &[u8], actor: &mut Runner, spell: Spell) -> Result<(), RomError> {
    if actor.raw[0x10] & 1 == 0 {
        return Err(RomError("post-cast drain requires an active actor"));
    }
    let mut a = actor.clone();
    let drain = spellbooks::cast_drain(rom, &a, spell)?;
    if (drain as i16) > 0 {
        let mut remaining = drain;
        for slot in (0..7).rev() {
            if a.raw[0x66 + slot] != 0x13 {
                continue;
            }
            let charges = a.raw[0x6e + slot];
            if (remaining as u8 as i8) > (charges as i8) {
                remaining =
                    (remaining & 0xff00) | u16::from((remaining as u8).wrapping_sub(charges));
                a.raw[0x66 + slot] = 0;
                a.raw[0x6e + slot] = 0;
            } else {
                let residue = charges.wrapping_sub(remaining as u8);
                a.raw[0x6e + slot] = residue;
                if (charges as i8 as i16) - (remaining as u8 as i8 as i16) <= 0 {
                    a.raw[0x66 + slot] = 0;
                    a.raw[0x6e + slot] = 0;
                }
                *actor = a;
                return Ok(());
            }
        }
        add_condition(&mut a, remaining.wrapping_neg() as u8);
    }
    if a.raw[0x9c] == 0 {
        add_condition(&mut a, 1);
    }
    *actor = a;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inactive_post_cast_actor_is_rejected_without_mutation() {
        let mut actor = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        let before = actor.raw;
        assert!(apply_drain(&[], &mut actor, Spell { id: 0, force: 1 }).is_err());
        assert_eq!(actor.raw, before);
    }
}
