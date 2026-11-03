//! Native player, ordinary NPC and bounded companion firearm integration. Host input/camera remain separate from the
//! recovered weapon, flight, damage, animation and FX routines.
use crate::{
    actors::AnimationClock,
    avoidance::State,
    collision::Collision,
    damage::DamageWorld,
    input::Controller,
    party::Runner,
    projectiles::{self, FlightContext, FlightEvent},
    weapons::{self, LaunchResult, LaunchWorld},
    RomError,
};

pub struct Runtime {
    pub pool: [[u8; 256]; 6],
    pub launch: LaunchWorld,
    pub damage: DamageWorld,
    pub flight: FlightContext,
    pub shots: u64,
    pub hits: u64,
    pub status: &'static str,
    pub player_attempt: bool,
    pub npc_targets: crate::combat::TargetLists,
    pub npc_force: bool,
    target_tick: Option<u16>,
    pub npc_fire: crate::npc_combat::FireContext,
    pub npc_shots: u64,
    pub companion_shots: u64,
    pub spellbooks: Option<crate::spellbooks::Spellbooks>,
    /// Explicit boundary for spell/inventory selection and non-firearm actions.
    pub companion_deferred: [Option<&'static str>; 3],
}
impl Default for Runtime {
    fn default() -> Self {
        Self {
            pool: [[0; 256]; 6],
            launch: Default::default(),
            damage: Default::default(),
            flight: Default::default(),
            shots: 0,
            hits: 0,
            status: "",
            player_attempt: false,
            npc_targets: Default::default(),
            npc_force: true,
            target_tick: None,
            npc_fire: Default::default(),
            npc_shots: 0,
            companion_shots: 0,
            spellbooks: None,
            companion_deferred: [None; 3],
        }
    }
}
pub fn handler(actor: &Runner) -> u32 {
    u32::from_be_bytes(actor.raw[0x3a..0x3e].try_into().unwrap())
}
pub fn action(actor: &Runner) -> bool {
    matches!(handler(actor), 0xb3ee | 0xb07e | 0xb38a)
}
pub fn holds_movement(actor: &Runner) -> bool {
    action(actor) || actor.raw[0x10] & 0x50 != 0 || actor.raw[0x4c] == 0
}

/// $5fdc then $600e up to frame lookup: use last tick's stored increment,
/// mark terminal state, clamp/loop frame, then load the next increment.
pub fn tick_animation(rom: &[u8], actor: &mut Runner) -> Result<(), RomError> {
    let seq = actor.sequence(rom)?;
    let mut clock = AnimationClock {
        phase: actor.word(0x1a),
        frame: actor.word(0x1e),
        ended: false,
    };
    let advanced =
        (actor.word(0x1a).wrapping_add(actor.word(0x18)) as i16) >= (actor.word(0x1c) as i16);
    clock.tick(
        actor.word(0x1c),
        actor.word(0x18),
        seq.frames.len() as u16,
        seq.terminal,
    )?;
    if advanced {
        actor.raw[0x11] |= 1;
    }
    actor.set_word(0x1a, clock.phase);
    actor.set_word(0x1e, clock.frame);
    actor.set_word(0x18, seq.increment(actor.word(0xca), clock.frame));
    if clock.ended {
        actor.set_word(0x16, actor.word(0x16) | 0x8000);
    }
    actor.clock = clock;
    Ok(())
}
fn actor(raw: [u8; 256]) -> Runner {
    Runner {
        clock: AnimationClock {
            phase: u16::from_be_bytes([raw[0x1a], raw[0x1b]]),
            frame: u16::from_be_bytes([raw[0x1e], raw[0x1f]]),
            ended: u16::from_be_bytes([raw[0x16], raw[0x17]]) & 0x8000 != 0,
        },
        raw,
        template: 0,
    }
}

/// Process a current actor pose at its own dispatch position.
pub fn prepare_actor(r: &mut Runner) {
    if r.raw[0x10] & 1 == 0 {
        return;
    }
    match handler(r) {
        0xb38a => {
            r.set_word(0x38, 0);
            if (r.word(0x16) as i16) < 0 {
                r.raw[0x10] |= 0x40;
            }
        }
        0xb3ee | 0xb07e => {
            r.set_word(0x38, 0);
            if handler(r) == 0xb07e {
                r.raw[0xce] = r.raw[0xce].wrapping_sub(1);
            }
            if (r.word(0x16) as i16) < 0 {
                r.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
                r.select_motion(false);
                r.set_word(0x1a, 0);
                r.set_word(0x1e, 0);
                r.clock = Default::default();
            }
        }
        _ => {}
    }
}

impl Runtime {
    pub fn has_live(&self) -> bool {
        self.pool.iter().any(|p| p[0x10] & 1 != 0)
    }
    pub fn clear_map(&mut self) {
        self.pool = [[0; 256]; 6];
        self.flight = Default::default();
        self.launch.signals = Default::default();
        self.npc_targets = Default::default();
        self.npc_force = true;
        self.target_tick = None;
        self.npc_fire = Default::default();
        self.companion_deferred = [None; 3];
    }
    pub fn actors_changed(&mut self) {
        self.npc_force = true;
    }
    /// $13874 runs once before actor dispatch, so list membership is stable
    /// throughout a frame even on the periodic refresh phases.
    pub fn begin_tick(&mut self, actors: &[Runner], tick: u16) {
        if self.target_tick != Some(tick) || self.npc_force {
            self.npc_targets
                .refresh(&actors.iter().collect::<Vec<_>>(), tick, self.npc_force);
            self.target_tick = Some(tick);
            self.npc_force = false;
        }
    }
    pub fn sprites(&self) -> Vec<Runner> {
        self.pool
            .iter()
            .filter(|p| p[0x10] & 1 != 0)
            .map(|p| actor(*p))
            .collect()
    }

