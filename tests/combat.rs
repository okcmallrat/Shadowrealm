use shadowrealm::{
    combat::{self, ActorId},
    game::Game,
    input, npc,
};

fn ordinary_npc_fixture(rom: &[u8], hostile: bool) -> Game {
    use shadowrealm::{collision::Collision, party::Runner};
    let mut game = Game::load(rom, 0, &[0]).unwrap();
    game.party[0].set_position(160 * 16, 200 * 16);
    game.party[0].set_word(0xc, 100);
    game.party[0].raw[0x66..0x85].fill(0);
    game.party[0].raw[0x76] = 0;
    game.party[0].set_word(0x90, 0);
    game.camera = (100, 100);
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let mut npc = Runner::load(rom, 3, 0).unwrap();
    npc.set_word(0x3e, 3);
    npc.set_position(340 * 16, 200 * 16);
    npc.set_word(0xc, 100);
    npc.raw[0x10] = 5;
    npc.raw[0x9e] = if hostile { 0x40 } else { 0 };
    npc.raw[0xb0] = 0;
    npc.raw[0xdf] = 0;
    npc.raw[0x56] = 0;
    npc.raw[0x57] = 2;
    npc.raw[0x4e] = 12;
    npc.raw[0x4f] = 0;
    npc.raw[0x4d] = 0;
    npc.raw[0x66..0x85].fill(0);
    npc.raw[0x78] = 3;
    npc.raw[0x81] = 8;
    npc.raw[0x82] = 8;
    npc.set_word(0xb6, 0);
    npc.set_word(0xb8, 0x1000);
    npc.set_word(0xdc, 0xffff);
    npc.set_word(0x16, 0);
    npc.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    npc.raw[0xa2..0xb0].fill(0);
    game.npcs.push(npc);
    game
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn game_hostile_npc_selects_pursues_fires_and_damages_player() {
    let rom = rom();
    let mut game = ordinary_npc_fixture(&rom, true);
    let initial =
        shadowrealm::followers::distance(game.party[0].position(), game.npcs[0].position());
    let mut closest = initial;
    for tick in 0..600 {
        game.update(&rom, 0).unwrap();
        closest = closest.min(shadowrealm::followers::distance(
            game.party[0].position(),
            game.npcs[0].position(),
        ));
        if tick % 60 == 0 {
            game.render(&rom).unwrap();
        }
        if game.party[0].raw[0x4c] < 10 {
            break;
        }
    }
    assert!(
        game.combat.npc_shots > 0,
        "ordinary hostile actor must fire automatically"
    );
    assert!(closest < initial, "far target must cause pursuit");
    assert!(
        game.party[0].raw[0x4c] < 10,
        "NPC projectiles must resolve real resistance/damage"
    );
    assert!(game.npcs[0].raw[0x4e] < 12);
    assert_eq!(
        game.combat.shots, 0,
        "player need not fire to trigger the original hostile flag"
    );
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn game_calm_npc_waits_for_combat_then_returns_fire() {
    let rom = rom();
    let mut game = ordinary_npc_fixture(&rom, false);
    // Keep the calm actor in its recovered firing band; a distant one may
    // legitimately lose the short ordinary alarm before it can close range.
    game.npcs[0].set_position(240 * 16, 200 * 16);
    for _ in 0..60 {
        game.update(&rom, 0).unwrap();
    }
    assert_eq!(game.combat.npc_shots, 0);
    assert_eq!(game.combat.launch.engaged, 0);
    game.update(&rom, input::A).unwrap();
    assert_eq!(game.combat.shots, 1);
    for _ in 0..600 {
        game.update(&rom, 0).unwrap();
        if game.combat.npc_shots > 0 {
            break;
        }
    }
    assert!(game.combat.npc_shots > 0);
    assert!(game.npcs[0].raw[0x4e] < 12);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn npc_fire_gate_connects_to_existing_launch_and_player_damage() {
    use shadowrealm::{
        avoidance::State,
        collision::Collision,
        damage::DamageWorld,
        npc_combat::{self, FireContext, FireDecision},
        party::Runner,
        projectiles::{self, FlightContext, FlightEvent},
        weapons::{self, LaunchResult, LaunchWorld},
    };
    let rom = rom();
    let collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let mut source = Runner::load(&rom, 3, 0).unwrap();
    source.set_word(0x3e, 3);
    source.set_position(1080 * 16, 1000 * 16);
    source.set_word(0xc, 1);
    source.set_word(0xdc, 0xffff);
    source.set_word(0x16, 0);
    source.raw[0x10] = 5;
    source.raw[0xdf] = 0;
    source.raw[0xb0] = 0;
    source.raw[0x56] = 0;
    source.raw[0x57] = 8;
    source.raw[0x4e] = 5;
    source.raw[0x81] = 60;
    source.raw[0x83] = 60;
    source.raw[0xa2..0xa6].copy_from_slice(&0xff0100u32.to_be_bytes());
    let mut target = Runner::load(&rom, 0, 0).unwrap();
    target.set_position(1000 * 16, 1000 * 16);
    target.raw[0x10] = 0xf;
    target.raw[0xdf] = 0;
    target.raw[0x4c] = 10;
    target.raw[0x4d] = 0;
    target.raw[0x66..0x85].fill(0);
    target.set_word(0x90, 0);
    let mut context = FireContext::default();
    assert_eq!(
        npc_combat::check_fire(&rom, &collision, &source, &target, &mut context)
            .unwrap()
            .decision,
        FireDecision::HoldFire
    );
    source.raw[0xb0] |= 0x10;
    assert_eq!(
        npc_combat::check_fire(&rom, &collision, &source, &target, &mut context)
            .unwrap()
            .decision,
        FireDecision::ReadyForLaunch
    );
    assert_eq!(context.firing, 1);
    let mut pool = [[0u8; 256]; 6];
    let mut random = State { seed: 333 };
    let (result, roll) = weapons::launch_ranged(
        &rom,
        &mut source,
        Some(&mut target),
        &mut pool,
        &mut LaunchWorld::default(),
        &mut random,
    )
    .unwrap();
    assert_eq!(
        source.raw[0x4e], 4,
        "NPC burst launch consumes one resource unit"
    );
    assert!(roll.is_some());
    let LaunchResult::Initialized { projectile, .. } = result else {
        panic!("expected NPC launch")
    };
    let mut actors = vec![
        target,
        Runner::load(&rom, 3, 1).unwrap(),
        Runner::load(&rom, 10, 2).unwrap(),
        source,
    ];
    actors[1].raw[0x10] = 0;
    actors[2].raw[0x10] = 0;
    let mut flight = FlightContext::default();
    let mut pending = None;
    for _ in 0..20 {
        let update = projectiles::advance(
            &rom,
            &collision,
            &mut pool[projectile],
            &mut actors,
            &mut flight,
            &mut random,
        )
        .unwrap();
        match update.event {
            FlightEvent::Moving => {}
            FlightEvent::DamagePending { actor: 0, .. } => {
                pending = Some(update.event);
                break;
            }
            event => panic!("unexpected NPC flight event {event:?}"),
        }
    }
    let mut world = DamageWorld::default();
    let impact = projectiles::complete_impact(
        &rom,
        &mut pool[projectile],
        &mut actors,
        pending.unwrap(),
        &mut world,
        &mut random,
    )
    .unwrap();
    assert!(impact.damage.health_after < impact.damage.health_before);
    assert_eq!(actors[0].raw[0x4c], 0);
    assert_eq!(world.defeat_timer, 60);
    assert_eq!(actors[3].raw[0x4e], 4);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn game_fire_input_consumes_ammo_renders_flash_and_retires_projectiles() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
    game.party[0].raw[0x4e] = rom[0xd4540 + usize::from(game.party[0].raw[0x57] - 1) * 48 + 0x24];
    game.refresh_hud(&rom).unwrap();
    let before = game.render(&rom).unwrap();
    let ammo = game.party[0].raw[0x4e];
    game.update(&rom, input::A).unwrap();
    assert_eq!(game.combat.shots, 1);
    assert_eq!(game.party[0].raw[0x4e], ammo - 1);
    assert!(game.combat.has_live());
    assert_eq!(&game.party[0].raw[0x3a..0x3e], &0xb3eeu32.to_be_bytes());
    let flash = game.render(&rom).unwrap();
    assert_ne!(flash, before);
    let pool = game.combat.pool;
    game.combat.pool = [[0; 256]; 6];
    let without_flash = game.render(&rom).unwrap();
    game.combat.pool = pool;
    assert_ne!(
        flash, without_flash,
        "resident muzzle graphics must contribute visible pixels"
    );
    for _ in 0..200 {
        game.update(&rom, 0).unwrap();
    }
    assert_eq!(game.combat.shots, 1);
    assert!(!game.combat.has_live());
    assert_eq!(game.party[0].raw[0x4e], ammo - 1);
    game.update(&rom, input::A).unwrap();
    assert_eq!(
        game.combat.shots, 2,
        "ended firing pose must return to controllable idle"
    );
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn game_targeted_fire_applies_damage_and_keeps_reaction_poses() {
    use shadowrealm::{collision::Collision, party::Runner};
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0]).unwrap();
    game.party[0].set_position(200 * 16, 200 * 16);
    game.party[0].raw[0x57] = 12;
    game.party[0].raw[0x4e] = 8;
    game.party[0].raw[0x84] = 20;
    game.party[0].raw[0x81] = 20;
    game.camera = (100, 100);
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let mut enemy = Runner::load(&rom, 3, 0).unwrap();
    enemy.set_word(0x3e, 3);
    enemy.set_word(0xc, 100);
    enemy.set_position(280 * 16, 200 * 16);
    enemy.raw[0x10] = 5;
    enemy.raw[0xdf] = 0;
    enemy.raw[0x9a] = 0;
    enemy.raw[0x9b] = 0;
    enemy.raw[0x9e] = 0;
    enemy.raw[0xb0] = 0;
    enemy.raw[0x66..0x85].fill(0);
    enemy.raw[0x4c] = 10;
    enemy.raw[0x4d] = 0;
    enemy.set_word(0x90, 0);
    game.npcs.push(enemy);
    game.update(&rom, input::C).unwrap();
    assert_eq!(game.selected_target().unwrap().word(0x3e), 3);
    game.update(&rom, input::A).unwrap();
    for _ in 0..30 {
        game.update(&rom, 0).unwrap();
        if game.combat.hits > 0 {
            break;
        }
    }
    assert_eq!(game.combat.shots, 1);
    assert_eq!(game.combat.hits, 1);
    assert!(game.npcs[0].raw[0x4c] < 10);
    assert!(matches!(
        shadowrealm::combat_runtime::handler(&game.npcs[0]),
        0xb07e | 0xb38a
    ));
    for tick in 0..120 {
        game.update(&rom, 0).unwrap();
        if tick < 8 || tick % 20 == 0 {
            game.render(&rom).unwrap();
        }
    }
    assert!(!game.combat.has_live());
    assert_eq!(game.combat.shots, 1);
    if game.npcs[0].raw[0x4c] == 0 {
        assert_eq!(game.npcs[0].raw[0x10] & 0x50, 0x50);
    }
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn game_held_burst_fire_is_bounded_and_targeting_is_optional() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0]).unwrap();
    game.party[0].raw[0x57] = 8;
    game.party[0].raw[0x4e] = 30;
    game.party[0].raw[0x4f] = 0;
    for tick in 0..240 {
        game.update(&rom, input::A).unwrap();
        if tick < 8 || tick % 20 == 0 {
            game.render(&rom).unwrap();
        }
    }
    assert!(!game.targeting.enabled);
    assert!(game.combat.shots >= 2 && game.combat.shots < 15);
    assert!(game.party[0].raw[0x4e] < 30);
    for _ in 0..240 {
        game.update(&rom, 0).unwrap();
    }
    assert!(
        !game.combat.has_live(),
        "burst/FX slots must retire rather than leak"
    );
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn game_queues_cooldown_tap_and_reloads_from_finite_reserves() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0]).unwrap();
    game.party[0].set_word(0xdc, 3);
    game.update(&rom, input::A).unwrap();
    assert_eq!(game.combat.shots, 0);
    for _ in 0..4 {
        game.update(&rom, 0).unwrap();
    }
    assert_eq!(
        game.combat.shots, 1,
        "a tapped request survives release during cooldown"
    );
    for _ in 0..200 {
        game.update(&rom, 0).unwrap();
    }
    game.party[0].raw[0x4e] = 0;
    game.party[0].raw[0x4f] = 1;
    game.update(&rom, input::A).unwrap();
    assert_eq!(game.combat.shots, 1);
    assert_eq!(game.party[0].raw[0x4f], 0);
    assert!(game.party[0].raw[0x4e] > 0);
    assert_eq!(game.combat.status, "Reloading");
    for _ in 0..200 {
        game.update(&rom, 0).unwrap();
    }
    game.party[0].raw[0x4e] = 0;
    game.update(&rom, input::A).unwrap();
    assert_eq!(game.combat.status, "No ammunition");
    assert_eq!(game.party[0].raw[0x4e], 0);
    assert_eq!(game.combat.shots, 1);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn launched_hit_applies_resistance_damage_defeat_rewards_and_death_reaction() {
    use shadowrealm::{
        avoidance::State,
        collision::Collision,
        damage::DamageWorld,
        party::Runner,
        projectiles::{self, FlightContext, FlightEvent},
        weapons::{self, LaunchResult, LaunchWorld},
    };
    let rom = rom();
    let mut source = Runner::load(&rom, 0, 0).unwrap();
    source.raw[0x10] = 0xd;
    source.raw[0xdf] = 0;
    source.raw[0x56] = 0;
    source.raw[0x57] = 11;
    source.raw[0x4e] = 3;
    source.set_word(0x16, 0);
    source.set_position(1000 * 16, 1000 * 16);
    source.raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
    let mut target = Runner::load(&rom, 3, 0).unwrap();
    target.set_word(0x3e, 3);
    target.set_position(1000 * 16 + 0x500, 1000 * 16);
    target.raw[0x10] = 1;
    target.raw[0x4c] = 10;
    target.raw[0x4d] = 0;
    target.raw[0xdf] = 0;
    target.raw[0x66..0x85].fill(0);
    target.set_word(0x90, 0);
    target.raw[0x9a] = 0;
    target.raw[0x9b] = 0x42;
    target.raw[0xb0] = 0;
    target.raw[0x9e] = 0;
    let mut pool = [[0u8; 256]; 6];
    // Supplied successes isolate this test's guaranteed lethal hit; original
    // launch-roll generation is covered by the separate differential/replay tests.
    let result = weapons::launch_with_resolved_roll(
        &rom,
        &mut source,
        Some(&mut target),
        &mut pool,
        &mut LaunchWorld::default(),
        20,
    )
    .unwrap();
    let LaunchResult::Initialized { projectile, .. } = result else {
        panic!("expected launch")
    };
    let ammo = source.raw[0x4e];
    let mut actors = vec![
        source,
        Runner::load(&rom, 3, 1).unwrap(),
        Runner::load(&rom, 10, 2).unwrap(),
        target,
    ];
    actors[1].raw[0x10] = 0;
    actors[2].raw[0x10] = 0;
    let rewards: Vec<_> = actors[..3].iter().map(|r| r.raw[0x8d]).collect();
    let collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let mut context = FlightContext::default();
    let mut random = State { seed: 4321 };
    let mut pending = None;
    for _ in 0..64 {
        let update = projectiles::advance(
            &rom,
            &collision,
            &mut pool[projectile],
            &mut actors,
            &mut context,
            &mut random,
        )
        .unwrap();
        match update.event {
            FlightEvent::Moving => {}
            FlightEvent::DamagePending { actor: 3, .. } => {
                pending = Some(update.event);
                break;
            }
            event => panic!("unexpected event {event:?}"),
        }
    }
    assert_eq!(actors[3].raw[0x4c], 10);
    let mut world = DamageWorld {
        counter42: 19,
        defeat_timer: 17,
        ..Default::default()
    };
    let result = projectiles::complete_impact(
        &rom,
        &mut pool[projectile],
        &mut actors,
        pending.unwrap(),
        &mut world,
        &mut random,
    )
    .unwrap();
    assert!(result.damage.npc_defeat);
    assert_eq!(result.damage.health_after, 0);
    assert!(result.sound.is_some());
    assert_ne!(random.seed, 4321);
    assert_eq!(world.counter42, 20);
    assert_eq!(world.reward_count, 15);
    assert_eq!(world.defeat_timer, 17);
    for (actor, before) in actors[..3].iter().zip(rewards) {
        assert_eq!(actor.raw[0x8d], before.wrapping_add(1));
    }
    assert_eq!(actors[3].raw[0x10] & 0x10, 0x10);
    assert_eq!(&actors[3].raw[0x3a..0x3e], &0xb38au32.to_be_bytes());
    assert_eq!(
        u16::from_be_bytes([pool[projectile][0x16], pool[projectile][0x17]]),
        0x10
    );
    let seed = random.seed;
    assert_eq!(
        projectiles::advance(
            &rom,
            &collision,
            &mut pool[projectile],
            &mut actors,
            &mut context,
            &mut random
        )
        .unwrap()
        .event,
        FlightEvent::ImpactAnimating
    );
    assert_eq!(random.seed, seed);
    assert_eq!(actors[0].raw[0x4e], ammo);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn burst_launch_flies_three_shots_and_retires_without_spending_more_ammo() {
    use shadowrealm::{
        avoidance::State,
        collision::Collision,
        party::Runner,
        projectiles::{self, FlightContext, FlightEvent},
        weapons::{self, LaunchResult, LaunchWorld},
    };
    let rom = rom();
    let mut source = Runner::load(&rom, 0, 0).unwrap();
    source.raw[0xdf] = 0;
    source.raw[0x10] = 0xd;
    source.raw[0x57] = 8;
    source.raw[0x56] = 0;
    source.raw[0x4e] = 20;
    source.raw[0xa2..0xa6].fill(0);
    source.set_word(0x16, 0);
    source.set_position(1000 * 16, 1000 * 16);
    let collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let mut pool = [[0u8; 256]; 6];
    let mut random = State { seed: 1234 };
    let (result, _) = weapons::launch_ranged(
        &rom,
        &mut source,
        None,
        &mut pool,
        &mut LaunchWorld::default(),
        &mut random,
    )
    .unwrap();
    let LaunchResult::Initialized { projectile, .. } = result else {
        panic!("expected launch")
    };
    let mut actors = vec![source];
    let mut context = FlightContext::default();
    for shot in 0..3 {
        let mut steps = 0;
        let seed = random.seed;
        loop {
            let update = projectiles::advance(
                &rom,
                &collision,
                &mut pool[projectile],
                &mut actors,
                &mut context,
                &mut random,
            )
            .unwrap();
            steps += update.steps;
            assert_eq!(random.seed, seed, "flight does not roll again");
            match update.event {
                FlightEvent::Moving => assert!(steps <= 32),
                FlightEvent::Miss { actor: None } => break,
                event => panic!("unexpected flight event: {event:?}"),
            }
        }
        assert_eq!(steps, 33);
        // Animation playback is owned by the caller. Simulate its end marker.
        pool[projectile][0x16..0x18].copy_from_slice(&0xffffu16.to_be_bytes());
        let update = projectiles::advance(
            &rom,
            &collision,
            &mut pool[projectile],
            &mut actors,
            &mut context,
            &mut random,
        )
        .unwrap();
        if shot < 2 {
            assert!(matches!(update.event, FlightEvent::Restarted { .. }));
            assert_ne!(random.seed, seed);
        } else {
            assert_eq!(update.event, FlightEvent::Deactivated);
            assert_eq!(random.seed, seed);
        }
        assert_eq!(
            actors[0].raw[0x4e], 17,
            "burst continuations do not spend ammunition again"
        );
    }
    assert_eq!(pool[projectile][0x10] & 1, 0);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn original_launch_roll_obeys_admission_and_unsupported_effect_boundaries() {
    use shadowrealm::{
        avoidance::State,
        party::Runner,
        weapons::{self, LaunchAdmission, LaunchResult, LaunchWorld},
    };
    let rom = rom();
    let mut source = Runner::load(&rom, 0, 0).unwrap();
    source.set_word(0x16, 0);
    source.raw[0xa2..0xa6].fill(0);
    source.raw[0xdf] = 1;
    source.raw[0x57] = 2;
    source.raw[0x56] = 0;
    source.raw[0x4e] = 10;
    let mut pool = [[0u8; 256]; 6];
    for raw in &mut pool {
        raw[0x10] = 1;
    }
    let mut world = LaunchWorld::default();
    let mut random = State { seed: 1234 };
    let (result, roll) =
        weapons::launch_ranged(&rom, &mut source, None, &mut pool, &mut world, &mut random)
            .unwrap();
    assert_eq!(result, LaunchResult::Rejected(LaunchAdmission::PoolFull));
    assert!(roll.is_none());
    assert_eq!(source.raw[0x4e], 9);
    assert_eq!(random.seed, 1234);
    pool[2][0x10] = 0;
    let before = source.raw;
    let before_pool = pool;
    assert!(
        weapons::launch_ranged(&rom, &mut source, None, &mut pool, &mut world, &mut random)
            .is_err()
    );
    assert_eq!(source.raw, before);
    assert_eq!(pool, before_pool);
    assert_eq!(random.seed, 1234);
    assert_eq!(world.engaged, 0);
    source.raw[0xdf] = 0;
    let health = source.raw[0x4c];
    let (result, roll) =
        weapons::launch_ranged(&rom, &mut source, None, &mut pool, &mut world, &mut random)
            .unwrap();
    assert_eq!(result, LaunchResult::FxPoolFull { projectile: 2 });
    let roll = roll.unwrap();
    assert!(roll.result.rolls >= 1);
    assert_ne!(random.seed, 1234);
    assert_eq!(source.raw[0x9f], roll.result.successes);
    assert_eq!(pool[2][0x9f], roll.result.successes);
    assert_eq!(source.raw[0x4e], 8);
    assert_eq!(source.raw[0x4c], health);
    assert_eq!(world.timeout, 120);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn launched_projectile_survives_fx_failure_and_success_preserves_fx_tail() {
    use shadowrealm::{
        party::Runner,
        weapons::{self, LaunchResult, LaunchWorld},
    };
    let rom = rom();
    let mut source = Runner::load(&rom, 0, 0).unwrap();
    source.set_word(0x16, 0);
    source.raw[0xa2..0xa6].fill(0);
    source.raw[0x57] = 8;
    source.raw[0x56] = 0;
    source.raw[0x4e] = 10;
    let mut pool = [[0xa5; 256]; 6];
    for raw in &mut pool {
        raw[0x10] = 1;
    }
    pool[2][0x10] = 0;
    let mut world = LaunchWorld::default();
    assert_eq!(
        weapons::launch_with_resolved_roll(&rom, &mut source, None, &mut pool, &mut world, 3)
            .unwrap(),
        LaunchResult::FxPoolFull { projectile: 2 }
    );
    assert_eq!(source.raw[0x4e], 7);
    assert_eq!(source.raw[0x9f], 3);
    assert_eq!(pool[2][0x10] & 1, 1);
    assert_eq!(pool[2][0x89], 0x20);
    assert_eq!(world.timeout, 120);
    // Two available entries now allow both projectile and muzzle FX.
    pool[0][0x10] = 0;
    pool[5][0x10] = 0;
    let old_projectile = pool[2];
    let fx_tail = pool[5][160..].to_vec();
    let result =
        weapons::launch_with_resolved_roll(&rom, &mut source, None, &mut pool, &mut world, 7)
            .unwrap();
    assert!(matches!(
        result,
        LaunchResult::Initialized {
            projectile: 0,
            fx: 5,
            ..
        }
    ));
    assert_eq!(pool[2], old_projectile);
    assert_eq!(&pool[5][160..], fx_tail);
    assert!(pool[0][160..].iter().all(|b| *b == 0));
    assert_eq!(pool[0][0x9f], 7);
    assert_eq!(pool[5][0x9f], 7);
    assert_eq!(source.raw[0x4e], 4);
    assert_eq!(pool[0][0x41], 2);
    assert_eq!(pool[5][0x41], 0);
    assert_eq!(pool[5][0xe], 0);
    assert_eq!(
        source.raw[0x4c],
        Runner::load(&rom, 0, 0).unwrap().raw[0x4c],
        "initialization does not apply damage"
    );
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn launch_prefix_preserves_resource_loss_on_full_pool_and_consumable_removal() {
    use shadowrealm::{
        party::Runner,
        weapons::{self, LaunchAdmission, LaunchSignals},
    };
    let rom = rom();
    let mut runner = Runner::load(&rom, 0, 0).unwrap();
    runner.set_word(0x16, 0);
    runner.raw[0x10] = 0xf;
    runner.raw[0x56] = 0;
    runner.raw[0x57] = 8;
    runner.raw[0x4e] = 2;
    let mut pool = [[0u8; 256]; 6];
    for entry in &mut pool {
        entry[0x10] = 1;
    }
    let mut signals = LaunchSignals::default();
    assert_eq!(
        weapons::admit_ranged_prefix(&rom, &mut runner, 0, &mut signals, &pool).unwrap(),
        LaunchAdmission::PoolFull
    );
    assert_eq!(runner.raw[0x4e], 0);
    assert!(signals.burst);
    pool[4][0x10] = 0;
    assert_eq!(
        weapons::admit_ranged_prefix(&rom, &mut runner, 0, &mut signals, &pool).unwrap(),
        LaunchAdmission::EmptyResource
    );
    runner.raw[0x57] = 13;
    runner.raw[0x4e] = 1;
    runner.raw[0x66] = 13;
    runner.raw[0x6e] = 1;
    assert_eq!(
        weapons::admit_ranged_prefix(&rom, &mut runner, 0, &mut signals, &pool).unwrap(),
        LaunchAdmission::ReadyForInitialization { slot: 4 }
    );
    assert_eq!(
        (
            runner.raw[0x56],
            runner.raw[0x57],
            runner.raw[0x66],
            runner.raw[0x6e]
        ),
        (0x7f, 0, 0, 0)
    );
    assert_eq!(runner.raw[0x4e], 1);
    assert_eq!(pool[4][0x10], 0, "prefix lookup does not reserve the entry");
    // State and mode rejections occur before resource consumption; the busy
    // signal persists until the owning controller explicitly clears it.
    runner.raw[0x57] = 8;
    runner.raw[0x56] = 0;
    runner.raw[0x4e] = 10;
    runner.set_word(0x16, 0x3d);
    let before = runner.raw;
    assert_eq!(
        weapons::admit_ranged_prefix(&rom, &mut runner, 3, &mut signals, &pool).unwrap(),
        LaunchAdmission::ModeBlocked
    );
    assert!(!signals.controlled_busy);
    assert_eq!(
        weapons::admit_ranged_prefix(&rom, &mut runner, 0, &mut signals, &pool).unwrap(),
        LaunchAdmission::StateBlocked
    );
    assert!(signals.controlled_busy);
    assert_eq!(runner.raw, before);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn cooldown_advances_active_actors_and_freezes_during_dialogue() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
    game.party[0].set_word(0xdc, 1);
    game.party[1].set_word(0xdc, 0x8000);
    game.party[2].raw[0x10] &= !1;
    game.party[2].set_word(0xdc, 20);
    game.update(&rom, 0).unwrap();
    assert_eq!(game.party[0].word(0xdc), 0);
    game.update(&rom, 0).unwrap();
    assert_eq!(game.party[0].word(0xdc), 0xffff);
    assert_eq!(game.party[1].word(0xdc), 0x8000);
    assert_eq!(game.party[2].word(0xdc), 20);
    // Enter the recovered Space Needle acknowledgement dialogue.
    game.party[0].set_position(528 * 16, 144 * 16);
    game.party[0].set_word(0x2e, 0);
    game.camera = (392, 32);
    for _ in 0..30 {
        game.update(&rom, input::UP).unwrap();
        if game.dialogue.is_some() {
            break;
        }
    }
    assert!(game.dialogue.is_some());
    game.party[0].set_word(0xdc, 12);
    game.update(&rom, 0).unwrap();
    assert_eq!(game.party[0].word(0xdc), 12);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn line_probe_marker_tracks_a_wall_without_firing_or_advancing_state() {
    use shadowrealm::{collision::Collision, combat::LineOutcome};
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0]).unwrap();
    game.party[0].set_position(208 * 16, 240 * 16);
    let mut npc = npc::space_needle(&rom).unwrap().remove(0);
    npc.set_position(336 * 16, 240 * 16);
    game.npcs.push(npc);
    let mut cells = vec![0; 4096];
    for y in 0..16 {
        cells[y * 64 + 8] = 0x7f;
    }
    game.collision = Collision {
        cells,
        rectangles: vec![],
        bounds: [0, 0, 512, 512],
        map_width_tiles: 32,
        overflow_writes: vec![],
    };
    game.update(&rom, 0).unwrap();
    game.update(&rom, input::C).unwrap();
    assert_eq!(
        game.selected_fire_probe(&rom).unwrap().unwrap().outcome,
        LineOutcome::Blocked
    );
    let tick = game.tick;
    let funds = game.nuyen;
    let actors: Vec<_> = game
        .party
        .iter()
        .chain(game.npcs.iter())
        .map(|r| r.raw)
        .collect();
    let frame = game.render(&rom).unwrap();
    assert!(
        frame
            .chunks_exact(3)
            .filter(|rgb| *rgb == [255, 64, 64])
            .count()
            >= 16
    );
    for y in 0..16 {
        game.collision.cells[y * 64 + 8] = 0;
    }
    assert_eq!(
        game.selected_fire_probe(&rom).unwrap().unwrap().outcome,
        LineOutcome::TargetReached
    );
    let frame = game.render(&rom).unwrap();
    assert!(
        frame
            .chunks_exact(3)
            .filter(|rgb| *rgb == [255, 224, 64])
            .count()
            >= 16
    );
    assert_eq!(game.tick, tick);
    assert_eq!(game.nuyen, funds);
    assert_eq!(
        game.party
            .iter()
            .chain(game.npcs.iter())
            .map(|r| r.raw)
            .collect::<Vec<_>>(),
        actors
    );
}
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var_os("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM"))
        .unwrap()
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn target_toggle_marker_retention_and_leader_switch_do_not_attack() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3]).unwrap();
    game.party[0].set_position(528 * 16, 160 * 16);
    game.party[1].set_position(700 * 16, 200 * 16);
    let mut npc = npc::space_needle(&rom).unwrap().remove(0);
    npc.set_position(560 * 16, 160 * 16);
    game.npcs.push(npc); // isolated target-selection fixture, not a new map spawn
    game.update(&rom, 0).unwrap();
    let stats: Vec<_> = game
        .party
        .iter()
        .chain(game.npcs.iter())
        .map(|r| r.raw[0x4c..0x92].to_vec())
        .collect();
    let funds = game.nuyen;
    assert!(game.selected_target().is_none());
    game.update(&rom, input::C).unwrap();
    assert!(game.targeting.enabled);
    assert_eq!(combat::target(&game.party[0]), Some(ActorId(3)));
    assert!(
        !combat::close_range(&rom, &game.party[0], &game.npcs[0])
            .unwrap()
            .in_reach
    );
    let tick = game.tick;
    let frame = game.render(&rom).unwrap();
    assert_eq!(game.tick, tick);
    assert!(
        frame
            .chunks_exact(3)
            .filter(|rgb| *rgb == [255, 224, 64])
            .count()
            >= 16
    );
    game.update(&rom, input::C).unwrap();
    assert!(
        game.targeting.enabled,
        "holding target input must not toggle repeatedly"
    );
    assert_eq!(combat::target(&game.party[0]), Some(ActorId(3)));
    game.select(&rom, 1).unwrap();
    game.update(&rom, 0).unwrap();
    assert!(combat::target(&game.party[0]).is_none());
    assert_eq!(combat::target(&game.party[1]), Some(ActorId(3)));
    game.update(&rom, input::C).unwrap();
    assert!(!game.targeting.enabled);
    assert!(game.party.iter().all(|r| combat::target(r).is_none()));
    for (r, stat) in game.party.iter().chain(game.npcs.iter()).zip(stats) {
        assert_eq!(
            &r.raw[0x4c..0x92],
            stat,
            "targeting must not consume ammo or apply damage"
        );
    }
    assert_eq!(game.nuyen, funds);
    assert!(game.dialogue.is_none());
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn selecting_trent_preserves_the_original_wider_conversation_contact() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0]).unwrap();
    game.party[0].set_position(528 * 16, 144 * 16);
    game.party[0].set_word(0x2e, 0);
    game.camera = (392, 32);
    for _ in 0..30 {
        game.update(&rom, input::UP).unwrap();
        if game.dialogue.is_some() {
            break;
        }
    }
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0x110b5e);
    game.update(&rom, input::A).unwrap();
    for _ in 0..200 {
        game.update(&rom, 0).unwrap();
        if game.completed_transitions == 1 && game.transition.is_none() {
            break;
        }
    }
    assert_eq!(game.map_id(), 10);
    let (x, y) = game.npcs[0].position();
    game.party[0].set_position(x + 128, y);
    game.update(&rom, 0).unwrap();
    assert!(
        game.dialogue.is_none(),
        "eight pixels is outside the unlinked contact window"
    );
    let health = game.npcs[0].health();
    game.update(&rom, input::C).unwrap();
    assert_eq!(combat::target(&game.party[0]), Some(ActorId(3)));
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfeb4c);
    assert_eq!(game.knowledge.runner_known, 4);
    assert_eq!(game.npcs[0].health(), health);
}
