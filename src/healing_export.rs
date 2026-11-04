use serde_json::json;
use shadowrealm::{
    avoidance::State, casting, collision::Collision, damage::DamageWorld, party::Runner,
    weapons::LaunchWorld,
};
use std::{error::Error, fs, path::Path};
fn raw(a: &[Runner]) -> Vec<u8> {
    a.iter().flat_map(|a| a.raw).collect()
}
fn actor(rom: &[u8], slot: usize) -> Result<Runner, Box<dyn Error>> {
    let mut a = Runner::load(rom, [0, 5, 10][slot % 3], slot % 3)?;
    a.set_word(0x3e, slot as u16);
    a.raw[0x10] = if slot < 3 { 13 } else { 0 };
    a.raw[0xdf] = 0;
    a.raw[0xb2] = 0;
    a.raw[0x9e] = 0;
    a.raw[0xb0] = 0;
    a.raw[0x11] = 0;
    a.set_word(0xe, 3);
    a.set_word(0xc, 100);
    a.set_word(0x16, 0);
    a.set_position(16000 + slot as u16 * 400, 16000);
    a.set_word(0x34, 0x8000);
    a.set_word(0xb8, 0x1000);
    a.set_word(0xc8, 0x900);
    a.raw[0x4c] = 10;
    a.raw[0x9c] = 10;
    a.raw[0x66..0x76].fill(0);
    a.set_word(0x90, 0);
    Ok(a)
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut callbacks = vec![];
    let mut completions = vec![];
    let mut casts = vec![];
    let mut effects = vec![];
    let open = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let blocked = Collision {
        cells: vec![0x7f; 4096],
        rectangles: vec![],
        bounds: open.bounds,
        map_width_tiles: open.map_width_tiles,
        overflow_writes: vec![],
    };
    for i in 0..1024usize {
        let mut a = actor(rom, i % 3)?;
        a.raw[0x10] = [13, 15, 0x1d, 0x4d][i % 4];
        a.set_word(0xc, if i % 5 == 0 { 0 } else { 100 });
        a.raw[0x78] = (i % 10) as u8;
        a.set_word(0x2e, (i as u16).wrapping_mul(971));
        a.set_word(0x34, (i as u16).wrapping_mul(139));
        a.set_word(0xc8, [0, 0x100, 0x360, 0x900][i / 7 % 4]);
        a.raw[0x11] = (i as u8) & 5;
        let before = a.raw.to_vec();
        let engaged = i % 2 == 0;
        let wall = i % 7 == 0;
        let c = if wall { &blocked } else { &open };
        shadowrealm::followers::healing_callback(rom, c, &mut a, engaged)?;
        callbacks
            .push(json!({"before":before,"after":a.raw.to_vec(),"engaged":engaged,"wall":wall}));
    }
    for i in 0..2048usize {
        let mut a = (0..3)
            .map(|slot| actor(rom, slot))
            .collect::<Result<Vec<_>, _>>()?;
        let caster = (i % 3) as u8;
        let target = (i / 3 % 3) as u8;
        a[usize::from(caster)].raw[0x9f] = [0, 1, 2, 10, 127, 128, 255][i / 9 % 7];
        let t = &mut a[usize::from(target)];
        t.raw[0x4c] = [0, 1, 3, 8, 10][i / 7 % 5];
        t.raw[0x9c] = [0, 1, 10][i / 11 % 3];
        t.raw[0x10] = [0, 1, 13, 0x1d, 0x4d, 0x5d][i / 13 % 6];
        t.raw[0x3a..0x3e].copy_from_slice(&0xb38au32.to_be_bytes());
        let before = raw(&a);
        casting::complete_heal(&mut a, caster, target)?;
        completions.push(json!({"before":before,"after":raw(&a),"caster":caster,"target":target}));
    }
    for i in 0..1024usize {
        let mut a = (0..10)
            .map(|slot| actor(rom, slot))
            .collect::<Result<Vec<_>, _>>()?;
        let caster = (i % 3) as u8;
        let target = (i / 3 % 3) as u8;
        for x in a.iter_mut().take(3) {
            x.raw[0x10] = 13;
        }
        a[0].raw[0x10] |= 2;
        let source = &mut a[usize::from(caster)];
        source.raw[0x56] = 255;
        source.raw[0x57] = 12;
        source.raw[0x9d] = (1 + i % 6) as u8;
        source.raw[0x80] = (i / 3 % 12) as u8;
        source.raw[0x7c] = (i / 5 % 12) as u8;
        source.raw[0x4d] = (i / 7 % 4) as u8;
        source.raw[0x7f] = 6;
        source.raw[0xa2..0xa6].copy_from_slice(&(0xff0100 + u32::from(target) * 256).to_be_bytes());
        let before = raw(&a);
        let seed = (i as u16).wrapping_mul(971);
        let mut rng = State { seed };
        let mut runtime = casting::Runtime::default();
        if i % 11 == 0 {
            runtime.graphics = [true; 18];
        }
        let occupied = runtime.graphics;
        let mut world = LaunchWorld {
            timeout: 17,
            ..Default::default()
        };
        runtime.cast_with_geometry(
            rom,
            Some(&open),
            &mut a,
            caster,
            target,
            &mut world,
            &mut rng,
        )?;
        casts.push(json!({"before":before,"after":raw(&a),"caster":caster,"target":target,"seed":seed,"after_seed":rng.seed,"occupied":occupied,"after_occupied":runtime.graphics,"engaged":world.engaged,"timeout":world.timeout}));
    }
    for i in 0..1024usize {
        let mut a = (0..3)
            .map(|slot| actor(rom, slot))
            .collect::<Result<Vec<_>, _>>()?;
        let caster = (i % 3) as u8;
        let target = (i / 3 % 3) as u8;
        a[usize::from(caster)].raw[0x9f] = [0, 1, 3, 8, 127, 128, 255][i / 9 % 7];
        let t = &mut a[usize::from(target)];
        t.raw[0xb2] = 12;
        t.raw[2] = [0, 1, 4, 5, 255][i / 7 % 5];
        t.raw[3] = [0, 1, 14, 15, 16][i / 11 % 5];
        t.raw[0x4c] = [0, 1, 8, 10][i / 13 % 4];
        t.raw[0x9c] = [0, 1, 10][i / 17 % 3];
        t.raw[0x10] = [13, 0x1d, 0x4d, 0x5d][i / 19 % 4];
        t.set_word(0x32, 17);
        t.raw[0xd8..0xdc].copy_from_slice(&(0xff0100 + u32::from(caster) * 256).to_be_bytes());
        let before = raw(&a);
        let mut state = casting::Runtime::default();
        state.graphics[17] = true;
        let mut rng = State { seed: 123 };
        state.advance(rom, &mut a, &mut DamageWorld::default(), &mut rng)?;
        effects.push(json!({"before":before,"after":raw(&a),"caster":caster,"target":target,"occupied":state.graphics[17],"seed":rng.seed}));
    }
    fs::write(
        out.join("healing.json"),
        serde_json::to_vec(
            &json!({"callbacks":callbacks,"completions":completions,"casts":casts,"effects":effects}),
        )?,
    )?;
    Ok(())
}
