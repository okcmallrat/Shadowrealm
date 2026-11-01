use shadowrealm::{
    collision::Contact,
    game::Game,
    input,
    transitions::{TransitionPlan, TransitionStage},
};
use std::path::PathBuf;
fn rom() -> Vec<u8> {
    let path = std::env::var_os("SHADOWREALM_TEST_ROM")
        .expect("set SHADOWREALM_TEST_ROM for ROM-backed transition tests");
    std::fs::read(PathBuf::from(path)).expect("read the supplied ROM")
}
fn persistent_state(r: &shadowrealm::party::Runner) -> Vec<u8> {
    // Compact save fields plus portrait/identity data. Navigation targets,
    // timers and collision samples are working state and now change while walking.
    r.raw[0xe..0x10]
        .iter()
        .chain(&r.raw[0x2c..0x2e])
        .chain(&r.raw[0x4c..0x92])
        .chain(&r.raw[0x9a..0xa2])
        .copied()
        .collect()
}
fn press(game: &mut Game, rom: &[u8], button: u8) {
    game.update(rom, 0).unwrap();
    game.update(rom, button).unwrap();
}
fn at_trent(rom: &[u8], templates: &[usize], nuyen: u32) -> Game {
    let mut game = Game::load(rom, 0, templates).unwrap();
    game.nuyen = nuyen;
    game.party[0].set_position(528 * 16, 144 * 16);
    game.party[0].set_word(0x2e, 0);
    game.camera = (392, 32);
    for _ in 0..20 {
        game.update(rom, input::UP).unwrap();
        if game.dialogue.is_some() {
            break;
        }
    }
    press(&mut game, rom, input::A);
    drive_until_settled(&mut game, rom, 0, 10);
    game.party[0].set_position(112 * 16, 85 * 16);
    game.party[0].set_word(0x2e, 0);
    for _ in 0..8 {
        game.update(rom, input::UP).unwrap();
        if game.dialogue.is_some() {
            break;
        }
    }
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfeb4c);
    game
}
fn drive_until_settled(game: &mut Game, rom: &[u8], held: u8, target: usize) {
    let before = game.completed_transitions;
    let mut reached = false;
    for _ in 0..300 {
        game.update(rom, held).unwrap();
        if game.completed_transitions > before {
            assert_eq!(game.map_id(), target);
            reached = true;
        }
        if reached && game.transition.is_none() {
            let target_scene = shadowrealm::map_render::MapScene::load(rom, target).unwrap();
            assert_eq!(game.scene.palette, target_scene.palette);
            return;
        }
        if let Some(active) = game.transition {
            if active.stage != TransitionStage::FadeOut {
                assert_eq!(
                    game.camera,
                    (
                        active.plan.camera_fixed.0 >> 4,
                        active.plan.camera_fixed.1 >> 4
                    )
                );
                for (runner, expected) in game.party.iter().zip(active.plan.positions) {
                    assert_eq!(runner.position(), expected);
                }
            }
        }
        if game.tick % 16 == 0 || game.transition.is_some_and(|a| a.ticks == 0) {
            game.render(rom).unwrap();
        }
    }
    panic!("transition to map {target} did not settle");
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn exterior_round_trips_preserve_party_and_reset_assets() {
    let rom = rom();
    for (east, target, outward, inward) in [
        (false, 3, input::LEFT, input::RIGHT),
        (true, 5, input::RIGHT, input::LEFT),
    ] {
        let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
        let trigger = game
            .collision
            .rectangles
            .iter()
            .find(|r| r.code & 31 == 16 && if east { r.left > 1000 } else { r.left == 0 })
            .unwrap();
        let x = if east {
            trigger.left - 16
        } else {
            trigger.right + 16
        };
        let y = (trigger.top + trigger.bottom) / 2;
        game.party[0].set_position(x * 16, y * 16);
        game.party[0].set_word(0x2e, if east { 0x4000 } else { 0xc000 });
        game.camera = (if east { 1640 } else { 0 }, y.saturating_sub(112));
        game.party[0].raw[0x4c] = 7;
        game.party[1].raw[0x4e] = 2;
        game.refresh_hud(&rom).unwrap();
        assert_eq!(
            game.collision.contact(game.party[0].position()).unwrap(),
            Contact::Clear
        );
        let stats: Vec<_> = game.party.iter().map(persistent_state).collect();
        drive_until_settled(&mut game, &rom, outward, target);
        drive_until_settled(&mut game, &rom, inward, 0);
        assert_eq!(game.completed_transitions, 2);
        assert_eq!(game.selected, 0);
        for (runner, expected) in game.party.iter().zip(&stats) {
            assert_eq!(&persistent_state(runner), expected);
        }
    }
}
#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn unsupported_door_does_not_start_a_transition() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0]).unwrap();
    assert!(TransitionPlan::exterior(&rom, 0, 0xc0, 0)
        .unwrap()
        .is_none());
    let doorway = game
        .collision
        .rectangles
        .iter()
        .find(|r| r.code == 0xc0)
        .unwrap();
    game.party[0].set_position(
        ((doorway.left + doorway.right) / 2) * 16,
        ((doorway.top + doorway.bottom) / 2) * 16,
    );
    game.party[0].set_word(0x2e, 0);
    game.update(&rom, input::UP).unwrap();
    assert_eq!(game.pending_transition, Some(0xc0));
    assert!(game.transition.is_none());
    assert!(game.blocked);
    assert_eq!(game.completed_transitions, 0);
    assert_eq!(game.map_id(), 0);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn space_needle_exposition_entry_and_return_preserve_state() {
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
    game.party[0].set_position(528 * 16, 144 * 16);
    game.party[0].set_word(0x2e, 0);
    game.camera = (392, 32);
    game.party[0].raw[0x4c] = 7;
    game.party[1].raw[0x4e] = 2;
    game.knowledge = shadowrealm::dialogue_data::Knowledge {
        runner_known: 4,
        contacts: 1 << 12,
    };
    let stats: Vec<_> = game.party.iter().map(persistent_state).collect();
    for _ in 0..20 {
        game.update(&rom, input::UP).unwrap();
        if game.dialogue.is_some() {
            break;
        }
    }
    let exposition = game
        .dialogue
        .as_ref()
        .expect("Space Needle entry exposition");
    assert_eq!(exposition.node, 0x110b5e);
    assert!(exposition.text.contains("Space Needle"));
    let return_plan = TransitionPlan::space_needle(
        &rom,
        &shadowrealm::maps::MapDescriptor::read(&rom, 0).unwrap(),
        0xc6,
        (game.camera.0 * 16, game.camera.1 * 16),
        game.party[0].position(),
    )
    .unwrap()
    .unwrap()
    .exterior_return
    .unwrap();
    let before = game.party[0].position();
    for _ in 0..8 {
        game.update(&rom, input::UP).unwrap();
    }
    assert_eq!(game.party[0].position(), before);
    assert_eq!(game.map_id(), 0);
    game.render(&rom).unwrap();
    game.select(&rom, 1).unwrap();
    assert_eq!(game.selected, 0);
    game.update(&rom, input::A).unwrap();
    assert!(game.dialogue.is_none());
    drive_until_settled(&mut game, &rom, 0, 10);
    assert_eq!(game.npcs.len(), 1);
    assert_eq!(game.npcs[0].position(), (0x700, 0x500));
    game.render(&rom).unwrap();
    drive_until_settled(&mut game, &rom, input::DOWN, 0);
    assert!(game.saved_exterior.is_none());
    assert!(game.npcs.is_empty());
    assert_eq!(
        game.camera,
        (
            return_plan.camera_fixed.0 >> 4,
            return_plan.camera_fixed.1 >> 4
        )
    );
    for ((runner, position), expected) in game.party.iter().zip(return_plan.positions).zip(&stats) {
        assert_eq!(runner.position(), position);
        assert_eq!(&persistent_state(runner), expected);
    }
    // Reusing the door must ask again and create a fresh return context.
    // Turning also walks. Curve back from the south-facing arrival, then align
    // with the door after contact deflections, using only controller input
    // (no pose/map reassignment).
    for (count, held) in [(16, input::UP), (15, input::RIGHT), (30, input::UP)] {
        for _ in 0..count {
            game.update(&rom, held).unwrap();
            if game.dialogue.is_some() {
                break;
            }
        }
        if game.dialogue.is_some() {
            break;
        }
    }
    assert!(game.dialogue.is_some());
    assert!(game.transition.is_none());
    game.update(&rom, input::A).unwrap();
    drive_until_settled(&mut game, &rom, 0, 10);
    assert_eq!(game.npcs.len(), 1);
    drive_until_settled(&mut game, &rom, input::DOWN, 0);
    assert_eq!(game.completed_transitions, 4);
    assert!(game.saved_exterior.is_none());
    assert_eq!(
        game.knowledge,
        shadowrealm::dialogue_data::Knowledge {
            runner_known: 4,
            contacts: 1 << 12
        }
    );
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn trent_contact_choices_and_knowledge_flags() {
    fn press(game: &mut Game, rom: &[u8], button: u8) {
        game.update(rom, 0).unwrap();
        game.update(rom, button).unwrap();
    }
    let rom = rom();
    let mut game = Game::load(&rom, 0, &[0, 3, 10]).unwrap();
    game.party[0].set_position(528 * 16, 144 * 16);
    game.party[0].set_word(0x2e, 0);
    game.camera = (392, 32);
    for _ in 0..20 {
        game.update(&rom, input::UP).unwrap();
        if game.dialogue.is_some() {
            break;
        }
    }
    press(&mut game, &rom, input::A);
    drive_until_settled(&mut game, &rom, 0, 10);
    // Seed just outside the real unlinked NPC contact radius, then walk in.
    // Entry/exit navigation is covered independently; this is an interaction fixture.
    game.party[0].set_position(112 * 16, 85 * 16);
    game.party[0].set_word(0x2e, 0);
    for _ in 0..8 {
        game.update(&rom, input::UP).unwrap();
        if game.dialogue.is_some() {
            break;
        }
    }
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfeb4c);
    assert_eq!(game.knowledge.runner_known, 4);
    let position = game.party[0].position();
    let clock = game.npcs[0].clock.frame;
    for _ in 0..8 {
        game.update(&rom, input::UP).unwrap();
    }
    assert_eq!(game.party[0].position(), position);
    assert_eq!(game.npcs[0].clock.frame, clock);
    game.select(&rom, 1).unwrap();
    assert_eq!(game.selected, 0);
    assert_eq!(game.knowledge.contacts, 0);
    press(&mut game, &rom, input::A); // information
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfeba0);
    press(&mut game, &rom, input::A); // original action BE: biography and known bit
    assert!(game
        .dialogue
        .as_ref()
        .unwrap()
        .text
        .contains("My name is Trent"));
    game.render(&rom).unwrap();
    press(&mut game, &rom, input::RIGHT);
    game.render(&rom).unwrap();
    press(&mut game, &rom, input::A);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfec5c);
    press(&mut game, &rom, input::B); // questions, slot 2
    press(&mut game, &rom, input::B); // original action E5: contact knowledge
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfecc4);
    assert_eq!(game.knowledge.contacts, 1 << 12);
    assert_eq!(game.knowledge.runner_known, 4);
    game.render(&rom).unwrap();
    press(&mut game, &rom, input::C); // original action 4: topic menu, opinion zero
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfebf4);
    press(&mut game, &rom, input::C);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xff970);
    press(&mut game, &rom, input::A);
    assert!(game.dialogue.is_none());
    assert!(game.transition.is_none());
    assert_eq!(game.map_id(), 10);
    assert_eq!(game.knowledge.contacts, 1 << 12);
    game.update(&rom, 0).unwrap();
    assert!(game.dialogue.is_none()); // original scheduling cooldown
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn trent_hiring_payments_capacity_and_contracts() {
    let rom = rom();
    for (templates, funds, button, price, success) in [
        (&[0][..], 285, input::A, 285, true),
        (&[0, 3][..], 2850, input::B, 2850, true),
        (&[0][..], 284, input::A, 285, false),
        (&[0, 3][..], 2849, input::B, 2850, false),
        (&[0, 3, 10][..], 99999, input::A, 285, false),
    ] {
        let mut game = at_trent(&rom, templates, funds);
        game.party[0].raw[0x4c] = 7;
        game.party[0].raw[0x4e] = 2;
        let before: Vec<_> = game.party.iter().map(|r| r.raw).collect();
        let npc = game.npcs[0].position();
        press(&mut game, &rom, input::B);
        assert_eq!(game.dialogue.as_ref().unwrap().node, 0xfecfc);
        press(&mut game, &rom, input::A);
        assert_eq!(game.nuyen, funds);
        assert_eq!(game.party.len(), templates.len());
        if templates.len() == 3 {
            assert_eq!(game.dialogue.as_ref().unwrap().node, 0xff988);
        } else {
            assert_eq!(game.dialogue.as_ref().unwrap().node, 0xff958);
            assert!(game.dialogue.as_ref().unwrap().text.contains("2850"));
            game.render(&rom).unwrap();
            if button == input::B {
                press(&mut game, &rom, input::DOWN);
            }
            press(&mut game, &rom, input::A); // same selector/confirm path as desktop
        }
        assert_eq!(game.party.len(), templates.len() + usize::from(success));
        for (i, raw) in before.iter().enumerate() {
            assert_eq!(&game.party[i].raw, raw);
        }
        if success {
            assert_eq!(game.nuyen, funds - price);
            assert!(game.npcs.is_empty());
            let member = game.party.last().unwrap();
            assert_eq!(member.name(), "Trent");
            assert_eq!(member.position(), npc);
            assert_eq!(member.raw[0x4c], 10);
            assert_eq!(member.raw[0x9c], 10);
            assert_eq!(member.raw[0x8f], if button == input::A { 0 } else { 255 });
            let slot = game.party.len() - 1;
            assert_eq!(member.word(0x3e), slot as u16);
            assert_eq!(member.word(0x20), [1, 0x19, 0x31][slot]);
            for _ in 0..8 {
                game.update(&rom, input::A).unwrap();
            }
            assert_eq!(game.nuyen, funds - price);
            press(&mut game, &rom, input::A); // acknowledge success
            game.select(&rom, slot).unwrap();
            assert_eq!(game.party[slot].raw[0x10] & 2, 2);
            assert_eq!(game.party[0].raw[0x10] & 2, 0);
            // Door navigation is seeded separately; the hired runner uses normal
            // input to exit, and the next visit must not respawn recruited Trent.
            game.party[slot].set_position(240 * 16, 204 * 16);
            game.party[slot].set_word(0x2e, 0x8000);
            drive_until_settled(&mut game, &rom, input::DOWN, 0);
            assert_eq!(game.nuyen, funds - price);
            assert_eq!(
                game.party[slot].raw[0x8f],
                if button == input::A { 0 } else { 255 }
            );
            game.select(&rom, 0).unwrap();
            game.party[0].set_position(528 * 16, 144 * 16);
            game.party[0].set_word(0x2e, 0);
            game.camera = (392, 32);
            for _ in 0..20 {
                game.update(&rom, input::UP).unwrap();
                if game.dialogue.is_some() {
                    break;
                }
            }
            press(&mut game, &rom, input::A);
            drive_until_settled(&mut game, &rom, 0, 10);
            assert!(game.npcs.is_empty());
            game.render(&rom).unwrap();
        } else {
            assert_eq!(game.nuyen, funds);
            assert_eq!(game.npcs.len(), 1);
            if templates.len() < 3 {
                assert_eq!(game.dialogue.as_ref().unwrap().node, 0xff978);
            }
        }
    }
    let mut game = at_trent(&rom, &[0], 3000);
    press(&mut game, &rom, input::B);
    press(&mut game, &rom, input::A);
    press(&mut game, &rom, input::C);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xff970);
    assert_eq!(game.nuyen, 3000);
    assert_eq!(game.party.len(), 1);
    assert_eq!(game.npcs.len(), 1);
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn failed_recruit_assets_do_not_charge_or_remove_npc() {
    let rom = rom();
    let mut game = at_trent(&rom, &[0], 3000);
    press(&mut game, &rom, input::B);
    press(&mut game, &rom, input::A);
    let mut damaged = rom.clone();
    let at = 0x7291a + 5 * 256 + 0x50;
    damaged[at..at + 4].copy_from_slice(&u32::MAX.to_be_bytes());
    game.update(&rom, 0).unwrap();
    assert!(game.update(&damaged, input::A).is_err());
    assert_eq!(game.nuyen, 3000);
    assert_eq!(game.party.len(), 1);
    assert_eq!(game.npcs.len(), 1);
    assert_eq!(game.dialogue.as_ref().unwrap().node, 0xff958);
    press(&mut game, &rom, input::A);
    assert_eq!(game.nuyen, 2715);
    assert_eq!(game.party.len(), 2);
    assert!(game.npcs.is_empty());
}

