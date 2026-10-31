//! Routine-level differential fixtures; not an alternate follower implementation.
use serde_json::json;
use shadowrealm::{
    collision::{Collision, Contact},
    followers,
    maps::MapDescriptor,
    party::Runner,
};
use std::{error::Error, fs, io::Write, path::Path};
fn bytes(r: &Runner) -> Vec<u8> {
    r.raw.to_vec()
}
fn pose(rom: &[u8], slot: usize, position: (u16, u16)) -> Result<Runner, Box<dyn Error>> {
    let mut r = Runner::load(rom, [0, 3, 5][slot], slot)?;
    r.raw[0x10] &= !2;
    r.set_position(position.0, position.1);
    r.set_word(0xc, 128);
    Ok(r)
}
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut math = vec![];
    let mut decisions = vec![];
    let mut movement = vec![];
    for i in 0..1024u32 {
        let from = ((i * 971 + 304) & 32767, (i * 239 + 419) & 32767);
        let to = ((i * 337 + 89) & 32767, (i * 659 + 853) & 32767);
        let (angle, range) = followers::heading(
            rom,
            (from.0 as u16, from.1 as u16),
            (to.0 as u16, to.1 as u16),
        )?;
        math.push(json!({"from":[from.0,from.1],"to":[to.0,to.1],"angle":angle,"range":range,"stop_distance":followers::distance((from.0 as u16,from.1 as u16),(to.0 as u16,to.1 as u16))}));
    }
    for map_id in [0, 3, 5, 10] {
        let map = MapDescriptor::read(rom, map_id)?;
        let collision = Collision::load(rom, &map)?;
        let mut positions = vec![];
        for i in 0..4096u32 {
            let left = collision.bounds[0].max(48);
            let top = collision.bounds[1].max(48);
            let x = left as u32
                + (i * 73 + 11) % u32::from(collision.bounds[2].saturating_sub(left + 48).max(1));
            let y = top as u32
                + (i * 97 + 19) % u32::from(collision.bounds[3].saturating_sub(top + 48).max(1));
            let p = (x as u16 * 16, y as u16 * 16);
            if collision.contact(p)? == Contact::Clear {
                positions.push(p);
                if positions.len() == 32 {
                    break;
                }
            }
        }
        if positions.len() < 16 {
            return Err("insufficient clear follower fixture points".into());
        }
        for i in 0..192usize {
            let pair = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)][i % 6];
            let mut leader = pose(rom, pair.0, positions[(i * 7 + 3) % positions.len()])?;
            leader.raw[0x10] |= 2;
            leader.set_word(0x2e, ((i / 6) % 16) as u16 * 0x1000);
            let mut r = pose(rom, pair.1, positions[(i * 11 + 7) % positions.len()])?;
            r.set_word(0xb8, [0, 0x1000, 0xffff][(i / 3) % 3]);
            let before = bytes(&r);
            let goal = followers::formation_goal(rom, &collision, &r, &leader)?;
            let supported = followers::decide(rom, &collision, &mut r, &leader)?;
            decisions.push(json!({"map":map_id,"leader":bytes(&leader),"before":before,"goal":[goal.0,goal.1],"supported":supported,"after":bytes(&r)}));
        }
        for i in 0..256usize {
            let mut r = pose(rom, 1, positions[i % positions.len()])?;
            r.set_word(0x2e, ((i * 7) % 32) as u16 * 0x800);
            r.set_word(0x34, ((i * 13) % 32) as u16 * 0x800);
            r.set_word(
                0xc8,
                [
                    0, 0xff, 0x100, 0x1ff, 0x200, 0x35f, 0x360, 0x45f, 0x460, 0x1000,
                ][i % 10],
            );
            r.set_word(0xc, if i % 3 == 0 { 0 } else { 128 });
            r.raw[0x3a..0x3e].copy_from_slice(&0xb574u32.to_be_bytes());
            let before = bytes(&r);
            followers::move_one(rom, &collision, &mut r)?;
            movement.push(json!({"map":map_id,"before":before,"after":bytes(&r)}));
        }
    }
    let mut cells = vec![0x7f; 4096];
    for y in 4..=10 {
        for x in 4..=14 {
            cells[y * 64 + x] = 0;
        }
    }
    for y in 4..=8 {
        cells[y * 64 + 8] = 0x7f;
    }
    let corridor = Collision {
        cells,
        rectangles: vec![],
        bounds: [0, 0, 512, 512],
        overflow_writes: vec![],
    };
    let mut party = vec![
        pose(rom, 0, (368 * 16, 240 * 16))?,
        pose(rom, 1, (176 * 16, 240 * 16))?,
        pose(rom, 2, (176 * 16, 270 * 16))?,
    ];
    party[0].raw[0x10] |= 2;
    party[0].set_word(0x2e, 0x4000);
    let initial: Vec<_> = party.iter().flat_map(|r| r.raw).collect();
    let mut ticks = vec![];
    for frame in 0..900 {
        let leader = party[0].clone();
        if frame < 31 {
            followers::warmup(rom, &corridor, &mut party, 0)?;
        } else {
            followers::tick(rom, &corridor, &mut party, 0, &leader)?;
        }
        ticks.push(party.iter().flat_map(|r| r.raw).collect::<Vec<_>>());
    }
    let timeline =
        json!({"cells":corridor.cells,"initial":initial,"ticks":ticks,"warmup_frames":31});
    let mut boxed = Collision {
        cells: vec![0x7f; 4096],
        rectangles: vec![],
        bounds: [0, 0, 512, 512],
        overflow_writes: vec![],
    };
    boxed.cells[7 * 64 + 5] = 0;
    for y in 4..=10 {
        for x in 10..=14 {
            boxed.cells[y * 64 + x] = 0;
        }
    }
    let mut withheld_cases = vec![];
    for angle in [0, 0x4000, 0x8000, 0xc000] {
        let mut leader = pose(rom, 0, (368 * 16, 240 * 16))?;
        leader.set_word(0x2e, angle);
        let mut runner = pose(rom, 1, (176 * 16, 240 * 16))?;
        runner.set_word(0xb8, 0xffff);
        let before = bytes(&runner);
        let supported = followers::decide(rom, &boxed, &mut runner, &leader)?;
        withheld_cases.push(json!({"leader":bytes(&leader),"before":before,"supported":supported,"after":bytes(&runner)}));
    }
    fs::write(
        out.join("followers.json"),
        serde_json::to_vec_pretty(
            &json!({"math":math,"decisions":decisions,"movement":movement,"timeline":timeline,"withheld_cases":withheld_cases,"boxed_cells":boxed.cells}),
        )?,
    )?;
    // Seeded visual replay of the desktop core; earned hiring is tested separately.
    let mut game = shadowrealm::game::Game::load(rom, 0, &[0, 5])?;
    game.party[0].set_position(528 * 16, 160 * 16);
    game.party[0].set_word(0x2e, 0x8000);
    game.party[1].set_position(548 * 16, 160 * 16);
    game.camera = (392, 32);
    let mut trace = fs::File::create(out.join("game-replay.csv"))?;
    writeln!(trace, "tick,selected,x0,y0,x1,y1,camera_x,camera_y")?;
    let mut frame = 0;
    for (selected, count, held) in [
        (0, 90, shadowrealm::input::LEFT),
        (0, 180, 0),
        (1, 90, shadowrealm::input::RIGHT),
        (1, 180, 0),
    ] {
        game.select(rom, selected)?;
        for _ in 0..count {
            game.update(rom, held)?;
            let p0 = game.party[0].position();
            let p1 = game.party[1].position();
            writeln!(
                trace,
                "{frame},{selected},{},{},{},{},{},{}",
                p0.0, p0.1, p1.0, p1.1, game.camera.0, game.camera.1
            )?;
            if frame % 30 == 0 {
                let mut f = fs::File::create(out.join(format!("replay-{frame:04}.ppm")))?;
                f.write_all(b"P6\n320 224\n255\n")?;
                f.write_all(&game.render(rom)?)?;
            }
            frame += 1;
        }
    }
    println!("Exported 1024 math, 768 formation/decision and 1024 movement cases");
    Ok(())
}
