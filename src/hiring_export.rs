//! Seeded oracle fixtures use the real door, contact and dialogue state machine.
use serde_json::json;
use shadowrealm::{
    dialogue_data::Catalog,
    game::Game,
    hiring::{self, Contract},
    input,
    party::Runner,
};
use std::{error::Error, fs, io::Write, path::Path};
fn press(g: &mut Game, r: &[u8], button: u8) -> Result<(), Box<dyn Error>> {
    g.update(r, 0)?;
    g.update(r, button)?;
    Ok(())
}
pub fn enter(g: &mut Game, r: &[u8]) -> Result<(), Box<dyn Error>> {
    g.party[0].set_position(528 * 16, 144 * 16);
    g.party[0].set_word(0x2e, 0);
    g.camera = (392, 32);
    for _ in 0..20 {
        g.update(r, input::UP)?;
        if g.dialogue.is_some() {
            break;
        }
    }
    if g.dialogue.is_none() {
        return Err("missing entrance dialogue".into());
    }
    press(g, r, input::A)?;
    for _ in 0..150 {
        g.update(r, 0)?;
        if g.map_id() == 10 && g.transition.is_none() {
            break;
        }
    }
    if g.map_id() != 10 || g.transition.is_some() {
        return Err("room entry did not settle".into());
    }
    // Bounded interaction fixture: seed outside NPC radius, then walk in.
    g.party[0].set_position(112 * 16, 85 * 16);
    g.party[0].set_word(0x2e, 0);
    for _ in 0..8 {
        g.update(r, input::UP)?;
        if g.dialogue.is_some() {
            break;
        }
    }
    if g.dialogue.as_ref().map(|d| d.node) != Some(0xfeb4c) {
        return Err("Trent conversation did not open".into());
    }
    press(g, r, input::B)?; // hire pitch
    Ok(())
}
fn frame(g: &mut Game, r: &[u8], directory: &Path, name: &str) -> Result<(), Box<dyn Error>> {
    let mut f = fs::File::create(directory.join(format!("{name}.ppm")))?;
    f.write_all(b"P6\n320 224\n255\n")?;
    f.write_all(&g.render(r)?)?;
    Ok(())
}
pub fn run(r: &[u8], directory: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(directory)?;
    let catalog = Catalog::load(r)?;
    let mut quotes = vec![];
    let mut cases = vec![];
    for opinion in -1..=4 {
        for stats in [
            [0, 0, 0],
            [1, 1, 1],
            [3, 6, 6],
            [6, 12, 12],
            [255, 255, 255],
            [128, 0, 0],
            [127, 255, 1],
            [0, 255, 1],
            [8, 8, 8],
        ] {
            let mut leader = Runner::load(r, 0, 0)?;
            leader.raw[0x7a] = stats[0];
            leader.raw[0x8b] = stats[1];
            leader.raw[0x8c] = stats[2];
            let q = hiring::quote(r, &catalog.trent.hiring, &leader, opinion)?;
            quotes.push(json!({"opinion":opinion,"stats":stats,"short_term":q.short_term,"permanent":q.permanent}));
        }
    }
    for templates in [&[0usize][..], &[0, 3][..], &[0, 3, 10][..]] {
        for contract in [Contract::SingleRun, Contract::Permanent] {
            let leader = Runner::load(r, 0, 0)?;
            let q = hiring::quote(r, &catalog.trent.hiring, &leader, 0)?;
            let price = if contract == Contract::SingleRun {
                q.short_term
            } else {
                q.permanent
            };
            for funds in [0, price - 1, price, price + 1] {
                let case_dir = directory.join(format!("case-{:02}", cases.len()));
                fs::create_dir(&case_dir)?;
                let mut g = Game::load(r, 0, templates)?;
                g.nuyen = funds;
                enter(&mut g, r)?;
                g.party[0].raw[0x4c] = 7;
                g.party[0].raw[0x4e] = 2;
                let before: Vec<_> = g.party.iter().flat_map(|p| p.raw).collect();
                fs::write(case_dir.join("before-party.bin"), before)?;
                fs::write(case_dir.join("before-npc.bin"), g.npcs[0].raw)?;
                let initial_indices = hiring::recruit_indices(&g.party);
                press(&mut g, r, input::A)?; // action 3: quote or capacity refusal
                let quote_node = g
                    .dialogue
                    .as_ref()
                    .ok_or("missing quote/capacity response")?
                    .node;
                if templates.len() < 3 {
                    if funds == price + 1 {
                        frame(&mut g, r, &case_dir, "quote")?;
                    }
                    press(
                        &mut g,
                        r,
                        if contract == Contract::SingleRun {
                            input::A
                        } else {
                            input::B
                        },
                    )?;
                }
                let response = g.dialogue.as_ref().ok_or("missing hire response")?.node;
                if funds == price + 1 {
                    frame(&mut g, r, &case_dir, "outcome")?;
                }
                fs::write(
                    case_dir.join("after-party.bin"),
                    g.party.iter().flat_map(|p| p.raw).collect::<Vec<_>>(),
                )?;
                cases.push(json!({"directory":case_dir.file_name().unwrap().to_string_lossy(),"initial_indices":initial_indices,"party_size":templates.len(),"funds":funds,"contract":contract.flag(),"price":price,"quote_node":quote_node,"response":response,"balance":g.nuyen,"party_after":g.party.len(),"indices_after":hiring::recruit_indices(&g.party),"npc_present":!g.npcs.is_empty()}));
            }
        }
    }
    fs::write(
        directory.join("hiring.json"),
        serde_json::to_vec_pretty(
            &json!({"initial_nuyen":hiring::initial_balance(r)?,"quotes":quotes,"cases":cases}),
        )?,
    )?;
    println!("Exported 54 quote and 24 complete hiring-state fixtures");
    Ok(())
}
