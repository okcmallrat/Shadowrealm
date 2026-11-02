//! Original-routine fixtures for target lists, retention/RNG and close range.
use serde_json::json;
use shadowrealm::{
    avoidance::State,
    collision::{Collision, Contact, Rect},
    combat::{self, ActorId, TargetLists},
    maps::MapDescriptor,
    party::Runner,
};
use std::{error::Error, fs, path::Path};
fn fixture_actors(rom: &[u8], variant: usize) -> Result<Vec<Runner>, Box<dyn Error>> {
    let mut actors = vec![];
    for slot in 0..10usize {
        let mut r = Runner::load(rom, slot % 13, slot % 3)?;
        r.set_word(0x3e, slot as u16);
        r.set_position(4000 + slot as u16 * 17, 5000 + slot as u16 * 29);
        r.raw[0x10] = [0, 1, 5, 9, 13, 0x11, 0x41, 0x51][(variant * 7 + slot * 3) % 8];
        r.set_word(0xc, [0, 1, 128, 0xffff][(variant / 3 + slot) % 4]);
        r.raw[0xb0] = if (variant + slot) % 7 == 0 { 0x80 } else { 0 };
        r.raw[0x9e] = [0, 8, 0x10, 0x20, 0x28][(variant / 7 + slot) % 5];
        if (variant + slot) % 11 == 0 {
            r.raw[0x4c] = 0;
        }
        actors.push(r);
    }
    Ok(actors)
}
fn raw(actors: &[Runner]) -> Vec<u8> {
    actors.iter().flat_map(|r| r.raw).collect()
}
fn ids(list: &[ActorId]) -> Vec<u32> {
    list.iter().map(|id| id.pointer()).collect()
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut controls = vec![];
    let mut failures = vec![];
    let mut reloads = vec![];
    let mut animations = vec![];
    for i in 0..1024usize {
        let mut runner = Runner::load(rom, i % 13, i % 3)?;
        runner.set_word(0xdc, [0, 1, 15, 0x7fff, 0x8000, 0xffff][i / 3 % 6]);
        let input = shadowrealm::input::Controller {
            held: i as u8,
            pressed: (i.wrapping_mul(17) as u8) & (i as u8),
        };
        let pending = i / 5 % 2 != 0;
        let mut signals = shadowrealm::weapons::LaunchSignals {
            controlled_busy: pending,
            ..Default::default()
        };
        let request = shadowrealm::weapons::controlled_fire_request(&runner, &mut signals, input);
        controls.push(json!({"actor":runner.raw.to_vec(),"input":input.packed(),"pending":pending,"after_pending":signals.controlled_busy,"request":request}));
        runner.raw[0x10] = 1;
        runner.raw[0x56] = (i % 8) as u8;
        runner.raw[0x57] = (1 + i % 12) as u8;
        runner.raw[0x4e] = [0, 1, 12][i / 5 % 3];
        runner.raw[0x4f] = [0, 1, 2, 255][i / 7 % 4];
        for j in 0..8 {
            runner.raw[0x6e + j] = (i + j) as u8;
        }
        let before = runner.raw.to_vec();
        let mut failed = runner.clone();
        let reloaded = shadowrealm::weapons::reload_firearm(rom, &mut runner)?;
        let direction = if i % 3 == 0 { Some(0x4000) } else { None };
        let failure_reloaded =
            shadowrealm::weapons::failed_controlled_launch(rom, &mut failed, direction)?;
        failures.push(json!({"before":before.clone(),"after":failed.raw.to_vec(),"direction":direction.unwrap_or(0xffff),"reloaded":failure_reloaded}));
        reloads.push(json!({"before":before,"after":runner.raw.to_vec(),"reloaded":reloaded}));
        runner.raw[0x11] = (i * 13) as u8;
        if i % 3 == 0 {
            runner.set_word(0xe, 1);
            runner.set_word(0x16, [0u16, 4, 0xc, 0x10, 0x14][i / 3 % 5]);
        } else if i % 3 == 1 {
            runner.set_word(0xe, 0);
            runner.set_word(0x16, ((i / 3 % 16) * 4) as u16);
        } else {
            runner.set_word(0x16, if i % 2 == 0 { 0 } else { 0x40 });
        }
        let seq = runner.sequence(rom)?;
        runner.set_word(0x1a, [0, 0xfff, 0x1000, 0x7fff, 0x8000, 0xffff][i / 7 % 6]);
        runner.set_word(
            0x1e,
            [
                0,
                seq.frames.len() as u16 - 1,
                seq.frames.len() as u16,
                seq.frames.len() as u16 + 1,
            ][i / 11 % 4],
        );
        runner.set_word(0x18, [0, 2, 0x80, 0x800, 0x270f, 0xffff][i / 13 % 6]);
        runner.set_word(0x1c, 0x1000);
        let before = runner.raw.to_vec();
        shadowrealm::combat_runtime::tick_animation(rom, &mut runner)?;
        animations.push(json!({"before":before,"after":runner.raw.to_vec()}));
    }
    let mut reactions = vec![];
    for i in 0..512usize {
        let mut actors = fixture_actors(rom, i)?;
        for actor in &mut actors {
            if actor.word(0x3e) < 3 {
                actor.raw[0x10] = [0, 1, 0x40, 0x41][(i / 5 + usize::from(actor.word(0x3e))) % 4];
            }
        }
        let defender = (i % 10) as u8;
        let actor = &mut actors[usize::from(defender)];
        actor.raw[0x4c] = if i % 2 == 0 { 0 } else { 5 };
        actor.raw[0x10] = [0, 1, 9, 0x11, 0x41, 0x51][i / 3 % 6];
        actor.set_word(0xe, (i / 7 % 16) as u16);
        actor.set_word(0x2e, (i as u16).wrapping_mul(997));
        let before = raw(&actors);
        let mut world = shadowrealm::damage::DamageWorld {
            defeat_timer: 17,
            ..Default::default()
        };
        let sound = shadowrealm::damage::react(rom, &mut actors, defender, &mut world)?;
        reactions.push(json!({"before":before,"defender":defender,"after":raw(&actors),"sound":sound,"timer":world.defeat_timer}));
    }
    let mut damages = vec![];
    for i in 0..4096usize {
        use shadowrealm::damage::{self, DamageWorld};
        let mut actors = fixture_actors(rom, i)?;
        let defender = (i % 10) as u8;
        let actor = &mut actors[usize::from(defender)];
        actor.raw[0xdf] = 0;
        actor.raw[0x78] = (i * 37) as u8;
        actor.raw[0x7b] = (i * 53) as u8;
        actor.raw[0x7c] = (i / 7 * 71) as u8;
        actor.raw[0x4d] = [0, 1, 2, 3, 4, 128, 255][i / 10 % 7];
        actor.raw[0x4c] = [0, 1, 2, 3, 5, 8, 10, 127, 128, 255][i / 7 % 10];
        actor.raw[0x77] = (i * 29) as u8;
        actor.raw[0x76] = (i / 3 % 8) as u8;
        actor.raw[0x9a] = [0, 2, 0x80, 0xff][i / 5 % 4];
        actor.raw[0x9b] = [0, 0x42, 0x13][i / 11 % 3];
        actor.set_word(0x90, ((i / 13 % 8) as u16) << 9);
        if i % 3 == 0 {
            actor.raw[0x66 + i % 8] = 0x28;
            actor.raw[0x6e + i % 8] = (i * 7) as u8;
        }
        let mut projectile = [0u8; 256];
        projectile[0x57] = (i % 16) as u8;
        projectile[0x9f] = [0, 1, 2, 3, 10, 127, 128, 255][i / 16 % 8];
        projectile[0x11] = if i % 5 == 0 { 0x20 } else { 0 };
        if i % 7 == 0 {
            projectile[0x66 + i % 8] = 0x27;
            projectile[0x6e + i % 8] = (i * 31) as u8;
        }
        let before = raw(&actors);
        let mut world = DamageWorld {
            mode: if i % 13 == 0 { 3 } else { 0 },
            reward_count: [0, 1, 15, 128, 255][i / 5 % 5],
            special_flag: (i % 2) as u8,
            counter42: [0, 19, 20, 0x7fff, 0xffff][i / 11 % 5],
            counter13: [0, 7, 8, 127, 255][i / 17 % 5],
            damage_kind: 0xaa,
            penetration: 0x1234,
            last_successes: 0xac,
            defeat_timer: 0x1234,
        };
        let before_world = json!({"mode":world.mode,"reward":world.reward_count,"special":world.special_flag,"counter42":world.counter42,"counter13":world.counter13});
        let seed = (i as u16).wrapping_mul(337);
        let mut random = State { seed };
        let result = damage::resolve(
            rom,
            &projectile,
            &mut actors,
            defender,
            &mut world,
            &mut random,
        )?;
        damages.push(json!({"defender":defender,"before":before,"projectile":projectile.to_vec(),"world":before_world,"seed":seed,"after":raw(&actors),"after_seed":random.seed,"after_world":{"reward":world.reward_count,"counter42":world.counter42,"counter13":world.counter13,"kind":world.damage_kind,"penetration":world.penetration,"successes":world.last_successes},"pool":result.pool,"difficulty":result.difficulty,"rolls":result.resistance.rolls,"amount":result.amount,"defeat":result.npc_defeat}));
    }
    let mut dice_cases = vec![];
    for i in 0..1538usize {
        let seed = (i as u16).wrapping_mul(971);
        let pool = if i == 1536 {
            0x8000
        } else if i == 1537 {
            0x7fff
        } else {
            [0, 1, 2, 5, 20, 256, 510, 0x8001, 0xffff][i % 9]
        };
        let difficulty = [0, 1, 2, 4, 6, 7, 12, 32, 0x7fff, 0x8000, 0xffff][i / 9 % 11];
        let mut random = State { seed };
        let result = shadowrealm::attack_roll::dice(&mut random, pool, difficulty);
        dice_cases.push(json!({"seed":seed,"pool":pool,"difficulty":difficulty,"successes":result.successes,"rolls":result.rolls,"after_seed":random.seed}));
    }
    let mut attack_rolls = vec![];
    for i in 0..4096usize {
        let mut source = Runner::load(rom, i % 13, i % 3)?;
        for j in 0..256 {
            source.raw[j] = (i * 37 + j * 53 + i / 16) as u8;
        }
        source.raw[0xdf] &= !3;
        source.raw[0x57] = [0, 1, 2, 7, 8, 9, 10, 11, 12, 13, 15, 16, 127][i % 13];
        source.raw[0xb4] = [0, 1, 2, 3, 0x80, 0xff][i / 13 % 6];
        source.set_word(0x90, (i / 3 % 32) as u16);
        // Include duplicate boosts to verify first-match inventory priority.
        if i % 3 == 0 {
            source.raw[0x66 + i % 8] = 0x28;
            source.raw[0x66 + (i + 3) % 8] = 0x28;
        }
        if i % 5 == 0 {
            source.raw[0x66 + (i + 1) % 8] = 0x16;
        }
        let seed = (i as u16).wrapping_mul(337);
        let mut random = State { seed };
        let value = shadowrealm::attack_roll::ranged(&source, &mut random)?;
        attack_rolls.push(json!({"source":source.raw.to_vec(),"seed":seed,"pool":value.pool,"difficulty":value.difficulty,"successes":value.result.successes,"rolls":value.result.rolls,"after_seed":random.seed}));
    }
    let mut launches = vec![];
    for i in 0..2048usize {
        use shadowrealm::weapons::{LaunchResult, LaunchWorld};
        let mut source = Runner::load(rom, i % 13, i % 3)?;
        let source_slot = [0u16, 1, 2, 3, 9][i % 5];
        source.set_word(0x3e, source_slot);
        source.raw[0x10] = [1, 3, 9, 0xf][i / 5 % 4];
        source.raw[0xdf] &= !3;
        source.set_word(0x16, [0, 0x3c, 0x3d, 0x800][i / 7 % 4]);
        source.raw[0x57] = (1 + i % 15) as u8;
        source.raw[0x56] = [0, 1, 2, 3, 4, 5, 6, 7, 0x7f, 0x80, 0xff][i / 11 % 11];
        source.raw[0x4e] = [0, 1, 2, 3, 4, 32, 127, 255][i / 13 % 8];
        for j in 0..8 {
            source.raw[0x6e + j] = ((i / 17 + j * 7) % 8) as u8;
        }
        source.set_position((i as u16).wrapping_mul(337), (i as u16).wrapping_mul(971));
        source.set_word(0x2e, (i as u16).wrapping_mul(997));
        source.set_word(2, 0x1234);
        source.set_word(6, 0xabcd);
        let target_slot = if source_slot == 1 { 5 } else { 1 };
        let mut target = Runner::load(rom, 3, 1)?;
        target.set_word(0x3e, target_slot);
        target.raw[0x10] = if i % 3 == 0 { 0 } else { 1 };
        target.set_position((i as u16).wrapping_mul(181), (i as u16).wrapping_mul(733));
        let has_target = i % 4 != 0;
        source.raw[0xa2..0xa6].copy_from_slice(
            &(if has_target {
                0xff0100u32 + u32::from(target_slot) * 256
            } else {
                0
            })
            .to_be_bytes(),
        );
        let before_source = source.raw.to_vec();
        let before_target = target.raw.to_vec();
        let mut pool = [[0u8; 256]; 6];
        for (slot, raw) in pool.iter_mut().enumerate() {
            for (j, b) in raw.iter_mut().enumerate() {
                *b = (i + slot * 17 + j * 23) as u8;
            }
            raw[0x10] = 0x40 | (((i / 3 % 64) >> slot) as u8 & 1);
        }
        let before_pool: Vec<u8> = pool.iter().flatten().copied().collect();
        let mut world = LaunchWorld {
            mode: if i % 13 == 0 { 3 } else { 0 },
            map_id: if i % 5 == 0 { 54 } else { 0 },
            map54_override: (i % 2) as u8,
            engaged: if i % 3 == 0 { 0x1234 } else { 0 },
            timeout: 17,
            persistent_event: if i % 7 == 0 { 1 } else { 0 },
            environment_mode: [0, 1, 3][i % 3],
            last_ram_byte: (i * 29) as u8,
            alternate_ammo: 0x42,
            fx_origin: (0x1234, 0x5678),
            ..Default::default()
        };
        world.signals.controlled_busy = i % 2 != 0;
        let before_world = json!({"mode":world.mode,"map_id":world.map_id,"override":world.map54_override,"engaged":world.engaged,"timeout":world.timeout,"event":world.persistent_event,"environment":world.environment_mode,"last":world.last_ram_byte,"alternate":world.alternate_ammo,"origin":world.fx_origin,"busy":world.signals.controlled_busy});
        let successes = [0, 1, 3, 127, 255][i / 3 % 5];
        let mut rolled_source = source.clone();
        let mut rolled_target = target.clone();
        let mut rolled_pool = pool;
        let mut rolled_world = world.clone();
        let seed = (i as u16).wrapping_mul(971);
        let mut random = State { seed };
        let (rolled_result, roll) = shadowrealm::weapons::launch_ranged(
            rom,
            &mut rolled_source,
            if has_target {
                Some(&mut rolled_target)
            } else {
                None
            },
            &mut rolled_pool,
            &mut rolled_world,
            &mut random,
        )?;
        let rolled = json!({"after_source":rolled_source.raw.to_vec(),"after_target":rolled_target.raw.to_vec(),"after_pool":rolled_pool.iter().flatten().copied().collect::<Vec<_>>(),"after_seed":random.seed,"rolls":roll.map_or(0,|r| r.result.rolls),"successes":roll.map(|r|r.result.successes)});
        let result = shadowrealm::weapons::launch_with_resolved_roll(
            rom,
            &mut source,
            if has_target { Some(&mut target) } else { None },
            &mut pool,
            &mut world,
            successes,
        )?;
        let (outcome, projectile, fx, sound) = match result {
            LaunchResult::Rejected(_) => (0, None, None, None),
            LaunchResult::FxPoolFull { projectile } => (1, Some(projectile), None, None),
            LaunchResult::Initialized {
                projectile,
                fx,
                sound,
            } => (2, Some(projectile), Some(fx), Some(sound)),
        };
        assert_eq!(rolled_result, result);
        assert_eq!(rolled_world.engaged, world.engaged);
        assert_eq!(rolled_world.timeout, world.timeout);
        assert_eq!(rolled_world.signals, world.signals);
        assert_eq!(rolled_world.fx_origin, world.fx_origin);
        assert_eq!(rolled_world.alternate_ammo, world.alternate_ammo);
        launches.push(json!({"rolled":rolled,"seed":seed,"source_slot":source_slot,"target_slot":target_slot,"has_target":has_target,"source":before_source,"target":before_target,"pool":before_pool,"world":before_world,"successes":successes,"after_source":source.raw.to_vec(),"after_target":target.raw.to_vec(),"after_pool":pool.iter().flatten().copied().collect::<Vec<_>>(),"after_world":{"engaged":world.engaged,"timeout":world.timeout,"alternate":world.alternate_ammo,"origin":world.fx_origin,"busy":world.signals.controlled_busy,"burst":world.signals.burst},"outcome":outcome,"projectile":projectile,"fx":fx,"sound":sound}));
    }
    let mut resources = vec![];
    for i in 0..18432usize {
        let mut r = Runner::load(rom, i % 13, i % 3)?;
        r.raw[0x57] = (i / 256 % 18) as u8;
        r.raw[0x4e] = i as u8;
        r.raw[0x10] = if i % 7 == 0 { 0 } else { 0xd };
        r.raw[0x56] = [0, 1, 2, 3, 4, 5, 6, 7, 0x7f, 0x80, 0xff][i / 3 % 11];
        r.set_word(0x3e, [0, 1, 2, 3, 9, 0xffff, 0x8000, 0x7fff][i / 11 % 8]);
        for j in 0..8 {
            r.raw[0x66 + j] = (j + 13) as u8;
            r.raw[0x6e + j] = (i / 18 + j * 53) as u8;
        }
        let before = r.raw.to_vec();
        let admitted = shadowrealm::weapons::consume_launch_resource(rom, &mut r)?;
        resources.push(json!({"before":before,"after":r.raw.to_vec(),"admitted":admitted}));
    }
    let mut admissions = vec![];
    for i in 0..6144usize {
        let mut r = Runner::load(rom, i % 13, i % 3)?;
        r.raw[0x57] = (i % 18) as u8;
        r.raw[0x4e] = [0, 1, 2, 3, 4, 127, 128, 255][i / 18 % 8];
        r.raw[0x56] = (i / 7 % 8) as u8;
        r.raw[0x10] = [0, 1, 3, 0xd, 0xf][i / 11 % 5];
        r.set_word(0x3e, (i / 13 % 5) as u16);
        r.set_word(
            0x16,
            [0, 0x3c, 0x3d, 0x800, 0x83c, 0x83d, 0xffff][i / 3 % 7],
        );
        let mut pool = [[0u8; 256]; 6];
        let mask = (i / 5 % 64) as u8;
        for (slot, raw) in pool.iter_mut().enumerate() {
            raw[0x10] = 0x40 | ((mask >> slot) & 1);
        }
        let before = r.raw.to_vec();
        let mode = [0, 1, 2, 3, 4, 255][i / 17 % 6];
        let initial_busy = i % 2 != 0;
        let mut signals = shadowrealm::weapons::LaunchSignals {
            burst: !initial_busy,
            controlled_busy: initial_busy,
        };
        let result =
            shadowrealm::weapons::admit_ranged_prefix(rom, &mut r, mode, &mut signals, &pool)?;
        use shadowrealm::weapons::LaunchAdmission::*;
        let (outcome, slot) = match result {
            ReadyForInitialization { slot } => (0, Some(slot)),
            ModeBlocked => (1, None),
            StateBlocked => (2, None),
            EmptyResource => (3, None),
            PoolFull => (4, None),
        };
        admissions.push(json!({"before":before,"after":r.raw.to_vec(),"mode":mode,"mask":mask,"initial_busy":initial_busy,"burst":signals.burst,"busy":signals.controlled_busy,"outcome":outcome,"slot":slot}));
    }
    let mut weapon_timing = vec![];
    for i in 0..4096usize {
        let mut r = Runner::load(rom, i % 13, i % 3)?;
        r.raw[0x78] = (i * 37) as u8;
        r.raw[0x7b] = (i / 16 * 53) as u8;
        r.set_word(0x90, ((i % 16) as u16) << 11);
        r.raw[0x57] = (i % 17) as u8;
        r.raw[0x10] = (i % 16) as u8;
        r.set_word(0xdc, [0, 1, 15, 32767, 32768, 65535, 65534][i % 7]);
        r.set_word(0x1a, 123);
        r.set_word(0x1e, 456);
        let before = r.raw.to_vec();
        let mode = [0, 1, 2, 3, 0xffff][i % 5];
        let timer = shadowrealm::weapons::cooldown(&r, mode);
        let mut tick = r.clone();
        shadowrealm::weapons::tick_cooldown(&mut tick);
        let angle = (i as u16).wrapping_mul(997);
        shadowrealm::weapons::prepare_ranged(rom, &mut r, angle)?;
        weapon_timing.push(json!({"before":before,"mode":mode,"timer":timer,"tick":tick.raw.to_vec(),"angle":angle,"ranged":r.raw.to_vec()}));
    }
    let mut selections = vec![];
    for i in 0..768usize {
        let mut actors = fixture_actors(rom, i)?;
        let source = if i % 2 == 0 { 0 } else { 3 };
        actors[source].raw[0x10] = if source == 0 { 0xd } else { 5 };
        let mut lists = TargetLists::default();
        lists.refresh(&actors.iter().collect::<Vec<_>>(), 0, true);
        let current = match (i / 2) % 6 {
            0 => 0,
            1 => if source == 0 {
                lists.npcs.first()
            } else {
                lists.party.first()
            }
            .map_or(0, |id| id.pointer()),
            2 => 0xff0a00,
            3 => 0xff0100 + source as u32 * 256,
            4 => 0xff0101,
            _ => 0x12345678,
        };
        actors[source].raw[0xa2..0xa6].copy_from_slice(&current.to_be_bytes());
        let before = raw(&actors);
        let seed = (i as u16).wrapping_mul(971);
        let mut random = State { seed };
        let empty_request = lists.acquire(&mut actors[source], &mut random);
        selections.push(json!({"source":source,"before":before,"after":raw(&actors),"npcs":ids(&lists.npcs),"party":ids(&lists.party),"seed":seed,"after_seed":random.seed,"empty_request":empty_request}));
    }
    let mut range = vec![];
    let mut source = Runner::load(rom, 0, 0)?;
    source.set_position(4000, 5000);
    for i in 0..1024usize {
        let offsets = [
            0u16, 1, 0xffff, 0x1ff, 0x200, 0x201, 0xfe01, 0xfe00, 0xfdff, 0x180, 0xff80, 0x1000,
        ];
        let mut target = Runner::load(rom, 3, 1)?;
        target.set_position(
            4000u16.wrapping_add(offsets[i % offsets.len()]),
            5000u16.wrapping_add(offsets[(i / offsets.len()) % offsets.len()]),
        );
        target.raw[0x10] = [1, 0x41, 0, 0x11, 5, 13][(i / 13) % 6];
        let probe = combat::close_range(rom, &source, &target)?;
        range.push(json!({"source":source.raw.to_vec(),"target":target.raw.to_vec(),"heading":probe.heading,"range":probe.range,"in_reach":probe.in_reach}));
    }
    let mut actors = fixture_actors(rom, 0)?;
    let mut lists = TargetLists::default();
    let mut random = State { seed: 0x3344 };
    let mut ticks = vec![];
    for phase in 0..256u16 {
        let fresh = fixture_actors(rom, phase as usize)?;
        for (r, new) in actors.iter_mut().zip(fresh) {
            r.raw[0x10] = new.raw[0x10];
            r.raw[0xb0] = new.raw[0xb0];
            r.raw[0x9e] = new.raw[0x9e];
            r.set_word(0xc, new.word(0xc));
        }
        actors[0].raw[0x10] = 0xd;
        let before = raw(&actors);
        lists.refresh(&actors.iter().collect::<Vec<_>>(), phase, phase == 0);
        let empty_request = lists.acquire(&mut actors[0], &mut random);
        ticks.push(json!({"before":before,"after":raw(&actors),"npcs":ids(&lists.npcs),"party":ids(&lists.party),"seed":random.seed,"empty_request":empty_request}));
    }
    let mut line_grids = vec![];
    let mut flights = vec![];
    let mut lines = vec![];
    let mut grids = vec![];
    for map in [0, 3, 5, 10, 31, 55] {
        grids.push((
            Some(map),
            Collision::load(rom, &MapDescriptor::read(rom, map)?)?,
        ));
    }
    for code in [0u8, 0x7f, 0x7e, 0x40, 0x60, 0x20, 0x28, 0x30, 0x38, 0x80, 1] {
        let mut cells = vec![0; 4096];
        for y in 0..16 {
            cells[y * 64 + 8] = code;
        }
        let rectangles = vec![
            Rect {
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
                code: 0,
            },
            Rect {
                left: 256,
                right: 287,
                top: 0,
                bottom: 511,
                code: 1,
            },
        ];
        grids.push((
            None,
            Collision {
                cells,
                rectangles,
                bounds: [0, 0, 512, 512],
                map_width_tiles: 32,
                overflow_writes: vec![],
            },
        ));
    }
    for (grid, (map, c)) in grids.iter().enumerate() {
        let mut positions = vec![];
        for i in 0..8192u32 {
            let left = c.bounds[0].max(32);
            let top = c.bounds[1].max(32);
            let x =
                left + (i * 73 % u32::from(c.bounds[2].saturating_sub(left + 32).max(1))) as u16;
            let y = top + (i * 97 % u32::from(c.bounds[3].saturating_sub(top + 32).max(1))) as u16;
            if c.contact((x * 16, y * 16))? == Contact::Clear {
                positions.push((x * 16, y * 16));
            }
            if positions.len() == 64 {
                break;
            }
        }
        if positions.len() < 16 {
            return Err("insufficient projectile fixture points".into());
        }
        for i in 0..192usize {
            use shadowrealm::projectiles::{self, FlightContext, FlightEvent};
            let mut actors = fixture_actors(rom, i)?;
            actors[0].raw[0xdf] = 0;
            actors[0].raw[0x57] = (1 + i % 12) as u8;
            actors[0].raw[0x10] = 0xd;
            actors[0].set_position(
                positions[i % positions.len()].0,
                positions[i % positions.len()].1,
            );
            actors[0].set_word(0x2e, (i as u16).wrapping_mul(997));
            let (mut projectile, _) = shadowrealm::weapons::initialize_projectile(
                rom,
                &actors[0],
                None,
                0,
                [0, 1, 3][i % 3],
            )?;
            let origin = (
                u16::from_be_bytes([projectile[0], projectile[1]]),
                u16::from_be_bytes([projectile[4], projectile[5]]),
            );
            projectile[0x58..0x5a].copy_from_slice(&origin.0.to_be_bytes());
            projectile[0x5a..0x5c].copy_from_slice(&origin.1.to_be_bytes());
            projectile[2..4].copy_from_slice(&((i * 971) as u16).to_be_bytes());
            projectile[6..8].copy_from_slice(&((i * 337) as u16).to_be_bytes());
            projectile[0x89] = [0, 1, 2, 3, 4, 0x7f, 0x80, 0xff][i / 3 % 8];
            projectile[0x41] = [0, 1, 2, 0x7f, 0x80, 0xff][i / 5 % 6];
            projectile[0x10] = if i % 2 == 0 { 9 } else { 1 };
            if i % 4 == 0 {
                projectile[0x11] = 9;
                projectile[0x16..0x18]
                    .copy_from_slice(&[0u16, 0xffff, 0x8000][i / 4 % 3].to_be_bytes());
            }
            // Overlapping candidates retain different eligibility flags; some
            // collisions must skip an earlier slot to reach a later one.
            for slot in 1..10 {
                if (i + slot) % 3 == 0 {
                    actors[slot].set_position(origin.0.wrapping_add((i % 5) as u16 * 32), origin.1);
                }
            }
            let before = projectile.to_vec();
            let before_actors = raw(&actors);
            let initial_samples = [[0u8; 4], [0, 0x20, 0x28, 0x38], [0, 0x7f, 0xc0, 0x68]][i % 3];
            let mut context = FlightContext {
                samples: initial_samples,
                deactivate_on_impact: i % 7 == 0,
            };
            let seed = (i as u16).wrapping_mul(971);
            let mut random = State { seed };
            let update = projectiles::advance(
                rom,
                c,
                &mut projectile,
                &mut actors,
                &mut context,
                &mut random,
            )?;
            let (event, actor, successes, rolls) = match update.event {
                FlightEvent::Moving => (0, None, None, 0),
                FlightEvent::Miss { actor } => (1, actor, None, 0),
                FlightEvent::DamagePending { actor, .. } => (2, Some(actor), None, 0),
                FlightEvent::ImpactAnimating => (3, None, None, 0),
                FlightEvent::Restarted { roll } => {
                    (4, None, Some(roll.result.successes), roll.result.rolls)
                }
                FlightEvent::Deactivated => (5, None, None, 0),
            };
            let resolved = if matches!(update.event, FlightEvent::DamagePending { .. }) {
                let mut end_projectile = projectile;
                let mut end_actors = actors.clone();
                let mut end_random = State { seed: random.seed };
                let mut world = shadowrealm::damage::DamageWorld {
                    defeat_timer: 17,
                    ..Default::default()
                };
                let result = projectiles::complete_impact(
                    rom,
                    &mut end_projectile,
                    &mut end_actors,
                    update.event,
                    &mut world,
                    &mut end_random,
                )?;
                Some(
                    json!({"projectile":end_projectile.to_vec(),"actors":raw(&end_actors),"seed":end_random.seed,"sound":result.sound,"rolls":result.damage.resistance.rolls,"world":{"reward":world.reward_count,"counter42":world.counter42,"counter13":world.counter13,"successes":world.last_successes,"kind":world.damage_kind,"penetration":world.penetration,"timer":world.defeat_timer}}),
                )
            } else {
                None
            };
            flights.push(json!({"resolved":resolved,"grid":grid,"before":before,"actors":before_actors,"samples":initial_samples,"deactivate":i%7==0,"after_deactivate":context.deactivate_on_impact,"seed":seed,"after":projectile.to_vec(),"after_actors":raw(&actors),"after_samples":context.samples,"after_seed":random.seed,"event":event,"actor":actor,"steps":update.steps,"phase":update.loop_phase,"sound":update.sound,"successes":successes,"rolls":rolls}));
        }
        for i in 0..128usize {
            let mut source = Runner::load(rom, 0, (i / 3) % 3)?;
            let mut target = Runner::load(rom, 3, 1)?;
            target.set_word(0x3e, 3);
            let (from, to) = if map.is_some() {
                (
                    positions[i % positions.len()],
                    positions[(i * 17 + 5) % positions.len()],
                )
            } else {
                (
                    (
                        [176u16, 208, 304, 336][i % 4] * 16,
                        (144 + (i as u16 * 19) % 224) * 16,
                    ),
                    (
                        [336u16, 368, 176, 208][i % 4] * 16,
                        (144 + (i as u16 * 47) % 224) * 16,
                    ),
                )
            };
            source.set_position(from.0, from.1);
            source.set_word(2, (i * 971) as u16);
            source.set_word(6, (i * 337) as u16);
            source.set_word(0x2e, (i as u16).wrapping_mul(0x800).wrapping_add(0x400));
            source.raw[0x10] = if i % 2 == 0 { 0xd } else { 5 };
            target.set_position(to.0, to.1);
            target.raw[0x10] = [5, 5, 5, 0, 0x45, 0x15, 1][i % 7];
            target.raw[0x4c] = if i % 11 == 0 { 0 } else { 10 };
            let scratch = [[0u8; 4], [0, 0x20, 0x28, 0x38], [0, 0x7f, 0xc0, 0x68]][i % 3];
            let probe = combat::line_of_fire(rom, c, &source, &target, scratch)?;
            lines.push(json!({"grid":grid,"source":source.raw.to_vec(),"target":target.raw.to_vec(),"initial_samples":scratch,"outcome":probe.outcome as u8,"steps":probe.steps,"projectile":probe.projectile.to_vec(),"samples":probe.samples}));
        }
        line_grids.push(json!({"map":map,"cells":c.cells,"bounds":c.bounds,"rectangles":c.rectangles.iter().map(|r|[r.left,r.right,r.top,r.bottom,u16::from(r.code)]).collect::<Vec<_>>()}));
    }
    fs::write(
        out.join("combat.json"),
        serde_json::to_vec(
            &json!({"failures":failures,"controls":controls,"reloads":reloads,"animations":animations,"reactions":reactions,"damages":damages,"flights":flights,"dice":dice_cases,"attack_rolls":attack_rolls,"launches":launches,"resources":resources,"admissions":admissions,"weapon_timing":weapon_timing,"selections":selections,"range":range,"ticks":ticks,"lines":lines,"line_grids":line_grids}),
        )?,
    )?;
    println!(
        "Exported 768 target selections, 1024 close-range probes, 256 refresh/retention ticks"
    );
    Ok(())
}
