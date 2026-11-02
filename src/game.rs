//! Stateful exploration and bounded player gunfire, using recovered templates,
//! collision, animation and travel. General story and combat AI remain incomplete.
use crate::{
    actors::{project, ActorFrame},
    avoidance,
    collision::{Collision, Contact},
    combat::{self, Targeting},
    dialogue::Dialogue,
    dialogue_data::{Catalog, Knowledge},
    environment::Environment,
    followers,
    hiring::{self, Contract, Outcome, Quote},
    hud::Hud,
    input::Controller,
    map_render::MapScene,
    maps::MapDescriptor,
    npc,
    party::Runner,
    scene::{self, PlacedActor},
    transitions::{
        ActiveTransition, ExteriorReturn, TransitionKind, TransitionPlan, TransitionStage,
    },
    video, RomError,
};
pub struct Game {
    pub scene: MapScene,
    pub party: Vec<Runner>,
    pub npcs: Vec<Runner>,
    pub dialogue: Option<Dialogue>,
    pub saved_exterior: Option<ExteriorReturn>,
    pub knowledge: Knowledge,
    pub nuyen: u32,
    pub targeting: Targeting,
    pub combat: crate::combat_runtime::Runtime,
    pub encounters: crate::encounters::Runtime,
    pub defeated: bool,
    defeat_notice: Option<Dialogue>,
    clinic_plan: Option<TransitionPlan>,
    /// Original recovery/insurance flag word ($fffc1e); baseline starts uninsured.
    pub recovery_flags: u32,
    catalog: Catalog,
    hiring_quote: Option<Quote>,
    npc_dialogue_cooldown: u8,
    dialogue_target: Option<TransitionPlan>,
    pub selected: usize,
    pub camera: (u16, u16),
    pub collision: Collision,
    pub environment: Environment,
    pub tick: u64,
    pub blocked: bool,
    pub pending_transition: Option<u8>,
    pub hud: Hud,
    pub transition: Option<ActiveTransition>,
    pub completed_transitions: u64,
    map: MapDescriptor,
    input: Controller,
    avoidance: avoidance::State,
}
impl Game {
    /// Template selection is explicit: it is not an earned/recruited save-game roster.
    pub fn load(rom: &[u8], map_id: usize, templates: &[usize]) -> Result<Self, RomError> {
        if templates.is_empty() || templates.len() > 3 {
            return Err(RomError("party must contain one to three runners"));
        }
        let map = MapDescriptor::read(rom, map_id)?;
        let catalog = Catalog::load(rom)?;
        let mut scene = MapScene::load(rom, map_id)?;
        let collision = Collision::load(rom, &map)?;
        scene.apply_collision_overflow(&collision.overflow_writes)?;
        let environment = Environment::load(rom, &map)?;
        let camera = (
            u16::from_be_bytes([map.raw[0], map.raw[1]]) >> 4,
            u16::from_be_bytes([map.raw[2], map.raw[3]]) >> 4,
        );
        let mut party = Vec::new();
        for (slot, template) in templates.iter().enumerate() {
            let mut runner = Runner::load(rom, *template, slot)?;
            // Host development roster: templates contain provisional resource
            // values (32) above several equipped magazines. Match the existing
            // HUD capacity clamp before play, without inventing a new-game save.
            if runner.raw[0x56] < 8 && (1..=12).contains(&runner.raw[0x57]) {
                let at = 0xd4540 + usize::from(runner.raw[0x57] - 1) * 48 + 0x24;
                runner.raw[0x4e] = runner.raw[0x4e].min(
                    *rom.get(at)
                        .ok_or(RomError("starting magazine capacity missing"))?,
                );
            }
            let offset = 0x42 + slot * 4;
            runner.set_position(
                u16::from_be_bytes([map.raw[offset], map.raw[offset + 1]]),
                u16::from_be_bytes([map.raw[offset + 2], map.raw[offset + 3]]),
            );
            runner.set_word(0xca, 30);
            // $595b2/$595ba: only the initially controlled runner owns bit 1.
            runner.raw[0x10] = (runner.raw[0x10] & !2) | if slot == 0 { 2 } else { 0 };
            party.push(runner);
        }
        let hud = Hud::load(rom, &mut scene)?;
        let mut result = Self {
            scene,
            party,
            npcs: vec![],
            dialogue: None,
            saved_exterior: None,
            knowledge: Knowledge::default(),
            nuyen: hiring::initial_balance(rom)?,
            targeting: Targeting::default(),
            combat: Default::default(),
            encounters: Default::default(),
            defeated: false,
            defeat_notice: None,
            clinic_plan: None,
            recovery_flags: 0,
            hiring_quote: None,
            catalog,
            npc_dialogue_cooldown: 0,
            dialogue_target: None,
            selected: 0,
            camera,
            collision,
            environment,
            tick: 0,
            blocked: false,
            pending_transition: None,
            hud,
            transition: None,
            completed_transitions: 0,
            map,
            input: Controller::default(),
            avoidance: avoidance::State::default(),
        };
        result.refresh_hud(rom)?;
        result.sync_projections();
        Ok(result)
    }
    pub fn map_id(&self) -> usize {
        self.map.id
    }
    fn sync_projections(&mut self) {
        let camera = (
            self.camera.0.wrapping_mul(16),
            self.camera.1.wrapping_mul(16),
        );
        for r in self.party.iter_mut().chain(self.npcs.iter_mut()) {
            let origin = crate::actors::project_origin(r.position(), camera, r.word(8) as i16);
            let visible = project(r.position(), camera, r.word(8) as i16, false).is_some();
            r.set_word(0xa, origin.0);
            r.set_word(0xc, if visible { origin.1 } else { 0 });
        }
    }
    fn follow_dialogue(&mut self, rom: &[u8], slot: usize) -> Result<(), RomError> {
        let dialogue = self
            .dialogue
            .as_ref()
            .ok_or(RomError("no active conversation"))?;
        let Some(choice) = dialogue.choices.get(slot).filter(|c| c.available) else {
            return Ok(());
        };
        let hire = &self.catalog.trent.hiring;
        if choice.destination == hire.quote_action.id {
            if self.party.len() >= 3 {
                self.dialogue = Some(Dialogue::from_record(
                    rom,
                    self.catalog
                        .node(hire.full_party.0)
                        .ok_or(RomError("missing full-party response"))?,
                    &self.catalog,
                )?);
                self.hiring_quote = None;
            } else {
                let quote = hiring::quote(rom, hire, &self.party[0], self.catalog.trent.opinion)?;
                self.dialogue = Some(Dialogue::hiring_offer(
                    rom,
                    &self.catalog,
                    quote,
                    self.nuyen,
                )?);
                self.hiring_quote = Some(quote);
            }
            return Ok(());
        }
        if choice.destination == hire.short_term_action.id
            || choice.destination == hire.permanent_action.id
        {
            if dialogue.node != hire.offer.options_address.0 {
                return Err(RomError("purchase outside hiring offer"));
            }
            let quote = self
                .hiring_quote
                .ok_or(RomError("purchase without a quoted price"))?;
            let npc_index = self
                .npcs
                .iter()
                .position(|n| n.template == 0x171a2)
                .ok_or(RomError("hiring NPC no longer present"))?;
            let contract = if choice.destination == hire.short_term_action.id {
                Contract::SingleRun
            } else {
                Contract::Permanent
            };
            let outcome = hiring::prepare(
                rom,
                &self.party,
                &self.npcs[npc_index],
                self.nuyen,
                quote,
                contract,
            )?;
            let response = match &outcome {
                Outcome::FullParty => hire.full_party,
                Outcome::InsufficientFunds => hire.insufficient_funds,
                Outcome::Hired { .. } => hire.success,
            };
            let mut next = Dialogue::from_record(
                rom,
                self.catalog
                    .node(response.0)
                    .ok_or(RomError("missing hiring outcome"))?,
                &self.catalog,
            )?;
            if let Outcome::Hired { member, balance } = outcome {
                let mut party = self.party.clone();
                party.push(*member);
                // Prepare HUD/graphics on a copy: a failed asset load must not
                // deduct money, add a partial actor or remove the NPC.
                let mut scene = self.scene.clone();
                let mut hud = Hud::load(rom, &mut scene)?;
                hud.select(rom, self.selected)?;
                for (i, r) in party.iter().enumerate() {
                    r.update_hud(rom, &mut scene, &mut hud, i)?;
                    hud.condition(
                        rom,
                        i,
                        if (r.raw[0x9c] as i8) > 0 {
                            r.raw[0x9c].min(10)
                        } else {
                            0
                        },
                        i == self.selected,
                    )?;
                }
                self.party = party;
                self.scene = scene;
                self.hud = hud;
                self.nuyen = balance;
                self.npcs.remove(npc_index);
            }
            next.funds = Some(self.nuyen);
            self.dialogue = Some(next);
            self.hiring_quote = None;
            return Ok(());
        }
        if let Some(next) = dialogue.follow(rom, &self.catalog, slot, &mut self.knowledge)? {
            self.dialogue = Some(next);
            self.hiring_quote = None;
        }
        Ok(())
    }
    fn resolve_trigger(
        &self,
        rom: &[u8],
        code: u8,
        orientation: u8,
    ) -> Result<Option<TransitionPlan>, RomError> {
        if let Some(plan) =
            TransitionPlan::exterior(rom, self.map.id, code, self.camera.0.wrapping_mul(16))?
        {
            return Ok(Some(plan));
        }
        if let Some(saved) = self.saved_exterior {
            return Ok(TransitionPlan::leave_interior(self.map.id, code, saved));
        }
        TransitionPlan::space_needle_from_contact(
            rom,
            &self.map,
            code,
            orientation,
            (
                self.camera.0.wrapping_mul(16),
                self.camera.1.wrapping_mul(16),
            ),
            self.party[self.selected].position(),
        )
    }
    fn start_transition(&mut self, rom: &[u8], plan: TransitionPlan) -> Result<(), RomError> {
        if plan.kind == TransitionKind::EnterSpaceNeedle {
            self.dialogue = Some(Dialogue::exposition(rom, self.catalog.entrance.address.0)?);
            self.dialogue_target = Some(plan);
        } else {
            self.transition = Some(ActiveTransition {
                plan,
                stage: TransitionStage::FadeOut,
                ticks: 0,
                target_palette: self.scene.palette,
            });
        }
        self.blocked = false;
        Ok(())
    }
    fn apply_transition(&mut self, rom: &[u8], plan: TransitionPlan) -> Result<(), RomError> {
        // Prepare every fallible asset before replacing the active scene.
        let mut map = MapDescriptor::read(rom, plan.target_map)?;
        if plan.kind == TransitionKind::ReturnToExterior {
            map.raw = plan
                .exterior_return
                .ok_or(RomError("missing exterior return state"))?
                .descriptor;
        }
        let mut scene = MapScene::load(rom, map.id)?;
        let collision = Collision::load(rom, &map)?;
        scene.apply_collision_overflow(&collision.overflow_writes)?;
        let environment = Environment::load(rom, &map)?;
        let mut party = self.party.clone();
        for (slot, runner) in party.iter_mut().enumerate() {
            runner.set_position(plan.positions[slot].0, plan.positions[slot].1);
            runner.select_motion(false);
            if plan.kind == TransitionKind::Clinic {
                runner.set_word(0x1a, 0);
                runner.set_word(0x1e, 0);
                runner.clock = Default::default();
            }
            if crate::combat_runtime::action(runner) && runner.raw[0x4c] != 0 {
                runner.raw[0x3a..0x3e].copy_from_slice(&0xb45cu32.to_be_bytes());
                runner.set_word(0x1a, 0);
                runner.set_word(0x1e, 0);
                runner.clock = Default::default();
            }
        }
        let mut hud = Hud::load(rom, &mut scene)?;
        hud.select(rom, self.selected)?;
        for (slot, runner) in party.iter().enumerate() {
            runner.update_hud(rom, &mut scene, &mut hud, slot)?;
            let level = if (runner.raw[0x9c] as i8) > 0 {
                runner.raw[0x9c].min(10)
            } else {
                0
            };
            hud.condition(rom, slot, level, slot == self.selected)?;
        }
        let npcs = if plan.kind == TransitionKind::EnterSpaceNeedle
            && !party.iter().any(|r| r.template == 5)
        {
            npc::space_needle(rom)?
        } else {
            vec![]
        };
        self.saved_exterior = match plan.kind {
            TransitionKind::EnterSpaceNeedle => plan.exterior_return,
            TransitionKind::ReturnToExterior | TransitionKind::Clinic => None,
            TransitionKind::Exterior => self.saved_exterior,
        };
        self.npcs = npcs;
        self.combat.clear_map();
        if plan.kind == TransitionKind::Clinic {
            self.combat.status = "";
            self.targeting.enabled = false;
        }
        self.encounters.clear_map();
        self.map = map;
        self.scene = scene;
        self.collision = collision;
        self.environment = environment;
        self.targeting.invalidate(&mut party);
        self.party = party;
        self.hud = hud;
        self.camera = (plan.camera_fixed.0 >> 4, plan.camera_fixed.1 >> 4);
        self.sync_projections();
        self.completed_transitions += 1;
        self.pending_transition = None;
        self.blocked = false;
        Ok(())
    }
    fn advance_transition(&mut self, rom: &[u8]) -> Result<(), RomError> {
        let mut active = self.transition.expect("transition is active");
        active.ticks += 1;
        match active.stage {
            TransitionStage::FadeOut => {
                if active.ticks % 2 == 0 {
                    video::fade_out(&mut self.scene.palette);
                }
                if active.ticks == 34 {
                    self.apply_transition(rom, active.plan)?;
                    active.target_palette = self.scene.palette;
                    self.scene.palette = [0; 64];
                    active.stage = TransitionStage::Warmup;
                    active.ticks = 0;
                }
            }
            TransitionStage::Warmup => {
                for runner in self.party.iter_mut().chain(self.npcs.iter_mut()) {
                    crate::weapons::tick_cooldown(runner);
                }
                followers::warmup(rom, &self.collision, &mut self.party, self.selected)?;
                for runner in self.party.iter_mut().chain(self.npcs.iter_mut()) {
                    runner.tick_animation(rom)?;
                }
                self.environment.tick(rom, &mut self.scene)?;
                if active.ticks == 31 {
                    active.stage = TransitionStage::FadeIn;
                    active.ticks = 0;
                }
            }
            TransitionStage::FadeIn => {
                if active.ticks % 2 == 0 {
                    video::fade_in(&mut self.scene.palette, &active.target_palette);
                }
                if active.ticks == 34 {
                    self.transition = None;
                    return Ok(());
                }
            }
        }
        self.transition = Some(active);
        Ok(())
    }
    pub fn refresh_hud(&mut self, rom: &[u8]) -> Result<(), RomError> {
        self.hud = Hud::load(rom, &mut self.scene)?;
        self.hud.select(rom, self.selected)?;
        for (slot, runner) in self.party.iter().enumerate() {
            runner.update_hud(rom, &mut self.scene, &mut self.hud, slot)?;
            let condition = if (runner.raw[0x9c] as i8) > 0 {
                runner.raw[0x9c].min(10)
            } else {
                0
            };
            self.hud
                .condition(rom, slot, condition, slot == self.selected)?;
        }
        Ok(())
    }
    pub fn select(&mut self, rom: &[u8], slot: usize) -> Result<(), RomError> {
        if slot >= self.party.len() {
            return Err(RomError("party slot is inactive"));
        }
        if self.transition.is_some()
            || self.dialogue.is_some()
            || self.defeated
            || self.combat.damage.defeat_timer != 0
        {
            return Ok(());
        }
        if slot != self.selected && self.targeting.enabled {
            self.targeting.invalidate(&mut self.party);
        }
        self.selected = slot;
        self.combat.status = "";
        self.combat.launch.signals.controlled_busy = false;
        // $3992/$39a8 transfers the controlled-player bit. Without this, a
        // switched or newly hired runner could walk but could not use doors.
        for (i, runner) in self.party.iter_mut().enumerate() {
            runner.raw[0x10] = (runner.raw[0x10] & !2) | if i == slot { 2 } else { 0 };
        }
        self.pending_transition = None;
        self.refresh_hud(rom)
    }
    pub fn update(&mut self, rom: &[u8], buttons: u8) -> Result<(), RomError> {
        if self.defeated {
            self.input.update(buttons);
            if self.clinic_plan.is_some()
                && self.input.pressed & (crate::input::A | crate::input::START) != 0
            {
                let plan = self.clinic_plan.take().unwrap();
                self.dialogue = None;
                self.defeated = false;
                self.transition = Some(ActiveTransition {
                    plan,
                    stage: TransitionStage::FadeOut,
                    ticks: 0,
                    target_palette: self.scene.palette,
                });
            }
            return Ok(());
        }

        self.tick = self.tick.wrapping_add(1);
        self.input.update(buttons);
        if self.transition.is_some() {
            return self.advance_transition(rom);
        }
        if let Some(dialogue) = &mut self.dialogue {
            let pressed = self.input.pressed;
            if pressed & crate::input::LEFT != 0 {
                dialogue.change_page(-1)?;
            }
            if pressed & crate::input::RIGHT != 0 {
                dialogue.change_page(1)?;
            }
            if dialogue.choices.is_empty() {
                if pressed
                    & (crate::input::A | crate::input::B | crate::input::C | crate::input::START)
                    != 0
                {
                    self.dialogue = None;
                    if let Some(plan) = self.dialogue_target.take() {
                        self.transition = Some(ActiveTransition {
                            plan,
                            stage: TransitionStage::FadeOut,
                            ticks: 0,
                            target_palette: self.scene.palette,
                        });
                    }
                }
            } else {
                let count = dialogue.choices.len();
                if pressed & crate::input::UP != 0 {
                    dialogue.selected = (dialogue.selected + count - 1) % count;
                }
                if pressed & crate::input::DOWN != 0 {
                    dialogue.selected = (dialogue.selected + 1) % count;
                }
                let choice = if pressed & crate::input::B != 0 {
                    Some(1)
                } else if pressed & crate::input::C != 0 {
                    Some(2)
                } else if pressed & (crate::input::A | crate::input::START) != 0 {
                    Some(dialogue.selected)
                } else {
                    None
                };
                if let Some(slot) = choice {
                    self.follow_dialogue(rom, slot)?;
                }
            }
            return Ok(());
        }
        for runner in self.party.iter_mut().chain(self.npcs.iter_mut()) {
            crate::weapons::tick_cooldown(runner);
        }
        if self.input.pressed & crate::input::C != 0 {
            self.targeting.enabled = !self.targeting.enabled;
            self.targeting.invalidate(&mut self.party);
        }
        if self.targeting.enabled {
            let actors: Vec<_> = self.party.iter().chain(self.npcs.iter()).collect();
            self.targeting
                .lists
                .refresh(&actors, self.tick as u16, self.targeting.force_refresh);
            self.targeting.force_refresh = false;
            self.targeting
                .lists
                .acquire(&mut self.party[self.selected], &mut self.avoidance);
        }
        self.blocked = false;
        self.pending_transition = None;
        if self.npc_dialogue_cooldown > 0 {
            self.npc_dialogue_cooldown -= 1;
        }
        if self.map.id == 10
            && self.combat.damage.defeat_timer == 0
            && self.input.held & crate::input::A == 0
            && !self.combat.has_live()
            && self.saved_exterior.is_some()
            && self.npc_dialogue_cooldown == 0
            && self
                .npcs
                .iter()
                .any(|n| npc::trent_contact(&self.party[self.selected], n))
        {
            self.dialogue = Some(Dialogue::from_record(
                rom,
                self.catalog
                    .node(self.catalog.trent.root.0)
                    .ok_or(RomError("missing Trent conversation"))?,
                &self.catalog,
            )?);
            self.knowledge.runner_known |= 1 << self.catalog.trent.runner_index;
            self.npc_dialogue_cooldown = 60;
            return Ok(());
        }
        let mut serviced = false;
        for slot in 0..self.party.len() {
            crate::combat_runtime::prepare_actor(&mut self.party[slot]);
            if slot == self.selected {
                self.dispatch_weapon(rom)?;
            }
            if self.party[slot].raw[0x10] & 0x51 != 1 {
                continue;
            }
            if crate::combat_runtime::holds_movement(&self.party[slot])
                || (slot == self.selected && self.combat.player_attempt)
            {
                continue;
            }
            if slot != self.selected {
                let leader = self.party[self.selected].clone();
                if self.party[slot].raw[0x10] & 8 != 0 {
                    followers::schedule(
                        rom,
                        &self.collision,
                        &mut self.party[slot],
                        &leader,
                        &mut serviced,
                    )?;
                }
                avoidance::scan(
                    rom,
                    &mut self.party,
                    &mut self.npcs,
                    slot,
                    &mut self.avoidance,
                )?;
                if !avoidance::respond(
                    rom,
                    &self.collision,
                    &mut self.party[slot],
                    self.tick as u16,
                )? {
                    followers::move_one(rom, &self.collision, &mut self.party[slot])?;
                }
                continue;
            }
            let target = self.input.direction();
            if let Some(angle) = target {
                self.party[slot].set_word(0x34, angle);
            }
            self.party[slot].raw[0x3a..0x3e].copy_from_slice(
                &(if target.is_some() { 0xb574u32 } else { 0xb45c }).to_be_bytes(),
            );
            avoidance::scan(
                rom,
                &mut self.party,
                &mut self.npcs,
                slot,
                &mut self.avoidance,
            )?;
            let response = avoidance::respond_with_travel(
                rom,
                &self.collision,
                &mut self.party[slot],
                self.tick as u16,
                Some(avoidance::TravelScope {
                    map: self.map.id,
                    has_return: self.saved_exterior.is_some(),
                }),
            )?;
            if response.handled {
                if let Some(trigger) = response.trigger {
                    self.pending_transition = Some(trigger.code);
                    if let Some(plan) =
                        self.resolve_trigger(rom, trigger.code, trigger.orientation)?
                    {
                        self.start_transition(rom, plan)?;
                        return Ok(());
                    }
                }
                continue;
            }
            let heading = self.input.direction();
            let source_map = self.map.id;
            let camera_fixed_x = self.camera.0.wrapping_mul(16);
            let runner = &mut self.party[self.selected];
            let mut plan = None;
            let mut moved = false;
            if let Some(target) = heading {
                runner.turn_toward(target);
                let direction = (runner.word(0x2e).rotate_left(3) & 7) as usize;
                // Default player table selected by $b5ea for template flag bit 3.
                let base = 0xbec2 + direction * 4;
                let dx = i16::from_be_bytes(
                    rom.get(base..base + 2)
                        .ok_or(RomError("movement table missing"))?
                        .try_into()
                        .unwrap(),
                );
                let dy = i16::from_be_bytes(
                    rom.get(base + 2..base + 4)
                        .ok_or(RomError("movement table missing"))?
                        .try_into()
                        .unwrap(),
                );
                let (x, y) = runner.position();
                let candidate = (x.wrapping_add(dx as u16), y.wrapping_add(dy as u16));
                match self.collision.contact(candidate)? {
                    Contact::Clear => {
                        if self.collision.footprint_samples(candidate) == [0; 4] {
                            runner.raw[0xb0] &= !0x20;
                        }
                        runner.set_position(candidate.0, candidate.1);
                        moved = true;
                    }
                    Contact::Blocked => self.blocked = true,
                    // Stop rather than entering a door/mission without its unresolved script.
                    Contact::Trigger(code) => {
                        self.pending_transition = Some(code);
                        self.blocked = true;
                        if runner.raw[0x10] & 2 != 0
                            && self.collision.trigger_corners(candidate) >= 2
                        {
                            plan = TransitionPlan::exterior(rom, source_map, code, camera_fixed_x)?;
                            if plan.is_none() {
                                if let Some(saved) = self.saved_exterior {
                                    plan = TransitionPlan::leave_interior(source_map, code, saved);
                                } else {
                                    plan = TransitionPlan::space_needle(
                                        rom,
                                        &self.map,
                                        code,
                                        (camera_fixed_x, self.camera.1.wrapping_mul(16)),
                                        runner.position(),
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
            if heading.is_none() && self.collision.footprint_samples(runner.position()) == [0; 4] {
                runner.raw[0xb0] &= !0x20;
            }
            runner.select_motion(moved);
            runner.raw[0x3a..0x3e]
                .copy_from_slice(&(if moved { 0xb574u32 } else { 0xb45c }).to_be_bytes());
            runner.set_word(0x38, if moved { 1 } else { 0 });
            if heading.is_some() {
                runner.set_word(0xb8, if moved { 0x1000 } else { 0xffff });
                runner.set_word(0xca, 30);
            }
            if let Some(plan) = plan {
                self.start_transition(rom, plan)?;
                return Ok(());
            }
        }
        for index in 0..self.npcs.len() {
            crate::combat_runtime::prepare_actor(&mut self.npcs[index]);
            if self.encounters.finish_death(&self.npcs[index]) {
                let mut actors = self.combat_actors();
                self.combat.launch.end_combat(&mut actors);
                self.accept_combat_actors(&actors);
            }
            let mut actors = self.combat_actors();
            let managed = self.combat.npc(
                rom,
                &self.collision,
                &mut actors,
                self.npcs[index].word(0x3e),
                serviced,
                self.tick as u16,
                &mut self.avoidance,
            )?;
            if self.accept_combat_actors(&actors) {
                self.refresh_hud(rom)?;
            }
            let npc = &mut self.npcs[index];
            if npc.raw[0x10] & 0x41 == 1 && !crate::combat_runtime::holds_movement(npc) {
                let responded = avoidance::respond(rom, &self.collision, npc, self.tick as u16)?;
                if managed && !responded {
                    followers::move_npc(rom, &self.collision, npc)?;
                }
            }
        }
        self.dispatch_projectiles(rom)?;
        if crate::damage::tick_defeat(&mut self.combat.damage.defeat_timer) {
            self.begin_recovery(rom)?;
            return Ok(());
        }
        self.dispatch_encounters(rom)?;
        for runner in self.party.iter_mut().chain(self.npcs.iter_mut()) {
            if crate::combat_runtime::action(runner) {
                crate::combat_runtime::tick_animation(rom, runner)?;
            } else {
                runner.tick_animation(rom)?;
            }
        }
        self.environment.tick(rom, &mut self.scene)?;
        // Host camera keeps the controlled runner in the playable area. Original
        // look-ahead/hysteresis scheduling is still separate and not claimed here.
        let (x, y) = self.party[self.selected].position();
        let (px, py) = (x >> 4, y >> 4);
        let width = self.map.row_stride_bytes() as usize / 2 * 32;
        if px < self.camera.0.saturating_add(48) {
            self.camera.0 = px.saturating_sub(48);
        }
        if px > self.camera.0.saturating_add(224) {
            self.camera.0 = px.saturating_sub(224);
        }
        if py < self.camera.1.saturating_add(40) {
            self.camera.1 = py.saturating_sub(40);
        }
        if py > self.camera.1.saturating_add(184) {
            self.camera.1 = py.saturating_sub(184);
        }
        self.camera.0 = self.camera.0.min(width.saturating_sub(272) as u16);
        self.camera.1 = self
            .camera
            .1
            .min(self.collision.bounds[3].saturating_sub(224));
        self.sync_projections();
        Ok(())
    }
    fn begin_recovery(&mut self, rom: &[u8]) -> Result<(), RomError> {
        if let Some(recovery) = crate::clinic::prepare(
            &self.party,
            self.nuyen,
            self.recovery_flags,
            self.combat.launch.mode,
        ) {
            let record = crate::clinic::Record::load(rom)?;
            let mut notice = Dialogue::exposition(rom, record.node.0)?;
            notice.caption = "LITTLE CHIBA";
            let plan = TransitionPlan::clinic(rom, self.map.id, &record)?;
            // Validate destination assets before charging; apply_transition loads the same ROM again.
            let map = MapDescriptor::read(rom, plan.target_map)?;
            let mut scene = MapScene::load(rom, map.id)?;
            let collision = Collision::load(rom, &map)?;
            scene.apply_collision_overflow(&collision.overflow_writes)?;
            Environment::load(rom, &map)?;
            let mut hud = Hud::load(rom, &mut scene)?;
            recovery.actor.update_hud(rom, &mut scene, &mut hud, 0)?;
            self.party[0] = recovery.actor;
            self.nuyen = recovery.balance;
            self.dialogue = Some(notice);
            self.clinic_plan = Some(plan);
            self.defeat_notice = None;
            self.defeated = true;
            self.combat.status = "Medical recovery - Enter to continue";
            let mut actors = self.combat_actors();
            self.combat.launch.end_combat(&mut actors);
            self.accept_combat_actors(&actors);
        } else {
            self.defeat_notice = Some(Dialogue::defeat_notice(rom)?);
            self.defeated = true;
            self.combat.status = "Party defeated - R to restart";
        }
        Ok(())
    }
    pub fn selected_target(&self) -> Option<&Runner> {
        if !self.targeting.enabled {
            return None;
        }
        let id = combat::target(&self.party[self.selected])?;
        self.party
            .iter()
            .chain(self.npcs.iter())
            .find(|r| r.word(0x3e) == u16::from(id.0) && r.raw[0x10] & 0x41 == 1)
    }
    fn combat_actors(&self) -> Vec<Runner> {
        (0..10)
            .map(|slot| {
                self.party
                    .iter()
                    .chain(self.npcs.iter())
                    .find(|r| r.word(0x3e) == slot)
                    .cloned()
                    .unwrap_or_else(|| {
                        let mut r = Runner {
                            raw: [0; 256],
                            template: 0,
                            clock: Default::default(),
                        };
                        r.set_word(0x3e, slot);
                        r
                    })
            })
            .collect()
    }
    fn accept_combat_actors(&mut self, actors: &[Runner]) -> bool {
        let mut dirty = false;
        for r in self.party.iter_mut().chain(self.npcs.iter_mut()) {
            if let Some(updated) = actors.iter().find(|a| a.word(0x3e) == r.word(0x3e)) {
                if r.word(0x3e) < 3 {
                    dirty |= r.raw[0x4c..0x58] != updated.raw[0x4c..0x58]
                        || r.raw[0x9c] != updated.raw[0x9c];
                }
                *r = updated.clone();
            }
        }
        dirty
    }
    fn dispatch_weapon(&mut self, rom: &[u8]) -> Result<(), RomError> {
        let mut actors = self.combat_actors();
        self.combat.launch.map_id = self.map.id as u16;
        self.combat.player(
            rom,
            &mut actors,
            self.selected as u16,
            self.input,
            &mut self.avoidance,
        )?;
        if self.accept_combat_actors(&actors) {
            self.refresh_hud(rom)?;
        }
        Ok(())
    }
    fn dispatch_projectiles(&mut self, rom: &[u8]) -> Result<(), RomError> {
        let mut actors = self.combat_actors();
        self.combat
            .projectiles(rom, &self.collision, &mut actors, &mut self.avoidance)?;
        self.combat.launch.tick_alert(&mut actors, false);
        if self.accept_combat_actors(&actors) {
            self.refresh_hud(rom)?;
        }
        Ok(())
    }
    fn dispatch_encounters(&mut self, rom: &[u8]) -> Result<(), RomError> {
        let mut actors = self.combat_actors();
        let before = actors.iter().map(|a| a.raw[0x10]).collect::<Vec<_>>();
        self.encounters.update(
            rom,
            &self.map,
            (
                self.camera.0.wrapping_mul(16),
                self.camera.1.wrapping_mul(16),
            ),
            &self.collision,
            &mut actors,
            self.tick as u16,
            &mut self.combat.launch,
            &mut self.avoidance,
        )?;
        self.accept_combat_actors(&actors);
        for actor in actors
            .into_iter()
            .filter(|a| a.word(0x3e) >= 3 && a.raw[0x10] & 1 != 0)
        {
            if !self.npcs.iter().any(|a| a.word(0x3e) == actor.word(0x3e)) {
                self.npcs.push(actor);
            }
        }
        self.npcs.retain(|a| a.raw[0x10] & 1 != 0);
        self.npcs.sort_by_key(|a| a.word(0x3e));
        if self
            .combat_actors()
            .iter()
            .map(|a| a.raw[0x10])
            .collect::<Vec<_>>()
            != before
        {
            self.combat.actors_changed();
            self.targeting.force_refresh = true;
        }
        Ok(())
    }
    /// Independent, non-mutating diagnostic with cleared collision scratch.
    pub fn selected_fire_probe(&self, rom: &[u8]) -> Result<Option<combat::LineProbe>, RomError> {
        self.selected_target()
            .map(|target| {
                combat::line_of_fire(
                    rom,
                    &self.collision,
                    &self.party[self.selected],
                    target,
                    [0; 4],
                )
            })
            .transpose()
    }
    pub fn render(&mut self, rom: &[u8]) -> Result<Vec<u8>, RomError> {
        let projectiles = self.combat.sprites();
        let actors: Vec<&Runner> = self
            .party
            .iter()
            .chain(self.npcs.iter())
            .chain(projectiles.iter())
            .filter(|r| r.raw[0x10] & 1 != 0)
            .collect();
        let mut frames = Vec::with_capacity(actors.len());
        for runner in &actors {
            let seq = runner.sequence(rom)?;
            let index = (runner.clock.frame as usize).min(seq.frames.len() - 1);
            frames.push(ActorFrame::load(
                rom,
                seq.frames[index],
                &mut self.scene,
                runner.word(0x20),
                runner.palette(),
            )?);
        }
        let mut order: Vec<usize> = (0..actors.len()).collect();
        order.sort_by_key(|i| std::cmp::Reverse(actors[*i].position().1));
        let mut placed = Vec::new();
        for i in order {
            let runner = actors[i];
            if let Some((x, y)) = project(
                runner.position(),
                (
                    self.camera.0.wrapping_mul(16),
                    self.camera.1.wrapping_mul(16),
                ),
                runner.word(8) as i16,
                false,
            ) {
                placed.push(PlacedActor {
                    frame: &frames[i],
                    x,
                    y,
                });
            }
        }
        let mut pixels = scene::render(&self.scene, self.camera, &placed, Some(&self.hud))?;
        let target_color = match self.selected_fire_probe(rom)?.map(|probe| probe.outcome) {
            Some(combat::LineOutcome::Blocked) => [255, 64, 64],
            Some(combat::LineOutcome::OutsideModel) => [160, 160, 160],
            _ => [255, 224, 64],
        };
        // Native target brackets are a host UI aid, not a recovered VDP sprite.
        if self.dialogue.is_none() {
            if let Some(target) = self.selected_target() {
                if let Some((x, y)) = project(
                    target.position(),
                    (
                        self.camera.0.wrapping_mul(16),
                        self.camera.1.wrapping_mul(16),
                    ),
                    target.word(8) as i16,
                    false,
                ) {
                    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                        for d in 0..5 {
                            for (px, py) in [
                                (x + sx * 12 - sx * d, y - 16 + sy * 12),
                                (x + sx * 12, y - 16 + sy * 12 - sy * d),
                            ] {
                                if (0..272).contains(&px) && (0..224).contains(&py) {
                                    let at = (py as usize * 320 + px as usize) * 3;
                                    pixels[at..at + 3].copy_from_slice(&target_color);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(dialogue) = &self.dialogue {
            dialogue.render(rom, &mut pixels)?;
        }
        if let Some(notice) = &self.defeat_notice {
            notice.render(rom, &mut pixels)?;
        }
        Ok(pixels)
    }
}
