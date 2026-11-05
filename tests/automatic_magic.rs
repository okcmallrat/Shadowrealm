use shadowrealm::{
    avoidance::State, collision::Collision, combat_runtime::Runtime, companion_magic, game::Game,
    party::Runner, spellbooks::Spellbooks,
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn fixture(rom: &[u8]) -> Game {
    let mut game = Game::load(rom, 0, &[0, 3, 10]).unwrap();
    game.encounters.count = 1;
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    for n in 0..4096 {
        if n / 64 == 0 || n / 64 == 63 || n % 64 == 0 || n % 64 == 63 {
            game.collision.cells[n] = 127;
        }
    }
    game.camera = (100, 100);
    for (i, a) in game.party.iter_mut().enumerate() {
        a.set_position((180 + i as u16 * 20) * 16, 200 * 16);
        a.set_word(0xc, 100);
        a.set_word(0xb6, 0);
        a.set_word(0xdc, 65535);
    }
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
    target.raw[0x77] = 10;
    target.raw[0x7c] = 10;
    target.set_word(0xc, 100);
    game.npcs.push(target);
    game.combat.launch.engaged = 65535;
    game.combat.launch.timeout = 3000;
    game
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn default_party_chooses_and_completes_magic_in_a_real_encounter_without_shortcuts() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0, 3, 10]).unwrap();
    let mut first_impact = None;
    let mut defeated_enemy = false;
    for _ in 0..10000 {
        game.update(&rom, 0).unwrap();
        if game.combat.casting.impacts_by_actor[1] + game.combat.casting.impacts_by_actor[2] > 0 {
            first_impact.get_or_insert(game.tick);
        }
        defeated_enemy |= game
            .npcs
            .iter()
            .any(|a| a.raw[0x4c] == 0 && a.raw[0x10] & 0x40 != 0);
        if (game.combat.casting.impacts_by_actor[1] > 0
            && game.combat.casting.impacts_by_actor[2] > 0
            && defeated_enemy)
            || game.defeated
        {
            break;
        }
    }
    assert!(game.encounters.spawned > 0);
    assert!(
        game.combat.companion_spell_changes > 0,
        "no automatic equipment change: {:?}",
        game.combat.companion_deferred
    );
    assert!(
        game.combat.casting.impacts_by_actor[1] > 0 && game.combat.casting.impacts_by_actor[2] > 0,
        "both companions must complete automatic magic: {:?}",
        game.combat.companion_deferred
    );
    assert!(
        defeated_enemy,
        "automatic combat must finish an enemy defeat"
    );
    assert!(!game.defeated);
    assert_eq!(
        game.combat.shots, 0,
        "no controlled-player attack input was supplied"
    );
    eprintln!("Automatic real encounter: first impact {:?}, completed enemy defeat and both casters at tick {}, spell changes {}, casts {:?}, impacts {:?}, encounters {}",first_impact,game.tick,game.combat.companion_spell_changes,game.combat.casting.casts_by_actor,game.combat.casting.impacts_by_actor,game.encounters.spawned);
    game.render(&rom).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn freya_automatically_equips_heal_and_uses_the_original_critical_party_priority() {
    let rom = rom();
    for target in 0..3 {
        let mut game = fixture(&rom);
        game.party[target].raw[0x4c] = 2;
        let mut saw_heal = false;
        for _ in 0..1800 {
            game.update(&rom, 0).unwrap();
            saw_heal |= game.party[target].raw[0xb2] == 12;
            if game.party[target].raw[0x4c] > 2 {
                break;
            }
        }
        assert!(
            saw_heal,
            "target {target}, no Heal effect: {:?}",
            game.combat.companion_deferred
        );
        assert!(
            game.party[target].raw[0x4c] > 2,
            "target {target} was not healed"
        );
        assert!(game.combat.casting.impacts_by_actor[2] > 0);
        assert!(game.combat.companion_spell_changes > 0);
        assert_eq!(game.combat.healing_hint, target as u8);
        game.render(&rom).unwrap();
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn blocked_enemy_probe_precedes_heal_redirection_and_requires_original_approach() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.party[0].raw[0x4c] = 1;
    for row in 5..7 {
        game.collision.cells[row * 64 + 8] = 127;
    }
    let before = game.party[2].position();
    let mut moved_before_heal = false;
    for _ in 0..2200 {
        game.update(&rom, 0).unwrap();
        if game.combat.casting.casts_by_actor[2] == 0 {
            moved_before_heal |= game.party[2].position() != before;
        }
        if game.party[0].raw[0x4c] > 1 {
            break;
        }
    }
    assert!(moved_before_heal);
    assert!(
        game.party[0].raw[0x4c] > 1,
        "Heal did not complete after enemy-path recovery: {:?}",
        game.combat.companion_deferred
    );
    assert!(game.combat.casting.impacts_by_actor[2] > 0);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn missing_or_outside_rom_conversion_context_keeps_source_and_rng_staged() {
    let rom = rom();
    let game = fixture(&rom);
    for context in [None, Some(65535)] {
        let mut actors = game.party.clone();
        actors.push(game.npcs[0].clone());
        let before = actors[1].raw;
        let mut runtime = Runtime::default();
        runtime.spellbooks = Some(Spellbooks::load(&rom).unwrap());
        runtime.launch.engaged = 65535;
        runtime.companion_capacity_index = context;
        let mut rng = State { seed: 321 };
        runtime
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
        let mut expected = before;
        expected[0xb6..0xb8].copy_from_slice(&20u16.to_be_bytes());
        assert_eq!(actors[1].raw, expected);
        assert_eq!(rng.seed, 321);
        assert_eq!(runtime.companion_spell_changes, 0);
        assert!(!runtime.has_live());
        assert!(runtime.companion_deferred[1].is_some());
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn prior_actor_context_and_json_reject_special_or_unverified_dispatches() {
    let rom = rom();
    companion_magic::Catalog::load(&rom).unwrap();
    let game = fixture(&rom);
    let mut actor = game.party[0].clone();
    actor.set_position(3200, 4000);
    actor.raw[0xb0] = 0;
    actor.raw[0x9e] = 0;
    actor.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    assert_eq!(
        companion_magic::after_actor(&game.collision, &actor),
        Some(200)
    );
    actor.raw[0x3a..0x3e].copy_from_slice(&0xb574u32.to_be_bytes());
    assert_eq!(
        companion_magic::after_actor(&game.collision, &actor),
        Some(3200)
    );
    actor.raw[0x3a..0x3e].copy_from_slice(&0xb49cu32.to_be_bytes());
    assert_eq!(companion_magic::after_actor(&game.collision, &actor), None);
    assert!(companion_magic::Catalog::parse(
        &companion_magic::Catalog::JSON.replace("0xac8e", "0xac8c"),
        &rom
    )
    .is_err());
    assert!(companion_magic::Catalog::parse(
        &companion_magic::Catalog::JSON.replacen('{', "{\"unknown\":1,", 1),
        &rom
    )
    .is_err());
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn automatic_spell_falls_back_to_a_firearm_when_its_original_drain_gate_is_unsafe() {
    let rom = rom();
    let mut game = fixture(&rom);
    for _ in 0..200 {
        game.update(&rom, 0).unwrap();
        if (game.party[1].raw[0x56] as i8) < 0 {
            break;
        }
    }
    assert!((game.party[1].raw[0x56] as i8) < 0);
    // Explicit boundary fixture: force the original unsafe-current-spell branch.
    game.party[1].raw[0x9c] = 1;
    game.party[1].set_word(0xdc, 65535);
    game.party[1].set_word(0xb6, 0);
    for _ in 0..300 {
        game.update(&rom, 0).unwrap();
        if (game.party[1].raw[0x56] as i8) >= 0 {
            break;
        }
    }
    assert!((game.party[1].raw[0x56] as i8) >= 0);
    assert_eq!(game.party[1].raw[0x57], 1);
    assert!(game.combat.companion_deferred[1].is_none());
}
