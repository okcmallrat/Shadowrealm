//! Developer fixture: seed outside a real door, then use the desktop game core
//! for dialogue, entry and return. No map reassignment after initialization.
use shadowrealm::{game::Game, input, maps::MapDescriptor, transitions::TransitionPlan};
use std::{error::Error, fs, io::Write, path::Path};
fn capture(
    game: &mut Game,
    rom: &[u8],
    directory: &Path,
    name: &str,
) -> Result<(), Box<dyn Error>> {
    let mut out = fs::File::create(directory.join(format!("{name}.ppm")))?;
    out.write_all(b"P6\n320 224\n255\n")?;
    out.write_all(&game.render(rom)?)?;
    Ok(())
}
fn settle(game: &mut Game, rom: &[u8], held: u8, target: usize) -> Result<(), Box<dyn Error>> {
    let before = game.completed_transitions;
    for _ in 0..300 {
        game.update(rom, held)?;
        if game.completed_transitions > before && game.transition.is_none() {
            if game.map_id() != target {
                return Err("interior replay reached unexpected map".into());
            }
            return Ok(());
        }
    }
    Err("interior replay did not finish its transition".into())
}
pub fn run(rom: &[u8], directory: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(directory)?;
    let mut game = Game::load(rom, 0, &[0, 3, 10])?;
    game.party[0].set_position(528 * 16, 144 * 16);
    game.party[0].set_word(0x2e, 0);
    game.camera = (392, 32);
    capture(&mut game, rom, directory, "approach")?;
    for _ in 0..20 {
        game.update(rom, input::UP)?;
        if game.dialogue.is_some() {
            break;
        }
    }
    let dialogue = game
        .dialogue
        .as_ref()
        .ok_or("entrance did not show exposition")?;
    fs::write(directory.join("text.txt"), &dialogue.text)?;
    fs::write(directory.join("font.bin"), &dialogue.font)?;
    let (x, y) = game.party[0].position();
    let plan = TransitionPlan::space_needle(
        rom,
        &MapDescriptor::read(rom, 0)?,
        0xc6,
        (game.camera.0 * 16, game.camera.1 * 16),
        (x, y),
    )?
    .ok_or("missing entry plan")?;
    let saved = plan.exterior_return.ok_or("missing return plan")?;
    fs::write(directory.join("saved-descriptor.bin"), saved.descriptor)?;
    let mut out = fs::File::create(directory.join("plan.csv"))?;
    writeln!(out,"source_map,trigger,player_x,player_y,saved_camera_x,saved_camera_y,target_map,camera_x,camera_y,x0,y0,x1,y1,x2,y2,return_x0,return_y0,return_x1,return_y1,return_x2,return_y2,node,text")?;
    writeln!(
        out,
        "0,198,{x},{y},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        saved.camera_fixed.0,
        saved.camera_fixed.1,
        plan.target_map,
        plan.camera_fixed.0,
        plan.camera_fixed.1,
        plan.positions[0].0,
        plan.positions[0].1,
        plan.positions[1].0,
        plan.positions[1].1,
        plan.positions[2].0,
        plan.positions[2].1,
        saved.positions[0].0,
        saved.positions[0].1,
        saved.positions[1].0,
        saved.positions[1].1,
        saved.positions[2].0,
        saved.positions[2].1,
        dialogue.node,
        dialogue.text_address
    )?;
    fs::write(
        directory.join("npc.bin"),
        shadowrealm::npc::space_needle(rom)?[0].raw,
    )?;
    capture(&mut game, rom, directory, "exposition")?;
    game.update(rom, input::A)?;
    settle(&mut game, rom, 0, 10)?;
    capture(&mut game, rom, directory, "interior")?;
    settle(&mut game, rom, input::DOWN, 0)?;
    capture(&mut game, rom, directory, "return")?;
    println!("Captured Space Needle approach, exposition, entry and exterior return through Game::update");
    Ok(())
}
