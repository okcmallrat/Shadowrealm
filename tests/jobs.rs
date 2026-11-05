use shadowrealm::{
    avoidance::State,
    collision::Contact,
    game::Game,
    input,
    jobs::{Catalog, Runtime, Visit},
    party::Runner,
};
use std::collections::{BinaryHeap, HashMap};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn press(game: &mut Game, rom: &[u8], key: u8) {
    game.update(rom, 0).unwrap();
    game.update(rom, key).unwrap();
}
fn capture(game: &mut Game, rom: &[u8], name: &str) {
    use std::io::Write;
    if let Ok(directory) = std::env::var("SHADOWREALM_JOB_CAPTURES") {
        std::fs::create_dir_all(&directory).unwrap();
        let mut f =
            std::fs::File::create(std::path::Path::new(&directory).join(format!("{name}.ppm")))
                .unwrap();
        f.write_all(b"P6\n320 224\n255\n").unwrap();
        f.write_all(&game.render(rom).unwrap()).unwrap();
    }
}
fn finish_transition(game: &mut Game, rom: &[u8], map: usize) {
    if game.dialogue.is_some() {
        press(game, rom, input::A);
    }
    for _ in 0..200 {
        game.update(rom, 0).unwrap();
        if game.transition.is_none() && game.map_id() == map {
            return;
        }
    }
    panic!(
        "transition did not arrive at {map}: {} {:?}",
        game.map_id(),
        game.pending_transition
    );
}
// Controller route over real collision, replanned after physical party contacts.
// No actor coordinates/stats, map IDs, encounters or job state are assigned.
fn walk(game: &mut Game, rom: &[u8], goal: (u16, u16), expected_dialogue: bool) {
    for _ in 0..5000 {
        if game.dialogue.is_some() || game.transition.is_some() {
            return;
        }
        assert!(!game.defeated, "party defeated during courier route");
        let (fx, fy) = game.party[game.selected].position();
        let start = ((fx + 64) / 128, (fy + 64) / 128);
        let end = ((goal.0 + 4) / 8, (goal.1 + 4) / 8);
        let goal_contact = game.collision.contact((goal.0 * 16, goal.1 * 16)).unwrap();
        if fx.abs_diff(goal.0 * 16) < 64 && fy.abs_diff(goal.1 * 16) < 64 && !expected_dialogue {
            return;
        }
        let mut queue = BinaryHeap::new();
        let mut distance = HashMap::new();
        let mut previous = HashMap::new();
        distance.insert(start, 0i32);
        queue.push((0i32, start));
        let dirs = [
            (0i16, -1i16, input::UP),
            (-1, 0, input::LEFT),
            (1, 0, input::RIGHT),
            (0, 1, input::DOWN),
        ];
        while let Some((_, point)) = queue.pop() {
            if point == end {
                break;
            }
            let cost = distance[&point];
            for (dx, dy, _) in dirs {
                let x = point.0 as i32 + i32::from(dx);
                let y = point.1 as i32 + i32::from(dy);
                if !(2..=252).contains(&x) || !(2..=252).contains(&y) {
                    continue;
                }
                let next = (x as u16, y as u16);
                let fixed = (next.0 * 128, next.1 * 128);
                let contact = game.collision.contact(fixed);
                let pass = matches!(contact, Ok(Contact::Clear))
                    || (matches!(goal_contact, Contact::Trigger(_)) && contact == Ok(goal_contact));
                if !pass {
                    continue;
                }
                let new = cost + 1;
                if new < *distance.get(&next).unwrap_or(&i32::MAX) {
                    distance.insert(next, new);
                    previous.insert(next, point);
                    let h = i32::from(next.0.abs_diff(end.0) + next.1.abs_diff(end.1));
                    queue.push((-(new + h), next));
                }
            }
        }
        assert!(
            previous.contains_key(&end),
            "no route {:?}->{:?}, map {}",
            start,
            end,
            game.map_id()
        );
        let mut next = end;
        while previous.get(&next).is_some_and(|p| *p != start) {
            next = previous[&next];
        }
        let dx = i32::from(next.0 * 128) - i32::from(fx);
        let dy = i32::from(next.1 * 128) - i32::from(fy);
        let mut key = if dx.abs() > dy.abs() {
            if dx < 0 {
                input::LEFT
            } else {
                input::RIGHT
            }
        } else if dy < 0 {
            input::UP
        } else {
            input::DOWN
        };
        let candidate = match key {
            input::UP => (fx, fy.wrapping_sub(32)),
            input::DOWN => (fx, fy.wrapping_add(32)),
            input::LEFT => (fx.wrapping_sub(32), fy),
            _ => (fx.wrapping_add(32), fy),
        };
        if matches!(game.collision.contact(candidate), Ok(Contact::Blocked)) {
            let ax = i32::from(start.0 * 128) - i32::from(fx);
            let ay = i32::from(start.1 * 128) - i32::from(fy);
            if matches!(key, input::LEFT | input::RIGHT) && ay.abs() > 8 {
                key = if ay < 0 { input::UP } else { input::DOWN };
            } else if matches!(key, input::UP | input::DOWN) && ax.abs() > 8 {
                key = if ax < 0 { input::LEFT } else { input::RIGHT };
            }
        }
        for _ in 0..2 {
            game.update(rom, key).unwrap();
            if game.dialogue.is_some() || game.transition.is_some() {
                return;
            }
        }
    }
    panic!(
        "did not reach {:?} from {:?}, map {}, flags {:?}",
        goal,
        game.party[game.selected].position(),
        game.map_id(),
        &game.party[game.selected].raw[0xb0..0xba]
    );
}
fn door(game: &Game, location: u8) -> (u16, u16) {
    let r = game
        .collision
        .rectangles
        .iter()
        .find(|r| r.code == 0xc0 + location)
        .expect("real Redmond door");
    ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn real_jump_house_courier_loop_uses_controls_pickup_payment_and_can_repeat() {
    let rom = rom();
    let mut game = Game::load(&rom, 2, &[0, 3, 10]).unwrap();
    let balance = game.nuyen;
    let karma = game.party.iter().map(|a| a.raw[0x8d]).collect::<Vec<_>>();
    let entrance = door(&game, 4);
    walk(&mut game, &rom, entrance, true);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0x110d1e);
    finish_transition(&mut game, &rom, 13);
    walk(&mut game, &rom, (48, 48), true);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0x105114);
    // Real offers/declines retain the shared original RNG, with no forced job ID.
    for _ in 0..64 {
        press(&mut game, &rom, input::DOWN);
        press(&mut game, &rom, input::A); // ask for work
        press(&mut game, &rom, input::A); // original MORE -> generation
        let courier = game.jobs.courier();
        press(&mut game, &rom, input::A); // brief -> original confirmation
        if courier {
            let quote = &game.dialogue.as_ref().unwrap().text;
            assert!(quote.contains(&format!(" {}$", game.jobs.base_pay)));
            assert!(quote.contains(&format!(" {}$", game.jobs.pay())));
            capture(&mut game, &rom, "confirmation");
            press(&mut game, &rom, input::A);
            press(&mut game, &rom, input::A);
            break;
        }
        assert!(!game.dialogue.as_ref().unwrap().choices[0].available);
        press(&mut game, &rom, input::B);
        press(&mut game, &rom, input::A);
        walk(&mut game, &rom, (80, 112), false);
        walk(&mut game, &rom, (48, 48), true);
    }
    assert!(game.jobs.accepted && game.jobs.courier());
    let source = game.jobs.raw[3];
    let target = game.jobs.raw[6];
    let fee = game.jobs.pay();
    let gain = game.jobs.raw[7];
    game.show_job(&rom).unwrap();
    assert!(game.dialogue.as_ref().unwrap().text.contains("Payment:"));
    capture(&mut game, &rom, "accepted");
    press(&mut game, &rom, input::A);
    walk(&mut game, &rom, (96, 240), true);
    finish_transition(&mut game, &rom, 2);
    let pickup = door(&game, source);
    walk(&mut game, &rom, pickup, true);
    assert_eq!(game.jobs.flags & 1, 1);
    assert_eq!(game.nuyen, balance);
    capture(&mut game, &rom, "pickup");
    while game.dialogue.is_some() {
        press(&mut game, &rom, input::A);
    }
    let destination = door(&game, target);
    walk(&mut game, &rom, destination, true);
    assert_eq!(game.jobs.completed, 1);
    assert!(!game.jobs.active());
    assert_eq!(game.nuyen, balance + u32::from(fee));
    capture(&mut game, &rom, "paid");
    for (a, before) in game.party.iter().zip(karma) {
        assert_eq!(a.raw[0x8d], before.wrapping_add(gain));
    }
    while game.dialogue.is_some() {
        press(&mut game, &rom, input::A);
    }
    let entrance = door(&game, 4);
    walk(&mut game, &rom, entrance, true);
    finish_transition(&mut game, &rom, 13);
    walk(&mut game, &rom, (48, 48), true);
    press(&mut game, &rom, input::DOWN);
    press(&mut game, &rom, input::A);
    press(&mut game, &rom, input::A);
    assert!(game.jobs.active());
    assert_eq!(game.jobs.completed, 1);
    capture(&mut game, &rom, "next-offer");
    game.render(&rom).unwrap();
    eprintln!("Real courier loop at tick {}, pickup {}, delivery {}, payment {}$, encounters {}, next type {}",game.tick,source,target,fee,game.encounters.spawned,game.jobs.raw[1]);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn wrong_stop_delivery_before_pickup_and_duplicate_visits_cannot_pay() {
    let rom = rom();
    let mut job = Runtime::default();
    job.raw = [0, 2, 2, 1, 0, 2, 11, 1, 0, 45];
    job.accepted = true;
    let mut party = vec![Runner::load(&rom, 0, 0).unwrap()];
    let mut balance = 100;
    assert_eq!(job.visit(2, 11, &mut balance, &mut party), Visit::None);
    assert_eq!(job.visit(3, 1, &mut balance, &mut party), Visit::None);
    assert_eq!(job.visit(2, 1, &mut balance, &mut party), Visit::Pickup);
    assert_eq!(job.visit(2, 1, &mut balance, &mut party), Visit::None);
    assert_eq!(balance, 100);
    assert_eq!(job.visit(2, 11, &mut balance, &mut party), Visit::Delivery);
    assert_eq!(balance, 145);
    assert_eq!(job.visit(2, 11, &mut balance, &mut party), Visit::None);
    assert_eq!(balance, 145);
    assert_eq!(job.completed, 1);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn invalid_json_or_truncated_generation_keeps_job_and_rng_uncommitted() {
    let rom = rom();
    let catalog = Catalog::load(&rom).unwrap();
    for json in [
        Catalog::JSON.replace("0x1e69c", "0x1e69e"),
        Catalog::JSON.replacen('{', "{\"invented_reward\":999,", 1),
        Catalog::JSON.replace("0x11166e", "0x111676"),
    ] {
        assert!(Catalog::parse(&json, &rom).is_err());
    }
    let mut job = Runtime::default();
    job.raw = [0, 2, 2, 1, 0, 2, 11, 1, 0, 45];
    job.accepted = true;
    job.flags = 1;
    job.carried = 1 << 9;
    let before = job.clone();
    let mut rng = State { seed: 123 };
    assert!(job
        .generate(&rom[..0xf3ebc], &catalog, 1, &mut rng)
        .is_err());
    assert_eq!(job, before);
    assert_eq!(rng.seed, 123);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn defeat_loses_a_collected_package_before_medical_recovery_and_prevents_reward() {
    let rom = rom();
    let mut game = Game::load(&rom, 2, &[0, 3, 10]).unwrap();
    game.jobs.raw = [0, 2, 2, 1, 0, 2, 11, 1, 0, 45];
    game.jobs.accepted = true;
    game.jobs.flags = 1;
    game.jobs.carried = 1 << 9;
    for a in &mut game.party {
        a.raw[0x4c] = 0;
        a.raw[0x9c] = 0;
        a.raw[0x10] |= 0x50;
    }
    game.combat.damage.defeat_timer = 1;
    game.update(&rom, 0).unwrap();
    assert!(game.defeated);
    assert!(!game.jobs.active());
    assert_eq!(game.jobs.carried & (1 << 9), 0);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf2510);
    press(&mut game, &rom, input::A);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xc25bc);
    press(&mut game, &rom, input::A);
    finish_transition(&mut game, &rom, 2);
    assert!(!game.defeated);
    assert_eq!(game.jobs.completed, 0);
}
