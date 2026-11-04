use serde_json::json;
use shadowrealm::{
    avoidance::State, casting, damage::DamageWorld, party::Runner, weapons::LaunchWorld,
};
use std::{error::Error, fs, path::Path};
fn actors(rom: &[u8], i: usize) -> Result<Vec<Runner>, Box<dyn Error>> {
    let mut actors = vec![];
    for slot in 0..10 {
        let mut a = Runner::load(rom, slot % 13, slot % 3)?;
        a.set_word(0x3e, slot as u16);
        a.set_position(16000 + slot as u16 * 150, 16000);
        a.raw[0x10] = if slot < 3 { 13 } else { 5 };
        a.set_word(0xc, 100);
        a.set_word(0xe, 3);
        a.raw[0xb0] = 0;
        a.raw[0xb2] = 0;
        a.raw[0xdf] = 0;
        a.raw[0x9a] = 0;
        a.raw[0x9b] = if slot == 0 { 0 } else { 2 };
        a.raw[0x4c] = 10;
        a.raw[0x9c] = 10;
        a.raw[0x66..0x76].fill(0);
        a.raw[0x76] = 0;
        a.raw[0x80] = (i % 12) as u8;
        a.raw[0x7c] = (i / 7 % 12) as u8;
        a.raw[0x77] = (i / 5 % 12) as u8;
        a.raw[0x4d] = (i / 11 % 4) as u8;
        a.raw[0x7f] = 6;
        a.raw[0x56] = 255;
        a.raw[0x57] = [1, 2, 8, 9][i % 4];
        a.raw[0x9d] = (1 + i / 3 % 12) as u8;
        a.raw[0x9f] = (i / 13 % 16) as u8;
        a.set_word(0x90, 0);
        a.set_word(0x30, slot.saturating_sub(3) as u16);
        actors.push(a);
    }
    Ok(actors)
}
fn raw(a: &[Runner]) -> Vec<u8> {
    a.iter().flat_map(|a| a.raw).collect()
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut casts = vec![];
    let mut damages = vec![];
    let mut effects = vec![];
    let mut poses = vec![];
    for i in 0..2048 {
        let mut a = actors(rom, i)?;
        a[0].raw[0x9c] = [1, 4, 5, 8, 10][i / 3 % 5];
        a[0].raw[0x4c] = [1, 4, 5, 8, 10][i / 5 % 5];
        a[0].raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
        a[0].set_word(0x90, if i % 3 == 0 { 2 } else { 0 });
        a[3].raw[0x11] = if i % 7 == 0 { 0x80 } else { 0 };
        a[3].raw[0x9b] = if i % 11 == 0 { 0x16 } else { 2 };
        if i % 3 == 0 {
            a[0].raw[0x66 + 6] = 0x18;
            a[0].raw[0x6e + 6] = 3;
        }
        if i % 5 == 0 {
            a[0].raw[0x66 + 3] = 0x19 + (a[0].raw[0x57] - 1);
            a[0].raw[0x6e + 3] = 2;
        }
        if i % 4 == 0 {
            a[0].raw[0x6d] = [0x29, 0x2a, 0x2b][i / 4 % 3];
        }
        if i % 7 == 0 {
            a[0].raw[0x66 + 5] = 0x13;
            a[0].raw[0x6e + 5] = 4;
        }
        if i < 1024 {
            let mut pose = a[0].clone();
            pose.set_word(0x2e, (i as u16).wrapping_mul(971));
            let before_pose = pose.raw.to_vec();
            casting::prepare_pose(rom, &mut pose)?;
            poses.push(json!({"before":before_pose,"after":pose.raw.to_vec()}));
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
            engaged: 0,
            timeout: 17,
            ..Default::default()
        };
        let result = runtime.cast(rom, &mut a, 0, 3, &mut world, &mut rng)?;
        casts.push(json!({"before":before,"after":raw(&a),"seed":seed,"after_seed":rng.seed,"occupied":occupied,"after_occupied":runtime.graphics,"engaged":world.engaged,"timeout":world.timeout,"result":format!("{result:?}")}));
    }
    for i in 0..4096 {
        let mut a = actors(rom, i)?;
        let source = if i / 4 % 2 == 0 { 0 } else { 3 };
        let target = if source == 0 { 3 } else { (i / 3 % 3) as u8 };
        for slot in 0..3 {
            if (i + slot) % 5 == 0 {
                a[slot].raw[0x9b] = 1;
            }
            if (i + slot) % 7 == 0 {
                a[slot].raw[0x9b] = 0;
            }
            if (i + slot) % 3 == 0 {
                let focus = i / 11 % 7;
                a[slot].raw[0x66 + focus] = 0x18;
                a[slot].raw[0x6e + focus] = (i / 13 % 5) as u8;
            }
        }
        a[usize::from(target)].raw[0x4c] = [1, 3, 6, 10][i / 7 % 4];
        a[usize::from(source)].raw[0x9f] = (i / 5 % 20) as u8;
        let before = raw(&a);
        let seed = (i as u16).wrapping_mul(337);
        let mut rng = State { seed };
        let mut world = DamageWorld {
            reward_count: 15,
            ..Default::default()
        };
        shadowrealm::damage::resolve_magic(rom, &mut a, source, target, &mut world, &mut rng)?;
        damages.push(json!({"before":before,"after":raw(&a),"source":source,"target":target,"seed":seed,"after_seed":rng.seed,"reward":world.reward_count,"successes":world.last_successes}));
    }
    let catalog = casting::Catalog::load(rom)?;
    for i in 0..2048 {
        let mut a = actors(rom, i)?;
        let item = a[0].raw[0x57];
        a[3].raw[0xb2] = item;
        a[3].raw[2] = [0, 1, 5, 128, 255][i / 3 % 5];
        let count = catalog.effect(item).unwrap().frame_count;
        a[3].raw[3] = [0, 1, count - 1, count][i / 5 % 4];
        a[3].raw[0xde] = 0xa5;
        a[3].raw[0xd8..0xdc].copy_from_slice(&0xff0100u32.to_be_bytes());
        let graphic = [0, 3, 10, 17][i / 7 % 4];
        a[3].set_word(0x32, graphic as u16);
        a[3].set_word(0x2a, 0x49 + graphic as u16 * 24);
        a[3].raw[0x4c] = [1, 3, 6, 10][i / 7 % 4];
        let before = raw(&a);
        let seed = (i as u16).wrapping_mul(337);
        let mut rng = State { seed };
        let mut runtime = casting::Runtime::default();
        runtime.graphics[graphic] = true;
        let mut world = DamageWorld {
            reward_count: 15,
            ..Default::default()
        };
        runtime.advance(rom, &mut a, &mut world, &mut rng)?;
        effects.push(json!({"before":before,"after":raw(&a),"seed":seed,"after_seed":rng.seed,"item":item,"graphic":graphic,"occupied":runtime.graphics[graphic],"reward":world.reward_count}));
    }
    fs::write(
        out.join("casting.json"),
        serde_json::to_vec(
            &json!({"poses":poses,"casts":casts,"damages":damages,"effects":effects}),
        )?,
    )?;
    Ok(())
}
