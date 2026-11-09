use serde_json::json;
use shadowrealm::{archives, game::Game, newgame::Archetype};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut cases = vec![];
    for a in Archetype::ALL {
        let g = Game::new_game(rom, a)?;
        let r = &g.party[0];
        cases.push(json!({"template":a.template(),"packed":archives::pack(r).to_vec(),"actor":r.raw.to_vec(),"archives":g.archives.records.iter().flat_map(|r|*r).collect::<Vec<_>>(),"opinions":g.archives.opinions.to_vec(),"books":g.combat.spellbooks.as_ref().unwrap().records.iter().flat_map(|r|*r).collect::<Vec<_>>(),"nuyen":g.nuyen,"map":g.map_id(),"camera":[g.camera.0*16,g.camera.1*16],"position":r.position(),"leads":g.story.leads,"recovery":g.recovery_flags,"contacts":g.knowledge.contacts,"known":g.knowledge.runner_known,"inquiries":g.story.inquiries,"job":g.jobs.raw.to_vec(),"job_flags":g.jobs.flags,"carried":g.jobs.carried}));
    }
    fs::write(out.join("newgame.json"), serde_json::to_vec(&cases)?)?;
    Ok(())
}
