//! Full companion decisions with already equipped attack spells; no scripted casts.
use serde_json::json;
use shadowrealm::{avoidance::State, collision::Collision, party::Runner, spellbooks::Spellbooks};
use std::{error::Error, fs, path::Path};

pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut arena = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    let books = Spellbooks::load(rom)?;
    let mut decisions = vec![];
    for i in 0..2048usize {
        let slot = 1 + i / 4 % 2;
        let mut actors = vec![];
        for n in 0..10usize {
            let mut a = Runner::load(rom, if n == slot { 3 } else { 0 }, n % 3)?;
            a.set_word(0x3e, n as u16);
            a.raw[0x10] = if n == 0 {
                15
            } else if n == slot {
                13
            } else if (3..5).contains(&n) && i % 31 != 0 {
                5
            } else {
                0
            };
            a.set_position(16000 + (n as u16).wrapping_mul(100), 16000 + n as u16 * 30);
            a.set_word(0xc, 100);
            a.set_word(0xe, 3);
            a.raw[0x9b] = if n == slot { 1 } else { 2 };
            a.raw[0x9e] = 0;
            a.raw[0xb0] = 0;
            a.raw[0xb2] = 0;
            a.raw[0xdf] = 0;
            a.raw[0x4c] = 10;
            a.raw[0x9c] = 10;
            a.raw[0x66..0x76].fill(0);
            a.raw[0xa2..0xa6].fill(0);
            a.raw[0xac..0xb0].fill(0);
            a.set_word(0x30, n.saturating_sub(3) as u16);
            a.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
            a.set_word(0x16, 0);
            actors.push(a);
        }
        actors[0].set_position(15500, 15600);
        for n in 3..5 {
            actors[n].set_position(
                16000 + [0x100, 0x3c0, 0x640, 0x900][i / 7 % 4],
                16000 + (n as u16 - 3) * 500,
            );
        }
        let source = &mut actors[slot];
        source.set_position(16000, 16000);
        source.raw[0x4d] = (1 + i / 11 % 8) as u8;
        source.raw[0x56] = 255;
        source.raw[0x57] = [1, 2, 8, 9][i % 4];
        source.raw[0x9d] = (1 + i / 13 % 6) as u8;
        source.raw[0x80] = 6;
        source.raw[0x7f] = 6;
        source.raw[0x7c] = 6;
        source.raw[0x9c] = [5, 8, 10][i / 17 % 3];
        source.raw[0x4e] = if i % 19 == 0 { 0 } else { 70 };
        source.raw[0x4f] = if i % 5 == 0 { 0 } else { 2 };
        source.raw[0x66] = 1; // A supported inventory firearm for original unsafe-spell fallback.
        source.raw[0xb5] = [0, 6, 255][i / 19 % 3];
        source.raw[0xb0] = if i % 3 == 0 { 0x10 } else { 0 };
        source.set_word(0xa0, 0); // Classifier book without Heal: current attacks stay equipped.
        source.set_word(0xb6, [0, 1, 0xffff][i / 23 % 3]);
        source.set_word(0xb8, 0x1000);
        source.set_word(0xdc, [0xffff, 0, 1][i / 29 % 3]);
        source.set_word(0xc, if i % 7 == 0 { 0 } else { 100 });
        source.raw[0xa2..0xa6]
            .copy_from_slice(&(if i % 2 == 0 { 0xff0400u32 } else { 0 }).to_be_bytes());
        if i % 13 == 0 {
            source.raw[0xac..0xb0].copy_from_slice(&0xff0500u32.to_be_bytes());
        }
        let wall = i / 5 % 4 == 0;
        arena.cells.fill(0);
        for cell in 0..4096 {
            if cell / 64 == 0 || cell / 64 == 63 || cell % 64 == 0 || cell % 64 == 63 {
                arena.cells[cell] = 0x7f;
            }
        }
        if wall {
            for row in 0..64 {
                arena.cells[row * 64 + 32] = 0x7f;
            }
        }
        let mut runtime = shadowrealm::combat_runtime::Runtime::default();
        runtime.spellbooks = Some(books.clone());
        runtime.launch.engaged = if i % 37 == 0 { 0 } else { 0xffff };
        runtime.launch.timeout = 17;
        runtime.npc_fire.firing = [0, 1, 0xffff][i / 31 % 3];
        runtime.npc_fire.samples = [0x11, 0x22, 0x33, 0x44];
        if i % 17 == 0 {
            runtime.casting.graphics = [true; 18];
        }
        for a in actors
            .iter()
            .filter(|a| a.word(0x3e) >= 3 && a.raw[0x10] & 9 == 1)
        {
            runtime.casting.graphics[usize::from(a.word(0x30))] = true;
        }
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let occupied = runtime.casting.graphics;
        let engaged = runtime.launch.engaged;
        let firing = runtime.npc_fire.firing;
        let samples = runtime.npc_fire.samples;
        let serviced = i % 41 == 0;
        let mut after_serviced = serviced;
        let seed = (i as u16).wrapping_mul(971);
        let mut random = State { seed };
        runtime.companion(
            rom,
            &arena,
            &mut actors,
            slot as u16,
            &mut after_serviced,
            1,
            &mut random,
        )?;
        decisions.push(json!({
            "before":before,"after":actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>(),
            "slot":slot,"seed":seed,"after_seed":random.seed,"occupied":occupied,
            "after_occupied":runtime.casting.graphics,"engaged":engaged,
            "after_engaged":runtime.launch.engaged,"after_timeout":runtime.launch.timeout,
            "firing":firing,"after_firing":runtime.npc_fire.firing,"samples":samples,
            "after_samples":runtime.npc_fire.samples,"serviced":serviced,"after_serviced":after_serviced,
            "wall":wall,"npcs":runtime.npc_targets.npcs.iter().map(|id|id.pointer()).collect::<Vec<_>>(),
            "deferred":runtime.companion_deferred[slot],"casts":runtime.casting.casts
            ,"after_pool":runtime.pool.iter().flatten().copied().collect::<Vec<_>>()
            ,"busy":runtime.launch.signals.controlled_busy,"burst":runtime.launch.signals.burst
            ,"shots":runtime.companion_shots
        }));
    }
    fs::write(
        out.join("caster-movement.json"),
        serde_json::to_vec(
            &json!({"books":books.records.iter().flatten().copied().collect::<Vec<_>>(),"decisions":decisions}),
        )?,
    )?;
    Ok(())
}
