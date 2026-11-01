//! Original-routine fixtures for target lists, retention/RNG and close range.
use serde_json::json;
use shadowrealm::{
    avoidance::State,
    collision::{Collision, Contact, Rect},
    combat::{self, ActorId, TargetLists},
    maps::MapDescriptor,
    party::Runner,
};
use std::{error::Error, fs, path::Path};
fn fixture_actors(rom: &[u8], variant: usize) -> Result<Vec<Runner>, Box<dyn Error>> {
    let mut actors = vec![];
    for slot in 0..10usize {
        let mut r = Runner::load(rom, slot % 13, slot % 3)?;
        r.set_word(0x3e, slot as u16);
        r.set_position(4000 + slot as u16 * 17, 5000 + slot as u16 * 29);
        r.raw[0x10] = [0, 1, 5, 9, 13, 0x11, 0x41, 0x51][(variant * 7 + slot * 3) % 8];
        r.set_word(0xc, [0, 1, 128, 0xffff][(variant / 3 + slot) % 4]);
        r.raw[0xb0] = if (variant + slot) % 7 == 0 { 0x80 } else { 0 };
        r.raw[0x9e] = [0, 8, 0x10, 0x20, 0x28][(variant / 7 + slot) % 5];
        if (variant + slot) % 11 == 0 {
            r.raw[0x4c] = 0;
        }
        actors.push(r);
    }
    Ok(actors)
}
fn raw(actors: &[Runner]) -> Vec<u8> {
    actors.iter().flat_map(|r| r.raw).collect()
}
fn ids(list: &[ActorId]) -> Vec<u32> {
    list.iter().map(|id| id.pointer()).collect()
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut weapon_timing = vec![];
    for i in 0..4096usize {
        let mut r = Runner::load(rom, i % 13, i % 3)?;
        r.raw[0x78] = (i * 37) as u8;
        r.raw[0x7b] = (i / 16 * 53) as u8;
        r.set_word(0x90, ((i % 16) as u16) << 11);
        r.raw[0x57] = (i % 17) as u8;
        r.raw[0x10] = (i % 16) as u8;
        r.set_word(0xdc, [0, 1, 15, 32767, 32768, 65535, 65534][i % 7]);
        r.set_word(0x1a, 123);
        r.set_word(0x1e, 456);
        let before = r.raw.to_vec();
        let mode = [0, 1, 2, 3, 0xffff][i % 5];
        let timer = shadowrealm::weapons::cooldown(&r, mode);
        let mut tick = r.clone();
        shadowrealm::weapons::tick_cooldown(&mut tick);
        let angle = (i as u16).wrapping_mul(997);
        shadowrealm::weapons::prepare_ranged(rom, &mut r, angle)?;
        weapon_timing.push(json!({"before":before,"mode":mode,"timer":timer,"tick":tick.raw.to_vec(),"angle":angle,"ranged":r.raw.to_vec()}));
    }
    let mut selections = vec![];
    for i in 0..768usize {
        let mut actors = fixture_actors(rom, i)?;
        let source = if i % 2 == 0 { 0 } else { 3 };
        actors[source].raw[0x10] = if source == 0 { 0xd } else { 5 };
        let mut lists = TargetLists::default();
        lists.refresh(&actors.iter().collect::<Vec<_>>(), 0, true);
        let current = match (i / 2) % 6 {
            0 => 0,
            1 => if source == 0 {
                lists.npcs.first()
            } else {
                lists.party.first()
            }
            .map_or(0, |id| id.pointer()),
            2 => 0xff0a00,
            3 => 0xff0100 + source as u32 * 256,
            4 => 0xff0101,
            _ => 0x12345678,
        };
        actors[source].raw[0xa2..0xa6].copy_from_slice(&current.to_be_bytes());
        let before = raw(&actors);
        let seed = (i as u16).wrapping_mul(971);
        let mut random = State { seed };
        let empty_request = lists.acquire(&mut actors[source], &mut random);
        selections.push(json!({"source":source,"before":before,"after":raw(&actors),"npcs":ids(&lists.npcs),"party":ids(&lists.party),"seed":seed,"after_seed":random.seed,"empty_request":empty_request}));
    }
    let mut range = vec![];
    let mut source = Runner::load(rom, 0, 0)?;
    source.set_position(4000, 5000);
    for i in 0..1024usize {
        let offsets = [
            0u16, 1, 0xffff, 0x1ff, 0x200, 0x201, 0xfe01, 0xfe00, 0xfdff, 0x180, 0xff80, 0x1000,
        ];
        let mut target = Runner::load(rom, 3, 1)?;
        target.set_position(
            4000u16.wrapping_add(offsets[i % offsets.len()]),
            5000u16.wrapping_add(offsets[(i / offsets.len()) % offsets.len()]),
        );
        target.raw[0x10] = [1, 0x41, 0, 0x11, 5, 13][(i / 13) % 6];
        let probe = combat::close_range(rom, &source, &target)?;
        range.push(json!({"source":source.raw.to_vec(),"target":target.raw.to_vec(),"heading":probe.heading,"range":probe.range,"in_reach":probe.in_reach}));
    }
    let mut actors = fixture_actors(rom, 0)?;
    let mut lists = TargetLists::default();
    let mut random = State { seed: 0x3344 };
    let mut ticks = vec![];
    for phase in 0..256u16 {
        let fresh = fixture_actors(rom, phase as usize)?;
        for (r, new) in actors.iter_mut().zip(fresh) {
            r.raw[0x10] = new.raw[0x10];
            r.raw[0xb0] = new.raw[0xb0];
            r.raw[0x9e] = new.raw[0x9e];
            r.set_word(0xc, new.word(0xc));
        }
        actors[0].raw[0x10] = 0xd;
        let before = raw(&actors);
        lists.refresh(&actors.iter().collect::<Vec<_>>(), phase, phase == 0);
        let empty_request = lists.acquire(&mut actors[0], &mut random);
        ticks.push(json!({"before":before,"after":raw(&actors),"npcs":ids(&lists.npcs),"party":ids(&lists.party),"seed":random.seed,"empty_request":empty_request}));
    }
    let mut line_grids = vec![];
    let mut lines = vec![];
    let mut grids = vec![];
    for map in [0, 3, 5, 10, 31, 55] {
        grids.push((
            Some(map),
            Collision::load(rom, &MapDescriptor::read(rom, map)?)?,
        ));
    }
    for code in [0u8, 0x7f, 0x7e, 0x40, 0x60, 0x20, 0x28, 0x30, 0x38, 0x80, 1] {
        let mut cells = vec![0; 4096];
        for y in 0..16 {
            cells[y * 64 + 8] = code;
        }
        let rectangles = vec![
            Rect {
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
                code: 0,
            },
            Rect {
                left: 256,
                right: 287,
                top: 0,
                bottom: 511,
                code: 1,
            },
        ];
        grids.push((
            None,
            Collision {
                cells,
                rectangles,
                bounds: [0, 0, 512, 512],
                map_width_tiles: 32,
                overflow_writes: vec![],
            },
        ));
    }
    for (grid, (map, c)) in grids.iter().enumerate() {
        let mut positions = vec![];
        for i in 0..8192u32 {
            let left = c.bounds[0].max(32);
            let top = c.bounds[1].max(32);
            let x =
                left + (i * 73 % u32::from(c.bounds[2].saturating_sub(left + 32).max(1))) as u16;
            let y = top + (i * 97 % u32::from(c.bounds[3].saturating_sub(top + 32).max(1))) as u16;
            if c.contact((x * 16, y * 16))? == Contact::Clear {
                positions.push((x * 16, y * 16));
            }
            if positions.len() == 64 {
                break;
            }
        }
        if positions.len() < 16 {
            return Err("insufficient projectile fixture points".into());
        }
        for i in 0..128usize {
            let mut source = Runner::load(rom, 0, (i / 3) % 3)?;
            let mut target = Runner::load(rom, 3, 1)?;
            target.set_word(0x3e, 3);
            let (from, to) = if map.is_some() {
                (
                    positions[i % positions.len()],
                    positions[(i * 17 + 5) % positions.len()],
                )
            } else {
                (
                    (
                        [176u16, 208, 304, 336][i % 4] * 16,
                        (144 + (i as u16 * 19) % 224) * 16,
                    ),
                    (
                        [336u16, 368, 176, 208][i % 4] * 16,
                        (144 + (i as u16 * 47) % 224) * 16,
                    ),
                )
            };
            source.set_position(from.0, from.1);
            source.set_word(2, (i * 971) as u16);
            source.set_word(6, (i * 337) as u16);
            source.set_word(0x2e, (i as u16).wrapping_mul(0x800).wrapping_add(0x400));
            source.raw[0x10] = if i % 2 == 0 { 0xd } else { 5 };
            target.set_position(to.0, to.1);
            target.raw[0x10] = [5, 5, 5, 0, 0x45, 0x15, 1][i % 7];
            target.raw[0x4c] = if i % 11 == 0 { 0 } else { 10 };
            let scratch = [[0u8; 4], [0, 0x20, 0x28, 0x38], [0, 0x7f, 0xc0, 0x68]][i % 3];
            let probe = combat::line_of_fire(rom, c, &source, &target, scratch)?;
            lines.push(json!({"grid":grid,"source":source.raw.to_vec(),"target":target.raw.to_vec(),"initial_samples":scratch,"outcome":probe.outcome as u8,"steps":probe.steps,"projectile":probe.projectile.to_vec(),"samples":probe.samples}));
        }
        line_grids.push(json!({"map":map,"cells":c.cells,"bounds":c.bounds,"rectangles":c.rectangles.iter().map(|r|[r.left,r.right,r.top,r.bottom,u16::from(r.code)]).collect::<Vec<_>>()}));
    }
    fs::write(
        out.join("combat.json"),
        serde_json::to_vec(
            &json!({"weapon_timing":weapon_timing,"selections":selections,"range":range,"ticks":ticks,"lines":lines,"line_grids":line_grids}),
        )?,
    )?;
    println!(
        "Exported 768 target selections, 1024 close-range probes, 256 refresh/retention ticks"
    );
    Ok(())
}
