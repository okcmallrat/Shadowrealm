use shadowrealm::{
    game::Game,
    input,
    party::Runner,
    shops::{self, Catalog, Category, Outcome},
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn press(g: &mut Game, r: &[u8], key: u8) {
    g.update(r, 0).unwrap();
    g.update(r, key).unwrap();
}
fn choose(g: &mut Game, r: &[u8], target: usize) {
    for _ in 0..10 {
        if g.dialogue.as_ref().unwrap().selected == target {
            break;
        }
        press(g, r, input::DOWN);
    }
    assert_eq!(g.dialogue.as_ref().unwrap().selected, target);
    press(g, r, input::A);
}
#[test]
fn catalog_rejects_new_rules_before_rom_access() {
    for json in [
        Catalog::JSON.replace("0x5794e", "0x57950"),
        Catalog::JSON.replacen('{', "{\"free_supplies\":true,", 1),
        Catalog::JSON.replace("\"shop\": 1", "\"shop\": 0"),
    ] {
        assert!(Catalog::parse(&json, &[]).is_err());
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn ammunition_full_inventory_limit_and_refusals_follow_original_order() {
    let r = rom();
    let c = Catalog::load(&r).unwrap();
    let mut a = Runner::load(&r, 0, 0).unwrap();
    a.raw[0x66..0x6e].fill(7);
    a.raw[0x4f] = 19;
    let p = c
        .purchase(&r, Category::Equipment, 0, &a, 25, true)
        .unwrap();
    assert_eq!(p.outcome, Outcome::Bought);
    assert_eq!((p.balance, p.actor.raw[0x4f]), (0, 20));
    assert_eq!(p.actor.raw[0x66..0x76], a.raw[0x66..0x76]);
    let p = c
        .purchase(&r, Category::Weapons, 0, &a, 10000, true)
        .unwrap();
    assert_eq!(p.outcome, Outcome::InventoryFull);
    assert_eq!(p.actor.raw, a.raw);
    a.raw[0x4f] = 20;
    let p = c
        .purchase(&r, Category::Equipment, 0, &a, 24, true)
        .unwrap();
    assert_eq!(p.outcome, Outcome::InsufficientFunds);
    assert_eq!(p.actor.raw, a.raw);
    let p = c
        .purchase(&r, Category::Equipment, 0, &a, 25, true)
        .unwrap();
    assert_eq!(p.outcome, Outcome::ClipLimit);
    assert_eq!(p.balance, 25);
    assert_eq!(p.actor.raw, a.raw);
    let p = c
        .purchase(&r, Category::Equipment, 0, &a, 25, false)
        .unwrap();
    assert_eq!(p.outcome, Outcome::Cancelled);
    assert_eq!(p.actor.raw, a.raw);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn inventory_freezes_world_and_equips_owned_firearm_with_transactional_hud() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0, 3, 10]).unwrap();
    // Explicit second-weapon fixture; the connected courier test earns/buys it.
    g.party[0].raw[0x6a] = 5;
    let before = g.party.iter().map(|a| a.position()).collect::<Vec<_>>();
    let funds = g.nuyen;
    g.show_inventory(&r).unwrap();
    g.select(&r, 1).unwrap();
    assert_eq!(g.selected, 0);
    for _ in 0..60 {
        g.update(&r, input::RIGHT).unwrap();
    }
    assert_eq!(
        g.party.iter().map(|a| a.position()).collect::<Vec<_>>(),
        before
    );
    choose(&mut g, &r, 4);
    assert_eq!(
        (
            g.party[0].raw[0x56],
            g.party[0].raw[0x57],
            g.party[0].raw[0x4e]
        ),
        (4, 5, 6)
    );
    assert_eq!(g.nuyen, funds);
    choose(&mut g, &r, 4);
    assert_eq!(
        (
            g.party[0].raw[0x56],
            g.party[0].raw[0x57],
            g.party[0].raw[0x4e]
        ),
        (127, 0, 100)
    );
    choose(&mut g, &r, 0);
    assert_eq!((g.party[0].raw[0x57], g.party[0].raw[0x4e]), (2, 12));
    g.render(&r).unwrap();
    choose(&mut g, &r, 8);
    assert!(g.dialogue.is_none());
    g.party[0].raw[0x57] = 14;
    g.party[0].raw[0x56] = 2;
    let before = g.party[0].raw;
    assert!(shops::equip_firearm(&r, &mut g.party[0], 0).is_err());
    assert_eq!(g.party[0].raw, before);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn failed_inventory_hud_does_not_commit_equipment_or_money() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0]).unwrap();
    g.party[0].raw[0x6a] = 5;
    g.show_inventory(&r).unwrap();
    for _ in 0..4 {
        press(&mut g, &r, input::DOWN);
    }
    g.update(&r, 0).unwrap();
    let before = g.party[0].raw;
    let funds = g.nuyen;
    let mut broken = r.clone();
    broken[0xd4540 + 4 * 48 + 26..0xd4540 + 4 * 48 + 30]
        .copy_from_slice(&0x1fffffu32.to_be_bytes());
    assert!(g.update(&broken, input::A).is_err());
    assert_eq!(g.party[0].raw, before);
    assert_eq!(g.nuyen, funds);
    assert!(g.dialogue.is_some());
}
