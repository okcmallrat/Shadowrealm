use serde_json::json;
use shadowrealm::{
    party::Runner,
    progression::{pay_taxi, Catalog, Story},
    transitions::TransitionPlan,
};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let c = Catalog::load(rom)?;
    let mut purchases = vec![];
    let mut taxis = vec![];
    let mut gates = vec![];
    let mut routes = vec![];
    let mut armbands = vec![];
    for i in 0..2048usize {
        let mut party = vec![];
        for slot in 0..3 {
            let mut r = Runner::load(rom, [0, 3, 10][slot], slot)?;
            r.raw[0x8d] = [0, 1, 254, 255][(i + slot) % 4];
            r.raw[0x8f] = (i.wrapping_mul(17 + slot)) as u8;
            party.push(r);
        }
        let before = party.iter().flat_map(|r| r.raw).collect::<Vec<_>>();
        let balance = [0, 20, 249, 250, 251, 1000, 0x7fffffff, 0xfffffff0][i % 8];
        let flags = (i as u32).wrapping_mul(0x31415926);
        let leads = (i as u32).wrapping_mul(0x87654321);
        let inquiries = (i / 8) as u8;
        let mut story = Story { leads, inquiries };
        let mut after_balance = balance;
        let mut after_flags = flags;
        let node = story.buy_belongings(&mut after_balance, &mut after_flags, &mut party);
        purchases.push(json!({"before":before,"balance":balance,"flags":flags,"leads":leads,"inquiries":inquiries,"after":party.iter().flat_map(|r|r.raw).collect::<Vec<_>>(),"after_balance":after_balance,"after_flags":after_flags,"after_leads":story.leads,"after_inquiries":story.inquiries,"node":node}));
        let cursor = i % 7;
        let current = i / 7 % 7;
        let (target, _, fare) = c.district(rom, cursor)?;
        let carried = (i as u32).wrapping_mul(0x915763);
        let result = pay_taxi(target, fare, balance, carried);
        taxis.push(json!({"cursor":cursor,"current":current,"target":target,"fare":fare,"balance":balance,"carried":carried,"after_balance":result.balance,"after_carried":result.carried,"travelled":result.travelled,"node":result.node}));
        let (page, _) = c.taxi_quote(rom, cursor, current, flags, &party, carried)?;
        gates.push(json!({"cursor":cursor,"current":current,"flags":flags,"node":if flags&1==0 {page.node}else if target==current {page.node}else{0xfe5d4}}));
        if current != target {
            party[0].raw[0x8f] |= 0x20;
            let (page, free) = c.taxi_quote(rom, cursor, current, 1, &party, carried)?;
            assert_eq!(free, Some(0));
            let result = pay_taxi(target, 0, balance, carried);
            armbands.push(json!({"cursor":cursor,"current":current,"target":target,"fare":fare,"carried":carried,"node":page.node,"after_carried":result.carried,"travelled":result.travelled}));
        }
    }
    for source in 0..6 {
        for target in 0..6 {
            if source == target {
                continue;
            }
            let p = TransitionPlan::taxi(rom, source, target)?;
            routes.push(json!({"source":source,"target":target,"camera":p.camera_fixed,"positions":p.positions}));
        }
    }
    fs::write(
        out.join("progression.json"),
        serde_json::to_vec(
            &json!({"purchases":purchases,"taxis":taxis,"gates":gates,"routes":routes,"armbands":armbands}),
        )?,
    )?;
    Ok(())
}
