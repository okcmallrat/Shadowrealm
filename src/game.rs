//! Stateful exploration integration. Uses recovered templates, collision and
//! animation assets and traced area transitions. General story/combat handlers
//! are not yet reconstructed.
use crate::{
    actors::{project, ActorFrame},
    collision::{Collision, Contact},
    dialogue::Dialogue,
    dialogue_data::{Catalog, Knowledge},
    environment::Environment,
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
    catalog: Catalog,
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
            let offset = 0x42 + slot * 4;
            runner.set_position(
                u16::from_be_bytes([map.raw[offset], map.raw[offset + 1]]),
                u16::from_be_bytes([map.raw[offset + 2], map.raw[offset + 3]]),
            );
            runner.set_word(0xca, 30);
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
        };
        result.refresh_hud(rom)?;
        Ok(result)
    }
    pub fn map_id(&self) -> usize {
        self.map.id
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
            TransitionKind::ReturnToExterior => None,
            TransitionKind::Exterior => self.saved_exterior,
        };
        self.npcs = npcs;
        self.map = map;
        self.scene = scene;
        self.collision = collision;
        self.environment = environment;
        self.party = party;
        self.hud = hud;
        self.camera = (plan.camera_fixed.0 >> 4, plan.camera_fixed.1 >> 4);
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
        if self.transition.is_some() || self.dialogue.is_some() {
            return Ok(());
        }
        self.selected = slot;
        self.pending_transition = None;
        self.refresh_hud(rom)
    }
    pub fn update(&mut self, rom: &[u8], buttons: u8) -> Result<(), RomError> {
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
                    if let Some(next) =
                        dialogue.follow(rom, &self.catalog, slot, &mut self.knowledge)?
                    {
                        self.dialogue = Some(next);
                    }
                }
            }
            return Ok(());
        }
        self.blocked = false;
        self.pending_transition = None;
        if self.npc_dialogue_cooldown > 0 {
            self.npc_dialogue_cooldown -= 1;
        }
        if self.map.id == 10
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
                    runner.set_position(candidate.0, candidate.1);
                    moved = true;
                }
                Contact::Blocked => self.blocked = true,
                // Stop rather than entering a door/mission without its unresolved script.
                Contact::Trigger(code) => {
                    self.pending_transition = Some(code);
                    self.blocked = true;
                    if runner.raw[0x10] & 2 != 0 && self.collision.trigger_corners(candidate) >= 2 {
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
        runner.select_motion(moved);
        if let Some(plan) = plan {
            if plan.kind == TransitionKind::EnterSpaceNeedle {
                self.dialogue = Some(Dialogue::exposition(rom, self.catalog.entrance.address.0)?);
                self.dialogue_target = Some(plan);
                self.blocked = false;
                return Ok(());
            }
            self.transition = Some(ActiveTransition {
                plan,
                stage: TransitionStage::FadeOut,
                ticks: 0,
                target_palette: self.scene.palette,
            });
            self.blocked = false;
            return Ok(());
        }
        for runner in self.party.iter_mut().chain(self.npcs.iter_mut()) {
            runner.tick_animation(rom)?;
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
        Ok(())
    }
    pub fn render(&mut self, rom: &[u8]) -> Result<Vec<u8>, RomError> {
        let actors: Vec<&Runner> = self.party.iter().chain(self.npcs.iter()).collect();
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
        if let Some(dialogue) = &self.dialogue {
            dialogue.render(rom, &mut pixels)?;
        }
        Ok(pixels)
    }
}
