//! Source-side weapon timing ($abfc/$d7ea) and ranged animation setup ($b39a).
//! Launch initialization accepts resolved rolls or computes bounded original
//! rolls. Live updates and damage remain separate from these backend routines.
use crate::{party::Runner, RomError};

/// $ad1a..$ad82 fire-request gate. Outer dispatcher state/health permission is
/// separate. A fresh A press during cooldown queues one shot after release.
pub fn controlled_fire_request(
    runner: &Runner,
    signals: &mut LaunchSignals,
    input: crate::input::Controller,
) -> bool {
    if (runner.word(0xdc) as i16) >= 0 {
        if input.pressed & crate::input::A != 0 {
            signals.controlled_busy = true;
        }
        return false;
    }
    if signals.controlled_busy {
        signals.controlled_busy = false;
        return true;
    }
    input.held & crate::input::A != 0
}

/// $aad2/$14226 ordinary firearm reload after a failed launch. The refill is
/// the original +38 clamped by $36fc's capacity rule; one reserve is consumed.
pub fn reload_firearm(rom: &[u8], runner: &mut Runner) -> Result<bool, RomError> {
    if runner.raw[0x4e] != 0 || runner.raw[0x4f] == 0 || (runner.word(0xdc) as i16) >= 0 {
        return Ok(false);
    }
    if !(1..=12).contains(&runner.raw[0x57]) || runner.raw[0x56] >= 8 || runner.raw[0x10] & 1 == 0 {
        return Err(RomError("unsupported firearm reload"));
    }
    let capacity = *rom
        .get(0xd4540 + usize::from(runner.raw[0x57] - 1) * 48 + 0x24)
        .ok_or(RomError("reload capacity outside ROM"))?;
    let mode = if matches!(runner.raw[0x57], 5 | 11 | 12) {
        2
    } else {
        0
    };
    let timer = cooldown(runner, mode);
    runner.set_word(0xdc, timer);
    runner.raw[0x4f] = runner.raw[0x4f].wrapping_sub(1);
    runner.raw[0x4e] = if (capacity.wrapping_sub(38) as i8) < 0 {
        capacity
    } else {
        38
    };
    Ok(true)
}

/// Controlled failure continuation $adf8..$ae4e. A successful reload's timer
/// is overwritten by mode 0 here; a failed reload does not create a timer.
pub fn failed_controlled_launch(
    rom: &[u8],
    runner: &mut Runner,
    direction: Option<u16>,
) -> Result<bool, RomError> {
    let reloaded = reload_firearm(rom, runner)?;
    if reloaded {
        runner.set_word(0xdc, cooldown(runner, 0));
    }
    if reloaded {
        if let Some(angle) = direction {
            runner.set_word(0x34, angle);
            runner.raw[0x3a..0x3e].copy_from_slice(&0xb574u32.to_be_bytes());
            return Ok(true);
        }
        if runner.raw[0x3a..0x3e] == 0xb45cu32.to_be_bytes() {
            return Ok(true);
        }
    }
    runner.raw[0x11] |= 1;
    runner.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    Ok(reloaded)
}

fn word(rom: &[u8], at: usize) -> Result<u16, RomError> {
    let b = rom
        .get(at..at + 2)
        .ok_or(RomError("launch table outside ROM"))?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}
fn put(raw: &mut [u8; 256], at: usize, value: u16) {
    raw[at..at + 2].copy_from_slice(&value.to_be_bytes());
}
fn octant(angle: u16) -> usize {
    let rotated = angle.rotate_left(3);
    usize::from(rotated.wrapping_add(u16::from((rotated as i16) < 0)) & 7)
}

