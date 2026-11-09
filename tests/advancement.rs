use shadowrealm::{
    advancement::{self, Catalog, Refusal},
    party::Runner,
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
#[test]
fn catalog_rejects_unverified_scope_and_rules() {
    for json in [
        Catalog::JSON.replace("0xd82a", "0xd82c"),
        Catalog::JSON.replace("\"fee\": 75", "\"fee\": 1"),
        Catalog::JSON.replacen('{', "{\"full_heal\":true,", 1),
    ] {
        assert!(Catalog::parse(&json, &[]).is_err());
    }
    assert!(advancement::field(20).is_err());
    assert_eq!(advancement::field(7).unwrap(), 0x7f);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn advancement_caps_specializations_eligibility_and_cancellation() {
    let rom = rom();
    let c = Catalog::load(&rom).unwrap();
    let mut a = Runner::load(&rom, 0, 0).unwrap();
    a.raw[0x8d] = 20;
    a.raw[0x77] = 6;
    assert_eq!(
        c.quote(&rom, &a, 0).unwrap().refusal,
        Some(Refusal::Maximum)
    );
    a.raw[0x9a] = 4; // Troll Body cap 11.
    assert_eq!(c.quote(&rom, &a, 0).unwrap().next, 7);
    for id in [6, 7] {
        assert_eq!(c.advance(&rom, &a, id, true).unwrap().0.raw, a.raw);
    }
    assert_eq!(
        c.quote(&rom, &a, 8).unwrap().refusal,
        Some(Refusal::MageOnly)
    );
    a.raw[0x9b] = 1;
    assert_eq!(c.quote(&rom, &a, 8).unwrap().cost, 2);
    a.raw[0x82] = 0;
    a.raw[0x81] = 4;
    let q = c.quote(&rom, &a, 10).unwrap();
    assert_eq!((q.next, q.cost), (5, 8));
    let before = a.raw;
    assert_eq!(c.advance(&rom, &a, 10, false).unwrap().0.raw, before);
    a.raw[0x8d] = 7;
    let (after, r) = c.advance(&rom, &a, 10, true).unwrap();
    assert_eq!(r, Some(Refusal::InsufficientKarma));
    assert_eq!(after.raw, a.raw);
    a.raw[0x8d] = 8;
    let (after, r) = c.advance(&rom, &a, 10, true).unwrap();
    assert_eq!(r, None);
    assert_eq!(
        (after.raw[0x82], after.raw[0x81], after.raw[0x8d]),
        (5, 4, 0)
    );
    a.raw[0x82] = 12;
    assert_eq!(
        c.quote(&rom, &a, 10).unwrap().refusal,
        Some(Refusal::Maximum)
    );
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn room_rest_is_partial_and_severe_wounds_need_medical_care() {
    let rom = rom();
    let mut party = (0..3)
        .map(|i| Runner::load(&rom, [0, 3, 10][i], i).unwrap())
        .collect::<Vec<_>>();
    for (a, h) in party.iter_mut().zip([2, 3, 5]) {
        a.raw[0x4c] = h;
        a.raw[0x9c] = 0;
        a.raw[0x10] |= 0x10;
    }
    assert_eq!(advancement::rest(&mut party), vec![0]);
    assert_eq!(
        party.iter().map(|a| a.raw[0x4c]).collect::<Vec<_>>(),
        vec![2, 4, 7]
    );
    assert!(party.iter().all(|a| a.raw[0x9c] == 10));
    assert_eq!(party[0].raw[0x10] & 0x10, 0x10);
    assert_eq!(party[1].raw[0x10] & 0x10, 0);
    party[0].raw[0x10] &= !1;
    let before = party[0].raw;
    advancement::rest(&mut party);
    assert_eq!(party[0].raw, before);
}
