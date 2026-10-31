//! Noncombat party following recovered with the supplied Ghidra reference and
//! checked against the ROM routines. Combat, dynamic actor avoidance and the
//! original fallback wall-hugging search are outside this bounded dispatcher.
use crate::{
    collision::{Collision, Contact},
    party::Runner,
    RomError,
};
fn word(rom: &[u8], p: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(
        rom.get(p..p + 2)
            .ok_or(RomError("follower table outside ROM"))?
            .try_into()
            .unwrap(),
    ))
}
fn behavior(r: &Runner) -> u32 {
    u32::from_be_bytes(r.raw[0x3a..0x3e].try_into().unwrap())
}
fn set_behavior(r: &mut Runner, value: u32) {
    r.raw[0x3a..0x3e].copy_from_slice(&value.to_be_bytes());
}
fn abs_word(value: u16) -> u16 {
    if (value as i16) < 0 {
        value.wrapping_neg()
    } else {
        value
    }
}
pub fn distance(a: (u16, u16), b: (u16, u16)) -> u16 {
    let x = abs_word(a.0.wrapping_sub(b.0));
    let y = abs_word(a.1.wrapping_sub(b.1));
    x.max(y).wrapping_add((x.min(y) >> 1) + (x.min(y) & 1)) // $15ca4 rounds the half up
}
/// $15d3c/$15d4e: table-quantized heading and its floor-half distance estimate.
pub fn heading(rom: &[u8], from: (u16, u16), to: (u16, u16)) -> Result<(u16, u16), RomError> {
    let dy = to.1.wrapping_sub(from.1);
    let dx = to.0.wrapping_sub(from.0);
    let (ax, ay) = (abs_word(dx), abs_word(dy));
    let mut major = ax.max(ay);
    let mut minor = ax.min(ay);
    let estimate = major.wrapping_add(minor >> 1);
    while major >= 32 {
        major >>= 1;
        minor >>= 1;
    }
    let (mut a, mut b) = if ax < ay {
        (major, minor)
    } else {
        (minor, major)
    };
    let signs = (if (dy as i16) < 0 {
        8
    } else if dy == 0 {
        4
    } else {
        0
    }) | (if (dx as i16) < 0 {
        2
    } else if dx == 0 {
        1
    } else {
        0
    });
    let base = word(rom, 0x15df6 + signs * 2)?;
    if base & 0x40 == 0 {
        std::mem::swap(&mut a, &mut b);
    }
    let offset = *rom
        .get(0x15f96 + usize::from(a) * 32 + usize::from(b))
        .ok_or(RomError("heading lookup outside ROM"))?;
    Ok((((base as u8).wrapping_add(offset) as u16) << 8, estimate))
}
/// $15dcc followed by EXG at $14540; signed products truncate via their high word.
pub fn formation_offset(rom: &[u8], angle: u16) -> Result<(u16, u16), RomError> {
    let index = usize::from((angle >> 8) & 0xfe);
    let component = |p| -> Result<u16, RomError> {
        let coefficient = word(rom, p)? as i16 as i32;
        Ok(((coefficient * 224 * 2) >> 16) as u16)
    };
    Ok((
        component(0x15e16 + index + 0x40)?,
        component(0x15e16 + index)?,
    ))
}
pub fn formation_goal(
    rom: &[u8],
    collision: &Collision,
    runner: &Runner,
    leader: &Runner,
) -> Result<(u16, u16), RomError> {
    let left = runner.word(0x3e) == 0 || (runner.word(0x3e) == 1 && leader.word(0x3e) == 0);
    let angle = leader
        .word(0x2e)
        .wrapping_add(if left { 0xa000 } else { 0x6000 });
    let delta = formation_offset(rom, angle)?;
    let p = leader.position();
    for sign in [1i16, -1] {
        let candidate = (
            p.0.wrapping_add(delta.0.wrapping_mul(sign as u16)),
            p.1.wrapping_add(delta.1.wrapping_mul(sign as u16)),
        );
        if !matches!(collision.contact(candidate)?, Contact::Blocked) {
            return Ok(candidate);
        }
    }
    Ok(p)
}
fn idle(r: &mut Runner, snap: bool) {
    if snap {
        r.raw[0xb0] &= !2;
        r.set_word(0x2e, r.word(0x34));
        r.raw[0x11] |= 1;
    }
    set_behavior(r, 0xb45c);
    r.set_word(0x38, 0);
    r.select_motion(false);
}
fn cell(p: (u16, u16)) -> u16 {
    ((p.1 >> 4).wrapping_mul(2) & 0xffc0).wrapping_add(p.0 >> 9)
}
fn cell_open(c: &Collision, index: u16) -> bool {
    c.cells.get(index as usize).is_some_and(|v| *v == 0)
}
fn grid_step(direction: u16) -> u16 {
    match direction & 6 {
        0 => 0xffc0,
        2 => 1,
        4 => 64,
        _ => 0xffff,
    }
}
fn route_signs(r: &mut Runner) {
    r.raw[0xb1] = 0;
    if r.word(0x96) >= r.position().0 {
        r.raw[0xb1] |= 1;
    }
    if r.word(0x98) >= r.position().1 {
        r.raw[0xb1] |= 2;
    }
}
fn clear_route(r: &mut Runner) {
    r.raw[0xb0] &= !(2 | 8);
}
/// $14682 regular one/two-corner route. The more complex $1470e fallback
/// conservatively stops here; no invented A*/teleport route replaces it.
fn plan_route(r: &mut Runner, c: &Collision) -> bool {
    r.raw[0xb0] |= 2;
    if r.raw[0xb0] & 8 != 0 {
        return false;
    }
    let p = r.position();
    let goal = (r.word(0x96), r.word(0x98));
    let dx = p.0.wrapping_sub(goal.0);
    let dy = p.1.wrapping_sub(goal.1);
    let mut side = if (dy as i16) < 0 { 4 } else { 0 };
    let mut forward = if (dx as i16) < 0 { 2 } else { 6 };
    if abs_word(dx) < abs_word(dy) {
        std::mem::swap(&mut side, &mut forward);
    }
    let start = cell(p);
    if !cell_open(c, start) {
        clear_route(r);
        return true; // $14756 clears the route, then caller resumes direct steering
    }
    r.set_word(0xba, 0);
    let advance = grid_step(forward);
    // $148aa scans to the end of the clear strip before returning its step.
    // Preserve that eligibility check; do not follow scans into adjacent RAM.
    let mut end = start;
    for _ in 0..4096 {
        let next = end.wrapping_add(advance);
        let Some(value) = c.cells.get(next as usize) else {
            clear_route(r);
            return false;
        };
        if *value != 0 {
            break;
        }
        end = next;
    }
    if end != start {
        r.set_word(0xbc, start.wrapping_add(advance));
        r.set_word(0xbe, 0xffff);
        return true;
    }
    for direction in [side, side ^ 4] {
        let step = grid_step(direction);
        let mut at = start;
        for _ in 0..4096 {
            at = at.wrapping_add(step);
            if c.cells.get(at as usize).is_none() {
                clear_route(r);
                return false;
            }
            if !cell_open(c, at) {
                break;
            }
            let next = at.wrapping_add(advance);
            if c.cells.get(next as usize).is_none() {
                clear_route(r);
                return false;
            }
            if cell_open(c, next) {
                r.set_word(0xbc, at);
                r.set_word(0xbe, next);
                r.set_word(0xc0, 0xffff);
                return true;
            }
        }
    }
    clear_route(r);
    false
}
fn need_route(r: &mut Runner) -> bool {
    let dx = r.word(0x96).wrapping_sub(r.position().0) as i16;
    let dy = r.word(0x98).wrapping_sub(r.position().1) as i16;
    if (dx >= 0) == (r.raw[0xb1] & 1 == 0) {
        r.raw[0xb1] |= 4;
    }
    if (dy >= 0) == (r.raw[0xb1] & 2 == 0) {
        r.raw[0xb1] |= 8;
    }
    let blocked = r.word(0xb8);
    (blocked as i16) < 0
        || if r.raw[0xb1] & 4 == 0 {
            r.raw[0xb1] & 8 != 0 && blocked & 2 != 0
        } else {
            blocked & 2 == 0
        }
}
fn steer(r: &mut Runner, rom: &[u8], c: &Collision) -> Result<bool, RomError> {
    if (r.word(0x16) as i16) > 0x3c {
        return Ok(true);
    }
    if r.raw[0xb0] & 2 == 0 {
        let pending = r.raw[0xb0] & 4 != 0;
        r.raw[0xb0] &= !4;
        if pending {
            route_signs(r);
        } else if r.word(0xb8) != 0x1000 {
            if need_route(r) && !plan_route(r, c) {
                idle(r, false);
                return Ok(false);
            }
            if r.raw[0xb0] & 2 == 0 {
                route_signs(r);
            }
        }
    }
    let mut target = (r.word(0x96), r.word(0x98));
    let mut waypoint_heading = None;
    if r.raw[0xb0] & 2 != 0 {
        if r.raw[0xb0] & 8 != 0 {
            clear_route(r);
            idle(r, false);
            return Ok(false);
        }
        let mut offset = usize::from(r.word(0xba));
        if offset > 8 || offset & 1 != 0 {
            clear_route(r);
            idle(r, false);
            return Ok(false);
        }
        let mut next = r.word(0xbc + offset);
        if cell(r.position()) == next {
            if r.word(0xbe + offset) == 0xffff {
                clear_route(r);
                route_signs(r);
            } else {
                offset += 2;
                r.set_word(0xba, offset as u16);
                next = r.word(0xbc + offset);
            }
        }
        if r.raw[0xb0] & 2 != 0 {
            let here = cell(r.position());
            let direction = if here & 0xffc0 == next & 0xffc0 {
                if here & 63 < next & 63 {
                    2
                } else {
                    6
                }
            } else if here & 0xffc0 < next & 0xffc0 {
                4
            } else {
                0
            };
            waypoint_heading = Some((direction as u16).rotate_right(3));
            target = (
                ((next & 63) << 9).wrapping_add(0x100),
                ((next >> 6) << 9).wrapping_add(0x100),
            );
        }
    }
    let (angle, range) = if let Some(angle) = waypoint_heading {
        (angle, distance(r.position(), target))
    } else {
        heading(rom, r.position(), target)?
    };
    r.set_word(0x34, angle);
    r.set_word(0xc8, range);
    set_behavior(r, 0xb574);
    Ok(true)
}
/// $14508 + stop gate $144e8/$1460e. Returns false for a withheld fallback route.
pub fn decide(
    rom: &[u8],
    c: &Collision,
    r: &mut Runner,
    leader: &Runner,
) -> Result<bool, RomError> {
    let goal = formation_goal(rom, c, r, leader)?;
    r.set_word(0x96, goal.0);
    r.set_word(0x98, goal.1);
    if distance(r.position(), goal) < 0x1e0 {
        r.raw[0xb0] = (r.raw[0xb0] & 0x80) | 4;
        r.raw[0xb1] = 0;
        set_behavior(r, 0xb45c);
        return Ok(true);
    }
    steer(r, rom, c)
}
fn probe(
    c: &Collision,
    r: &mut Runner,
    rom: &[u8],
    table: usize,
    direction: usize,
) -> Result<((u16, u16), Contact), RomError> {
    let p = r.position();
    let candidate = (
        p.0.wrapping_add(word(rom, table + direction * 4)?),
        p.1.wrapping_add(word(rom, table + direction * 4 + 2)?),
    );
    r.raw[0xd0..0xd4].copy_from_slice(&c.footprint_samples(candidate));
    let contact = match c.contact(candidate)? {
        Contact::Trigger(_) => Contact::Blocked,
        x => x,
    };
    Ok((candidate, contact))
}
/// $b574/$b6e4/$bade: noncombat party step, including diagonal/cardinal sliding.
pub fn move_one(rom: &[u8], c: &Collision, r: &mut Runner) -> Result<(), RomError> {
    if behavior(r) != 0xb574 {
        let before = r.word(0x16);
        r.raw[0x11] &= !4;
        r.set_word(0x38, 0);
        r.select_motion(false);
        let samples = c.footprint_samples(r.position());
        r.raw[0xd0..0xd4].copy_from_slice(&samples);
        if before != r.word(0x16) {
            r.raw[0x11] |= 1;
        }
        return Ok(());
    }
    let before = r.word(0x16);
    r.raw[0x11] &= !4;
    r.set_word(0x38, 1);
    r.turn_toward(r.word(0x34));
    let range = r.word(0xc8);
    let table = if r.word(0xc) == 0 {
        0xc022
    } else if range >= 0x460 {
        0xbfe2
    } else if range >= 0x360 {
        0xbd72
    } else if range >= 0x200 {
        0xbd92
    } else if range >= 0x100 {
        0xbdb2
    } else {
        0xbdd2
    };
    r.set_word(0xca, word(rom, table + 4)?);
    let direction = usize::from(r.word(0x2e).rotate_left(3) & 7);
    let (mut candidate, contact) = probe(c, r, rom, table, direction)?;
    let mut moved = matches!(contact, Contact::Clear);
    if moved {
        r.set_word(0xb8, 0x1000);
    } else {
        let alternative = if r.word(0x2e) & 0x3fff != 0 {
            let first = (direction as u16) & 6;
            r.set_word(0xb8, first);
            let (p, result) = probe(c, r, rom, table, usize::from(first))?;
            if matches!(result, Contact::Clear) {
                Some((first, p))
            } else {
                let second = (first + 2) & 7;
                let (p, result) = probe(c, r, rom, table, usize::from(second))?;
                if matches!(result, Contact::Clear) {
                    Some((second, p))
                } else {
                    None
                }
            }
        } else {
            let samples = r.raw[0xd0..0xd4].to_vec();
            let (a, b, first, second) = match direction & 6 {
                0 => (0, 1, 6, 2),
                2 => (1, 3, 0, 4),
                4 => (2, 3, 6, 2),
                _ => (0, 2, 0, 4),
            };
            let clear = |v: u8| v == 0 || v >= 128;
            let slide = if clear(samples[a]) {
                Some(first)
            } else if clear(samples[b]) {
                Some(second)
            } else {
                None
            };
            if let Some(dir) = slide {
                let (p, result) = probe(c, r, rom, table, dir)?;
                if matches!(result, Contact::Clear) {
                    Some((dir as u16, p))
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some((dir, p)) = alternative {
            candidate = p;
            moved = true;
            r.set_word(0x36, dir.rotate_right(3));
            r.raw[0x11] |= 4;
        } else {
            r.set_word(0xb8, 0xffff);
            idle(r, true);
            let samples = c.footprint_samples(r.position());
            r.raw[0xd0..0xd4].copy_from_slice(&samples);
        }
    }
    if moved {
        r.raw[0xb0] &= !1;
        r.set_position(candidate.0, candidate.1);
        let facing = if r.raw[0x11] & 4 != 0 {
            r.word(0x36)
        } else {
            r.word(0x2e)
        };
        r.set_word(0x16, ((facing.rotate_left(3) & 7) + 8) * 4);
    }
    if before != r.word(0x16) {
        r.raw[0x11] |= 1;
    }
    Ok(())
}
pub fn tick(
    rom: &[u8],
    c: &Collision,
    party: &mut [Runner],
    selected: usize,
    previous_leader: &Runner,
) -> Result<(), RomError> {
    dispatch(rom, c, party, selected, previous_leader, false)
}
pub fn warmup(
    rom: &[u8],
    c: &Collision,
    party: &mut [Runner],
    selected: usize,
) -> Result<(), RomError> {
    let previous = party
        .get(selected)
        .ok_or(RomError("missing warm-up leader"))?
        .clone();
    dispatch(rom, c, party, selected, &previous, true)
}
fn dispatch(
    rom: &[u8],
    c: &Collision,
    party: &mut [Runner],
    selected: usize,
    previous_leader: &Runner,
    warmup: bool,
) -> Result<(), RomError> {
    let leader = party
        .get(selected)
        .ok_or(RomError("missing follower leader"))?
        .clone();
    let mut serviced = false;
    for (slot, r) in party.iter_mut().enumerate() {
        if r.raw[0x10] & 0x51 != 1 || r.raw[0x10] & 8 == 0 {
            continue;
        }
        if slot == selected {
            if warmup {
                r.raw[0x11] |= 1;
                set_behavior(r, 0xb45c);
                move_one(rom, c, r)?;
            }
            continue;
        }
        let timer = r.word(0xb6).wrapping_sub(1);
        r.set_word(0xb6, timer);
        if (timer as i16) < 0 && !serviced {
            serviced = true;
            r.set_word(0xb6, 20);
            decide(
                rom,
                c,
                r,
                if slot < selected {
                    previous_leader
                } else {
                    &leader
                },
            )?;
        }
        // $ac9e/$ae5e keeps warm-up positions fixed while allowing goals/timers
        // to reset for the destination. Otherwise old-room waypoints survive.
        if warmup {
            r.raw[0x11] |= 1;
            set_behavior(r, 0xb45c);
        }
        move_one(rom, c, r)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stop_distance_rounding_and_word_wrap() {
        assert_eq!(distance((0, 0), (479, 0)), 479);
        assert_eq!(distance((0, 0), (480, 0)), 480);
        assert_eq!(distance((0, 0), (463, 33)), 480);
        assert_eq!(distance((0xfff0, 0), (0x10, 0)), 32);
    }
    #[test]
    fn selected_inactive_and_dead_actors_do_not_consume_ai_slots() {
        let c = Collision {
            cells: vec![0; 4096],
            rectangles: vec![],
            bounds: [0, 0, 512, 512],
            overflow_writes: vec![],
        };
        let mut p: Vec<_> = [15, 0, 0x4d]
            .iter()
            .map(|flag| {
                let mut r = Runner {
                    raw: [0; 256],
                    template: 0,
                    clock: Default::default(),
                };
                r.raw[0x10] = *flag;
                r
            })
            .collect();
        let leader = p[0].clone();
        let before: Vec<_> = p.iter().map(|r| r.raw).collect();
        tick(&[], &c, &mut p, 0, &leader).unwrap();
        assert_eq!(p.iter().map(|r| r.raw).collect::<Vec<_>>(), before);
    }
}