    /// Ordinary engaged NPC firearm dispatch. Returns whether the NPC belongs
    /// to the supported combat-motion path; special/patrol/script actors defer.
    pub fn npc(
        &mut self,
        rom: &[u8],
        collision: &Collision,
        actors: &mut [Runner],
        slot: u16,
        party_serviced: bool,
        tick: u16,
        random: &mut State,
    ) -> Result<bool, RomError> {
        use crate::npc_combat::{self, DispatchBranch};
        self.begin_tick(actors, tick);
        let index = actors
            .iter()
            .position(|a| a.word(0x3e) == slot)
            .ok_or(RomError("NPC actor missing"))?;
        if actors[index].raw[0x10] & 0x57 != 5
            || actors[index].raw[0x4c] == 0
            || actors[index].raw[0x9c] == 0
            || holds_movement(&actors[index])
            || (actors[index].word(0x16) as i16) > 0x3c
        {
            return Ok(false);
        }
        let mut source = actors[index].clone();
        let due = npc_combat::decision_due(&mut source, party_serviced);
        actors[index] = source.clone();
        let branch = npc_combat::dispatch_branch(&source, self.launch.engaged != 0);
        let offscreen = branch == DispatchBranch::Offscreen
            && self.launch.mode == 0
            && (self.launch.engaged != 0 || source.raw[0x9e] & 0x40 != 0);
        let supported = (matches!(branch, DispatchBranch::Combat | DispatchBranch::Alert)
            || offscreen)
            && source.bank() != 13
            && source.raw[0x56] < 8
            && (1..=12).contains(&source.raw[0x57])
            && source.raw[0xdf] & 3 == 0;
        if !supported {
            return Ok(false);
        }
        if !due {
            return Ok(self.launch.engaged != 0 || offscreen);
        }
        if offscreen {
            let leader = actors
                .iter()
                .find(|a| a.raw[0x10] & 3 == 3)
                .ok_or(RomError("offscreen NPC leader missing"))?;
            self.npc_fire.samples =
                crate::followers::decide_with_samples(rom, collision, &mut source, leader)?.1;
            actors[index] = source;
            return Ok(true);
        }
        if branch == DispatchBranch::Alert {
            self.launch.alert();
        }
        if self.launch.engaged == 0 {
            return Ok(false);
        }
        self.npc_targets.acquire(&mut source, random);
        let target_ptr = crate::combat::target(&source);
        let Some(target_index) =
            target_ptr.and_then(|id| actors.iter().position(|a| a.word(0x3e) == u16::from(id.0)))
        else {
            actors[index] = source;
            return Ok(true);
        };
        let mut target = actors[target_index].clone();
        let ordinary = self.firearm(rom, collision, &mut source, &mut target, random)?;
        actors[target_index] = target;
        actors[index] = source;
        Ok(ordinary)
    }

