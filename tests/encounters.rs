use shadowrealm::{
    avoidance::State, collision::Collision, encounters, game::Game, maps::MapDescriptor,
    party::Runner,
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn real_street_encounter_spawns_pursues_and_fires_in_game() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0]).unwrap();
    let mut spawned_at = None;
    let mut initial = vec![];
    for tick in 0..120_000 {
        game.update(&rom, 0).unwrap();
        if spawned_at.is_none() && game.encounters.spawned > 0 {
            spawned_at = Some(tick);
            initial = game
                .npcs
                .iter()
                .map(|a| (a.word(0x3e), a.position()))
                .collect();
            assert!(game
                .npcs
                .iter()
                .all(|a| a.raw[0x9e] & 0x40 != 0 && a.raw[0x4d] == 4));
            game.render(&rom).unwrap();
        }
        if game.combat.npc_shots > 0 && game.party[0].raw[0x4c] < 10 {
            break;
        }
        if let Some(start) = spawned_at {
            if tick - start > 3000 {
                break;
            }
        }
    }
    assert!(
        spawned_at.is_some(),
        "no original firearm group selected: deferred={}",
        game.encounters.deferred
    );
    assert!(
        game.combat.npc_shots > 0,
        "spawned actors failed to attack: {:?}",
        game.npcs
            .iter()
            .map(|a| (
                a.position(),
                a.word(0xc),
                a.raw[0xb0],
                a.word(0x16),
                a.raw[0x4e]
            ))
            .collect::<Vec<_>>()
    );
    assert!(game.npcs.iter().any(|a| initial
        .iter()
        .any(|(id, p)| *id == a.word(0x3e) && *p != a.position())));
    assert!(
        game.party[0].raw[0x4c] < 10,
        "real encounter did not damage the player"
    );
    eprintln!(
        "Original street encounter at tick {}, actors {}, NPC shots {}, player health {}",
        spawned_at.unwrap() + 1,
        game.encounters.spawned,
        game.combat.npc_shots,
        game.party[0].raw[0x4c]
    );
    assert_eq!(game.combat.shots, 0);
    game.render(&rom).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn unsupported_group_does_not_partially_allocate_or_consume_initialization_rng() {
    let rom = rom();
    let map = MapDescriptor::read(&rom, 0).unwrap();
    let mut collision = Collision::load(&rom, &map).unwrap();
    collision.cells.fill(0); // Explicit clear test arena for the placement gate.
    let mut actors = (0..10)
        .map(|slot| {
            let mut actor = Runner::load(&rom, 0, 0).unwrap();
            actor.set_word(0x3e, slot);
            actor.raw[0x10] = if slot == 0 { 0xf } else { 0 };
            actor
        })
        .collect::<Vec<_>>();
    let before = actors.iter().map(|a| a.raw).collect::<Vec<_>>();
    let mut rng = State::default();
    let mut runtime = encounters::Runtime::default();
    let mut world = shadowrealm::weapons::LaunchWorld::default();
    runtime
        .update(
            &rom,
            &map,
            (0x1000, 0x1000),
            &collision,
            &mut actors,
            10,
            &mut world,
            &mut rng,
        )
        .unwrap();
    assert_eq!(
        runtime.last,
        Some(encounters::Selection {
            kind: 0,
            address: 0x1d6d86
        })
    );
    assert_eq!(runtime.deferred, 1);
    assert_eq!(runtime.count, 0);
    assert_eq!(world.engaged, 0);
    assert_eq!(actors.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    let mut expected = State::default();
    assert!(encounters::due(
        encounters::Gate {
            tick: 10,
            chance: 0x6000,
            ..Default::default()
        },
        &mut expected
    ));
    encounters::select(&rom, 0, &mut expected).unwrap();
    assert_eq!(
        rng.seed, expected.seed,
        "unsupported group consumed initialization RNG or rerolled"
    );
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn encounter_death_counts_once_releases_offscreen_slot_and_allows_new_group() {
    let rom = rom();
    let map = MapDescriptor::read(&rom, 5).unwrap();
    let c = Collision::load(&rom, &map).unwrap();
    let mut actors = (0..10)
        .map(|slot| {
            let mut r = Runner::load(&rom, 0, 0).unwrap();
            r.set_word(0x3e, slot);
            r.raw[0x10] = if slot == 0 { 0xf } else { 0 };
            r
        })
        .collect::<Vec<_>>();
    let mut runtime = encounters::Runtime::default();
    let mut world = shadowrealm::weapons::LaunchWorld::default();
    let mut rng = State::default();
    for _ in 0..1000 {
        runtime
            .update(
                &rom,
                &map,
                (0x1000, 0x1000),
                &c,
                &mut actors,
                10,
                &mut world,
                &mut rng,
            )
            .unwrap();
        if runtime.count > 0 {
            break;
        }
    }
    assert!(runtime.count > 0);
    let spawned = runtime.spawned;
    for a in actors.iter_mut().skip(3).filter(|a| a.raw[0x10] & 1 != 0) {
        a.raw[0x10] |= 0x40;
        a.set_word(0xc, 0);
    }
    runtime
        .update(
            &rom,
            &map,
            (0x1000, 0x1000),
            &c,
            &mut actors,
            11,
            &mut world,
            &mut rng,
        )
        .unwrap();
    assert_eq!(runtime.count, 0);
    assert_eq!(world.engaged, 0);
    assert!(actors.iter().skip(3).all(|a| a.raw[0x10] & 1 == 0));
    runtime
        .update(
            &rom,
            &map,
            (0x1000, 0x1000),
            &c,
            &mut actors,
            12,
            &mut world,
            &mut rng,
        )
        .unwrap();
    assert_eq!(runtime.count, 0);
    for _ in 0..1000 {
        runtime
            .update(
                &rom,
                &map,
                (0x1000, 0x1000),
                &c,
                &mut actors,
                10,
                &mut world,
                &mut rng,
            )
            .unwrap();
        if runtime.count > 0 {
            break;
        }
    }
    assert!(runtime.spawned > spawned);
    assert!(runtime.count > 0);
}
