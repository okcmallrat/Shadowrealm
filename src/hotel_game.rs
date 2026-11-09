use super::*;
use crate::advancement::{self, field, refusal_text, Refusal};
use crate::dialogue::Choice;

#[derive(Clone, Copy)]
pub(super) enum Modal {
    Front,
    Paid,
    Rested,
    Room,
    Checkout,
    Runners,
    Categories {
        runner: usize,
    },
    Stats {
        runner: usize,
        page: usize,
    },
    Confirm {
        runner: usize,
        page: usize,
        id: usize,
    },
    Notice {
        runner: usize,
        page: usize,
    },
}
fn choice(text: impl Into<String>, destination: usize) -> Choice {
    Choice {
        text_address: 0,
        text: text.into(),
        destination,
        available: true,
    }
}
fn panel(rom: &[u8], text: String, choices: Vec<Choice>) -> Result<Dialogue, RomError> {
    let mut d = Dialogue::with_text(rom, 0, 0, text)?;
    d.caption = "HOTEL / KARMA";
    d.choices = choices;
    Ok(d)
}
impl Game {
    pub(super) fn pay_room(&mut self, rom: &[u8], fee: u8) -> Result<(), RomError> {
        let payment = advancement::payment(self.nuyen, fee);
        let mut page = self
            .hotel_catalog
            .dialogue(rom, if payment.is_some() { 0xf5ab8 } else { 0xf5af8 })?;
        if self.map_id() == 2 {
            page.caption = "STOKER'S";
        }
        if payment.is_some() {
            page.choices.push(choice("Continue", 0));
            let mut party = self.party.clone();
            advancement::rest(&mut party);
            self.prepare_party_hud(rom, &party)?;
            Dialogue::read_text(rom, 0xc265e)?;
        }
        if let Some(balance) = payment {
            self.nuyen = balance;
        }
        self.dialogue = Some(page);
        self.hotel_dialogue = Some(if payment.is_some() {
            Modal::Paid
        } else {
            Modal::Front
        });
        Ok(())
    }
    fn hotel_room(&self, rom: &[u8]) -> Result<Dialogue, RomError> {
        panel(
            rom,
            "You are in your hotel room.".into(),
            vec![choice("Spend Karma", 0), choice("Check out", 1)],
        )
    }
    fn hotel_categories(&self, rom: &[u8], runner: usize) -> Result<Dialogue, RomError> {
        panel(
            rom,
            format!(
                "{}: {} Karma",
                self.party[runner].name(),
                self.party[runner].raw[0x8d]
            ),
            vec![
                choice("Attributes", 0),
                choice("Combat skills", 1),
                choice("Other skills", 2),
                choice("Back", 3),
            ],
        )
    }
    fn hotel_stats(
        &self,
        rom: &[u8],
        runner: usize,
        page: usize,
        party: &[Runner],
    ) -> Result<Dialogue, RomError> {
        let actor = &party[runner];
        let ids = match page {
            0 => 0..6,
            1 => 8..14,
            2 => 14..20,
            _ => return Err(RomError("invalid Karma page")),
        };
        let mut choices = vec![];
        for id in ids {
            let label = self.hotel_catalog.label(rom, id)?;
            // Validate each quote while preparing the menu; refusals remain
            // selectable so the original explanation can be shown.
            self.hotel_catalog.quote(rom, actor, id)?;
            choices.push(choice(format!("{} {}", label, actor.raw[field(id)?]), id));
        }
        choices.push(choice("Back", 20));
        panel(
            rom,
            format!("{}: {} Karma", actor.name(), actor.raw[0x8d]),
            choices,
        )
    }
    fn hotel_notice(&self, rom: &[u8], r: Refusal) -> Result<Dialogue, RomError> {
        let at = refusal_text(r);
        let mut d = panel(rom, Dialogue::read_text(rom, at)?, vec![choice("Back", 0)])?;
        d.text_address = at;
        Ok(d)
    }
    pub(super) fn follow_hotel(&mut self, rom: &[u8], slot: usize) -> Result<(), RomError> {
        let Some(ch) = self
            .dialogue
            .as_ref()
            .and_then(|d| d.choices.get(slot))
            .filter(|c| c.available)
        else {
            return Ok(());
        };
        let destination = ch.destination;
        let (page, modal) = match self.hotel_dialogue.unwrap() {
            Modal::Front => {
                if destination == 6 {
                    self.pay_room(rom, 75)?;
                    return Ok(());
                } else {
                    (self.hotel_catalog.dialogue(rom, destination)?, Modal::Front)
                }
            }
            Modal::Paid => {
                let mut party = self.party.clone();
                let medical = advancement::rest(&mut party);
                let mut text = String::new();
                for slot in medical {
                    text.push_str(&party[slot].name());
                    text.push_str(&Dialogue::read_text(rom, 0xc2643)?);
                    text.push('\n');
                }
                text.push_str(&Dialogue::read_text(rom, 0xc265e)?);
                let page = panel(rom, text, vec![choice("Continue", 0)])?;
                let (scene, hud) = self.prepare_party_hud(rom, &party)?;
                self.party = party;
                self.scene = scene;
                self.hud = hud;
                (page, Modal::Rested)
            }
            Modal::Rested => (self.hotel_room(rom)?, Modal::Room),
            Modal::Room => {
                if destination == 0 {
                    let mut choices = self
                        .party
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.raw[0x10] & 1 != 0)
                        .map(|(i, r)| choice(format!("{}: {} Karma", r.name(), r.raw[0x8d]), i))
                        .collect::<Vec<_>>();
                    choices.push(choice("Back", 3));
                    (
                        panel(rom, "Choose a runner.".into(), choices)?,
                        Modal::Runners,
                    )
                } else {
                    let mut d = panel(
                        rom,
                        "Check out of the hotel?".into(),
                        vec![choice("Yes", 0), choice("No", 1)],
                    )?;
                    d.selected = 1;
                    (d, Modal::Checkout)
                }
            }
            Modal::Checkout => {
                if destination == 0 {
                    self.dialogue = None;
                    self.hotel_dialogue = None;
                    return Ok(());
                }
                (self.hotel_room(rom)?, Modal::Room)
            }
            Modal::Runners => {
                if destination == 3 {
                    (self.hotel_room(rom)?, Modal::Room)
                } else {
                    if destination >= self.party.len() {
                        return Err(RomError("invalid hotel runner"));
                    }
                    (
                        self.hotel_categories(rom, destination)?,
                        Modal::Categories {
                            runner: destination,
                        },
                    )
                }
            }
            Modal::Categories { runner } => {
                if destination == 3 {
                    (self.hotel_room(rom)?, Modal::Room)
                } else {
                    (
                        self.hotel_stats(rom, runner, destination, &self.party)?,
                        Modal::Stats {
                            runner,
                            page: destination,
                        },
                    )
                }
            }
            Modal::Stats { runner, page } => {
                if destination == 20 {
                    (
                        self.hotel_categories(rom, runner)?,
                        Modal::Categories { runner },
                    )
                } else {
                    let q = self
                        .hotel_catalog
                        .quote(rom, &self.party[runner], destination)?;
                    let refusal = q.refusal.or_else(|| {
                        (self.party[runner].raw[0x8d] < q.cost)
                            .then_some(Refusal::InsufficientKarma)
                    });
                    if let Some(r) = refusal {
                        (self.hotel_notice(rom, r)?, Modal::Notice { runner, page })
                    } else {
                        let label = self.hotel_catalog.label(rom, destination)?;
                        let text = format!(
                            "{}\n{} {} -> {}\nCost: {} Karma\nKarma: {}\n{}",
                            self.party[runner].name(),
                            label,
                            self.party[runner].raw[field(destination)?],
                            q.next,
                            q.cost,
                            self.party[runner].raw[0x8d],
                            Dialogue::read_text(rom, 0xc217d)?
                        );
                        // Original $d53c starts on Yes.
                        (
                            panel(rom, text, vec![choice("Yes", 0), choice("No", 1)])?,
                            Modal::Confirm {
                                runner,
                                page,
                                id: destination,
                            },
                        )
                    }
                }
            }
            Modal::Notice { runner, page } => (
                self.hotel_stats(rom, runner, page, &self.party)?,
                Modal::Stats { runner, page },
            ),
            Modal::Confirm { runner, page, id } => {
                if destination != 0 {
                    (
                        self.hotel_stats(rom, runner, page, &self.party)?,
                        Modal::Stats { runner, page },
                    )
                } else {
                    let (actor, refusal) =
                        self.hotel_catalog
                            .advance(rom, &self.party[runner], id, true)?;
                    if let Some(r) = refusal {
                        (self.hotel_notice(rom, r)?, Modal::Notice { runner, page })
                    } else {
                        let mut party = self.party.clone();
                        party[runner] = actor;
                        let d = self.hotel_stats(rom, runner, page, &party)?;
                        let (scene, hud) = self.prepare_party_hud(rom, &party)?;
                        self.party = party;
                        self.scene = scene;
                        self.hud = hud;
                        (d, Modal::Stats { runner, page })
                    }
                }
            }
        };
        self.dialogue = Some(page);
        self.hotel_dialogue = Some(modal);
        Ok(())
    }
}
