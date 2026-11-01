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
