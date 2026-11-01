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
        map_width_tiles: 32,
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

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn wall_hugging_escapes_rotated_u_pockets_and_rejoins_formation() {
    let rom = rom();
    for rotation in 0..4 {
        let rotate = |mut x: usize, mut y: usize| {
            for _ in 0..rotation {
                (x, y) = (16 - y, x);
            }
            (x, y)
        };
        let mut c = corridor();
        c.cells.fill(0x7f);
        for y in 2..=14 {
            for x in 2..=14 {
                c.cells[y * 64 + x] = 0;
            }
        }
        for y in 4..=10 {
            let (x, y) = rotate(8, y);
            c.cells[y * 64 + x] = 0x7f;
        }
        for x in 4..=8 {
            for y in [4, 10] {
                let (x, y) = rotate(x, y);
                c.cells[y * 64 + x] = 0x7f;
            }
        }
        let position = |x, y| {
            let (x, y) = rotate(x, y);
            ((x as u16 * 32 + 16) * 16, (y as u16 * 32 + 16) * 16)
        };
        let mut p = party(&rom);
        let leader_position = position(12, 7);
        p[0].set_position(leader_position.0, leader_position.1);
        p[0].set_word(0x2e, 0x4000u16.wrapping_add(rotation as u16 * 0x4000));
        let start = position(6, 7);
        p[1].set_position(start.0, start.1);
        p[1].set_word(0xb8, 0xffff);
        let leader = p[0].clone();
        let goal = followers::formation_goal(&rom, &c, &p[1], &leader).unwrap();
        let mut wall_route = false;
        let mut finished_wall_route = false;
        for _ in 0..1800 {
            followers::tick(&rom, &c, &mut p, 0, &leader).unwrap();
            assert_eq!(c.contact(p[1].position()).unwrap(), Contact::Clear);
            wall_route |= p[1].raw[0xb0] & 8 != 0;
            finished_wall_route |= wall_route && p[1].raw[0xb0] & 8 == 0;
        }
        assert!(wall_route, "rotation {rotation}: fallback was never used");
        assert!(
            finished_wall_route,
            "rotation {rotation}: fallback never cleared"
        );
        assert!(
            followers::distance(p[1].position(), goal) < 0x1e0,
            "rotation {rotation}: did not rejoin formation: {:?}, goal {goal:?}",
            p[1].position()
        );
        assert_eq!(p[0].position(), leader_position);
        assert_eq!(p[1].raw[0x10] & 2, 0);
    }
}
