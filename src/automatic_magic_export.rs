use serde_json::json;
use shadowrealm::{
    avoidance::State, collision::Collision, combat_runtime, companion_magic, followers,
    party::Runner, spellbooks::Spellbooks,
};
use std::{error::Error, fs, path::Path};
fn arena() -> Collision {
    let mut c = Collision {
        cells: vec![0; 4096],
        rectangles: vec![],
        bounds: [0, 0, 2048, 2048],
        map_width_tiles: 128,
        overflow_writes: vec![],
    };
    for n in 0..4096 {
        if n / 64 == 0 || n / 64 == 63 || n % 64 == 0 || n % 64 == 63 {
            c.cells[n] = 127;
        }
    }
    c
}
fn raw(a: &[Runner]) -> Vec<u8> {
    a.iter().flat_map(|a| a.raw).collect()
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    companion_magic::Catalog::load(rom)?;
    let mut contexts = vec![];
    for i in 0..1024usize {
        let mut c = arena();
        let solid = i / 9 % 4 == 0;
        if solid {
            c.cells.fill(127);
        }
        let mut a = Runner::load(rom, 0, 0)?;
        a.set_word(0xe, 3);
        a.raw[0x10] = if i % 2 == 0 { 15 } else { 13 };
        a.raw[0x9e] = 0;
        a.raw[0xb0] = 0;
        a.raw[0x11] = 0;
        a.set_word(0xc, 100);
        a.set_position(3200 + (i as u16).wrapping_mul(97) % 20000, 14000);
        a.set_word(0x34, (i as u16).wrapping_mul(971));
        a.set_word(0xb8, 0x1000);
        a.set_word(0xc8, 1500);
        let handler = [
            0xb45cu32, 0xb574, 0xb3ee, 0xb07e, 0xb38a, 0xafe0, 0xafe2, 0xb0d4, 0xb10c,
        ][i % 9];
        a.raw[0x3a..0x3e].copy_from_slice(&handler.to_be_bytes());
        a.set_word(0x16, if i % 9 < 2 { 0 } else { 0x40 });
        a.raw[0xce] = 8;
        let before = a.raw.to_vec();
        if i % 9 == 0 {
            followers::move_one(rom, &c, &mut a)?;
        } else if i % 9 == 1 {
            let before_state = a.word(0x16);
            followers::healing_callback(rom, &c, &mut a, true)?;
            if a.word(0x16) != before_state {
                a.raw[0x11] |= 1;
            }
        }
        let d5 =
            companion_magic::after_actor(&c, &a).ok_or("fixture context unexpectedly withheld")?;
        contexts.push(json!({"before":before,"after":a.raw.to_vec(),"d5":d5,"incoming":(i as u16).wrapping_mul(1973),"solid":solid}));
    }
    let books = Spellbooks::load(rom)?;
    let mut decisions = vec![];
    for i in 0..3072usize {
        let slot = 1 + i / 3 % 2;
        let mut actors = vec![];
        for n in 0..10usize {
            let mut a = Runner::load(
                rom,
                if n == slot {
                    if slot == 1 {
                        3
                    } else {
                        10
                    }
                } else {
                    0
                },
                n % 3,
            )?;
            a.set_word(0x3e, n as u16);
            a.set_word(0xe, 3);
            a.raw[0x10] = if n == 0 {
                15
            } else if n == slot {
                13
            } else if n < 3 {
                13
            } else if n < 5 && i % 31 != 0 {
                5
            } else {
                0
            };
            a.set_position(15000 + n as u16 * 300, 16000 + n as u16 * 30);
            a.set_word(0xc, 100);
            a.raw[0x4c] = 10;
            a.raw[0x9c] = 10;
            a.raw[0x9e] = 0;
            a.raw[0xb0] = 0;
            a.raw[0xb2] = 0;
            a.raw[0xdf] = 0;
            a.raw[0x11] = 0;
            a.raw[0xa2..0xa6].fill(0);
            a.raw[0xac..0xb0].fill(0);
            a.set_word(0x30, n.saturating_sub(3) as u16);
            a.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
            a.set_word(0x16, 0);
            actors.push(a);
        }
        actors[0].raw[0x4c] = [1, 3, 4, 10][i / 5 % 4];
        if i % 11 == 0 {
            actors[1].raw[0x4c] = 2;
        }
        if i % 17 == 0 {
            actors[2].raw[0x4c] = 2;
        }
        for n in 3..5 {
            actors[n].set_position(
                16000 + [0x100, 0x3c0, 0x640, 0x900][i / 7 % 4],
                16000 + (n as u16 - 3) * 500,
            );
        }
        let source = &mut actors[slot];
        source.set_position(16000, 16000);
        source.raw[0x4d] = (1 + i / 13 % 4) as u8;
        source.set_word(0xb6, [0, 1, 65535][i / 19 % 3]);
        source.set_word(0xdc, [65535, 0, 1][i / 23 % 3]);
        source.set_word(0xb8, 0x1000);
        source.raw[0x9c] = [1, 4, 5, 8, 10][i / 29 % 5];
        source.raw[0x80] = [0, 3, 6, 10][i / 31 % 4];
        source.raw[0x4e] = if i % 37 == 0 { 0 } else { 12 };
        source.raw[0x4f] = if i % 7 == 0 { 0 } else { 2 };
        source.raw[0x66] = if slot == 1 { 1 } else { 7 };
        source.raw[0x56] = 0;
        source.raw[0x57] = source.raw[0x66];
        if i % 3 != 0 {
            source.raw[0x56] = 255;
            source.raw[0x57] = if slot == 2 && i % 3 == 1 {
                12
            } else if slot == 2 {
                9
            } else {
                2
            };
            source.raw[0x9d] = [1, 3, 4][i / 11 % 3];
            source.raw[0x4e] = [0, 50, 100][i / 17 % 3];
        }
        source.set_word(0xc, if i % 13 == 0 { 0 } else { 100 });
        source.raw[0xb5] = 6;
        source.raw[0xa2..0xa6]
            .copy_from_slice(&(if i % 2 == 0 { 0xff0400u32 } else { 0 }).to_be_bytes());
        let wall = i / 5 % 4 == 0;
        let mut c = arena();
        if wall {
            for row in 1..63 {
                c.cells[row * 64 + 32] = 127;
            }
        }
        let prior = &mut actors[slot - 1];
        prior.raw[0xb0] = 0;
        prior.raw[0x9e] = 0;
        prior.raw[0x3a..0x3e]
            .copy_from_slice(&(if i % 5 == 0 { 0xb574u32 } else { 0xb45c }).to_be_bytes());
        prior.set_word(0xb8, 0x1000);
        prior.set_word(0xc8, 1500);
        prior.set_word(0x34, 0x4000);
        let prior_before = prior.raw.to_vec();
        if i % 5 == 0 {
            let before_state = prior.word(0x16);
            followers::healing_callback(rom, &c, prior, true)?;
            if prior.word(0x16) != before_state {
                prior.raw[0x11] |= 1;
            }
        } else {
            followers::move_one(rom, &c, prior)?;
        }
        let index =
            companion_magic::after_actor(&c, prior).ok_or("automatic fixture caller unknown")?;
        let before = raw(&actors);
        let seed = (i as u16).wrapping_mul(971);
        let mut random = State { seed };
        let mut runtime = combat_runtime::Runtime::default();
        runtime.spellbooks = Some(books.clone());
        runtime.companion_capacity_index = Some(index);
        runtime.healing_hint = (i % 3) as u8;
        let hint = runtime.healing_hint;
        runtime.launch.engaged = if i % 41 == 0 { 0 } else { 65535 };
        runtime.launch.timeout = 17;
        let engaged = runtime.launch.engaged;
        runtime.npc_fire.firing = [0, 1, 65535][i / 43 % 3];
        let firing = runtime.npc_fire.firing;
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
        let occupied = runtime.casting.graphics;
        let serviced = i % 47 == 0;
        let mut after_serviced = serviced;
        runtime.companion(
            rom,
            &c,
            &mut actors,
            slot as u16,
            &mut after_serviced,
            1,
            &mut random,
        )?;
        decisions.push(json!({"before":before,"after":raw(&actors),"prior_before":prior_before,"slot":slot,"index":index,"seed":seed,"after_seed":random.seed,"hint":hint,"after_hint":runtime.healing_hint,
            "engaged":engaged,"after_engaged":runtime.launch.engaged,"after_timeout":runtime.launch.timeout,"firing":firing,"after_firing":runtime.npc_fire.firing,"after_samples":runtime.npc_fire.samples,
            "occupied":occupied,"after_occupied":runtime.casting.graphics,"serviced":serviced,"after_serviced":after_serviced,"wall":wall,
            "npcs":runtime.npc_targets.npcs.iter().map(|id|id.pointer()).collect::<Vec<_>>(),"after_pool":runtime.pool.iter().flatten().copied().collect::<Vec<_>>(),
            "busy":runtime.launch.signals.controlled_busy,"burst":runtime.launch.signals.burst,"casts":runtime.casting.casts,"equips":runtime.companion_spell_changes,"deferred":runtime.companion_deferred[slot]}));
    }
    fs::write(
        out.join("automatic.json"),
        serde_json::to_vec(
            &json!({"books":books.records.iter().flatten().copied().collect::<Vec<_>>(),"contexts":contexts,"decisions":decisions}),
        )?,
    )?;
    Ok(())
}
