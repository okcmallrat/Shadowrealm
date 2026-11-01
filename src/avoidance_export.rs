//! Differential fixtures for physical actor contact and temporary motion.
use serde_json::json;
use shadowrealm::{
    avoidance::{self, State},
    collision::Collision,
    followers,
    maps::MapDescriptor,
    party::Runner,
};
use std::{error::Error, fs, path::Path};
fn actor(rom: &[u8], slot: usize, p: (u16, u16)) -> Result<Runner, Box<dyn Error>> {
    let mut r = Runner::load(rom, [0, 3, 5][slot % 3], slot % 3)?;
    r.set_word(0x3e, slot as u16);
    r.set_position(p.0, p.1);
    r.set_word(0xc, 128);
    r.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    Ok(r)
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut contacts = vec![];
    for i in 0..1536u16 {
        let (sa, sb) = [(0, 1), (0, 3), (1, 2), (8, 9)][usize::from(i % 4)];
        let p = if i % 11 == 0 {
            (0u16, 0xffffu16)
        } else {
            (4000, 5000)
        };
        let offsets: [i16; 16] = [
            -257, -256, -255, -65, -64, -63, -1, 0, 1, 62, 63, 64, 65, 254, 255, 256,
        ];
        let mut a = actor(rom, sa, p)?;
        let mut b = actor(
            rom,
            sb,
            (
                p.0.wrapping_add(offsets[usize::from(i % 16)] as u16),
                p.1.wrapping_add(offsets[usize::from((i / 16) % 16)] as u16),
            ),
        )?;
        a.raw[0x10] = if i % 31 == 0 { 0x4d } else { 0xd };
        b.raw[0x10] = [0xd, 0xd, 0xd, 0, 0x4d][usize::from((i / 17) % 5)];
        a.set_word(0x38, i % 4);
        b.set_word(0x38, [0, 1, 2, 0xffff][usize::from((i / 4) % 4)]);
        match i / 256 {
            1 => a.raw[0xa2..0xa6].copy_from_slice(&(0xff0100u32 + sb as u32 * 256).to_be_bytes()),
            2 => b.raw[0xa2..0xa6].copy_from_slice(&(0xff0100u32 + sa as u32 * 256).to_be_bytes()),
            3 => a.raw[0xac..0xb0].copy_from_slice(&(0xff0100u32 + sb as u32 * 256).to_be_bytes()),
            4 => b.raw[0xac..0xb0].copy_from_slice(&(0xff0100u32 + sa as u32 * 256).to_be_bytes()),
            _ => {}
        }
        let before = vec![a.raw.to_vec(), b.raw.to_vec()];
        let seed = i.wrapping_mul(751);
        let mut state = State { seed };
        let found = avoidance::pair(rom, &mut a, &mut b, &mut state)?;
        contacts.push(json!({"slots":[sa,sb],"before":before,"after":[a.raw.to_vec(),b.raw.to_vec()],"seed":seed,"after_seed":state.seed,"found":found}));
    }
    let mut movements = vec![];
    for map_id in [0, 3, 5, 10] {
        let c = Collision::load(rom, &MapDescriptor::read(rom, map_id)?)?;
        let mut positions = vec![];
        'positions: for y in (16..c.bounds[3]).step_by(7) {
            for x in (16..c.bounds[2]).step_by(3) {
                let p = (x * 16, y * 16);
                if c.contact(p)? != shadowrealm::collision::Contact::Clear {
                    continue;
                }
                for (dx, dy) in [(0u16, 25u16), (25, 0), (0, 0xffe7), (0xffe7, 0)] {
                    if c.contact((p.0.wrapping_add(dx), p.1.wrapping_add(dy)))?
                        != shadowrealm::collision::Contact::Clear
                    {
                        positions.push(p);
                        if positions.len() == 128 {
                            break 'positions;
                        }
                        break;
                    }
                }
            }
        }
        for i in 0..256usize {
            let mut r = actor(rom, 1, positions[(i * 47) % positions.len()])?;
            r.raw[0x10] &= !2;
            r.raw[0x11] = 0x26;
            r.set_word(0x34, (i as u16).wrapping_mul(0x800));
            r.set_word(0x92, (i as u16).wrapping_mul(0x3800));
            r.set_word(0x38, [0, 1, 2][i % 3]);
            let before = r.raw.to_vec();
            avoidance::respond(rom, &c, &mut r, i as u16)?;
            movements.push(json!({"map":map_id,"phase":i,"before":before,"after":r.raw.to_vec()}));
        }
    }
    let mut cells = vec![0x7f; 4096];
    for y in 2..=14 {
        for x in 2..=14 {
            cells[y * 64 + x] = 0;
        }
    }
    for y in 6..=10 {
        cells[y * 64 + 9] = 0x7f;
    }
    let c = Collision {
        cells,
        rectangles: vec![],
        bounds: [0, 0, 512, 512],
        map_width_tiles: 32,
        overflow_writes: vec![],
    };
    let mut timelines = vec![];
    for selected in 0..3 {
        let mut party = vec![
            actor(rom, 0, (4080, 4080))?,
            actor(rom, 1, (4096, 4096))?,
            actor(rom, 2, (4112, 4112))?,
        ];
        for (slot, r) in party.iter_mut().enumerate() {
            r.raw[0x10] = if slot == selected { 0xf } else { 0xd };
            followers::move_one(rom, &c, r)?;
        }
        let initial: Vec<_> = party.iter().flat_map(|r| r.raw).collect();
        let seed = 0x1234;
        let mut state = State { seed };
        let mut ticks = vec![];
        for phase in 0..900u16 {
            let mut serviced = false;
            for slot in 0..party.len() {
                if slot != selected {
                    let leader = party[selected].clone();
                    followers::schedule(rom, &c, &mut party[slot], &leader, &mut serviced)?;
                }
                avoidance::scan(rom, &mut party, &mut [], slot, &mut state)?;
                if !avoidance::respond(rom, &c, &mut party[slot], phase)? {
                    followers::move_one(rom, &c, &mut party[slot])?;
                }
            }
            ticks.push(json!({"actors":party.iter().flat_map(|r|r.raw).collect::<Vec<_>>(),"seed":state.seed}));
        }
        timelines.push(json!({"selected":selected,"initial":initial,"seed":seed,"ticks":ticks}));
    }
    fs::write(
        out.join("avoidance.json"),
        serde_json::to_vec(
            &json!({"contacts":contacts,"movements":movements,"cells":c.cells,"timelines":timelines}),
        )?,
    )?;
    println!("Exported 1536 contacts, 1024 response steps and 2700 ordered party ticks");
    Ok(())
}
