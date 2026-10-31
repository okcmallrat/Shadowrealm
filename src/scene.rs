//! Composes native map planes, linked sprite pieces and the original HUD window.
use crate::{actors::ActorFrame, hud::Hud, map_render::MapScene, RomError};
pub struct PlacedActor<'a> {
    pub frame: &'a ActorFrame,
    pub x: i32,
    pub y: i32,
}
pub fn render(
    scene: &MapScene,
    origin: (u16, u16),
    actors: &[PlacedActor<'_>],
    hud: Option<&Hud>,
) -> Result<Vec<u8>, RomError> {
    // Validate crop once, including horizontal stride and fixed-buffer bounds.
    scene.render(
        origin.0,
        origin.1,
        if hud.is_some() { 272 } else { 320 },
        224,
    )?;
    let mut pixels = Vec::with_capacity(320 * 224 * 3);
    for y in 0..224 {
        for x in 0..320 {
            let mut sprite = None;
            for actor in actors {
                sprite = actor
                    .frame
                    .pixel(scene, x as i32, y as i32, actor.x, actor.y)?;
                if sprite.is_some() {
                    break;
                }
            }
            pixels.extend_from_slice(&scene.frame_pixel(
                origin.0 + x as u16,
                origin.1 + y as u16,
                x,
                y,
                hud.and_then(|h| h.attribute(x, y)),
                sprite,
            )?);
        }
    }
    Ok(pixels)
}
