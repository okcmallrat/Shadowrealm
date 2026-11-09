use shadowrealm::{
    game::Game,
    input,
    progression::{pay_taxi, Catalog},
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn press(g: &mut Game, r: &[u8], key: u8) {
    g.update(r, 0).unwrap();
    g.update(r, key).unwrap();
}
fn choose(g: &mut Game, r: &[u8], slot: usize) {
    for _ in 0..8 {
        if g.dialogue.as_ref().unwrap().selected == slot {
            break;
        }
        press(g, r, input::DOWN);
    }
    press(g, r, input::A);
}
#[test]
fn taxi_funds_and_visa_boundaries_preserve_original_free_trip_rule() {
    assert_eq!(pay_taxi(2, 40, 40, 512).balance, 0);
    let free = pay_taxi(2, 40, 39, 512);
    assert!(free.travelled);
    assert_eq!((free.balance, free.carried, free.node), (39, 512, 0xfe610));
    let signed = pay_taxi(2, 40, 0x80000000, 512);
    assert_eq!((signed.balance, signed.node), (0x80000000, 0xfe610));
    let denied = pay_taxi(6, 100, 1000, 512);
    assert!(!denied.travelled);
    assert_eq!((denied.balance, denied.carried), (1000, 512));
    let visa = pay_taxi(6, 100, 1000, 514);
    assert!(visa.travelled);
    assert_eq!((visa.balance, visa.carried), (900, 512));
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn taxi_gate_cancel_same_district_and_visa_do_not_charge_or_move() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0, 3, 10]).unwrap();
    let positions = g.party.iter().map(|a| a.position()).collect::<Vec<_>>();
    let funds = g.nuyen;
    g.call_taxi(&r).unwrap();
    choose(&mut g, &r, 4);
    assert_eq!(g.dialogue.as_ref().unwrap().node, 0xfe620);
    press(&mut g, &r, input::A);
    assert!(g.transition.is_none());
    assert_eq!(g.map_id(), 2);
    g.recovery_flags = 1;
    g.call_taxi(&r).unwrap();
    choose(&mut g, &r, 2);
    assert_eq!(g.dialogue.as_ref().unwrap().node, 0xfe628);
    press(&mut g, &r, input::A);
    g.call_taxi(&r).unwrap();
    choose(&mut g, &r, 0);
    choose(&mut g, &r, 0);
    assert_eq!(g.dialogue.as_ref().unwrap().node, 0xfe618);
    press(&mut g, &r, input::A);
    g.call_taxi(&r).unwrap();
    choose(&mut g, &r, 7);
    assert!(g.dialogue.is_none());
    assert_eq!(g.nuyen, funds);
    assert_eq!(g.completed_transitions, 0);
    assert_eq!(
        g.party.iter().map(|a| a.position()).collect::<Vec<_>>(),
        positions
    );
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn paid_free_and_armband_city_trips_preserve_party_jobs_and_story() {
    let r = rom();
    for (funds, armband) in [(1000, false), (19, false), (1000, true)] {
        let mut g = Game::load(&r, 2, &[0, 3, 10]).unwrap();
        g.recovery_flags = 1;
        g.nuyen = funds;
        if armband {
            g.party[0].raw[0x8f] |= 0x20;
        } else {
            g.party[0].raw[0x8f] &= !0x20;
        }
        g.jobs.raw = [0, 2, 2, 1, 0, 2, 11, 1, 0, 45];
        g.jobs.accepted = true;
        g.jobs.carried = 512;
        g.story.leads = 0x1c;
        let equipment = g
            .party
            .iter()
            .map(|a| [a.raw[0x4c..0x92].to_vec(), a.raw[0x9a..0xa2].to_vec()].concat())
            .collect::<Vec<_>>();
        let job = g.jobs.clone();
        g.call_taxi(&r).unwrap();
        choose(&mut g, &r, 4);
        if !armband {
            choose(&mut g, &r, 0);
        }
        assert_eq!(
            g.nuyen,
            if armband || funds < 20 {
                funds
            } else {
                funds - 20
            }
        );
        press(&mut g, &r, input::A);
        for _ in 0..100 {
            g.update(&r, 0).unwrap();
            if g.transition.is_none() {
                break;
            }
        }
        assert_eq!(g.map_id(), 0);
        assert_eq!(g.completed_transitions, 1);
        assert!(g.saved_exterior.is_none());
        assert_eq!(g.jobs, job);
        assert_eq!(g.recovery_flags, 1);
        assert_eq!(g.story.leads, 0x1c);
        for (a, before) in g.party.iter().zip(equipment) {
            assert_eq!(
                [a.raw[0x4c..0x92].to_vec(), a.raw[0x9a..0xa2].to_vec()].concat(),
                before
            );
        }
        g.render(&r).unwrap();
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn invalid_progression_records_and_failed_arrival_assets_leave_payment_uncommitted() {
    let r = rom();
    for json in [
        Catalog::JSON.replace("0x213fa", "0x213fc"),
        Catalog::JSON.replacen('{', "{\"invented_reward\":9,", 1),
        Catalog::JSON.replace("0xf48e0", "0xf48f4"),
    ] {
        assert!(Catalog::parse(&json, &r).is_err());
    }
    let mut g = Game::load(&r, 2, &[0]).unwrap();
    g.recovery_flags = 1;
    g.nuyen = 1000;
    g.call_taxi(&r).unwrap();
    choose(&mut g, &r, 4);
    g.update(&r, 0).unwrap();
    let node = g.dialogue.as_ref().unwrap().node;
    let mut broken = r.clone();
    broken[0x3232..0x3236].copy_from_slice(&0x1fffffu32.to_be_bytes());
    assert!(g.update(&broken, input::A).is_err());
    assert_eq!(g.nuyen, 1000);
    assert_eq!(g.map_id(), 2);
    assert!(g.transition.is_none());
    assert_eq!(g.dialogue.as_ref().unwrap().node, node);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn wilderness_route_is_guarded_for_both_paid_and_armband_visa_owners() {
    let r = rom();
    for armband in [false, true] {
        let mut g = Game::load(&r, 2, &[0]).unwrap();
        g.recovery_flags = 1;
        g.jobs.carried = 2;
        g.nuyen = 1000;
        if armband {
            g.party[0].raw[0x8f] |= 0x20;
        }
        g.call_taxi(&r).unwrap();
        choose(&mut g, &r, 0);
        if !armband {
            choose(&mut g, &r, 0);
        }
        assert_eq!(g.map_id(), 2);
        assert_eq!(g.nuyen, 1000);
        assert_eq!(g.jobs.carried, 2);
        assert!(g.transition.is_none());
        assert_eq!(g.combat.status, "Salish travel is not available yet");
    }
}
