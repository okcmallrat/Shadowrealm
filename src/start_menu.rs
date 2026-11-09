//! Host opening menu; leaves a running game intact until replacement is ready.
use crate::{
    dialogue::{Choice, Dialogue},
    input::{self, Controller},
    newgame::{Archetype, Catalog},
    RomError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    NewGame(Archetype),
    Load(u8),
    Resume,
}
#[derive(Clone, Copy)]
enum Page {
    Root,
    Classes,
    Preview(Archetype),
    Slots,
    Notice,
}
pub struct Menu {
    pub dialogue: Dialogue,
    catalog: Catalog,
    page: Page,
    return_page: Page,
    input: Controller,
    resumable: bool,
    pending: bool,
}
fn choice(text: impl Into<String>, destination: usize) -> Choice {
    Choice {
        text_address: 0,
        text: text.into(),
        destination,
        available: true,
    }
}
impl Menu {
    pub fn new(rom: &[u8], resumable: bool) -> Result<Self, RomError> {
        let mut m = Self {
            dialogue: Dialogue::with_text(rom, 0, 0, String::new())?,
            catalog: Catalog::load(rom)?,
            page: Page::Root,
            return_page: Page::Root,
            input: Controller::default(),
            resumable,
            pending: false,
        };
        m.dialogue = m.build(rom, Page::Root)?;
        Ok(m)
    }
    fn build(&self, rom: &[u8], page: Page) -> Result<Dialogue, RomError> {
        let (caption, text, choices, selected) = match page {
            Page::Root => {
                let mut choices = vec![choice("New Game", 0), choice("Load Game", 1)];
                if self.resumable {
                    choices.push(choice("Resume", 2));
                }
                ("SHADOWREALM", "Choose how to begin.".into(), choices, 0)
            }
            Page::Classes => (
                "CHOOSE ARCHETYPE",
                "Choose Joshua's archetype.".into(),
                Archetype::ALL
                    .iter()
                    .enumerate()
                    .map(|(i, a)| choice(a.name(), i))
                    .chain([choice("Back", 3)])
                    .collect(),
                0,
            ),
            Page::Preview(a) => {
                let mut text = self.catalog.preview(rom, a)?;
                if self.resumable {
                    text.push_str("\n\nBeginning replaces unsaved progress.");
                }
                (
                    "NEW GAME",
                    text,
                    vec![choice("Begin new game", 0), choice("Back", 1)],
                    1,
                )
            }
            Page::Slots => (
                "LOAD GAME",
                "Choose a native checkpoint.".into(),
                vec![choice("Slot 1", 1), choice("Slot 2", 2), choice("Back", 0)],
                0,
            ),
            Page::Notice => return Err(RomError("notice requires text")),
        };
        let mut d = Dialogue::with_text(rom, 0, 0, text)?;
        d.caption = caption;
        d.choices = choices;
        d.selected = selected;
        Ok(d)
    }
    pub fn update(&mut self, rom: &[u8], held: u8) -> Result<Option<Request>, RomError> {
        self.input.update(held);
        if self.pending {
            return Ok(None);
        }
        let pressed = self.input.pressed;
        if pressed & input::LEFT != 0 {
            self.dialogue.change_page(-1)?;
        }
        if pressed & input::RIGHT != 0 {
            self.dialogue.change_page(1)?;
        }
        let count = self.dialogue.choices.len();
        if pressed & input::UP != 0 {
            self.dialogue.selected = (self.dialogue.selected + count - 1) % count;
        }
        if pressed & input::DOWN != 0 {
            self.dialogue.selected = (self.dialogue.selected + 1) % count;
        }
        if pressed & (input::A | input::START) == 0 {
            return Ok(None);
        }
        let destination = self.dialogue.choices[self.dialogue.selected].destination;
        let next = match self.page {
            Page::Root => match destination {
                0 => Page::Classes,
                1 => Page::Slots,
                _ => return Ok(Some(Request::Resume)),
            },
            Page::Classes => {
                if destination == 3 {
                    Page::Root
                } else {
                    Page::Preview(Archetype::ALL[destination])
                }
            }
            Page::Preview(a) => {
                if destination == 1 {
                    Page::Classes
                } else {
                    self.pending = true;
                    return Ok(Some(Request::NewGame(a)));
                }
            }
            Page::Slots => {
                if destination == 0 {
                    Page::Root
                } else {
                    self.pending = true;
                    return Ok(Some(Request::Load(destination as u8)));
                }
            }
            Page::Notice => self.return_page,
        };
        let d = self.build(rom, next)?;
        self.dialogue = d;
        self.page = next;
        Ok(None)
    }
    pub fn failed(&mut self, rom: &[u8], text: String) -> Result<(), RomError> {
        let mut d = Dialogue::with_text(rom, 0, 0, text)?;
        d.caption = "UNABLE TO CONTINUE";
        d.choices.push(choice("Back", 0));
        self.return_page = self.page;
        self.page = Page::Notice;
        self.dialogue = d;
        self.pending = false;
        Ok(())
    }
    pub fn missing_save_text(&self, rom: &[u8]) -> Result<String, RomError> {
        Ok(format!(
            "{}\n\nNo usable checkpoint or backup was found in this slot.",
            self.catalog.missing_save(rom)?
        ))
    }
    pub fn loaded_save_text(&self, rom: &[u8], backup: bool) -> Result<String, RomError> {
        Ok(format!(
            "{}{}",
            self.catalog.loaded_save(rom)?,
            if backup {
                "\n\nLoaded the backup checkpoint."
            } else {
                ""
            }
        ))
    }
    pub fn render(&self, rom: &[u8]) -> Result<Vec<u8>, RomError> {
        let mut pixels = vec![0; 320 * 224 * 3];
        self.dialogue.render(rom, &mut pixels)?;
        Ok(pixels)
    }
}
