use shadowrealm::{avoidance, collision::Contact, game::Game, party::Runner};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var_os("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM"))
        .unwrap()
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn overlapping_party_separates_with_each_selected_slot() {
    let rom = rom();
    for selected in 0..3 {
        let mut game = Game::load(&rom, 0, &[0, 3, 5]).unwrap();
        game.select(&rom, selected).unwrap();
        for (i, r) in game.party.iter_mut().enumerate() {
            r.set_position(528 * 16 + i as u16 * 8, 160 * 16 + i as u16 * 8);
        }
        let initial: Vec<_> = game.party.iter().map(|r| r.position()).collect();
        let wallet = game.nuyen;
        let stats: Vec<_> = game
            .party
            .iter()
            .map(|r| r.raw[0x4c..0x92].to_vec())
            .collect();
        assert!(avoidance::overlapping(&game.party[0], &game.party[1]));
        for _ in 0..120 {
            game.update(&rom, 0).unwrap();
            for r in &game.party {
                assert_eq!(
                    game.collision.contact(r.position()).unwrap(),
                    Contact::Clear
                );
            }
        }
        assert!(game
            .party
            .iter()
            .zip(initial)
            .any(|(r, p)| r.position() != p));
        for a in 0..3 {
            for b in a + 1..3 {
                assert!(
                    !avoidance::overlapping(&game.party[a], &game.party[b]),
                    "selected {selected}, pair {a}/{b}"
                );
            }
        }
        for (r, stat) in game.party.iter().zip(stats) {
            assert_eq!(&r.raw[0x4c..0x92], stat);
        }
        assert_eq!(game.nuyen, wallet);
        assert_eq!(game.selected, selected);
        assert_eq!(game.completed_transitions, 0);
    }
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn idle_npc_receives_contact_and_restores_its_behavior() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0]).unwrap();
    game.party[0].set_position(528 * 16, 160 * 16);
    let mut npc = Runner::load(&rom, 5, 1).unwrap();
    npc.set_word(0x3e, 3);
    npc.raw[0x10] = 5;
    npc.raw[0x9e] = 0; // physical contact fixture has no dialogue script
    npc.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    npc.set_position(528 * 16, 160 * 16);
    let start = npc.position();
    game.npcs.push(npc);
    for _ in 0..20 {
        game.update(&rom, 0).unwrap();
    }
    assert_ne!(game.npcs[0].position(), start);
    assert!(!avoidance::overlapping(&game.party[0], &game.npcs[0]));
    assert_eq!(&game.npcs[0].raw[0x3a..0x3e], &0xb45cu32.to_be_bytes());
    assert_eq!(game.npcs[0].raw[0x11] & 2, 0);
    assert_eq!(
        game.collision.contact(game.npcs[0].position()).unwrap(),
        Contact::Clear
    );
    assert!(game.dialogue.is_none());
}

