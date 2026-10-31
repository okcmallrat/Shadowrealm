//! Map tile-animation scheduler from $164a8/$163da/$1641a.
use crate::{map_render::MapScene, maps::MapDescriptor, RomError};
fn w(b: &[u8], p: usize) -> Result<u16, RomError> {
    let a = b
        .get(p..p + 2)
        .ok_or(RomError("animation data outside ROM"))?;
    Ok(u16::from_be_bytes([a[0], a[1]]))
}
fn l(b: &[u8], p: usize) -> Result<usize, RomError> {
    let a = b
        .get(p..p + 4)
        .ok_or(RomError("animation data outside ROM"))?;
    Ok(u32::from_be_bytes(a.try_into().unwrap()) as usize)
}
pub struct TileAnimation {
    pub base: usize,
    pub frames: Vec<u16>,
    pub tiles: Vec<u8>,
    pub period: i16,
    pub delay: i16,
    pub frame: usize,
}
pub struct Environment {
    pub animations: Vec<TileAnimation>,
    graphics: usize,
}
impl Environment {
    pub fn load(rom: &[u8], map: &MapDescriptor) -> Result<Self, RomError> {
        let graphics = l(&map.raw, 0x38)?;
        let mut cursor = l(&map.raw, 0x3c)?;
        let mut animations = Vec::new();
        loop {
            let base = l(rom, cursor)?;
            if base == 0 {
                break;
            }
            if animations.len() == 10 {
                return Err(RomError("map animation slots exhausted"));
            }
            let spec = l(rom, cursor + 4)?;
            cursor += 8;
            let count = w(rom, spec)? as usize;
            if count == 0 || count > 256 {
                return Err(RomError("invalid map animation count"));
            }
            let period = w(rom, spec + 2)? as i16;
            let mut frames = Vec::new();
            for i in 0..count {
                frames.push(w(rom, spec + 4 + i * 2)?);
            }
            let mut tiles = Vec::new();
            let mut p = spec + 4 + count * 2;
            loop {
                let index = *rom
                    .get(p)
                    .ok_or(RomError("unterminated map animation indices"))?;
                p += 1;
                if index & 0x80 != 0 {
                    break;
                }
                tiles.push(index);
                if tiles.len() > 128 {
                    return Err(RomError("map animation tile list too long"));
                }
            }
            animations.push(TileAnimation {
                base,
                frames,
                tiles,
                period,
                delay: 0,
                frame: 0,
            });
        }
        Ok(Self {
            animations,
            graphics,
        })
    }
    pub fn tick(&mut self, rom: &[u8], scene: &mut MapScene) -> Result<usize, RomError> {
        let mut transfers = 0;
        for animation in &mut self.animations {
            animation.delay = animation.delay.wrapping_sub(1);
            if animation.delay >= 0 {
                continue;
            }
            animation.frame = (animation.frame + 1) % animation.frames.len();
            animation.delay = animation.period;
            let target = w(rom, animation.base)?;
            let source = w(
                rom,
                animation.base + animation.frames[animation.frame] as usize * 2,
            )?;
            for tile in &animation.tiles {
                let src = scene.definition_word(source, *tile as usize)? & 0x7ff;
                let displacement = (src.wrapping_sub(0x201).wrapping_shl(5) as i16) as isize;
                let address = self
                    .graphics
                    .checked_add_signed(displacement)
                    .ok_or(RomError("animated graphics pointer overflow"))?;
                let destination = scene
                    .definition_word(target, *tile as usize)?
                    .wrapping_shl(5) as usize;
                scene.transfer(rom, address, destination, 32)?;
                transfers += 1;
            }
        }
        Ok(transfers)
    }
}
