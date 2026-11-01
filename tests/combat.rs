use shadowrealm::{
    combat::{self, ActorId},
    game::Game,
    input, npc,
};

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
