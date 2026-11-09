//! Hotel payment/rest and Karma byte rules recovered from the original routines.
use crate::{
    dialogue::{Choice, Dialogue},
    dialogue_data::{long, Address, NodeRecord},
    party::Runner,
    RomError,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    schema_version: u8,
    scope: String,
    district: u8,
    location: u8,
    fee: u8,
    entry: Address,
    payment: Address,
    rest: Address,
    quote: Address,
    upgrade: Address,
    race_caps: Address,
    attribute_labels: Address,
    skill_labels: Address,
    nodes: Vec<NodeRecord>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Fixed,
    MageOnly,
    Maximum,
    InsufficientKarma,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quote {
    pub next: u8,
    pub cost: u8,
    pub refusal: Option<Refusal>,
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/hotels/gates.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid hotel JSON"))?;
        if c.schema_version != 1
            || c.scope.is_empty()
            || (c.district, c.location, c.fee) != (0, 3, 75)
            || [
                c.entry.0,
                c.payment.0,
                c.rest.0,
                c.quote.0,
                c.upgrade.0,
                c.race_caps.0,
                c.attribute_labels.0,
                c.skill_labels.0,
            ] != [
                0x1f65a, 0x21458, 0xef08, 0xd82a, 0xd33a, 0xc22a6, 0xc1a2a, 0xc1a4a,
            ]
            || c.nodes.iter().map(|n| n.address.0).collect::<Vec<_>>()
                != [0xf4910, 0xf4924, 0xf493c, 0xf4b2c, 0xf5ab8, 0xf5af8]
            || long(rom, 0x12ccb4 + 6 * 4)? != c.payment.0
        {
            return Err(RomError("hotel references outside verified scope"));
        }
        for n in &c.nodes {
            if long(rom, n.address.0)? != n.text.0 || n.choices.len() > 2 {
                return Err(RomError("hotel passage differs from ROM"));
            }
            Dialogue::read_text(rom, n.text.0)?;
            if n.choices.is_empty() {
                if long(rom, n.address.0 + 4)? != 0xffffffff {
                    return Err(RomError("hotel terminal differs from ROM"));
                }
            } else {
                for (i, ch) in n.choices.iter().enumerate() {
                    if long(rom, n.address.0 + 4 + i * 4)? != ch.text.0
                        || long(rom, n.address.0 + 16 + i * 4)? != ch.destination.0
                    {
                        return Err(RomError("hotel choice differs from ROM"));
                    }
                    Dialogue::read_text(rom, ch.text.0)?;
                }
                if long(rom, n.address.0 + 4 + n.choices.len() * 4)? != 0 {
                    return Err(RomError("hotel JSON omits choices"));
                }
            }
        }
        for id in 0..20 {
            c.label(rom, id)?;
        }
        if rom.get(c.race_caps.0..c.race_caps.0 + 30)
            != Some(&[
                6, 6, 6, 6, 6, 6, 6, 7, 6, 8, 6, 6, 7, 5, 8, 6, 6, 7, 9, 6, 8, 5, 5, 6, 11, 5, 10,
                4, 4, 5,
            ])
        {
            return Err(RomError("race caps differ from ROM"));
        }
        Ok(c)
    }
    pub fn dialogue(&self, rom: &[u8], node: usize) -> Result<Dialogue, RomError> {
        let n = self
            .nodes
            .iter()
            .find(|n| n.address.0 == node)
            .ok_or(RomError("unsupported hotel passage"))?;
        let mut d = Dialogue::with_text(rom, node, n.text.0, Dialogue::read_text(rom, n.text.0)?)?;
        d.caption = "GATES UNDERSOUND";
        for ch in &n.choices {
            d.choices.push(Choice {
                text_address: ch.text.0,
                text: Dialogue::read_text(rom, ch.text.0)?,
                destination: ch.destination.0,
                available: true,
            });
        }
        Ok(d)
    }
    /// $21458 tests N after SUB.L, including its original overflow behavior.
    pub fn payment(&self, balance: u32) -> Option<u32> {
        payment(balance, self.fee)
    }
    pub fn label(&self, rom: &[u8], id: usize) -> Result<String, RomError> {
        field(id)?;
        let p = if id < 8 {
            self.attribute_labels.0 + id * 4
        } else {
            self.skill_labels.0 + (id - 8) * 4
        };
        Ok(Dialogue::read_text(rom, long(rom, p)?)?.trim().to_owned())
    }
    /// $d82a. IDs 0..7 are attributes (including fixed Essence/Magic), 8..19 skills.
    pub fn quote(&self, rom: &[u8], actor: &Runner, id: usize) -> Result<Quote, RomError> {
        let at = field(id)?;
        let mut next = actor.raw[at].wrapping_add(1);
        let mut q = Quote {
            next,
            cost: 0,
            refusal: None,
        };
        if id == 6 || id == 7 {
            q.refusal = Some(Refusal::Fixed);
            return Ok(q);
        }
        if id == 8 && (actor.raw[0x9b].wrapping_sub(2) as i8) >= 0 {
            q.refusal = Some(Refusal::MageOnly);
            return Ok(q);
        }
        if id < 6 {
            if actor.raw[0x9a] > 4 {
                return Err(RomError("unsupported runner race"));
            }
            let cap = *rom
                .get(self.race_caps.0 + usize::from(actor.raw[0x9a]) * 6 + id)
                .ok_or(RomError("race cap missing"))?;
            if (next as i8) > (cap as i8) {
                q.refusal = Some(Refusal::Maximum);
                return Ok(q);
            }
            q.cost = next;
        } else {
            if (next.wrapping_sub(13) as i8) >= 0 {
                q.refusal = Some(Refusal::Maximum);
                return Ok(q);
            }
            if (10..=12).contains(&id) {
                // An untrained gun specialization starts one above base Firearms.
                // The original checks the skill cap before this substitution.
                if actor.raw[at] == 0 {
                    next = actor.raw[0x81].wrapping_add(1);
                }
                q.cost = (u16::from(next) * 15).div_ceil(10) as u8;
            } else {
                q.cost = next.wrapping_mul(2);
            }
        }
        q.next = next;
        Ok(q)
    }
    /// $d33a through its actor writes, excluding confirmation and redraw.
    pub fn advance(
        &self,
        rom: &[u8],
        actor: &Runner,
        id: usize,
        confirmed: bool,
    ) -> Result<(Runner, Option<Refusal>), RomError> {
        let q = self.quote(rom, actor, id)?;
        let mut after = actor.clone();
        if q.refusal.is_some() {
            return Ok((after, q.refusal));
        }
        if actor.raw[0x8d] < q.cost {
            return Ok((after, Some(Refusal::InsufficientKarma)));
        }
        if confirmed {
            after.raw[0x8d] -= q.cost;
            after.raw[field(id)?] = q.next;
        }
        Ok((after, None))
    }
}
pub fn field(id: usize) -> Result<usize, RomError> {
    match id {
        0..=6 => Ok(0x77 + id),
        7 => Ok(0x7f), // Essence occupies $7d/$7e; Magic follows it.
        8..=13 => Ok(0x80 + id - 8),
        14..=19 => Ok(0x81 + id - 8),
        _ => Err(RomError("invalid advancement stat")),
    }
}
pub fn payment(balance: u32, fee: u8) -> Option<u32> {
    let next = balance.wrapping_sub(u32::from(fee));
    ((next as i32) >= 0).then_some(next)
}
pub fn refusal_text(r: Refusal) -> usize {
    match r {
        Refusal::Fixed => 0xc2186,
        Refusal::MageOnly => 0xc2122,
        Refusal::Maximum => 0xc1b51,
        Refusal::InsufficientKarma => 0xc214e,
    }
}
/// $ef08: only active party slots; severe wounds require medical attention.
pub fn rest(party: &mut [Runner]) -> Vec<usize> {
    let mut medical = vec![];
    for (slot, a) in party.iter_mut().take(3).enumerate() {
        if a.raw[0x10] & 1 == 0 {
            continue;
        }
        let health = a.raw[0x4c];
        if health >= 3 {
            crate::spell_equipment::add_health(a, if health >= 5 { 2 } else { 1 });
        } else {
            medical.push(slot);
        }
        crate::spell_equipment::add_condition(a, 10);
        if health >= 3 {
            a.raw[0x10] &= !0x10;
        }
    }
    medical
}
