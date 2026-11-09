//! Street encounter gate, district selection and collision-grid placement.
//! Scripted conversations and non-firearm groups remain explicit deferred exits.
use crate::{avoidance, collision::Collision, maps::MapDescriptor, party::Runner, RomError};
fn word(b: &[u8], p: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes(
        b.get(p..p + 2)
            .ok_or(RomError("encounter word outside ROM"))?
            .try_into()
            .unwrap(),
    ))
}
fn long(b: &[u8], p: usize) -> Result<u32, RomError> {
    Ok(u32::from_be_bytes(
        b.get(p..p + 4)
            .ok_or(RomError("encounter pointer outside ROM"))?
            .try_into()
            .unwrap(),
    ))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub kind: u8,
    pub address: usize,
}
/// $59986. Two independent samples of five form the district table index.
pub fn select(rom: &[u8], map: usize, rng: &mut avoidance::State) -> Result<Selection, RomError> {
    if map >= crate::maps::MAP_COUNT {
        return Err(RomError("encounter map outside descriptor table"));
    }
    let district = usize::from(
        *rom.get(0x3312 + map)
            .ok_or(RomError("encounter district missing"))?,
    );
    let mut staged = avoidance::State { seed: rng.seed };
    let sum = usize::from(staged.sample(5) + staged.sample(5));
    let kind = *rom
        .get(0x1d70e4 + district * 9 + sum)
        .ok_or(RomError("encounter kind missing"))?;
    let mut address = long(rom, 0x1d7174 + district * 36 + sum * 4)? as usize;
    if kind != 0 {
        address = long(rom, address)? as usize;
    }
    rng.seed = staged.seed;
    Ok(Selection { kind, address })
}
/// Inputs to $1e166. Mode/script encounters and mission callbacks are not inferred.
#[derive(Default, Clone, Copy)]
pub struct Gate {
    pub tick: u16,
    pub paused: u16,
    pub player_flags: u8,
    pub blocked: u8,
    pub script: u8,
    pub engaged: u16,
    pub count: u8,
    pub chance: u16,
}
/// $1e166..$1e1d0: guard order matters for shared RNG consumption.
pub fn due(g: Gate, rng: &mut avoidance::State) -> bool {
    if g.paused != 0
        || g.player_flags & 0x80 != 0
        || g.blocked != 0
        || (g.script == 0 && g.engaged != 0)
        || g.count != 0
        || g.tick & 0x3ff != 10
    {
        return false;
    }
    rng.next_word() < g.chance
}
/// $1e302/$1e40c. Camera and extents are original fixed-point words.
/// The raster lookup intentionally tests one cell, not a footprint or actor scan.
pub fn positions(
    map: &MapDescriptor,
    camera: (u16, u16),
    c: &Collision,
    close: bool,
) -> Result<Vec<(u16, u16)>, RomError> {
    let center = (camera.0.wrapping_add(0xa00), camera.1.wrapping_add(0x700));
    let mut dx = if close { 0x500u16 } else { 0xbe0 };
    let mut dy = if close { 0x500u16 } else { 0x8c0 };
    if word(&map.raw, 8)? >> 1 < center.0 {
        dx = dx.wrapping_neg();
    }
    if word(&map.raw, 10)? >> 1 < center.1 {
        dy = dy.wrapping_neg();
    }
    let align = |v: u16| (v & 0xfe00).wrapping_add(0x100);
    let mut out = vec![];
    for axis in 0..2 {
        let origin = if axis == 0 {
            (align(center.0.wrapping_add(dx)), align(center.1))
        } else {
            (align(center.0), align(center.1.wrapping_add(dy)))
        };
        for shift in [0u16, 0x200, 0x400, 0xfe00, 0xfc00] {
            let (x, y) = if axis == 0 {
                (origin.0, origin.1.wrapping_add(shift))
            } else {
                (origin.0.wrapping_add(shift), origin.1)
            };
            let offset =
                usize::from(((y >> 4).wrapping_mul(2) & 0xffc0).wrapping_add((x >> 4) >> 5));
            let cell = *c
                .cells
                .get(offset)
                .ok_or(RomError("encounter raster lookup outside model"))?;
            if cell == 0 {
                out.push((x, y));
            }
        }
    }
    Ok(out)
}
/// $1e216/$1e25a firearm group prefix. Allocation stops at the first full pool.
/// All actor and RNG changes are staged, so unsupported groups are deferred whole.
pub fn spawn(
    rom: &[u8],
    group: usize,
    points: &[(u16, u16)],
    actors: &[Runner],
    map: u16,
    seed: u16,
) -> Result<(Vec<Runner>, u16), RomError> {
    spawn_inner(rom, group, points, actors, map, seed, false)
}
fn spawn_inner(
    rom: &[u8],
    group: usize,
    points: &[(u16, u16)],
    actors: &[Runner],
    map: u16,
    seed: u16,
    unarmed: bool,
) -> Result<(Vec<Runner>, u16), RomError> {
    if points.is_empty() || points[0].0 == 0 {
        return Ok((vec![], seed));
    }
    let mut allocated = actors.to_vec();
    let mut added = vec![];
    let mut cursor = group;
    let mut rng = avoidance::State { seed };
    for index in 0..8 {
        let prototype = long(rom, cursor)? as usize;
        if prototype == 0 {
            return Ok((added, rng.seed));
        }
        cursor += 4;
        if prototype & 0x80000000 != 0 {
            return Err(RomError(
                "encounter group requires unsupported combat continuation",
            ));
        }
        let Some(slot) = (3..10u16).find(|slot| {
            !allocated
                .iter()
                .any(|a| a.word(0x3e) == *slot && a.raw[0x10] & 1 != 0)
        }) else {
            return Ok((added, rng.seed));
        };
        let Some(graphic) = (0..18u16).find(|g| {
            !allocated.iter().any(|a| {
                (a.word(0x3e) >= 3 && a.raw[0x10] & 9 == 1 && a.word(0x30) == *g)
                    || (a.raw[0x10] & 1 != 0 && a.raw[0xb2] != 0 && a.word(0x32) == *g)
            })
        }) else {
            return Ok((added, rng.seed));
        };
        let mut actor = crate::npc::initialize(
            rom,
            prototype,
            0x1e300,
            slot,
            graphic,
            0,
            map,
            false,
            &mut cursor,
            &mut rng,
        )?;
        if actor.bank() == 13
            || actor.bank() == 15
            || !((actor.raw[0x56] < 8 && (1..=12).contains(&actor.raw[0x57]))
                || (unarmed
                    && actor.raw[0x57] == 0
                    && actor.bank() < 12
                    && actor.raw[0x11] & 0x40 == 0))
            || actor.raw[0xdf] & 3 != 0
            || actor.raw[0x9e] & 0x28 != 0
            || actor.raw[0xb0] & 0xc0 != 0
        {
            return Err(RomError(
                "encounter group requires unsupported combat continuation",
            ));
        }
        actor.raw[0x4d] = 4;
        actor.raw[0x10] |= 0x20;
        actor.raw[0x9e] |= 0x40;
        let point = points[index % points.len()];
        actor.set_position(point.0, point.1);
        actor.set_word(0x96, point.0);
        actor.set_word(0x98, point.1);
        allocated.retain(|a| a.word(0x3e) != slot);
        allocated.push(actor.clone());
        added.push(actor);
    }
    Err(RomError("unterminated encounter group"))
}