#[test]
#[ignore = "requires the user-supplied ROM; run with --ignored and SHADOWREALM_TEST_ROM"]
fn hired_companion_follows_and_changes_leader() {
    let rom = rom();
    let mut game = at_trent(&rom, &[0], 3000);
    press(&mut game, &rom, input::B);
    press(&mut game, &rom, input::A);
    press(&mut game, &rom, input::A);
    press(&mut game, &rom, input::A);
    assert_eq!(game.party[1].name(), "Trent");
    assert_eq!(game.nuyen, 2715);
    game.party[0].set_position(240 * 16, 204 * 16);
    game.party[0].set_word(0x2e, 0x8000);
    drive_until_settled(&mut game, &rom, input::DOWN, 0);
    let before = game.party[1].position();
    for _ in 0..90 {
        game.update(&rom, input::LEFT).unwrap();
    }
    assert_ne!(
        game.party[1].position(),
        before,
        "hired runner must follow automatically"
    );
    assert!(game.party[1].position().0 < before.0);
    // Original 21-tick decisions and gradual turning allow lag during a turn.
    // Stop the leader and require the companion to catch up without teleporting.
    for _ in 0..300 {
        game.update(&rom, 0).unwrap();
        if shadowrealm::followers::distance(game.party[0].position(), game.party[1].position())
            < 0x400
        {
            break;
        }
    }
    assert!(
        shadowrealm::followers::distance(game.party[0].position(), game.party[1].position())
            < 0x400
    );
    for r in &game.party {
        assert_eq!(
            game.collision.contact(r.position()).unwrap(),
            Contact::Clear
        );
    }
    game.select(&rom, 1).unwrap();
    let previous = game.party[0].position();
    for _ in 0..90 {
        game.update(&rom, input::RIGHT).unwrap();
    }
    for _ in 0..300 {
        game.update(&rom, 0).unwrap();
        if shadowrealm::followers::distance(game.party[0].position(), game.party[1].position())
            < 0x400
        {
            break;
        }
    }
    assert_ne!(
        game.party[0].position(),
        previous,
        "old leader must become a follower"
    );
    assert!(
        shadowrealm::followers::distance(game.party[0].position(), game.party[1].position())
            < 0x400
    );
    assert_eq!(game.party[1].raw[0x10] & 2, 2);
    assert_eq!(game.party[0].raw[0x10] & 2, 0);
    assert_eq!(game.nuyen, 2715);
    assert_eq!(game.party[1].raw[0x8f], 0);
    game.render(&rom).unwrap();
}
