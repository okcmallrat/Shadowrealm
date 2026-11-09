//! Native opening/load lifecycle, independent of the desktop window.
use crate::{
    game::Game,
    start_menu::{Menu, Request},
    RomError,
};
use std::path::Path;
pub struct Session {
    pub game: Option<Game>,
    pub menu: Option<Menu>,
    blocked_input: u8,
}
impl Session {
    pub fn new(rom: &[u8]) -> Result<Self, RomError> {
        Ok(Self {
            game: None,
            menu: Some(Menu::new(rom, false)?),
            blocked_input: 0,
        })
    }
    pub fn development(game: Game) -> Self {
        Self {
            game: Some(game),
            menu: None,
            blocked_input: 0,
        }
    }
    pub fn open_start(&mut self, rom: &[u8]) -> Result<(), RomError> {
        let menu = Menu::new(rom, self.game.is_some())?;
        self.menu = Some(menu);
        Ok(())
    }
    pub fn update(&mut self, rom: &[u8], held: u8, saves: &Path) -> Result<(), RomError> {
        if let Some(menu) = &mut self.menu {
            let Some(request) = menu.update(rom, held)? else {
                return Ok(());
            };
            let result: Result<Game, Box<dyn std::error::Error>> = match request {
                Request::NewGame(a) => Game::new_game(rom, a).map_err(Into::into),
                Request::Load(slot) => {
                    Game::load_slot(rom, saves, slot).and_then(|(mut next, backup)| {
                        next.file_notice(rom, menu.loaded_save_text(rom, backup)?)?;
                        Ok(next)
                    })
                }
                Request::Resume => {
                    self.menu = None;
                    self.blocked_input = held;
                    return Ok(());
                }
            };
            match result {
                Ok(next) => {
                    self.game = Some(next);
                    self.menu = None;
                    self.blocked_input = held;
                }
                Err(error) => {
                    eprintln!("Opening menu request failed: {error}");
                    let mut text = match request {
                        Request::Load(_) => menu.missing_save_text(rom)?,
                        _ => "Unable to prepare this new game.".into(),
                    };
                    if self.game.is_some() {
                        text.push_str("\nYour current game is retained.");
                    }
                    menu.failed(rom, text)?;
                }
            }
            return Ok(());
        }
        let buttons = held & !self.blocked_input;
        self.blocked_input &= held;
        if let Some(game) = &mut self.game {
            game.update(rom, buttons)?;
        }
        Ok(())
    }
    pub fn render(&mut self, rom: &[u8]) -> Result<Vec<u8>, RomError> {
        if let Some(menu) = &self.menu {
            menu.render(rom)
        } else {
            self.game
                .as_mut()
                .ok_or(RomError("session has no game"))?
                .render(rom)
        }
    }
}
