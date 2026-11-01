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
    state.seed = state.seed.wrapping_mul(0x20e5).wrapping_add(1);
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
fn probe(
    rom: &[u8],
    c: &Collision,
    r: &mut Runner,
    angle: u16,
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
    // Uncontrolled actors cannot travel. Controlled trigger scripts during
    // separation remain guarded until their $ac5e side effects are recovered.
    Ok(if c.contact(candidate)? == Contact::Clear {
        Some(candidate)
    } else {
        None
    })
}
/// $aa88/$aa92/$aaa0 -> $b49c. A response replaces one motion dispatch and
/// restores its handler without turning, changing animation state, or path data.
pub fn respond(rom: &[u8], c: &Collision, r: &mut Runner, phase: u16) -> Result<bool, RomError> {
    let pending = r.raw[0x11] & 2 != 0;
    r.raw[0x11] &= !2;
    if !pending {
        return Ok(false);
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
    let candidate = if let Some(p) = probe(rom, c, r, first)? {
        Some(p)
    } else {
        r.set_word(0x38, 2);
        if let Some(p) = probe(rom, c, r, r.word(0x92).wrapping_add(0x4000))? {
            Some(p)
        } else {
            // $b518/$b51e discards the third candidate for every status value.
            // Preserve that observed branch, including its footprint samples.
            probe(rom, c, r, r.word(0x92).wrapping_add(0x8000))?;
            None
        }
    };
    if let Some((x, y)) = candidate {
        r.set_position(x, y);
    }
    // $ac5e/$b7b8/$baae rejects a non-controlled party actor's final
    // trigger sample and records the original blocked-route flag.
    let samples = &r.raw[0xd0..0xd4];
    if r.raw[0x10] & 0xa == 8 && !samples.contains(&0x7f) && samples.iter().any(|v| *v >= 0x80) {
        r.raw[0xb0] = (r.raw[0xb0] & !0x20) | 1;
    }
    Ok(true)
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