    /// $138b8: shared companion scheduling, formation and ordinary firearms.
    /// Recovered spell selection uses session books. Unsupported equipment/cast
    /// actions defer with staged source/RNG instead of an invented gun preference.
    pub fn companion(
        &mut self,
        rom: &[u8],
        collision: &Collision,
        actors: &mut [Runner],
        slot: u16,
        serviced: &mut bool,
        tick: u16,
        random: &mut State,
    ) -> Result<(), RomError> {
        self.begin_tick(actors, tick);
        if slot >= 3 {
            return Err(RomError("companion slot outside party"));
        }
        let index = actors
            .iter()
            .position(|a| a.word(0x3e) == slot)
            .ok_or(RomError("companion actor missing"))?;
        if actors[index].raw[0x10] & 0x5b != 9
            || holds_movement(&actors[index])
            || actors[index].raw[0x9c] == 0
        {
            return Ok(());
        }
        if !crate::followers::decision_due(&mut actors[index], serviced) {
            return Ok(());
        }
        let mut source = actors[index].clone();
        let leader = actors
            .iter()
            .find(|a| a.raw[0x10] & 3 == 3)
            .ok_or(RomError("companion leader missing"))?
            .clone();
        if source.raw[0x4d] == 0 || self.launch.engaged == 0 {
            self.companion_deferred[usize::from(slot)] = None;
            self.npc_fire.samples =
                crate::followers::decide_with_samples(rom, collision, &mut source, &leader)?.1;
            actors[index] = source;
            return Ok(());
        }
        // $138f4/$13a92: selection is skipped while cooling down. Mundane
        // archetypes ($9b >= 2 signed) return immediately from $13a92.
        if (source.word(0xdc) as i16) < 0
            && (source.raw[0x9b] as i8) < 2
            && ((source.raw[0x56] as i8) < 0 || (source.raw[0x9c] as i8) >= 5)
        {
            if let Some(books) = &self.spellbooks {
                let mut staged = State { seed: random.seed };
                let (selection, _) =
                    crate::spellbooks::select(rom, books, &source, actors, &mut staged)?;
                match selection {
                    crate::spellbooks::Selection::KeepEquipped => {
                        random.seed = staged.seed;
                    }
                    crate::spellbooks::Selection::Spell(_) => {
                        self.companion_deferred[usize::from(slot)] =
                            Some("Companion spell casting is not available yet");
                        return Ok(());
                    }
                    crate::spellbooks::Selection::InventoryWeapon => {
                        let mut equipped = source.clone();
                        crate::spell_equipment::equip_inventory(rom, &mut equipped)?;
                        if equipped.raw[0x56] < 8 && (1..=12).contains(&equipped.raw[0x57]) {
                            source = equipped;
                            random.seed = staged.seed;
                        } else {
                            self.companion_deferred[usize::from(slot)] =
                                Some("Companion melee action is not available yet");
                            return Ok(());
                        }
                    }
                }
            } else {
                self.companion_deferred[usize::from(slot)] =
                    Some("Companion spellbook state is not available yet");
                return Ok(());
            }
        }
        if source.raw[0x56] >= 8
            || !(1..=12).contains(&source.raw[0x57])
            || source.raw[0xdf] & 3 != 0
            || source.raw[0x4d] > 8
        {
            self.companion_deferred[usize::from(slot)] =
                Some("Companion weapon action is not available yet");
            return Ok(());
        }
        self.companion_deferred[usize::from(slot)] = None;
        if self.npc_targets.acquire(&mut source, random) {
            // $14492 calls the real ordinary shutdown when the NPC list is empty.
            actors[index] = source;
            self.launch.end_combat(actors);
            // $13ffe then tails into leader formation; $13d56 sees no target.
            source = actors[index].clone();
            self.npc_fire.samples =
                crate::followers::decide_with_samples(rom, collision, &mut source, &leader)?.1;
            actors[index] = source;
            return Ok(());
        }
        let target_index = crate::combat::target(&source)
            .and_then(|id| actors.iter().position(|a| a.word(0x3e) == u16::from(id.0)))
            .ok_or(RomError("companion target missing from actor pool"))?;
        let mut target = actors[target_index].clone();
        let ordinary = self.firearm(rom, collision, &mut source, &mut target, random)?;
        if !ordinary {
            self.companion_deferred[usize::from(slot)] =
                Some("Companion melee action is not available yet");
        }
        actors[target_index] = target;
        actors[index] = source;
        Ok(())
    }

