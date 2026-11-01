//! Source-side weapon timing ($abfc/$d7ea) and ranged animation setup ($b39a).
//! These do not admit a shot, allocate projectiles, consume ammunition or deal damage.
use crate::{party::Runner, RomError};

/// Preserve byte addition, first-set enhancement priority and signed word shifts.
pub fn cooldown(runner: &Runner, mode: u16) -> u16 {
    let bonus = (12..=14)
        .find(|bit| runner.word(0x90) & (1 << bit) != 0)
        .map_or(0, |bit| (bit - 11) * 4);
    let rating =
        (u16::from(runner.raw[0x78].wrapping_add(runner.raw[0x7b])) / 2 + bonus).saturating_sub(1);
    let base = 106u16.wrapping_sub(rating.wrapping_mul(4));
    let value = match mode {
        0 => base as i16,
        2 => base.wrapping_shl(1) as i16,
        _ => (base as i16) >> 1,
    };
    value.max(15) as u16
}

/// Dispatcher prefix $a8fc..$a912: inactive actors and negative timers hold.
/// Zero becomes $ffff; timer readiness alone does not authorize an attack.
pub fn tick_cooldown(runner: &mut Runner) {
    let value = runner.word(0xdc);
    if runner.raw[0x10] & 1 != 0 && (value as i16) >= 0 {
        runner.set_word(0xdc, value.wrapping_sub(1));
    }
}

/// Source-only setup after a successful original ranged launch. Callers must
/// implement launch admission separately; the native window does not call this yet.
pub fn prepare_ranged(rom: &[u8], runner: &mut Runner, angle: u16) -> Result<(), RomError> {
    let rotated = angle.rotate_left(3);
    let octant = rotated.wrapping_add(u16::from((rotated as i16) < 0)) & 7;
    let at = 0xc042 + usize::from(octant) * 2;
    let state = rom
        .get(at..at + 2)
        .ok_or(RomError("weapon state outside ROM"))?;
    let state = u16::from_be_bytes([state[0], state[1]]);
    let mode = if matches!(runner.raw[0x57], 11 | 12) {
        0
    } else {
        1
    };
    let timer = cooldown(runner, mode);
    runner.set_word(0x2e, angle);
    runner.raw[0x3a..0x3e].copy_from_slice(&0xb3eeu32.to_be_bytes());
    runner.set_word(0x16, state);
    runner.set_word(0xdc, timer);
    runner.set_word(0x1a, 0);
    runner.set_word(0x1e, 0);
    runner.raw[0x11] |= 1;
    runner.clock = Default::default();
    Ok(())
}
