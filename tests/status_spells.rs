use shadowrealm::{
    avoidance::State, casting, collision::Collision, damage, game::Game, input, party::Runner,
    status_spells,
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
    assert_eq!(game.party[slot].raw[0x57], item);
    assert!(game
        .combat
        .spellbooks
        .as_ref()
        .unwrap()
        .known(game.party[slot].word(0xa0))
        .unwrap()
        .iter()
        .any(|s| s.id + 1 == item && s.force == game.party[slot].raw[0x9d]));
    assert_eq!(game.party_spell_selected(), matches!(item, 4 | 11 | 12));
}
fn fixture(rom: &[u8], slot: usize, item: u8) -> Game {
    let mut game = Game::load(rom, 0, &[0, 3, 10]).unwrap();
    game.encounters.count = 1;
    game.collision = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    game.camera = (100, 100);
    for (i, a) in game.party.iter_mut().enumerate() {
        a.set_position((180 + i as u16 * 20) * 16, 200 * 16);
        a.set_word(0xc, 100);
    }
    equip(&mut game, rom, slot, item);
    game
}
fn cast_until(game: &mut Game, rom: &[u8], slot: usize, flag: u8) {
    for _ in 0..500 {
        game.update(rom, input::A).unwrap();
        if game.party[slot].raw[0x11] & flag != 0 {
            break;
        }
    }
    assert_ne!(
        game.party[slot].raw[0x11] & flag,
        0,
        "status was not applied: {}",
        game.combat.status
    );
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn learned_invisibility_initializes_expires_and_preserves_equipment_resources() {
    let rom = rom();
    let mut game = fixture(&rom, 2, 4);
    assert_eq!(game.party[2].raw[0x9d], 3);
    let stored = (game.party[2].raw[0x4e], game.party[2].raw[0x4f]);
    cast_until(&mut game, &rom, 2, 0x80);
    assert_eq!(game.party[2].raw[0xb2], 0);
    assert_eq!(game.combat.casting.live, 0);
    assert_eq!((game.party[2].raw[0x4e], game.party[2].raw[0x4f]), stored);
    let timer = game.party[2].raw[0x40];
    assert!(timer > 0);
    let before = game.party.iter().map(|a| a.raw).collect::<Vec<_>>();
    let image = game.render(&rom).unwrap();
    assert_eq!(image, game.render(&rom).unwrap());
    assert_eq!(
        game.party.iter().map(|a| a.raw).collect::<Vec<_>>(),
        before,
        "drawing must not advance status clocks"
    );
    let graphic = usize::from(game.party[2].word(0x32));
    assert!(game.combat.casting.graphics[graphic]);
    for _ in 0..(usize::from(timer) + 2) * 64 {
        game.update(&rom, 0).unwrap();
        if game.party[2].raw[0x11] & 0x80 == 0 {
            break;
        }
    }
    assert_eq!(game.party[2].raw[0x11] & 0x80, 0);
    assert_eq!(game.party[2].raw[0x40], 255);
    assert!(
        game.combat.casting.graphics[graphic],
        "the original instant initializer/expiry does not release this slot"
    );
    game.equip_inventory_weapon(&rom).unwrap();
    game.render(&rom).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn learned_rockskin_targets_a_party_member_and_renders_original_colors() {
    let rom = rom();
    let mut game = fixture(&rom, 1, 11);
    assert_eq!(game.party[1].raw[0x9d], 2);
    for _ in 0..3 {
        if shadowrealm::combat::target(&game.party[1]).is_some_and(|id| id.0 == 0) {
            break;
        }
        game.cycle_party_target();
    }
    assert_eq!(game.selected_target().unwrap().word(0x3e), 0);
    cast_until(&mut game, &rom, 0, 0x20);
    assert_eq!(game.party[0].raw[0xb2], 0);
    assert!(game.party[0].raw[0x41] > 0);
    assert_eq!(game.combat.casting.live, 0);
    assert_eq!(shadowrealm::combat::target(&game.party[1]).unwrap().0, 0);
    game.render(&rom).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn status_clocks_preserve_visibility_hold_and_signed_byte_expiry_rules() {
    let rom = rom();
    let mut a = Runner::load(&rom, 0, 0).unwrap();
    a.raw[0x11] = 0xa0;
    a.raw[0x40] = 0;
    a.raw[0x41] = 128;
    a.set_word(0xc, 0);
    status_spells::tick(std::slice::from_mut(&mut a), 64);
    assert_eq!((a.raw[0x40], a.raw[0x41]), (0, 128));
    a.set_word(0xc, 100);
    a.raw[0x9e] |= 0x40;
    status_spells::tick(std::slice::from_mut(&mut a), 65);
    assert_eq!(a.raw[0x41], 128);
    status_spells::tick(std::slice::from_mut(&mut a), 128);
    assert_eq!(
        (a.raw[0x40], a.raw[0x41], a.raw[0x11] & 0xa0),
        (0, 127, 0xa0)
    );
    a.raw[0x9e] &= !0x40;
    status_spells::tick(std::slice::from_mut(&mut a), 192);
    assert_eq!(a.raw[0x40], 255);
    assert_eq!(a.raw[0x11] & 0x80, 0);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn spell_combat_keeps_original_invisibility_penalties_and_source_side_rockskin_reduction() {
    let rom = rom();
    let game = fixture(&rom, 2, 9);
    let mut source = game.party[2].clone();
    source.set_word(0x3e, 0);
    source.raw[0x11] = 0;
    source.raw[0x9a] = 0;
    source.set_word(0x90, 0);
    let mut target = Runner::load(&rom, 0, 0).unwrap();
    target.set_word(0x3e, 3);
    target.raw[0x10] = 5;
    target.raw[0x9b] = 2;
    target.raw[0x11] = 0;
    target.raw[0xdf] = 0;
    target.raw[0x77] = 0;
    target.raw[0x7c] = 0;
    target.raw[0x76] = 0;
    let normal = casting::roll(&rom, &mut source.clone(), &target, &mut State { seed: 1 }).unwrap();
    target.raw[0x11] |= 0x80;
    let hidden = casting::roll(&rom, &mut source.clone(), &target, &mut State { seed: 1 }).unwrap();
    assert_eq!(hidden.difficulty, normal.difficulty + 4);
    source.set_word(0x90, 2);
    let sensed = casting::roll(&rom, &mut source.clone(), &target, &mut State { seed: 1 }).unwrap();
    assert_eq!(sensed.difficulty, 3);
    source.raw[0x9f] = 12;
    target.raw[0x11] = 0;
    let mut a = vec![
        source.clone(),
        game.party[1].clone(),
        game.party[0].clone(),
        target,
    ];
    a[1].set_word(0x3e, 1);
    a[2].set_word(0x3e, 2);
    let resolve = |actors: &mut [Runner]| {
        damage::resolve_magic(
            &rom,
            actors,
            0,
            3,
            &mut Default::default(),
            &mut State { seed: 1 },
        )
        .unwrap()
        .amount
    };
    let normal = resolve(&mut a.clone());
    let mut coated = a.clone();
    coated[0].raw[0x11] |= 0x20;
    assert_eq!(
        resolve(&mut coated),
        normal - 2,
        "$55e62 reads A0, the source, on this path"
    );
    a[3].raw[0x11] |= 0x20;
    assert_eq!(
        resolve(&mut a),
        normal,
        "do not invent defender-side protection"
    );
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn real_street_invisibility_uses_the_learned_book_then_retains_status_on_weapon_restore() {
    let rom = rom();
    let mut game = Game::load(&rom, 5, &[0, 3, 10]).unwrap();
    let ammo = game.party[2].raw[0x4e];
    equip(&mut game, &rom, 2, 4);
    for _ in 0..6000 {
        if game.encounters.spawned > 0 {
            break;
        }
        game.update(&rom, 0).unwrap();
    }
    assert!(
        game.encounters.spawned > 0,
        "original street encounter did not arrive"
    );
    cast_until(&mut game, &rom, 2, 0x80);
    game.equip_inventory_weapon(&rom).unwrap();
    assert_eq!(game.party[2].raw[0x4e], ammo);
    assert!(game.encounters.spawned > 0);
    assert!(game.combat.casting.casts > 0);
    assert_ne!(game.party[2].raw[0x11] & 0x80, 0);
    eprintln!(
        "Real street Invisibility at tick {}, encounters {}, duration {}",
        game.tick, game.encounters.spawned, game.party[2].raw[0x40]
    );
    game.render(&rom).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn status_catalog_rejects_changed_rules_and_full_graphics_pool_does_not_apply_a_status() {
    let rom = rom();
    status_spells::Catalog::load(&rom).unwrap();
    for json in [
        status_spells::Catalog::JSON.replace("\"flag\": 128", "\"flag\": 64"),
        status_spells::Catalog::JSON.replace("0x4dfe", "0x4dca"),
    ] {
        assert!(status_spells::Catalog::parse(&json, &rom).is_err());
    }
    let game = fixture(&rom, 2, 4);
    let mut actors = game.party.clone();
    let mut runtime = casting::Runtime::default();
    runtime.graphics = [true; 18];
    let result = runtime
        .cast(
            &rom,
            &mut actors,
            2,
            2,
            &mut Default::default(),
            &mut State { seed: 1 },
        )
        .unwrap();
    assert_eq!(result, casting::CastResult::GraphicsFull);
    assert_eq!(actors[2].raw[0x11] & 0x80, 0);
    assert_eq!(runtime.live, 0);
    assert_eq!(runtime.graphics, [true; 18]);
}
