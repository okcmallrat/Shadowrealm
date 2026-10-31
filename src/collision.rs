//! Static collision raster and queries from $ee10/$e7c8. Transition codes are
//! returned to the caller; their story/door scripts are not silently executed.
use crate::{maps::MapDescriptor, RomError};
fn w(b: &[u8], p: usize) -> Result<u16, RomError> {
    let a = b
        .get(p..p + 2)
        .ok_or(RomError("collision data outside ROM"))?;
    Ok(u16::from_be_bytes([a[0], a[1]]))
}
#[derive(Clone, Debug)]
pub struct Rect {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
    pub code: u8,
}
pub struct Collision {
    pub cells: Vec<u8>,
    pub rectangles: Vec<Rect>,
    pub bounds: [u16; 4],
    pub overflow_writes: Vec<(i16, u8)>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contact {
    Clear,
    Blocked,
    Trigger(u8),
}
impl Collision {
    pub fn footprint_samples(&self, world: (u16, u16)) -> [u8; 4] {
        let (x, y) = (world.0 >> 4, world.1 >> 4);
        [(-12i16, -7i16), (12, -7), (-12, 7), (12, 7)].map(|(dx, dy)| {
            let px = x.wrapping_add(dx as u16);
            let py = y.wrapping_add(dy as u16);
            let offset = ((py.wrapping_mul(2) & 0xffc0).wrapping_add(px >> 5)) as usize;
            self.cells.get(offset).copied().unwrap_or(0x7f)
        })
    }
    pub fn load(rom: &[u8], map: &MapDescriptor) -> Result<Self, RomError> {
        let mut p = u32::from_be_bytes(map.raw[0x26..0x2a].try_into().unwrap()) as usize;
        let mut cells = vec![0; 4096];
        let mut rectangles = Vec::new();
        let mut overflow_writes = Vec::new();
        loop {
            let left = w(rom, p)?;
            if left & 0x8000 != 0 {
                break;
            }
            if rectangles.len() >= 4096 {
                return Err(RomError("unterminated collision rectangles"));
            }
            let r = Rect {
                left,
                right: w(rom, p + 2)?,
                top: w(rom, p + 4)?,
                bottom: w(rom, p + 6)?,
                code: w(rom, p + 8)? as u8,
            };
            p += 10;
            // Original increments coordinates by 32 from the exact left/top,
            // while writing consecutive grid cells (not rounded max corners).
            let columns = if r.right >= r.left {
                (r.right - r.left) / 32 + 1
            } else {
                1
            };
            let rows = if r.bottom >= r.top {
                (r.bottom - r.top) / 32 + 1
            } else {
                1
            };
            let start = ((r.top & 0xffe0) as usize) * 2 + (r.left >> 5) as usize;
            for y in 0..rows as usize {
                for x in 0..columns as usize {
                    let offset = (start + y * 64 + x) as u16 as i16;
                    if (0..4096).contains(&offset) {
                        cells[offset as usize] = r.code;
                    } else {
                        overflow_writes.push((offset, r.code));
                    }
                }
            }
            rectangles.push(r);
        }
        Ok(Self {
            cells,
            rectangles,
            bounds: [
                w(&map.raw, 0x10)?,
                w(&map.raw, 0x12)?,
                w(&map.raw, 0xc)?,
                w(&map.raw, 0xe)?,
            ],
            overflow_writes,
        })
    }
    /// $b7fa requires at least two signed-negative corner samples before invoking
    /// transition handling. A single maximum trigger byte is not sufficient.
    pub fn trigger_corners(&self, world: (u16, u16)) -> usize {
        let (x, y) = (world.0 >> 4, world.1 >> 4);
        [(-12i16, -7i16), (12, -7), (-12, 7), (12, 7)]
            .iter()
            .filter(|(dx, dy)| {
                let px = x.wrapping_add(*dx as u16);
                let py = y.wrapping_add(*dy as u16);
                let offset = (((py.wrapping_mul(2)) & 0xffc0).wrapping_add(px >> 5)) as usize;
                self.cells.get(offset).is_some_and(|code| *code & 0x80 != 0)
            })
            .count()
    }
    pub fn contact(&self, world: (u16, u16)) -> Result<Contact, RomError> {
        let x = world.0 >> 4;
        let y = world.1 >> 4;
        if x < 12
            || y < 7
            || x < self.bounds[0]
            || y < self.bounds[1]
            || x >= self.bounds[2]
            || y >= self.bounds[3]
        {
            return Ok(Contact::Blocked);
        }
        let points = [
            (x.wrapping_sub(12), y.wrapping_sub(7)),
            (x.wrapping_add(12), y.wrapping_sub(7)),
            (x.wrapping_sub(12), y.wrapping_add(7)),
            (x.wrapping_add(12), y.wrapping_add(7)),
        ];
        let mut samples = [0; 4];
        let mut maximum = 0;
        let mut index = 0;
        let mut hard = false;
        for (i, (px, py)) in points.iter().enumerate() {
            let offset = (((py.wrapping_mul(2)) & 0xffc0).wrapping_add(px >> 5)) as usize;
            let Some(&code) = self.cells.get(offset) else {
                return Ok(Contact::Blocked);
            };
            samples[i] = code;
            if code >= maximum {
                maximum = code;
                index = offset;
            }
            if code == 0x7f {
                hard = true;
            }
        }
        if hard || maximum == 0x7f {
            return Ok(Contact::Blocked);
        }
        if maximum == 0 {
            return Ok(Contact::Clear);
        }
        if maximum >= 0x80 {
            return Ok(Contact::Trigger(maximum));
        }
        if maximum & 0x40 != 0 && (maximum == 0x7e || maximum & 0x20 == 0) {
            return Ok(Contact::Blocked);
        }
        if maximum & 0x20 == 0 {
            let r = self
                .rectangles
                .get(maximum as usize)
                .ok_or(RomError("collision rectangle index outside table"))?;
            let inside = (x as i16) >= (r.left.wrapping_sub(12) as i16)
                && (x as i16) <= (r.right.wrapping_add(12) as i16)
                && (y as i16) >= (r.top.wrapping_sub(7) as i16)
                && (y as i16) <= (r.bottom.wrapping_add(7) as i16);
            return Ok(if inside {
                Contact::Blocked
            } else {
                Contact::Clear
            });
        }
        let ox = ((index as u16) & 63) << 5;
        let oy = ((index as u16) & 0xffc0) >> 1;
        for (i, (px, py)) in points.iter().enumerate() {
            if samples[i] & 0x20 == 0 {
                continue;
            }
            let dx = px.wrapping_sub(ox);
            let dy = py.wrapping_sub(oy);
            let inv = oy.wrapping_add(31).wrapping_sub(*py);
            let blocked = match samples[i] & 0x38 {
                0x20 => dx < inv,
                0x28 => dx >= dy,
                0x38 => dx >= inv,
                _ => dx < dy,
            };
            if blocked {
                return Ok(Contact::Blocked);
            }
        }
        Ok(Contact::Clear)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transition_requires_two_negative_footprint_corners() {
        let mut c = Collision {
            cells: vec![0; 4096],
            rectangles: vec![],
            bounds: [12, 7, 200, 200],
            overflow_writes: vec![],
        };
        c.cells[0] = 0xb0;
        assert_eq!(c.trigger_corners((32 * 16, 32 * 16)), 1);
        c.cells[1] = 0xf0;
        assert_eq!(c.trigger_corners((32 * 16, 32 * 16)), 2);
    }
    #[test]
    fn hard_collision_overrides_trigger_and_bounds() {
        let mut c = Collision {
            cells: vec![0; 4096],
            rectangles: vec![],
            bounds: [12, 7, 200, 200],
            overflow_writes: vec![],
        };
        assert_eq!(c.contact((100 * 16, 100 * 16)).unwrap(), Contact::Clear);
        c.cells[3 * 64 + 3] = 0xc0;
        assert_eq!(
            c.contact((100 * 16, 100 * 16)).unwrap(),
            Contact::Trigger(0xc0)
        );
        c.cells[3 * 64 + 2] = 0x7f;
        assert_eq!(c.contact((100 * 16, 100 * 16)).unwrap(), Contact::Blocked);
        assert_eq!(c.contact((0, 0)).unwrap(), Contact::Blocked);
    }
}
