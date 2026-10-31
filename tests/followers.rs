use shadowrealm::{
    collision::{Collision, Contact},
    followers,
    party::Runner,
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var_os("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM"))
        .unwrap()
}
fn corridor() -> Collision {
    let mut cells = vec![0x7f; 4096];
    for y in 4..=10 {
        for x in 4..=14 {
            cells[y * 64 + x] = 0;
        }
    }
    Collision {
        cells,
        rectangles: vec![],
        bounds: [0, 0, 512, 512],
        overflow_writes: vec![],
    }
}
fn party(rom: &[u8]) -> Vec<Runner> {
    let mut leader = Runner::load(rom, 0, 0).unwrap();
    leader.set_position(368 * 16, 240 * 16);
    leader.set_word(0x2e, 0x4000);
    leader.set_word(0xc, 128);
    let mut follower = Runner::load(rom, 5, 1).unwrap();
    follower.set_position(176 * 16, 240 * 16);
    follower.set_word(0xc, 128);
    vec![leader, follower]
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn regular_route_goes_around_a_wall_without_crossing_it() {
    let rom = rom();
    let mut c = corridor();
    for y in 4..=8 {
        c.cells[y * 64 + 8] = 0x7f;
    }
    let mut p = party(&rom);
    let leader = p[0].clone();
    let goal = followers::formation_goal(&rom, &c, &p[1], &leader).unwrap();
    let mut detour = false;
    for _ in 0..900 {
        followers::tick(&rom, &c, &mut p, 0, &leader).unwrap();
        assert_eq!(c.contact(p[1].position()).unwrap(), Contact::Clear);
        detour |= p[1].position().1 > 288 * 16;
        if followers::distance(p[1].position(), goal) < 0x1e0 {
            break;
        }
    }
    assert!(detour, "route must use the opening below the wall");
    assert!(
        followers::distance(p[1].position(), goal) < 0x1e0,
        "follower did not settle near formation goal: {:?}",
        p[1].position()
    );
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn follower_cannot_cross_an_area_trigger() {
    let rom = rom();
    let mut c = corridor();
    for y in 4..=10 {
        c.cells[y * 64 + 10] = 0x80;
    }
    let mut p = party(&rom);
    let leader = p[0].clone();
    for _ in 0..600 {
        followers::tick(&rom, &c, &mut p, 0, &leader).unwrap();
        assert_eq!(c.contact(p[1].position()).unwrap(), Contact::Clear);
    }
    assert!(p[1].position().0 < 320 * 16);
    assert_eq!(p[1].raw[0x10] & 2, 0);
}