#[derive(Default, Clone)]
pub struct Runtime {
    /// Original $e177, restricted to actors created by this bounded encounter path.
    pub count: u8,
    pub spawned: u64,
    pub deferred: u64,
    pub last: Option<Selection>,
    counted: [bool; 7],
}
impl Runtime {
    /// $1e1fc/$1e216 close placement for the verified Stoker group only.
    pub fn stoker(
        &mut self,
        rom: &[u8],
        map: &MapDescriptor,
        camera: (u16, u16),
        c: &Collision,
        actors: &mut [Runner],
        world: &mut crate::weapons::LaunchWorld,
        rng: &mut avoidance::State,
    ) -> Result<usize, RomError> {
        if map.id != 2 {
            return Err(RomError("Stoker encounter outside Redmond"));
        }
        let points = positions(map, camera, c, true)?;
        let (added, seed) = spawn_inner(rom, 0x1d6e2a, &points, actors, 2, rng.seed, true)?;
        let count = added.len();
        for actor in added {
            let index = usize::from(actor.word(0x3e) - 3);
            let destination = actors
                .iter_mut()
                .find(|a| a.word(0x3e) == actor.word(0x3e))
                .ok_or(RomError("scripted encounter pool missing slot"))?;
            *destination = actor;
            self.counted[index] = true;
            self.count = self.count.wrapping_add(1);
            self.spawned += 1;
            world.alert();
        }
        rng.seed = seed;
        Ok(count)
    }
    pub fn clear_map(&mut self) {
        self.count = 0;
        self.counted = [false; 7];
        self.last = None;
    }
    /// $ab34..$ab48: count the first completed death at its actor dispatch slot.
    pub fn finish_death(&mut self, a: &Runner) -> bool {
        let Some(index) = a
            .word(0x3e)
            .checked_sub(3)
            .map(usize::from)
            .filter(|i| *i < 7)
        else {
            return false;
        };
        if !self.counted[index] || a.raw[0x10] & 0x40 == 0 {
            return false;
        }
        self.counted[index] = false;
        self.count = self.count.wrapping_sub(1);
        self.count == 0
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        rom: &[u8],
        map: &MapDescriptor,
        camera: (u16, u16),
        c: &Collision,
        actors: &mut [Runner],
        tick: u16,
        world: &mut crate::weapons::LaunchWorld,
        rng: &mut avoidance::State,
    ) -> Result<(), RomError> {
        // $ab34/$1dea0: count a completed death once, then release offscreen corpses.
        let mut ended = false;
        for a in actors.iter_mut().filter(|a| a.word(0x3e) >= 3) {
            let index = usize::from(a.word(0x3e) - 3);
            if index >= 7 {
                continue;
            }
            ended |= self.finish_death(a);
            if a.raw[0x10] & 0x49 == 0x41 && a.word(0xc) == 0 {
                a.raw[0xb0] &= !0x80;
                a.raw[0x10] &= !1;
            }
        }
        if ended {
            world.end_combat(actors);
        }
        if world.mode != 0 {
            return Ok(());
        }
        let Some(player) = actors.iter().find(|a| a.raw[0x10] & 3 == 3) else {
            return Ok(());
        };
        if !due(
            Gate {
                tick,
                player_flags: player.raw[0x11],
                engaged: world.engaged,
                count: self.count,
                chance: word(&map.raw, 0x52)?,
                ..Default::default()
            },
            rng,
        ) {
            return Ok(());
        }
        let selection = select(rom, map.id, rng)?;
        self.last = Some(selection);
        if selection.kind != 0 {
            self.deferred += 1;
            return Ok(());
        }
        let points = match positions(map, camera, c, false) {
            Ok(points) => points,
            Err(RomError("encounter raster lookup outside model")) => {
                self.deferred += 1;
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        let (added, seed) = match spawn(
            rom,
            selection.address,
            &points,
            actors,
            map.id as u16,
            rng.seed,
        ) {
            Ok(result) => result,
            Err(RomError("encounter group requires unsupported combat continuation")) => {
                self.deferred += 1;
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        if added.is_empty() {
            return Ok(());
        }
        rng.seed = seed;
        for actor in added {
            let index = usize::from(actor.word(0x3e) - 3);
            let destination = actors
                .iter_mut()
                .find(|a| a.word(0x3e) == actor.word(0x3e))
                .ok_or(RomError("encounter pool missing slot"))?;
            *destination = actor;
            self.counted[index] = true;
            self.count = self.count.wrapping_add(1);
            self.spawned += 1;
            world.alert();
        }
        Ok(())
    }
}
