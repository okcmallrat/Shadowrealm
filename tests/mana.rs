use shadowrealm::{
    avoidance::State, casting, collision::Collision, combat_runtime, damage, game::Game, input,
    party::Runner,
};

fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn equip(game: &mut Game, rom: &[u8], slot: usize, item: u8) {
    game.select(rom, slot).unwrap();
    for _ in 0..16 {
        game.cycle_spell(rom).unwrap();
        if game.party[slot].raw[0x57] == item {
            break;
        }
    }
    assert_eq!(game.party[slot].raw[0x56], 255);
    assert_eq!(game.party[slot].raw[0x57], item);
    assert_eq!(game.party[slot].raw[0x9d], 3);
    let book = game.combat.spellbooks.as_ref().unwrap();
    assert!(book
        .known(game.party[slot].word(0xa0))
        .unwrap()
        .iter()
        .any(|s| s.id + 1 == item && s.force == game.party[slot].raw[0x9d]));
}
fn fixture(rom: &[u8], item: u8, health: u8) -> Game {
    let roster = if item == 8 { [2, 3, 10] } else { [0, 3, 10] };
    let mut game = Game::load(rom, 0, &roster).unwrap();
    game.encounters.count = 1; // Isolated component fixture; hold random encounters.
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    game.camera = (100, 100);
    for (i, a) in game.party.iter_mut().enumerate() {
        a.set_position((160 + i as u16 * 20) * 16, 200 * 16);
        a.set_word(0xc, 100);
    }
    equip(&mut game, rom, if item == 8 { 0 } else { 2 }, item);
    let mut target = Runner::load(rom, 0, 0).unwrap();
    target.set_word(0x3e, 3);
    target.set_position(280 * 16, 200 * 16);
    target.raw[0x10] = 5;
    target.raw[0x56] = 127;
    target.raw[0x57] = 0;
    target.raw[0x9b] = 2;
    target.raw[0x9e] = 0;
    target.raw[0xb0] = 0;
    target.raw[0xb2] = 0;
    target.raw[0xdf] = 0;
    target.raw[0x4c] = health;
    target.raw[0x7c] = 0;
    target.set_word(0xc, 100);
    game.npcs.push(target);
    game
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn learned_mana_spells_render_every_frame_and_finish_the_original_defeat_pose() {
    let rom = rom();
    for item in [8, 9] {
        for health in [1, 10] {
            let mut game = fixture(&rom, item, health);
            let slot = game.selected;
            let resources = (game.party[slot].raw[0x4e], game.party[slot].raw[0x4f]);
            game.update(&rom, input::C).unwrap();
            let mut frames = 0u8;
            let count = if item == 8 { 3 } else { 6 };
            for _ in 0..500 {
                game.update(&rom, input::A).unwrap();
                if game.npcs[0].raw[0xb2] == item {
                    let frame = game.npcs[0].raw[3];
                    if frame < count && frames & (1 << frame) == 0 {
                        game.render(&rom).unwrap();
                        frames |= 1 << frame;
                    }
                }
                if game.combat.casting.impacts > 0 {
                    break;
                }
            }
            assert!(
                game.combat.casting.impacts > 0,
                "item {item}: {}",
                game.combat.status
            );
            assert_eq!(frames, (1 << count) - 1);
            assert!(game.npcs[0].raw[0x4c] < health);
            assert_eq!(game.combat.shots, 0);
            assert_eq!(
                (game.party[slot].raw[0x4e], game.party[slot].raw[0x4f]),
                resources
            );
            assert_eq!(game.combat.casting.live, 0);
            assert!(!game.combat.casting.graphics[usize::from(game.npcs[0].word(0x32))]);
            if health == 1 {
                assert_eq!(combat_runtime::handler(&game.npcs[0]), 0xb10c);
                assert_eq!(game.npcs[0].word(0x16), 0xe4);
                for _ in 0..240 {
                    game.update(&rom, 0).unwrap();
                    if game.npcs[0].raw[0x10] & 0x40 != 0 {
                        break;
                    }
                }
                assert_ne!(game.npcs[0].raw[0x10] & 0x40, 0, "mana defeat must finish");
            }
            game.render(&rom).unwrap();
        }
    }
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn real_street_encounter_accepts_freyas_learned_mana_blast() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0, 3, 10]).unwrap();
    equip(&mut game, &rom, 2, 9);
    let mut targeted = false;
    for _ in 0..6000 {
        let visible = game
            .npcs
            .iter()
            .any(|a| a.raw[0x10] & 0x41 == 1 && a.word(0xc) != 0 && a.raw[0xb0] & 0x80 == 0);
        let mut buttons = if targeted { input::A } else { 0 };
        if visible && !targeted {
            buttons |= input::C;
            targeted = true;
        }
        game.update(&rom, buttons).unwrap();
        if game.combat.casting.impacts > 0 {
            break;
        }
    }
    assert!(game.encounters.spawned > 0);
    assert!(game.combat.casting.impacts > 0, "{}", game.combat.status);
    eprintln!(
        "Real Mana Blast completion at tick {}, casts {}, encounters {}",
        game.tick, game.combat.casting.casts, game.encounters.spawned
    );
    game.render(&rom).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn mana_resistance_uses_willpower_and_preserves_the_original_reaction_difference() {
    let rom = rom();
    for item in [8, 9] {
        let game = fixture(&rom, item, 10);
        let mut actors = game.party.clone();
        actors.push(game.npcs[0].clone());
        let slot = game.selected;
        actors[slot].raw[0x9f] = 0;
        actors[3].raw[0x77] = 1;
        actors[3].raw[0x7c] = 40;
        let mut altered = actors.clone();
        altered[3].raw[0x77] = 30;
        let mut rng = State { seed: 1 };
        let mut other_rng = State { seed: 1 };
        let first = damage::resolve_magic(
            &rom,
            &mut actors,
            slot as u8,
            3,
            &mut Default::default(),
            &mut rng,
        )
        .unwrap();
        let second = damage::resolve_magic(
            &rom,
            &mut altered,
            slot as u8,
            3,
            &mut Default::default(),
            &mut other_rng,
        )
        .unwrap();
        assert_eq!(first.pool, 40);
        assert_eq!(first, second, "body must not change mana resistance");
        assert_eq!(rng.seed, other_rng.seed);
        assert_eq!(actors[3].raw[0x4c], 10);

        let mut incoming = actors.clone();
        incoming[3].raw[0x57] = item;
        incoming[3].raw[0x9d] = 3;
        incoming[3].raw[0x9f] = 0;
        incoming[0].raw[0x77] = 1;
        incoming[0].raw[0x7c] = 40;
        let defense: u16 = incoming[..3]
            .iter()
            .filter(|a| (a.raw[0x9b] as i8) <= 1)
            .map(|a| {
                let m = casting::magic_attribute(a);
                u16::from(m / 4) + u16::from(m % 4 >= 3)
            })
            .sum();
        let resisted = damage::resolve_magic(
            &rom,
            &mut incoming,
            3,
            0,
            &mut Default::default(),
            &mut State { seed: 1 },
        )
        .unwrap();
        assert_eq!(
            resisted.pool,
            defense + 1,
            "party defense leaves the original D7 focus-scan index; this selects body"
        );

        actors[3].raw[0xb2] = item;
        actors[3].raw[2] = 0;
        actors[3].raw[3] = if item == 8 { 2 } else { 5 };
        actors[3].raw[0xd8..0xdc].copy_from_slice(&(0xff0100 + slot as u32 * 256).to_be_bytes());
        actors[3].raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
        actors[3].set_word(0x32, 17);
        let mut runtime = casting::Runtime::default();
        runtime.graphics[17] = true;
        runtime.live = 1;
        runtime
            .advance(
                &rom,
                &mut actors,
                &mut Default::default(),
                &mut State { seed: 1 },
            )
            .unwrap();
        assert_eq!(actors[3].raw[0x4c], 10);
        assert_eq!(
            combat_runtime::handler(&actors[3]),
            if item == 8 { 0xb07e } else { 0xb45c }
        );
        assert_eq!(runtime.live, 0);
        assert!(!runtime.graphics[17]);
    }
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn equipped_mana_companion_can_complete_an_attack_without_player_fire_input() {
    let rom = rom();
    let mut game = fixture(&rom, 9, 10);
    game.select(&rom, 0).unwrap();
    game.combat.launch.engaged = 65535;
    game.combat.launch.timeout = 120;
    for _ in 0..500 {
        game.update(&rom, 0).unwrap();
        if game.combat.casting.impacts > 0 {
            break;
        }
    }
    assert!(game.combat.casting.casts > 0);
    assert!(game.combat.casting.impacts > 0);
    assert!(game.npcs[0].raw[0x4c] < 10);
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn casting_catalog_rejects_changed_mana_timing_palette_and_completion_rules() {
    let rom = rom();
    casting::Catalog::load(&rom).unwrap();
    for (key, replacement) in [
        ("period", serde_json::json!(2)),
        ("palette", serde_json::json!(8192)),
        ("reaction", serde_json::json!("allocation")),
        ("clear_marker", serde_json::json!(false)),
        ("defeat", serde_json::json!("0xb0ac")),
        ("initializer", serde_json::json!("0xb574")),
    ] {
        let mut data: serde_json::Value = serde_json::from_str(casting::Catalog::JSON).unwrap();
        data["effects"][3][key] = replacement;
        assert!(
            casting::Catalog::parse(&data.to_string(), &rom).is_err(),
            "{key}"
        );
    }
}
