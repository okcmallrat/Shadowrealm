use super::*;
use crate::{
    archives::Archives,
    saves::{self, Native, Payload, COORDINATES, GLOBAL_BYTES, GLOBAL_OFFSET},
};
impl Game {
    /// Original save/load gates plus conservative native transient-state guards.
    pub fn save_ready(&self) -> bool {
        self.map.id <= 5
            && self.saved_exterior.is_none()
            && self.dialogue.is_none()
            && self.transition.is_none()
            && !self.defeated
            && self.combat.launch.engaged == 0
            && !self.combat.has_live()
            && self.encounters.count == 0
            && self.npcs.iter().all(|r| r.raw[0x10] & 0x41 != 1)
            && !self.party.is_empty()
            && self.party.len() <= 3
            && self.party[0].template <= 2
            && self.party.iter().all(|r| {
                r.raw[0x10] & 0x51 == 1 && r.raw[0x4c] > 0 && !crate::combat_runtime::action(r)
            })
    }
    pub fn save_bytes(&self, rom: &[u8]) -> Result<Vec<u8>, RomError> {
        if !self.save_ready() {
            return Err(RomError(
                "save needs a safe city street outside combat or recovery",
            ));
        }
        saves::validate_references(rom)?;
        let mut archive = self.archives.clone();
        archive.snapshot(&self.party)?;
        let mut records = [[0; 74]; 13];
        records[..11].copy_from_slice(&archive.records);
        let mut global = [0u8; GLOBAL_BYTES];
        let books = self
            .combat
            .spellbooks
            .as_ref()
            .ok_or(RomError("save spellbooks are missing"))?;
        for (i, record) in books.records.iter().enumerate() {
            global[i * 34..(i + 1) * 34].copy_from_slice(record);
        }
        let offset = |address: usize| address - 0xfff9fa;
        global[offset(0xfffbec)..offset(0xfffbec) + 10].copy_from_slice(&self.jobs.raw);
        global[offset(0xfffbf8)] = self.map.id as u8;
        saves::put_long(&mut global, offset(0xfffbfe), self.nuyen);
        saves::put_long(
            &mut global,
            offset(0xfffc04),
            0xff0100 + self.selected as u32 * 256,
        );
        global[offset(0xfffc0c)..offset(0xfffc0e)].fill(255);
        let mut identities = std::collections::HashSet::new();
        for (slot, runner) in self.party.iter().enumerate().skip(1) {
            let id = Archives::index(runner)?;
            if id == 0 || !identities.insert(id) {
                return Err(RomError("save has duplicate or invalid companions"));
            }
            global[offset(0xfffc0c) + slot - 1] = id as u8;
        }
        global[offset(0xfffc12)..offset(0xfffc1c)].copy_from_slice(&archive.opinions);
        global[offset(0xfffc1c)] = self.jobs.flags;
        saves::put_long(&mut global, offset(0xfffc1e), self.recovery_flags);
        saves::put_long(&mut global, offset(0xfffc22), self.jobs.carried);
        global[offset(0xfffc27)] = self.story.inquiries;
        saves::put_word(&mut global, offset(0xfffc2a), self.knowledge.runner_known);
        saves::put_long(&mut global, offset(0xfffc2c), self.knowledge.contacts);
        saves::put_long(&mut global, offset(0xfffc30), self.story.leads);
        let mut positions = [(0, 0); 3];
        for (slot, r) in self.party.iter().enumerate() {
            positions[slot] = r.position();
        }
        let image = saves::image(
            rom,
            &records,
            &global,
            self.map.id as u16,
            (
                self.camera.0.wrapping_mul(16),
                self.camera.1.wrapping_mul(16),
            ),
            positions,
        )?;
        Payload::new(
            image,
            Native {
                hero_template: self.party[0].template,
                tick: self.tick,
                rng: self.avoidance.seed,
                job_base_pay: self.jobs.base_pay,
                job_accepted: self.jobs.accepted,
                completed_jobs: self.jobs.completed,
            },
        )
        .encode()
    }
    /// $594e0/$5966e rebuild persistent party from compact archives. Prepare all
    /// assets, records and a rendered frame before returning the replacement.
    pub fn from_save_bytes(rom: &[u8], bytes: &[u8]) -> Result<Self, RomError> {
        saves::validate_references(rom)?;
        let parsed = crate::Rom::parse(rom)?;
        if rom.len() != 0x200000
            || parsed.checksum() != 0x6baf
            || parsed.stored_checksum() != 0x6baf
            || rom.get(0x180..0x18e) != Some(b"GM MK-1352 -00")
        {
            return Err(RomError("save requires the supported ROM revision"));
        }
        let payload = Payload::decode(bytes)?;
        let image = payload.validate()?;
        let global = &image[GLOBAL_OFFSET..COORDINATES];
        let offset = |address: usize| address - 0xfff9fa;
        let map_id = usize::from(saves::word(&image, COORDINATES));
        if map_id > 5
            || global[offset(0xfffbf8)] != map_id as u8
            || saves::long(global, offset(0xfffc3c)) != 0
        {
            return Err(RomError(
                "save location or script state is outside native support",
            ));
        }
        let mut templates = vec![payload.native.hero_template];
        let mut ids = std::collections::HashSet::new();
        let mut absent = false;
        for &id in &global[offset(0xfffc0c)..offset(0xfffc0e)] {
            if id == 255 {
                absent = true;
                continue;
            }
            if absent || !(1..=10).contains(&id) || !ids.insert(id) {
                return Err(RomError("save companion identities are invalid"));
            }
            templates.push(usize::from(id) + 2);
        }
        let mut next = Self::load(rom, map_id, &templates)?;
        for i in 0..11 {
            next.archives.records[i].copy_from_slice(&image[20 + i * 74..20 + (i + 1) * 74]);
        }
        next.archives
            .opinions
            .copy_from_slice(&global[offset(0xfffc12)..offset(0xfffc1c)]);
        for (slot, r) in next.party.iter_mut().enumerate() {
            next.archives.restore(r)?;
            if r.raw[0x4c] == 0 {
                return Err(RomError("save contains an unresolved party recovery"));
            }
            r.set_position(
                saves::word(&image, COORDINATES + 6 + slot * 4),
                saves::word(&image, COORDINATES + 8 + slot * 4),
            );
            r.raw[0x10] = if slot == 0 { 0x0f } else { 0x0d };
            r.select_motion(false);
            let (x, y) = r.position();
            let (x, y) = (x >> 4, y >> 4);
            if x < next.collision.bounds[0]
                || y < next.collision.bounds[1]
                || x >= next.collision.bounds[2]
                || y >= next.collision.bounds[3]
            {
                return Err(RomError("saved party position exceeds the map"));
            }
            next.collision.contact(r.position())?;
        }
        next.camera = (
            saves::word(&image, COORDINATES + 2) >> 4,
            saves::word(&image, COORDINATES + 4) >> 4,
        );
        if saves::word(&image, COORDINATES + 2) & 15 != 0
            || saves::word(&image, COORDINATES + 4) & 15 != 0
        {
            return Err(RomError("saved camera precision is outside native support"));
        }
        let portrait = match next.party[0].raw[0x9b] {
            2 => 0x720fau32,
            3 => 0x7223a,
            _ => 0x7219a,
        };
        next.party[0].raw[0x50..0x54].copy_from_slice(&portrait.to_be_bytes());
        let maximum = (
            (usize::from(next.map.row_stride_bytes()) / 2 * 32).saturating_sub(272) as u16,
            next.collision.bounds[3].saturating_sub(224),
        );
        if next.camera.0 > maximum.0 || next.camera.1 > maximum.1 {
            return Err(RomError("saved camera exceeds the map"));
        }
        next.nuyen = saves::long(global, offset(0xfffbfe));
        next.recovery_flags = saves::long(global, offset(0xfffc1e));
        next.jobs
            .raw
            .copy_from_slice(&global[offset(0xfffbec)..offset(0xfffbec) + 10]);
        next.jobs.flags = global[offset(0xfffc1c)];
        next.jobs.carried = saves::long(global, offset(0xfffc22));
        next.jobs.base_pay = payload.native.job_base_pay;
        next.jobs.accepted = payload.native.job_accepted;
        next.jobs.completed = payload.native.completed_jobs;
        if next.jobs.active()
            && (!next.jobs.courier()
                || !next.jobs.accepted
                || next.jobs.raw[2] != 2
                || next.jobs.raw[5] != 2
                || ![0, 1, 2, 3, 4, 5, 6, 7, 9, 11].contains(&next.jobs.raw[3])
                || ![0, 1, 2, 3, 4, 5, 6, 7, 9, 11].contains(&next.jobs.raw[6]))
        {
            return Err(RomError("saved job is outside supported courier state"));
        }
        next.story.inquiries = global[offset(0xfffc27)];
        next.story.leads = saves::long(global, offset(0xfffc30));
        next.knowledge.runner_known = saves::word(global, offset(0xfffc2a));
        next.knowledge.contacts = saves::long(global, offset(0xfffc2c));
        let books = next.combat.spellbooks.as_mut().unwrap();
        for (i, record) in books.records.iter_mut().enumerate() {
            record.copy_from_slice(&global[i * 34..(i + 1) * 34]);
        }
        for i in 0..5 {
            books.known(i)?;
        }
        // Validate even inactive archives before they can later be rehired.
        let mut validation_scene = next.scene.clone();
        let mut validation_hud = Hud::load(rom, &mut validation_scene)?;
        for i in 0..11 {
            let mut actor = Runner::load(
                rom,
                if i == 0 { templates[0] } else { i + 2 },
                if i == 0 { 0 } else { 1 },
            )?;
            next.archives.restore(&mut actor)?;
            if actor.word(0xa0) >= 5
                || actor.raw[0x66..0x6e].iter().any(|&id| id > 69)
                || (actor.raw[0x56] < 8
                    && actor.raw[0x57] != 0
                    && actor.raw[0x66 + usize::from(actor.raw[0x56])] != actor.raw[0x57])
            {
                return Err(RomError(
                    "saved equipment or spellbook reference is invalid",
                ));
            }
            actor.select_motion(false);
            actor.sequence(rom)?;
            actor.update_hud(
                rom,
                &mut validation_scene,
                &mut validation_hud,
                if i == 0 { 0 } else { 1 },
            )?;
        }
        next.tick = payload.native.tick;
        next.avoidance.seed = payload.native.rng;
        next.sync_projections();
        next.refresh_hud(rom)?;
        next.render(rom)?;
        Ok(next)
    }
    pub fn restore_save_bytes(&mut self, rom: &[u8], bytes: &[u8]) -> Result<(), RomError> {
        let next = Self::from_save_bytes(rom, bytes)?;
        *self = next;
        Ok(())
    }
    pub fn write_save_slot(
        &self,
        rom: &[u8],
        directory: &std::path::Path,
        slot: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.save_bytes(rom)?;
        let path = saves::slot_path(directory, slot)?;
        saves::write_atomic_checked(&path, &bytes, |data| {
            Self::from_save_bytes(rom, data).is_ok()
        })?;
        Ok(())
    }
    pub fn load_slot(
        rom: &[u8],
        directory: &std::path::Path,
        slot: u8,
    ) -> Result<(Self, bool), Box<dyn std::error::Error>> {
        let path = saves::slot_path(directory, slot)?;
        let attempt = |p: &std::path::Path| -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self::from_save_bytes(rom, &saves::read_bounded(p)?)?)
        };
        match attempt(&path) {
            Ok(game) => Ok((game, false)),
            Err(primary) => match attempt(&saves::backup_path(&path)) {
                Ok(game) => Ok((game, true)),
                Err(_) => Err(primary),
            },
        }
    }
}
