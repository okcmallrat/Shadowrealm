//! Flame Dart/Bolt, Mana Zap/Blast and Heal: original rolls, effects, drain and timing.
use crate::{
    attack_roll, avoidance::State, damage::DamageWorld, dialogue_data::Address, party::Runner,
    RomError,
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    cast_routine: Address,
    damage_routine: Address,
    pub effects: Vec<Effect>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub item: u8,
    pub handler: Address,
    pub frames: Address,
    pub frame_count: u8,
    pub period: u8,
    pub kind: EffectKind,
    pub palette: u16,
    pub initializer: Address,
    pub clear_marker: bool,
    pub reaction: ReactionBasis,
    pub defeat: Option<Address>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    Damage,
    Healing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionBasis {
    Allocation,
    Health,
    None,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/combat/casting.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid casting JSON"))?;
        if c.schema_version != 3
            || c.cast_routine.0 != 0x1033e
            || c.damage_routine.0 != 0x55b78
            || c.effects.len() != 5
        {
            return Err(RomError("casting catalog outside verified scope"));
        }
        for (e, (item, handler, frames, count, period, kind, palette)) in c.effects.iter().zip([
            (1, 0x521e, 0x58b6, 4, 1, EffectKind::Damage, 0x2000),
            (2, 0x5152, 0x589e, 6, 4, EffectKind::Damage, 0x2000),
            (8, 0x508a, 0x5142, 3, 1, EffectKind::Damage, 0x4000),
            (9, 0x4fe2, 0x512a, 6, 4, EffectKind::Damage, 0x4000),
            (12, 0x56de, 0x5756, 16, 4, EffectKind::Healing, 0x6000),
        ]) {
            if (
                e.item,
                e.handler.0,
                e.frames.0,
                e.frame_count,
                e.period,
                e.kind,
                e.palette,
            ) != (item, handler, frames, count, period, kind, palette)
                || e.initializer.0
                    != if kind == EffectKind::Healing {
                        0xb574
                    } else {
                        0x49a0
                    }
                || long(rom, 0x4964 + usize::from(item) * 4)? != e.initializer.0
                || long(rom, 0x490a + usize::from(item) * 4)? != handler
                || e.clear_marker != matches!(item, 2 | 9)
                || e.reaction
                    != match item {
                        9 => ReactionBasis::Health,
                        12 => ReactionBasis::None,
                        _ => ReactionBasis::Allocation,
                    }
                || e.defeat.as_ref().map(|a| a.0)
                    != match item {
                        1 | 2 => Some(0xb0ac),
                        8 | 9 => Some(0xb0e4),
                        _ => None,
                    }
            {
                return Err(RomError("casting effect differs from ROM"));
            }
            for frame in 0..usize::from(count) {
                let at = long(rom, frames + frame * 4)?;
                if rom.get(at..at + 2) != Some(&[0x80, 2]) {
                    return Err(RomError("casting frame outside model"));
                }
            }
        }
        Ok(c)
    }
    pub fn effect(&self, item: u8) -> Option<&Effect> {
        self.effects.iter().find(|e| e.item == item)
    }
}
fn long(rom: &[u8], at: usize) -> Result<usize, RomError> {
    Ok(u32::from_be_bytes(
        rom.get(at..at + 4)
            .ok_or(RomError("casting pointer outside ROM"))?
            .try_into()
            .unwrap(),
    ) as usize)
}
pub fn magic_attribute(a: &Runner) -> u8 {
    let mut v = a.raw[0x80];
    if let Some(slot) = (0..7).rev().find(|slot| a.raw[0x66 + slot] == 0x18) {
        v = v.wrapping_add(a.raw[0x6e + slot]);
    }
    v
}
fn penalty(v: u8) -> u16 {
    if (v.wrapping_sub(8) as i8) >= 0 {
        0
    } else if (v.wrapping_sub(5) as i8) < 0 {
        2
    } else {
        1
    }
}
/// $1035a..$103ba: normal baseline without ongoing effects.
pub fn roll(
    rom: &[u8],
    source: &mut Runner,
    target: &Runner,
    rng: &mut State,
) -> Result<attack_roll::AttackRoll, RomError> {
    if source.raw[0xdf] & 3 != 0 || !matches!(source.raw[0x57], 1 | 2 | 8 | 9 | 12) {
        return Err(RomError("cast modifiers outside baseline"));
    }
    if source.raw[0x9c] == 1 {
        source.raw[0x9f] = 0;
        return Ok(attack_roll::AttackRoll {
            pool: 0,
            difficulty: 0,
            result: attack_roll::DiceRoll {
                successes: 0,
                rolls: 0,
            },
        });
    }
    let magic = magic_attribute(source);
    let mut pool = (u32::from(magic) * u32::from(source.raw[0x4d]) * 25 / 100) as u16;
    if (pool as u8 as i8) > (source.raw[0x7f] as i8) {
        pool = (pool & 0xff00) | u16::from(source.raw[0x7f]);
    }
    pool = (pool & 0xff00) | u16::from((pool as u8).wrapping_add(source.raw[0x9d]));
    if let Some(slot) = (0..7)
        .rev()
        .find(|slot| source.raw[0x66 + slot] == source.raw[0x57].wrapping_add(24))
    {
        pool = (pool & 0xff00) | u16::from((pool as u8).wrapping_add(source.raw[0x6e + slot]));
    }
    if [0x29, 0x2a, 0x2b].contains(&source.raw[0x6d]) {
        let category = *rom
            .get(0xc8ae0 + usize::from(source.raw[0x57] - 1))
            .ok_or(RomError("totem category outside ROM"))?;
        if (category as i8) >= 0 {
            let at =
                0xd4540 + usize::from(source.raw[0x6d] - 1) * 48 + 0x22 + usize::from(category) * 2;
            let bonus = u16::from_be_bytes(
                rom.get(at..at + 2)
                    .ok_or(RomError("totem bonus outside ROM"))?
                    .try_into()
                    .unwrap(),
            );
            pool = pool.wrapping_add(bonus);
            if (pool as i16) <= 0 {
                pool = 1;
            }
        }
    }
    let mut difficulty = 4u16;
    if source.word(0x90) & 2 != 0 {
        difficulty = difficulty.wrapping_sub(1);
    }
    if target.raw[0x9b] == 0x16 {
        difficulty += 2;
    } else if source.word(0x90) & 2 == 0
        && !matches!(source.raw[0x9a], 2 | 4)
        && target.raw[0x11] & 0x80 != 0
    {
        difficulty += 4;
    }
    difficulty = difficulty
        .wrapping_add(penalty(source.raw[0x4c]))
        .wrapping_add(penalty(source.raw[0x9c]));
    if (difficulty.wrapping_sub(2) as i16) < 0 {
        difficulty = 2;
    }
    let result = attack_roll::dice(rng, pool, difficulty);
    source.raw[0x9f] = result.successes;
    Ok(attack_roll::AttackRoll {
        pool,
        difficulty,
        result,
    })
}
#[derive(Clone, Default)]
pub struct Runtime {
    pub graphics: [bool; 18],
    npc_graphics: [bool; 18],
    pub live: usize,
    pub casts: u64,
    pub impacts: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastResult {
    Failed,
    Effect { graphic: usize },
    GraphicsFull,
}
/// $af7c: ordinary casting pose and cooldown, after the admission/roll attempt.
pub fn prepare_pose(rom: &[u8], a: &mut Runner) -> Result<(), RomError> {
    prepare_result_pose(rom, a, false)
}
/// $afae enters $afe2 directly after a nonzero roll; $af7c enters its $afe0 NOP.
pub fn prepare_result_pose(rom: &[u8], a: &mut Runner, successful: bool) -> Result<(), RomError> {
    let rotated = a.word(0x2e).rotate_left(3);
    let octant = rotated.wrapping_add(u16::from((rotated as i16) < 0)) & 7;
    let at = 0xc082 + usize::from(octant) * 2;
    let state = u16::from_be_bytes(
        rom.get(at..at + 2)
            .ok_or(RomError("casting pose outside ROM"))?
            .try_into()
            .unwrap(),
    );
    a.set_word(0xdc, crate::weapons::cooldown(a, 0));
    a.raw[0x3a..0x3e].copy_from_slice(&(if successful { 0xafe2u32 } else { 0xafe0 }).to_be_bytes());
    a.set_word(0x16, state);
    a.set_word(0x1a, 0);
    a.set_word(0x1e, 0);
    a.raw[0x11] |= 1;
    a.clock = Default::default();
    Ok(())
}
impl Runtime {
    /// $1033e full baseline after admission; allocation failure still drains.
    pub fn cast(
        &mut self,
        rom: &[u8],
        actors: &mut [Runner],
        caster: u8,
        target: u8,
        world: &mut crate::weapons::LaunchWorld,
        rng: &mut State,
    ) -> Result<CastResult, RomError> {
        self.cast_with_geometry(rom, None, actors, caster, target, world, rng)
    }
    pub fn cast_with_geometry(
        &mut self,
        rom: &[u8],
        collision: Option<&crate::collision::Collision>,
        actors: &mut [Runner],
        caster: u8,
        target: u8,
        world: &mut crate::weapons::LaunchWorld,
        rng: &mut State,
    ) -> Result<CastResult, RomError> {
        let si = actors
            .iter()
            .position(|a| a.word(0x3e) == u16::from(caster))
            .ok_or(RomError("cast source missing"))?;
        let ti = actors
            .iter()
            .position(|a| a.word(0x3e) == u16::from(target))
            .ok_or(RomError("cast target missing"))?;
        let healing = actors[si].raw[0x57] == 12;
        if healing && (target >= 3 || collision.is_none()) {
            return Err(RomError("Heal requires party target geometry"));
        }
        if caster >= 3
            || actors[si].raw[0x10] & 1 == 0
            || (!healing && (actors[ti].raw[0x9b] == 0x17 || caster == target))
            || world.mode != 0
            || actors
                .iter()
                .any(|a| a.raw[0x10] & 1 != 0 && a.raw[0xdf] & 3 != 0)
            || actors[ti].raw[0xb2] != 0
        {
            return Err(RomError("cast context outside baseline"));
        }
        Catalog::load(rom)?;
        let mut staged = actors.to_vec();
        let mut random = State { seed: rng.seed };
        let target_actor = staged[ti].clone();
        let rolled = roll(rom, &mut staged[si], &target_actor, &mut random)?;
        if rolled.result.successes == 0 {
            actors.clone_from_slice(&staged);
            rng.seed = random.seed;
            return Ok(CastResult::Failed);
        }
        let mut launch = world.clone();
        launch.alert();
        let mut graphics = self.graphics;
        for (g, npc) in graphics.iter_mut().zip(self.npc_graphics) {
            if npc {
                *g = false;
            }
        }
        let mut npc_graphics = [false; 18];
        for a in actors
            .iter()
            .filter(|a| a.word(0x3e) >= 3 && a.raw[0x10] & 9 == 1)
        {
            if let Some(g) = graphics.get_mut(usize::from(a.word(0x30))) {
                *g = true;
                npc_graphics[usize::from(a.word(0x30))] = true;
            }
        }
        let free = graphics.iter().position(|g| !*g);
        if let Some(g) = free {
            graphics[g] = true;
            let angle =
                crate::followers::heading(rom, staged[si].position(), staged[ti].position())?.0;
            staged[si].set_word(0x2e, angle);
            staged[ti].raw[0xd8..0xdc]
                .copy_from_slice(&(0xff0100 + u32::from(caster) * 256).to_be_bytes());
            staged[ti].raw[0xb2] = staged[si].raw[0x57];
            staged[ti].set_word(0x32, g as u16);
            staged[ti].set_word(0x2a, 0x49 + g as u16 * 24);
            staged[ti].raw[2] = 5;
            staged[ti].raw[3] = 0;
            if healing {
                crate::followers::healing_callback(
                    rom,
                    collision.unwrap(),
                    &mut staged[ti],
                    launch.engaged != 0,
                )?;
            }
        }
        let spell = crate::spellbooks::Spell {
            id: staged[si].raw[0x57] - 1,
            force: staged[si].raw[0x9d],
        };
        crate::spell_equipment::apply_drain(rom, &mut staged[si], spell)?;
        actors.clone_from_slice(&staged);
        rng.seed = random.seed;
        self.graphics = graphics;
        self.npc_graphics = npc_graphics;
        if free.is_some() {
            self.live += 1;
        }
        self.casts += 1;
        *world = launch;
        Ok(
            free.map_or(CastResult::GraphicsFull, |graphic| CastResult::Effect {
                graphic,
            }),
        )
    }
    /// Target effect phase after actor animation. Offscreen effects hold their timer.
    pub fn advance(
        &mut self,
        rom: &[u8],
        actors: &mut [Runner],
        world: &mut DamageWorld,
        rng: &mut State,
    ) -> Result<(), RomError> {
        if !actors
            .iter()
            .any(|a| a.raw[0x10] & 1 != 0 && a.raw[0xb2] != 0)
        {
            return Ok(());
        }
        let mut staged = actors.to_vec();
        let mut state = self.clone();
        let mut damage = world.clone();
        let mut random = State { seed: rng.seed };
        state.advance_inner(rom, &mut staged, &mut damage, &mut random)?;
        actors.clone_from_slice(&staged);
        *self = state;
        *world = damage;
        rng.seed = random.seed;
        Ok(())
    }
    fn advance_inner(
        &mut self,
        rom: &[u8],
        actors: &mut [Runner],
        world: &mut DamageWorld,
        rng: &mut State,
    ) -> Result<(), RomError> {
        let catalog = Catalog::load(rom)?;
        for slot in 0..10u16 {
            let Some(i) = actors.iter().position(|a| {
                a.word(0x3e) == slot && a.raw[0x10] & 1 != 0 && a.word(0xc) != 0 && a.raw[0xb2] != 0
            }) else {
                continue;
            };
            let e = catalog
                .effect(actors[i].raw[0xb2])
                .ok_or(RomError("spell effect not supported"))?;
            let timer = actors[i].raw[2].wrapping_sub(1);
            actors[i].raw[2] = timer;
            if (timer as i8) < 0 {
                actors[i].raw[2] = e.period;
                actors[i].raw[3] = actors[i].raw[3].wrapping_add(1);
            }
            if (actors[i].raw[3] as i8) < (e.frame_count as i8) {
                continue;
            }
            let caster = crate::combat::ActorId::from_pointer(u32::from_be_bytes(
                actors[i].raw[0xd8..0xdc].try_into().unwrap(),
            ))
            .ok_or(RomError("spell effect source outside pool"))?;
            actors[i].raw[0xb2] = 0;
            if e.clear_marker {
                actors[i].raw[0xde] = 0;
            }
            let graphic = usize::from(actors[i].word(0x32));
            if let Some(g) = self.graphics.get_mut(graphic) {
                *g = false;
            } else {
                return Err(RomError("spell graphic outside pool"));
            }
            if e.kind == EffectKind::Healing {
                complete_heal(actors, caster.0, slot as u8)?;
                self.impacts += 1;
                self.live = self.live.saturating_sub(1);
                continue;
            }
            let reaction_health = match e.reaction {
                ReactionBasis::Allocation => graphic as u8,
                ReactionBasis::Health => actors[i].raw[0x4c],
                ReactionBasis::None => return Err(RomError("damage effect reaction missing")),
            };
            let _result =
                crate::damage::resolve_magic(rom, actors, caster.0, slot as u8, world, rng)?;
            // Mana Blast saves pre-impact health in D0. Zap and the flame
            // handlers compare the restored graphics-allocation D0 instead.
            if actors[i].raw[0x4c] != reaction_health {
                if actors[i].raw[0x4c] == 0
                    && actors[i].raw[0x10] & 8 == 0
                    && actors[i].bank() != 12
                {
                    let mana = e.defeat.as_ref().map(|a| a.0) == Some(0xb0e4);
                    let handler: u32 = if mana { 0xb10c } else { 0xb0d4 };
                    actors[i].raw[0x3a..0x3e].copy_from_slice(&handler.to_be_bytes());
                    actors[i].set_word(0x16, if mana { 0xe4 } else { 0xe0 });
                    actors[i].set_word(0x1a, 0);
                    actors[i].set_word(0x1e, 0);
                    actors[i].raw[0x11] |= 1;
                    actors[i].clock = Default::default();
                } else {
                    crate::damage::react(rom, actors, slot as u8, world)?;
                }
            }
            self.impacts += 1;
            self.live = self.live.saturating_sub(1);
        }
        Ok(())
    }
}

/// $55c2c/$3674/$37ec: Heal reads current caster successes, restores health,
/// raises zero condition to one, clears fallen flags and resumes idle behavior.
pub fn complete_heal(actors: &mut [Runner], caster: u8, target: u8) -> Result<(), RomError> {
    let successes = actors
        .iter()
        .find(|a| a.word(0x3e) == u16::from(caster))
        .ok_or(RomError("healing caster missing"))?
        .raw[0x9f];
    let a = actors
        .iter_mut()
        .find(|a| a.word(0x3e) == u16::from(target))
        .ok_or(RomError("healing target missing"))?;
    let amount = if (successes as i8) > 0 { successes } else { 1 };
    let old = a.raw[0x4c];
    let sum = old.wrapping_add(amount);
    a.raw[0x4c] = if a.raw[0x10] & 1 == 0 || (old as i8 as i16) + (amount as i8 as i16) <= 0 {
        0
    } else if (sum as i8) > 10 {
        10
    } else {
        sum
    };
    if a.raw[0x9c] == 0 && a.raw[0x10] & 1 != 0 {
        a.raw[0x9c] = 1;
    }
    a.raw[0x10] = (a.raw[0x10] & !0x50) | 1;
    a.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_rejects_unverified_versions_and_unknown_fields_before_rom_access() {
        assert_eq!(
            Catalog::parse(
                &Catalog::JSON.replace("\"schema_version\": 3", "\"schema_version\": 4"),
                &[]
            )
            .err()
            .unwrap()
            .0,
            "casting catalog outside verified scope"
        );
        assert_eq!(
            Catalog::parse(&Catalog::JSON.replacen('{', "{\"unknown\":true,", 1), &[])
                .err()
                .unwrap()
                .0,
            "invalid casting JSON"
        );
    }
    #[test]
    fn unsupported_ongoing_modifiers_do_not_spend_rng_or_change_source() {
        let mut a = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        a.raw[0xdf] = 1;
        a.raw[0x57] = 1;
        let target = a.clone();
        let before = a.raw;
        let mut rng = State { seed: 123 };
        assert!(roll(&[], &mut a, &target, &mut rng).is_err());
        assert_eq!(a.raw, before);
        assert_eq!(rng.seed, 123);
    }
}
