use serde_json::json;
use shadowrealm::{
    avoidance::State,
    jobs::{Catalog, Runtime, Visit},
    party::Runner,
};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let catalog = Catalog::load(rom)?;
    let mut offers = vec![];
    for i in 0..2048usize {
        let mut job = Runtime::default();
        job.raw = [0xa5; 10];
        job.flags = 0x81;
        job.carried = 0x12345678;
        let before = job.clone();
        let seed = (i as u16).wrapping_mul(971);
        let rep = [0, 1, 4, 10, 127, 255][i / 8 % 6];
        let mut rng = State { seed };
        job.generate(rom, &catalog, rep, &mut rng)?;
        offers.push(json!({"before":before.raw.to_vec(),"flags":before.flags,"carried":before.carried,"seed":seed,"negotiating_stat":rep,"after":job.raw.to_vec(),"after_flags":job.flags,"after_seed":rng.seed,"base_pay":job.base_pay}));
    }
    let mut visits = vec![];
    let mut failures = vec![];
    for i in 0..2048usize {
        let locations = [0u8, 1, 2, 3, 4, 5, 6, 7, 9, 11];
        let source = locations[i % 10];
        let target = locations[(i / 10 + 1 + i % 10) % 10];
        let mut job = Runtime::default();
        job.raw = [
            0,
            2,
            2,
            source,
            7,
            2,
            target,
            [0, 1, 2, 255][i / 31 % 4],
            (i >> 8) as u8,
            i as u8,
        ];
        if i % 17 == 0 {
            job.raw[0..2].fill(255);
        }
        job.accepted = true;
        job.flags = [0, 1, 128, 129, 2, 3][i / 7 % 6];
        job.carried = (i as u32).wrapping_mul(917351);
        let mut party = vec![];
        for slot in 0..3 {
            let mut a = Runner::load(rom, [0, 3, 10][slot], slot)?;
            a.raw[0x8d] = [0, 253, 255][(i + slot) % 3];
            if i % 5 == 0 && slot > 0 {
                a.raw[0x10] = 0;
            }
            party.push(a);
        }
        let before = party.iter().flat_map(|a| a.raw).collect::<Vec<_>>();
        let raw = job.raw;
        let flags = job.flags;
        let carried = job.carried;
        let district = if i % 11 == 0 { 3 } else { 2 };
        let location = if i % 3 == 0 { source } else { target };
        let balance = [0u32, 1000, 0x7fffffff, 0xfffffff0][i / 13 % 4];
        let mut after_balance = balance;
        let event = job.visit(district, location, &mut after_balance, &mut party);
        visits.push(json!({"raw":raw.to_vec(),"flags":flags,"carried":carried,"before":before,"district":district,"location":location,"balance":balance,"after_raw":job.raw.to_vec(),"after_flags":job.flags,"after_carried":job.carried,"after":party.iter().flat_map(|a|a.raw).collect::<Vec<_>>(),"after_balance":after_balance,"event":match event {Visit::None=>0,Visit::Pickup=>255,Visit::Delivery=>1}}));
        let mut failed = Runtime::default();
        failed.raw = raw;
        failed.flags = flags;
        failed.carried = carried;
        let lost = failed.fail_on_defeat();
        failures.push(json!({"raw":raw.to_vec(),"flags":flags,"carried":carried,"after_raw":failed.raw.to_vec(),"after_flags":failed.flags,"after_carried":failed.carried,"lost":lost}));
    }
    let stops = catalog
        .stops
        .iter()
        .map(|s| json!({"location":s.location,"pickup":s.pickup.0,"delivery":s.delivery.0}))
        .collect::<Vec<_>>();
    let mut entrances = vec![];
    for i in 0..64u16 {
        let map = shadowrealm::maps::MapDescriptor::read(rom, 2)?;
        let player = (0x1d00 + i * 97, 0x2000 + i * 53);
        let camera = (i * 128, i * 256);
        let plan = shadowrealm::transitions::TransitionPlan::jump_house_from_contact(
            rom, &map, 0xc4, 2, camera, player,
        )?
        .ok_or("missing Jump House route")?;
        let saved = plan.exterior_return.ok_or("missing Jump House return")?;
        entrances.push(json!({"player":[player.0,player.1],"camera":[camera.0,camera.1],"saved":saved.descriptor.to_vec(),"positions":plan.positions,"return_positions":saved.positions,"target":plan.target_map,"arrival_camera":[plan.camera_fixed.0,plan.camera_fixed.1]}));
    }
    fs::write(
        out.join("jobs.json"),
        serde_json::to_vec(
            &json!({"offers":offers,"visits":visits,"failures":failures,"stops":stops,"entrances":entrances}),
        )?,
    )?;
    Ok(())
}