fn contact_edge(game: &Game, rom: &[u8], wanted: u8) -> ((u16, u16), u16, (u16, u16)) {
    for r in game
        .collision
        .rectangles
        .iter()
        .filter(|r| r.code == wanted)
    {
        let left = r.left.saturating_sub(14);
        let right = r.right.saturating_add(14).min(game.collision.bounds[2] - 1);
        let top = r.top.saturating_sub(9);
        let bottom = r.bottom.saturating_add(9).min(game.collision.bounds[3] - 1);
        for y in top..=bottom {
            for x in left..=right {
                let p = (x * 16, y * 16);
                if game.collision.contact(p).unwrap() != Contact::Clear {
                    continue;
                }
                for direction in 0..8 {
                    let base = 0xbf82 + direction * 4;
                    let dx = u16::from_be_bytes(rom[base..base + 2].try_into().unwrap());
                    let dy = u16::from_be_bytes(rom[base + 2..base + 4].try_into().unwrap());
                    let q = (p.0.wrapping_add(dx), p.1.wrapping_add(dy));
                    if game.collision.contact(q).unwrap() == Contact::Trigger(wanted)
                        && game.collision.trigger_corners(q) >= 2
                    {
                        return (p, (direction as u16).rotate_right(3), q);
                    }
                }
            }
        }
    }
    panic!("no clear contact edge for {wanted:x}");
}
fn place_for_contact(game: &mut Game, rom: &[u8], code: u8) -> (u16, u16) {
    // An odd phase tests the normal desired direction; both actors genuinely
    // overlap so Game::update supplies the pending contact, without input.
    if game.tick & 1 != 0 {
        game.update(rom, 0).unwrap();
    }
    let (p, angle, q) = contact_edge(game, rom, code);
    for r in &mut game.party {
        r.set_position(p.0, p.1);
    }
    game.party[game.selected].set_word(0x34, angle);
    game.camera = (
        (p.0 >> 4).saturating_sub(136),
        (p.1 >> 4).saturating_sub(112),
    );
    q
}
fn finish_contact_transition(game: &mut Game, rom: &[u8], target: usize) {
    let before = game.completed_transitions;
    for _ in 0..200 {
        game.update(rom, 0).unwrap();
        if game.completed_transitions > before && game.transition.is_none() {
            assert_eq!(game.map_id(), target);
            return;
        }
    }
    panic!("contact transition did not finish");
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn contact_can_activate_supported_exterior_seams_with_a_switched_leader() {
    let rom = rom();
    for map in [0, 3, 5] {
        let mut game = Game::load(&rom, map, &[0, 3]).unwrap();
        game.select(&rom, 1).unwrap();
        let code = game
            .collision
            .rectangles
            .iter()
            .find(|r| r.code & 31 == 16 && r.code >= 128)
            .unwrap()
            .code;
        let q = place_for_contact(&mut game, &rom, code);
        let expected =
            shadowrealm::transitions::TransitionPlan::exterior(&rom, map, code, game.camera.0 * 16)
                .unwrap()
                .unwrap();
        let wallet = game.nuyen;
        game.update(&rom, 0).unwrap();
        assert_eq!(game.party[1].position(), q);
        assert!(game.transition.is_some());
        assert!(game.dialogue.is_none());
        finish_contact_transition(&mut game, &rom, expected.target_map);
        assert_eq!(game.selected, 1);
        assert_eq!(game.nuyen, wallet);
        for (r, p) in game.party.iter().zip(expected.positions) {
            assert_eq!(r.position(), p);
        }
    }
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn contact_space_needle_entry_acknowledgement_and_exit_preserve_return_context() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3]).unwrap();
    let q = place_for_contact(&mut game, &rom, 0xc6);
    let samples = game.collision.footprint_samples(q);
    let orientation = samples.iter().find(|v| **v >= 128).unwrap().rotate_left(3) & 3;
    let expected = shadowrealm::transitions::TransitionPlan::space_needle_from_contact(
        &rom,
        &shadowrealm::maps::MapDescriptor::read(&rom, 0).unwrap(),
        0xc6,
        orientation,
        (game.camera.0 * 16, game.camera.1 * 16),
        q,
    )
    .unwrap()
    .unwrap()
    .exterior_return
    .unwrap();
    let wallet = game.nuyen;
    game.update(&rom, 0).unwrap();
    assert_eq!(game.party[0].position(), q);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0x110b5e);
    assert!(game.transition.is_none());
    game.update(&rom, shadowrealm::input::A).unwrap();
    finish_contact_transition(&mut game, &rom, 10);
    assert_eq!(game.saved_exterior, Some(expected));
    assert_eq!(
        game.party[0].raw[0xb0] & 0x20,
        0,
        "clear warm-up must release the trigger latch"
    );
    let exit = place_for_contact(&mut game, &rom, 0x80);
    game.update(&rom, 0).unwrap();
    assert_eq!(game.party[0].position(), exit);
    assert!(game.transition.is_some());
    finish_contact_transition(&mut game, &rom, 0);
    assert_eq!(game.saved_exterior, None);
    assert_eq!(game.nuyen, wallet);
    for (r, p) in game.party.iter().zip(expected.positions) {
        assert_eq!(r.position(), p);
    }
}
