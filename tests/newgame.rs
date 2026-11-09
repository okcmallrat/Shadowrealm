use shadowrealm::{
    archives,
    game::Game,
    input,
    newgame::{Archetype, Catalog},
    saves,
    session::Session,
};
use std::path::{Path, PathBuf};
fn rom() -> Vec<u8> {
    std::fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn temp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("shadowrealm-start-{name}-{}", std::process::id()));
    std::fs::create_dir(&p).unwrap();
    p
}
fn press(s: &mut Session, r: &[u8], key: u8, p: &Path) {
    s.update(r, 0, p).unwrap();
    s.update(r, key, p).unwrap();
}
fn choose(s: &mut Session, r: &[u8], target: usize, p: &Path) {
    for _ in 0..8 {
        if s.menu.as_ref().unwrap().dialogue.selected == target {
            break;
        }
        press(s, r, input::DOWN, p);
    }
    assert_eq!(s.menu.as_ref().unwrap().dialogue.selected, target);
    press(s, r, input::A, p);
}
fn capture(s: &mut Session, r: &[u8], name: &str) {
    use std::io::Write;
    if let Ok(p) = std::env::var("SHADOWREALM_START_CAPTURES") {
        std::fs::create_dir_all(&p).unwrap();
        let mut f = std::fs::File::create(Path::new(&p).join(format!("{name}.ppm"))).unwrap();
        f.write_all(b"P6\n320 224\n255\n").unwrap();
        f.write_all(&s.render(r).unwrap()).unwrap();
    }
}
fn begin(s: &mut Session, r: &[u8], a: Archetype, p: &Path) {
    choose(s, r, 0, p);
    choose(s, r, a.template(), p);
    capture(s, r, a.name());
    choose(s, r, 0, p);
    assert!(s.menu.is_none());
    assert_eq!(s.game.as_ref().unwrap().party[0].template, a.template());
}
#[test]
fn newgame_catalog_rejects_unverified_scope_and_fields() {
    for json in [
        Catalog::JSON.replace("0x42b0", "0x42b2"),
        Catalog::JSON.replace("\"initial_lead\": 1", "\"initial_lead\": 2"),
        Catalog::JSON.replacen('{', "{\"free_companions\":true,", 1),
    ] {
        assert!(Catalog::parse(&json, &[]).is_err());
    }
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn all_archetypes_start_solo_with_original_inventory_books_archives_and_story() {
    let r = rom();
    let p = temp("classes");
    for a in Archetype::ALL {
        let mut s = Session::new(&r).unwrap();
        assert!(s.game.is_none());
        capture(&mut s, &r, "opening-menu");
        begin(&mut s, &r, a, &p);
        let g = s.game.as_ref().unwrap();
        assert_eq!(
            (
                g.map_id(),
                g.nuyen,
                g.party.len(),
                g.story.leads,
                g.recovery_flags
            ),
            (2, 20, 1, 1, 0)
        );
        assert_eq!(g.party[0].raw[0x9b], [2, 3, 1][a.template()]);
        assert_eq!(g.party[0].raw[0x8d], 0);
        assert_eq!(g.archives, archives::Archives::load(&r, &[]).unwrap());
        assert_eq!(g.jobs.raw, [255, 255, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(!g.jobs.active());
        let actor = g.party[0].raw;
        let cash = g.nuyen;
        for _ in 0..60 {
            s.update(&r, input::A, &p).unwrap();
        }
        assert_eq!(s.game.as_ref().unwrap().party[0].raw, actor);
        assert!(
            s.game.as_ref().unwrap().dialogue.is_some(),
            "held confirm cannot dismiss briefing"
        );
        capture(&mut s, &r, "redmond-briefing");
        press(&mut s, &r, input::A, &p);
        assert!(s.game.as_ref().unwrap().dialogue.is_none());
        let before = s.game.as_ref().unwrap().party[0].position();
        for _ in 0..12 {
            s.update(&r, input::DOWN, &p).unwrap();
        }
        assert_ne!(s.game.as_ref().unwrap().party[0].position(), before);
        assert_eq!(s.game.as_ref().unwrap().nuyen, cash);
        let g = s.game.as_ref().unwrap();
        assert!(g.save_ready());
        g.write_save_slot(&r, &p, 1).unwrap();
        let (loaded, _) = Game::load_slot(&r, &p, 1).unwrap();
        assert_eq!(loaded.party.len(), 1);
        assert_eq!(loaded.party[0].template, a.template());
        assert_eq!(loaded.story.leads, 1);
        for range in [0x66..0x8f, 0x9a..0x9d] {
            assert_eq!(loaded.party[0].raw[range.clone()], g.party[0].raw[range]);
        }
    }
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn opening_menu_cancel_freeze_failed_newgame_and_resume_retain_the_live_game() {
    let r = rom();
    let p = temp("retain");
    let mut s = Session::development(Game::load(&r, 2, &[0, 3, 10]).unwrap());
    s.open_start(&r).unwrap();
    let g = s.game.as_ref().unwrap();
    let tick = g.tick;
    let party = g.party.iter().map(|a| a.raw).collect::<Vec<_>>();
    for _ in 0..60 {
        s.update(&r, input::RIGHT, &p).unwrap();
    }
    assert_eq!(s.game.as_ref().unwrap().tick, tick);
    choose(&mut s, &r, 0, &p);
    choose(&mut s, &r, 1, &p); // Decker preview
    choose(&mut s, &r, 1, &p); // Back
    choose(&mut s, &r, 0, &p); // Samurai preview
                               // Break only the selected actor's equipment icon; fonts/menu still render.
    let mut broken = r.clone();
    broken[0xd4540 + 48 + 26..0xd4540 + 48 + 30].copy_from_slice(&0x1fffffu32.to_be_bytes());
    choose(&mut s, &broken, 0, &p);
    assert!(s
        .menu
        .as_ref()
        .unwrap()
        .dialogue
        .text
        .contains("Unable to prepare"));
    assert_eq!(
        s.game
            .as_ref()
            .unwrap()
            .party
            .iter()
            .map(|a| a.raw)
            .collect::<Vec<_>>(),
        party
    );
    assert_eq!(s.game.as_ref().unwrap().tick, tick);
    choose(&mut s, &r, 0, &p); // back to preview
    choose(&mut s, &r, 1, &p); // classes
    choose(&mut s, &r, 3, &p); // root
    choose(&mut s, &r, 2, &p); // Resume
    assert!(s.menu.is_none());
    assert_eq!(
        s.game
            .as_ref()
            .unwrap()
            .party
            .iter()
            .map(|a| a.raw)
            .collect::<Vec<_>>(),
        party
    );
    assert_eq!(s.game.as_ref().unwrap().tick, tick);
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn title_load_works_without_a_running_game_and_falls_back_to_backup() {
    let r = rom();
    let p = temp("load");
    let mut g = Game::new_game(&r, Archetype::GatorShaman).unwrap();
    g.update(&r, input::A).unwrap();
    g.write_save_slot(&r, &p, 2).unwrap();
    g.write_save_slot(&r, &p, 2).unwrap();
    std::fs::write(saves::slot_path(&p, 2).unwrap(), b"damaged").unwrap();
    let mut s = Session::new(&r).unwrap();
    choose(&mut s, &r, 1, &p);
    choose(&mut s, &r, 0, &p);
    assert!(s.game.is_none());
    assert!(s
        .menu
        .as_ref()
        .unwrap()
        .dialogue
        .text
        .contains("No usable checkpoint"));
    capture(&mut s, &r, "missing-checkpoint");
    choose(&mut s, &r, 0, &p); // back to slots
    choose(&mut s, &r, 1, &p); // slot 2
    assert!(s.menu.is_none());
    assert_eq!(s.game.as_ref().unwrap().party[0].template, 2);
    assert!(s
        .game
        .as_ref()
        .unwrap()
        .dialogue
        .as_ref()
        .unwrap()
        .text
        .contains("backup"));
    capture(&mut s, &r, "loaded-checkpoint");
    let position = s.game.as_ref().unwrap().party[0].position();
    for _ in 0..60 {
        s.update(&r, input::A, &p).unwrap();
    }
    assert!(s.game.as_ref().unwrap().dialogue.is_some());
    assert_eq!(s.game.as_ref().unwrap().party[0].position(), position);
    press(&mut s, &r, input::A, &p);
    assert!(s.game.as_ref().unwrap().dialogue.is_none());
    // Explicit defeat fixture: opening-menu loading does not require a safe
    // temporary street game, unlike the in-world F9 shortcut.
    s.game.as_mut().unwrap().defeated = true;
    s.open_start(&r).unwrap();
    choose(&mut s, &r, 1, &p);
    choose(&mut s, &r, 1, &p);
    assert!(s.menu.is_none());
    assert!(!s.game.as_ref().unwrap().defeated);
    assert_eq!(s.game.as_ref().unwrap().party[0].template, 2);
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn failed_title_load_and_canceled_new_game_preserve_existing_progress_and_files() {
    let r = rom();
    let p = temp("fail-load");
    let mut g = Game::new_game(&r, Archetype::Decker).unwrap();
    g.update(&r, input::A).unwrap();
    g.write_save_slot(&r, &p, 1).unwrap();
    let file = std::fs::read(saves::slot_path(&p, 1).unwrap()).unwrap();
    let mut s = Session::development(g);
    s.open_start(&r).unwrap();
    let party = s.game.as_ref().unwrap().party[0].raw;
    choose(&mut s, &r, 1, &p);
    choose(&mut s, &r, 1, &p); // missing slot 2
    assert_eq!(s.game.as_ref().unwrap().party[0].raw, party);
    choose(&mut s, &r, 0, &p);
    choose(&mut s, &r, 2, &p); // slots -> root
    choose(&mut s, &r, 0, &p);
    choose(&mut s, &r, 0, &p); // Samurai preview
    choose(&mut s, &r, 1, &p);
    choose(&mut s, &r, 3, &p);
    choose(&mut s, &r, 2, &p); // back -> resume
    assert_eq!(s.game.as_ref().unwrap().party[0].raw, party);
    assert_eq!(
        std::fs::read(saves::slot_path(&p, 1).unwrap()).unwrap(),
        file
    );
    std::fs::remove_dir_all(p).unwrap();
}
