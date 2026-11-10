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
    for step in 0..5000 {
        if step % 500 == 0 && std::env::var_os("SHADOWREALM_ROUTE_TRACE").is_some() {
            eprintln!(
                "Walk step {step}, map {}, position {:?}, goal {goal:?}, encounters {}, HP {}",
                game.map_id(),
                game.party[game.selected].position(),
                game.encounters.spawned,
                game.party[0].raw[0x4c]
            );
        }
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
        let initial_h = i32::from(start.0.abs_diff(end.0) + start.1.abs_diff(end.1));
        queue.push((-initial_h, start));
        let dirs = [
            (0i16, -1i16, input::UP),
            (-1, 0, input::LEFT),
            (1, 0, input::RIGHT),
            (0, 1, input::DOWN),
        ];
        while let Some((priority, point)) = queue.pop() {
            let h = i32::from(point.0.abs_diff(end.0) + point.1.abs_diff(end.1));
            if priority != -(distance[&point] + h) {
                continue;
            }
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
fn choose_destination(game: &mut Game, rom: &[u8], destination: usize) {
    let slot = game
        .dialogue
        .as_ref()
        .unwrap()
        .choices
        .iter()
        .position(|c| c.destination == destination && c.available)
        .expect("available destination");
    for _ in 0..8 {
        if game.dialogue.as_ref().unwrap().selected == slot {
            break;
        }
        press(game, rom, input::DOWN);
    }
    press(game, rom, input::A);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn stoker_refuses_back_rent_and_failed_passage_assets_cannot_commit_a_purchase() {
    let rom = rom();
    let mut game = Game::load(&rom, 2, &[0]).unwrap();
    let stoker = door(&game, 2);
    walk(&mut game, &rom, stoker, true);
    choose_destination(&mut game, &rom, 0xf4b10);
    choose_destination(&mut game, &rom, 48);
    choose_destination(&mut game, &rom, 5);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf5ab0);
    assert_eq!(game.nuyen, 20);
    assert_eq!(game.recovery_flags, 0);
    assert_eq!(game.story.inquiries & 1, 1);
    press(&mut game, &rom, input::A);
    // Explicit funds fixture for the failure boundary; not the connected
    // controller-only earnings scenario below.
    game.nuyen = 250;
    for _ in 0..24 {
        game.update(&rom, input::DOWN).unwrap();
    }
    walk(&mut game, &rom, stoker, true);
    choose_destination(&mut game, &rom, 0xf4b10);
    choose_destination(&mut game, &rom, 48);
    game.update(&rom, 0).unwrap();
    let before = game.party[0].raw;
    let story = game.story.clone();
    let mut broken = rom.clone();
    broken[0xc1064..0xc1068].fill(255);
    assert!(game.update(&broken, input::A).is_err());
    assert_eq!(game.nuyen, 250);
    assert_eq!(game.recovery_flags, 0);
    assert_eq!(game.party[0].raw, before);
    assert_eq!(game.story, story);
    choose_destination(&mut game, &rom, 5);
    assert_eq!(game.nuyen, 500);
    assert_eq!(game.recovery_flags, 1);
    while game.dialogue.is_some() {
        press(&mut game, &rom, input::A);
    }
    for _ in 0..24 {
        game.update(&rom, input::DOWN).unwrap();
    }
    walk(&mut game, &rom, stoker, true);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf4aa0);
    assert_eq!(game.nuyen, 500);
    assert!(game
        .dialogue
        .as_ref()
        .unwrap()
        .choices
        .iter()
        .all(|c| c.destination != 48 && c.destination != 5));
}
fn earn_belongings(rom: &[u8]) -> Game {
    let game = Game::load(&rom, 2, &[0, 3, 10]).unwrap();
    earn_belongings_from(rom, game)
}
fn earn_belongings_from(rom: &[u8], mut game: Game) -> Game {
    // Only ordinary inputs and the desktop's read-only/menu shortcuts from this
    // point: no assignment to money, party, RNG, missions, map or positions.
    for _ in 0..16 {
        if std::env::var_os("SHADOWREALM_ROUTE_TRACE").is_some() {
            eprintln!(
                "Job {}, balance {}, tick {}",
                game.jobs.completed, game.nuyen, game.tick
            );
        }
        let entrance = door(&game, 4);
        walk(&mut game, &rom, entrance, true);
        finish_transition(&mut game, &rom, 13);
        walk(&mut game, &rom, (48, 48), true);
        for _ in 0..64 {
            choose_destination(&mut game, &rom, 13);
            choose_destination(&mut game, &rom, 7);
            let courier = game.jobs.courier();
            choose_destination(&mut game, &rom, 9);
            if courier {
                choose_destination(&mut game, &rom, 50);
                press(&mut game, &rom, input::A);
                break;
            }
            choose_destination(&mut game, &rom, 11);
            press(&mut game, &rom, input::A);
            walk(&mut game, &rom, (80, 112), false);
            walk(&mut game, &rom, (48, 48), true);
        }
        assert!(game.jobs.accepted && game.jobs.courier());
        let source = game.jobs.raw[3];
        let target = game.jobs.raw[6];
        walk(&mut game, &rom, (96, 240), true);
        finish_transition(&mut game, &rom, 2);
        let pickup = door(&game, source);
        walk(&mut game, &rom, pickup, true);
        assert_eq!(game.jobs.flags & 1, 1);
        while game.dialogue.is_some() {
            press(&mut game, &rom, input::A);
        }
        let destination = door(&game, target);
        walk(&mut game, &rom, destination, true);
        if std::env::var_os("SHADOWREALM_ROUTE_TRACE").is_some() {
            eprintln!(
                "Delivered? node {:?}, balance {}, job {:?}",
                game.dialogue
                    .as_ref()
                    .map(|d| (d.node, d.caption, d.choices.len())),
                game.nuyen,
                game.jobs.raw
            );
        }
        assert!(!game.jobs.active());
        for _ in 0..32 {
            if game.dialogue.is_none() {
                break;
            }
            press(&mut game, &rom, input::A);
        }
        assert!(
            game.dialogue.is_none(),
            "delivery passage did not end: {:?}",
            game.dialogue
                .as_ref()
                .map(|d| (d.node, d.caption, d.choices.len()))
        );
        if game.nuyen >= 250 {
            break;
        }
    }
    assert!(
        game.nuyen >= 250,
        "courier earnings did not reach back rent"
    );
    let earned = game.nuyen;
    let completed = game.jobs.completed;
    let stoker = door(&game, 2);
    walk(&mut game, &rom, stoker, true);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf4afc);
    choose_destination(&mut game, &rom, 0xf4b10);
    choose_destination(&mut game, &rom, 48);
    let karma = game.party.iter().map(|a| a.raw[0x8d]).collect::<Vec<_>>();
    choose_destination(&mut game, &rom, 5);
    assert_eq!(game.nuyen, earned + 250);
    assert_eq!(game.recovery_flags & 1, 1);
    assert_eq!(game.story.leads & 0x1f, 0x1c);
    assert_ne!(game.party[0].raw[0x8f] & 1, 0);
    for (a, k) in game.party.iter().zip(karma) {
        assert_eq!(a.raw[0x8d], k.wrapping_add(1));
    }
    capture(&mut game, &rom, "michaels-belongings");
    while game.dialogue.is_some() {
        press(&mut game, &rom, input::A);
    }
    assert_eq!(game.jobs.completed, completed);
    game
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn courier_earnings_buy_michaels_belongings_then_unlock_real_city_travel() {
    let rom = rom();
    let mut game = earn_belongings(&rom);
    let completed = game.jobs.completed;
    let earned = game.nuyen - 250;
    game.call_taxi(&rom).unwrap();
    capture(&mut game, &rom, "taxi-destinations");
    choose_destination(&mut game, &rom, 4);
    capture(&mut game, &rom, "taxi-quote");
    choose_destination(&mut game, &rom, 15);
    assert_eq!(game.nuyen, earned + 230);
    finish_transition(&mut game, &rom, 0);
    assert_eq!(game.recovery_flags, 1);
    assert_eq!(game.jobs.completed, completed);
    assert_eq!(game.map_id(), 0);
    capture(&mut game, &rom, "downtown-arrival");
    eprintln!("Connected courier -> belongings -> taxi: {} jobs, {} earned balance -> {} final, tick {}, encounters {}",completed,earned,game.nuyen,game.tick,game.encounters.spawned);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn courier_earnings_buy_equip_fire_reload_save_and_resume() {
    let rom = rom();
    let mut game = earn_belongings(&rom);
    let balance = game.nuyen;
    let completed = game.jobs.completed;
    let ares = door(&game, 11);
    walk(&mut game, &rom, ares, true);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0x110dd2);
    capture(&mut game, &rom, "ares-opening");
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    capture(&mut game, &rom, "ares-stock");
    choose_destination(&mut game, &rom, 3);
    assert!(game.dialogue.as_ref().unwrap().text.contains("Warhawk"));
    assert!(game.dialogue.as_ref().unwrap().text.contains("500"));
    capture(&mut game, &rom, "ares-confirmation");
    choose_destination(&mut game, &rom, 0);
    assert_eq!(game.nuyen, balance - 500);
    assert_eq!(game.party[0].raw[0x6a], 5);
    assert_eq!(game.party[0].raw[0x57], 2);
    choose_destination(&mut game, &rom, 8);
    choose_destination(&mut game, &rom, 1);
    let clips = game.party[0].raw[0x4f];
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    assert_eq!(game.nuyen, balance - 525);
    assert_eq!(game.party[0].raw[0x4f], clips + 1);
    capture(&mut game, &rom, "ares-ammo-purchased");
    choose_destination(&mut game, &rom, 8);
    choose_destination(&mut game, &rom, 2);
    game.show_inventory(&rom).unwrap();
    choose_destination(&mut game, &rom, 4);
    assert_eq!(
        (
            game.party[0].raw[0x56],
            game.party[0].raw[0x57],
            game.party[0].raw[0x4e]
        ),
        (4, 5, 6)
    );
    capture(&mut game, &rom, "warhawk-equipped");
    choose_destination(&mut game, &rom, 8);
    let shots = game.combat.shots;
    for _ in 0..2000 {
        game.update(&rom, input::A).unwrap();
        if game.combat.shots >= shots + 7 {
            break;
        }
    }
    assert_eq!(game.combat.shots, shots + 7);
    assert_eq!(game.party[0].raw[0x4f], clips);
    assert_eq!(game.party[0].raw[0x4e], 5);
    assert_eq!(game.nuyen, balance - 525);
    assert_eq!(game.jobs.completed, completed);
    assert_eq!(game.recovery_flags, 1);
    capture(&mut game, &rom, "warhawk-reloaded");
    eprintln!("Earn -> Ares -> equip -> 7 shots/reload: {} jobs, {} -> {} nuyen, clips {} -> purchased {} -> reloaded {}, tick {}, encounters {}",completed,balance,game.nuyen,clips,clips+1,game.party[0].raw[0x4f],game.tick,game.encounters.spawned);
    for _ in 0..1000 {
        game.update(&rom, 0).unwrap();
        if game.save_ready() {
            break;
        }
    }
    assert!(game.save_ready());
    let cash = game.nuyen;
    let packed = game
        .party
        .iter()
        .map(shadowrealm::archives::pack)
        .collect::<Vec<_>>();
    game.open_file_menu(&rom, shadowrealm::game::FileMode::Save)
        .unwrap();
    capture(&mut game, &rom, "save-slots");
    choose_destination(&mut game, &rom, 1);
    let request = game.take_file_request().unwrap();
    assert_eq!(request.slot, 1);
    let directory = std::env::temp_dir().join(format!(
        "shadowrealm-controller-save-{}",
        std::process::id()
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    game.write_save_slot(&rom, &directory, request.slot)
        .unwrap();
    let tick = game.tick;
    drop(game);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_shadowrealm"))
        .arg("save-summary")
        .arg(std::env::var("SHADOWREALM_TEST_ROM").unwrap())
        .arg(directory.join("slot-1.json"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(summary["nuyen"], cash);
    assert_eq!(summary["jobs_completed"], completed);
    assert_eq!(summary["party"][0]["weapon"], 5);
    assert_eq!(summary["party"][0]["ammo"], 5);
    assert_eq!(summary["recovery_flags"], 1);
    let (mut resumed, backup) = Game::load_slot(&rom, &directory, 1).unwrap();
    assert!(!backup);
    assert_eq!(resumed.tick, tick);
    assert_eq!(
        resumed
            .party
            .iter()
            .map(shadowrealm::archives::pack)
            .collect::<Vec<_>>(),
        packed
    );
    capture(&mut resumed, &rom, "resumed-warhawk");
    for _ in 0..200 {
        resumed.update(&rom, input::A).unwrap();
        if resumed.combat.shots > 0 {
            break;
        }
    }
    assert_eq!(resumed.combat.shots, 1);
    assert_eq!(resumed.party[0].raw[0x4e], 4);
    assert_eq!(resumed.nuyen, cash);
    assert_eq!(resumed.jobs.completed, completed);
    eprintln!("Disk checkpoint at tick {tick}: separate process read slot, native reload resumed Warhawk fire with {cash} nuyen and {completed} completed jobs");
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn collected_courier_checkpoint_delivers_once_after_native_load() {
    let rom = rom();
    let mut game = Game::load(&rom, 2, &[0, 3, 10]).unwrap();
    // Explicit in-progress job fixture; earnings are tested without state edits above.
    game.jobs.raw = [0, 2, 2, 1, 0, 2, 11, 1, 0, 45];
    game.jobs.accepted = true;
    game.jobs.flags = 1;
    game.jobs.carried = 512;
    let bytes = game.save_bytes(&rom).unwrap();
    let mut resumed = Game::from_save_bytes(&rom, &bytes).unwrap();
    let destination = door(&resumed, 11);
    walk(&mut resumed, &rom, destination, true);
    assert_eq!(resumed.nuyen, 65);
    assert_eq!(resumed.jobs.completed, 1);
    assert_eq!(resumed.jobs.carried & 512, 0);
    while resumed.dialogue.is_some() {
        press(&mut resumed, &rom, input::A);
    }
    for _ in 0..24 {
        resumed.update(&rom, input::DOWN).unwrap();
    }
    walk(&mut resumed, &rom, destination, true);
    assert_eq!(resumed.nuyen, 65);
    assert_eq!(resumed.jobs.completed, 1);
    assert_eq!(resumed.dialogue.as_ref().unwrap().node, 0x110dd2);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn ares_cancellation_and_failed_hud_cannot_charge_or_grant_a_weapon() {
    let rom = rom();
    let mut game = Game::load(&rom, 2, &[0]).unwrap();
    game.nuyen = 1000; // Explicit transaction boundary fixture, not earned-route proof.
    let ares = door(&game, 11);
    walk(&mut game, &rom, ares, true);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    assert!(
        !game.dialogue.as_ref().unwrap().choices[5].available,
        "unimplemented grenade use must remain guarded"
    );
    choose_destination(&mut game, &rom, 3);
    choose_destination(&mut game, &rom, 1);
    assert_eq!(game.nuyen, 1000);
    assert_eq!(game.party[0].raw[0x6a], 0);
    choose_destination(&mut game, &rom, 3);
    press(&mut game, &rom, input::DOWN); // Default No -> Yes.
    game.update(&rom, 0).unwrap();
    let before = game.party[0].raw;
    let frame = game.render(&rom).unwrap();
    let mut broken = rom.clone();
    broken[0xd4540 + 48 + 26..0xd4540 + 48 + 30].copy_from_slice(&0x1fffffu32.to_be_bytes());
    assert!(game.update(&broken, input::A).is_err());
    assert_eq!(game.nuyen, 1000);
    assert_eq!(game.party[0].raw, before);
    assert_eq!(game.render(&rom).unwrap(), frame);
    choose_destination(&mut game, &rom, 0);
    assert_eq!(game.nuyen, 500);
    assert_eq!(game.party[0].raw[0x6a], 5);
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

#[test]
#[ignore = "requires the user-supplied ROM"]
fn courier_earnings_pay_for_hotel_train_two_runners_and_save_upgrades() {
    let rom = rom();
    let mut game = earn_belongings(&rom);
    assert_eq!((game.jobs.completed, game.nuyen), (6, 530));
    game.call_taxi(&rom).unwrap();
    choose_destination(&mut game, &rom, 4);
    choose_destination(&mut game, &rom, 15);
    finish_transition(&mut game, &rom, 0);
    let hotel = door(&game, 3);
    walk(&mut game, &rom, hotel, true);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf4910);
    capture(&mut game, &rom, "gates-opening");
    choose_destination(&mut game, &rom, 0xf4924);
    choose_destination(&mut game, &rom, 0xf493c);
    capture(&mut game, &rom, "hotel-room-quote");
    choose_destination(&mut game, &rom, 6);
    assert_eq!(game.nuyen, 435);
    assert!(!game.save_ready());
    choose_destination(&mut game, &rom, 0); // rest
    capture(&mut game, &rom, "hotel-rest");
    choose_destination(&mut game, &rom, 0); // room
    let positions = game.party.iter().map(|a| a.position()).collect::<Vec<_>>();
    for _ in 0..60 {
        game.update(&rom, input::RIGHT).unwrap();
    }
    assert_eq!(
        game.party.iter().map(|a| a.position()).collect::<Vec<_>>(),
        positions
    );
    choose_destination(&mut game, &rom, 0); // runners
    choose_destination(&mut game, &rom, 0); // Joshua
    choose_destination(&mut game, &rom, 0); // attributes
    let body = game.party[0].raw[0x77];
    let karma = game.party[0].raw[0x8d];
    assert_eq!(body, 4);
    assert!(karma >= 5);
    choose_destination(&mut game, &rom, 0); // Body quote
    capture(&mut game, &rom, "karma-body-quote");
    choose_destination(&mut game, &rom, 1); // cancellation
    assert_eq!(
        (game.party[0].raw[0x77], game.party[0].raw[0x8d]),
        (body, karma)
    );
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0); // buy
    assert_eq!(
        (game.party[0].raw[0x77], game.party[0].raw[0x8d]),
        (5, karma - 5)
    );
    capture(&mut game, &rom, "karma-body-improved");
    choose_destination(&mut game, &rom, 20); // categories
    choose_destination(&mut game, &rom, 3); // room
    choose_destination(&mut game, &rom, 0); // runners
    choose_destination(&mut game, &rom, 1); // Ricky
    choose_destination(&mut game, &rom, 1); // combat skills
    let firearms = game.party[1].raw[0x81];
    let ricky_karma = game.party[1].raw[0x8d];
    assert_eq!(firearms, 2);
    assert!(ricky_karma >= 6);
    choose_destination(&mut game, &rom, 9); // Firearms quote
    capture(&mut game, &rom, "karma-companion-quote");
    choose_destination(&mut game, &rom, 0);
    assert_eq!(
        (game.party[1].raw[0x81], game.party[1].raw[0x8d]),
        (3, ricky_karma - 6)
    );
    assert_eq!(game.selected, 0);
    choose_destination(&mut game, &rom, 20);
    choose_destination(&mut game, &rom, 3);
    choose_destination(&mut game, &rom, 1); // check out
    choose_destination(&mut game, &rom, 1); // stay
    choose_destination(&mut game, &rom, 1);
    choose_destination(&mut game, &rom, 0); // leave
    assert!(game.dialogue.is_none());
    for _ in 0..100 {
        game.update(&rom, 0).unwrap();
    }
    assert!(game.save_ready());
    let directory = std::env::temp_dir().join(format!("shadowrealm-hotel-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    game.write_save_slot(&rom, &directory, 1).unwrap();
    let tick = game.tick;
    drop(game);
    let (mut resumed, _) = Game::load_slot(&rom, &directory, 1).unwrap();
    assert_eq!(
        (resumed.party[0].raw[0x77], resumed.party[0].raw[0x8d]),
        (5, karma - 5)
    );
    assert_eq!(
        (resumed.party[1].raw[0x81], resumed.party[1].raw[0x8d]),
        (3, ricky_karma - 6)
    );
    assert_eq!(
        (resumed.map_id(), resumed.nuyen, resumed.jobs.completed),
        (0, 435, 6)
    );
    capture(&mut resumed, &rom, "hotel-upgrades-resumed");
    eprintln!("Earned hotel route: six jobs -> belongings -> taxi -> 75-nuyen room -> Joshua Body 5 and Ricky Firearms 3 -> disk reload; tick {tick}, encounters {}",resumed.encounters.spawned);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn hotel_refusal_cancellation_and_failed_assets_preserve_transactions() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
    let hotel = door(&game, 3);
    walk(&mut game, &rom, hotel, true);
    choose_destination(&mut game, &rom, 0xf4924);
    choose_destination(&mut game, &rom, 0xf493c);
    let before = game.party.iter().map(|a| a.raw).collect::<Vec<_>>();
    choose_destination(&mut game, &rom, 6);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf5af8);
    assert_eq!(game.nuyen, 20);
    assert_eq!(game.party.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    press(&mut game, &rom, input::A);
    for _ in 0..24 {
        game.update(&rom, input::DOWN).unwrap();
    }
    walk(&mut game, &rom, hotel, true);
    choose_destination(&mut game, &rom, 0xf4924);
    choose_destination(&mut game, &rom, 0xf4b2c);
    press(&mut game, &rom, input::A);
    assert_eq!(game.nuyen, 20);
    // Explicit transaction fixture; connected route above uses only earned funds.
    game.nuyen = 100;
    game.party[0].raw[0x8d] = 20;
    for _ in 0..24 {
        game.update(&rom, input::DOWN).unwrap();
    }
    walk(&mut game, &rom, hotel, true);
    choose_destination(&mut game, &rom, 0xf4924);
    choose_destination(&mut game, &rom, 0xf493c);
    game.update(&rom, 0).unwrap();
    let mut broken = rom.clone();
    broken[0xc1064..0xc1068].fill(255);
    let before = game.party.iter().map(|a| a.raw).collect::<Vec<_>>();
    assert!(game.update(&broken, input::A).is_err());
    assert_eq!(game.nuyen, 100);
    assert_eq!(game.party.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    choose_destination(&mut game, &rom, 6);
    assert_eq!(game.nuyen, 25);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0); // Body confirmation
    game.update(&rom, 0).unwrap();
    let before = game.party.iter().map(|a| a.raw).collect::<Vec<_>>();
    let frame = game.render(&rom).unwrap();
    let mut broken = rom.clone();
    broken[0xd4540 + 48 + 26..0xd4540 + 48 + 30].copy_from_slice(&0x1fffffu32.to_be_bytes());
    assert!(game.update(&broken, input::A).is_err());
    assert_eq!(game.party.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    assert_eq!(game.nuyen, 25);
    assert_eq!(game.render(&rom).unwrap(), frame);
    choose_destination(&mut game, &rom, 0);
    assert_eq!((game.party[0].raw[0x77], game.party[0].raw[0x8d]), (5, 15));
    for _ in 0..60 {
        game.update(&rom, input::A).unwrap();
    }
    assert_eq!((game.party[0].raw[0x77], game.party[0].raw[0x8d]), (5, 15));
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn fresh_solo_new_game_earns_michaels_belongings_and_resumes_from_the_opening_menu() {
    use shadowrealm::session::Session;
    let rom = rom();
    let directory =
        std::env::temp_dir().join(format!("shadowrealm-newgame-route-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    fn menu_press(s: &mut Session, rom: &[u8], key: u8, path: &std::path::Path) {
        s.update(rom, 0, path).unwrap();
        s.update(rom, key, path).unwrap();
    }
    let mut session = Session::new(&rom).unwrap();
    menu_press(&mut session, &rom, input::A, &directory); // New Game
    menu_press(&mut session, &rom, input::A, &directory); // Samurai
    menu_press(&mut session, &rom, input::UP, &directory); // Begin
    menu_press(&mut session, &rom, input::A, &directory);
    menu_press(&mut session, &rom, input::A, &directory); // acknowledge briefing
    let game = session.game.take().unwrap();
    assert_eq!(
        (
            game.party.len(),
            game.party[0].template,
            game.nuyen,
            game.story.leads
        ),
        (1, 0, 20, 1)
    );
    let mut game = earn_belongings_from(&rom, game);
    assert_eq!(game.party.len(), 1);
    assert_eq!(game.recovery_flags, 1);
    capture(&mut game, &rom, "newgame-solo-earned-stash");
    for _ in 0..100 {
        game.update(&rom, 0).unwrap();
    }
    assert!(game.save_ready());
    game.write_save_slot(&rom, &directory, 1).unwrap();
    let balance = game.nuyen;
    let jobs = game.jobs.completed;
    let karma = game.party[0].raw[0x8d];
    let tick = game.tick;
    let encounters = game.encounters.spawned;
    assert_eq!(
        encounters, 0,
        "unexpected encounter in the recorded solo route"
    );
    drop(game);
    drop(session);
    let mut session = Session::new(&rom).unwrap();
    assert!(session.game.is_none());
    menu_press(&mut session, &rom, input::DOWN, &directory);
    menu_press(&mut session, &rom, input::A, &directory); // Load Game
    menu_press(&mut session, &rom, input::A, &directory); // Slot 1
    let game = session.game.as_mut().unwrap();
    assert_eq!(
        (
            game.party.len(),
            game.party[0].template,
            game.nuyen,
            game.jobs.completed,
            game.party[0].raw[0x8d]
        ),
        (1, 0, balance, jobs, karma)
    );
    assert_eq!(game.recovery_flags, 1);
    assert_ne!(game.party[0].raw[0x8f] & 1, 0);
    capture(game, &rom, "newgame-solo-title-load");
    eprintln!("Opening-menu solo Samurai -> {jobs} courier jobs -> Michael's belongings -> disk save -> fresh opening-menu load: {balance} nuyen, {karma} Karma, tick {tick}, encounters {encounters}");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn fresh_solo_can_rent_stokers_room_before_michaels_belongings() {
    let rom = rom();
    let mut game = Game::new_game(&rom, shadowrealm::newgame::Archetype::Samurai).unwrap();
    press(&mut game, &rom, input::A);
    let stoker = door(&game, 2);
    walk(&mut game, &rom, stoker, true);
    choose_destination(&mut game, &rom, 0xf4b10);
    choose_destination(&mut game, &rom, 0xf4ad4);
    capture(&mut game, &rom, "stoker-room-quote");
    choose_destination(&mut game, &rom, 6);
    assert_eq!(game.nuyen, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 0);
    assert!(game.dialogue.as_ref().unwrap().text.contains("enough"));
    choose_destination(&mut game, &rom, 0);
    choose_destination(&mut game, &rom, 20);
    choose_destination(&mut game, &rom, 3);
    choose_destination(&mut game, &rom, 1);
    choose_destination(&mut game, &rom, 0);
    for _ in 0..50 {
        game.update(&rom, 0).unwrap();
    }
    assert_eq!(game.recovery_flags, 0);
    assert_eq!(game.story.leads, 1);
    let bytes = game.save_bytes(&rom).unwrap();
    let loaded = Game::from_save_bytes(&rom, &bytes).unwrap();
    assert_eq!(loaded.nuyen, 0);
}

#[test]
#[ignore = "requires the user-supplied ROM"]
fn earned_stash_post_objective_stoker_rolls_spawn_a_real_fight_then_allow_room_and_save() {
    earned_stoker_fight(false);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn earned_stoker_route_puts_away_joshuas_gun_fights_with_fists_rests_trains_and_reloads() {
    earned_stoker_fight(true);
}
fn earned_stoker_fight(barehanded: bool) {
    let rom = rom();
    let mut game = earn_belongings(&rom);
    let stoker = door(&game, 2);
    let mut fought = false;
    let mut rested = false;
    for visit in 0..32 {
        for _ in 0..24 {
            game.update(&rom, input::DOWN).unwrap();
        }
        walk(&mut game, &rom, stoker, true);
        assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf4aa0);
        choose_destination(&mut game, &rom, 233);
        if game.dialogue.as_ref().unwrap().node == 0xf4acc {
            assert_eq!(game.npcs.iter().filter(|a| a.raw[0x57] == 0).count(), 2);
            assert_eq!(game.encounters.count, 3);
            capture(&mut game, &rom, "stoker-ambush");
            press(&mut game, &rom, input::A);
            if barehanded {
                game.show_inventory(&rom).unwrap();
                choose_destination(&mut game, &rom, 0); // Put away the equipped gun.
                assert_eq!((game.party[0].raw[0x56], game.party[0].raw[0x57]), (127, 0));
                choose_destination(&mut game, &rom, 8);
            }
            if !game.targeting.enabled {
                press(&mut game, &rom, input::C);
            }
            let mut captured_punch = false;
            for _ in 0..4000 {
                game.update(&rom, input::A).unwrap();
                if barehanded && !captured_punch && game.combat.punch_hits > 0 {
                    capture(&mut game, &rom, "stoker-fists-strike");
                    captured_punch = true;
                }
                assert!(!game.defeated, "party defeated during Stoker encounter");
                if game.dialogue.is_some() {
                    panic!("unexpected recovery during Stoker fight");
                }
                if game.encounters.count == 0 && !game.combat.has_live() {
                    break;
                }
            }
            assert_eq!(
                game.encounters.count,
                0,
                "Stoker enemies remain: {:?}",
                game.npcs
                    .iter()
                    .map(|a| (
                        a.name(),
                        a.position(),
                        a.raw[0x4c],
                        a.raw[0x9c],
                        a.word(0x16)
                    ))
                    .collect::<Vec<_>>()
            );
            if barehanded {
                assert!(game.combat.punches > 0);
                assert!(game.combat.punch_hits > 0);
                assert_eq!(game.combat.shots, 0);
                assert_eq!(game.party[0].raw[0x4e], 100); // Stored ammunition is untouched by fists.
                capture(&mut game, &rom, "stoker-fists-fight-complete");
            } else {
                assert!(game.combat.shots > 0 || game.combat.companion_shots > 0);
                capture(&mut game, &rom, "stoker-fight-complete");
            }
            fought = true;
            eprintln!(
                "Stoker fight visit {visit}, tick {}, NPC punches {}, NPC shots {}, party {:?}",
                game.tick,
                game.combat.npc_punches,
                game.combat.npc_shots,
                game.party
                    .iter()
                    .map(|a| (a.raw[0x4c], a.raw[0x9c], a.raw[0x10]))
                    .collect::<Vec<_>>()
            );
        } else {
            assert_eq!(game.dialogue.as_ref().unwrap().node, 0xf4ab4);
            choose_destination(&mut game, &rom, 0xf4ad4);
            let balance = game.nuyen;
            choose_destination(&mut game, &rom, 6);
            assert_eq!(game.nuyen, balance - 20);
            choose_destination(&mut game, &rom, 0);
            choose_destination(&mut game, &rom, 0);
            capture(&mut game, &rom, "stoker-room-karma");
            if fought {
                let karma = game.party[0].raw[0x8d];
                choose_destination(&mut game, &rom, 0);
                choose_destination(&mut game, &rom, 0);
                choose_destination(&mut game, &rom, 0);
                choose_destination(&mut game, &rom, 0);
                choose_destination(&mut game, &rom, 0);
                assert_eq!(game.party[0].raw[0x77], 5);
                assert_eq!(game.party[0].raw[0x8d], karma - 5);
                capture(&mut game, &rom, "stoker-karma-improved");
                choose_destination(&mut game, &rom, 20);
                choose_destination(&mut game, &rom, 3);
            }
            choose_destination(&mut game, &rom, 1);
            choose_destination(&mut game, &rom, 0);
            if fought {
                rested = true;
            }
        }
        if fought && rested {
            break;
        }
    }
    assert!(fought && rested);
    for _ in 0..100 {
        game.update(&rom, 0).unwrap();
    }
    assert!(
        game.save_ready(),
        "party {:?}",
        game.party
            .iter()
            .map(|a| (a.raw[0x4c], a.raw[0x9c], a.raw[0x10]))
            .collect::<Vec<_>>()
    );
    let directory = std::env::temp_dir().join(format!(
        "shadowrealm-stoker-fight-{}-{barehanded}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    game.write_save_slot(&rom, &directory, 1).unwrap();
    let (mut loaded, _) = Game::load_slot(&rom, &directory, 1).unwrap();
    assert_eq!(loaded.nuyen, game.nuyen);
    assert_eq!(loaded.recovery_flags, 1);
    assert_eq!(loaded.jobs.completed, 6);
    assert_eq!(loaded.party[0].raw[0x77], 5);
    assert_eq!(loaded.party[0].raw[0x8d], game.party[0].raw[0x8d]);
    if barehanded {
        assert_eq!(
            (
                loaded.party[0].raw[0x56],
                loaded.party[0].raw[0x57],
                loaded.party[0].raw[0x4e]
            ),
            (127, 0, 100)
        );
        capture(&mut loaded, &rom, "stoker-fists-checkpoint-resumed");
        eprintln!("Earned barehanded route: player punches {}, hits {}, companion punches {}, NPC punches {}, ticks {}",game.combat.punches,game.combat.punch_hits,game.combat.companion_punches,game.combat.npc_punches,game.tick);
    } else {
        capture(&mut loaded, &rom, "stoker-checkpoint-resumed");
    }
    std::fs::remove_dir_all(directory).unwrap();
    eprintln!("Earned Stoker loop: tick {}, balance {}, punches {}, shots {}, party survived and checkpoint loaded",game.tick,game.nuyen,game.combat.npc_punches,game.combat.shots);
}
