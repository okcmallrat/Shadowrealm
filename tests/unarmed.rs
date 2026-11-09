use shadowrealm::{
    avoidance::State, damage::DamageWorld, game::Game, npc, party::Runner, stoker, unarmed,
    weapons::LaunchWorld,
};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
#[test]
fn unarmed_and_stoker_catalogs_reject_invented_rules() {
    for json in [
        unarmed::Catalog::JSON.replace("0x55c76", "0x55c78"),
        unarmed::Catalog::JSON.replacen('{', "{\"automatic_hit\":true,", 1),
    ] {
        assert!(unarmed::Catalog::parse(&json, &[]).is_err());
    }
    for json in [
        stoker::Catalog::JSON.replace("\"room_fee\": 20", "\"room_fee\": 0"),
        stoker::Catalog::JSON.replace("0x249c4", "0x249c6"),
    ] {
        assert!(stoker::Catalog::parse(&json, &[]).is_err());
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn human_unarmed_npc_reaches_and_punches_through_live_game_dispatch() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0]).unwrap();
    // Explicit close attacker fixture; the earned route separately uses action 233.
    let mut cursor = 0x1d6e2a + 8;
    let mut rng = State::default();
    let mut a = npc::initialize(
        &r,
        0x1d665a,
        0x1e300,
        3,
        0,
        0,
        2,
        false,
        &mut cursor,
        &mut rng,
    )
    .unwrap();
    let p = g.party[0].position();
    a.set_position(p.0.wrapping_add(0x100), p.1);
    a.raw[0x9e] |= 0x40;
    a.raw[0x10] |= 0x20;
    g.npcs.push(a);
    g.combat.actors_changed();
    for _ in 0..200 {
        g.update(&r, 0).unwrap();
        g.render(&r).unwrap();
        if g.combat.npc_punches > 0 && (g.party[0].raw[0x9c] < 10 || g.party[0].raw[0x4c] < 10) {
            break;
        }
    }
    assert!(g.combat.npc_punches > 0);
    assert!(g.party[0].raw[0x9c] < 10 || g.party[0].raw[0x4c] < 10);
    assert_eq!(g.combat.npc_shots, 0);
    assert_eq!(g.combat.shots, 0);
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn exhausted_condition_falls_with_health_preserved_and_enters_defeat_recovery() {
    let r = rom();
    for template in 0..3 {
        let mut g = Game::load(&r, 2, &[template]).unwrap();
        g.party[0].raw[0x9c] = 0; // Explicit incapacitation boundary.
        g.update(&r, 0).unwrap();
        assert_eq!(g.party[0].raw[0x4c], 10);
        assert_ne!(g.party[0].raw[0x10] & 0x10, 0);
        assert_eq!(g.combat.damage.defeat_timer, 59);
        for _ in 0..90 {
            g.update(&r, 0).unwrap();
            if g.dialogue.is_some() {
                break;
            }
        }
        assert!(g.dialogue.is_some());
        assert!(g.defeated || g.transition.is_some() || g.party[0].raw[0x10] & 0x10 != 0);
        g.render(&r).unwrap();
        assert_eq!(g.nuyen, 18);
        assert_eq!(g.party[0].raw[0x4c], 10);
        assert_eq!(g.party[0].raw[0x9c], 8);
        g.update(&r, 0).unwrap();
        g.update(&r, shadowrealm::input::A).unwrap();
        for _ in 0..120 {
            g.update(&r, 0).unwrap();
            if g.transition.is_none() {
                break;
            }
        }
        assert!(!g.defeated);
        assert_eq!(g.party[0].template, template);
        assert_eq!(g.party[0].raw[0x9c], 8);
        g.render(&r).unwrap();
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn failed_unarmed_pose_assets_leave_attack_actors_fx_world_and_rng_intact() {
    let r = rom();
    let mut actors = (0..10)
        .map(|slot| {
            let mut a = Runner::load(&r, 0, 0).unwrap();
            a.set_word(0x3e, slot);
            a.set_position(0x1800, 0x1800);
            a
        })
        .collect::<Vec<_>>();
    actors[3].set_word(0xe, 8);
    actors[3].raw[0x57] = 0;
    actors[3].raw[0x56] = 127;
    let before = actors.iter().map(|a| a.raw).collect::<Vec<_>>();
    let mut pool = [[0; 256]; 6];
    let mut launch = LaunchWorld::default();
    let mut damage = DamageWorld::default();
    let mut rng = State { seed: 123 };
    let mut broken = r.clone();
    broken[0xc0a2..0xc0b2].fill(255);
    assert!(unarmed::strike(
        &broken,
        &mut actors,
        3,
        0,
        &mut pool,
        &mut launch,
        &mut damage,
        &mut rng
    )
    .is_err());
    assert_eq!(actors.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    assert_eq!(pool, [[0; 256]; 6]);
    assert_eq!(rng.seed, 123);
    assert_eq!(launch.engaged, 0);
}
