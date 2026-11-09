use serde_json::json;
use shadowrealm::{
    archives,
    game::Game,
    party::Runner,
    saves::{self, Payload, GLOBAL_BYTES},
};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut writes = vec![];
    let mut loads = vec![];
    for i in 0..512usize {
        let mut before = [[0u8; 74]; 13];
        for (n, r) in before.iter_mut().enumerate() {
            for (j, b) in r.iter_mut().enumerate() {
                *b = (i.wrapping_mul(17) + n * 37 + j * 13) as u8;
            }
        }
        let mut party = vec![];
        for slot in 0..3 {
            let mut r = Runner::load(rom, [0, 3, 10][slot], slot)?;
            for (j, b) in r.raw.iter_mut().enumerate() {
                *b = (i * 31 + slot * 23 + j * 7) as u8;
            }
            r.set_position((i * 59 + slot * 113) as u16, (i * 97 + slot * 71) as u16);
            party.push(r);
        }
        let ids = [
            if i % 5 == 0 { 255 } else { (i % 13) as u8 },
            if i % 7 == 0 { 255 } else { (i / 13 % 13) as u8 },
        ];
        let mut after = before;
        after[0] = archives::pack(&party[0]);
        for (slot, &id) in ids.iter().enumerate() {
            if id != 255 {
                after[usize::from(id)] = archives::pack(&party[slot + 1]);
            }
        }
        let mut global = [0u8; GLOBAL_BYTES];
        for (j, b) in global.iter_mut().enumerate() {
            *b = (i * 47 + j * 11) as u8;
        }
        global[0x212..0x214].copy_from_slice(&ids);
        let map = (i % 56) as u16;
        let camera = ((i * 193) as u16, (i * 317) as u16);
        let positions = [
            party[0].position(),
            party[1].position(),
            party[2].position(),
        ];
        let image = saves::image(rom, &after, &global, map, camera, positions)?;
        writes.push(json!({"slot":i%2,"archives":before.iter().flat_map(|r|*r).collect::<Vec<_>>(),"global":global.to_vec(),"actors":party.iter().flat_map(|r|r.raw).collect::<Vec<_>>(),"map":map,"camera":camera,"image":image.to_vec()}));
        let templates = if i % 3 == 0 {
            vec![i % 3]
        } else if i % 3 == 1 {
            vec![i % 3, 3]
        } else {
            vec![i % 3, 3, 10]
        };
        let mut game = Game::load(rom, i % 6, &templates)?;
        game.nuyen = (i as u32).wrapping_mul(0x314159);
        game.recovery_flags = (i % 2) as u32;
        game.knowledge.contacts = (i as u32) * 13;
        game.story.leads = (i as u32) * 17;
        for (slot, r) in game.party.iter_mut().enumerate() {
            let id = ((i + slot) % 12 + 1) as u8;
            r.raw[0x66] = id;
            r.raw[0x56] = 0;
            r.raw[0x57] = id;
            r.raw[0x4e] = ((i + slot) % 6) as u8;
            r.raw[0x4f] = (i % 21) as u8;
            r.raw[0x8d] = i as u8;
            r.raw[0x4c] = ((i + slot) % 10 + 1) as u8;
        }
        game.select(rom, i % game.party.len())?;
        game.party[0].raw[0x9b] = [0, 2, 3][i / 3 % 3];
        let bytes = game.save_bytes(rom)?;
        let payload = Payload::decode(&bytes)?;
        let restored = Game::from_save_bytes(rom, &bytes)?;
        loads.push(json!({"image":payload.image,"packed":restored.party.iter().map(|r|archives::pack(r).to_vec()).collect::<Vec<_>>(),"positions":restored.party.iter().map(|r|r.position()).collect::<Vec<_>>(),"flags":restored.party.iter().map(|r|r.raw[0x10]).collect::<Vec<_>>(),"portrait":u32::from_be_bytes(restored.party[0].raw[0x50..0x54].try_into().unwrap()),"camera":[restored.camera.0*16,restored.camera.1*16],"map":restored.map_id()}));
    }
    fs::write(
        out.join("saves.json"),
        serde_json::to_vec(&json!({"writes":writes,"loads":loads}))?,
    )?;
    Ok(())
}
