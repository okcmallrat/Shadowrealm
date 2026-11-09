//! First Stoker objective and taxi decisions. Rust executes recovered rules;
//! the typed catalog contains only bounded ROM references.
use crate::{
    dialogue::{Choice, Dialogue},
    dialogue_data::{long, Address, ChoiceRecord, NodeRecord},
    party::Runner,
    RomError,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Action {
    id: usize,
    routine: Address,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Offer {
    address: Address,
    choices: Vec<ChoiceRecord>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    scope: String,
    selector: Address,
    initial_selector: Address,
    district_records: Address,
    taxi_gate: Address,
    taxi_arrival: Address,
    actions: Vec<Action>,
    nodes: Vec<NodeRecord>,
    taxi_offer: Offer,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/dialogue/stoker-taxi.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self =
            serde_json::from_str(json).map_err(|_| RomError("invalid progression JSON"))?;
        let addresses = [
            0xf4afc, 0xf4b10, 0xf4aa0, 0xf4ad4, 0xf4b2c, 0xf4e8c, 0xf5ab0, 0xf487c, 0xf4890,
            0xf48a4, 0xf48b8, 0xf48cc, 0xf48e0, 0xf48f4, 0xf4908, 0xfe5d4, 0xfe600, 0xfe608,
            0xfe610, 0xfe618, 0xfe620, 0xfe628, 0xfe630, 0xf4ab4, 0xf4acc,
        ];
        let actions = [
            (5, 0x213fa),
            (48, 0x213ca),
            (49, 0x213e2),
            (33, 0x219b6),
            (34, 0x219c8),
            (14, 0x2076a),
            (15, 0x208a8),
            (233, 0x249c4),
            (6, 0x21458),
        ];
        if c.schema_version != 1
            || c.scope.is_empty()
            || [
                c.selector.0,
                c.initial_selector.0,
                c.district_records.0,
                c.taxi_gate.0,
                c.taxi_arrival.0,
            ] != [0xd445d, 0xd4456, 0xd4464, 0x20714, 0x136de]
            || c.nodes.iter().map(|n| n.address.0).collect::<Vec<_>>() != addresses
            || c.actions
                .iter()
                .map(|a| (a.id, a.routine.0))
                .collect::<Vec<_>>()
                != actions
        {
            return Err(RomError("progression references outside verified scope"));
        }
        for a in &c.actions {
            if long(rom, 0x12ccb4 + a.id * 4)? != a.routine.0 {
                return Err(RomError("progression action differs from ROM"));
            }
        }
        if c.taxi_offer.address.0 != 0xfe5ec
            || c.taxi_offer
                .choices
                .iter()
                .map(|ch| (ch.text.0, ch.destination.0))
                .collect::<Vec<_>>()
                != [(0xfe545, 15), (0xfe559, 0xfe600)]
            || long(rom, 0xfe5ec + 8)? != 0
        {
            return Err(RomError("taxi offer outside verified scope"));
        }
        for (i, ch) in c.taxi_offer.choices.iter().enumerate() {
            if long(rom, 0xfe5ec + i * 4)? != ch.text.0
                || long(rom, 0xfe5ec + 12 + i * 4)? != ch.destination.0
            {
                return Err(RomError("taxi offer differs from ROM"));
            }
        }
        for n in &c.nodes {
            if long(rom, n.address.0)? != n.text.0 || n.choices.len() > 3 {
                return Err(RomError("progression node differs from ROM"));
            }
            Dialogue::read_text(rom, n.text.0)?;
            if n.choices.is_empty() {
                if long(rom, n.address.0 + 4)? != 0xffffffff {
                    return Err(RomError("progression terminal differs from ROM"));
                }
            } else {
                for (i, ch) in n.choices.iter().enumerate() {
                    if long(rom, n.address.0 + 4 + i * 4)? != ch.text.0
                        || long(rom, n.address.0 + 16 + i * 4)? != ch.destination.0
                    {
                        return Err(RomError("progression choice differs from ROM"));
                    }
                }
                if n.choices.len() < 3 && long(rom, n.address.0 + 4 + n.choices.len() * 4)? != 0 {
                    return Err(RomError("progression JSON omits choices"));
                }
            }
        }
        if rom.get(c.selector.0..c.selector.0 + 7) != Some(&[6, 4, 2, 5, 0, 3, 1])
            || rom.get(c.initial_selector.0..c.initial_selector.0 + 7)
                != Some(&[4, 6, 2, 5, 1, 3, 0])
        {
            return Err(RomError("taxi selectors differ from ROM"));
        }
        Ok(c)
    }
    pub fn dialogue(&self, rom: &[u8], address: usize) -> Result<Dialogue, RomError> {
        let n = self
            .nodes
            .iter()
            .find(|n| n.address.0 == address)
            .ok_or(RomError("progression node unavailable"))?;
        let mut d =
            Dialogue::with_text(rom, address, n.text.0, Dialogue::read_text(rom, n.text.0)?)?;
        d.caption = "STOKER'S";
        for ch in &n.choices {
            d.choices.push(Choice {
                text_address: ch.text.0,
                text: Dialogue::read_text(rom, ch.text.0)?,
                destination: ch.destination.0,
                available: self.actions.iter().any(|a| a.id == ch.destination.0)
                    || self.nodes.iter().any(|n| n.address.0 == ch.destination.0),
            });
        }
        Ok(d)
    }
    pub fn district(&self, rom: &[u8], cursor: usize) -> Result<(usize, String, u8), RomError> {
        if cursor >= 7 {
            return Err(RomError("taxi cursor outside menu"));
        }
        let district = usize::from(
            *rom.get(self.selector.0 + cursor)
                .ok_or(RomError("taxi selector missing"))?,
        );
        let at = self.district_records.0 + district * 30;
        Ok((
            district,
            Dialogue::read_text(rom, at + 10)?.trim().to_string(),
            *rom.get(at + 29).ok_or(RomError("taxi fare missing"))?,
        ))
    }
    pub fn taxi_menu(
        &self,
        rom: &[u8],
        current: usize,
        balance: u32,
    ) -> Result<Dialogue, RomError> {
        if current > 6 {
            return Err(RomError("taxi requires an exterior district"));
        }
        let mut d = Dialogue::with_text(rom, 0, 0, "Call a taxi. Choose your destination.".into())?;
        d.caption = "CALL TAXI";
        d.funds = Some(balance);
        d.selected = usize::from(rom[self.initial_selector.0 + current]);
        for cursor in 0..7 {
            let (_, name, _) = self.district(rom, cursor)?;
            d.choices.push(Choice {
                text_address: 0,
                text: name,
                destination: cursor,
                available: true,
            });
        }
        d.choices.push(Choice {
            text_address: 0,
            text: "Cancel".into(),
            destination: 7,
            available: true,
        });
        Ok(d)
    }
    pub fn taxi_quote(
        &self,
        rom: &[u8],
        cursor: usize,
        current: usize,
        flags: u32,
        party: &[Runner],
        carried: u32,
    ) -> Result<(Dialogue, Option<u8>), RomError> {
        let (target, name, fare) = self.district(rom, cursor)?;
        let node = if flags & 1 == 0 {
            0xfe620
        } else if target == current {
            0xfe628
        } else {
            0xfe5d4
        };
        let mut d = self.dialogue(rom, node)?;
        d.caption = "TAXI";
        if node != 0xfe5d4 {
            return Ok((d, None));
        }
        // $2076a checks Joshua's armband before building a paid quote.
        if party.first().is_some_and(|r| r.raw[0x8f] & 0x20 != 0) {
            return Ok((
                self.taxi_notice(
                    rom,
                    if target == 6 && carried & 2 == 0 {
                        0xfe618
                    } else {
                        0xfe630
                    },
                )?,
                Some(0),
            ));
        }
        let opening = Dialogue::read_text(rom, 0xfe2b5)?;
        let comment = Dialogue::read_text(
            rom,
            match fare {
                20 => 0x103c22,
                50 => 0xfe2d1,
                _ => 0xfe2ea,
            },
        )?;
        let cost = Dialogue::read_text(rom, 0xfe312)?.replacen("   $", &format!("{fare}$"), 1);
        d.text = format!("{opening} {name}?  {comment}\n\n{cost}");
        d.choices.clear();
        for ch in &self.taxi_offer.choices {
            d.choices.push(Choice {
                text_address: ch.text.0,
                text: Dialogue::read_text(rom, ch.text.0)?,
                destination: ch.destination.0,
                available: true,
            });
        }
        Ok((d, Some(fare)))
    }
    pub fn taxi_notice(&self, rom: &[u8], node: usize) -> Result<Dialogue, RomError> {
        let mut d = self.dialogue(rom, node)?;
        d.caption = "TAXI";
        Ok(d)
    }
}

/// Additional recovered fields; the shared story/DocWagon word stays in Game.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Story {
    pub leads: u32,
    pub inquiries: u8,
}
impl Story {
    /// $213fa, with $ffdd64=0 from the first Stoker dispatcher. Includes the
    /// discovered 500 nuyen and byte Karma; no speculative cyberdeck mechanics.
    pub fn buy_belongings(
        &mut self,
        balance: &mut u32,
        flags: &mut u32,
        party: &mut [Runner],
    ) -> usize {
        if (*balance as i32) < 250 {
            self.inquiries |= 1;
            return 0xf5ab0;
        }
        *balance = balance.wrapping_sub(250).wrapping_add(500);
        *flags |= 1;
        for r in party.iter_mut() {
            r.raw[0x8d] = r.raw[0x8d].wrapping_add(1);
        }
        if let Some(joshua) = party.first_mut() {
            joshua.raw[0x8f] |= 1;
        }
        self.leads = (self.leads & !3) | 0x1c;
        0xf487c
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaxiResult {
    pub node: usize,
    pub balance: u32,
    pub carried: u32,
    pub travelled: bool,
}
/// $208a8: no-funds trips really are free; retain the signed long comparison.
pub fn pay_taxi(target: usize, fare: u8, balance: u32, carried: u32) -> TaxiResult {
    if target == 6 && carried & 2 == 0 {
        return TaxiResult {
            node: 0xfe618,
            balance,
            carried,
            travelled: false,
        };
    }
    let paid = (fare as i32) <= (balance as i32);
    TaxiResult {
        node: if paid { 0xfe608 } else { 0xfe610 },
        balance: if paid {
            balance.wrapping_sub(u32::from(fare))
        } else {
            balance
        },
        carried: if target == 6 { carried & !2 } else { carried },
        travelled: true,
    }
}
