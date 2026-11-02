//! $55640 ranged-launch roll against the freshly cleared projectile record.
//! Ongoing-effect data ($55786) is explicitly unsupported, never treated as zero.
use crate::{avoidance::State, party::Runner, RomError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiceRoll {
    pub successes: u8,
    pub rolls: u32,
}

/// $dba: a zero pool still rolls once; sixes explode; successes wrap as a byte.
/// CMP/BMI checks the subtraction sign alone, including overflow boundaries.
pub fn dice(random: &mut State, pool: u16, difficulty: u16) -> DiceRoll {
    let counter = pool.wrapping_sub(1);
    let counter = if (counter as i16) < 0 { 0 } else { counter };
    let mut result = DiceRoll {
        successes: 0,
        rolls: 0,
    };
    for _ in 0..=counter {
        let mut total = 0u16;
        loop {
            let face = random.sample(6) + 1;
            result.rolls += 1;
            total = total.wrapping_add(face);
            if face != 6 {
                break;
            }
        }
        if (total.wrapping_sub(difficulty) as i16) >= 0 {
            result.successes = result.successes.wrapping_add(1);
        }
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackRoll {
    pub pool: u16,
    pub difficulty: u16,
    pub result: DiceRoll,
}

fn condition_penalty(value: u8) -> u16 {
    if (value.wrapping_sub(8) as i8) >= 0 {
        0
    } else if (value.wrapping_sub(5) as i8) < 0 {
        2
    } else {
        1
    }
}

/// Execute after launch resource consumption, before projectile initialization.
/// $55740's A1 is the newly cleared projectile, not the selected defender:
/// its $9b/$11 penalties are therefore zero on this path.
pub fn ranged(source: &Runner, random: &mut State) -> Result<AttackRoll, RomError> {
    if source.raw[0xdf] & 3 != 0 {
        return Err(RomError("ongoing-effect attack roll data not implemented"));
    }
    let item = source.raw[0x57];
    if item >= 0x80 {
        return Err(RomError("unsupported attack roll item"));
    }
    // $55ed6: byte-wrapped sum followed by the first inventory item $28 bonus.
    let mut physical = source.raw[0x78]
        .wrapping_add(source.raw[0x7b])
        .wrapping_add(source.raw[0x7c])
        / 2;
    if let Some(slot) = (0..8).find(|slot| source.raw[0x66 + slot] == 0x28) {
        physical = physical.wrapping_add(source.raw[0x6e + slot]);
    }
    // $55f20 with D7=0 selects the computed percentage, not the remainder.
    let injury_pool = (u32::from(physical) * u32::from(source.raw[0x4d]) * 25 / 100) as u16;
    let specialized = source.raw[if item < 8 {
        0x82
    } else if item < 11 {
        0x83
    } else {
        0x84
    }];
    let general = source.raw[0x81];
    // $12c74 uses CMP.B/BPL rather than an unsigned maximum operation.
    let skill = u16::from(if (general.wrapping_sub(specialized) as i8) >= 0 {
        general
    } else {
        specialized
    });
    let pool = injury_pool.min(skill).wrapping_add(skill).max(1);
    let range = source.raw[0xb4];
    let mut difficulty: u16 = if range == 0 {
        4
    } else if (range.wrapping_sub(2) as i8) < 0 {
        5
    } else if range == 2 {
        6
    } else {
        8
    };
    if source.word(0x90) & 2 != 0 {
        difficulty = difficulty.wrapping_sub(1);
    }
    // $556c6 indexes flags by item ID minus one, not equipped slot.
    let flags = source.raw[(0x6ei16 + i16::from(item) - 1) as usize];
    if (8..=10).contains(&item) {
        difficulty = difficulty.wrapping_add(3);
        if flags & 0x10 != 0 {
            difficulty = difficulty.wrapping_sub(2);
        } else if flags & 0x20 != 0 {
            difficulty = difficulty.wrapping_sub(3);
        }
    }
    if flags & 8 != 0 {
        difficulty = difficulty.wrapping_sub(1);
    }
    if flags & 1 != 0 {
        if source.word(0x90) & 0x10 != 0 {
            difficulty = difficulty.wrapping_sub(2);
        } else if source.raw[0x66..0x6e].contains(&0x16) {
            difficulty = difficulty.wrapping_sub(1);
        }
    }
    difficulty = difficulty
        .wrapping_add(condition_penalty(source.raw[0x4c]))
        .wrapping_add(condition_penalty(source.raw[0x9c]));
    if (difficulty.wrapping_sub(2) as i16) < 0 {
        difficulty = 2;
    }
    Ok(AttackRoll {
        pool,
        difficulty,
        result: dice(random, pool, difficulty),
    })
}
