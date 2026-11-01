//! Noncombat actor contacts ($eb92) and temporary separation motion ($b49c).
//! Dialogue/quest callbacks and target selection remain owned by their systems.
use crate::{
    collision::{Collision, Contact},
    followers,
    party::Runner,
    RomError,
};

#[derive(Default)]
pub struct State {
    /// Original $fff9f8 recurrence. The native session starts deterministically
    /// at zero; it does not reproduce hardware/input-dependent boot seeding.
    pub seed: u16,
}
impl State {
    pub fn next_word(&mut self) -> u16 {
        self.seed = self.seed.wrapping_mul(0x20e5).wrapping_add(1);
        self.seed
    }
    /// $d3a: high word of the next seed times the candidate count.
    pub fn sample(&mut self, count: u16) -> u16 {
        ((u32::from(self.next_word()) * u32::from(count)) >> 16) as u16
    }
}
fn pointer(r: &Runner, offset: usize) -> u32 {
    u32::from_be_bytes(r.raw[offset..offset + 4].try_into().unwrap())
}
fn address(r: &Runner) -> u32 {
    0xff0100 + u32::from(r.word(0x3e)) * 256
}
/// Unsigned, wrapping bounds, lower inclusive / upper exclusive, as in $eb92.
pub fn overlapping(a: &Runner, b: &Runner) -> bool {
    let linked = [0xa2, 0xac]
        .iter()
        .any(|&p| pointer(a, p) == address(b) || pointer(b, p) == address(a));
    let radius = if linked { 0x100 } else { 0x40 };
    let (x, y) = a.position();
    let (bx, by) = b.position();
    x >= bx.wrapping_sub(radius)
        && x < bx.wrapping_add(radius)
        && y >= by.wrapping_sub(radius)
        && y < by.wrapping_add(radius)
}
/// Apply one later-slot contact, including $d44's one-in-eight adjustment.
/// Returns whether a contact was found; does not invent dialogue/quest effects.
pub fn pair(
    rom: &[u8],
    a: &mut Runner,
    b: &mut Runner,
    state: &mut State,
) -> Result<bool, RomError> {
    if a.raw[0x10] & 0x40 != 0
        || address(a) == 0xff0a00
        || b.raw[0x10] & 0x41 != 1
        || !overlapping(a, b)
    {
        return Ok(false);
    }
    let (angle, _) = followers::heading(rom, a.position(), b.position())?;
    a.raw[0x11] |= 2;
    b.raw[0x11] |= 2;
    a.set_word(0x92, angle.wrapping_add(0xc000));
    b.set_word(0x92, angle.wrapping_add(0x4000));
    a.set_word(0x94, b.word(0x38));
    b.set_word(0x94, a.word(0x38));
    state.next_word();
    if state.seed & 0x7000 == 0 {
        a.set_word(0x92, a.word(0x92).wrapping_add(0xc000));
        b.set_word(0x92, b.word(0x92).wrapping_add(0x4000));
        a.set_word(0x94, a.word(0x94).wrapping_mul(2));
        b.set_word(0x94, b.word(0x94).wrapping_mul(2));
    }
    Ok(true)
}
/// Scan only later active slots. Earlier contacts may already have set this
/// actor's pending bit, which must survive until its own motion dispatch.
pub fn scan(
    rom: &[u8],
    party: &mut [Runner],
    npcs: &mut [Runner],
    slot: usize,
    state: &mut State,
) -> Result<(), RomError> {
    let (earlier, later) = party.split_at_mut(slot + 1);
    let a = &mut earlier[slot];
    for b in later.iter_mut().chain(npcs.iter_mut()) {
        pair(rom, a, b, state)?;
    }
    Ok(())
}
/// Only the previously recovered noncombat travel routes may be activated.
#[derive(Clone, Copy)]
pub struct TravelScope {
    pub map: usize,
    pub has_return: bool,
}
impl TravelScope {
    pub fn allows(self, code: u8) -> bool {
        code >= 0x80
            && (([0, 3, 5].contains(&self.map) && code & 31 == 16)
                || (self.map == 0 && code == 0xc6)
                || (self.map == 10 && self.has_return && code == 0x80))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerRequest {
    pub code: u8,
    /// $bc36 uses the first negative corner, which need not be the maximum code.
    pub orientation: u8,
}
#[derive(Debug, Clone, Copy)]
pub struct Response {
    pub handled: bool,
    pub trigger: Option<TriggerRequest>,
}
fn probe(
    rom: &[u8],
    c: &Collision,
    r: &mut Runner,
    angle: u16,
    travel: Option<TravelScope>,
) -> Result<Option<(u16, u16)>, RomError> {
    let offset = 0xbf82 + usize::from(angle.rotate_left(3) & 7) * 4;
    let data = rom
        .get(offset..offset + 4)
        .ok_or(RomError("separation table outside ROM"))?;
    let p = r.position();
    let candidate = (
        p.0.wrapping_add(u16::from_be_bytes([data[0], data[1]])),
        p.1.wrapping_add(u16::from_be_bytes([data[2], data[3]])),
    );
    let (x, y) = (candidate.0 >> 4, candidate.1 >> 4);
    let samples = if x < c.bounds[0] || x >= c.bounds[2] || y < c.bounds[1] || y >= c.bounds[3] {
        [0, 0x7f, 0, 0x7f] // $e81e writes two words, rather than four identical bytes
    } else {
        c.footprint_samples(candidate)
    };
    r.raw[0xd0..0xd4].copy_from_slice(&samples);
    let accepted = match c.contact(candidate)? {
        Contact::Clear => true,
        Contact::Trigger(code) => {
            r.raw[0x10] & 2 != 0 && travel.is_some_and(|scope| scope.allows(code))
        }
        Contact::Blocked => false,
    };
    Ok(if accepted { Some(candidate) } else { None })
}
/// $aa88/$aa92/$aaa0 -> $b49c. A response replaces one motion dispatch and
/// restores its handler without turning, changing animation state, or path data.
pub fn respond(rom: &[u8], c: &Collision, r: &mut Runner, phase: u16) -> Result<bool, RomError> {
    Ok(respond_with_travel(rom, c, r, phase, None)?.handled)
}
/// Controlled separation may request one of the already recovered travel paths.
/// A request is dispatched after motion, including a discarded third probe.
pub fn respond_with_travel(
    rom: &[u8],
    c: &Collision,
    r: &mut Runner,
    phase: u16,
    travel: Option<TravelScope>,
) -> Result<Response, RomError> {
    let pending = r.raw[0x11] & 2 != 0;
    r.raw[0x11] &= !2;
    if !pending {
        return Ok(Response {
            handled: false,
            trigger: None,
        });
    }
    r.raw.copy_within(0x3a..0x3e, 0xd4);
    r.raw[0x11] &= !4;
    if r.word(0x38) == 0 {
        r.set_word(0x38, 1);
    }
    let first = if phase & 1 != 0 {
        r.word(0x34)
    } else {
        r.word(0x92)
    };
    let candidate = if let Some(p) = probe(rom, c, r, first, travel)? {
        Some(p)
    } else {
        r.set_word(0x38, 2);
        if let Some(p) = probe(rom, c, r, r.word(0x92).wrapping_add(0x4000), travel)? {
            Some(p)
        } else {
            // $b518/$b51e discards the third candidate for every status value.
            // Preserve that observed branch, including its footprint samples.
            probe(rom, c, r, r.word(0x92).wrapping_add(0x8000), travel)?;
            None
        }
    };
    if let Some((x, y)) = candidate {
        r.set_position(x, y);
    }
    let trigger = post_motion_trigger(r, travel);
    Ok(Response {
        handled: true,
        trigger,
    })
}
/// $ac5e -> $b7b8's physical gate, stopping before its location/script call.
fn post_motion_trigger(r: &mut Runner, travel: Option<TravelScope>) -> Option<TriggerRequest> {
    if r.raw[0x10] & 8 == 0 {
        return None;
    }
    let samples: [u8; 4] = r.raw[0xd0..0xd4].try_into().unwrap();
    let code = if samples.contains(&0x7f) {
        0x7f
    } else {
        *samples.iter().max().unwrap()
    };
    if code == 0 {
        r.raw[0xb0] &= !0x20;
    }
    if code < 0x80 || r.raw[0xb0] & 0x20 != 0 {
        return None;
    }
    r.raw[0xb0] |= 0x20;
    if r.raw[0x10] & 2 == 0
        || samples.iter().filter(|v| **v >= 0x80).count() < 2
        || !travel.is_some_and(|scope| scope.allows(code))
    {
        r.raw[0xb0] = (r.raw[0xb0] & !0x20) | 1;
        return None;
    }
    let first = samples.iter().find(|v| **v >= 0x80).copied().unwrap();
    let orientation = first.rotate_left(3) & 3;
    r.set_word(0xcc, u16::from(orientation).rotate_right(2));
    r.set_word(0xc6, code as i8 as i16 as u16);
    Some(TriggerRequest { code, orientation })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn actor(slot: u16, p: (u16, u16)) -> Runner {
        let mut r = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        r.set_word(0x3e, slot);
        r.set_position(p.0, p.1);
        r.raw[0x10] = 0xd;
        r
    }
    #[test]
    fn contact_edges_links_and_wrap_follow_unsigned_bounds() {
        let b = actor(1, (1000, 1000));
        assert!(overlapping(&actor(0, (936, 936)), &b));
        assert!(!overlapping(&actor(0, (1064, 1000)), &b));
        assert!(!overlapping(&actor(0, (1000, 1064)), &b));
        let mut a = actor(0, (744, 744));
        assert!(!overlapping(&a, &b));
        a.raw[0xac..0xb0].copy_from_slice(&address(&b).to_be_bytes());
        assert!(overlapping(&a, &b));
        a.set_position(1256, 1000);
        assert!(!overlapping(&a, &b));
        // The original compares two wrapped unsigned bounds; it does not
        // reinterpret a wrapped interval as a torus around zero.
        assert!(!overlapping(&actor(0, (0, 0)), &actor(1, (0, 0))));
    }
    #[test]
    fn mixed_corner_orientation_and_trigger_latch_match_the_physical_gate() {
        let mut r = actor(0, (4000, 4000));
        r.raw[0x10] = 0xf;
        r.raw[0xd0..0xd4].copy_from_slice(&[0xa6, 0xc6, 0, 0]);
        let scope = Some(TravelScope {
            map: 0,
            has_return: false,
        });
        assert_eq!(
            post_motion_trigger(&mut r, scope),
            Some(TriggerRequest {
                code: 0xc6,
                orientation: 1
            })
        );
        assert_eq!(r.word(0xcc), 0x4000);
        assert_eq!(r.word(0xc6), 0xffc6);
        assert_eq!(
            post_motion_trigger(&mut r, scope),
            None,
            "latched triggers must not dispatch twice"
        );
        r.raw[0xd0..0xd4].fill(0);
        assert_eq!(post_motion_trigger(&mut r, scope), None);
        assert_eq!(r.raw[0xb0] & 0x20, 0);
        r.raw[0xd0] = 0xc6;
        assert_eq!(
            post_motion_trigger(&mut r, scope),
            None,
            "one corner cannot activate travel"
        );
        assert_eq!(r.raw[0xb0] & 0x21, 1);
    }
    #[test]
    fn contact_travel_scope_keeps_unrecovered_locations_blocked() {
        let exterior = TravelScope {
            map: 0,
            has_return: false,
        };
        assert!(exterior.allows(0xc6));
        assert!(exterior.allows(0xb0));
        assert!(!exterior.allows(0xc0));
        assert!(!exterior.allows(0x10));
        assert!(!TravelScope {
            map: 10,
            has_return: false
        }
        .allows(0x80));
        assert!(TravelScope {
            map: 10,
            has_return: true
        }
        .allows(0x80));
        assert!(!TravelScope {
            map: 20,
            has_return: true
        }
        .allows(0x80));
    }
    #[test]
    fn inactive_and_dead_targets_do_not_consume_randomness_or_mutate_actors() {
        for flag in [0, 0x4d] {
            let mut a = actor(0, (1000, 1000));
            let mut b = actor(1, (1000, 1000));
            b.raw[0x10] = flag;
            let before = (a.raw, b.raw);
            let mut state = State { seed: 42 };
            assert!(!pair(&[], &mut a, &mut b, &mut state).unwrap());
            assert_eq!((a.raw, b.raw), before);
            assert_eq!(state.seed, 42);
        }
    }
}