    /// $139d8/$13ffe and $13d56: firearm action and common goal steering.
    fn firearm(
        &mut self,
        rom: &[u8],
        collision: &Collision,
        source: &mut Runner,
        target: &mut Runner,
        random: &mut State,
    ) -> Result<bool, RomError> {
        use crate::npc_combat::{self, FireDecision, PositionDecision};
        self.npc_fire.mode = self.launch.mode;
        let check = npc_combat::check_fire(rom, collision, source, target, &mut self.npc_fire)?;
        let mut fallback = matches!(
            check.decision,
            FireDecision::PathBlocked | FireDecision::NotVisible
        );
        let mut ordinary = true;
        match check.decision {
            FireDecision::ReadyForLaunch => {
                let (result, _) = weapons::launch_ranged(
                    rom,
                    source,
                    Some(&mut *target),
                    &mut self.pool,
                    &mut self.launch,
                    random,
                )?;
                match result {
                    LaunchResult::Initialized { fx, .. } => {
                        weapons::prepare_ranged(
                            rom,
                            source,
                            u16::from_be_bytes([self.pool[fx][0x2e], self.pool[fx][0x2f]]),
                        )?;
                        if source.raw[0x10] & 8 != 0 {
                            self.companion_shots += 1;
                        } else {
                            self.npc_shots += 1;
                        }
                    }
                    LaunchResult::FxPoolFull { .. } => {
                        if source.raw[0x10] & 8 != 0 {
                            self.companion_shots += 1;
                        } else {
                            self.npc_shots += 1;
                        }
                        fallback =
                            npc_cooldown_path(rom, collision, source, target, &mut self.npc_fire)?;
                    }
                    LaunchResult::Rejected(_) => {
                        fallback =
                            npc_cooldown_path(rom, collision, source, target, &mut self.npc_fire)?;
                    }
                }
                if source.raw[0x57] == 0 {
                    ordinary = false;
                }
            }
            FireDecision::CooldownContinuation => {
                fallback = npc_cooldown_path(rom, collision, source, target, &mut self.npc_fire)?;
                if source.raw[0x57] == 0 {
                    ordinary = false;
                }
            }
            FireDecision::HoldFire => {}
            FireDecision::PathBlocked | FireDecision::NotVisible => {}
            _ => {
                ordinary = false;
            }
        }
        if ordinary {
            if fallback {
                self.npc_fire.samples =
                    crate::followers::decide_with_samples(rom, collision, source, target)?.1;
            } else if npc_combat::position_goal(rom, source, target, self.npc_fire.firing)?
                == PositionDecision::Goal
            {
                let reached = crate::followers::distance(
                    source.position(),
                    (source.word(0x96), source.word(0x98)),
                ) < 0x1e0;
                crate::followers::steer_goal(rom, collision, source)?;
                if reached {
                    source.raw[0xb5] = 0;
                }
            }
        } else if (source.word(0x16) as i16) <= 0x3c {
            // Host boundary: do not enter the still-unimplemented melee/magic
            // continuation after weapon exhaustion.
            source.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
        }
        Ok(ordinary)
    }

    /// Service input at the selected actor's dispatch position, after pose
    /// preparation. Companion/NPC decisions use their separate original entries.
    pub fn player(
        &mut self,
        rom: &[u8],
        actors: &mut [Runner],
        selected: u16,
        input: Controller,
        random: &mut State,
    ) -> Result<(), RomError> {
        self.player_attempt = false;
        let index = actors
            .iter()
            .position(|r| r.word(0x3e) == selected)
            .ok_or(RomError("controlled actor missing"))?;
        let source = &actors[index];
        if self.status == "Reloading" && (source.word(0xdc) as i16) < 0 {
            self.status = "";
        }
        if source.raw[0x10] & 0x53 != 3
            || source.raw[0x4c] == 0
            || source.raw[0x9c] == 0
            || holds_movement(source)
            || source.word(0x16) & 0x7ff > 0x3c
        {
            return Ok(());
        }
        if !weapons::controlled_fire_request(source, &mut self.launch.signals, input) {
            return Ok(());
        }
        self.player_attempt = true;
        if source.raw[0x56] >= 8 || !(1..=12).contains(&source.raw[0x57]) {
            self.status = "This weapon is not available yet";
            return Ok(());
        }
        if actors
            .iter()
            .any(|a| a.raw[0x10] & 1 != 0 && a.raw[0xdf] & 3 != 0)
        {
            self.status = "This weapon effect is not available yet";
            return Ok(());
        }
        let mut source = source.clone();
        source.raw[0x43] = 0;
        let pointer = u32::from_be_bytes(source.raw[0xa2..0xa6].try_into().unwrap());
        let target_index = actors
            .iter()
            .position(|r| 0xff0100 + u32::from(r.word(0x3e)) * 256 == pointer);
        let mut target = target_index.map(|i| actors[i].clone());
        if pointer != 0 && target.is_none() {
            source.raw[0xa2..0xa6].fill(0);
        }
        let result = weapons::launch_ranged(
            rom,
            &mut source,
            target.as_mut(),
            &mut self.pool,
            &mut self.launch,
            random,
        )?;
        if let Some(i) = target_index {
            actors[i] = target.unwrap();
        }
        match result.0 {
            LaunchResult::Initialized { fx, .. } => {
                let angle = u16::from_be_bytes([self.pool[fx][0x2e], self.pool[fx][0x2f]]);
                weapons::prepare_ranged(rom, &mut source, angle)?;
                self.shots += 1;
                self.status = "";
            }
            LaunchResult::FxPoolFull { .. } => {
                self.shots += 1;
                let reloaded =
                    weapons::failed_controlled_launch(rom, &mut source, input.direction())?;
                self.player_attempt = !reloaded;
                self.status = if reloaded {
                    "Reloading"
                } else if source.raw[0x4e] == 0 {
                    "No ammunition"
                } else {
                    ""
                };
            }
            LaunchResult::Rejected(_) => {
                if weapons::failed_controlled_launch(rom, &mut source, input.direction())? {
                    self.player_attempt = false;
                    self.status = "Reloading";
                } else {
                    self.status = if source.raw[0x4e] == 0 {
                        "No ammunition"
                    } else {
                        ""
                    };
                }
            }
        }
        actors[index] = source;
        Ok(())
    }

