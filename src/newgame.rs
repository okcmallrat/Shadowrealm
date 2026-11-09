//! Verified new-game references. The opening UI is native host presentation.
use crate::{
    dialogue::Dialogue,
    dialogue_data::{long, Address},
    party::Runner,
    RomError,
};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    Samurai,
    Decker,
    GatorShaman,
}
impl Archetype {
    pub const ALL: [Self; 3] = [Self::Samurai, Self::Decker, Self::GatorShaman];
    pub fn template(self) -> usize {
        match self {
            Self::Samurai => 0,
            Self::Decker => 1,
            Self::GatorShaman => 2,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Samurai => "Samurai",
            Self::Decker => "Decker",
            Self::GatorShaman => "Gator Shaman",
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassRecord {
    archetype: Archetype,
    template: usize,
    class_byte: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    scope: String,
    menu: Address,
    class_selection: Address,
    initialize_archives: Address,
    initialize_hero: Address,
    initialize_globals: Address,
    initialize_jobs: Address,
    initialize_spellbooks: Address,
    start_text: Address,
    class_text: Address,
    slot_text: Address,
    missing_save: Address,
    loaded_save: Address,
    templates: Address,
    initial_map: usize,
    initial_lead: u32,
    archetypes: Vec<ClassRecord>,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/newgame/references.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid new-game JSON"))?;
        if c.schema_version != 1
            || c.scope.is_empty()
            || (c.initial_map, c.initial_lead) != (2, 1)
            || [
                c.menu.0,
                c.class_selection.0,
                c.initialize_archives.0,
                c.initialize_hero.0,
                c.initialize_globals.0,
                c.initialize_jobs.0,
                c.initialize_spellbooks.0,
                c.start_text.0,
                c.class_text.0,
                c.slot_text.0,
                c.missing_save.0,
                c.loaded_save.0,
                c.templates.0,
            ] != [
                0x59092, 0x59248, 0x59318, 0x42b0, 0x7fc, 0x1e68c, 0x566a2, 0x1b6498, 0x1b64cf,
                0x1b6518, 0x1b6527, 0x1b6577, 0x7291a,
            ]
            || c.archetypes
                .iter()
                .map(|a| (a.archetype, a.template, a.class_byte))
                .collect::<Vec<_>>()
                != [
                    (Archetype::Samurai, 0, 2),
                    (Archetype::Decker, 1, 3),
                    (Archetype::GatorShaman, 2, 1),
                ]
        {
            return Err(RomError("new-game references outside verified scope"));
        }
        if long(rom, 0x850)? != 1
            || long(rom, 0x854)? != 0xfffc30
            || rom.get(0x858..0x868)
                != Some(&[
                    0x33, 0xfc, 0, 2, 0, 0xff, 0xdd, 0xc4, 0x13, 0xfc, 0, 2, 0, 0xff, 0xfb, 0xf8,
                ])
            || long(rom, 0x42b8)? != c.templates.0
        {
            return Err(RomError("new-game bootstrap differs from ROM"));
        }
        if crate::hiring::initial_balance(rom)? != 20 {
            return Err(RomError("new-game starting funds differ from ROM"));
        }
        let start = Dialogue::read_text(rom, c.start_text.0)?;
        let classes = Dialogue::read_text(rom, c.class_text.0)?;
        if !start.contains("New Game")
            || !start.contains("Load Game")
            || !classes.contains("Samurai")
            || !classes.contains("Decker")
            || !classes.contains("Shaman")
        {
            return Err(RomError("new-game labels differ from ROM"));
        }
        for at in [c.slot_text.0, c.missing_save.0, c.loaded_save.0] {
            Dialogue::read_text(rom, at)?;
        }
        for a in &c.archetypes {
            if rom.get(c.templates.0 + a.template * 256 + 0x9b) != Some(&a.class_byte) {
                return Err(RomError("archetype class differs from ROM"));
            }
        }
        Ok(c)
    }
    pub fn map(&self) -> usize {
        self.initial_map
    }
    pub fn lead(&self) -> u32 {
        self.initial_lead
    }
    pub fn preview(&self, rom: &[u8], a: Archetype) -> Result<String, RomError> {
        let r = Runner::load(rom, a.template(), 0)?;
        let mut text = format!(
            "{}\nBody {}  Quickness {}\nStrength {}  Charisma {}\nIntelligence {}  Willpower {}\n",
            a.name(),
            r.raw[0x77],
            r.raw[0x78],
            r.raw[0x79],
            r.raw[0x7a],
            r.raw[0x7b],
            r.raw[0x7c]
        );
        text.push_str("\nStart alone in Redmond.\n20 nuyen. No hired runners.");
        Ok(text)
    }
    pub fn missing_save(&self, rom: &[u8]) -> Result<String, RomError> {
        Dialogue::read_text(rom, self.missing_save.0)
    }
    pub fn loaded_save(&self, rom: &[u8]) -> Result<String, RomError> {
        Dialogue::read_text(rom, self.loaded_save.0)
    }
}
