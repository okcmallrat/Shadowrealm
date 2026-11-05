//! Gunderson's original offer generation and bounded Redmond courier lifecycle.
use crate::{
    avoidance::State,
    dialogue::Dialogue,
    dialogue_data::{long, Address, NodeRecord},
    party::Runner,
    RomError,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stop {
    pub location: u8,
    pub pickup: Address,
    pub delivery: Address,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    generator: Address,
    offer_table: Address,
    location_table: Address,
    location_names: Address,
    district_names: Address,
    visit: Address,
    reward: Address,
    clear: Address,
    pub nodes: Vec<NodeRecord>,
    pub stops: Vec<Stop>,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/jobs/gunderson-courier.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid courier JSON"))?;
        if c.schema_version != 1
            || [
                c.generator.0,
                c.offer_table.0,
                c.location_table.0,
                c.location_names.0,
                c.district_names.0,
                c.visit.0,
                c.reward.0,
                c.clear.0,
            ] != [
                0x1e69c, 0xf3ebc, 0xf3e80, 0xf1f1a, 0xf1e58, 0x1ef2c, 0x1f00a, 0x1e68c,
            ]
        {
            return Err(RomError("courier references outside verified scope"));
        }
        let expected = [
            (0, 0x1113b2, 0x1113ba),
            (1, 0x11147a, 0x111482),
            (2, 0xf4aec, 0xf4af4),
            (3, 0x11149a, 0x1114a2),
            (4, 0x1114b2, 0x1114ba),
            (5, 0x1114ca, 0x1114d2),
            (6, 0xf4c98, 0xf4ca0),
            (7, 0x1114da, 0x1114e2),
            (9, 0x11155e, 0x110d72),
            (11, 0x11166e, 0x111676),
        ];
        if c.stops
            .iter()
            .map(|s| (s.location, s.pickup.0, s.delivery.0))
            .collect::<Vec<_>>()
            != expected
        {
            return Err(RomError("courier stops outside verified scope"));
        }
        let addresses = [
            0x105114, 0x1051b4, 0x1051c8, 0x1058e8, 0x1058e0, 0x1058f8, 0x105900, 0x10592c,
            0x105918, 0x110d1e, 0x1113b2, 0x1113ba, 0x11147a, 0x111482, 0xf4aec, 0xf4af4, 0x11149a,
            0x1114a2, 0x1114b2, 0x1114ba, 0x1114ca, 0x1114d2, 0xf4c98, 0xf4ca0, 0x1114da, 0x1114e2,
            0x11155e, 0x110d72, 0x11166e, 0x111676, 0xf2510, 0x1113d2,
        ];
        if c.nodes.iter().map(|n| n.address.0).collect::<Vec<_>>() != addresses {
            return Err(RomError("courier dialogue outside verified scope"));
        }
        for n in &c.nodes {
            if long(rom, n.address.0)? != n.text.0 || n.choices.len() > 3 {
                return Err(RomError("courier node differs from ROM"));
            }
            Dialogue::read_text(rom, n.text.0)?;
            if n.choices.is_empty() {
                if long(rom, n.address.0 + 4)? != u32::MAX as usize {
                    return Err(RomError("courier terminal node differs from ROM"));
                }
            } else {
                for (i, ch) in n.choices.iter().enumerate() {
                    if long(rom, n.address.0 + 4 + i * 4)? != ch.text.0
                        || long(rom, n.address.0 + 16 + i * 4)? != ch.destination.0
                    {
                        return Err(RomError("courier choice differs from ROM"));
                    }
                }
                if n.choices.len() < 3 && long(rom, n.address.0 + 4 + n.choices.len() * 4)? != 0 {
                    return Err(RomError("courier JSON omits ROM choices"));
                }
            }
        }
        Ok(c)
    }
    pub fn node(&self, address: usize) -> Result<&NodeRecord, RomError> {
        self.nodes
            .iter()
            .find(|n| n.address.0 == address)
            .ok_or(RomError("courier node is unavailable"))
    }
    pub fn location(&self, rom: &[u8], district: u8, location: u8) -> Result<String, RomError> {
        if district != 2 || !self.stops.iter().any(|s| s.location == location) {
            return Err(RomError("courier location outside Redmond scope"));
        }
        Dialogue::read_text(
            rom,
            long(
                rom,
                long(rom, self.location_names.0 + usize::from(district) * 4)?
                    + usize::from(location) * 4,
            )?,
        )
    }
    pub fn dialogue(&self, rom: &[u8], address: usize) -> Result<Dialogue, RomError> {
        let node = self.node(address)?;
        let mut d = Dialogue::with_text(
            rom,
            address,
            node.text.0,
            Dialogue::read_text(rom, node.text.0)?,
        )?;
        d.caption = "MR. GUNDERSON";
        for c in &node.choices {
            d.choices.push(crate::dialogue::Choice {
                text_address: c.text.0,
                text: Dialogue::read_text(rom, c.text.0)?,
                destination: c.destination.0,
                available: matches!(c.destination.0, 7 | 11 | 12 | 13 | 50 | 100)
                    || self.nodes.iter().any(|n| n.address.0 == c.destination.0),
            });
        }
        Ok(d)
    }
    /// Original text fragments in a host panel; the original Pocket Secretary UI
    /// and fixed screen-coordinate layout are not reconstructed here.
    pub fn briefing(
        &self,
        rom: &[u8],
        run: &Runtime,
        confirmation: bool,
        status: bool,
    ) -> Result<Dialogue, RomError> {
        if !run.active() {
            return Err(RomError("no current job"));
        }
        let title = Dialogue::read_text(rom, long(rom, 0xf1dee + usize::from(run.raw[1]) * 4)?)?;
        let mut text = title;
        if run.courier() {
            let read = |p| Dialogue::read_text(rom, p);
            let district = read(long(rom, self.district_names.0 + 8)?)?;
            text = format!(
                "{}\n\n{} {} {}\n\n{} {} {}\n\n{}",
                text,
                read(0xf23b2)?,
                self.location(rom, 2, run.raw[3])?,
                district,
                read(0xf23b8)?,
                self.location(rom, 2, run.raw[6])?,
                district,
                read(0xf23ed)?
            );
        }
        text.push_str(&format!("\n\nPayment: {}$", run.pay()));
        if confirmation {
            let prompt =
                Dialogue::read_text(rom, if run.raw[1] == 0 { 0x106171 } else { 0x1060e0 })?;
            if !prompt.contains("      $") || !prompt.contains("     $") {
                return Err(RomError("job pay placeholders differ from ROM"));
            }
            text = prompt
                .replacen("      $", &format!(" {}$", run.base_pay), 1)
                .replacen("     $", &format!(" {}$", run.pay()), 1);
        }
        let address = if confirmation { 0x1058cc } else { 0x1058a4 };
        let mut d = Dialogue::with_text(rom, address, 0xffb740, text)?;
        if !status {
            d.quote = Some(u32::from(run.pay()));
        }
        d.caption = if status && run.courier() {
            if run.flags & 1 == 0 {
                "COURIER: PICKUP"
            } else {
                "COURIER: DELIVER"
            }
        } else {
            "GUNDERSON RUN"
        };
        if !status {
            let choices: Vec<(usize, usize)> = if confirmation {
                vec![(0x103a0e, 50), (0x103a13, 11)]
            } else {
                vec![(0xfe5bf, 9)]
            };
            for (at, destination) in choices {
                d.choices.push(crate::dialogue::Choice {
                    text_address: at,
                    text: Dialogue::read_text(rom, at)?,
                    destination,
                    available: destination != 50 || run.courier(),
                });
            }
        }
        Ok(d)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Runtime {
    /// Original $fffbec..$fffbf5, retained even after the active word clears.
    pub raw: [u8; 10],
    pub flags: u8,      // $fffc1c
    pub carried: u32,   // represented $fffc22, including package bit 9
    pub base_pay: u16,  // original review scratch $fff0c0
    pub accepted: bool, // host UI boundary; original offer review blocks the world
    pub completed: u64,
}
impl Default for Runtime {
    fn default() -> Self {
        let mut raw = [0; 10];
        raw[..2].fill(255);
        Self {
            raw,
            flags: 0,
            carried: 0,
            base_pay: 0,
            accepted: false,
            completed: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit {
    None,
    Pickup,
    Delivery,
}
impl Runtime {
    pub fn active(&self) -> bool {
        (u16::from_be_bytes(self.raw[..2].try_into().unwrap()) as i16) >= 0
    }
    pub fn courier(&self) -> bool {
        self.active() && self.raw[0] == 0 && self.raw[1] == 2
    }
    pub fn pay(&self) -> u16 {
        u16::from_be_bytes(self.raw[8..10].try_into().unwrap())
    }
    /// $1e68c clears only the active word and flag byte, not the remaining record.
    pub fn clear(&mut self) {
        self.raw[..2].fill(255);
        self.flags = 0;
        self.accepted = false;
    }
    /// $215ac explicitly removes the package before abandoning the old contract.
    pub fn cancel(&mut self) {
        self.carried &= !(1 << 9);
        // $215ac leaves the old record until the forced next offer at $2148a.
        self.accepted = false;
    }
    /// $1e69c plus Gunderson's $1e744/$1e9c0 initializers. Other Johnsons defer.
    pub fn generate(
        &mut self,
        rom: &[u8],
        catalog: &Catalog,
        negotiating_stat: u8,
        rng: &mut State,
    ) -> Result<(), RomError> {
        let mut next = self.clone();
        let mut random = State { seed: rng.seed };
        next.clear();
        let at = catalog.offer_table.0 + usize::from(random.sample(8)) * 6;
        let row = rom
            .get(at..at + 6)
            .ok_or(RomError("job offer table outside ROM"))?;
        if row[0] > 2 {
            return Err(RomError("Gunderson job type outside verified scope"));
        }
        next.raw[0] = 0;
        next.raw[1] = row[0];
        next.raw[7] = row[1];
        let base = u16::from_be_bytes([row[2], row[3]]);
        next.base_pay = base;
        let count = u16::from_be_bytes([row[4], row[5]])
            .wrapping_mul(u16::from(negotiating_stat))
            .wrapping_add(1);
        let pay = random.sample(count).wrapping_mul(5).wrapping_add(base);
        next.raw[8..10].copy_from_slice(&pay.to_be_bytes());
        next.raw[2] = 2;
        if row[0] == 0 {
            next.raw[4..6].fill(0);
        } else {
            let locations = rom
                .get(catalog.location_table.0 + 20..catalog.location_table.0 + 30)
                .ok_or(RomError("job location table outside ROM"))?;
            next.raw[3] = locations[usize::from(random.sample(10))];
            next.raw[4] = random.sample(12) as u8;
            next.raw[5] = 2;
            loop {
                next.raw[6] = locations[usize::from(random.sample(10))];
                if next.raw[6] != next.raw[3] {
                    break;
                }
            }
        }
        *self = next;
        rng.seed = random.seed;
        Ok(())
    }
    /// $1ef2c/$1eff2/$1f00a: pickup before delivery, one reward, byte Karma.
    pub fn visit(
        &mut self,
        district: u8,
        location: u8,
        balance: &mut u32,
        party: &mut [Runner],
    ) -> Visit {
        if !self.courier() || !self.accepted || self.flags & 0x80 != 0 {
            return Visit::None;
        }
        if self.flags & 1 == 0 {
            if (district, location) != (self.raw[2], self.raw[3]) {
                return Visit::None;
            }
            self.flags |= 1;
            self.carried |= 1 << 9;
            Visit::Pickup
        } else {
            if (district, location) != (self.raw[5], self.raw[6]) {
                return Visit::None;
            }
            self.carried &= !(1 << 9);
            *balance = balance.wrapping_add(u32::from(self.pay()));
            for a in party.iter_mut().take(3) {
                a.raw[0x8d] = a.raw[0x8d].wrapping_add(self.raw[7]);
            }
            self.clear();
            self.completed += 1;
            Visit::Delivery
        }
    }
    /// $c6d0..$c69a: a carried courier package is lost before medical recovery.
    pub fn fail_on_defeat(&mut self) -> bool {
        if self.courier() && self.flags & 1 != 0 {
            self.carried &= !(1 << 9);
            self.clear();
            true
        } else {
            false
        }
    }
}
