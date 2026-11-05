use shadowrealm::{avoidance::State, collision::Collision, combat, game::Game, input};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn equip_heal(game: &mut Game, rom: &[u8]) {
    game.select(rom, 2).unwrap();
    for _ in 0..16 {
        game.cycle_spell(rom).unwrap();
        if game.heal_selected() {
            break;
        }
    }
    assert!(game.heal_selected());
    assert_eq!(
        game.party[2].raw[0x9d], 4,
        "use Freya's original learned force"
    );
}
fn fixture(rom: &[u8]) -> Game {
    let mut game = Game::load(rom, 0, &[0, 3, 10]).unwrap();
    game.encounters.count = 1; // Hold encounter admission for this isolated component fixture.
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    for (i, a) in game.party.iter_mut().enumerate() {
        a.set_position((180 + i as u16 * 20) * 16, 200 * 16);
        a.set_word(0xc, 100);
    }
    game.camera = (100, 100);
    // Isolate Heal control/target tests from other companions' combat decisions.
    game.party[1].set_word(0xb6, 1000);
    equip_heal(&mut game, rom);
    game
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn learned_heal_casts_on_self_and_renders_all_original_effect_frames() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.party[2].raw[0x4c] = 2;
    assert_eq!(combat::target(&game.party[2]).unwrap().0, 2);
    let ammo = game.party[2].raw[0x4e];
    let reserve = game.party[2].raw[0x4f];
    let mut frames = 0u16;
    for _ in 0..500 {
        game.update(&rom, input::A).unwrap();
        if game.party[2].raw[0xb2] == 12 {
            let phase = game.party[2].raw[3];
            if phase < 16 && frames & (1 << phase) == 0 {
                game.render(&rom).unwrap();
                frames |= 1 << phase;
            }
        }
        if game.combat.casting.impacts_by_actor[2] > 0 {
            break;
        }
    }
    assert!(game.combat.casting.casts_by_actor[2] > 0);
    assert!(game.combat.casting.impacts_by_actor[2] > 0);
    assert!(game.party[2].raw[0x4c] > 2);
    assert_eq!(frames, 65535);
    assert_eq!(
        (game.party[2].raw[0x4e], game.party[2].raw[0x4f]),
        (ammo, reserve)
    );
    assert_eq!(game.combat.shots, 0);
    game.render(&rom).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn party_targeting_heals_a_fallen_runner_and_restores_control() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.party[0].raw[0x4c] = 0;
    game.party[0].raw[0x9c] = 0;
    game.party[0].raw[0x10] |= 0x50;
    game.party[0].raw[0x3a..0x3e].copy_from_slice(&0xb38au32.to_be_bytes());
    game.cycle_party_target();
    assert_eq!(combat::target(&game.party[2]).unwrap().0, 0);
    assert_eq!(game.selected_target().unwrap().word(0x3e), 0);
    for _ in 0..500 {
        game.update(&rom, input::A).unwrap();
        assert_eq!(
            combat::target(&game.party[2]).unwrap().0,
            0,
            "NPC targeting must not replace Heal's party pointer"
        );
        if game.combat.casting.impacts_by_actor[2] > 0 {
            break;
        }
    }
    assert!(game.combat.casting.impacts_by_actor[2] > 0);
    assert!(game.party[0].raw[0x4c] > 0);
    assert_eq!(game.party[0].raw[0x9c], 1);
    assert_eq!(game.party[0].raw[0x10] & 0x50, 0);
    assert_ne!(game.party[0].raw[0x10] & 1, 0);
    assert!(!game.defeated);
    game.select(&rom, 0).unwrap();
    let before = game.party[0].position();
    for _ in 0..12 {
        game.update(&rom, input::RIGHT).unwrap();
    }
    assert_ne!(game.party[0].position(), before);
    game.render(&rom).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn healing_rejects_npc_targets_without_spending_rng_or_resources() {
    let rom = rom();
    let mut game = fixture(&rom);
    let mut actors = game.party.clone();
    let mut npc = actors[0].clone();
    npc.set_word(0x3e, 3);
    npc.raw[0x10] = 5;
    actors.push(npc);
    actors[2].raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
    let before = actors.iter().map(|a| a.raw).collect::<Vec<_>>();
    let mut rng = State { seed: 123 };
    assert!(!game
        .combat
        .cast_actor(&rom, &game.collision, &mut actors, 2, &mut rng)
        .unwrap());
    assert_eq!(rng.seed, 123);
    assert_eq!(actors.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    assert_eq!(game.combat.casting.live, 0);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn unsupported_heal_callback_travel_rolls_back_the_whole_cast() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.collision.cells.fill(0xc5);
    let mut actors = game.party.clone();
    let before = actors.iter().map(|a| a.raw).collect::<Vec<_>>();
    let mut rng = State { seed: 1 };
    let graphics = game.combat.casting.graphics;
    assert!(!game
        .combat
        .cast_actor(&rom, &game.collision, &mut actors, 2, &mut rng)
        .unwrap());
    assert_eq!(rng.seed, 1);
    assert_eq!(actors.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    assert_eq!(game.combat.casting.graphics, graphics);
    assert_eq!(game.combat.casting.live, 0);
    assert_eq!(game.combat.launch.engaged, 0);
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn real_street_injury_can_be_healed_through_known_spell_and_party_controls() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0, 3, 10]).unwrap();
    equip_heal(&mut game, &rom);
    let mut injured = None;
    let mut completed = false;
    for _ in 0..12000 {
        if injured.is_none() {
            if let Some(index) = game.party.iter().position(|a| a.raw[0x4c] < 10) {
                injured = Some(index);
                for _ in 0..3 {
                    if combat::target(&game.party[2]).is_some_and(|id| usize::from(id.0) == index) {
                        break;
                    }
                    game.cycle_party_target();
                }
            }
        }
        let before = injured.map(|index| game.party[index].raw[0x4c]);
        game.update(&rom, if injured.is_some() { input::A } else { 0 })
            .unwrap();
        if game.combat.casting.impacts_by_actor[2] > 0 {
            let index = injured.unwrap();
            assert!(
                game.party[index].raw[0x4c] > before.unwrap(),
                "a real Heal completion must restore health"
            );
            eprintln!("Real street recovery at tick {}, target {}, health {} -> {}, encounters {}, casts {}",game.tick,index,before.unwrap(),game.party[index].raw[0x4c],game.encounters.spawned,game.combat.casting.casts_by_actor[2]);
            completed = true;
            break;
        }
        if game.defeated {
            break;
        }
    }
    assert!(game.encounters.spawned > 0);
    assert!(injured.is_some());
    assert!(
        completed,
        "real encounter injury did not reach Heal: {}",
        game.combat.status
    );
    game.render(&rom).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn already_equipped_companion_prioritizes_a_critical_party_member_without_fire_input() {
    let rom = rom();
    let mut game = fixture(&rom);
    game.select(&rom, 0).unwrap();
    game.party[0].raw[0x4c] = 1;
    // The original AI requires an enemy combat target before redirecting Heal.
    let mut enemy = shadowrealm::party::Runner::load(&rom, 0, 0).unwrap();
    enemy.set_word(0x3e, 3);
    enemy.set_position(300 * 16, 210 * 16);
    enemy.raw[0x10] = 5;
    enemy.raw[0x56] = 127;
    enemy.raw[0x57] = 0;
    enemy.raw[0x9b] = 2;
    enemy.raw[0x9e] = 0;
    enemy.raw[0xb0] = 0;
    enemy.raw[0xb2] = 0;
    enemy.raw[0xdf] = 0;
    enemy.set_word(0xc, 100);
    game.npcs.push(enemy);
    game.combat.launch.engaged = 65535;
    game.combat.launch.timeout = 120;
    for _ in 0..400 {
        game.update(&rom, 0).unwrap();
        if game.combat.casting.impacts_by_actor[2] > 0 {
            break;
        }
    }
    assert!(game.combat.casting.casts_by_actor[2] > 0);
    assert!(game.combat.casting.impacts_by_actor[2] > 0);
    assert!(game.party[0].raw[0x4c] > 1);
    assert_eq!(game.combat.shots, 0);
}
