use serde_json::json;
use shadowrealm::{
    party::Runner,
    shops::{equip_firearm, Catalog, Category, Outcome},
};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let catalog = Catalog::load(rom)?;
    let mut purchases = vec![];
    let mut equipment = vec![];
    for i in 0..4096usize {
        let slot = i % 3;
        let mut actor = Runner::load(rom, [0, 3, 10][slot], slot)?;
        actor.raw[0x8c] = (i / 16) as u8;
        actor.raw[0x4f] = [0, 1, 19, 20, 21, 127, 128, 148, 254, 255][i / 17 % 10];
        if i % 3 == 0 {
            actor.raw[0x66..0x6e].fill(7);
        } else {
            actor.raw[0x66..0x6e].fill(0);
            actor.raw[0x66..0x66 + i % 8].fill(7);
        }
        actor.raw[0x6e..0x76].fill(0xa5);
        let category = if i % 16 < 8 {
            Category::Weapons
        } else {
            Category::Equipment
        };
        let row = i % 8;
        let before = actor.raw.to_vec();
        let balance = [
            0, 24, 25, 249, 300, 500, 1000, 0x7fffffff, 0x80000000, 0xfffffff0,
        ][i / 13 % 10];
        let confirmed = i / 31 % 2 == 0;
        let p = catalog.purchase(rom, category, row, &actor, balance, confirmed)?;
        purchases.push(json!({"slot":slot,"category":if category==Category::Weapons{0}else{2},"row":row,"before":before,"balance":balance,"confirmed":confirmed,"after":p.actor.raw.to_vec(),"after_balance":p.balance,"price":p.price,"unavailable":p.outcome==Outcome::Unavailable}));
        let mut actor = Runner::load(rom, [0, 3, 10][slot], slot)?;
        let old_item = (i / 8 % 13) as u8;
        let target_slot = i % 8;
        actor.raw[0x56] = if i % 7 == 0 {
            255
        } else if i % 11 == 0 {
            127
        } else {
            (i / 23 % 8) as u8
        };
        actor.raw[0x57] = old_item;
        actor.raw[0x4e] = [0, 1, 3, 6, 12, 50, 100, 127, 128, 255][i / 5 % 10];
        actor.raw[0x9d] = 4;
        actor.raw[0x66 + target_slot] = (i / 13 % 12 + 1) as u8;
        if i % 17 == 0 {
            actor.raw[0x10] &= !1;
        }
        let before = actor.raw.to_vec();
        equip_firearm(rom, &mut actor, target_slot)?;
        equipment.push(
            json!({"slot":slot,"target":target_slot,"before":before,"after":actor.raw.to_vec()}),
        );
    }
    fs::write(
        out.join("shops.json"),
        serde_json::to_vec(&json!({"purchases":purchases,"equipment":equipment}))?,
    )?;
    Ok(())
}
