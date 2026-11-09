use serde_json::json;
use shadowrealm::{
    advancement::{self, Catalog, Refusal},
    party::Runner,
};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let c = Catalog::load(rom)?;
    let mut upgrades = vec![];
    for id in 0..20 {
        for value in 0..256usize {
            let slot = value % 3;
            let mut a = Runner::load(rom, [0, 3, 10][slot], slot)?;
            a.raw[advancement::field(id)?] = value as u8;
            a.raw[0x9a] = (value % 5) as u8;
            a.raw[0x9b] = [0, 1, 2, 3, 127, 128, 129, 130, 255][value / 3 % 9];
            if id != 9 {
                a.raw[0x81] = [0, 1, 4, 12, 13, 127, 128, 254, 255][value / 7 % 9];
            }
            let q = c.quote(rom, &a, id)?;
            a.raw[0x8d] = match value % 5 {
                0 => 0,
                1 => q.cost.saturating_sub(1),
                2 => q.cost,
                3 => q.cost.saturating_add(1),
                _ => 255,
            };
            let confirmed = value % 3 != 0;
            let (after, refusal) = c.advance(rom, &a, id, confirmed)?;
            let (col, row) = if id < 8 {
                (id / 4, id % 4)
            } else {
                ((id - 8) / 6, (id - 8) % 6 + 4)
            };
            upgrades.push(json!({"id":id,"slot":slot,"col":col,"row":row,"before":a.raw.to_vec(),"after":after.raw.to_vec(),"confirmed":confirmed,"maximum":q.refusal==Some(Refusal::Maximum),"quoted":q.refusal.is_none(),"next":q.next,"cost":q.cost,"refusal":refusal.map(advancement::refusal_text)}));
        }
    }
    let mut rests = vec![];
    for i in 0..1024usize {
        let mut party = vec![];
        for slot in 0..3 {
            let mut a = Runner::load(rom, [0, 3, 10][slot], slot)?;
            a.raw[0x4c] = i.wrapping_add(slot * 3) as u8;
            a.raw[0x9c] =
                [0, 1, 3, 9, 10, 118, 119, 120, 127, 128, 129, 200, 246, 255][(i / 7 + slot) % 14];
            a.raw[0x10] = [0, 1, 0x11, 0x51, 0xff][(i / 13 + slot) % 5];
            party.push(a);
        }
        let before = party.iter().map(|r| r.raw.to_vec()).collect::<Vec<_>>();
        let medical = advancement::rest(&mut party);
        rests.push(json!({"before":before,"after":party.iter().map(|r|r.raw.to_vec()).collect::<Vec<_>>(),"medical":medical}));
    }
    let payments = [
        0, 1, 20, 74, 75, 76, 100, 0x7fffffff, 0x80000000, 0x8000004a, 0x8000004b, 0xfffffffe,
        0xffffffff,
    ]
    .map(|balance| json!({"before":balance,"after":c.payment(balance)}));
    fs::write(
        out.join("advancement.json"),
        serde_json::to_vec(&json!({"upgrades":upgrades,"rests":rests,"payments":payments}))?,
    )?;
    Ok(())
}
