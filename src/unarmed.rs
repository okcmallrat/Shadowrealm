//! Ordinary human unarmed damage ($55c76), close range and attack ($b1dc).
use crate::{
    attack_roll::{self, DiceRoll},
    avoidance::State,
    damage::DamageWorld,
    dialogue_data::Address,
    party::Runner,
    RomError,
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    scope: String,
    damage: Address,
    controlled: Address,
    target_query: Address,
    attack: Address,
    decision: Address,
    range: Address,
    pose_table: Address,
    armor_table: Address,
    armor_stride: usize,
    impact_armor_offset: usize,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/combat/unarmed.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid unarmed JSON"))?;
        if c.schema_version != 1
            || c.scope.is_empty()
            || [
                c.damage.0,
                c.controlled.0,
                c.target_query.0,
                c.attack.0,
                c.decision.0,
                c.range.0,
                c.pose_table.0,
                c.armor_table.0,
                c.armor_stride,
                c.impact_armor_offset,
            ] != [
                0x55c76, 0xad1a, 0x15c84, 0xb1dc, 0x141a0, 0x15c58, 0xc0a2, 0xd4d50, 48, 38,
            ]
        {
            return Err(RomError("unarmed references outside verified scope"));
        }
        if rom.get(c.damage.0..c.damage.0 + 4) != Some(&[0x61, 0, 2, 0x5e])
            || rom.get(c.controlled.0..c.controlled.0 + 4) != Some(&[0x4a, 0x68, 0, 0xdc])
            || rom.get(c.target_query.0..c.target_query.0 + 4) != Some(&[0x20, 0x28, 0, 0xa2])
            || rom.get(c.attack.0..c.attack.0 + 4) != Some(&[0x30, 0x3c, 0, 1])
            || rom.get(c.pose_table.0..c.pose_table.0 + 16).is_none()
        {
            return Err(RomError("unarmed routines differ from ROM"));
        }
        Ok(c)
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResultData {
    pub attack: Option<DiceRoll>,
    pub defense: Option<DiceRoll>,
    pub resistance: Option<DiceRoll>,
    pub amount: u8,
    pub changed: bool,
}
fn physical(a: &Runner) -> u8 {
    let mut p = a.raw[0x78]
        .wrapping_add(a.raw[0x7b])
        .wrapping_add(a.raw[0x7c])
        / 2;
    if let Some(s) = (0..8).find(|s| a.raw[0x66 + s] == 0x28) {
        p = p.wrapping_add(a.raw[0x6e + s]);
    }
    p
}
fn allocated(a: &Runner, remaining: bool) -> u16 {
    let p = u16::from(physical(a));
    let n = (u32::from(p) * u32::from(a.raw[0x4d]) * 25 / 100) as u16;
    if remaining {
        p.wrapping_sub(n)
    } else {
        n
    }
}
fn difficulty(a: &Runner, b: &Runner) -> u16 {
    let mut n = 4u16;
    if a.word(0x90) & 2 != 0 {
        n -= 1;
    }
    if b.raw[0x9b] == 0x16 {
        n += 2;
    } else if a.word(0x90) & 2 == 0 && ![2, 4].contains(&a.raw[0x9a]) && b.raw[0x11] & 0x80 != 0 {
        n += 4;
    }
    n.max(2)
}
fn low_sub(n: &mut u16, v: u8) -> bool {
    let old = *n as u8;
    *n = (*n & 0xff00) | u16::from(old.wrapping_sub(v));
    i16::from(old as i8) - i16::from(v as i8) > 0
}
/// Staged $55c76. The ordinary $b1dc caller has D5's high byte zero after
/// its real heading/range calculation. Status-effect continuations are guarded.
pub fn resolve(
    rom: &[u8],
    actors: &mut [Runner],
    source: u8,
    target: u8,
    world: &mut DamageWorld,
    rng: &mut State,
) -> Result<ResultData, RomError> {
    let index = |id| {
        actors
            .iter()
            .position(|a| a.word(0x3e) == u16::from(id))
            .ok_or(RomError("unarmed actor missing"))
    };
    let s = index(source)?;
    let t = index(target)?;
    if s == t
        || actors[s].raw[0x57] != 0
        || actors[s].raw[0xdf] & 3 != 0
        || actors[t].raw[0xdf] & 3 != 0
    {
        return Err(RomError("unsupported unarmed damage context"));
    }
    let mut staged = actors.to_vec();
    let mut w = world.clone();
    let mut random = State { seed: rng.seed };
    let before = (staged[t].raw[0x4c], staged[t].raw[0x9c]);
    let result = inner(rom, &mut staged, s, t, &mut w, &mut random)?;
    let mut result = result;
    result.changed = before != (staged[t].raw[0x4c], staged[t].raw[0x9c]);
    actors.clone_from_slice(&staged);
    *world = w;
    rng.seed = random.seed;
    Ok(result)
}
fn roll(random: &mut State, w: &mut DamageWorld, pool: u16, difficulty: u16) -> DiceRoll {
    let r = attack_roll::dice(random, pool, difficulty);
    w.last_successes = r.successes;
    r
}
fn inner(
    rom: &[u8],
    actors: &mut [Runner],
    s: usize,
    t: usize,
    w: &mut DamageWorld,
    rng: &mut State,
) -> Result<ResultData, RomError> {
    let a = actors[s].clone();
    let b = actors[t].clone();
    let mut out = ResultData::default();
    let skill = u16::from(a.raw[0x85]);
    let pool = allocated(&a, false).min(skill).wrapping_add(skill).max(1);
    let attack = roll(rng, w, pool, difficulty(&a, &b));
    out.attack = Some(attack);
    if attack.successes == 0 {
        return Ok(out);
    }
    let mut amount = u16::from(attack.successes) + 3;
    if b.raw[0x76] != 0 {
        let at = 0xd4d50 + usize::from(b.raw[0x76] - 1) * 48 + 38;
        let armor = *rom.get(at).ok_or(RomError("unarmed armor outside ROM"))?;
        amount = (amount & 0xff00) | u16::from((amount as u8).wrapping_sub(armor));
    }
    if (amount as u8 as i8) <= 0 {
        return Ok(out);
    }
    if a.raw[0x11] & 0x20 != 0 {
        amount = amount.wrapping_sub(2);
    } else if let Some(slot) = (0..8).find(|slot| a.raw[0x66 + slot] == 0x27) {
        amount = amount.wrapping_sub(u16::from(a.raw[0x6e + slot]) / 2);
    }
    if (amount as i16) <= 0 {
        return Ok(out);
    }
    // $55740 leaves D7.w holding defender cyberware. $55cea replaces only
    // its low byte with Melee, preserving enhancement bits in the high byte.
    let skill = (b.word(0x90) & 0xff00) | u16::from(b.raw[0x85]);
    let allocated_pool = allocated(&b, false);
    let mut pool = if (skill.wrapping_sub(allocated_pool) as i16) >= 0 {
        allocated_pool
    } else {
        skill
    };
    if b.raw[0x57] == 0 {
        pool = pool.wrapping_add(skill);
    }
    let defense = roll(rng, w, pool.max(1), difficulty(&b, &a));
    out.defense = Some(defense);
    if !low_sub(&mut amount, defense.successes) {
        return Ok(out);
    }
    let power = u16::from(a.raw[0x79])
        + (5..=8)
            .find(|bit| a.word(0x90) & (1 << bit) != 0)
            .map_or(0, |bit| bit - 4);
    let mut pool = allocated(&b, true);
    pool = (pool & 0xff00) | u16::from((pool as u8).wrapping_add(b.raw[0x77]));
    pool = pool.wrapping_add(
        (9..=11)
            .find(|bit| b.word(0x90) & (1 << bit) != 0)
            .map_or(0, |bit| bit - 8),
    );
    let resistance = roll(rng, w, pool, power.max(2));
    out.resistance = Some(resistance);
    if !low_sub(&mut amount, resistance.successes) {
        return Ok(out);
    }
    let lethal = if a.word(0x90) & 8 != 0 {
        true
    } else if a.word(0x90) & 4 != 0 {
        if !low_sub(&mut amount, 2) {
            return Ok(out);
        }
        true
    } else {
        false
    };
    out.amount = amount as u8;
    let delta = (amount as u8).wrapping_neg();
    if b.word(0x3e) < 3 {
        if w.mode == 3 {
            return Ok(out);
        }
        if lethal {
            crate::spell_equipment::add_health(&mut actors[t], delta);
        } else {
            crate::spell_equipment::add_condition(&mut actors[t], delta);
        }
        return Ok(out);
    }
    let delta = if (b.raw[0x9a] as i8) < 0 { 255 } else { delta };
    let field = if lethal { 0x4c } else { 0x9c };
    let sum = actors[t].raw[field].wrapping_add(delta);
    actors[t].raw[field] = if (sum as i8) < 0 { 0 } else { sum };
    if lethal && (sum as i8) >= 0 {
        return Ok(out);
    }
    w.reward_count = w.reward_count.wrapping_sub(1);
    if (w.reward_count as i8) < 0 {
        w.reward_count = 15;
        for slot in 0..3 {
            let a = actors
                .iter_mut()
                .find(|a| a.word(0x3e) == slot)
                .ok_or(RomError("unarmed reward requires party records"))?;
            a.raw[0x8d] = a.raw[0x8d].wrapping_add(1);
        }
    }
    Ok(out)
}
pub fn in_range(rom: &[u8], source: &Runner, target: &Runner) -> Result<Option<u16>, RomError> {
    if target.raw[0x10] & 0x41 != 1 {
        return Ok(None);
    }
    let (angle, range) = crate::followers::heading(rom, source.position(), target.position())?;
    Ok((range < 0x200).then_some(angle))
}
pub fn prepare(rom: &[u8], source: &mut Runner, target: Option<&Runner>) -> Result<(), RomError> {
    if source.bank() >= 12 || source.raw[0x11] & 0x40 != 0 {
        return Err(RomError("special unarmed pose is unavailable"));
    }
    let angle = match target {
        Some(target) => crate::followers::heading(rom, source.position(), target.position())?.0,
        None => source.word(0x34),
    };
    let rotated = angle.rotate_left(3);
    let octant = usize::from(rotated.wrapping_add(u16::from((rotated as i16) < 0)) & 7);
    let at = 0xc0a2 + octant * 2;
    let bytes = rom
        .get(at..at + 2)
        .ok_or(RomError("unarmed pose table missing"))?;
    source.set_word(0xdc, crate::weapons::cooldown(source, 1));
    source.set_word(0x34, angle);
    source.set_word(0x2e, angle);
    source.set_word(0x16, u16::from_be_bytes(bytes.try_into().unwrap()));
    source.raw[0x3a..0x3e].copy_from_slice(&0xb316u32.to_be_bytes());
    source.set_word(0x1a, 0);
    source.set_word(0x1e, 0);
    source.raw[0x11] |= 1;
    source.clock = Default::default();
    Ok(())
}
/// Full ordinary $b1dc attack after a close target has been acquired. Prepare
/// actor, dice, reactions and hit FX together before replacing the live state.
pub fn strike(
    rom: &[u8],
    actors: &mut [Runner],
    source: u8,
    target: u8,
    pool: &mut [[u8; 256]; 6],
    launch: &mut crate::weapons::LaunchWorld,
    damage: &mut DamageWorld,
    rng: &mut State,
) -> Result<ResultData, RomError> {
    let source_actor = actors
        .iter()
        .find(|a| a.word(0x3e) == u16::from(source))
        .ok_or(RomError("unarmed source missing"))?;
    let defender = actors
        .iter()
        .find(|a| a.word(0x3e) == u16::from(target))
        .ok_or(RomError("unarmed target missing"))?;
    if in_range(rom, source_actor, defender)?.is_none() {
        return Err(RomError("unarmed strike outside close range"));
    }
    swing(rom, actors, source, Some(target), pool, launch, damage, rng)
}
/// Full $b1dc, including swings without a target, inactive targets and misses.
/// The caller resolves the original $a2 pointer; no automatic hit is inferred.
/// Every swing consumes the mode-1 cooldown and pose; alert/dice/FX require range.
pub fn swing(
    rom: &[u8],
    actors: &mut [Runner],
    source: u8,
    target: Option<u8>,
    pool: &mut [[u8; 256]; 6],
    launch: &mut crate::weapons::LaunchWorld,
    damage: &mut DamageWorld,
    rng: &mut State,
) -> Result<ResultData, RomError> {
    let mut staged = actors.to_vec();
    let mut fx = *pool;
    let mut world = launch.clone();
    let mut wounds = damage.clone();
    let mut random = State { seed: rng.seed };
    let s = staged
        .iter()
        .position(|a| a.word(0x3e) == u16::from(source))
        .ok_or(RomError("unarmed source missing"))?;
    let t = target
        .map(|target| {
            staged
                .iter()
                .position(|a| a.word(0x3e) == u16::from(target))
                .ok_or(RomError("unarmed target missing"))
        })
        .transpose()?;
    let defender = t.map(|t| staged[t].clone());
    prepare(rom, &mut staged[s], defender.as_ref())?;
    let mut result = ResultData::default();
    if let Some(defender) = &defender {
        if let Some(angle) = in_range(rom, &staged[s], defender)? {
            staged[s].set_word(0x34, angle);
            world.alert();
            result = resolve(
                rom,
                &mut staged,
                source,
                defender.word(0x3e) as u8,
                &mut wounds,
                &mut random,
            )?;
        }
    }
    if result.changed {
        let defender = defender.as_ref().unwrap();
        let target = defender.word(0x3e) as u8;
        crate::damage::react(rom, &mut staged, target, &mut wounds)?;
        if let Some(slot) = crate::weapons::free_projectile(&fx) {
            let raw = &mut fx[slot];
            let put = |raw: &mut [u8; 256], at: usize, n: u16| {
                raw[at..at + 2].copy_from_slice(&n.to_be_bytes())
            };
            put(raw, 0x3e, 9 + slot as u16);
            raw[0x11] |= 1;
            put(raw, 0x16, 4);
            put(raw, 0xe, 1);
            raw[0x3a..0x3e].copy_from_slice(&0xac4eu32.to_be_bytes());
            raw[0x10] = 1;
            put(raw, 0, defender.position().0);
            put(raw, 4, defender.position().1.wrapping_add(0x10));
            put(raw, 0x2c, 0x2000);
        }
    }
    // Validate the new pose/FX references without advancing a simulation clock.
    staged[s].sequence(rom)?;
    if result.changed {
        staged[t.unwrap()].sequence(rom)?;
        crate::actors::Sequence::read(rom, 1, 1)?;
    }
    actors.clone_from_slice(&staged);
    *pool = fx;
    *launch = world;
    *damage = wounds;
    rng.seed = random.seed;
    Ok(result)
}
