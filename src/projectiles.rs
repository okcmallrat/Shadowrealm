//! Live $14e0a movement and burst continuation. Hits stop at $55948's entry;
//! damage/reactions must be resolved before advancing a pending impact again.
use crate::{attack_roll, avoidance::State, collision::Collision, combat, party::Runner, RomError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightEvent {
    Moving,
    /// Geometry, expiry, or the original zero-success actor contact path.
    Miss {
        actor: Option<u8>,
    },
    DamagePending {
        actor: u8,
        health_before: u8,
    },
    ImpactAnimating,
    Restarted {
        roll: attack_roll::AttackRoll,
    },
    Deactivated,
}
pub struct FlightUpdate {
    pub event: FlightEvent,
    pub steps: u8,
    /// $ffdd50 is written only by the four-step flight loop.
    pub loop_phase: Option<u16>,
    pub sound: Option<u16>,
}
pub struct ImpactResult {
    pub damage: crate::damage::DamageResult,
    pub sound: Option<u16>,
}

/// Complete a just-returned DamagePending event through $55948, impact visuals
/// and $aff2 reaction setup. The caller must consume each event once. Stale
/// health/state inputs and model errors leave the complete operation unchanged.
pub fn complete_impact(
    rom: &[u8],
    projectile: &mut [u8; 256],
    actors: &mut [Runner],
    pending: FlightEvent,
    world: &mut crate::damage::DamageWorld,
    random: &mut State,
) -> Result<ImpactResult, RomError> {
    let FlightEvent::DamagePending {
        actor,
        health_before,
    } = pending
    else {
        return Err(RomError("impact is not awaiting damage"));
    };
    let target = actors
        .iter()
        .find(|a| a.word(0x3e) == u16::from(actor))
        .ok_or(RomError("impact defender missing"))?;
    if target.raw[0x4c] != health_before
        || projectile[0x11] & 9 != 9
        || word(projectile, 0x4c) != 0
        || word(projectile, 0x4e) != 0
    {
        return Err(RomError("stale pending impact"));
    }
    let mut staged = *projectile;
    let mut staged_actors = actors.to_vec();
    let mut staged_world = world.clone();
    let mut staged_random = State { seed: random.seed };
    let damage = crate::damage::resolve(
        rom,
        &staged,
        &mut staged_actors,
        actor,
        &mut staged_world,
        &mut staged_random,
    )?;
    let loss = health_before.wrapping_sub(damage.health_after);
    let sound = if loss == 0 {
        Some(0x3c)
    } else {
        put(&mut staged, 0x2c, 0x2000);
        put(
            &mut staged,
            0x16,
            if (loss as i8) < 3 {
                4
            } else if (loss as i8) < 6 {
                0xc
            } else {
                0x10
            },
        );
        crate::damage::react(rom, &mut staged_actors, actor, &mut staged_world)?
    };
    *projectile = staged;
    actors.clone_from_slice(&staged_actors);
    *world = staged_world;
    random.seed = staged_random.seed;
    Ok(ImpactResult { damage, sound })
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlightContext {
    pub samples: [u8; 4],
    /// $ffdf68 scratch: each point query clears it, then sets it on bounds exit.
    pub deactivate_on_impact: bool,
}
fn word(raw: &[u8; 256], at: usize) -> u16 {
    u16::from_be_bytes([raw[at], raw[at + 1]])
}
fn put(raw: &mut [u8; 256], at: usize, value: u16) {
    raw[at..at + 2].copy_from_slice(&value.to_be_bytes());
}
fn table(rom: &[u8], at: usize) -> Result<i16, RomError> {
    let b = rom
        .get(at..at + 2)
        .ok_or(RomError("projectile velocity outside ROM"))?;
    Ok(i16::from_be_bytes([b[0], b[1]]))
}

/// Execute a single original handler call, not the outer active-actor dispatcher.
/// Absent actor slots represent inactive records; supplied slots must be unique.
/// Actors are scanned in physical slot order, not caller slice order. Caller
/// owns animation advancement and resolution of returned DamagePending events.
/// Host/model errors leave projectile, actors, collision scratch and RNG unchanged.
pub fn advance(
    rom: &[u8],
    collision: &Collision,
    projectile: &mut [u8; 256],
    actors: &mut [Runner],
    context: &mut FlightContext,
    random: &mut State,
) -> Result<FlightUpdate, RomError> {
    let mut seen = [false; 10];
    for actor in actors.iter() {
        let slot = usize::from(actor.word(0x3e));
        if slot >= 10 || seen[slot] {
            return Err(RomError("invalid projectile actor pool"));
        }
        seen[slot] = true;
    }
    let mut staged = *projectile;
    let mut scratch = *context;
    let mut staged_random = State { seed: random.seed };
    // Only a burst continuation can change an actor (the source's roll byte).
    let mut source_change = None;
    let result = if staged[0x11] & 8 != 0 {
        if word(&staged, 0x16) as i16 >= 0 {
            FlightUpdate {
                event: FlightEvent::ImpactAnimating,
                steps: 0,
                loop_phase: None,
                sound: None,
            }
        } else {
            staged[0x41] = staged[0x41].wrapping_sub(1);
            if (staged[0x41] as i8) < 0 {
                staged[0x10] &= !1;
                FlightUpdate {
                    event: FlightEvent::Deactivated,
                    steps: 0,
                    loop_phase: None,
                    sound: None,
                }
            } else {
                if staged[0x9b] != 0 || staged[0x11] & 0x80 != 0 {
                    return Err(RomError("modified burst roll context not implemented"));
                }
                let pointer = u32::from_be_bytes(staged[0x60..0x64].try_into().unwrap());
                let id = combat::ActorId::from_pointer(pointer)
                    .ok_or(RomError("burst source outside actor pool"))?;
                let index = actors
                    .iter()
                    .position(|r| r.word(0x3e) == u16::from(id.0))
                    .ok_or(RomError("burst source missing from actor pool"))?;
                let roll = attack_roll::ranged(&actors[index], &mut staged_random)?;
                source_change = Some((index, roll.result.successes));
                staged[0x9f] = roll.result.successes;
                put(&mut staged, 0x16, 0x14);
                staged[0x89] = 0x20;
                let (x, y) = (word(&staged, 0x58), word(&staged, 0x5a));
                put(&mut staged, 0, x);
                put(&mut staged, 4, y);
                let angle = if staged[0x41] != 0 {
                    word(&staged, 0x2e).wrapping_sub(0x400)
                } else {
                    word(&staged, 0x2e).wrapping_add(0x800)
                };
                put(&mut staged, 0x2e, angle);
                put(&mut staged, 0x2c, 0x4000);
                put(&mut staged, 0x1e, 0);
                put(&mut staged, 0x18, 0);
                staged[0x11] &= !8;
                let offset = usize::from((angle >> 8) & 0xfe);
                put(
                    &mut staged,
                    0x4c,
                    ((i32::from(table(rom, 0x15e56 + offset)?) * 12) >> 4) as u16,
                );
                put(
                    &mut staged,
                    0x4e,
                    ((i32::from(table(rom, 0x15e16 + offset)?) * 12) >> 4) as u16,
                );
                FlightUpdate {
                    event: FlightEvent::Restarted { roll },
                    steps: 0,
                    loop_phase: None,
                    sound: None,
                }
            }
        }
    } else {
        if staged[0x10] & 4 != 0 {
            return Err(RomError("projectile footprint collision not modeled"));
        }
        flight(collision, &mut staged, actors, &mut scratch)?
    };
    if let Some((index, value)) = source_change {
        actors[index].raw[0x9f] = value;
    }
    *projectile = staged;
    *context = scratch;
    random.seed = staged_random.seed;
    Ok(result)
}

fn flight(
    collision: &Collision,
    raw: &mut [u8; 256],
    actors: &[Runner],
    context: &mut FlightContext,
) -> Result<FlightUpdate, RomError> {
    for step in 1..=4 {
        let x = u32::from_be_bytes(raw[..4].try_into().unwrap())
            .wrapping_add(((i32::from(word(raw, 0x4c) as i16)) << 8) as u32);
        let y = u32::from_be_bytes(raw[4..8].try_into().unwrap())
            .wrapping_add(((i32::from(word(raw, 0x4e) as i16)) << 8) as u32);
        let position = ((x >> 16) as u16, (y >> 16) as u16);
        let (px, py) = (position.0 >> 4, position.1 >> 4);
        context.deactivate_on_impact = px < collision.bounds[0]
            || py < collision.bounds[1]
            || px >= collision.bounds[2]
            || py >= collision.bounds[3];
        let blocked = combat::point_blocks(collision, position, &mut context.samples)
            .ok_or(RomError("projectile collision outside modeled geometry"))?;
        let mut stop = blocked;
        let mut hit = None;
        let mut needs_damage = false;
        if !stop {
            raw[0x89] = raw[0x89].wrapping_sub(1);
            stop = (raw[0x89] as i8) < 0;
        }
        if !stop {
            raw[..4].copy_from_slice(&x.to_be_bytes());
            raw[4..8].copy_from_slice(&y.to_be_bytes());
            for slot in 0..10 {
                let Some(actor) = actors.iter().find(|r| r.word(0x3e) == slot) else {
                    continue;
                };
                if actor.raw[0x10] & 0x51 != 1 || actor.raw[0x4c] == 0 {
                    continue;
                }
                let (ax, ay) = actor.position();
                if position.0 < ax.wrapping_sub(0x90)
                    || position.0 >= ax.wrapping_add(0x90)
                    || position.1 < ay.wrapping_sub(0x90)
                    || position.1 >= ay.wrapping_add(0x90)
                {
                    continue;
                }
                if raw[0x10] & 8 != 0
                    && (actor.raw[0xb0] & 0x80 != 0
                        || actor.raw[0x10] & 8 != 0
                        || actor.raw[0x9e] & 0x10 != 0)
                {
                    continue;
                }
                hit = Some(actor);
                needs_damage = raw[0x9f] != 0;
                put(raw, 0, ax);
                put(raw, 4, ay.wrapping_add(0x10));
                stop = true;
                break;
            }
        }
        if stop {
            if context.deactivate_on_impact {
                raw[0x10] &= !1;
            }
            raw[0x11] |= 9;
            put(raw, 0x16, 0);
            put(raw, 0x4c, 0);
            put(raw, 0x4e, 0);
            return Ok(FlightUpdate {
                event: if needs_damage {
                    let actor = hit.unwrap();
                    FlightEvent::DamagePending {
                        actor: actor.word(0x3e) as u8,
                        health_before: actor.raw[0x4c],
                    }
                } else {
                    FlightEvent::Miss {
                        actor: hit.map(|r| r.word(0x3e) as u8),
                    }
                },
                steps: step,
                loop_phase: Some(4 - u16::from(step)),
                sound: if needs_damage { None } else { Some(0x3c) },
            });
        }
    }
    Ok(FlightUpdate {
        event: FlightEvent::Moving,
        steps: 4,
        loop_phase: Some(0),
        sound: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn clear() -> Collision {
        Collision {
            cells: vec![0; 4096],
            rectangles: vec![],
            bounds: [0, 0, 512, 512],
            map_width_tiles: 32,
            overflow_writes: vec![],
        }
    }
    fn bullet() -> [u8; 256] {
        let mut raw = [0; 256];
        put(&mut raw, 0, 208 * 16);
        put(&mut raw, 4, 240 * 16);
        put(&mut raw, 0x4c, 0x1000);
        raw[0x10] = 1;
        raw[0x89] = 10;
        raw[0x9f] = 1;
        raw
    }
    #[test]
    fn actor_slot_order_and_zero_success_contacts_are_preserved_without_damage() {
        let collision = clear();
        let mut actors = vec![];
        for slot in [2, 1] {
            let mut actor = Runner {
                raw: [0; 256],
                template: 0,
                clock: Default::default(),
            };
            actor.set_word(0x3e, slot);
            actor.set_position(208 * 16 + 160, 240 * 16);
            actor.raw[0x10] = 1;
            actor.raw[0x4c] = 10;
            actors.push(actor);
        }
        let before: Vec<_> = actors.iter().map(|r| r.raw).collect();
        let mut context = FlightContext::default();
        let mut random = State { seed: 1234 };
        let mut raw = bullet();
        let event = advance(
            &[],
            &collision,
            &mut raw,
            &mut actors,
            &mut context,
            &mut random,
        )
        .unwrap();
        assert_eq!(event.steps, 1);
        assert_eq!(
            event.event,
            FlightEvent::DamagePending {
                actor: 1,
                health_before: 10
            }
        );
        assert!(event.sound.is_none());
        assert_eq!(actors.iter().map(|r| r.raw).collect::<Vec<_>>(), before);
        let mut raw = bullet();
        raw[0x9f] = 0;
        let event = advance(
            &[],
            &collision,
            &mut raw,
            &mut actors,
            &mut context,
            &mut random,
        )
        .unwrap();
        assert_eq!(event.event, FlightEvent::Miss { actor: Some(1) });
        assert_eq!(event.sound, Some(0x3c));
        assert_eq!(random.seed, 1234);
        assert_eq!(actors.iter().map(|r| r.raw).collect::<Vec<_>>(), before);
    }
    #[test]
    fn missing_geometry_and_burst_sources_fail_without_partial_state() {
        let mut collision = clear();
        collision.cells.clear();
        let mut raw = bullet();
        let before = raw;
        let mut context = FlightContext {
            samples: [1, 2, 3, 4],
            deactivate_on_impact: true,
        };
        let saved = context;
        let mut random = State { seed: 7 };
        assert!(advance(
            &[],
            &collision,
            &mut raw,
            &mut [],
            &mut context,
            &mut random
        )
        .is_err());
        assert_eq!(raw, before);
        assert_eq!(context, saved);
        assert_eq!(random.seed, 7);
        raw[0x11] = 8;
        put(&mut raw, 0x16, 0xffff);
        raw[0x41] = 2;
        let before = raw;
        assert!(advance(&[], &clear(), &mut raw, &mut [], &mut context, &mut random).is_err());
        assert_eq!(raw, before);
        assert_eq!(context, saved);
        assert_eq!(random.seed, 7);
    }
}
