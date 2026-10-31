//! Stateful exploration integration. Uses recovered templates, collision and
//! animation assets; story/combat/transition handlers are not yet reconstructed.
use crate::{
    actors::{project, ActorFrame},
    collision::{Collision, Contact},
    environment::Environment,
    hud::Hud,
    input::Controller,
    map_render::MapScene,
    maps::MapDescriptor,
    party::Runner,
    scene::{self, PlacedActor},
    RomError,
};
pub struct Game {
    pub scene: MapScene,
    pub party: Vec<Runner>,
    pub selected: usize,
    pub camera: (u16, u16),
    pub collision: Collision,
    pub environment: Environment,
    pub tick: u64,
    pub blocked: bool,
    pub pending_transition: Option<u8>,
    pub hud: Hud,
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
            selected: 0,
            camera,
            collision,
            environment,
            tick: 0,
            blocked: false,
            pending_transition: None,
            hud,
            map,
            input: Controller::default(),
        };
        result.refresh_hud(rom)?;
        Ok(result)
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
        self.selected = slot;
        self.pending_transition = None;
        self.refresh_hud(rom)
    }
    pub fn update(&mut self, rom: &[u8], buttons: u8) -> Result<(), RomError> {
        self.tick = self.tick.wrapping_add(1);
        self.input.update(buttons);
        self.blocked = false;
        self.pending_transition = None;
        let heading = self.input.direction();
        let runner = &mut self.party[self.selected];
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
                }
            }
        }
        runner.select_motion(moved);
        for runner in &mut self.party {
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
        self.camera.0 = self.camera.0.min(width.saturating_sub(320) as u16);
        self.camera.1 = self
            .camera
            .1
            .min(self.collision.bounds[3].saturating_sub(224));
        Ok(())
    }
    pub fn render(&mut self, rom: &[u8]) -> Result<Vec<u8>, RomError> {
        let mut frames = Vec::with_capacity(self.party.len());
        for runner in &self.party {
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
        let mut order: Vec<usize> = (0..self.party.len()).collect();
        order.sort_by_key(|i| std::cmp::Reverse(self.party[*i].position().1));
        let mut placed = Vec::new();
        for i in order {
            let runner = &self.party[i];
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
        scene::render(&self.scene, self.camera, &placed, Some(&self.hud))
    }
}
