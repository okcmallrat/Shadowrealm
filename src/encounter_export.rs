//! Private original-routine fixtures; never a runtime encounter script.
use serde_json::json;
use shadowrealm::{
    avoidance::State,
    collision::Collision,
    encounters::{self, Gate},
    maps::MapDescriptor,
    party::Runner,
};
use std::{error::Error, fs, io::Write, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut selections = vec![];
    let mut gates = vec![];
    let mut positions = vec![];
    let mut groups = vec![];
    let mut offscreen = vec![];
    for map in 0..56 {
        for i in 0..32u16 {
            let seed = i.wrapping_mul(1777);
            let mut rng = State { seed };
            let s = encounters::select(rom, map, &mut rng)?;
            selections.push(json!({"map":map,"seed":seed,"kind":s.kind,"address":s.address,"after_seed":rng.seed}));
        }
    }
    for i in 0..1024u16 {
        let g = Gate {
            tick: if i % 3 == 0 { 10 } else { i },
            paused: if i % 17 == 0 { 1 } else { 0 },
            player_flags: if i % 19 == 0 { 0x80 } else { 0 },
            blocked: if i % 23 == 0 { 1 } else { 0 },
            script: if i % 7 == 0 { 1 } else { 0 },
            engaged: if i % 11 == 0 { 0xffff } else { 0 },
            count: if i % 13 == 0 { 1 } else { 0 },
            chance: [0, 0x6000, 0x8000, 0xffff][i as usize % 4],
        };
        let seed = i.wrapping_mul(9349);
        let mut rng = State { seed };
        let result = encounters::due(g, &mut rng);
        gates.push(json!({"tick":g.tick,"paused":g.paused,"flags":g.player_flags,"blocked":g.blocked,"script":g.script,"engaged":g.engaged,"count":g.count,"chance":g.chance,"seed":seed,"after_seed":rng.seed,"due":result}));
    }
    for map in [0, 3, 5, 10] {
        let desc = MapDescriptor::read(rom, map)?;
        let c = Collision::load(rom, &desc)?;
        for i in 0..64u16 {
            let camera = (i.wrapping_mul(137) & 0x3fff, i.wrapping_mul(83) & 0x1fff);
            for close in [false, true] {
                let Ok(points) = encounters::positions(&desc, camera, &c, close) else {
                    continue;
                };
                positions.push(json!({"map":map,"camera":camera,"close":close,"points":points}));
            }
        }
    }
    for i in 0..128u16 {
        let group = [0x1d6db6, 0x1d6dae][i as usize % 2];
        let map = if i % 7 == 0 { 55 } else { 5 };
        let mut actors = vec![];
        for slot in 0..10u16 {
            let mut a = Runner::load(rom, 0, 0)?;
            a.set_word(0x3e, slot);
            a.raw[0x10] = if slot < 3 {
                0xd
            } else if (i >> (slot - 3)) & 1 != 0 {
                5
            } else {
                0
            };
            a.set_word(0x30, slot.saturating_sub(3) + i % 7);
            actors.push(a);
        }
        let points = vec![(0x1100, 0x2200), (0x3300, 0x4400)];
        let seed = i.wrapping_mul(811);
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let (added, after_seed) = encounters::spawn(rom, group, &points, &actors, map, seed)?;
        let count = added.len();
        for a in added {
            let slot = a.word(0x3e) as usize;
            actors[slot] = a;
        }
        groups.push(json!({"group":group,"map":map,"seed":seed,"after_seed":after_seed,"before":before,"after":actors.iter().flat_map(|a|a.raw).collect::<Vec<_>>(),"count":count,"points":points}));
    }
    // Real offscreen $13904 -> $1450e pursuit, using the complete native decision.
    let desc = MapDescriptor::read(rom, 5)?;
    let c = Collision::load(rom, &desc)?;
    for i in 0..128u16 {
        let mut actors = vec![];
        for slot in 0..10u16 {
            let mut a = Runner::load(rom, 0, 0)?;
            a.set_word(0x3e, slot);
            a.raw[0x10] = if slot == 0 { 0xf } else { 0 };
            actors.push(a);
        }
        actors[0].set_position(0x1000 + (i % 8) * 0x200, 0x1000 + (i / 8) * 0x100);
        actors[0].set_word(0x2e, i.wrapping_mul(1111));
        let mut cursor = 0x1d6dba;
        let mut rng = State { seed: i };
        actors[3] = shadowrealm::npc::initialize(
            rom,
            0x1d657c,
            0x1e300,
            3,
            0,
            0,
            5,
            false,
            &mut cursor,
            &mut rng,
        )?;
        let origin = actors[0].position();
        actors[3].set_position(origin.0.wrapping_add(0x1000), origin.1.wrapping_add(0x800));
        actors[3].set_word(0xc, 0);
        actors[3].set_word(0xb6, 0);
        let before = actors.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let mut runtime = shadowrealm::combat_runtime::Runtime::default();
        runtime.launch.engaged = 0xffff;
        runtime.npc(rom, &c, &mut actors, 3, false, i, &mut rng)?;
        offscreen.push(
            json!({"before":before,"after":actors.iter().flat_map(|a|a.raw).collect::<Vec<_>>()}),
        );
    }
    // Reproducible native gameplay evidence: normal map 5 arrival and real RNG.
    let mut game = shadowrealm::game::Game::load(rom, 5, &[0])?;
    let mut trace = String::from("tick,npcs,encounters,npc_shots,health,ammo\n");
    let mut captured = false;
    for _ in 0..120_000 {
        game.update(rom, 0)?;
        if game.encounters.spawned > 0 {
            trace.push_str(&format!(
                "{},{},{},{},{},{}\n",
                game.tick,
                game.npcs.len(),
                game.encounters.spawned,
                game.combat.npc_shots,
                game.party[0].raw[0x4c],
                game.party[0].raw[0x4e]
            ));
            if !captured || game.party[0].raw[0x4c] < 10 {
                let name = if captured {
                    "street-impact.ppm"
                } else {
                    "street-spawn.ppm"
                };
                let mut file = fs::File::create(out.join(name))?;
                file.write_all(b"P6\n320 224\n255\n")?;
                file.write_all(&game.render(rom)?)?;
                captured = true;
            }
            if game.party[0].raw[0x4c] < 10 {
                break;
            }
        }
    }
    if !captured || game.party[0].raw[0x4c] >= 10 {
        return Err("native street encounter replay did not reach damage".into());
    }
    fs::write(out.join("street-trace.csv"), trace)?;
    // Explicit wounded-party fixture for the host defeat UI, not a new-game rule.
    let mut wounded = shadowrealm::game::Game::load(rom, 5, &[0])?;
    wounded.recovery_flags = 2; // Preserve the unsupported-flag host notice fixture.
    wounded.party[0].raw[0x4c] = 1;
    for _ in 0..12_000 {
        wounded.update(rom, 0)?;
        if wounded.defeated {
            break;
        }
    }
    if !wounded.defeated {
        return Err("wounded encounter did not reach defeat".into());
    }
    let mut file = fs::File::create(out.join("wounded-defeat.ppm"))?;
    file.write_all(b"P6\n320 224\n255\n")?;
    file.write_all(&wounded.render(rom)?)?;
    fs::write(
        out.join("encounters.json"),
        serde_json::to_vec(
            &json!({"selections":selections,"gates":gates,"positions":positions,"groups":groups,"offscreen":offscreen}),
        )?,
    )?;
    Ok(())
}
