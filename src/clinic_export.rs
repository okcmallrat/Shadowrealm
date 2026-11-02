//! Private clinic recovery fixtures and a source-backed solo gameplay replay.
use serde_json::json;
use shadowrealm::{clinic, game::Game, input, party::Runner, transitions::TransitionPlan};
use std::{error::Error, fs, io::Write, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let record = clinic::Record::load(rom)?;
    let mut cases = vec![];
    for i in 0..1024u32 {
        let mut actor = Runner::load(rom, 0, 0)?;
        actor.raw[0x4c] = 0;
        actor.raw[0x9c] = i as u8;
        actor.raw[0x10] = (i >> 2) as u8;
        actor.raw[0x11] = i.wrapping_mul(43) as u8;
        actor.set_word(0x16, 0x8084);
        actor.set_word(0xdc, i as u16);
        let balance = match i % 12 {
            0 => 0,
            1 => 1,
            2 => 7,
            3 => 8,
            4 => 39_999,
            5 => 40_000,
            6 => 40_007,
            7 => u32::MAX,
            _ => i.wrapping_mul(497_631),
        };
        let before = actor.raw.to_vec();
        let result = clinic::prepare(&[actor], balance, 0, 0).unwrap();
        cases.push(json!({"before":before,"after":result.actor.raw.to_vec(),"balance":balance,"after_balance":result.balance,"fee":result.fee}));
    }
    let plan = TransitionPlan::clinic(rom, 5, &record)?;
    fs::write(
        out.join("clinic.json"),
        serde_json::to_vec(
            &json!({"cases":cases,"plan":{"map":plan.target_map,"spawn":plan.spawn_address,"camera":plan.camera_fixed,"positions":plan.positions},"node":record.node.0,"text":record.text.0}),
        )?,
    )?;
    let mut game = Game::load(rom, 5, &[0])?;
    game.party[0].raw[0x4c] = 1;
    game.party[0].raw[0x9c] = 3;
    game.nuyen = 40_001;
    for _ in 0..12_000 {
        game.update(rom, 0)?;
        if game.defeated {
            break;
        }
    }
    if game.dialogue.as_ref().map(|d| d.node) != Some(record.node.0) {
        return Err("clinic replay did not reach original passage".into());
    }
    for name in ["clinic-passage.ppm", "clinic-arrival.ppm"] {
        if name == "clinic-arrival.ppm" {
            game.update(rom, input::A)?;
            for _ in 0..120 {
                if game.transition.is_none() {
                    break;
                }
                game.update(rom, 0)?;
            }
        }
        let mut f = fs::File::create(out.join(name))?;
        f.write_all(b"P6\n320 224\n255\n")?;
        f.write_all(&game.render(rom)?)?;
    }
    fs::write(
        out.join("recovery.json"),
        serde_json::to_vec_pretty(
            &json!({"map":game.map_id(),"health":game.party[0].raw[0x4c],"condition":game.party[0].raw[0x9c],"balance":game.nuyen,"position":game.party[0].position(),"ammo":game.party[0].raw[0x4e],"npc_shots":game.combat.npc_shots}),
        )?,
    )?;
    Ok(())
}
