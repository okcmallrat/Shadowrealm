use shadowrealm::{
    game::{FileMode, Game},
    input,
    saves::{self, Payload},
};
use std::{fs, path::PathBuf};
fn rom() -> Vec<u8> {
    fs::read(std::env::var("SHADOWREALM_TEST_ROM").expect("set SHADOWREALM_TEST_ROM")).unwrap()
}
fn directory(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("shadowrealm-save-{name}-{}", std::process::id()));
    if p.exists() {
        fs::remove_dir_all(&p).unwrap();
    }
    fs::create_dir(&p).unwrap();
    p
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn checkpoint_rebuilds_money_party_equipment_books_jobs_and_original_load_control() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0, 3, 10]).unwrap();
    g.select(&r, 2).unwrap();
    g.nuyen = 12345;
    g.story.leads = 0x1c;
    g.story.inquiries = 1;
    g.recovery_flags = 1;
    g.knowledge.contacts = 19;
    g.knowledge.runner_known = 4;
    g.jobs.raw = [0, 2, 2, 2, 0, 2, 11, 1, 0, 45];
    g.jobs.accepted = true;
    g.jobs.flags = 1;
    g.jobs.carried = 512;
    g.jobs.completed = 7;
    g.jobs.base_pay = 35;
    let bytes = g.save_bytes(&r).unwrap();
    let mut saved = Game::from_save_bytes(&r, &bytes).unwrap();
    assert_eq!(saved.nuyen, 12345);
    assert_eq!(saved.map_id(), 2);
    assert_eq!(saved.camera, g.camera);
    assert_eq!(saved.selected, 0);
    assert_eq!(saved.jobs, g.jobs);
    assert_eq!(saved.story, g.story);
    assert_eq!(saved.recovery_flags, g.recovery_flags);
    assert_eq!(saved.knowledge, g.knowledge);
    assert_eq!(
        saved.combat.spellbooks.as_ref().unwrap().records,
        g.combat.spellbooks.as_ref().unwrap().records
    );
    for (a, b) in saved.party.iter().zip(&g.party) {
        assert_eq!(a.position(), b.position());
        assert_eq!(
            shadowrealm::archives::pack(a),
            shadowrealm::archives::pack(b)
        );
    }
    assert_eq!(saved.archives.opinions, g.archives.opinions);
    saved.render(&r).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn failed_load_and_busy_checkpoint_never_replace_live_progress() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0]).unwrap();
    let bytes = g.save_bytes(&r).unwrap();
    g.nuyen = 77;
    let position = g.party[0].position();
    let mut corrupt = bytes.clone();
    let index = corrupt.iter().position(|&b| b == b'G').unwrap();
    corrupt[index] = b'Z';
    assert!(g.restore_save_bytes(&r, &corrupt).is_err());
    assert_eq!(g.nuyen, 77);
    let mut incompatible = serde_json::to_value(Payload::decode(&bytes).unwrap()).unwrap();
    incompatible["version"] = 2.into();
    let incompatible: Payload = serde_json::from_value(incompatible).unwrap();
    let crc = saves::crc32(&serde_json::to_vec(&incompatible).unwrap());
    let incompatible =
        serde_json::to_vec(&serde_json::json!({"payload":incompatible,"crc32":crc})).unwrap();
    assert!(g.restore_save_bytes(&r, &incompatible).is_err());
    assert_eq!(g.nuyen, 77);
    for corrupt_offset in [20usize, saves::COORDINATES + 6] {
        let mut bad = Payload::decode(&bytes).unwrap();
        saves::put_word(&mut bad.image, corrupt_offset, 65535);
        let checksum = bad.image[20..saves::CHECKSUM]
            .iter()
            .map(|&b| u32::from(b))
            .sum();
        saves::put_long(&mut bad.image, saves::CHECKSUM, checksum);
        assert!(g.restore_save_bytes(&r, &bad.encode().unwrap()).is_err());
        assert_eq!(g.nuyen, 77);
    }
    assert_eq!(g.party[0].position(), position);
    let mut payload = Payload::decode(&bytes).unwrap();
    saves::put_word(&mut payload.image, saves::COORDINATES, 55);
    let checksum = payload.image[20..saves::CHECKSUM]
        .iter()
        .map(|&b| u32::from(b))
        .sum();
    saves::put_long(&mut payload.image, saves::CHECKSUM, checksum);
    assert!(g
        .restore_save_bytes(&r, &payload.encode().unwrap())
        .is_err());
    assert_eq!(g.nuyen, 77);
    g.combat.launch.engaged = 65535;
    assert!(g.save_bytes(&r).is_err());
    g.open_file_menu(&r, FileMode::Save).unwrap();
    assert!(g.dialogue.is_none());
    let interior = Game::load(&r, 13, &[0]).unwrap();
    assert!(interior.save_bytes(&r).is_err());
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn slot_replacement_backup_fallback_and_bad_writes_preserve_previous_data() {
    let r = rom();
    let dir = directory("backup");
    let path = saves::slot_path(&dir, 1).unwrap();
    let mut g = Game::load(&r, 2, &[0]).unwrap();
    let first = g.save_bytes(&r).unwrap();
    g.write_save_slot(&r, &dir, 1).unwrap();
    g.nuyen = 99;
    let second = g.save_bytes(&r).unwrap();
    g.write_save_slot(&r, &dir, 1).unwrap();
    assert_eq!(
        saves::read_bounded(&saves::backup_path(&path)).unwrap(),
        first
    );
    assert_eq!(Game::load_slot(&r, &dir, 1).unwrap().0.nuyen, 99);
    fs::write(&path, b"truncated").unwrap();
    let (backup, used) = Game::load_slot(&r, &dir, 1).unwrap();
    assert!(used);
    assert_eq!(backup.nuyen, 20);
    g.write_save_slot(&r, &dir, 1).unwrap();
    assert_eq!(
        saves::read_bounded(&saves::backup_path(&path)).unwrap(),
        first
    );
    assert!(saves::write_atomic(&path, b"invalid").is_err());
    assert_eq!(saves::read_bounded(&path).unwrap(), second);
    // An intact envelope with unsupported game state must not replace a usable backup.
    let mut bad = Payload::decode(&second).unwrap();
    saves::put_word(&mut bad.image, saves::COORDINATES, 55);
    let checksum = bad.image[20..saves::CHECKSUM]
        .iter()
        .map(|&b| u32::from(b))
        .sum();
    saves::put_long(&mut bad.image, saves::CHECKSUM, checksum);
    fs::write(&path, bad.encode().unwrap()).unwrap();
    assert!(Game::load_slot(&r, &dir, 1).unwrap().1);
    g.write_save_slot(&r, &dir, 1).unwrap();
    assert_eq!(
        saves::read_bounded(&saves::backup_path(&path)).unwrap(),
        first
    );
    g.write_save_slot(&r, &dir, 2).unwrap();
    assert_eq!(Game::load_slot(&r, &dir, 2).unwrap().0.nuyen, 99);
    assert_eq!(saves::read_bounded(&path).unwrap(), second);
    // A destination directory cannot be replaced as a save file.
    let blocked = saves::slot_path(&dir, 2).unwrap();
    fs::remove_file(&blocked).unwrap();
    fs::create_dir(&blocked).unwrap();
    assert!(saves::write_atomic(&blocked, &first).is_err());
    assert_eq!(saves::read_bounded(&path).unwrap(), second);
    assert!(fs::read_dir(&dir).unwrap().all(|p| !p
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));
    fs::remove_dir_all(dir).unwrap();
}
#[test]
#[ignore = "requires the user-supplied ROM"]
fn file_menu_freezes_world_and_emits_a_bounded_request() {
    let r = rom();
    let mut g = Game::load(&r, 2, &[0, 3, 10]).unwrap();
    let positions = g.party.iter().map(|a| a.position()).collect::<Vec<_>>();
    g.open_file_menu(&r, FileMode::Save).unwrap();
    for _ in 0..30 {
        g.update(&r, input::RIGHT).unwrap();
    }
    assert_eq!(
        g.party.iter().map(|a| a.position()).collect::<Vec<_>>(),
        positions
    );
    g.update(&r, 0).unwrap();
    g.update(&r, input::A).unwrap();
    let tick = g.tick;
    g.update(&r, input::UP).unwrap();
    assert_eq!(g.tick, tick);
    let request = g.take_file_request().unwrap();
    assert_eq!(request.mode, FileMode::Save);
    assert_eq!(request.slot, 1);
    assert!(g.dialogue.is_none());
    g.save_bytes(&r).unwrap();
    g.open_file_menu(&r, FileMode::Load).unwrap();
    g.update(&r, 0).unwrap();
    g.update(&r, input::DOWN).unwrap();
    g.update(&r, 0).unwrap();
    g.update(&r, input::A).unwrap();
    let request = g.take_file_request().unwrap();
    assert_eq!(request.mode, FileMode::Load);
    assert_eq!(request.slot, 2);
    g.file_notice(&r, "Loaded slot 2.".into()).unwrap();
    g.update(&r, input::A).unwrap();
    assert!(
        g.dialogue.is_some(),
        "held selection must not dismiss a fresh notice"
    );
    g.update(&r, 0).unwrap();
    g.update(&r, input::A).unwrap();
    assert!(g.dialogue.is_none());
}
