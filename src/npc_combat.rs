//! Recovered ordinary NPC branch selection, timing, firearm gates and range goals.
//! Target acquisition and live execution are connected by combat_runtime.
use crate::{
    collision::Collision,
    combat::{self, LineOutcome, LineProbe},
    party::Runner,
    RomError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchBranch {
    Combat,
    Alert,
    Patrol,
    Protected,
    Ignore,
    Special,
    Offscreen,
}
/// Original $13920..$139d8 classification after a due decision. Special
/// script/magic banks and noncombat branches are retained as explicit exits.
pub fn dispatch_branch(actor: &Runner, engaged: bool) -> DispatchBranch {
    if actor.word(0xe) == 15 {
        return DispatchBranch::Special;
    }
    if actor.raw[0xb0] & 0x80 != 0 {
        return DispatchBranch::Protected;
    }
    if actor.raw[0x9e] & 8 != 0 {
        return DispatchBranch::Ignore;
    }
    if actor.raw[0x9e] & 0x20 != 0 && engaged {
        return DispatchBranch::Special;
    }
    if actor.word(0xc) == 0 && actor.raw[0x9e] & 4 != 0 {
        return DispatchBranch::Special;
    }
    if actor.raw[0xb0] & 0x40 != 0 {
        return DispatchBranch::Special;
    }
    if actor.word(0xc) == 0 {
        return DispatchBranch::Offscreen;
    }
    if actor.raw[0x9e] & 0x40 != 0 {
        return DispatchBranch::Alert;
    }
    if !engaged {
        return DispatchBranch::Patrol;
    }
    DispatchBranch::Combat
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionDecision {
    Hold,
    Goal,
}
fn rom_word(rom: &[u8], at: usize) -> Result<u16, RomError> {
    let b = rom
        .get(at..at + 2)
        .ok_or(RomError("NPC range table outside ROM"))?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}
/// $13d56..$13e40 ordinary firearm goal selection, before common steering.
/// Retains the original $b5 range-band counter and $b0 firing-position flag.
pub fn position_goal(
    rom: &[u8],
    actor: &mut Runner,
    target: &Runner,
    firing: u16,
) -> Result<PositionDecision, RomError> {
    let mut staged = actor.clone();
    let (angle, range) = crate::followers::heading(rom, staged.position(), target.position())?;
    staged.raw[0xb0] &= !0x10;
    let band = if staged.raw[0x10] & 8 != 0 {
        if staged.raw[0x4d] > 8 {
            return Err(RomError("companion combat behavior outside range table"));
        }
        Some((0x13e58, usize::from(staged.raw[0x4d])))
    } else if staged.raw[0xb5] != 0 {
        staged.raw[0xb5] = staged.raw[0xb5].wrapping_sub(1);
        let table = if (staged.raw[0x56] as i8) < 0 {
            0xd4ea0i32
        } else {
            0xd4540
        };
        let at = (table + (i32::from(staged.raw[0x57]) - 1) * 48 + 0x27) as usize;
        let band = usize::from(
            *rom.get(at)
                .ok_or(RomError("NPC weapon range index outside ROM"))?,
        );
        if band >= 4 {
            return Err(RomError("NPC weapon range band outside model"));
        }
        Some((0x13e7c, band))
    } else {
        None
    };
    let chase = if let Some((table, band)) = band {
        let near = rom_word(rom, table + band * 4)?;
        let far = rom_word(rom, table + band * 4 + 2)?;
        if range < near {
            false
        } else if range >= far {
            true
        } else {
            staged.raw[0xb5] = 0;
            false
        }
    } else {
        false
    };
    if chase {
        staged.set_word(0x96, target.position().0);
        staged.set_word(0x98, target.position().1);
    } else {
        staged.raw[0xb0] |= 0x10;
        if firing != 0 {
            if (staged.word(0x16) as i16) <= 0x3c {
                staged.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
            }
            *actor = staged;
            return Ok(PositionDecision::Hold);
        }
        let offset = usize::from((angle.wrapping_add(0x4000) >> 8) & 0xfe);
        let distance = range.wrapping_mul(2) as i16;
        let component = |at| -> Result<u16, RomError> {
            Ok((i32::from(rom_word(rom, at)? as i16)
                .wrapping_mul(i32::from(distance))
                .wrapping_mul(2)
                >> 16) as u16)
        };
        staged.set_word(
            0x96,
            staged
                .position()
                .0
                .wrapping_add(component(0x15e56 + offset)?),
        );
        staged.set_word(
            0x98,
            staged
                .position()
                .1
                .wrapping_add(component(0x15e16 + offset)?),
        );
    }
    *actor = staged;
    Ok(PositionDecision::Goal)
}

/// $13904..$13920. Outer active/health/handler checks belong to the caller.
/// A companion-serviced frame keeps a due NPC timer negative instead of resetting it.
pub fn decision_due(actor: &mut Runner, party_serviced: bool) -> bool {
    let timer = actor.word(0xb6).wrapping_sub(1);
    actor.set_word(0xb6, timer);
    if (timer as i16) >= 0 || party_serviced {
        return false;
    }
    actor.set_word(0xb6, 20);
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireDecision {
    ReadyForLaunch,
    CooldownContinuation,
    NotVisible,
    MagicContinuation,
    ThrowableContinuation,
    PathBlocked,
    HoldFire,
    MeleeContinuation,
}
pub struct FireCheck {
    pub decision: FireDecision,
    pub probe: Option<LineProbe>,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FireContext {
    pub mode: u8,
    /// Original $ffdfde: set only on the ready branch, cleared on a blocked probe.
    pub firing: u16,
    pub samples: [u8; 4],
}

/// $14020..$1413c: the existing target's weapon/cooldown/visibility/probe gates.
/// Ready is the entry to $1514e, not a completed attack. The attack-position
/// flag $b0 bit 4 matters in mode zero even when the projectile probe is clear.
/// Missing geometry/invalid references fail without changing shared context.
pub fn check_fire(
    rom: &[u8],
    collision: &Collision,
    source: &Runner,
    target: &Runner,
    context: &mut FireContext,
) -> Result<FireCheck, RomError> {
    let pointer = u32::from_be_bytes(source.raw[0xa2..0xa6].try_into().unwrap());
    if source.word(0x3e) >= 10
        || target.word(0x3e) >= 10
        || pointer != 0xff0100 + u32::from(target.word(0x3e)) * 256
    {
        return Err(RomError("NPC fire target outside modeled actor pool"));
    }
    let early = if source.raw[0x57] == 0 {
        Some(FireDecision::MeleeContinuation)
    } else if (source.word(0xdc) as i16) >= 0 {
        Some(FireDecision::CooldownContinuation)
    } else if source.word(0xc) == 0 {
        Some(FireDecision::NotVisible)
    } else if (source.raw[0x56] as i8) < 0 {
        Some(FireDecision::MagicContinuation)
    } else if (13..=15).contains(&source.raw[0x57]) {
        Some(FireDecision::ThrowableContinuation)
    } else {
        None
    };
    if let Some(decision) = early {
        return Ok(FireCheck {
            decision,
            probe: None,
        });
    }
    if source.raw[0x57] >= 0x80 {
        return Err(RomError("NPC firearm ID outside model"));
    }
    let probe = combat::line_of_fire(rom, collision, source, target, context.samples)?;
    let decision = match probe.outcome {
        LineOutcome::OutsideModel => {
            return Err(RomError("NPC fire probe outside modeled geometry"))
        }
        LineOutcome::Blocked => {
            context.firing = 0;
            FireDecision::PathBlocked
        }
        _ if context.mode == 0 && source.raw[0xb0] & 0x10 == 0 => FireDecision::HoldFire,
        _ => {
            context.firing = 1;
            FireDecision::ReadyForLaunch
        }
    };
    context.samples = probe.samples;
    Ok(FireCheck {
        decision,
        probe: Some(probe),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_target_and_unmodeled_probe_leave_context_unchanged() {
        let mut source = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        source.set_word(0x3e, 3);
        source.set_word(0xdc, 0xffff);
        source.set_word(0xc, 1);
        source.raw[0x57] = 2;
        source.set_position(200 * 16, 200 * 16);
        let mut target = source.clone();
        target.set_word(0x3e, 0);
        let collision = Collision {
            cells: vec![0; 4096],
            rectangles: vec![],
            bounds: [0, 0, 512, 512],
            map_width_tiles: 32,
            overflow_writes: vec![],
        };
        let mut context = FireContext {
            mode: 0,
            firing: 0x1234,
            samples: [1, 2, 3, 4],
        };
        let before = context;
        assert!(check_fire(&[], &collision, &source, &target, &mut context).is_err());
        assert_eq!(context, before);
        source.raw[0xa2..0xa6].copy_from_slice(&0xff0100u32.to_be_bytes());
        assert!(check_fire(&[], &collision, &source, &target, &mut context).is_err());
        assert_eq!(context, before);
    }
}