/// Original shared state touched by $1dbaa and the normal $1514e launch path.
#[derive(Debug, Clone, Default)]
pub struct LaunchWorld {
    pub signals: LaunchSignals,
    pub mode: u8,
    pub map_id: u16,
    pub map54_override: u8,
    pub engaged: u16,
    pub timeout: u16,
    pub persistent_event: u32,
    pub environment_mode: u16,
    pub last_ram_byte: u8,
    pub alternate_ammo: u8,
    pub fx_origin: (u16, u16),
}
impl LaunchWorld {
    /// $1de00..$1de3e/$1dbf0 ordinary combat timeout and actor-link cleanup.
    /// The following auto-target/UI continuation remains separate.
    pub fn tick_alert(&mut self, actors: &mut [Runner], script_active: bool) -> bool {
        if self.engaged == 0 || self.mode != 0 || script_active || self.map_id == 20 {
            return false;
        }
        self.timeout = self.timeout.wrapping_sub(1);
        if (self.timeout as i16) >= 0 {
            return false;
        }
        self.end_combat(actors);
        true
    }
    /// Original $1dbf0 link cleanup, also called when the last encounter actor dies.
    pub fn end_combat(&mut self, actors: &mut [Runner]) {
        self.engaged = 0;
        for a in actors {
            a.raw[0xa2..0xa6].fill(0);
            a.raw[0xac..0xb0].fill(0);
            a.raw[0x43] = 0;
            a.raw[0xb0] &= !0x40;
            a.raw[0x11] &= !0x10;
        }
    }
    pub(crate) fn alert(&mut self) {
        if self.map_id == 54 && self.map54_override == 0 {
            return;
        }
        if self.engaged == 0 {
            self.engaged = 0xffff;
        }
        self.timeout = if self.persistent_event != 0 { 600 } else { 120 };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchResult {
    Rejected(LaunchAdmission),
    /// The live projectile survives even though the original returns carry set.
    FxPoolFull {
        projectile: usize,
    },
    Initialized {
        projectile: usize,
        fx: usize,
        sound: u16,
    },
}

/// $14ff8 after the separate $55640 roll. Clear stale projectile bytes, retain
/// source coordinates' integer words only, aim at an active target, and mark
/// that target's attacker. This deliberately requires a supplied roll result;
/// it does not calculate or assume a hit. No target health is changed.
pub fn initialize_projectile(
    rom: &[u8],
    source: &Runner,
    target: Option<&mut Runner>,
    slot: usize,
    resolved_successes: u8,
) -> Result<([u8; 256], (u16, u16)), RomError> {
    if slot >= 6 || source.word(0x3e) >= 10 {
        return Err(RomError("launch actor slot outside modeled pool"));
    }
    let mut target = target;
    let angle = match target.as_deref() {
        Some(t) if t.raw[0x10] & 1 != 0 => {
            crate::followers::heading(rom, source.position(), t.position())?.0
        }
        _ => source.word(0x2e),
    };
    let octant = octant(angle);
    let origin = (
        source
            .position()
            .0
            .wrapping_add(word(rom, 0x15a4c + octant * 4)?),
        source
            .position()
            .1
            .wrapping_add(word(rom, 0x15a4e + octant * 4)?),
    );
    let fx = (
        source
            .position()
            .0
            .wrapping_add(word(rom, 0x15a6c + octant * 4)?),
        source
            .position()
            .1
            .wrapping_add(word(rom, 0x15a6e + octant * 4)?),
    );
    let index = usize::from((angle >> 8) & 0xfe);
    let vx = ((i32::from(word(rom, 0x15e56 + index)? as i16) * 12) >> 4) as u16;
    let vy = ((i32::from(word(rom, 0x15e16 + index)? as i16) * 12) >> 4) as u16;
    let mut raw = [0; 256];
    for (at, value) in [
        (0, origin.0),
        (4, origin.1),
        (8, 0xfff4),
        (0xe, 1),
        (0x16, 0x14),
        (0x1c, 0x1000),
        (0x2c, 0x4000),
        (0x2e, angle),
        (0x3e, 9 + slot as u16),
        (0x4c, vx),
        (0x4e, vy),
    ] {
        put(&mut raw, at, value);
    }
    raw[0x10] = 1 | (source.raw[0x10] & 8);
    raw[0x3a..0x3e].copy_from_slice(&0x14e0au32.to_be_bytes());
    let pointer = 0xff0100u32 + u32::from(source.word(0x3e)) * 256;
    raw[0x60..0x64].copy_from_slice(&pointer.to_be_bytes());
    for at in [0x56, 0x57, 0x9d] {
        raw[at] = source.raw[at];
    }
    raw[0x9f] = resolved_successes;
    if let Some(t) = target.as_mut().filter(|t| t.raw[0x10] & 1 != 0) {
        t.raw[0xac..0xb0].copy_from_slice(&pointer.to_be_bytes());
    }
    Ok((raw, fx))
}

/// Normal-world $1514e launch with an EXPLICIT externally resolved $55640 roll.
/// This backend is not attack permission, a roll implementation or damage.
/// Environment mode 2 has additional alarm/script effects and is unsupported.
/// Modeled-ROM errors leave state unchanged; original gameplay allocation
/// failures preserve ammunition loss, alarm/roll effects and a live projectile.
pub fn launch_with_resolved_roll(
    rom: &[u8],
    source: &mut Runner,
    target: Option<&mut Runner>,
    pool: &mut [[u8; 256]; 6],
    world: &mut LaunchWorld,
    resolved_successes: u8,
) -> Result<LaunchResult, RomError> {
    launch_transaction(rom, source, target, pool, world, |_| Ok(resolved_successes))
}

/// Normal-world launch with the original bounded $55640 roll and shared RNG.
/// Rejections before initialization do not roll; FX allocation failures do.
pub fn launch_ranged(
    rom: &[u8],
    source: &mut Runner,
    target: Option<&mut Runner>,
    pool: &mut [[u8; 256]; 6],
    world: &mut LaunchWorld,
    random: &mut crate::avoidance::State,
) -> Result<(LaunchResult, Option<crate::attack_roll::AttackRoll>), RomError> {
    let mut staged_random = crate::avoidance::State { seed: random.seed };
    let mut roll = None;
    let result = launch_transaction(rom, source, target, pool, world, |source| {
        let value = crate::attack_roll::ranged(source, &mut staged_random)?;
        roll = Some(value);
        Ok(value.result.successes)
    })?;
    random.seed = staged_random.seed;
    Ok((result, roll))
}

fn launch_transaction(
    rom: &[u8],
    source: &mut Runner,
    target: Option<&mut Runner>,
    pool: &mut [[u8; 256]; 6],
    world: &mut LaunchWorld,
    roll: impl FnOnce(&Runner) -> Result<u8, RomError>,
) -> Result<LaunchResult, RomError> {
    if world.environment_mode == 2 {
        return Err(RomError(
            "launch environment mode 2 scripts not implemented",
        ));
    }
    if source.word(0x3e) >= 10 {
        return Err(RomError("launch source outside actor pool"));
    }
    let target_pointer = u32::from_be_bytes(source.raw[0xa2..0xa6].try_into().unwrap());
    match target.as_deref() {
        Some(t)
            if t.word(0x3e) < 10
                && target_pointer == 0xff0100 + u32::from(t.word(0x3e)) * 256
                && t.word(0x3e) != source.word(0x3e) => {}
        None if target_pointer == 0 => {}
        _ => return Err(RomError("launch target does not match modeled actor")),
    }
    // Stage host errors separately from original nontransactional failures.
    let mut staged_source = source.clone();
    let mut staged_target = target.as_deref().cloned();
    let mut staged_pool = *pool;
    let mut staged_world = world.clone();
    let result = launch_inner(
        rom,
        &mut staged_source,
        staged_target.as_mut(),
        &mut staged_pool,
        &mut staged_world,
        roll,
    )?;
    *source = staged_source;
    if let (Some(dst), Some(value)) = (target, staged_target) {
        *dst = value;
    }
    *pool = staged_pool;
    *world = staged_world;
    Ok(result)
}

fn launch_inner(
    rom: &[u8],
    source: &mut Runner,
    target: Option<&mut Runner>,
    pool: &mut [[u8; 256]; 6],
    world: &mut LaunchWorld,
    roll: impl FnOnce(&Runner) -> Result<u8, RomError>,
) -> Result<LaunchResult, RomError> {
    let admission = admit_ranged_prefix(rom, source, world.mode, &mut world.signals, pool)?;
    let projectile = match admission {
        LaunchAdmission::ReadyForInitialization { slot } => slot,
        other => return Ok(LaunchResult::Rejected(other)),
    };
    world.alert();
    let successes = roll(source)?;
    source.raw[0x9f] = successes;
    let (mut raw, fx_origin) = initialize_projectile(rom, source, target, projectile, successes)?;
    world.fx_origin = fx_origin;
    raw[0x89] = if world.signals.burst { 0x20 } else { 0x7f };
    if world.signals.burst {
        raw[0x41] = 2;
        raw[0x8b] = 1;
        let x = u16::from_be_bytes([raw[0], raw[1]]);
        let y = u16::from_be_bytes([raw[4], raw[5]]);
        put(&mut raw, 0x58, x);
        put(&mut raw, 0x5a, y);
    }
    pool[projectile] = raw;
    let Some(fx) = free_projectile(pool) else {
        return Ok(LaunchResult::FxPoolFull { projectile });
    };
    // $15200/$15204 add the offset as a byte before indexed addressing.
    let ammo_at = usize::from(source.raw[0x56].wrapping_add(0x6e));
    let ammo = source.raw[ammo_at];
    world.alternate_ammo = if ammo & 6 != 0 {
        world.last_ram_byte
    } else {
        0
    };
    let sound_index = i32::from(source.raw[0x57].wrapping_sub(1) as i8) * 2;
    let sound_table = if ammo & 6 != 0 { 0x15336i32 } else { 0x15318 };
    let sound = word(rom, (sound_table + sound_index) as usize)?;
    // Original copies only the first 160 bytes: the FX slot's tail persists.
    pool[fx][..160].copy_from_slice(&raw[..160]);
    put(&mut pool[fx], 0, fx_origin.0);
    put(&mut pool[fx], 4, fx_origin.1);
    put(&mut pool[fx], 0x3e, 9 + fx as u16);
    pool[fx][0x11] |= 1;
    put(&mut pool[fx], 0xe, 0);
    pool[fx][0x3a..0x3e].copy_from_slice(&0xac4eu32.to_be_bytes());
    pool[fx][0x64] = world.alternate_ammo;
    pool[fx][0x41] = 0;
    let at = if world.signals.burst { 0x62fc } else { 0x62ec };
    put(
        &mut pool[fx],
        0x16,
        word(
            rom,
            at + octant(u16::from_be_bytes([raw[0x2e], raw[0x2f]])) * 2,
        )?,
    );
    Ok(LaunchResult::Initialized {
        projectile,
        fx,
        sound,
    })
}

/// $38ac with the actor-data portion of $36fc. HUD refresh is the caller's job.
/// A true return is the original admission result, not proof that data changed:
/// inactive actors and absent/negative equipment slots can still return true.
pub fn consume_launch_resource(rom: &[u8], runner: &mut Runner) -> Result<bool, RomError> {
    let item = runner.raw[0x57];
    if runner.raw[0x4e] == 0 || item == 0 {
        return Ok(false);
    }
    if (runner.word(0x3e) as i16) > 2 {
        runner.raw[0x4e] = runner.raw[0x4e].wrapping_sub(1);
        return Ok(true);
    }
    let slot = runner.raw[0x56];
    if runner.raw[0x10] & 1 == 0 || slot >= 0x7f {
        return Ok(true);
    }
    // Malformed item IDs and out-of-inventory addressing are not modeled RAM.
    if item >= 0x80 {
        return Err(RomError("unsupported launch item"));
    }
    let delta: i16 = if (8..=10).contains(&item) { -3 } else { -1 };
    if item < 13 {
        // ADD.B followed by BGT uses N==V and !Z, including signed overflow.
        let sum = i16::from(runner.raw[0x4e] as i8) + delta;
        if sum <= 0 {
            runner.raw[0x4e] = 0;
        } else {
            let at = 0xd4540 + usize::from(item - 1) * 48 + 0x24;
            let capacity = *rom.get(at).ok_or(RomError("weapon capacity outside ROM"))?;
            let quantity = sum as u8;
            // Original CMP.B/BPL tests N alone, not a signed less-than condition.
            runner.raw[0x4e] = if (capacity.wrapping_sub(quantity) as i8) < 0 {
                capacity
            } else {
                quantity
            };
        }
    } else if item < 16 {
        if slot >= 8 {
            return Err(RomError("launch inventory slot outside modeled inventory"));
        }
        let quantity_at = 0x6e + usize::from(slot);
        let sum = i16::from(runner.raw[quantity_at] as i8) + delta;
        if sum > 0 {
            runner.raw[quantity_at] = sum as u8;
        } else {
            runner.raw[quantity_at] = 0;
            runner.raw[0x66 + usize::from(slot)] = 0;
            runner.raw[0x56] = 0x7f;
            runner.raw[0x57] = 0;
        }
    }
    Ok(true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchAdmission {
    ReadyForInitialization { slot: usize },
    ModeBlocked,
    StateBlocked,
    EmptyResource,
    PoolFull,
}

/// Shared original $ffdd3a and sticky $ffdfe6 flags, represented explicitly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LaunchSignals {
    pub burst: bool,
    pub controlled_busy: bool,
}

/// $15a8c searches six entries, in order, without reserving or clearing one.
pub fn free_projectile(pool: &[[u8; 256]; 6]) -> Option<usize> {
    pool.iter().position(|raw| raw[0x10] & 1 == 0)
}

/// $1514e..$151be: mode/state/resource gates and first projectile lookup.
/// A successful prefix does not launch a shot. Cooldown/controller permission,
/// initialization, the second FX allocation and damage remain separate.
/// Allocation failure deliberately does not roll back consumed ammunition.
pub fn admit_ranged_prefix(
    rom: &[u8],
    runner: &mut Runner,
    mode: u8,
    signals: &mut LaunchSignals,
    pool: &[[u8; 256]; 6],
) -> Result<LaunchAdmission, RomError> {
    signals.burst = matches!(runner.raw[0x57], 8..=10);
    if mode == 3 {
        return Ok(LaunchAdmission::ModeBlocked);
    }
    if runner.word(0x16) & 0x7ff > 0x3c {
        if runner.raw[0x10] & 2 != 0 {
            signals.controlled_busy = true;
        }
        return Ok(LaunchAdmission::StateBlocked);
    }
    if !consume_launch_resource(rom, runner)? {
        return Ok(LaunchAdmission::EmptyResource);
    }
    Ok(match free_projectile(pool) {
        Some(slot) => LaunchAdmission::ReadyForInitialization { slot },
        None => LaunchAdmission::PoolFull,
    })
}

/// Preserve byte addition, first-set enhancement priority and signed word shifts.
pub fn cooldown(runner: &Runner, mode: u16) -> u16 {
    let bonus = (12..=14)
        .find(|bit| runner.word(0x90) & (1 << bit) != 0)
        .map_or(0, |bit| (bit - 11) * 4);
    let rating =
        (u16::from(runner.raw[0x78].wrapping_add(runner.raw[0x7b])) / 2 + bonus).saturating_sub(1);
    let base = 106u16.wrapping_sub(rating.wrapping_mul(4));
    let value = match mode {
        0 => base as i16,
        2 => base.wrapping_shl(1) as i16,
        _ => (base as i16) >> 1,
    };
    value.max(15) as u16
}

/// Dispatcher prefix $a8fc..$a912: inactive actors and negative timers hold.
/// Zero becomes $ffff; timer readiness alone does not authorize an attack.
pub fn tick_cooldown(runner: &mut Runner) {
    let value = runner.word(0xdc);
    if runner.raw[0x10] & 1 != 0 && (value as i16) >= 0 {
        runner.set_word(0xdc, value.wrapping_sub(1));
    }
}

/// Source-only setup after a successful original ranged launch. Callers must
/// implement launch admission separately; the native window does not call this yet.
pub fn prepare_ranged(rom: &[u8], runner: &mut Runner, angle: u16) -> Result<(), RomError> {
    let rotated = angle.rotate_left(3);
    let octant = rotated.wrapping_add(u16::from((rotated as i16) < 0)) & 7;
    let at = 0xc042 + usize::from(octant) * 2;
    let state = rom
        .get(at..at + 2)
        .ok_or(RomError("weapon state outside ROM"))?;
    let state = u16::from_be_bytes([state[0], state[1]]);
    let mode = if matches!(runner.raw[0x57], 11 | 12) {
        0
    } else {
        1
    };
    let timer = cooldown(runner, mode);
    runner.set_word(0x2e, angle);
    runner.raw[0x3a..0x3e].copy_from_slice(&0xb3eeu32.to_be_bytes());
    runner.set_word(0x16, state);
    runner.set_word(0xdc, timer);
    runner.set_word(0x1a, 0);
    runner.set_word(0x1e, 0);
    runner.raw[0x11] |= 1;
    runner.clock = Default::default();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_launch_tables_and_unsupported_scripts_leave_state_unchanged() {
        let mut source = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        source.raw[0x10] = 1;
        source.raw[0x57] = 1;
        source.raw[0x4e] = 1;
        let before = source.raw;
        let mut world = LaunchWorld::default();
        let mut pool = [[0x40; 256]; 6];
        let before_pool = pool;
        assert!(
            launch_with_resolved_roll(&[], &mut source, None, &mut pool, &mut world, 3).is_err()
        );
        assert_eq!(source.raw, before);
        assert_eq!(pool, before_pool);
        assert_eq!(world.engaged, 0);
        assert_eq!(world.fx_origin, (0, 0));
        let mut random = crate::avoidance::State { seed: 222 };
        assert!(launch_ranged(&[], &mut source, None, &mut pool, &mut world, &mut random).is_err());
        assert_eq!(
            random.seed, 222,
            "host table errors roll back staged RNG as well as actors"
        );
        assert_eq!(source.raw, before);
        assert_eq!(pool, before_pool);
        world.environment_mode = 2;
        assert!(
            launch_with_resolved_roll(&[], &mut source, None, &mut pool, &mut world, 3).is_err()
        );
        assert_eq!(source.raw, before);
        assert_eq!(pool, before_pool);
        assert_eq!(world.engaged, 0);
    }
    #[test]
    fn unmodeled_resource_reads_fail_without_changing_actor() {
        let mut runner = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        runner.raw[0x10] = 1;
        runner.raw[0x4e] = 20;
        runner.raw[0x57] = 2;
        let before = runner.raw;
        assert!(consume_launch_resource(&[], &mut runner).is_err());
        assert_eq!(runner.raw, before);
        runner.raw[0x57] = 13;
        runner.raw[0x56] = 8;
        let before = runner.raw;
        assert!(consume_launch_resource(&[], &mut runner).is_err());
        assert_eq!(runner.raw, before);
    }
}
