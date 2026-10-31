//! Character animation and sprite records from $5fdc, $600e and $622c.
use crate::{map_render::MapScene, RomError};
fn data(b: &[u8], p: usize, n: usize) -> Result<&[u8], RomError> {
    b.get(p..p.checked_add(n).ok_or(RomError("actor offset overflow"))?)
        .ok_or(RomError("actor data outside ROM"))
}
fn w(b: &[u8], p: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(data(b, p, 2)?.try_into().unwrap()))
}
fn l(b: &[u8], p: usize) -> Result<u32, RomError> {
    Ok(u32::from_be_bytes(data(b, p, 4)?.try_into().unwrap()))
}
#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub x: i16,
    pub y: i16,
    pub width: u8,
    pub height: u8,
    pub attribute: u16,
}
pub struct ActorFrame {
    pub address: usize,
    pub pieces: Vec<Piece>,
}
impl ActorFrame {
    /// Maps relative tile indices to caller-selected VRAM space, as $6288 does.
    pub fn load(
        rom: &[u8],
        address: usize,
        scene: &mut MapScene,
        tile_base: u16,
        palette: u16,
    ) -> Result<Self, RomError> {
        let mut p = address;
        let dynamic = w(rom, p)? == 0x8002;
        if dynamic {
            let source = l(rom, p + 2)? as usize;
            let size = w(rom, p + 6)? as usize;
            scene.transfer(rom, source, tile_base as usize * 32, size)?;
            p += 8;
        }
        let mut pieces = Vec::new();
        for _ in 0..80 {
            let x = w(rom, p)?;
            if x == 0x8000 {
                return Ok(Self { address, pieces });
            }
            let y = w(rom, p + 2)?;
            let size = w(rom, p + 4)? as u8;
            let mut attr = w(rom, p + 6)? | palette;
            if dynamic {
                attr = attr.wrapping_add(tile_base);
            }
            pieces.push(Piece {
                x: x as i16,
                y: y as i16,
                width: ((size >> 2) & 3) + 1,
                height: (size & 3) + 1,
                attribute: attr,
            });
            p += 8;
        }
        Err(RomError(
            "sprite frame exceeds 80 pieces or lacks terminator",
        ))
    }
    pub fn pixel(
        &self,
        scene: &MapScene,
        x: i32,
        y: i32,
        anchor_x: i32,
        anchor_y: i32,
    ) -> Result<Option<(u16, u8)>, RomError> {
        // First nontransparent sprite wins overlap, regardless of its priority bit.
        for piece in &self.pieces {
            let mut px = x - anchor_x - i32::from(piece.x);
            let mut py = y - anchor_y - i32::from(piece.y);
            let width = i32::from(piece.width) * 8;
            let height = i32::from(piece.height) * 8;
            if px < 0 || py < 0 || px >= width || py >= height {
                continue;
            }
            if piece.attribute & 0x800 != 0 {
                px = width - 1 - px;
            }
            if piece.attribute & 0x1000 != 0 {
                py = height - 1 - py;
            }
            // Genesis sprites store tiles in columns, unlike background blocks.
            let tile = (piece.attribute & 0x7ff)
                .wrapping_add((px / 8 * piece.height as i32 + py / 8) as u16);
            if tile > 0x7ff {
                return Err(RomError("sprite tile span exceeds VRAM"));
            }
            let index = scene.tile_pixel(tile, px as usize & 7, py as usize & 7)?;
            if index != 0 {
                return Ok(Some((piece.attribute, index)));
            }
        }
        Ok(None)
    }
}
pub struct Sequence {
    pub bank: usize,
    pub address: usize,
    pub terminal: bool,
    pub speed: u16,
    pub frames: Vec<usize>,
}
impl Sequence {
    /// Sixteen animation banks at $62ac; table boundaries establish state counts.
    /// IDs remain numeric until their corresponding entity definitions are traced.
    pub fn increment(&self, entity_speed: u16, frame: u16) -> u16 {
        if self.terminal {
            self.speed
        } else if self.bank == 13 {
            if frame == 2 {
                0x90
            } else {
                0x180
            }
        } else {
            entity_speed.wrapping_mul(if entity_speed < 22 { 28 } else { 16 })
        }
    }
    pub fn read(rom: &[u8], bank: usize, state: usize) -> Result<Self, RomError> {
        if bank >= 16 || state >= state_count(bank)? {
            return Err(RomError("actor bank/state outside recovered tables"));
        }
        let table = l(rom, 0x62ac + bank * 4)? as usize;
        let address = l(rom, table + state * 4)? as usize;
        let header = w(rom, address)?;
        let count = (header & 0x7fff) as usize;
        if count == 0 || count > 256 {
            return Err(RomError("invalid animation frame count"));
        }
        let mut frames = Vec::with_capacity(count);
        for i in 0..count {
            frames.push(l(rom, address + 4 + i * 4)? as usize);
        }
        Ok(Self {
            bank,
            address,
            terminal: header & 0x8000 != 0,
            speed: w(rom, address + 2)?,
            frames,
        })
    }
}
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationClock {
    pub phase: u16,
    pub frame: u16,
    pub ended: bool,
}
impl AnimationClock {
    /// $5fdc increments at most once per update, then $600e clamps or loops.
    /// Period and increment are entity state, not an inferred fixed frame rate.
    pub fn tick(
        &mut self,
        period: u16,
        increment: u16,
        count: u16,
        terminal: bool,
    ) -> Result<(), RomError> {
        if period == 0 || period > i16::MAX as u16 || count == 0 || count > i16::MAX as u16 {
            return Err(RomError("invalid animation clock parameters"));
        }
        let sum = self.phase.wrapping_add(increment);
        self.phase = sum;
        if (sum as i16) >= (period as i16) {
            self.frame = self.frame.wrapping_add(1);
            self.phase = sum.wrapping_sub(period);
        }
        if self.frame as i16 >= count as i16 {
            if terminal {
                self.frame = count - 1;
                self.ended = true;
            } else {
                self.frame = 0;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clock_loops_and_terminal_clamps() {
        let mut c = AnimationClock::default();
        for _ in 0..7 {
            c.tick(100, 50, 3, false).unwrap();
        }
        assert_eq!((c.frame, c.phase), (0, 50));
        for _ in 0..7 {
            c.tick(100, 50, 3, true).unwrap();
        }
        assert_eq!(c.frame, 2);
        assert!(c.ended);
    }
    #[test]
    fn original_clock_advances_only_once_per_tick() {
        let mut c = AnimationClock::default();
        c.tick(100, 350, 10, false).unwrap();
        assert_eq!((c.frame, c.phase), (1, 250));
    }
    #[test]
    fn invalid_clock_rejected() {
        assert!(AnimationClock::default().tick(0, 1, 1, false).is_err());
    }
}

/// Banks 0/1/2 hold 16/6/5 states; banks 3..15 each hold 66.
pub fn state_count(bank: usize) -> Result<usize, RomError> {
    match bank {
        0 => Ok(16),
        1 => Ok(6),
        2 => Ok(5),
        3..=15 => Ok(66),
        _ => Err(RomError("bank must be below 16")),
    }
}

#[cfg(test)]
mod sprite_tests {
    use super::*;
    fn fixture() -> MapScene {
        let mut vram = vec![0; 65536];
        for tile in 1..=6 {
            vram[tile * 32..tile * 32 + 32].fill((tile * 17) as u8);
        }
        MapScene::sprite_fixture(vram)
    }
    #[test]
    fn multi_tile_sprite_order_and_whole_piece_flip() {
        let scene = fixture();
        let mut frame = ActorFrame {
            address: 0,
            pieces: vec![Piece {
                x: 0,
                y: 0,
                width: 2,
                height: 3,
                attribute: 1,
            }],
        };
        for (x, y, index) in [(0, 0, 1), (0, 8, 2), (0, 16, 3), (8, 0, 4), (8, 16, 6)] {
            assert_eq!(frame.pixel(&scene, x, y, 0, 0).unwrap().unwrap().1, index);
        }
        frame.pieces[0].attribute |= 0x1800;
        assert_eq!(frame.pixel(&scene, 0, 0, 0, 0).unwrap().unwrap().1, 6);
        assert_eq!(frame.pixel(&scene, 15, 23, 0, 0).unwrap().unwrap().1, 1);
        assert!(frame.pixel(&scene, -1, 0, 0, 0).unwrap().is_none());
    }
    #[test]
    fn earlier_nontransparent_sprite_wins_and_offsets_are_signed() {
        let scene = fixture();
        let frame = ActorFrame {
            address: 0,
            pieces: vec![
                Piece {
                    x: -4,
                    y: -8,
                    width: 1,
                    height: 1,
                    attribute: 1,
                },
                Piece {
                    x: -4,
                    y: -8,
                    width: 1,
                    height: 1,
                    attribute: 2 | 0x8000,
                },
            ],
        };
        assert_eq!(frame.pixel(&scene, 6, 2, 10, 10).unwrap().unwrap().1, 1);
    }
}

/// $5a56 projects 1/16-pixel world coordinates to the Genesis sprite origin.
/// Coordinates wrap as words before ASR; the last shifted bit rounds the result.
/// Returns screen pixel coordinates (without the hardware's 128 bias) if visible.
pub fn project(
    world: (u16, u16),
    camera: (u16, u16),
    height_offset: i16,
    wide_culling: bool,
) -> Option<(i32, i32)> {
    fn axis(world: u16, camera: u16) -> u16 {
        let delta = world.wrapping_sub(camera);
        let mut pixel = ((delta as i16) >> 4) as u16;
        if delta & 8 != 0 {
            pixel = pixel.wrapping_add(1);
        }
        pixel.wrapping_add(128)
    }
    let x = axis(world.0, camera.0);
    let y = axis(world.1, camera.1).wrapping_add(height_offset as u16);
    let visible = if wide_culling {
        x < 512 && y < 416
    } else {
        (112..=429).contains(&x) && (99..378).contains(&y)
    };
    visible.then_some((i32::from(x) - 128, i32::from(y) - 128))
}
#[cfg(test)]
mod projection_tests {
    use super::*;
    #[test]
    fn signed_rounding_and_height_offset() {
        assert_eq!(project((7, 8), (0, 0), -1, false), Some((0, 0)));
        assert_eq!(project((0, 0), (8, 9), 0, false), Some((0, -1)));
        assert_eq!(
            project((160 * 16, 184 * 16), (0, 0), 0, false),
            Some((160, 184))
        );
    }
    #[test]
    fn culling_matches_biased_sprite_bounds() {
        assert_eq!(project((0, 0), (17 * 16, 0), 0, false), None);
        assert!(project((0, 0), (16 * 16, 0), 0, false).is_some());
        assert!(project((0, 0), (17 * 16, 0), 0, true).is_some());
    }
}
