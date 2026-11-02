//! Private clinic recovery fixtures and a source-backed solo gameplay replay.
use serde_json::json;
use shadowrealm::{
    archives::{self, Archives},
    clinic,
    game::Game,
    input,
    party::Runner,
    transitions::TransitionPlan,
};
use std::{error::Error, fs, io::Write, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let record = clinic::Record::load(rom)?;
    export_archives(rom, out)?;
    let mut cases = vec![];
    for i in 0..1024u32 {
        let mut actor = Runner::load(rom, 0, 0)?;
        actor.raw[0x4c] = 0;
        actor.raw[0x9c] = i as u8;
        actor.raw[0x10] = (i >> 2) as u8;
        actor.raw[0x11] = i.wrapping_mul(43) as u8;
        actor.set_word(0x16, 0x8084);
        actor.set_word(0xdc, i as u16);
        let balance = match i % 12 {
            0 => 0,
            1 => 1,
            2 => 7,
            3 => 8,
            4 => 39_999,
            5 => 40_000,
            6 => 40_007,
            7 => u32::MAX,
            _ => i.wrapping_mul(497_631),
        };
        let before = actor.raw.to_vec();
        let flags = i & 1;
        let result = clinic::prepare(&[actor], balance, flags, 0).unwrap();
        cases.push(json!({"flags":flags,"node":if result.insured { record.insured.node.0 } else { record.node.0 },"before":before,"after":result.actor.raw.to_vec(),"balance":balance,"after_balance":result.balance,"fee":result.fee}));
    }
    let plan = TransitionPlan::clinic(rom, 5, &record, false)?;
    let insured_plan = TransitionPlan::clinic(rom, 5, &record, true)?;
    fs::write(
        out.join("clinic.json"),
        serde_json::to_vec(
            &json!({"cases":cases,"plan":{"map":plan.target_map,"spawn":plan.spawn_address,"camera":plan.camera_fixed,"positions":plan.positions},"node":record.node.0,"text":record.text.0,"insured_plan":{"map":insured_plan.target_map,"spawn":insured_plan.spawn_address,"camera":insured_plan.camera_fixed,"positions":insured_plan.positions},"insured_node":record.insured.node.0,"insured_text":record.insured.text.0,"more_text":record.insured.more_text.0,"continuation":record.insured.continuation.0,"continuation_text":record.insured.continuation_text.0}),
        )?,
    )?;
    let mut game = Game::load(rom, 5, &[0])?;
    game.party[0].raw[0x4c] = 1;
    game.party[0].raw[0x9c] = 3;
    game.nuyen = 40_001;
    for _ in 0..12_000 {
        game.update(rom, 0)?;
        if game.defeated {
            break;
        }
    }
    if game.dialogue.as_ref().map(|d| d.node) != Some(record.node.0) {
        return Err("clinic replay did not reach original passage".into());
    }
    for name in ["clinic-passage.ppm", "clinic-arrival.ppm"] {
        if name == "clinic-arrival.ppm" {
            game.update(rom, input::A)?;
            for _ in 0..120 {
                if game.transition.is_none() {
                    break;
                }
                game.update(rom, 0)?;
            }
        }
        let mut f = fs::File::create(out.join(name))?;
        f.write_all(b"P6\n320 224\n255\n")?;
        f.write_all(&game.render(rom)?)?;
    }
    fs::write(
        out.join("recovery.json"),
        serde_json::to_vec_pretty(
            &json!({"map":game.map_id(),"health":game.party[0].raw[0x4c],"condition":game.party[0].raw[0x9c],"balance":game.nuyen,"position":game.party[0].position(),"ammo":game.party[0].raw[0x4e],"npc_shots":game.combat.npc_shots}),
        )?,
    )?;
    Ok(())
}