    /// After actor motion, in projectile slot order; then the frame clock for
    /// each active projectile/FX. All damage uses the shared simulation RNG.
    pub fn projectiles(
        &mut self,
        rom: &[u8],
        collision: &Collision,
        actors: &mut [Runner],
        random: &mut State,
    ) -> Result<(), RomError> {
        for raw in &mut self.pool {
            if raw[0x10] & 1 == 0 {
                continue;
            }
            let mut r = actor(*raw);
            match handler(&r) {
                0x14e0a => {
                    let update = projectiles::advance(
                        rom,
                        collision,
                        &mut r.raw,
                        actors,
                        &mut self.flight,
                        random,
                    )?;
                    if matches!(update.event, FlightEvent::DamagePending { .. }) {
                        projectiles::complete_impact(
                            rom,
                            &mut r.raw,
                            actors,
                            update.event,
                            &mut self.damage,
                            random,
                        )?;
                        self.hits += 1;
                    }
                }
                0xac4e => {
                    if (r.word(0x16) as i16) < 0 {
                        r.raw[0x10] &= !1;
                    }
                }
                _ => return Err(RomError("unknown live projectile handler")),
            }
            if r.raw[0x10] & 1 != 0 {
                tick_animation(rom, &mut r)?;
            }
            *raw = r.raw;
        }
        Ok(())
    }
}

/// $14146 firearm continuation. Party actors reload finite reserves; exhausted
/// non-party actors clear equipment before the unimplemented melee continuation.
fn npc_cooldown_path(
    rom: &[u8],
    collision: &Collision,
    source: &mut Runner,
    target: &Runner,
    context: &mut crate::npc_combat::FireContext,
) -> Result<bool, RomError> {
    if source.raw[0x4e] != 0 {
        let probe = crate::combat::line_of_fire(rom, collision, source, target, context.samples)?;
        if probe.outcome == crate::combat::LineOutcome::OutsideModel {
            return Err(RomError("NPC cooldown probe outside model"));
        }
        context.samples = probe.samples;
        return Ok(probe.outcome == crate::combat::LineOutcome::Blocked);
    }
    if source.raw[0x10] & 8 != 0 && source.raw[0x4f] != 0 {
        weapons::reload_firearm(rom, source)?;
        return Ok(false);
    }
    source.raw[0x57] = 0;
    source.raw[0x56] = 0x7f;
    source.set_word(0xdc, weapons::cooldown(source, 0));
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn periodic_target_refresh_is_stable_through_actor_dispatch() {
        let mut a = Runner {
            raw: [0; 256],
            template: 0,
            clock: Default::default(),
        };
        a.set_word(0x3e, 3);
        a.set_word(0xc, 100);
        a.raw[0x10] = 5;
        let mut actors = vec![a];
        let mut runtime = Runtime::default();
        runtime.begin_tick(&actors, 11);
        assert_eq!(runtime.npc_targets.npcs.len(), 1);
        actors[0].raw[0x10] |= 0x40;
        runtime.begin_tick(&actors, 11);
        assert_eq!(runtime.npc_targets.npcs.len(), 1);
        runtime.begin_tick(&actors, 12);
        assert_eq!(runtime.npc_targets.npcs.len(), 1);
        runtime.begin_tick(&actors, 75);
        assert!(runtime.npc_targets.npcs.is_empty());
        actors[0].raw[0x10] = 5;
        runtime.actors_changed();
        runtime.begin_tick(&actors, 75);
        assert_eq!(runtime.npc_targets.npcs.len(), 1);
    }
}
