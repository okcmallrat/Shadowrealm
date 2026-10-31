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
        let stats: Vec<_> = game.party.iter().map(|r| r.raw[0x4c..].to_vec()).collect();
        drive_until_settled(&mut game, &rom, outward, target);
        drive_until_settled(&mut game, &rom, inward, 0);
        assert_eq!(game.completed_transitions, 2);
        assert_eq!(game.selected, 0);
        for (runner, expected) in game.party.iter().zip(&stats) {
            assert_eq!(&runner.raw[0x4c..], expected);
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
    let stats: Vec<_> = game.party.iter().map(|r| r.raw[0x4c..].to_vec()).collect();
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
        assert_eq!(&runner.raw[0x4c..], expected);
    }
    // Reusing the door must ask again and create a fresh return context.
    // Turning also walks. Curve back from the south-facing arrival, then align
    // with the door, using only controller input (no pose/map reassignment).
    for (count, held) in [(16, input::UP), (11, input::RIGHT), (30, input::UP)] {
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
}
