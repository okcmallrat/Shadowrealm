//! Baseline physical projectile damage ($55948), excluding ongoing effects.
use crate::{
    attack_roll::{self, DiceRoll},
    avoidance::State,
    party::Runner,
    RomError,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DamageWorld {
    pub mode: u8,
    pub reward_count: u8,
    pub special_flag: u8,
    pub counter42: u16,
    pub counter13: u8,
    /// Original $f0cc/$f112 are cleared at damage entry on this baseline.
    pub damage_kind: u8,
    pub penetration: i16,
    pub last_successes: u8,
    pub defeat_timer: u16,
}

/// $1ddc6..$1ddda: zero holds; expiry requests the defeat callback once.
/// The original callback performs medical recovery, which is still separate.
pub fn tick_defeat(timer: &mut u16) -> bool {
    if *timer == 0 {
        return false;
    }
    *timer = timer.wrapping_sub(1);
    *timer == 0
}

/// $aff2 reaction setup, including the $abb6 party-defeat check on death.
/// Returns the immediate death sound ID; living hurt sounds are delayed by the
/// original reaction handler and are not played here.
pub fn react(
    rom: &[u8],
    actors: &mut [Runner],
    defender: u8,
    world: &mut DamageWorld,
) -> Result<Option<u16>, RomError> {
    let index = actors
        .iter()
        .position(|r| r.word(0x3e) == u16::from(defender))
        .ok_or(RomError("reaction defender missing"))?;
    if actors[index].raw[0x10] & 0x10 != 0 {
        return Ok(None);
    }
    let dead = actors[index].raw[0x4c] == 0;
    if dead && actors[index].word(0xe) >= 16 {
        return Err(RomError("death reaction animation bank outside model"));
    }
    let angle = actors[index].word(0x2e).rotate_left(3);
    let octant = usize::from(angle.wrapping_add(u16::from((angle as i16) < 0)) & 7);
    let at = (if dead { 0xc0b2 } else { 0xc072 }) + octant * 2;
    let state = u16::from_be_bytes([byte(rom, at)?, byte(rom, at + 1)?]);
    let sound = if dead {
        let at = 0xb05e + usize::from(actors[index].word(0xe)) * 2;
        Some(u16::from_be_bytes([byte(rom, at)?, byte(rom, at + 1)?]))
    } else {
        None
    };
    if dead {
        let mut flags = [0u8; 3];
        for actor in actors.iter() {
            if actor.word(0x3e) < 3 {
                flags[usize::from(actor.word(0x3e))] = actor.raw[0x10];
            }
        }
        if defender < 3 {
            flags[usize::from(defender)] |= 0x40;
        }
        if flags[0] & 0x40 != 0
            && (flags[1] & 1 == 0 || flags[1] & 0x40 != 0)
            && (flags[2] & 1 == 0 || flags[2] & 0x40 != 0)
        {
            world.defeat_timer = 60;
        }
    }
    let actor = &mut actors[index];
    if dead {
        actor.raw[0x10] = (actor.raw[0x10] & !0x40) | 0x10;
    } else {
        actor.raw[0xce] = 8;
    }
    actor.raw[0x3a..0x3e].copy_from_slice(&(if dead { 0xb38au32 } else { 0xb07e }).to_be_bytes());
    actor.set_word(0x16, state);
    actor.set_word(0x1a, 0);
    actor.set_word(0x1e, 0);
    actor.raw[0x11] |= 1;
    actor.clock = Default::default();
    Ok(sound)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageResult {
    pub pool: u16,
    pub difficulty: u16,
    pub resistance: DiceRoll,
    /// Staged amount after reductions, before mode immunity/NPC one-point caps.
    pub amount: u16,
    pub health_before: u8,
    pub health_after: u8,
    pub npc_defeat: bool,
}
fn byte(rom: &[u8], at: usize) -> Result<u8, RomError> {
    rom.get(at)
        .copied()
        .ok_or(RomError("damage table outside ROM"))
}
fn physical(actor: &Runner) -> u8 {
    let mut value = actor.raw[0x78]
        .wrapping_add(actor.raw[0x7b])
        .wrapping_add(actor.raw[0x7c])
        / 2;
    if let Some(slot) = (0..8).find(|slot| actor.raw[0x66 + slot] == 0x28) {
        value = value.wrapping_add(actor.raw[0x6e + slot]);
    }
    value
}

/// Execute $55948 and actor-data changes from the party HUD helper $3674.
/// No rendering, death/reaction handler or tabletop substitute is applied.
/// Host errors leave actors, world and RNG unchanged; original no-damage paths
/// still consume resistance dice. Missing slots represent inactive actors,
/// except defeat rewards require all three party records when they increment.
pub fn resolve(
    rom: &[u8],
    projectile: &[u8; 256],
    actors: &mut [Runner],
    defender: u8,
    world: &mut DamageWorld,
    random: &mut State,
) -> Result<DamageResult, RomError> {
    let mut seen = [false; 10];
    for r in actors.iter() {
        let slot = usize::from(r.word(0x3e));
        if slot >= 10 || seen[slot] {
            return Err(RomError("invalid damage actor pool"));
        }
        seen[slot] = true;
    }
    let index = actors
        .iter()
        .position(|r| r.word(0x3e) == u16::from(defender))
        .ok_or(RomError("damage defender missing"))?;
    if actors[index].raw[0xdf] & 3 != 0 {
        return Err(RomError("ongoing-effect damage modifiers not implemented"));
    }
    if projectile[0x57] >= 0x80 {
        return Err(RomError("unsupported damage item"));
    }
    let mut staged = actors.to_vec();
    let mut staged_world = world.clone();
    let mut staged_random = State { seed: random.seed };
    let result = inner(
        rom,
        projectile,
        &mut staged,
        index,
        &mut staged_world,
        &mut staged_random,
    )?;
    actors.clone_from_slice(&staged);
    *world = staged_world;
    random.seed = staged_random.seed;
    Ok(result)
}
fn staged_target_is_special(a: &Runner) -> bool {
    a.raw[0x9b] == 0x17
}

/// $55b78 baseline direct spell damage, reusing the original $559bc staging.
/// Single-target Flame Dart/Bolt and Mana Zap/Blast in normal world mode.
pub fn resolve_magic(
    rom: &[u8],
    actors: &mut [Runner],
    caster: u8,
    defender: u8,
    w: &mut DamageWorld,
    rng: &mut State,
) -> Result<DamageResult, RomError> {
    let source = actors
        .iter()
        .find(|a| a.word(0x3e) == u16::from(caster))
        .ok_or(RomError("spell caster missing"))?
        .clone();
    let index = actors
        .iter()
        .position(|a| a.word(0x3e) == u16::from(defender))
        .ok_or(RomError("spell defender missing"))?;
    if staged_target_is_special(&actors[index])
        || !matches!(source.raw[0x57], 1 | 2 | 8 | 9)
        || w.mode != 0
        || actors
            .iter()
            .any(|a| a.raw[0x10] & 1 != 0 && a.raw[0xdf] & 3 != 0)
    {
        return Err(RomError("spell damage outside baseline"));
    }
    let mut staged = actors.to_vec();
    let mut world = w.clone();
    let mut random = State { seed: rng.seed };
    let target = &staged[index];
    let mut pool = 0u16;
    let at = 0xd4ea0 + usize::from(source.raw[0x57] - 1) * 48;
    let mut resistance_index = i32::from(source.raw[0x57] - 1) * 48;
    if target.word(0x3e) < 3 {
        for a in staged
            .iter()
            .filter(|a| a.word(0x3e) < 3 && a.raw[0x10] & 1 != 0 && (a.raw[0x9b] as i8) <= 1)
        {
            let magic = crate::casting::magic_attribute(a);
            pool = pool.wrapping_add(u16::from(magic) / 4 + u16::from(magic % 4 >= 3));
            // $1047e saves D5/A0 but not D7. Its final $55f02 focus scan
            // replaces the spell-record offset with a slot index or -1.
            resistance_index = (0..7)
                .rev()
                .find(|slot| a.raw[0x66 + slot] == 0x18)
                .map_or(-1, |slot| slot as i32);
        }
    }
    // $55c08 selects body or willpower from the signed spell-record byte.
    let attribute = if (byte(rom, (0xd4ea0 + resistance_index + 0x27) as usize)? as i8) < 0 {
        0x7c
    } else {
        0x77
    };
    pool = (pool & 0xff00) | u16::from((pool as u8).wrapping_add(target.raw[attribute]));
    let power = byte(rom, at + 0x25)?;
    let base = u16::from(byte(rom, 0x166ad4 + usize::from(power))?.wrapping_neg());
    let mut difficulty = u16::from(source.raw[0x9d]);
    if source.raw[0x57] <= 3 && target.raw[0x9b] == 0x17 {
        pool = 12;
        difficulty = 2;
    }
    world.penetration = 1;
    if source.word(0x3e) >= 3 {
        let cost = byte(rom, at + 0x24)?.max(1);
        let a = staged
            .iter_mut()
            .find(|a| a.word(0x3e) == u16::from(caster))
            .unwrap();
        a.raw[0x9c] = a.raw[0x9c].saturating_sub(cost).max(1);
    }
    let result = staged_damage(
        rom,
        &source.raw,
        &mut staged,
        index,
        &mut world,
        &mut random,
        pool,
        difficulty,
        base,
    )?;
    actors.clone_from_slice(&staged);
    *w = world;
    rng.seed = random.seed;
    Ok(result)
}

fn inner(
    rom: &[u8],
    p: &[u8; 256],
    actors: &mut [Runner],
    index: usize,
    w: &mut DamageWorld,
    rng: &mut State,
) -> Result<DamageResult, RomError> {
    w.damage_kind = 0;
    w.penetration = 0;
    let item_at = (0xd4540i32 + (i32::from(p[0x57]) - 1) * 48) as usize;
    let power = byte(rom, item_at + 0x25)?;
    let difficulty = u16::from(byte(rom, item_at + 0x26)?);
    let staged = u16::from(byte(rom, 0x166ad4 + usize::from(power))?.wrapping_neg());
    let target = &actors[index];
    let physical = u16::from(physical(target));
    let injury = (u32::from(physical) * u32::from(target.raw[0x4d]) * 25 / 100) as u16;
    let base = physical.wrapping_sub(injury);
    let mut pool = (base & 0xff00) | u16::from((base as u8).wrapping_add(target.raw[0x77]));
    if let Some(bit) = (9..=11).find(|bit| target.word(0x90) & (1 << bit) != 0) {
        pool = pool.wrapping_add(bit - 8);
    }
    staged_damage(rom, p, actors, index, w, rng, pool, difficulty, staged)
}
fn staged_damage(
    rom: &[u8],
    p: &[u8; 256],
    actors: &mut [Runner],
    index: usize,
    w: &mut DamageWorld,
    rng: &mut State,
    pool: u16,
    mut difficulty: u16,
    mut staged: u16,
) -> Result<DamageResult, RomError> {
    let target = &actors[index];
    if (difficulty.wrapping_sub(2) as i16) < 0 {
        difficulty = 2;
    }
    let resistance = attack_roll::dice(rng, pool, difficulty);
    w.last_successes = resistance.successes;
    let health_before = target.raw[0x4c];
    let mut result = DamageResult {
        pool,
        difficulty,
        resistance,
        amount: 0,
        health_before,
        health_after: health_before,
        npc_defeat: false,
    };
    let net = p[0x9f].wrapping_sub(resistance.successes);
    if (net as i8) < 0 {
        // ADD.B/BGT uses signed-overflow flags, not just wrapped result sign.
        let sum = i16::from(staged as u8 as i8) + i16::from(net as i8);
        staged = (staged & 0xff00) | u16::from((staged as u8).wrapping_add(net));
        if sum <= 0 {
            return Ok(result);
        }
    } else {
        staged = staged.wrapping_add(u16::from(net));
        if (staged.wrapping_sub(11) as i16) >= 0 {
            staged = 10;
        }
    }
    let armor = target.raw[0x76];
    if w.penetration <= 0 && armor != 0 {
        let armor_value = byte(rom, 0xd4d50 + (usize::from(armor) - 1) * 48 + 0x25)?;
        staged = (staged & 0xff00) | u16::from((staged as u8).wrapping_sub(armor_value));
    }
    if (staged as u8 as i8) <= 0 {
        return Ok(result);
    }
    if p[0x11] & 0x20 != 0 {
        staged = staged.wrapping_sub(2);
    } else if let Some(slot) = (0..8).find(|slot| p[0x66 + slot] == 0x27) {
        staged = staged.wrapping_sub(u16::from(p[0x6e + slot]) / 2);
    }
    if (staged as i16) <= 0 {
        return Ok(result);
    }
    result.amount = staged;
    let target = &mut actors[index];
    if target.word(0x3e) < 3 {
        if w.mode == 3 {
            return Ok(result);
        }
        let sum = i16::from(target.raw[0x4c] as i8) - i16::from(staged as u8 as i8);
        target.raw[0x4c] = if target.raw[0x10] & 1 == 0 || sum <= 0 {
            0
        } else {
            (sum as u8).min(10)
        };
        result.health_after = target.raw[0x4c];
        return Ok(result);
    }
    let delta = if (target.raw[0x9a] as i8) < 0 {
        -1i16
    } else {
        i16::from((staged as u8).wrapping_neg() as i8)
    };
    let sum = i16::from(target.raw[0x4c] as i8) + delta;
    target.raw[0x4c] = if sum > 0 { sum as u8 } else { 0 };
    result.health_after = target.raw[0x4c];
    if sum > 0 {
        return Ok(result);
    }
    result.npc_defeat = true;
    if w.special_flag == 0 && target.raw[0x9b] == 0x42 {
        w.counter42 = w.counter42.wrapping_add(1);
        if (w.counter42.wrapping_sub(21) as i16) >= 0 {
            w.counter42 = 20;
        }
    }
    let special13 = target.raw[0x9b] == 0x13;
    w.reward_count = w.reward_count.wrapping_sub(1);
    if (w.reward_count as i8) < 0 {
        w.reward_count = 15;
        for slot in 0..3 {
            let actor = actors
                .iter_mut()
                .find(|a| a.word(0x3e) == slot)
                .ok_or(RomError("defeat reward requires all party records"))?;
            actor.raw[0x8d] = actor.raw[0x8d].wrapping_add(1);
        }
    }
    if special13 {
        w.counter13 = w.counter13.wrapping_add(1);
        if (w.counter13.wrapping_sub(9) as i8) >= 0 {
            w.counter13 = 8;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_tables_and_ongoing_effects_leave_all_state_unchanged() {
        let mut actor = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        actor.raw[0x4c] = 10;
        let mut actors = vec![actor];
        let before = actors[0].raw;
        let mut projectile = [0; 256];
        projectile[0x57] = 1;
        let mut world = DamageWorld {
            damage_kind: 0xaa,
            penetration: 123,
            ..Default::default()
        };
        let saved = world.clone();
        let mut random = State { seed: 1234 };
        assert!(resolve(&[], &projectile, &mut actors, 0, &mut world, &mut random).is_err());
        assert_eq!(actors[0].raw, before);
        assert_eq!(world, saved);
        assert_eq!(random.seed, 1234);
        actors[0].raw[0xdf] = 1;
        let before = actors[0].raw;
        assert!(resolve(&[], &projectile, &mut actors, 0, &mut world, &mut random).is_err());
        assert_eq!(actors[0].raw, before);
        assert_eq!(world, saved);
        assert_eq!(random.seed, 1234);
    }
}