/// These synthetic bytes test exact memory layout, not playable actor assets.
fn export_archives(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    let mut compact = vec![];
    let mut snapshots = vec![];
    let mut defeats = vec![];
    for i in 0..1024usize {
        let mut party = vec![
            Runner::load(rom, 0, 0)?,
            Runner::load(rom, 3 + i % 10, 1)?,
            Runner::load(rom, 3 + (i + 3) % 10, 2)?,
        ];
        for (slot, actor) in party.iter_mut().enumerate() {
            for (j, byte) in actor.raw.iter_mut().enumerate() {
                *byte =
                    (i.wrapping_mul(67) ^ j.wrapping_mul(31) ^ (i >> 3) ^ slot.wrapping_mul(113))
                        as u8;
            }
            actor.set_word(0x3e, slot as u16);
        }
        let mut initial = Archives::load(rom, &party)?;
        for (j, record) in initial.records.iter_mut().enumerate() {
            record.fill((i + j * 29) as u8);
        }
        initial.opinions = std::array::from_fn(|j| (i + j * 11) as u8);
        let mut restored = party[1].clone();
        let restore_before = restored.raw.to_vec();
        initial.restore(&mut restored)?;
        compact.push(json!({"before":party[0].raw.to_vec(),"packed":archives::pack(&party[0]).to_vec(),"restore_before":restore_before,"record":initial.records[Archives::index(&restored)?].to_vec(),"restored":restored.raw.to_vec()}));
        // Vary one/two/three occupied slots while preserving physical slot identity.
        party.truncate(1 + i % 3);
        let before = party.iter().flat_map(|r| r.raw).collect::<Vec<_>>();
        let indices = shadowrealm::hiring::recruit_indices(&party);
        let before_records = initial
            .records
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        let mut after = initial.clone();
        after.snapshot(&party)?;
        snapshots.push(json!({"party":before,"indices":indices,"before":before_records,"after":after.records.iter().flatten().copied().collect::<Vec<_>>()}));
        // Normal recovery clears health even for companions who were still alive.
        for (slot, actor) in party.iter_mut().enumerate() {
            actor.raw[0x10] = if slot == 0 {
                0x43
            } else {
                (i >> (slot + 2)) as u8 | 1
            };
            actor.raw[0xb0] &= 0x7f; // no currently allocated effect resource
            actor.set_word(0x30, (slot + 1) as u16);
        }
        party[0].raw[0x4c] = 0;
        let before_party = party.iter().flat_map(|r| r.raw).collect::<Vec<_>>();
        let after_defeat = initial.after_defeat(&party)?;
        defeats.push(json!({"party":before_party,"indices":indices,"before":before_records,"after":after_defeat.records.iter().flatten().copied().collect::<Vec<_>>(),"opinions":initial.opinions,"after_opinions":after_defeat.opinions}));
    }
    let mut penalties = vec![];
    for value in 0..=255u8 {
        let mut a = Archives::load(rom, &[])?;
        a.opinions.fill(value);
        a.penalize(3)?;
        penalties.push(json!({"before":value,"after":a.opinions[2]}));
    }
    let catalog = shadowrealm::dialogue_data::Catalog::load(rom)?;
    let mut opinions = vec![];
    for opinion in [-1, 0] {
        opinions.push(json!({"opinion":opinion,"root":catalog.runner_root(rom, opinion)?,"topic":catalog.runner_topic(rom, opinion)?}));
    }
    let mut rehires = vec![];
    for i in 0..256usize {
        let party = vec![Runner::load(rom, 0, 0)?];
        let mut archive = Archives::load(rom, &party)?;
        let mut former = Runner::load(rom, 5, 1)?;
        former.raw[0x4c] = 0;
        former.raw[0x4e] = (i % 8) as u8;
        former.raw[0x6e] = i as u8;
        former.raw[0x7a] = (1 + i % 12) as u8;
        former.raw[0x8f] = if i & 1 == 0 { 0 } else { 255 };
        former.raw[0x9d] = i as u8;
        archive.records[3] = archives::pack(&former);
        archive.opinions[2] = if i & 1 == 0 { 255 } else { 0 };
        let npc = shadowrealm::npc::space_needle(rom)?.remove(0);
        let quote =
            shadowrealm::hiring::quote(rom, &catalog.trent.hiring, &party[0], archive.opinion(2))?;
        let contract = if i & 2 == 0 {
            shadowrealm::hiring::Contract::SingleRun
        } else {
            shadowrealm::hiring::Contract::Permanent
        };
        let shadowrealm::hiring::Outcome::Hired { member, balance } =
            shadowrealm::hiring::prepare(rom, &archive, &party, &npc, 10000, quote, contract)?
        else {
            return Err("rehire fixture refused".into());
        };
        rehires.push(json!({"leader":party[0].raw.to_vec(),"npc":npc.raw.to_vec(),"archive":archive.records.iter().flatten().copied().collect::<Vec<_>>(),"opinions":archive.opinions,"contract":contract.flag(),"after":member.raw.to_vec(),"balance":balance,"price":10000-balance}));
    }
    fs::write(
        out.join("archives.json"),
        serde_json::to_vec(
            &json!({"compact":compact,"snapshots":snapshots,"defeats":defeats,"penalties":penalties,"opinions":opinions,"rehires":rehires}),
        )?,
    )?;
    Ok(())
}
