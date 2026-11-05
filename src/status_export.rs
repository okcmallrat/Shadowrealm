use serde_json::json;
use shadowrealm::{
    actors::ActorFrame, avoidance::State, casting, map_render::MapScene, maps::MapDescriptor,
    party::Runner, status_spells, weapons::LaunchWorld,
};
use std::{error::Error, fs, path::Path};
fn actors(rom: &[u8], i: usize) -> Result<Vec<Runner>, Box<dyn Error>> {
    let mut a = vec![];
    for slot in 0..10usize {
        let mut r = Runner::load(rom, if slot == 1 { 3 } else { 10 }, slot % 3)?;
        r.set_word(0x3e, slot as u16);
        r.set_position(16000 + slot as u16 * 100, 16000 + slot as u16 * 20);
        r.raw[0x10] = if slot < 3 { 13 } else { 0 };
        r.set_word(0xc, 100);
        r.raw[0xb2] = 0;
        r.raw[0xdf] = 0;
        r.raw[0x11] = [0, 0x20, 0x80, 0xa0][(i + slot) % 4];
        r.raw[0x40] = [0, 1, 127, 128, 255][i / 5 % 5];
        r.raw[0x41] = r.raw[0x40];
        r.raw[0x4c] = 10;
        r.raw[0x9c] = 10;
        r.raw[0x9b] = 0;
        r.raw[0x9e] = 0;
        r.raw[0x66..0x76].fill(0);
        r.raw[0x80] = 6;
        r.raw[0x7c] = 6;
        r.raw[0x7f] = 6;
        r.raw[0x4d] = (i / 7 % 4) as u8;
        r.raw[0x56] = 255;
        r.raw[0x57] = if i % 2 == 0 { 4 } else { 11 };
        r.raw[0x9d] = (1 + i / 11 % 8) as u8;
        a.push(r);
    }
    Ok(a)
}
fn raw(a: &[Runner]) -> Vec<u8> {
    a.iter().flat_map(|a| a.raw).collect()
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut casts = vec![];
    let mut clocks = vec![];
    let mut graphics = vec![];
    for i in 0..2048usize {
        let mut a = actors(rom, i)?;
        let caster = i / 2 % 3;
        let target = if i % 3 == 0 { caster } else { i / 5 % 3 };
        a[caster].raw[0x9c] = [1, 4, 5, 8, 10][i / 3 % 5];
        a[caster].raw[0xa2..0xa6].copy_from_slice(&(0xff0100 + target as u32 * 256).to_be_bytes());
        if i % 5 == 0 {
            a[caster].raw[0x66 + 6] = 0x18;
            a[caster].raw[0x6e + 6] = 3;
        }
        if i % 7 == 0 {
            a[caster].raw[0x66 + 3] = 0x13;
            a[caster].raw[0x6e + 3] = 4;
        }
        if i % 11 == 0 {
            a[caster].raw[0x6d] = [0x29, 0x2a, 0x2b][i / 11 % 3];
        }
        let before = raw(&a);
        let seed = (i as u16).wrapping_mul(971);
        let mut rng = State { seed };
        let mut runtime = casting::Runtime::default();
        if i % 13 == 0 {
            runtime.graphics = [true; 18];
        } else if i % 17 == 0 {
            runtime.graphics = [true; 18];
            runtime.graphics[17] = false;
        }
        let occupied = runtime.graphics;
        let mut world = LaunchWorld {
            timeout: 17,
            ..Default::default()
        };
        let result = runtime.cast(
            rom,
            &mut a,
            caster as u8,
            target as u8,
            &mut world,
            &mut rng,
        )?;
        casts.push(json!({"before":before,"after":raw(&a),"caster":caster,"target":target,"seed":seed,"after_seed":rng.seed,"occupied":occupied,"after_occupied":runtime.graphics,"engaged":world.engaged,"timeout":world.timeout,"result":format!("{result:?}")}));
    }
    for i in 0..4096usize {
        let mut a = actors(rom, i)?;
        for (slot, r) in a.iter_mut().enumerate() {
            r.raw[0x10] = if (i + slot) % 7 == 0 { 0 } else { 1 };
            r.set_word(0xc, if (i + slot) % 5 == 0 { 0 } else { 100 });
            r.raw[0x11] = [0, 0x20, 0x80, 0xa0][(i + slot) % 4];
            r.raw[0x40] = [0, 1, 2, 127, 128, 129, 255][(i / 3 + slot) % 7];
            r.raw[0x41] = [0, 1, 2, 127, 128, 129, 255][(i / 5 + slot) % 7];
            r.raw[0x9e] = if (i + slot) % 3 == 0 { 0x40 } else { 0 };
        }
        let phase = [0, 1, 63, 64, 128, 65535][i / 7 % 6];
        let before = raw(&a);
        status_spells::tick(&mut a, phase);
        clocks.push(json!({"before":before,"after":raw(&a),"phase":phase}));
    }
    let maps = [0, 5, 10, 21, 31];
    let scenes = maps
        .iter()
        .map(|id| MapScene::load(rom, *id))
        .collect::<Result<Vec<_>, _>>()?;
    for i in 0..512usize {
        let mi = i % maps.len();
        let map = MapDescriptor::read(rom, maps[mi])?;
        let mut a = Runner::load(rom, [0, 3, 10][i / 5 % 3], 0)?;
        a.raw[0x11] = [0x80, 0x20, 0xa0][i / 7 % 3];
        let x = u16::from_be_bytes([map.raw[0x42], map.raw[0x43]])
            .wrapping_add((i / 11 % 5) as u16 * 16);
        let y = u16::from_be_bytes([map.raw[0x44], map.raw[0x45]])
            .wrapping_add((i / 13 % 5) as u16 * 16);
        a.set_position(x, y);
        a.set_word(0x16, (i / 17 % 16) as u16 * 4);
        let camera = ((x >> 4).saturating_sub(100), (y >> 4).saturating_sub(100));
        let p = shadowrealm::actors::project_origin(
            a.position(),
            (camera.0 * 16, camera.1 * 16),
            a.word(8) as i16,
        );
        a.set_word(0xa, p.0);
        a.set_word(0xc, p.1);
        let seq = a.sequence(rom)?;
        let frame = seq.frames[i / 23 % seq.frames.len()];
        let bytes = status_spells::graphics(rom, &scenes[mi], frame, &a, camera)?;
        let mut scene = scenes[mi].clone();
        let loaded = ActorFrame::load_status(rom, frame, &mut scene, &a, camera)?;
        let base = usize::from(a.word(0x20)) * 32;
        assert_eq!(&scene.vram()[base..base + bytes.len()], bytes.as_slice());
        graphics.push(json!({"actor":a.raw.to_vec(),"map":maps[mi],"camera":camera,"frame":frame,"bytes":bytes,"pieces":loaded.pieces.iter().map(|p|json!([p.x,p.y,p.width,p.height,p.attribute])).collect::<Vec<_>>()}));
    }
    fs::write(
        out.join("statuses.json"),
        serde_json::to_vec(&json!({"casts":casts,"clocks":clocks,"graphics":graphics}))?,
    )?;
    Ok(())
}
