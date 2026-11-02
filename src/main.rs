use shadowrealm::Rom;
mod avoidance_export;
mod clinic_export;
mod combat_export;
mod dialogue_export;
mod encounter_export;
mod follower_export;
mod hiring_export;
mod interior_export;
use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
};
fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return Err(
            "usage: shadowrealm <inspect|strings|tile|region-screen|maps|map|render-map|render-animation> ROM [tile-offset output.ppm]"
                .into(),
        );
    }
    let data = fs::read(&args[2])?;
    let rom = Rom::parse(&data)?;
    match args[1].as_str() {
        "encounter-data" => {
            if args.len() != 4 {
                return Err("encounter-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            encounter_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "clinic-data" => {
            if args.len() != 4 {
                return Err("clinic-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            clinic_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "combat-data" => {
            if args.len() != 4 {
                return Err("combat-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            combat_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "avoidance-data" => {
            if args.len() != 4 {
                return Err("avoidance-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            avoidance_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "follower-data" => {
            if args.len() != 4 {
                return Err("follower-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            follower_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "hiring-data" => {
            if args.len() != 4 {
                return Err("hiring-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            hiring_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "dialogue-data" => {
            if args.len() != 4 {
                return Err("dialogue-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            dialogue_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "interior-data" => {
            if args.len() != 4 {
                return Err("interior-data ROM NEW_OUTPUT_DIRECTORY".into());
            }
            interior_export::run(&data, std::path::Path::new(&args[3]))?;
        }
        "inspect" => {
            println!(
                "Title: {}\nProduct: {}\nBytes: {}",
                rom.text(0x120, 0x150)?,
                rom.text(0x180, 0x18e)?,
                data.len()
            );
            println!(
                "Checksum: stored {:04x}, computed {:04x}",
                rom.stored_checksum(),
                rom.checksum()
            );
            for (index, name) in [
                (0, "initial stack"),
                (1, "reset"),
                (28, "level 4 interrupt"),
                (30, "level 6 interrupt"),
            ] {
                println!("{name}: 0x{:08x}", rom.vector(index)?);
            }
            if rom.stored_checksum() != rom.checksum() {
                return Err("ROM checksum mismatch".into());
            }
        }
        "strings" => {
            let mut out = io::BufWriter::new(io::stdout().lock());
            for (offset, text) in rom.strings(12) {
                writeln!(out, "{offset:06x}: {text}")?;
            }
        }
        "actor-catalog" => {
            if args.len() != 4 {
                return Err("actor-catalog ROM NEW_OUTPUT.csv".into());
            }
            let mut scene = shadowrealm::map_render::MapScene::load(&data, 0)?;
            let mut out = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[3])?;
            writeln!(out,"bank,state,frame_index,sequence,terminal,speed,frame,piece,x,y,width,height,attribute")?;
            for bank in 0..16 {
                for state in 0..shadowrealm::actors::state_count(bank)? {
                    let seq = shadowrealm::actors::Sequence::read(&data, bank, state)?;
                    for (index, address) in seq.frames.iter().enumerate() {
                        let frame = shadowrealm::actors::ActorFrame::load(
                            &data, *address, &mut scene, 0x100, 0x2000,
                        )?;
                        if frame.pieces.is_empty() {
                            writeln!(
                                out,
                                "{bank},{state},{index},{},{},{},{address},-1,0,0,0,0,0",
                                seq.address, seq.terminal, seq.speed
                            )?;
                        }
                        for (piece, p) in frame.pieces.iter().enumerate() {
                            writeln!(
                                out,
                                "{bank},{state},{index},{},{},{},{address},{piece},{},{},{},{},{}",
                                seq.address,
                                seq.terminal,
                                seq.speed,
                                p.x,
                                p.y,
                                p.width,
                                p.height,
                                p.attribute
                            )?;
                        }
                    }
                }
            }
        }
        "hud-assets" => {
            if args.len() != 4 {
                return Err("hud-assets ROM NEW_OUTPUT_DIRECTORY".into());
            }
            let mut scene = shadowrealm::map_render::MapScene::load(&data, 0)?;
            let mut hud = shadowrealm::hud::Hud::load(&data, &mut scene)?;
            for (slot, health) in [10, 7, 4].iter().enumerate() {
                hud.health(&data, slot, *health)?;
                hud.portrait(&data, &mut scene, slot, 0x34f8, 0x2000)?;
                hud.equipment(&data, &mut scene, slot, 0xd1860, 0x2000)?;
                hud.resource(&data, slot, 0)?;
            }
            let directory = std::path::Path::new(&args[3]);
            fs::create_dir(directory)?;
            fs::write(directory.join("vram.bin"), scene.vram())?;
            let words: Vec<u8> = hud.words().iter().flat_map(|w| w.to_be_bytes()).collect();
            fs::write(directory.join("window.bin"), words)?;
        }
        "replay" => {
            if args.len() != 6 {
                return Err("replay ROM MAP INPUT.csv NEW_OUTPUT_DIRECTORY".into());
            }
            let mut game = shadowrealm::game::Game::load(&data, args[3].parse()?, &[0, 3, 10])?;
            let input = fs::read_to_string(&args[4])?;
            let directory = std::path::Path::new(&args[5]);
            fs::create_dir(directory)?;
            let mut trace = fs::File::create(directory.join("state.csv"))?;
            writeln!(trace,"tick,selected,x,y,camera_x,camera_y,state,phase,animation_frame,blocked,transition,map,transition_stage,completed_transitions,shots,hits,ammo,reserves,projectiles")?;
            let mut total = 0;
            for line in input
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with("ticks,"))
            {
                let fields: Vec<_> = line.split(',').map(str::trim).collect();
                if fields.len() != 3 {
                    return Err("input rows must contain ticks,held-buttons,selected-slot".into());
                }
                let count: usize = fields[0].parse()?;
                let held: u8 = fields[1].parse()?;
                let selected: usize = fields[2].parse()?;
                if total + count > 600 {
                    return Err("replay is limited to 600 ticks".into());
                }
                if game.selected != selected {
                    game.select(&data, selected)?;
                }
                for _ in 0..count {
                    game.update(&data, held)?;
                    let pixels = game.render(&data)?;
                    let mut file =
                        fs::File::create(directory.join(format!("frame-{total:04}.ppm")))?;
                    file.write_all(b"P6\n320 224\n255\n")?;
                    file.write_all(&pixels)?;
                    let runner = &game.party[game.selected];
                    let (x, y) = runner.position();
                    writeln!(
                        trace,
                        "{total},{},{x},{y},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                        game.selected,
                        game.camera.0,
                        game.camera.1,
                        runner.state(),
                        runner.clock.phase,
                        runner.clock.frame,
                        game.blocked,
                        game.pending_transition
                            .map(|c| format!("0x{c:02x}"))
                            .unwrap_or_default(),
                        game.map_id(),
                        game.transition
                            .map(|a| format!("{:?}", a.stage))
                            .unwrap_or_default(),
                        game.completed_transitions,
                        game.combat.shots,
                        game.combat.hits,
                        runner.raw[0x4e],
                        runner.raw[0x4f],
                        game.combat.pool.iter().filter(|p| p[0x10] & 1 != 0).count()
                    )?;
                    total += 1;
                }
            }
            println!("Simulated and rendered {total} persistent exploration ticks");
        }
        "transition-data" => {
            if args.len() != 4 {
                return Err("transition-data ROM NEW_OUTPUT.csv".into());
            }
            let mut out = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[3])?;
            writeln!(out,"source_map,trigger,camera_input,target_map,spawn,camera_x,camera_y,x0,y0,x1,y1,x2,y2")?;
            for map in [0, 3, 5] {
                for trigger in [0xb0, 0xf0, 0xc0] {
                    for camera in [0, 0x1380, 0x13ff, 0x1400, 0x6c80, 0xffff] {
                        if let Some(p) = shadowrealm::transitions::TransitionPlan::exterior(
                            &data, map, trigger, camera,
                        )? {
                            writeln!(
                                out,
                                "{map},{trigger},{camera},{},{},{},{},{},{},{},{},{},{}",
                                p.target_map,
                                p.spawn_address,
                                p.camera_fixed.0,
                                p.camera_fixed.1,
                                p.positions[0].0,
                                p.positions[0].1,
                                p.positions[1].0,
                                p.positions[1].1,
                                p.positions[2].0,
                                p.positions[2].1
                            )?;
                        } else {
                            writeln!(out, "{map},{trigger},{camera},-1,0,0,0,0,0,0,0,0,0")?;
                        }
                    }
                }
            }
        }
        "world-data" => {
            if args.len() != 5 {
                return Err("world-data ROM MAP NEW_OUTPUT_DIRECTORY".into());
            }
            let map = shadowrealm::maps::MapDescriptor::read(&data, args[3].parse()?)?;
            let collision = shadowrealm::collision::Collision::load(&data, &map)?;
            let mut scene = shadowrealm::map_render::MapScene::load(&data, map.id)?;
            let mut environment = shadowrealm::environment::Environment::load(&data, &map)?;
            let directory = std::path::Path::new(&args[4]);
            fs::create_dir(directory)?;
            fs::write(directory.join("collision.bin"), &collision.cells)?;
            let mut queries = fs::File::create(directory.join("queries.csv"))?;
            writeln!(queries, "x,y,result")?;
            // Deterministic interior samples exercise several boundaries and cell corners.
            for i in 0..128u32 {
                let left = u32::from(collision.bounds[0]).max(12);
                let top = u32::from(collision.bounds[1]).max(7);
                let width = u32::from(collision.bounds[2]).saturating_sub(left).max(1);
                let height = u32::from(collision.bounds[3]).saturating_sub(top).max(1);
                let x = (left + (i * 73 + 11) % width) as u16;
                let y = (top + (i * 97 + 19) % height) as u16;
                let result = match collision.contact((x * 16, y * 16))? {
                    shadowrealm::collision::Contact::Clear => 0,
                    shadowrealm::collision::Contact::Blocked => 127,
                    shadowrealm::collision::Contact::Trigger(c) => c,
                };
                writeln!(queries, "{},{},{result}", x * 16, y * 16)?;
            }
            for tick in 0..16 {
                environment.tick(&data, &mut scene)?;
                fs::write(directory.join(format!("vram-{tick:02}.bin")), scene.vram())?;
            }
            let game = shadowrealm::game::Game::load(&data, map.id, &[0, 3, 10])?;
            let runners: Vec<u8> = game.party.iter().flat_map(|r| r.raw).collect();
            fs::write(directory.join("runners.bin"), runners)?;
            fs::write(directory.join("hud-vram.bin"), game.scene.vram())?;
            fs::write(
                directory.join("window.bin"),
                game.hud
                    .words()
                    .iter()
                    .flat_map(|w| w.to_be_bytes())
                    .collect::<Vec<u8>>(),
            )?;
        }
        "render-animation" => {
            if args.len() != 7 && args.len() != 8 {
                return Err("render-animation ROM MAP_ID BANK STATE NEW_OUTPUT_DIR [TICKS]".into());
            }
            let map_id = args[3].parse()?;
            let bank = args[4].parse()?;
            let state = args[5].parse()?;
            let ticks: usize = if args.len() == 8 {
                args[7].parse()?
            } else {
                96
            };
            if ticks == 0 || ticks > 600 {
                return Err("ticks must be 1..600".into());
            }
            let sequence = shadowrealm::actors::Sequence::read(&data, bank, state)?;
            let mut scene = shadowrealm::map_render::MapScene::load(&data, map_id)?;
            let mut hud = shadowrealm::hud::Hud::load(&data, &mut scene)?;
            // Deliberate demonstrator state, not recovered save-game statistics.
            for (slot, health) in [10, 7, 4].iter().enumerate() {
                hud.health(&data, slot, *health)?;
                hud.portrait(&data, &mut scene, slot, 0x34f8, 0x2000)?;
                hud.equipment(&data, &mut scene, slot, 0xd1860, 0x2000)?;
                hud.resource(&data, slot, 0)?;
            }
            let mut clock = shadowrealm::actors::AnimationClock::default();
            let directory = std::path::Path::new(&args[6]);
            fs::create_dir(directory)?;
            let mut metadata = fs::File::create(directory.join("frames.csv"))?;
            writeln!(metadata, "tick,sequence_frame,rom_frame,phase")?;
            let increment = sequence.increment(8, 0);
            for tick in 0..ticks {
                let frame = shadowrealm::actors::ActorFrame::load(
                    &data,
                    sequence.frames[clock.frame as usize],
                    &mut scene,
                    0x100,
                    0x2000,
                )?;
                let pixels = shadowrealm::scene::render(
                    &scene,
                    (0, 0),
                    &[shadowrealm::scene::PlacedActor {
                        frame: &frame,
                        x: 160,
                        y: 184,
                    }],
                    Some(&hud),
                )?;
                let mut out = fs::File::create(directory.join(format!("frame-{tick:04}.ppm")))?;
                out.write_all(b"P6\n320 224\n255\n")?;
                out.write_all(&pixels)?;
                writeln!(
                    metadata,
                    "{tick},{},0x{:06x},{}",
                    clock.frame, frame.address, clock.phase
                )?;
                clock.tick(
                    0x1000,
                    sequence.increment(8, clock.frame),
                    sequence.frames.len() as u16,
                    sequence.terminal,
                )?;
            }
            fs::write(directory.join("README.txt"),format!(
                "Native rendering demonstration, not a captured gameplay session.\nMap {map_id}; actor bank {bank}, state {state}; sequence 0x{:06x}.\nCharacter anchor=(160,184), camera=(0,0), VRAM tile base=0x100, palette=1.\nClock period=0x1000 (entity template); initial step={increment}; nonterminal speed sample=8.\nHUD health inputs are sample values [10,7,4]. Portraits are blank; icons use the ROM unarmed definition; item resource is zero.\nOne tick per output frame; presentation at 60 Hz is a preview choice.\nNo game AI, movement, save state, damage, or equipment simulation is implied.\n",sequence.address))?;
            println!(
                "Rendered {ticks} native animation frames to {}",
                directory.display()
            );
        }
        "render-map" => {
            if args.len() != 7 && args.len() != 9 {
                return Err("render-map ROM ID X Y output.ppm [WIDTH HEIGHT]".into());
            }
            let scene = shadowrealm::map_render::MapScene::load(&data, args[3].parse()?)?;
            let (width, height) = if args.len() == 9 {
                (args[7].parse()?, args[8].parse()?)
            } else {
                (320, 224)
            };
            let pixels = scene.render(args[4].parse()?, args[5].parse()?, width, height)?;
            let mut out = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[6])?;
            write!(out, "P6\n{width} {height}\n255\n")?;
            out.write_all(&pixels)?;
            println!("Rendered {width}x{height} static map planes to {}", args[6]);
        }
        "maps" => {
            for id in 0..shadowrealm::maps::MAP_COUNT {
                let m = shadowrealm::maps::MapDescriptor::read(&data, id)?;
                println!(
                    "{id:02} descriptor={:06x} stride={} grids={:06x},{:06x} tiles={:06x},{:06x}",
                    m.address,
                    m.row_stride_bytes(),
                    m.grid_pointer(0)?,
                    m.grid_pointer(1)?,
                    m.tile_pointer(0)?,
                    m.tile_pointer(1)?
                );
            }
        }
        "map" => {
            if args.len() != 5 {
                return Err("map requires a decimal map ID and a new output directory".into());
            }
            let m = shadowrealm::maps::MapDescriptor::read(&data, args[3].parse()?)?;
            // Validate both copies and the optional compressed stream before writing outputs.
            let grids = [m.grid_copy(&data, 0)?, m.grid_copy(&data, 1)?];
            let pointer = m.tile_pointer(0)?;
            let decoded = if pointer & 1 != 0 {
                let source = data
                    .get((pointer & !1) as usize..)
                    .ok_or("tile stream outside ROM")?;
                Some(shadowrealm::maps::decompress(source)?)
            } else {
                None
            };
            let dir = std::path::Path::new(&args[4]);
            fs::create_dir(dir)?;
            fs::write(dir.join("descriptor.bin"), m.raw)?;
            for (layer, grid) in grids.iter().enumerate() {
                fs::write(dir.join(format!("layer{layer}-copy.bin")), grid)?;
            }
            if let Some((tiles, consumed)) = decoded {
                fs::write(dir.join("shared-tile-definitions.bin"), &tiles)?;
                println!(
                    "Decoded {} tile-definition bytes from {consumed} source bytes",
                    tiles.len()
                );
            }
            fs::write(dir.join("metadata.txt"),format!(
                "map_id={}\ndescriptor=0x{:06x}\nrow_stride_bytes={}\nlayer0_grid=0x{:06x}\nlayer1_grid=0x{:06x}\nlayer0_tiles=0x{:06x}\nlayer1_tiles=0x{:06x}\nFixed grid copies include trailing data; logical height unknown.\nTile pointer bit 0 marks compression on layer 0; when set the engine shares the decoded definitions across layers.\n",
                m.id,m.address,m.row_stride_bytes(),m.grid_pointer(0)?,m.grid_pointer(1)?,m.tile_pointer(0)?,m.tile_pointer(1)?))?;
            println!("Extracted map {} to {}", m.id, dir.display());
        }
        "region-screen" => {
            if args.len() != 4 {
                return Err("region-screen requires output.ppm path".into());
            }
            let pixels = shadowrealm::video::region_screen(&data)?;
            let mut out = fs::File::create(&args[3])?;
            out.write_all(b"P6\n320 224\n255\n")?;
            out.write_all(&pixels)?;
        }
        "tile" => {
            if args.len() != 5 {
                return Err("tile requires an offset and output.ppm path".into());
            }
            let offset = if let Some(hex) = args[3].strip_prefix("0x") {
                usize::from_str_radix(hex, 16)?
            } else {
                args[3].parse()?
            };
            let pixels = rom.tile(offset)?;
            let mut out = fs::File::create(&args[4])?;
            out.write_all(b"P6\n8 8\n255\n")?;
            for pixel in pixels {
                out.write_all(&[pixel * 17; 3])?;
            }
        }
        _ => return Err("unknown command".into()),
    }
    Ok(())
}
