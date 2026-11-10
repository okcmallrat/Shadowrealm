use serde_json::json;
use shadowrealm::{avoidance::State, damage::DamageWorld, party::Runner, unarmed};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    unarmed::Catalog::load(rom)?;
    let mut damage = vec![];
    for i in 0..2048usize {
        let mut actors = vec![];
        for slot in 0..10 {
            let mut a = Runner::load(rom, 0, 0)?;
            a.set_word(0x3e, slot as u16);
            a.raw[0x10] = if slot == 0 {
                15
            } else if slot < 3 {
                13
            } else {
                5
            };
            a.set_position(0x1000 + slot as u16 * 0x100, 0x1800);
            actors.push(a);
        }
        let s = i % 4;
        let t = if s < 3 || i % 3 == 0 { 4 } else { i % 3 };
        actors[s].raw[0x57] = 0;
        actors[s].raw[0x56] = 127;
        for (j, slot) in [s, t].into_iter().enumerate() {
            let a = &mut actors[slot];
            a.raw[0x9b] = [0, 2, 0x16, 0x17][(i / 7 + j) % 4];
            a.raw[0x9a] = [0, 1, 2, 4, 128, 255][(i / 11 + j) % 6];
            a.raw[0x4d] = [0, 1, 2, 4, 8, 127, 255][(i / 13 + j) % 7];
            a.raw[0x4c] = [0, 1, 3, 8, 10, 127, 128, 255][(i / 17 + j) % 8];
            a.raw[0x9c] = [0, 1, 3, 10, 118, 127, 128, 255][(i / 19 + j) % 8];
            a.raw[0x85] = [0, 1, 4, 8, 12, 127, 128, 255][(i / 23 + j) % 8];
            for off in [0x77, 0x78, 0x79, 0x7b, 0x7c] {
                a.raw[off] = (i / (usize::from(off as u8) % 7 + 1) % 13) as u8;
            }
            a.raw[0x76] = (i / 29 % 8) as u8;
            a.set_word(
                0x90,
                [0, 2, 4, 8, 0x20, 0x100, 0x200, 0xe00, 0x60e][(i / 31 + j) % 9],
            );
            a.raw[0x11] = [0, 0x20, 0x80, 0xa0][(i / 37 + j) % 4];
            if i % 5 == 0 {
                a.raw[0x68] = 0x28;
                a.raw[0x70] = (i / 3) as u8;
            }
            if i % 7 == 0 {
                a.raw[0x6a] = 0x27;
                a.raw[0x72] = (i / 5) as u8;
            }
        }
        actors[t].raw[0x57] = if i % 2 == 0 { 0 } else { 2 };
        let mut w = DamageWorld {
            mode: if i % 17 == 0 { 3 } else { 0 },
            reward_count: (i / 41) as u8,
            ..Default::default()
        };
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let seed = (i as u16).wrapping_mul(811);
        let mut rng = State { seed };
        let count = w.reward_count;
        unarmed::resolve(rom, &mut actors, s as u8, t as u8, &mut w, &mut rng)?;
        damage.push(json!({"source":s,"target":t,"before":before,"after":actors.iter().flat_map(|a|a.raw).collect::<Vec<_>>(),"seed":seed,"after_seed":rng.seed,"mode":w.mode,"count":count,"after_count":w.reward_count,"last":w.last_successes}));
    }
    let mut strikes = vec![];
    let mut falls = vec![];
    let mut clocks = vec![];
    for i in 0..256usize {
        let mut actors = vec![];
        for slot in 0..10 {
            let mut a = Runner::load(rom, 0, 0)?;
            a.set_word(0x3e, slot);
            a.raw[0x10] = if slot == 0 {
                15
            } else if slot < 3 {
                13
            } else {
                5
            };
            a.set_position(0x1800, 0x1800);
            actors.push(a);
        }
        let s = 3;
        let t = i % 3;
        actors[s].set_word(0xe, 8);
        actors[s].raw[0x10] = 5;
        actors[s].raw[0x57] = 0;
        actors[s].raw[0x56] = 127;
        actors[s].raw[0x85] = (i % 9) as u8;
        actors[s].set_word(0x90, [0, 4, 8][i / 3 % 3]);
        actors[s].set_position(0x1800 + (i as u16 % 16) * 12, 0x1800 + (i as u16 / 16) * 12);
        actors[s].raw[0xa2..0xa6].copy_from_slice(&(0xff0100u32 + t as u32 * 256).to_be_bytes());
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let mut pool = [[0u8; 256]; 6];
        for (slot, raw) in pool.iter_mut().enumerate() {
            if slot < i % 7 {
                raw[0x10] = 1;
            }
        }
        let pool_before = pool.iter().flat_map(|p| *p).collect::<Vec<_>>();
        let mut launch = shadowrealm::weapons::LaunchWorld {
            map_id: 2,
            ..Default::default()
        };
        let mut damage = DamageWorld::default();
        let seed = (i as u16).wrapping_mul(3037);
        let mut rng = State { seed };
        unarmed::strike(
            rom,
            &mut actors,
            s as u8,
            t as u8,
            &mut pool,
            &mut launch,
            &mut damage,
            &mut rng,
        )?;
        strikes.push(json!({"source":s,"target":t,"before":before,"after":actors.iter().flat_map(|a|a.raw).collect::<Vec<_>>(),"pool_before":pool_before,"pool":pool.iter().flat_map(|p|*p).collect::<Vec<_>>(),"seed":seed,"after_seed":rng.seed,"engaged":launch.engaged,"timeout":launch.timeout,"defeat":damage.defeat_timer}));
        let slot = i % 4;
        actors[slot].raw[0x10] = if slot == 0 {
            15
        } else if slot < 3 {
            13
        } else {
            5
        };
        actors[slot].raw[0x9c] = 0;
        actors[slot].raw[0x4c] = 10;
        actors[slot].set_word(0x2e, (i as u16).wrapping_mul(8191));
        actors[slot].set_word(0xe, if slot < 3 { 3 } else { 8 });
        for other in 0..3 {
            if other != slot && (i >> (other + 2)) & 1 != 0 {
                actors[other].raw[0x10] |= 0x40;
            }
        }
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let mut damage = DamageWorld::default();
        shadowrealm::damage::incapacitate(rom, &mut actors, slot as u8, &mut damage)?;
        falls.push(json!({"slot":slot,"before":before,"after":actors.iter().flat_map(|a|a.raw).collect::<Vec<_>>(),"defeat":damage.defeat_timer}));
        let mut fx = Runner::load(rom, 0, 0)?;
        fx.set_word(0xe, 1);
        fx.set_word(0x16, 4);
        fx.set_word(0x1c, 0);
        fx.set_word(0x18, (i as u16).wrapping_mul(273));
        fx.set_word(0x1a, (i as u16).wrapping_mul(193));
        fx.set_word(0x1e, (i % 6) as u16);
        fx.raw[0x11] = 0;
        let before = fx.raw.to_vec();
        shadowrealm::combat_runtime::tick_animation(rom, &mut fx)?;
        clocks.push(json!({"before":before,"after":fx.raw.to_vec()}));
    }
    // Whole controlled entry $ad1a through the native player dispatcher.
    // Idle walking is outside this comparison; held/queued attacks and cooling
    // down enter with the ordinary standing handler already installed.
    let collision = shadowrealm::collision::Collision::load(
        rom,
        &shadowrealm::maps::MapDescriptor::read(rom, 2)?,
    )?;
    let mut controlled = vec![];
    for i in 0..768usize {
        let selected = i % 3;
        let mut actors = Vec::new();
        for slot in 0..10 {
            let mut a = Runner::load(
                rom,
                if slot < 3 { (i / 3 + slot) % 13 } else { 0 },
                slot.min(2),
            )?;
            a.set_word(0x3e, slot as u16);
            a.raw[0x10] = if slot == selected {
                15
            } else if slot < 3 {
                13
            } else {
                5
            };
            a.raw[0x11] = 1;
            a.set_word(0x16, 0);
            a.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
            a.set_position(0x1800, 0x1800);
            a.raw[0x4c] = 10;
            a.raw[0x9c] = 10;
            a.raw[0xdf] = 0;
            actors.push(a);
        }
        let target = 3;
        actors[target].set_word(0xe, 8);
        actors[target].set_position(
            0x1800 + [0, 80, 511, 512, 513, 800][i / 9 % 6],
            0x1800 + (i as u16 % 4) * 50,
        );
        actors[target].raw[0x57] = if i % 2 == 0 { 0 } else { 2 };
        actors[target].raw[0x76] = (i / 11 % 4) as u8;
        if i % 13 == 0 {
            actors[target].raw[0x10] |= 0x40;
        }
        if i % 17 == 0 {
            actors[target].raw[0x10] = 0;
        }
        let a = &mut actors[selected];
        a.raw[0x57] = 0;
        a.raw[0x56] = 127;
        a.raw[0x43] = 0;
        a.raw[0x85] = (i / 5 % 9) as u8;
        a.set_word(0x90, [0, 4, 8, 0x100, 0xe00][i / 7 % 5]);
        a.set_word(0x34, (i as u16).wrapping_mul(1259));
        a.set_word(0xdc, [65535, 0, 100, 65534][i / 23 % 4]);
        if i % 5 != 0 {
            a.raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
        } else {
            a.raw[0xa2..0xa6].fill(0);
        }
        let seed = (i as u16).wrapping_mul(6217);
        let mut random = State { seed };
        let mut runtime = shadowrealm::combat_runtime::Runtime::default();
        runtime.launch.map_id = 2;
        runtime.launch.signals.controlled_busy = i % 7 == 0;
        for (j, fx) in runtime.pool.iter_mut().enumerate() {
            fx[0x10] = u8::from(j < i % 7);
            fx[0x11] = (i as u8).wrapping_mul(7);
            fx[0x40] = (i as u8).wrapping_add(j as u8);
        }
        let input = shadowrealm::input::Controller {
            held: if i % 7 == 0 { 0 } else { shadowrealm::input::A },
            pressed: if i % 2 == 0 { shadowrealm::input::A } else { 0 },
        };
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let pool_before = runtime.pool.iter().flat_map(|p| *p).collect::<Vec<_>>();
        let busy = runtime.launch.signals.controlled_busy;
        runtime.player(
            rom,
            &collision,
            &mut actors,
            selected as u16,
            input,
            &mut random,
        )?;
        controlled.push(json!({"source":selected,"before":before,
            "after":actors.iter().flat_map(|a|a.raw).collect::<Vec<_>>(),
            "pool_before":pool_before,"pool":runtime.pool.iter().flat_map(|p|*p).collect::<Vec<_>>(),
            "seed":seed,"after_seed":random.seed,"input":input.packed(),"busy":busy,
            "after_busy":runtime.launch.signals.controlled_busy,"engaged":runtime.launch.engaged,
            "timeout":runtime.launch.timeout,"count":runtime.damage.reward_count,
            "last":runtime.damage.last_successes}));
    }
    let arena = shadowrealm::collision::Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let mut companions = vec![];
    for i in 0..768usize {
        let slot = 1 + i % 2;
        let mut actors = vec![];
        for n in 0..10 {
            let mut a = Runner::load(rom, if n == slot { 9 } else { 0 }, n % 3)?;
            a.set_word(0x3e, n as u16);
            a.raw[0x10] = if n == 0 {
                15
            } else if n < 3 {
                13
            } else if n == 3 {
                5
            } else {
                0
            };
            a.raw[0x11] = 0;
            a.raw[0x9e] = 0;
            a.raw[0xb0] = 0;
            a.raw[0xdf] = 0;
            a.raw[0xa2..0xa6].fill(0);
            a.raw[0xac..0xb0].fill(0);
            a.set_word(0xc, 100);
            a.set_word(0x16, 0);
            a.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
            a.set_position(16000, 16000);
            a.raw[0x4c] = 10;
            a.raw[0x9c] = 10;
            actors.push(a);
        }
        actors[3].set_word(0xe, 8);
        actors[3].set_position(
            16000 + [0, 100, 511, 512, 800, 1500][i / 7 % 6],
            16000 + (i as u16 % 5) * 35,
        );
        actors[3].raw[0x76] = (i / 17 % 4) as u8;
        let a = &mut actors[slot];
        a.raw[0x9b] = 2; // Explicit mundane close-combat fixture; no spell selection.
        a.raw[0x4d] = (1 + i / 5 % 4) as u8;
        a.raw[0x56] = if i % 3 == 0 { 0 } else { 127 };
        a.raw[0x57] = if i % 3 == 0 { 2 } else { 0 };
        a.raw[0x4e] = 0;
        a.raw[0x4f] = if i % 13 == 0 { 1 } else { 0 };
        a.set_word(0xb6, [0, 1, 65535][i / 19 % 3]);
        a.set_word(0xdc, [65535, 0, 100][i / 23 % 3]);
        a.set_word(0xb8, 0x1000);
        a.raw[0x85] = (i / 11 % 9) as u8;
        if i % 5 != 0 {
            a.raw[0xa2..0xa6].copy_from_slice(&0xff0400u32.to_be_bytes());
        }
        let seed = (i as u16).wrapping_mul(2687);
        let mut random = State { seed };
        let mut runtime = shadowrealm::combat_runtime::Runtime::default();
        runtime.launch.engaged = 65535;
        runtime.launch.timeout = 17;
        runtime.launch.map_id = 2;
        runtime.npc_fire.samples = [0x11, 0x22, 0x33, 0x44];
        let serviced = i % 29 == 0;
        let mut after_serviced = serviced;
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        runtime.companion(
            rom,
            &arena,
            &mut actors,
            slot as u16,
            &mut after_serviced,
            1,
            &mut random,
        )?;
        companions.push(json!({"source":slot,"before":before,
            "after":actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>(),
            "pool":runtime.pool.iter().flat_map(|p| *p).collect::<Vec<_>>(),
            "seed":seed,"after_seed":random.seed,"serviced":serviced,"after_serviced":after_serviced,
            "samples":runtime.npc_fire.samples,"engaged":runtime.launch.engaged,"timeout":runtime.launch.timeout,
            "count":runtime.damage.reward_count,"last":runtime.damage.last_successes}));
    }
    fs::write(
        out.join("unarmed.json"),
        serde_json::to_vec(
            &json!({"damage":damage,"strikes":strikes,"falls":falls,"clocks":clocks,"controlled":controlled,"companions":companions}),
        )?,
    )?;
    Ok(())
}
