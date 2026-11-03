use serde_json::json;
use shadowrealm::{
    avoidance::State,
    party::Runner,
    spellbooks::{self, Selection, Spell, Spellbooks},
};
use std::{error::Error, fs, path::Path};
pub fn run(rom: &[u8], out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let books = Spellbooks::load(rom)?;
    let mut drains = vec![];
    let mut equipment = vec![];
    let mut applied_drains = vec![];
    let mut decisions = vec![];
    for i in 0..4096usize {
        let mut actor = Runner::load(rom, i % 13, i % 3)?;
        actor.raw[0x80] = (i % 16) as u8;
        actor.raw[0x7c] = (i / 7 % 16) as u8;
        actor.raw[0x4d] = (i / 11 % 4) as u8;
        actor.raw[0x66..0x76].fill(0);
        for slot in [0, 3, 6, 7] {
            if (i + slot) % 3 == 0 {
                actor.raw[0x66 + slot] = 0x18;
                actor.raw[0x6e + slot] = (i / 13 % 8) as u8;
            }
        }
        let spell = Spell {
            id: (i % 13) as u8,
            force: [0, 1, 2, 3, 6, 12, 127, 128, 255][i / 13 % 9],
        };
        drains.push(json!({"actor":actor.raw.to_vec(),"id":spell.id,"force":spell.force,"drain":spellbooks::estimated_drain(rom,&actor,spell)?}));
        actor.raw[0x9b] = [0, 1, 2, 3][i / 17 % 4];
        actor.raw[0x9c] = [1, 4, 5, 8, 10][i / 19 % 5];
        actor.set_word(0xa0, (i % 5) as u16);
        actor.raw[0x56] = if i % 2 == 0 { 0 } else { 255 };
        actor.raw[0x57] = [1, 2, 3, 4, 8, 9, 11, 12][i / 5 % 8];
        actor.raw[0x9d] = [1, 2, 4, 8][i / 23 % 4];
        let mut party = vec![];
        for slot in 0..3 {
            let mut a = Runner::load(rom, 0, slot)?;
            a.raw[0x4c] = [0, 3, 4, 10][(i / 7 + slot) % 4];
            if slot == 0 || (i + slot) % 5 != 0 {
                party.push(a);
            }
        }
        let seed = (i as u16).wrapping_mul(971);
        let mut rng = State { seed };
        let (choice, heal) = spellbooks::select(rom, &books, &actor, &party, &mut rng)?;
        let (kind, id, force) = match choice {
            Selection::KeepEquipped => (0, 0, 0),
            Selection::Spell(s) => (1, s.id, s.force),
            Selection::InventoryWeapon => (2, 0, 0),
        };
        decisions.push(json!({"actor":actor.raw.to_vec(),"party":party.iter().map(|a|a.raw.to_vec()).collect::<Vec<_>>(),"seed":seed,"after_seed":rng.seed,"kind":kind,"id":id,"force":force,"heal":heal}));
    }
    for i in 0..4096usize {
        let mut a = Runner::load(rom, i % 13, i % 3)?;
        a.raw[0x10] = [0, 1, 13, 15][i % 4];
        a.raw[0x57] = (i % 16) as u8;
        a.raw[0x4e] = [0, 1, 12, 38, 127, 128, 255][i / 7 % 7];
        a.raw[0x56] = [0, 127, 255][i / 11 % 3];
        let spell = Spell {
            id: (i % 13) as u8,
            force: (i / 3 % 13) as u8,
        };
        let index = (i % 12) as u16;
        let before = a.raw.to_vec();
        shadowrealm::spell_equipment::equip_spell(rom, &mut a, spell, index)?;
        equipment.push(json!({"kind":0,"before":before,"after":a.raw.to_vec(),"id":spell.id,"force":spell.force,"index":index}));
        a.raw[0x66..0x6e].fill(0x18);
        let slot = i % 9;
        if slot < 8 {
            a.raw[0x66 + slot] = (i / 5 % 13) as u8;
        }
        a.raw[0x4e] = if slot >= 8 || a.raw[0x66 + slot] == 0 {
            0
        } else {
            [0, 1, 12, 38, 100, 127, 255][i / 7 % 7]
        };
        let before = a.raw.to_vec();
        shadowrealm::spell_equipment::equip_inventory(rom, &mut a)?;
        equipment.push(json!({"kind":1,"before":before,"after":a.raw.to_vec()}));
        a.raw[0x10] |= 1;
        a.raw[0x4c] = (i % 11) as u8;
        a.raw[0x9c] = (i / 11 % 11) as u8;
        a.raw[0x4d] = (i / 13 % 4) as u8;
        a.raw[0x80] = (i / 17 % 12) as u8;
        a.raw[0x7c] = (i / 19 % 12) as u8;
        a.raw[0x66..0x76].fill(0);
        for slot in 0..8 {
            if (i + slot) % 3 == 0 {
                a.raw[0x66 + slot] = 0x13;
                a.raw[0x6e + slot] = [0, 1, 2, 4, 12, 255][(i / 7 + slot) % 6];
            }
        }
        let spell = Spell {
            id: (i % 13) as u8,
            force: [1, 2, 6, 12, 127, 128, 255][i / 13 % 7],
        };
        let before = a.raw.to_vec();
        shadowrealm::spell_equipment::apply_drain(rom, &mut a, spell)?;
        applied_drains.push(
            json!({"before":before,"after":a.raw.to_vec(),"id":spell.id,"force":spell.force}),
        );
    }
    fs::write(
        out.join("spellbooks.json"),
        serde_json::to_vec(
            &json!({"equipment":equipment,"applied_drains":applied_drains,"initial":books.records.iter().flatten().copied().collect::<Vec<_>>(),"drains":drains,"decisions":decisions}),
        )?,
    )?;
    Ok(())
}
