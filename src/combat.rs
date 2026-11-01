//! Target-list construction/retention from $13874/$1438c..$144e6 and the
//! close-range probe at $15c58, and the $14d8e obstruction probe. Weapon actions,
//! hit rolls and damage are separate.
use crate::{avoidance::State, collision::Collision, followers, party::Runner, RomError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineOutcome {
    TargetReached,
    Blocked,
    TraceLimit,
    OutsideModel,
}
pub struct LineProbe {
    pub outcome: LineOutcome,
    pub steps: u16,
    pub projectile: [u8; 256],
    pub samples: [u8; 4],
}
fn table_word(rom: &[u8], at: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(
        rom.get(at..at + 2)
            .ok_or(RomError("projectile table outside ROM"))?
            .try_into()
            .unwrap(),
    ))
}
fn set_word(raw: &mut [u8; 256], at: usize, value: u16) {
    raw[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

/// Point-query variant of $e7c8, used by the flag-bit-2-clear probe particle.
/// Other corner bytes persist in the original shared collision scratch; keep
/// them explicit instead of silently substituting a full actor footprint.
fn point_blocks(c: &Collision, p: (u16, u16), samples: &mut [u8; 4]) -> Option<bool> {
    let (x, y) = (p.0 >> 4, p.1 >> 4);
    if x < c.bounds[0] || y < c.bounds[1] || x >= c.bounds[2] || y >= c.bounds[3] {
        *samples = [0, 0x7f, 0, 0x7f];
        return Some(true);
    }
    let index = (y.wrapping_mul(2) & 0xffc0).wrapping_add(x >> 5);
    let code = *c.cells.get(index as usize)?;
    samples[0] = code;
    if code == 0 {
        return Some(false);
    }
    if code == 0x7f || code >= 0x80 {
        return Some(true);
    }
    // $ffdfe2 is set for positive bit-6 codes after the $7f early return.
    // $14d8e lets these through even if the collision result itself is blocked.
    if code & 0x40 != 0 {
        return Some(false);
    }
    if code & 0x20 == 0 {
        let r = c.rectangles.get(code as usize)?;
        return Some(
            (x as i16) >= r.left.wrapping_sub(12) as i16
                && (x as i16) <= r.right.wrapping_add(12) as i16
                && (y as i16) >= r.top.wrapping_sub(7) as i16
                && (y as i16) <= r.bottom.wrapping_add(7) as i16,
        );
    }
    let ox = (index & 63) << 5;
    let oy = (index & 0xffc0) >> 1;
    for (i, (dx, dy)) in [(-12i16, -7i16), (12, -7), (-12, 7), (12, 7)]
        .iter()
        .enumerate()
    {
        if samples[i] & 0x20 == 0 {
            continue;
        }
        let px = x.wrapping_add(*dx as u16);
        let py = y.wrapping_add(*dy as u16);
        let dx = px.wrapping_sub(ox);
        let dy = py.wrapping_sub(oy);
        let inverse = oy.wrapping_add(31).wrapping_sub(py);
        let blocked = match samples[i] & 0x38 {
            0x20 => dx < inverse,
            0x28 => dx >= dy,
            0x38 => dx >= inverse,
            _ => dx < dy,
        };
        if blocked {
            return Some(true);
        }
    }
    Some(false)
}
/// $14d8e/$14ff8: initialize an independent probe and run at most 27 steps.
/// Initial collision scratch is explicit. The UI supplies cleared scratch for
/// its deterministic, read-only query; a future firing dispatcher must supply
/// its actual shared scratch. Exhausting the trace does not prove a target hit.
pub fn line_of_fire(
    rom: &[u8],
    c: &Collision,
    source: &Runner,
    target: &Runner,
    mut samples: [u8; 4],
) -> Result<LineProbe, RomError> {
    let source_id = source.word(0x3e);
    if source_id >= 10 {
        return Err(RomError("projectile source slot outside actor pool"));
    }
    let angle = if target.raw[0x10] & 1 != 0 {
        followers::heading(rom, source.position(), target.position())?.0
    } else {
        source.word(0x2e)
    };
    let mut rounded = angle.rotate_left(3);
    if (rounded as i16) < 0 {
        rounded = rounded.wrapping_add(1);
    }
    let octant = usize::from(rounded & 7);
    let origin = (
        source
            .position()
            .0
            .wrapping_add(table_word(rom, 0x15a4c + octant * 4)?),
        source
            .position()
            .1
            .wrapping_add(table_word(rom, 0x15a4e + octant * 4)?),
    );
    let index = usize::from((angle >> 8) & 0xfe);
    let vx = ((table_word(rom, 0x15e56 + index)? as i16 as i32 * 12) >> 4) as i16;
    let vy = ((table_word(rom, 0x15e16 + index)? as i16 as i32 * 12) >> 4) as i16;
    let mut raw = [0u8; 256];
    for (at, value) in [
        (8, 0xfff4),
        (0xe, 1),
        (0x16, 0x14),
        (0x1c, 0x1000),
        (0x2c, 0x4000),
        (0x2e, angle),
        (0x3e, 0xffff),
        (0x4c, vx as u16),
        (0x4e, vy as u16),
    ] {
        set_word(&mut raw, at, value);
    }
    raw[0x10] = 1 | (source.raw[0x10] & 8);
    raw[0x3a..0x3e].copy_from_slice(&0x14e0au32.to_be_bytes());
    raw[0x60..0x64].copy_from_slice(&(0xff0100u32 + u32::from(source_id) * 256).to_be_bytes());
    for at in [0x56, 0x57, 0x9d] {
        raw[at] = source.raw[at];
    }
    let mut x = u32::from(origin.0) << 16;
    let mut y = u32::from(origin.1) << 16;
    let mut outcome = LineOutcome::TraceLimit;
    let mut steps = 0;
    for _ in 0..27 {
        steps += 1;
        x = x.wrapping_add((i32::from(vx) << 8) as u32);
        y = y.wrapping_add((i32::from(vy) << 8) as u32);
        raw[..4].copy_from_slice(&x.to_be_bytes());
        raw[4..8].copy_from_slice(&y.to_be_bytes());
        let p = ((x >> 16) as u16, (y >> 16) as u16);
        match point_blocks(c, p, &mut samples) {
            None => {
                outcome = LineOutcome::OutsideModel;
                break;
            }
            Some(true) => {
                outcome = LineOutcome::Blocked;
                break;
            }
            Some(false) => {}
        }
        let (tx, ty) = target.position();
        if target.raw[0x10] & 0x51 == 1
            && target.raw[0x4c] != 0
            && p.0 >= tx.wrapping_sub(0x90)
            && p.0 < tx.wrapping_add(0x90)
            && p.1 >= ty.wrapping_sub(0x90)
            && p.1 < ty.wrapping_add(0x90)
        {
            outcome = LineOutcome::TargetReached;
            break;
        }
    }
    Ok(LineProbe {
        outcome,
        steps,
        projectile: raw,
        samples,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorId(pub u8);
impl ActorId {
    pub fn pointer(self) -> u32 {
        0xff0100 + u32::from(self.0) * 256
    }
    pub fn from_pointer(pointer: u32) -> Option<Self> {
        let offset = pointer.checked_sub(0xff0100)?;
        if offset < 0xa00 && offset & 255 == 0 {
            Some(Self((offset >> 8) as u8))
        } else {
            None
        }
    }
}
pub fn target(r: &Runner) -> Option<ActorId> {
    ActorId::from_pointer(u32::from_be_bytes(r.raw[0xa2..0xa6].try_into().unwrap()))
}
pub fn clear_target(r: &mut Runner) {
    r.raw[0xa2..0xa6].fill(0);
}

#[derive(Default)]
pub struct TargetLists {
    pub npcs: Vec<ActorId>,
    pub party: Vec<ActorId>,
}
fn eligible(r: &Runner, npc: bool) -> bool {
    r.raw[0x10] & 0x51 == 1
        && r.word(0xc) != 0
        && (!npc || (r.raw[0xb0] & 0x80 == 0 && r.raw[0x9e] & 0x28 == 0))
}
impl TargetLists {
    /// Original list refresh: NPCs on phase 11, party on phase 15, modulo 64;
    /// a forced refresh rebuilds both. These lists are not hostility verdicts.
    pub fn refresh(&mut self, actors: &[&Runner], phase: u16, force: bool) {
        if force || phase & 63 == 11 {
            self.npcs = (3..10)
                .filter_map(|slot| {
                    actors
                        .iter()
                        .find(|r| r.word(0x3e) == slot)
                        .filter(|r| eligible(r, true))
                        .map(|_| ActorId(slot as u8))
                })
                .collect();
        }
        if force || phase & 63 == 15 {
            self.party = (0..3)
                .filter_map(|slot| {
                    actors
                        .iter()
                        .find(|r| r.word(0x3e) == slot)
                        .filter(|r| eligible(r, false))
                        .map(|_| ActorId(slot as u8))
                })
                .collect();
        }
    }
    /// $143f6/$14492 preserves a listed target and consumes randomness only to
    /// replace an invalid target. The NPC-side selector marks a changed target.
    /// Returns the original empty-NPC-list end-combat callback request; callers
    /// must not confuse it with implemented combat shutdown or mission effects.
    pub fn acquire(&self, r: &mut Runner, random: &mut State) -> bool {
        let party_actor = r.raw[0x10] & 8 != 0;
        let list = if party_actor { &self.npcs } else { &self.party };
        if list.is_empty() {
            clear_target(r);
            return party_actor;
        }
        if target(r).is_some_and(|id| list.contains(&id)) {
            return false;
        }
        let id = list[usize::from(random.sample(list.len() as u16))];
        r.raw[0xa2..0xa6].copy_from_slice(&id.pointer().to_be_bytes());
        if !party_actor {
            r.raw[0xb5] = 6;
        }
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CloseRange {
    pub heading: Option<u16>,
    pub range: Option<u16>,
    pub in_reach: bool,
}
/// Active, non-dead target plus a strict $200 fixed-point range test. This does
/// not establish line of sight, weapon readiness, hit chance or permission.
pub fn close_range(rom: &[u8], source: &Runner, target: &Runner) -> Result<CloseRange, RomError> {
    if target.raw[0x10] & 0x41 != 1 {
        return Ok(CloseRange {
            heading: None,
            range: None,
            in_reach: false,
        });
    }
    let (heading, range) = followers::heading(rom, source.position(), target.position())?;
    Ok(CloseRange {
        heading: Some(heading),
        range: Some(range),
        in_reach: range < 0x200,
    })
}

#[derive(Default)]
pub struct Targeting {
    pub enabled: bool,
    pub force_refresh: bool,
    pub lists: TargetLists,
}
impl Targeting {
    pub fn invalidate(&mut self, party: &mut [Runner]) {
        for r in party {
            clear_target(r);
        }
        self.lists = TargetLists::default();
        self.force_refresh = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_point_geometry_is_guarded_instead_of_assumed_clear() {
        let mut c = Collision {
            cells: vec![0; 4096],
            rectangles: vec![],
            bounds: [0, 0, 512, 512],
            map_width_tiles: 32,
            overflow_writes: vec![],
        };
        c.cells[7 * 64 + 7] = 1;
        assert_eq!(point_blocks(&c, (240 * 16, 240 * 16), &mut [0; 4]), None);
        c.cells[7 * 64 + 7] = 0x7e;
        assert_eq!(
            point_blocks(&c, (240 * 16, 240 * 16), &mut [0; 4]),
            Some(false)
        );
        c.cells[7 * 64 + 7] = 0x7f;
        assert_eq!(
            point_blocks(&c, (240 * 16, 240 * 16), &mut [0; 4]),
            Some(true)
        );
    }
    #[test]
    fn actor_pointers_are_aligned_and_bounded() {
        assert_eq!(ActorId::from_pointer(0xff0100), Some(ActorId(0)));
        assert_eq!(ActorId::from_pointer(0xff0a00), Some(ActorId(9)));
        for pointer in [0, 0xff0101, 0xff0000, 0xff0b00, 0x12345678] {
            assert_eq!(ActorId::from_pointer(pointer), None);
        }
    }
    #[test]
    fn visible_neutral_candidate_is_not_a_hostility_or_health_verdict() {
        let mut actors: Vec<_> = (3..=10)
            .map(|slot| {
                let mut r = Runner {
                    raw: [0; 256],
                    template: 0,
                    clock: Default::default(),
                };
                r.set_word(0x3e, slot);
                r.set_word(0xc, 128);
                r.raw[0x10] = 5;
                r
            })
            .collect();
        actors[0].raw[0x9e] = 0x10; // talkable NPC, no hostility classification
        actors[1].raw[0xb0] = 0x80;
        actors[2].raw[0x9e] = 0x20;
        actors[3].raw[0x9e] = 8;
        actors[4].raw[0x10] = 0x11;
        actors[5].set_word(0xc, 0);
        actors[6].raw[0x10] = 0x45;
        let mut lists = TargetLists::default();
        lists.refresh(&actors.iter().collect::<Vec<_>>(), 0, true);
        assert_eq!(lists.npcs, vec![ActorId(3)]);
        assert_eq!(
            actors[0].raw[0x4c], 0,
            "this list does not infer death from health alone"
        );
    }
}
