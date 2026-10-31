//! Oracle input and host presentation captures for the bounded JSON pilot.
use serde_json::json;
use shadowrealm::{
    dialogue::Dialogue,
    dialogue_data::{Catalog, Knowledge},
    npc,
    party::Runner,
};
use std::{error::Error, fs, io::Write, path::Path};
pub fn run(rom: &[u8], directory: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(directory)?;
    let catalog = Catalog::load(rom)?;
    let mut nodes = vec![];
    let mut edges = vec![];
    let mut contacts = vec![];
    for record in &catalog.trent.nodes {
        let dialogue = Dialogue::from_record(rom, record, &catalog)?;
        nodes.push(json!({"node":dialogue.node,"text_address":dialogue.text_address,"text":dialogue.text,"choices":dialogue.choices.iter().map(|c|json!({"text_address":c.text_address,"text":c.text,"destination":c.destination,"available":c.available})).collect::<Vec<_>>()}));
        let mut frame = vec![0; 320 * 224 * 3];
        dialogue.render(rom, &mut frame)?;
        let mut image =
            fs::File::create(directory.join(format!("node-{:06x}.ppm", dialogue.node)))?;
        image.write_all(b"P6\n320 224\n255\n")?;
        image.write_all(&frame)?;
        for slot in 0..dialogue.choices.len() {
            let mut knowledge = Knowledge {
                runner_known: 0xa000,
                contacts: 0x10000000,
            };
            let next = dialogue.follow(rom, &catalog, slot, &mut knowledge)?;
            edges.push(json!({"node":dialogue.node,"slot":slot,"next":next.map(|n|n.node),"runner_known":knowledge.runner_known,"contacts":knowledge.contacts}));
        }
    }
    let npc = npc::space_needle(rom)?.remove(0);
    for dx in [-65i16, -64, -63, -1, 0, 63, 64, 65] {
        for dy in [-65i16, -64, -63, -1, 0, 63, 64, 65] {
            let mut player = Runner::load(rom, 0, 0)?;
            player.set_position(
                npc.position().0.wrapping_add(dx as u16),
                npc.position().1.wrapping_add(dy as u16),
            );
            contacts.push(json!({"x":player.position().0,"y":player.position().1,"player_flags":player.raw[0x10],"npc_flags":npc.raw[0x10],"npc_talk":npc.raw[0x9e],"contact":npc::trent_contact(&player,&npc)}));
        }
    }
    for (player_flags, npc_flags, npc_talk) in [
        (0x41, 5, 0x10),
        (1, 5, 0x10),
        (3, 4, 0x10),
        (3, 0x45, 0x10),
        (3, 5, 0),
    ] {
        let mut player = Runner::load(rom, 0, 0)?;
        player.raw[0x10] = player_flags;
        player.set_position(npc.position().0, npc.position().1);
        let mut other = npc.clone();
        other.raw[0x10] = npc_flags;
        other.raw[0x9e] = npc_talk;
        contacts.push(json!({"x":player.position().0,"y":player.position().1,"player_flags":player_flags,"npc_flags":npc_flags,"npc_talk":npc_talk,"contact":npc::trent_contact(&player,&other)}));
    }
    fs::write(directory.join("npc.bin"), npc.raw)?;
    fs::write(
        directory.join("dialogue.json"),
        serde_json::to_vec_pretty(
            &json!({"root":catalog.trent.root.0,"nodes":nodes,"edges":edges,"contacts":contacts}),
        )?,
    )?;
    println!("Exported 12-node dialogue pilot, branch effects and contact boundaries");
    Ok(())
}
