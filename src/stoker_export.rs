use serde_json::json;
use shadowrealm::{
    avoidance::State, collision::Collision, encounters::Runtime, maps::MapDescriptor,
    party::Runner, stoker::Catalog, weapons::LaunchWorld,
};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let c = Catalog::load(rom)?;
    let map = MapDescriptor::read(rom, 2)?;
    let collision = Collision::load(rom, &map)?;
    let mut visits = vec![];
    for i in 0..512usize {
        let mut actors = vec![];
        for slot in 0..10 {
            let mut a = Runner::load(rom, 0, 0)?;
            a.set_word(0x3e, slot);
            a.set_word(0x30, slot.saturating_sub(3));
            a.raw[0x10] = if slot < 3 {
                13
            } else if (i >> (slot - 3)) & 1 != 0 {
                5
            } else {
                0
            };
            actors.push(a);
        }
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let mut world = LaunchWorld {
            map_id: 2,
            timeout: 123,
            ..Default::default()
        };
        let mut runtime = Runtime::default();
        let seed = (i as u16).wrapping_mul(2137);
        let mut rng = State { seed };
        let camera = [
            0x3800 + (i as u16 % 8) * 0x100,
            0x2400 + (i as u16 / 8 % 8) * 0x100,
        ];
        let count = if c.roll(&mut rng) {
            runtime.stoker(
                rom,
                &map,
                (camera[0], camera[1]),
                &collision,
                &mut actors,
                &mut world,
                &mut rng,
            )?
        } else {
            0
        };
        visits.push(json!({"before":before,"after":actors.iter().flat_map(|a|a.raw).collect::<Vec<_>>(),"seed":seed,"after_seed":rng.seed,"camera":camera,"node":c.node(count),"count":runtime.count,"engaged":world.engaged,"timeout":world.timeout}));
    }
    fs::write(
        out.join("stoker.json"),
        serde_json::to_vec(&json!({"visits":visits}))?,
    )?;
    Ok(())
}
