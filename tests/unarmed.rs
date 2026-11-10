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
        unarmed::Catalog::JSON.replace("0xad1a", "0xad1c"),
        unarmed::Catalog::JSON.replace("0x15c84", "0x15c86"),
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

fn press(g: &mut Game, r: &[u8], key: u8) {
    g.update(r, 0).unwrap();
    g.update(r, key).unwrap();
}
fn choose_inventory(g: &mut Game, r: &[u8], row: usize) {
    for _ in 0..9 {
        if g.dialogue.as_ref().unwrap().selected == row {
            break;
        }
        press(g, r, shadowrealm::input::DOWN);
    }
    assert_eq!(g.dialogue.as_ref().unwrap().selected, row);
    press(g, r, shadowrealm::input::A);
}
fn attacker(r: &[u8], position: (u16, u16)) -> Runner {
    let mut cursor = 0x1d6e2a + 8;
    let mut a = npc::initialize(
        r,
        0x1d665a,
        0x1e300,
        3,
        0,
        0,
        2,
        false,
        &mut cursor,
        &mut State::default(),
    )
    .unwrap();
    a.set_position(position.0, position.1);
    a.raw[0x9e] |= 0x40;
    a.raw[0x10] |= 0x20;
    a
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn all_solo_archetypes_put_away_their_gun_swing_and_reload_a_fists_checkpoint() {
    use shadowrealm::{input, newgame::Archetype};
    let r = rom();
    for class in Archetype::ALL {
        let mut g = Game::new_game(&r, class).unwrap();
        press(&mut g, &r, input::A);
        let gun = g.party[0].raw[0x57];
        let ammo = g.party[0].raw[0x4e];
        let money = g.nuyen;
        g.show_inventory(&r).unwrap();
        choose_inventory(&mut g, &r, 0); // Original equipped-item toggle, no injected equipment.
        assert_eq!((g.party[0].raw[0x56], g.party[0].raw[0x57]), (127, 0));
        let stored = g.party[0].raw[0x4e];
        choose_inventory(&mut g, &r, 8);
        for _ in 0..200 {
            g.update(&r, input::A).unwrap();
        }
        assert!(g.combat.punches > 1);
        assert_eq!(g.combat.shots, 0);
        assert_eq!(g.combat.launch.engaged, 0); // Targetless swings do not alert or roll.
        assert_eq!(g.party[0].raw[0x4e], stored);
        assert_eq!(g.nuyen, money);
        for _ in 0..100 {
            g.update(&r, 0).unwrap();
        }
        assert!(g.save_ready());
        let bytes = g.save_bytes(&r).unwrap();
        let mut resumed = Game::from_save_bytes(&r, &bytes).unwrap();
        assert_eq!(
            (
                resumed.party[0].raw[0x56],
                resumed.party[0].raw[0x57],
                resumed.party[0].raw[0x4e]
            ),
            (127, 0, stored)
        );
        resumed.show_inventory(&r).unwrap();
        choose_inventory(&mut resumed, &r, 0);
        assert_eq!(
            (resumed.party[0].raw[0x57], resumed.party[0].raw[0x4e]),
            (gun, ammo)
        );
        resumed.render(&r).unwrap();
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn controlled_fists_and_two_fists_companion_apply_condition_or_cyberware_damage() {
    use shadowrealm::input;
    let r = rom();
    for templates in [vec![0], vec![1], vec![2], vec![0, 9]] {
        let controlled = templates.len() == 1;
        let mut g = Game::load(&r, 2, &templates).unwrap();
        let slot = usize::from(!controlled);
        if controlled {
            g.show_inventory(&r).unwrap();
            choose_inventory(&mut g, &r, 0);
            choose_inventory(&mut g, &r, 8);
        }
        let p = g.party[slot].position();
        let mut enemy = attacker(&r, (p.0.wrapping_add(80), p.1));
        if controlled && templates[0] != 0 {
            // Explicit low-defense opponent isolates nonlethal damage for the
            // low-Melee archetypes; the earned route uses untouched NPC records.
            for off in [0x76, 0x77, 0x78, 0x7b, 0x7c, 0x85] {
                enemy.raw[off] = 0;
            }
        }
        g.npcs.push(enemy);
        g.combat.actors_changed();
        if controlled {
            press(&mut g, &r, input::C);
            assert!(g.selected_fire_probe(&r).unwrap().is_none());
        }
        for _ in 0..1500 {
            g.update(&r, if controlled { input::A } else { 0 }).unwrap();
            if g.npcs[0].raw[0x9c] < 10 || g.npcs[0].raw[0x4c] < 10 {
                break;
            }
        }
        if g.party[slot].word(0x90) & 12 != 0 {
            assert!(
                g.npcs[0].raw[0x4c] < 10,
                "cyberware attack, party {templates:?}"
            );
            assert_eq!(g.npcs[0].raw[0x9c], 10);
        } else {
            assert!(
                g.npcs[0].raw[0x9c] < 10,
                "condition attack, party {templates:?}"
            );
            assert_eq!(g.npcs[0].raw[0x4c], 10);
        }
        assert_eq!(g.combat.shots + g.combat.companion_shots, 0);
        if controlled {
            assert!(g.combat.punches > 0);
        } else {
            assert!(g.combat.companion_punches > 0);
            assert!(g.combat.companion_deferred[slot].is_none());
        }
        g.render(&r).unwrap();
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn exhausted_companion_gun_enters_live_unarmed_pursuit_instead_of_deferring() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0, 12]).unwrap();
    // Explicit ammunition boundary. Both the depletion conversion and attack
    // then run through ordinary Game scheduling, steering, pose and damage.
    g.party[1].raw[0x4e] = 0;
    g.party[1].raw[0x4f] = 0;
    let p = g.party[1].position();
    g.npcs.push(attacker(&r, (p.0.wrapping_add(0x400), p.1)));
    g.combat.actors_changed();
    for _ in 0..1500 {
        g.update(&r, 0).unwrap();
        if g.combat.companion_punches > 0 {
            break;
        }
    }
    assert_eq!((g.party[1].raw[0x56], g.party[1].raw[0x57]), (127, 0));
    assert!(g.combat.companion_punches > 0);
    assert_eq!(g.combat.companion_shots, 0);
    assert!(g.combat.companion_deferred[1].is_none());
    g.render(&r).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn failed_controlled_swing_assets_preserve_actors_fx_alert_rewards_and_rng() {
    use shadowrealm::{combat_runtime::Runtime, input};
    let r = rom();
    let g = Game::load(&r, 2, &[0]).unwrap();
    let mut actors = (0..10)
        .map(|slot| {
            let mut a = g.party[0].clone();
            a.set_word(0x3e, slot);
            if slot > 0 {
                a.raw[0x10] = 0;
            }
            a
        })
        .collect::<Vec<_>>();
    actors[0].raw[0x56] = 127;
    actors[0].raw[0x57] = 0;
    actors[0].raw[0xa2..0xa6].fill(0);
    actors[0].set_word(0xdc, 65535);
    let before = actors.iter().map(|a| a.raw).collect::<Vec<_>>();
    let mut runtime = Runtime::default();
    let mut random = State { seed: 123 };
    let mut broken = r.clone();
    broken[0xc0a2..0xc0b2].fill(255);
    assert!(runtime
        .player(
            &broken,
            &g.collision,
            &mut actors,
            0,
            input::Controller {
                held: input::A,
                pressed: input::A
            },
            &mut random
        )
        .is_err());
    assert_eq!(actors.iter().map(|a| a.raw).collect::<Vec<_>>(), before);
    assert_eq!(runtime.pool, [[0; 256]; 6]);
    assert_eq!(runtime.launch.engaged, 0);
    assert_eq!(runtime.damage.reward_count, 0);
    assert_eq!(runtime.punches, 0);
    assert_eq!(random.seed, 123);
}
