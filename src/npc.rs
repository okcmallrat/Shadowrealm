//! Fixed NPC spawn record used by the traced Space Needle entry. AI, recruiting
//! and conversation behavior are not replaced with guessed actions.
use crate::{actors::AnimationClock, party::Runner, RomError};
/// $eb92/$eccc for the fixed recruitable NPC, including existing target links.
/// Bounds are asymmetric: lower included, upper excluded, in 1/16 pixels.
pub fn trent_contact(player: &Runner, npc: &Runner) -> bool {
    if player.raw[0x10] & 0x42 != 2 || npc.raw[0x10] & 0x41 != 1 || npc.raw[0x9e] & 0x10 == 0 {
        return false;
    }
    crate::avoidance::overlapping(player, npc)
}
fn bytes(b: &[u8], p: usize, n: usize) -> Result<&[u8], RomError> {
    b.get(p..p + n).ok_or(RomError("NPC data outside ROM"))
}
fn word(b: &[u8], p: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(bytes(b, p, 2)?.try_into().unwrap()))
}
fn long(b: &[u8], p: usize) -> Result<u32, RomError> {
    Ok(u32::from_be_bytes(bytes(b, p, 4)?.try_into().unwrap()))
}
pub fn space_needle(rom: &[u8]) -> Result<Vec<Runner>, RomError> {
    let script = 0x17018;
    let tagged = long(rom, script)?;
    if tagged & 0x80000000 == 0 {
        return Err(RomError(
            "NPC spawn requires unimplemented placement search",
        ));
    }
    let prototype = (tagged & 0x7fffffff) as usize;
    let coordinates = long(rom, script + 4)? as usize;
    let mut raw: [u8; 256] = bytes(rom, 0x7361a, 256)?.try_into().unwrap();
    let mut cursor = prototype;
    // $44bc deserializes the same compact fields as $598b0.
    for (destination, length) in [
        (0xe, 2),
        (0x2c, 2),
        (0x4c, 4),
        (0x56, 2),
        (0x58, 12),
        (0x64, 2),
        (0x66, 16),
        (0x76, 25),
        (0x9a, 3),
        (0x90, 2),
        (0xa0, 2),
        (0x8f, 1),
        (0x9d, 1),
    ] {
        raw[destination..destination + length].copy_from_slice(bytes(rom, cursor, length)?);
        cursor += length;
    }
    raw[0x10] = 5;
    let mut actor = Runner {
        raw,
        template: prototype,
        clock: AnimationClock::default(),
    };
    actor.set_position(word(rom, coordinates)?, word(rom, coordinates + 2)?);
    actor.set_word(0x96, actor.position().0);
    actor.set_word(0x98, actor.position().1);
    actor.raw[0xe0..0xe4].copy_from_slice(&(coordinates as u32).to_be_bytes());
    actor.set_word(0x30, 0); // first free NPC graphics allocation, $1a2cc
    actor.set_word(0x3e, 3);
    actor.set_word(0x20, 0x49);
    actor.set_word(0x26, 0x920);
    let mut cursor = script + 8;
    if long(rom, cursor)? == 2 {
        cursor += 4;
        let mut terminated = false;
        for _ in 0..128 {
            let pair = word(rom, cursor)?;
            cursor += 2;
            if pair == 0 {
                terminated = true;
                break;
            }
            actor.raw[(pair >> 8) as usize] = pair as u8;
        }
        if !terminated {
            return Err(RomError("unterminated NPC overrides"));
        }
    }
    if long(rom, cursor)? != 0 {
        return Err(RomError("additional NPC script entries are unimplemented"));
    }
    actor.set_word(0xca, 30);
    actor.select_motion(false);
    Ok(vec![actor])
}
