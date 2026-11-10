//! Ares stock, original purchase/bargaining rules and explicit firearm selection.
use crate::{
    dialogue::{Choice, Dialogue},
    dialogue_data::Address,
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
    shop: u8,
    entry: Address,
    opening: Address,
    purchase: Address,
    price: Address,
    negotiation: Address,
    weapon_stock: Address,
    equipment_stock: Address,
    items: Address,
    shop_stats: Address,
    equip: Address,
    full_inventory: Address,
    insufficient_funds: Address,
    clip_limit: Address,
    confirmation: Address,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Weapons,
    Equipment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stock {
    pub address: usize,
    pub item: u8,
    pub flags: u8,
    pub multiplier: u8,
}
impl Stock {
    pub fn supported(self) -> bool {
        self.item < 12 || self.item == 0x45
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Unavailable,
    InventoryFull,
    InsufficientFunds,
    Cancelled,
    Bought,
    ClipLimit,
}
pub struct Purchase {
    pub actor: Runner,
    pub balance: u32,
    pub price: u32,
    pub outcome: Outcome,
}
fn byte(rom: &[u8], at: usize) -> Result<u8, RomError> {
    rom.get(at)
        .copied()
        .ok_or(RomError("shop reference outside ROM"))
}
fn word(rom: &[u8], at: usize) -> Result<u16, RomError> {
    Ok(u16::from_be_bytes([byte(rom, at)?, byte(rom, at + 1)?]))
}
impl Catalog {
    pub const JSON: &'static str = include_str!("../data/shops/ares.json");
    pub fn load(rom: &[u8]) -> Result<Self, RomError> {
        Self::parse(Self::JSON, rom)
    }
    pub fn parse(json: &str, rom: &[u8]) -> Result<Self, RomError> {
        let c: Self = serde_json::from_str(json).map_err(|_| RomError("invalid shop JSON"))?;
        if c.schema_version != 1
            || c.scope.is_empty()
            || (c.district, c.location, c.shop) != (2, 11, 1)
            || [
                c.entry.0,
                c.opening.0,
                c.purchase.0,
                c.price.0,
                c.negotiation.0,
                c.weapon_stock.0,
                c.equipment_stock.0,
                c.items.0,
                c.shop_stats.0,
                c.equip.0,
                c.full_inventory.0,
                c.insufficient_funds.0,
                c.clip_limit.0,
                c.confirmation.0,
            ] != [
                0x1fa78, 0x110dd2, 0x5794e, 0x57a44, 0x57b04, 0x1aac86, 0x1aad77, 0xd4540,
                0x1aac6e, 0x12168, 0x1aa3c2, 0x1aa587, 0x1aa54d, 0x1aa715,
            ]
        {
            return Err(RomError("shop references outside verified scope"));
        }
        Dialogue::exposition(rom, c.opening.0)?;
        for category in [Category::Weapons, Category::Equipment] {
            for row in 0..8 {
                let s = c.stock(rom, category, row)?;
                if s.item < 128 && s.item != 69 {
                    word(rom, c.items.0 + usize::from(s.item) * 48 + 34)?;
                }
            }
        }
        for at in [
            c.full_inventory.0,
            c.insufficient_funds.0,
            c.clip_limit.0,
            c.confirmation.0,
        ] {
            Dialogue::read_text(rom, at)?;
        }
        byte(rom, c.shop_stats.0 + usize::from(c.shop) * 2)?;
        Ok(c)
    }
    pub fn opening(&self, rom: &[u8]) -> Result<Dialogue, RomError> {
        let mut d = Dialogue::exposition(rom, self.opening.0)?;
        d.caption = "ARES";
        d.choices.push(Self::choice("Browse".into(), 0, true));
        Ok(d)
    }
    pub fn stock(&self, rom: &[u8], category: Category, row: usize) -> Result<Stock, RomError> {
        if row >= 8 {
            return Err(RomError("shop row outside stock"));
        }
        let address = match category {
            Category::Weapons => self.weapon_stock.0,
            Category::Equipment => self.equipment_stock.0,
        } + usize::from(self.shop) * 24
            + row * 3;
        Ok(Stock {
            address,
            item: byte(rom, address)?,
            flags: byte(rom, address + 1)?,
            multiplier: byte(rom, address + 2)?,
        })
    }
    /// $57a44/$57a6e/$57b04. Ares has no contact discount. The low-word
    /// MULU/EXT result and signed-byte negotiating comparison are preserved.
    pub fn quote(&self, rom: &[u8], stock: Stock, actor: &Runner) -> Result<u32, RomError> {
        let base = if stock.item == 69 {
            u32::from(stock.multiplier)
        } else {
            let base = u32::from(word(rom, self.items.0 + usize::from(stock.item) * 48 + 34)?);
            if stock.multiplier == 0 {
                base
            } else {
                (base * u32::from(stock.multiplier) / 10) & 0xffff
            }
        };
        let threshold = byte(rom, self.shop_stats.0 + usize::from(self.shop) * 2)?;
        let difference = i16::from(actor.raw[0x8c] as i8) - i16::from(threshold as i8);
        let discount = if difference > 0 {
            let n = (difference as u8 as u16).min(10);
            (((base >> 5) as u16).wrapping_mul(n) as i16 as i32) as u32
        } else {
            0
        };
        Ok(base.wrapping_sub(discount))
    }
    /// Whole $5794e decision, excluding display and confirmation input. All
    /// state is returned staged so the caller can validate UI/HUD before commit.
    pub fn purchase(
        &self,
        rom: &[u8],
        category: Category,
        row: usize,
        actor: &Runner,
        balance: u32,
        confirmed: bool,
    ) -> Result<Purchase, RomError> {
        let stock = self.stock(rom, category, row)?;
        let mut p = Purchase {
            actor: actor.clone(),
            balance,
            price: 0,
            outcome: Outcome::Unavailable,
        };
        if stock.item >= 128 {
            return Ok(p);
        }
        let slot = (0..8).find(|&slot| actor.raw[0x66 + slot] == 0);
        if stock.item != 69 && slot.is_none() {
            p.outcome = Outcome::InventoryFull;
            return Ok(p);
        }
        p.price = self.quote(rom, stock, actor)?;
        if (p.price as i32) > (balance as i32) {
            p.outcome = Outcome::InsufficientFunds;
            return Ok(p);
        }
        if !confirmed {
            p.outcome = Outcome::Cancelled;
            return Ok(p);
        }
        if stock.item == 69 {
            p.actor.raw[0x4f] = p.actor.raw[0x4f].wrapping_add(1);
            if (p.actor.raw[0x4f].wrapping_sub(21) as i8) >= 0 {
                p.actor.raw[0x4f] = 20;
                p.outcome = Outcome::ClipLimit;
                return Ok(p);
            }
        } else {
            p.actor.raw[0x66 + slot.unwrap()] = stock.item.wrapping_add(1);
            p.actor.raw[0x6e + slot.unwrap()] = stock.flags;
        }
        p.balance = balance.wrapping_sub(p.price);
        p.outcome = Outcome::Bought;
        Ok(p)
    }
    fn choice(text: String, destination: usize, available: bool) -> Choice {
        Choice {
            text_address: 0,
            text,
            destination,
            available,
        }
    }
    pub fn root(&self, rom: &[u8], actor: &Runner, balance: u32) -> Result<Dialogue, RomError> {
        let mut d = Dialogue::with_text(
            rom,
            0,
            0,
            format!(
                "{}: choose merchandise.\nClips: {} / 20",
                actor.name(),
                actor.raw[0x4f]
            ),
        )?;
        d.caption = "ARES";
        d.funds = Some(balance);
        d.choices = vec![
            Self::choice("Weapons".into(), 0, true),
            Self::choice("Equipment".into(), 1, true),
            Self::choice("Leave".into(), 2, true),
        ];
        Ok(d)
    }
    pub fn stock_menu(
        &self,
        rom: &[u8],
        category: Category,
        actor: &Runner,
        balance: u32,
    ) -> Result<Dialogue, RomError> {
        let mut d = Dialogue::with_text(
            rom,
            0,
            0,
            format!(
                "{}: {}\nClips: {} / 20",
                actor.name(),
                match category {
                    Category::Weapons => "weapons",
                    Category::Equipment => "equipment",
                },
                actor.raw[0x4f]
            ),
        )?;
        d.caption = "ARES";
        d.funds = Some(balance);
        for row in 0..8 {
            let s = self.stock(rom, category, row)?;
            let (name, price) = if s.item >= 128 {
                ("--".into(), 0)
            } else {
                (
                    if s.item == 69 {
                        "Ammo clip".into()
                    } else {
                        item_name(rom, s.item + 1)?
                    },
                    self.quote(rom, s, actor)?,
                )
            };
            d.choices.push(Self::choice(
                if s.item >= 128 {
                    name
                } else {
                    format!("{name}  {price}$")
                },
                row,
                s.item < 128 && s.supported(),
            ));
        }
        d.choices.push(Self::choice("Back".into(), 8, true));
        Ok(d)
    }
    pub fn confirmation(
        &self,
        rom: &[u8],
        stock: Stock,
        price: u32,
        balance: u32,
    ) -> Result<Dialogue, RomError> {
        let name = if stock.item == 69 {
            "Ammo clip".into()
        } else {
            item_name(rom, stock.item + 1)?
        };
        let mut d = Dialogue::with_text(
            rom,
            0,
            self.confirmation.0,
            format!(
                "{name}: {price} nuyen.\n\n{}",
                Dialogue::read_text(rom, self.confirmation.0)?
            ),
        )?;
        d.caption = "ARES";
        d.funds = Some(balance);
        d.choices = vec![
            Self::choice("Yes".into(), 0, true),
            Self::choice("No".into(), 1, true),
        ];
        d.selected = 1;
        Ok(d)
    }
    pub fn refusal(
        &self,
        rom: &[u8],
        outcome: Outcome,
        balance: u32,
    ) -> Result<Dialogue, RomError> {
        let at = match outcome {
            Outcome::InventoryFull => self.full_inventory.0,
            Outcome::InsufficientFunds => self.insufficient_funds.0,
            Outcome::ClipLimit => self.clip_limit.0,
            _ => return Err(RomError("shop refusal has no message")),
        };
        let mut d = Dialogue::with_text(rom, 0, at, Dialogue::read_text(rom, at)?)?;
        d.caption = "ARES";
        d.funds = Some(balance);
        d.choices.push(Self::choice("Back".into(), 0, true));
        Ok(d)
    }
}
pub fn item_name(rom: &[u8], id: u8) -> Result<String, RomError> {
    if id == 0 {
        return Ok("Empty".into());
    }
    let text = Dialogue::read_text(rom, 0xd4540 + usize::from(id - 1) * 48)?;
    Ok(text
        .lines()
        .find(|s| !s.trim().is_empty())
        .unwrap_or("Item")
        .trim()
        .to_string())
}
pub fn inventory(rom: &[u8], actor: &Runner) -> Result<Dialogue, RomError> {
    let ammunition = if actor.raw[0x56] < 8 && (1..=12).contains(&actor.raw[0x57]) {
        format!("Ammo: {}", actor.raw[0x4e])
    } else {
        format!("Stored ammo: {}%", actor.raw[0x4e])
    };
    let mut d = Dialogue::with_text(
        rom,
        0,
        0,
        format!(
            "{}: select a firearm.\nSelect it again for fists.\n{}  Clips: {}",
            actor.name(),
            ammunition,
            actor.raw[0x4f]
        ),
    )?;
    d.caption = "INVENTORY";
    for slot in 0..8 {
        let id = actor.raw[0x66 + slot];
        let equipped = actor.raw[0x56] == slot as u8;
        d.choices.push(Catalog::choice(
            format!(
                "{}{}",
                item_name(rom, id)?,
                if equipped { " (equipped)" } else { "" }
            ),
            slot,
            (1..=12).contains(&id),
        ));
    }
    d.choices.push(Catalog::choice("Close".into(), 8, true));
    Ok(d)
}
/// $12168..$12292 for a selected firearm. Unknown old-item D1 contexts are
/// rejected before mutation. This includes original toggle and ammo rounding.
pub fn equip_firearm(rom: &[u8], actor: &mut Runner, slot: usize) -> Result<(), RomError> {
    if slot >= 8 || !(1..=12).contains(&actor.raw[0x66 + slot]) {
        return Err(RomError("inventory selection is not a firearm"));
    }
    let mut a = actor.clone();
    let old = a.raw[0x57];
    if (a.raw[0x56] as i8) >= 0 && old >= 13 {
        return Err(RomError("old equipment conversion context is not modeled"));
    }
    let mut percentage = u32::from(a.raw[0x4e]);
    a.raw[0x4e] = 0;
    if (a.raw[0x56] as i8) >= 0 && old != 0 {
        let cap = u32::from(byte(rom, 0xd4540 + usize::from(old - 1) * 48 + 36)?);
        let product = percentage * 100;
        let (q, r) = if cap == 0 {
            (product, 0)
        } else {
            (product / cap, product % cap)
        };
        percentage = (q + u32::from(r != 0 || q >= 256)) & 0xffff;
    }
    if old != 0 && a.raw[0x56] == slot as u8 {
        a.raw[0x4e] = percentage as u8;
        a.raw[0x56] = 127;
        a.raw[0x57] = 0;
    } else {
        a.raw[0x9d] = 0;
        a.raw[0x56] = slot as u8;
        a.raw[0x57] = a.raw[0x66 + slot];
        let cap = byte(rom, 0xd4540 + usize::from(a.raw[0x57] - 1) * 48 + 36)?;
        let product = percentage * u32::from(cap);
        let increment = (product / 100 + u32::from(product % 100 >= 50)) as u8;
        if a.raw[0x10] & 1 != 0 && (increment as i8) > 0 {
            a.raw[0x4e] = if (cap.wrapping_sub(increment) as i8) < 0 {
                cap
            } else {
                increment
            };
        }
    }
    *actor = a;
    Ok(())
}
