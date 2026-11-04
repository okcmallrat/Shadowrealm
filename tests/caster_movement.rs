use shadowrealm::{collision::Collision, combat_runtime, game::Game, party::Runner};

fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn equip_freya(game: &mut Game, rom: &[u8]) {
    game.select(rom, 2).unwrap();
    game.cycle_spell(rom).unwrap();
    assert_eq!((game.party[2].raw[0x57], game.party[2].raw[0x9d]), (9, 3));
    game.select(rom, 0).unwrap();
}
fn fixture(rom: &[u8]) -> Game {
    let mut game = Game::load(rom, 0, &[0, 3, 10]).unwrap();
    game.encounters.count = 1; // Isolated fixture: no random encounter admission.
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    // Bound navigation scans with a physical arena border, as real maps do.
    for cell in 0..4096 {
        if cell / 64 == 0 || cell / 64 == 63 || cell % 64 == 0 || cell % 64 == 63 {
            game.collision.cells[cell] = 0x7f;
        }
    }
    equip_freya(&mut game, rom);
    game.party[0].set_position(210 * 16, 200 * 16);
    game.party[1].set_position(180 * 16, 180 * 16);
    game.party[1].set_word(0xb6, 100); // Let Freya service the first fixture decision.
    game.party[2].set_position(220 * 16, 200 * 16);
    game.party[2].set_word(0xdc, 0xffff);
    game.party[2].set_word(0xb6, 0);
    let mut target = Runner::load(rom, 0, 0).unwrap();
    target.set_word(0x3e, 3);
    target.set_position(300 * 16, 210 * 16);
    target.raw[0x10] = 5;
    target.raw[0x56] = 127;
    target.raw[0x57] = 0;
    target.raw[0x9b] = 2;
    target.raw[0x9e] = 0;
    target.raw[0xb0] = 0;
    target.raw[0xb2] = 0;
    target.raw[0xdf] = 0;
    target.raw[0x7c] = 0;
    target.set_word(0xc, 100);
    game.npcs.push(target);
    game.party[2].raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
    game.combat.launch.engaged = 65535;
    game.combat.launch.timeout = 3000;
    game
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn blocked_caster_navigates_around_a_wall_then_casts_without_player_fire() {
    let rom = rom();
    let mut game = fixture(&rom);
    for row in 5..7 {
        game.collision.cells[row * 64 + 8] = 0x7f;
    }
    let before = game.party[2].position();
    let mut moved_before_cast = false;
    for _ in 0..1800 {
        game.update(&rom, 0).unwrap();
        if game.combat.casting.casts == 0 && game.party[2].position() != before {
            moved_before_cast = true;
        }
        if game.combat.casting.impacts > 0 {
            break;
        }
    }
    assert!(
        moved_before_cast,
        "blocked casters must pursue before casting"
    );
    assert!(
        game.combat.casting.impacts > 0,
        "did not cast after wall recovery: {} / {:?}",
        game.combat.status,
        game.combat.companion_deferred
    );
    assert!(game.npcs[0].raw[0x4c] < 10);
    eprintln!(
        "Wall recovery cast at tick {}, Freya {:?} -> {:?}",
        game.tick,
        before,
        game.party[2].position()
    );
    game.render(&rom).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn offscreen_caster_pursues_a_retained_target_until_visible_then_casts() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.party[0].set_position(300 * 16, 300 * 16);
    game.party[1].set_position(280 * 16, 280 * 16);
    game.party[2].set_position(700 * 16, 300 * 16);
    game.party[2].set_word(0xc, 0);
    game.npcs[0].set_position(420 * 16, 300 * 16);
    let before = game.party[2].position();
    let mut offscreen_move = false;
    for _ in 0..1800 {
        game.update(&rom, 0).unwrap();
        if game.party[2].word(0xc) == 0 {
            assert_eq!(
                game.combat.casting.casts, 0,
                "offscreen caster must not cast"
            );
            offscreen_move |= game.party[2].position() != before;
        }
        if game.combat.casting.impacts > 0 {
            break;
        }
    }
    assert!(offscreen_move);
    assert!(
        game.combat.casting.impacts > 0,
        "caster never returned to action"
    );
    assert!(game.npcs[0].raw[0x4c] < 10);
    game.render(&rom).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn cooling_caster_repositions_without_spending_spell_or_weapon_resources() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.party[2].set_word(0xdc, 100);
    game.npcs[0].set_position(360 * 16, 210 * 16);
    let before = game.party[2].position();
    let resources = (
        game.party[2].raw[0x4e],
        game.party[2].raw[0x4f],
        game.party[2].raw[0x9c],
    );
    assert!(
        resources.0 > 0,
        "fixture retains original equipped weapon percentage"
    );
    for _ in 0..50 {
        game.update(&rom, 0).unwrap();
    }
    assert_ne!(
        game.party[2].position(),
        before,
        "state {:?} {:?} {:?} {}",
        &game.party[2].raw[0x96..0x9a],
        &game.party[2].raw[0xb0..0xba],
        &game.party[2].raw[0x3a..0x3e],
        game.combat.launch.engaged
    );
    assert_eq!(game.combat.casting.casts, 0);
    assert_eq!(game.combat.shots, 0);
    assert_eq!(
        (
            game.party[2].raw[0x4e],
            game.party[2].raw[0x4f],
            game.party[2].raw[0x9c]
        ),
        resources
    );
    assert_eq!(game.party[2].raw[0x57], 9);
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn caster_with_no_enemies_shuts_down_combat_and_rejoins_formation_immediately() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.npcs.clear();
    game.party[2].set_position(400 * 16, 300 * 16);
    game.update(&rom, 0).unwrap();
    assert_eq!(game.combat.launch.engaged, 0);
    assert!(shadowrealm::combat::target(&game.party[2]).is_none());
    assert_eq!(game.combat.casting.casts, 0);
    assert_eq!(combat_runtime::handler(&game.party[2]), 0xb574);
    assert_ne!(
        (game.party[2].word(0x96), game.party[2].word(0x98)),
        game.party[2].position()
    );
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn real_street_companion_completes_a_learned_spell_without_fire_input() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0, 3, 10]).unwrap();
    equip_freya(&mut game, &rom);
    let before = game.party[2].position();
    for _ in 0..6000 {
        game.update(&rom, 0).unwrap();
        if game.combat.casting.impacts > 0 || game.defeated {
            break;
        }
    }
    assert!(game.encounters.spawned > 0);
    assert!(
        game.combat.casting.impacts > 0,
        "no companion impact: {} / {:?}",
        game.combat.status,
        game.combat.companion_deferred
    );
    assert_ne!(game.party[2].position(), before);
    eprintln!(
        "Real companion spell completed at tick {}, casts {}, encounters {}",
        game.tick, game.combat.casting.casts, game.encounters.spawned
    );
    game.render(&rom).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn successful_and_failed_pose_entries_both_complete_and_release_control() {
    let rom = rom();
    for successful in [false, true] {
        let mut runner = Runner::load(&rom, 10, 2).unwrap();
        shadowrealm::casting::prepare_result_pose(&rom, &mut runner, successful).unwrap();
        assert_eq!(
            combat_runtime::handler(&runner),
            if successful { 0xafe2 } else { 0xafe0 }
        );
        assert!(combat_runtime::holds_movement(&runner));
        for _ in 0..240 {
            combat_runtime::tick_animation(&rom, &mut runner).unwrap();
            combat_runtime::prepare_actor(&mut runner);
            if !combat_runtime::holds_movement(&runner) {
                break;
            }
        }
        assert!(!combat_runtime::holds_movement(&runner));
    }
}
