use shadowrealm::{
    avoidance::State,
    game::Game,
    spellbooks::{self, Selection, Spellbooks},
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn original_spellbook_initialization_classification_and_map_clear_preserve_session_state() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
    let books = game.combat.spellbooks.as_mut().unwrap();
    assert_eq!(
        books.records.iter().flatten().copied().collect::<Vec<_>>(),
        rom[0x166ad8..0x166ad8 + 170]
    );
    let (attacks, heal) = books.candidates(0).unwrap();
    assert_eq!(
        attacks.iter().map(|s| (s.id, s.force)).collect::<Vec<_>>(),
        [(1, 1)]
    );
    assert!(heal.is_none());
    books.records[0][3] = 7;
    let before = books.clone();
    game.combat.clear_map();
    assert_eq!(game.combat.spellbooks.as_ref().unwrap(), &before);
    assert!(game
        .combat
        .spellbooks
        .as_ref()
        .unwrap()
        .candidates(5)
        .is_err());
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn default_magic_companion_selection_defers_casting_without_spending_rng_or_ammunition() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
    let mut actors = game.party.clone();
    for a in &mut actors {
        a.set_word(0xc, 100);
        a.raw[0x4c] = 10;
    }
    actors[1].set_word(0xb6, 0);
    actors[1].set_word(0xdc, 0xffff);
    actors[1].raw[0x80] = 20;
    let books = Spellbooks::load(&rom).unwrap();
    let mut rng = State { seed: 321 };
    let (selection, _) = spellbooks::select(&rom, &books, &actors[1], &actors, &mut rng).unwrap();
    assert!(matches!(selection, Selection::Spell(_)));
    let ammo = actors[1].raw[0x4e];
    let before = actors[1].raw;
    let mut rng = State { seed: 321 };
    game.combat.launch.engaged = 0xffff;
    game.combat
        .companion(
            &rom,
            &game.collision,
            &mut actors,
            1,
            &mut false,
            1,
            &mut rng,
        )
        .unwrap();
    assert!(game.combat.companion_deferred[1]
        .unwrap()
        .contains("casting"));
    assert_eq!(rng.seed, 321);
    assert_eq!(actors[1].raw[0x4e], ammo);
    assert!(!game.combat.has_live());
    let mut expected = before;
    expected[0xb6..0xb8].copy_from_slice(&20u16.to_be_bytes());
    assert_eq!(actors[1].raw, expected);
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn spell_equipment_round_trip_and_invalid_conversion_are_transactional() {
    use shadowrealm::{party::Runner, spell_equipment, spellbooks::Spell};
    let rom = rom();
    let mut a = Runner::load(&rom, 0, 1).unwrap();
    a.raw[0x10] = 13;
    a.raw[0x56] = 0;
    a.raw[0x57] = 2;
    a.raw[0x4e] = 6;
    a.raw[0x66] = 2;
    spell_equipment::equip_spell(&rom, &mut a, Spell { id: 1, force: 2 }, 1).unwrap();
    assert_eq!(
        (a.raw[0x56], a.raw[0x57], a.raw[0x4e], a.raw[0x9d]),
        (255, 2, 50, 2)
    );
    spell_equipment::equip_inventory(&rom, &mut a).unwrap();
    assert_eq!(
        (a.raw[0x56], a.raw[0x57], a.raw[0x4e], a.raw[0x9d]),
        (0, 2, 6, 0)
    );
    let before = a.raw;
    assert!(spell_equipment::equip_spell(&rom, &mut a, Spell { id: 1, force: 2 }, 65535).is_err());
    assert_eq!(a.raw, before);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn unsafe_current_spell_falls_back_to_native_firearm_with_restored_ammunition() {
    use shadowrealm::{collision::Collision, combat_runtime::Runtime, party::Runner};
    let rom = rom();
    let mut actors = (0..4)
        .map(|slot| {
            let mut a = Runner::load(&rom, 0, slot % 3).unwrap();
            a.set_word(0x3e, slot as u16);
            a.raw[0x10] = [15, 13, 0, 5][slot];
            a.set_word(0xc, 100);
            a.raw[0x4c] = 10;
            a
        })
        .collect::<Vec<_>>();
    actors[0].set_position(15000, 16000);
    actors[3].set_position(17000, 16000);
    let a = &mut actors[1];
    a.set_position(16000, 16000);
    a.set_word(0xb6, 0);
    a.set_word(0xb8, 0x1000);
    a.set_word(0xdc, 65535);
    a.set_word(0x16, 0);
    a.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    a.raw[0x9b] = 0;
    a.raw[0x9c] = 1;
    a.raw[0x56] = 255;
    a.raw[0x57] = 1;
    a.raw[0x9d] = 4;
    a.raw[0x4e] = 50;
    a.raw[0x66] = 2;
    a.raw[0xb0] = 0;
    a.raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
    let mut runtime = Runtime::default();
    runtime.spellbooks = Some(Spellbooks::load(&rom).unwrap());
    runtime.launch.engaged = 65535;
    let c = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    runtime
        .companion(
            &rom,
            &c,
            &mut actors,
            1,
            &mut false,
            1,
            &mut State { seed: 123 },
        )
        .unwrap();
    assert_eq!(
        (
            actors[1].raw[0x56],
            actors[1].raw[0x57],
            actors[1].raw[0x4e],
            actors[1].raw[0x9d]
        ),
        (0, 2, 6, 0)
    );
    assert!(runtime.companion_deferred[1].is_none());
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn actual_drain_consumes_foci_then_spills_into_health_and_restores_minimum_condition() {
    use shadowrealm::{party::Runner, spell_equipment, spellbooks::Spell};
    let rom = rom();
    let mut a = Runner::load(&rom, 0, 0).unwrap();
    a.raw[0x10] = 15;
    a.raw[0x66..0x76].fill(0);
    a.raw[0x80] = 0;
    a.raw[0x7c] = 0;
    a.raw[0x4d] = 2;
    a.raw[0x4c] = 10;
    a.raw[0x9c] = 1;
    a.raw[0x66 + 6] = 0x13;
    a.raw[0x6e + 6] = 1;
    spell_equipment::apply_drain(&rom, &mut a, Spell { id: 2, force: 6 }).unwrap();
    assert_eq!(a.raw[0x66 + 6], 0);
    assert_eq!(a.raw[0x6e + 6], 0);
    assert!(a.raw[0x4c] < 10);
    assert_eq!(a.raw[0x9c], 1);
}
