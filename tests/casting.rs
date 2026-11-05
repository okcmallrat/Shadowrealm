use shadowrealm::{collision::Collision, game::Game, input, party::Runner};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn equip_bolt(game: &mut Game, rom: &[u8]) {
    game.select(rom, 1).unwrap();
    for _ in 0..16 {
        game.cycle_spell(rom).unwrap();
        if game.party[1].raw[0x57] == 2 {
            break;
        }
    }
    assert_eq!(game.party[1].raw[0x56], 255);
    assert_eq!(game.party[1].raw[0x57], 2);
    assert_eq!(game.party[1].raw[0x9d], 1);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn native_flame_bolt_rolls_drains_animates_and_damages_without_ammunition_loss() {
    let rom = rom();
    for target_health in [1u8, 10] {
        let mut game = Game::load(&rom, 0, &[0, 3]).unwrap();
        equip_bolt(&mut game, &rom);
        let stored_ammo = game.party[1].raw[0x4e];
        let initial_condition = game.party[1].raw[0x9c];
        game.collision = Collision {
            cells: vec![0; 4096],
            rectangles: vec![],
            bounds: [0, 0, 2048, 2048],
            map_width_tiles: 128,
            overflow_writes: vec![],
        };
        game.camera = (100, 100);
        game.party[0].set_position(160 * 16, 210 * 16);
        game.party[1].set_position(200 * 16, 200 * 16);
        let mut target = Runner::load(&rom, 0, 0).unwrap();
        target.set_word(0x3e, 3);
        target.set_position(280 * 16, 200 * 16);
        target.raw[0x10] = 5;
        target.raw[0x57] = 0;
        target.raw[0x56] = 127;
        target.raw[0x9e] = 0;
        target.raw[0xb0] = 0;
        target.raw[0xdf] = 0;
        target.raw[0x9b] = 2;
        target.raw[0x77] = 0;
        target.set_word(0xc, 100);
        target.raw[0x4c] = target_health;
        target.raw[0xb2] = 0;
        game.npcs.push(target);
        game.update(&rom, input::C).unwrap();
        let mut saw_effect = false;
        let mut rendered_frames = 0u8;
        for _ in 0..400 {
            game.update(&rom, input::A).unwrap();
            if game.combat.casting.live > 0 {
                saw_effect = true;
                let phase = game.npcs[0].raw[3];
                if phase < 6 && rendered_frames & (1 << phase) == 0 {
                    game.render(&rom).unwrap();
                    rendered_frames |= 1 << phase;
                }
            }
            if game.combat.casting.impacts_by_actor[1] > 0 {
                break;
            }
        }
        assert!(saw_effect);
        assert!(game.combat.casting.casts_by_actor[1] > 0);
        assert!(game.combat.casting.impacts_by_actor[1] > 0);
        assert!(game.npcs[0].raw[0x4c] < target_health);
        assert_eq!(game.party[1].raw[0x4e], stored_ammo);
        assert!(game.party[1].raw[0x9c] <= initial_condition);
        assert_eq!(game.combat.shots, 0);
        game.render(&rom).unwrap();
        if target_health == 1 {
            for _ in 0..240 {
                game.update(&rom, 0).unwrap();
                if game.npcs[0].raw[0x10] & 0x40 != 0 {
                    break;
                }
            }
            assert_eq!(game.npcs[0].raw[0x4c], 0);
            assert_ne!(
                game.npcs[0].raw[0x10] & 0x40,
                0,
                "spell death pose must finish"
            );
            game.render(&rom).unwrap();
        }
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn equipment_bridge_preserves_ammo_and_known_forces_and_locks_during_effects() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3]).unwrap();
    game.select(&rom, 1).unwrap();
    let ammo = game.party[1].raw[0x4e];
    equip_bolt(&mut game, &rom);
    let book = game
        .combat
        .spellbooks
        .as_ref()
        .unwrap()
        .known(game.party[1].word(0xa0))
        .unwrap();
    assert!(book
        .iter()
        .any(|s| s.id + 1 == game.party[1].raw[0x57] && s.force == game.party[1].raw[0x9d]));
    game.equip_inventory_weapon(&rom).unwrap();
    assert_eq!(game.party[1].raw[0x4e], ammo);
    game.combat.casting.live = 1;
    let before = game.party[1].raw;
    game.cycle_spell(&rom).unwrap();
    assert_eq!(game.party[1].raw, before);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn original_roster_casts_flame_bolt_against_a_real_street_encounter() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0, 3, 10]).unwrap();
    equip_bolt(&mut game, &rom);
    let mut targeted = false;
    for _ in 0..6000 {
        let visible = game
            .npcs
            .iter()
            .any(|a| a.raw[0x10] & 0x41 == 1 && a.word(0xc) != 0 && a.raw[0xb0] & 0x80 == 0);
        let mut buttons = 0;
        if visible && !targeted {
            buttons |= input::C;
            targeted = true;
        }
        if targeted {
            buttons |= input::A;
        }
        game.update(&rom, buttons).unwrap();
        if game.combat.casting.impacts_by_actor[1] > 0 {
            break;
        }
    }
    assert!(game.encounters.spawned > 0);
    assert!(
        game.combat.casting.casts_by_actor[1] > 0,
        "no casts: {}",
        game.combat.status
    );
    assert!(
        game.combat.casting.impacts_by_actor[1] > 0,
        "effect never completed"
    );
    eprintln!(
        "Real street spell at tick {}, casts {}, impacts {}, spawned {}",
        game.tick,
        game.combat.casting.casts_by_actor[1],
        game.combat.casting.impacts_by_actor[1],
        game.encounters.spawned
    );
    game.render(&rom).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn blocked_spell_admission_does_not_spend_rng_drain_or_reserve_a_graphic() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3]).unwrap();
    equip_bolt(&mut game, &rom);
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    for row in 0..64 {
        game.collision.cells[row * 64 + 7] = 0x7f;
    }
    let mut a = game.party.clone();
    a[1].set_position(200 * 16, 200 * 16);
    let mut target = Runner::load(&rom, 0, 0).unwrap();
    target.set_word(0x3e, 3);
    target.set_position(280 * 16, 200 * 16);
    target.raw[0x10] = 5;
    target.raw[0xb2] = 0;
    a[1].raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
    a.push(target);
    let before = a.iter().map(|a| a.raw).collect::<Vec<_>>();
    let mut rng = shadowrealm::avoidance::State { seed: 321 };
    assert!(!game
        .combat
        .cast_actor(&rom, &game.collision, &mut a, 1, &mut rng)
        .unwrap());
    assert_eq!(rng.seed, 321);
    assert_eq!(a.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    assert_eq!(game.combat.casting.live, 0);
    assert_eq!(game.combat.casting.casts_by_actor[1], 0);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn offscreen_target_effect_holds_until_it_is_visible() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3]).unwrap();
    game.party[0].raw[0xb2] = 2;
    game.party[0].raw[2] = 5;
    game.party[0].raw[3] = 0;
    game.party[0].set_word(0xc, 0);
    let before = game.party.iter().map(|a| a.raw).collect::<Vec<_>>();
    let mut rng = shadowrealm::avoidance::State { seed: 1 };
    game.combat
        .casting
        .advance(&rom, &mut game.party, &mut game.combat.damage, &mut rng)
        .unwrap();
    assert_eq!(game.party.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    assert_eq!(rng.seed, 1);
    game.party[0].set_word(0xc, 100);
    game.combat
        .casting
        .advance(&rom, &mut game.party, &mut game.combat.damage, &mut rng)
        .unwrap();
    assert_eq!(game.party[0].raw[2], 4);
}
