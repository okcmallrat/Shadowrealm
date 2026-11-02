use shadowrealm::{game::Game, input};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn wounded_player_in_real_encounter_reaches_defeat_and_can_restart() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0]).unwrap();
    game.recovery_flags = 1; // Insured recovery is outside the current clinic subset.
    game.party[0].raw[0x4c] = 1; // Explicit wounded-party fixture; enemies still spawn normally.
    let mut started = None;
    for _ in 0..12_000 {
        game.update(&rom, 0).unwrap();
        if started.is_none() && game.combat.damage.defeat_timer > 0 {
            started = Some(game.tick);
            assert_eq!(game.combat.damage.defeat_timer, 59);
        }
        if game.defeated {
            break;
        }
    }
    assert!(game.defeated, "real encounter did not reach defeat");
    assert_eq!(game.party[0].raw[0x4c], 0);
    assert_eq!(game.tick - started.unwrap(), 59);
    let tick = game.tick;
    let shots = game.combat.npc_shots;
    let actor = game.party[0].raw;
    let balance = game.nuyen;
    let pixels = game.render(&rom).unwrap();
    for _ in 0..120 {
        game.update(&rom, input::A | input::C | input::RIGHT)
            .unwrap();
    }
    assert_eq!(game.tick, tick);
    assert_eq!(game.party[0].raw, actor);
    assert_eq!(game.nuyen, balance);
    assert_eq!(game.combat.npc_shots, shots);
    assert_eq!(game.render(&rom).unwrap(), pixels);
    // The desktop R command loads the same original map/roster/funds configuration.
    let mut retry = Game::load(&rom, 5, &[0]).unwrap();
    assert!(!retry.defeated);
    assert_eq!(retry.combat.damage.defeat_timer, 0);
    assert_eq!(retry.party[0].raw[0x4c], 10);
    assert!(retry.npcs.is_empty());
    retry.update(&rom, input::A).unwrap();
    assert_eq!(retry.combat.shots, 1);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn one_down_member_does_not_end_a_living_party() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3]).unwrap();
    let mut actors = game.party.clone();
    actors[0].raw[0x4c] = 0;
    shadowrealm::damage::react(&rom, &mut actors, 0, &mut game.combat.damage).unwrap();
    game.party = actors;
    assert_eq!(game.combat.damage.defeat_timer, 0);
    for _ in 0..80 {
        game.update(&rom, 0).unwrap();
    }
    assert!(!game.defeated);
    game.select(&rom, 1).unwrap();
    assert_eq!(game.selected, 1);
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn uninsured_solo_defeat_pays_once_and_returns_to_play_in_little_chiba() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0]).unwrap();
    game.party[0].raw[0x4c] = 1;
    game.party[0].raw[0x9c] = 3;
    game.nuyen = 40_001;
    game.knowledge.contacts = 7;
    let ammo = game.party[0].raw[0x4e];
    let equipment = game.party[0].raw[0x56..0x76].to_vec();
    for _ in 0..12_000 {
        game.update(&rom, 0).unwrap();
        if game.defeated {
            break;
        }
    }
    assert!(game.defeated);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xc25bc);
    assert_eq!(game.nuyen, 35_001);
    assert_eq!(game.party[0].raw[0x4c], 6);
    assert_eq!(game.party[0].raw[0x9c], 8);
    assert!(game
        .dialogue
        .as_ref()
        .unwrap()
        .text
        .contains("Little Chiba"));
    let actor = game.party[0].raw;
    let tick = game.tick;
    for _ in 0..30 {
        game.update(&rom, input::RIGHT | input::C).unwrap();
    }
    assert_eq!(game.tick, tick);
    assert_eq!(game.party[0].raw, actor);
    assert_eq!(game.nuyen, 35_001);
    game.render(&rom).unwrap();
    game.update(&rom, input::A).unwrap();
    assert!(!game.defeated);
    assert!(game.transition.is_some());
    for _ in 0..120 {
        if game.transition.is_none() {
            break;
        }
        game.update(&rom, 0).unwrap();
    }
    assert!(game.transition.is_none());
    assert_eq!(game.map_id(), 2);
    assert_eq!(game.party[0].position(), (0x6f00, 0x2780));
    assert_eq!(game.nuyen, 35_001);
    assert_eq!(game.knowledge.contacts, 7);
    assert_eq!(game.party[0].raw[0x4e], ammo);
    assert_eq!(game.party[0].raw[0x56..0x76], equipment);
    assert!(game.npcs.is_empty());
    assert!(!game.combat.has_live());
    assert_eq!(game.encounters.count, 0);
    assert_eq!(game.combat.launch.engaged, 0);
    let origin = game.party[0].position();
    for direction in [input::LEFT, input::UP, input::DOWN, input::RIGHT] {
        game.update(&rom, direction).unwrap();
        if game.party[0].position() != origin {
            break;
        }
    }
    assert_ne!(
        game.party[0].position(),
        origin,
        "clinic arrival cannot move"
    );
    game.update(&rom, input::A).unwrap();
    assert_eq!(game.combat.shots, 1);
    game.render(&rom).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn unsupported_recovery_contexts_keep_the_notice_without_charging() {
    let rom = rom();
    for (templates, flags, mode) in [(vec![0, 3], 0, 0), (vec![0], 1, 0), (vec![0], 0, 3)] {
        let mut game = Game::load(&rom, 0, &templates).unwrap();
        for actor in &mut game.party {
            actor.raw[0x4c] = 0;
            actor.raw[0x10] |= 0x40;
        }
        game.recovery_flags = flags;
        game.combat.launch.mode = mode;
        game.combat.damage.defeat_timer = 1;
        game.nuyen = 3000;
        game.update(&rom, 0).unwrap();
        assert!(game.defeated);
        assert!(game.dialogue.is_none());
        assert_eq!(game.nuyen, 3000);
        game.update(&rom, input::A).unwrap();
        assert!(game.defeated);
        assert!(game.transition.is_none());
    }
}
