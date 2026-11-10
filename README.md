# Shadowrealm

This a project for myself and is for my educational purposes only - *It's just a freaking project of mine*

Normal desktop launch now opens **New Game / Load Game**. Choose Samurai,
Decker or Gator Shaman to start Joshua alone in Redmond with the original stats,
equipment, spellbook and 20 nuyen. Load Game restores an existing native slot.
R opens this menu during play; Resume keeps the suspended game. `--explore`
launches the prior Joshua/Ricky/Freya development roster. See
[new-game scope and checks](docs/new-game-and-opening-menu.md).

Native Rust reconstruction of the Genesis version of Shadowrun, targeting a
complete Windows PC game. The current milestone is a **Windows exploration
build**, with a desktop window, persistent party state, collision-aware movement,
portraits/equipment/meters, character animation, and environmental tile updates.
Exterior seams between maps 0, 3 and 5 now work with fades and persistent party
state. The Space Needle entrance now shows its ROM passage, loads the interior,
and returns to the saved exterior formation. Trent now has a bounded conversation
with recovered information/knowledge branches and bounded hire/rehire flow,
using a small Serde/JSON pilot. General story/dialogue scripts,
combat, AI, original menus and audio remain unfinished.
Non-selected companions now perform bounded noncombat following, formation,
static wall sliding and recovered fallback wall hugging. Party/NPC overlap now
uses the original physical separation response, including player deflections
and contact-triggered travel through the verified seams and Space Needle path.
**Hold F to attack** with the selected runner's equipped firearm, supported spell
or fists. Tap T to toggle
target selection, or fire in the facing direction without a target. The native
game loop now runs cooldowns, finite ammunition/reloads, flashes/impacts, burst
continuations, damage and hurt/death setup. See [combat scope and verification](docs/combat-targets.md).
Blocked projectile probes show red target brackets; shots still obey collision.
See [companion behavior](docs/companion-following.md) and
[actor contacts and limits](docs/actor-contacts.md). Combat AI remains unfinished.

Uncontrolled mundane firearm companions now target, pursue, shoot and reload
through the original companion scheduler. Joshua uses this path after Tab
transfers control. Ricky/Freya now automatically choose supported learned attack
spells or Heal and restore inventory firearms when the original drain gate
rejects magic. No M/G/F input is needed for these companion decisions.
See [companion combat scope and checks](docs/companion-combat.md).

Typed Serde/JSON catalogs describe verified spellbooks and effect references;
Rust executes the rules. Flame Dart/Bolt, Mana Zap/Blast and Heal now run through
original rolls, drain and timed effects. Attack spells apply original resistance,
damage and reactions; Heal restores party health/condition and fallen-state flags.

Try Mana Blast with `--explore`: Tab to Freya, M once from her starting
weapon, T to target and F to cast. M cycles actual known spells/forces; G restores
an inventory weapon. For Heal, cycle M until Heal and use P to choose a party
member. M/G/P remain host shortcuts while the original menus are pending.
Automatic attack/Heal choices now connect equipment conversion, enemy targeting,
blocked-path recovery, cooldown movement and critical-party healing. The `--explore`
party completes magic in a real map-5 encounter using idle input alone. Special
register contexts, automatic status spells, other spell actions and full magic AI
remain unfinished. See [automatic companion magic](docs/automatic-companion-magic.md),
[companion casting movement](docs/companion-casting-movement.md),
[mana evidence and limits](docs/mana-spells.md),
[healing](docs/healing.md), [casting](docs/spell-casting.md), and
[equipment/drain](docs/spell-equipment.md).

Invisibility and Rockskin now use original instant status initializers, clocks
and tile transforms. Freya knows Invisibility and Ricky knows Rockskin; M selects,
P cycles living party targets and F casts. G restores a weapon while retaining
the status. See [status spell behavior and original quirks](docs/status-spells.md).

The first repeatable job loop is now connected: Gunderson's Redmond courier
runs support generated offers, pickup, delivery, nuyen/Karma payment and another
offer. Launch `Shadowrealm.exe Shadowrun.bin 2`, enter the Jump House on the
north side and approach the upper-left booth. J reviews the current run.
Courier offers can be accepted; other offered job types remain unavailable.
See [courier gameplay and original checks](docs/courier-job-loop.md).

New games start in Redmond (map 2), where this route is reachable.
Earn 250 nuyen through courier runs, then visit Stoker's Coffin Motel and ask for
Michael's belongings. Paying his back rent recovers the original stash, 500 nuyen,
Karma and DocWagon wristband, and unlocks taxi travel. Press **C** on a city street
to choose among the six city districts. Story gates, fares, free rides when short
of funds and descriptor arrivals use the recovered rules. The connected test
completes six courier runs, buys the belongings and arrives Downtown without
assigning money, positions, stats or job outcomes.
See [first objective and city travel](docs/stoker-and-taxi.md).

Downtown’s **Gates Undersound Hotel** now rents a 75-nuyen room for partial
rest and Karma advancement. Choose a runner, an attribute/skill, and confirm
the quoted cost; check out before saving. Courier-earned Karma can improve
Joshua and companions, and those upgrades survive native save/load.
See [hotel rules and validation](docs/hotel-and-karma.md).

Stoker's now rents its original **20-nuyen room**, including rest and Karma
spending. Post-objective visits execute the actual random Halloweener encounter:
one gunner and two unarmed attackers. NPC punches, condition knockouts and normal
medical recovery for all three Joshua archetypes are connected. An earned route
survives the fight, rests, improves Body and reloads its checkpoint. See
[Stoker combat, rest and source checks](docs/stoker-ambush-and-rest.md).

Player and companion **barehanded combat** is now connected. In **I**, select the
currently equipped firearm again to put it away; **T** selects a target and **F**
swings. Walk into close range. Normal fists damage condition; original cyberware
can make the attack lethal. Companions pursue and punch automatically, including
after exhausting ammunition and reserves. Stored ammunition survives fists and
native save/load. See [party unarmed combat and checks](docs/party-unarmed-combat.md).

Redmond's Ares Weapon Emporium now sells firearms and ammunition with original
stock, prices, negotiation, inventory-space and clip-limit rules. Press **I**
between actions to select an owned firearm; F fires it and reloads from reserves.
A connected test earns the money, buys a Warhawk and a clip, equips it and fires
through a reload without assigning money, gear or job outcomes.
See [Ares equipment and verification](docs/ares-equipment.md).

**F5 saves / F9 loads** one of two native checkpoints from a safe city street.
Money, purchased equipment/ammo, party archives, spellbooks, courier/package state,
story progress and positions survive closing the game. Saves live beside the EXE
in `saves/`, with a previous valid backup; failed loads retain the current game.
The earned-money/Ares route now saves to disk, loads in a separate process and
resumes firing the purchased Warhawk with the correct resources.
See [native save/load scope and checks](docs/native-save-load.md).

For resuming development, start with [the project handoff](docs/HANDOFF.md).
The corrected uploaded [Ghidra analysis reference](docs/ghidra-reference.md) is now
part of the reverse-engineering workflow; the desktop runtime remains native Rust.
Optional improvements are collected in [project ideas](docs/PROJECT_IDEAS.md).

## Run the Windows build

Download and extract [`Shadowrealm-windows-x64-exploration.zip`](releases/Shadowrealm-windows-x64-exploration.zip), put your
`Shadowrun.bin` beside `Shadowrealm.exe`, keep `libunwind.dll` in that folder,
and launch the EXE on Windows 10/11 x64. No Rust/Python/emulator installation is
needed for the package. The ROM is not bundled.

Arrow keys/WASD move, F fires, I selects a firearm, C calls a taxi, Tab switches runner, T toggles target selection outside
dialogue, Enter/Z acknowledges the entrance
passage or dialogue choice, F5 saves, F9 loads, Space pauses, R opens the start menu, Escape exits. During dialogue,
Up/Down selects a choice and Left/Right pages long text. Trent can be hired when
there is a free companion slot and sufficient nuyen.
Normal New Game starts Joshua alone. `--explore` selects the explicit
Joshua/Ricky/Freya development roster. R resets directly in development mode. Unfinished interior entrances still stop movement.

To test hiring with a free slot and explicitly seeded development funds, launch
`Shadowrealm.exe --solo --nuyen 3000`. The normal fresh-session balance is 20,
read from the original bootstrap. The Redmond courier loop now earns nuyen and
Karma; other job objectives, wilderness travel and general spending menus remain
unimplemented.
See [hiring evidence and limits](docs/trent-hiring.md).

The executable was cross-built and inspected here. Windows GUI/input testing
has not yet been performed. See [build details and remaining milestones](docs/exploration-and-windows.md)
and [package instructions](docs/WINDOWS-README.txt).

## Build and inspect

Use stable Rust (validated with Rust 1.99.1). In this cloud workspace:

```sh
export CARGO_HOME=/workspace/.rust/cargo
export RUSTUP_HOME=/workspace/.rust/rustup
export PATH="$CARGO_HOME/bin:$PATH"
cd /workspace/Shadowrealm
cargo test --locked
cargo build --release --locked
cargo run --locked -- inspect /path/to/Shadowrun.bin
cargo run --locked -- strings /path/to/Shadowrun.bin
cargo run --locked -- tile /path/to/Shadowrun.bin 0x200 tile.ppm
cargo run --locked -- region-screen /path/to/Shadowrun.bin region-screen.ppm
```

The tile command interprets 32 bytes as an uncompressed 8x8 tile and writes a
grayscale PPM. It does not establish that the selected offset contains graphics,
recover palettes, decompress assets, or reconstruct a scene.
`inspect` returns an error for a checksum mismatch. A checksum match is an
integrity check, not proof of ROM identity.

## Disassembly

`tools/disassemble.py` requires Python and `capstone==5.0.9`. Install it into a
virtual environment, then run:

```sh
python tools/disassemble.py /path/to/Shadowrun.bin --start 0x236 --end 0x2c4
```

The generated listing in `docs/startup.m68k.txt` covers the reset routine and
exception stubs. Linear disassembly stops at undecodable input and cannot
automatically distinguish data from instructions. Resume at known code addresses;
do not interpret the initialization table at 0x2c4 as instructions.

`src/boot.rs` translates reset and region decisions. `src/video.rs` reconstructs
boot-font decoding, the region-error screen, and palette fade routines.
`src/input.rs` implements controller held/pressed state and direction decoding.
The `region-screen` command previews the mismatch screen using ROM text and font,
even when the ROM matches the console region. It writes a 320x224 PPM image.
Its fixed asset offsets target the supplied revision, not arbitrary Genesis ROMs.

Console hardware initialization, audio, general scripts, combat, original menus and the
original title/intro cinematics remain unfinished. Native exploration and bounded dialogue/
hiring now use the map, character and HUD components below. The early reverse-
engineering notes in `docs/analysis.md` predate those milestones.

The supplied-ROM functional check is repeatable with:

```sh
python3 tools/validate_rom.py /path/to/Shadowrun.bin
```

Build the release binary first. This verifies ROM identity, the direction table,
and a pixel comparison against an independent transcription of the font's 68000
bit-expansion loop. It is not an emulator or a hardware screenshot comparison.


## Map extraction and loop trace

See [main loop and map loading](docs/main-loop-and-maps.md) for the traced
control flow, descriptor fields, compression, RAM addresses, and open questions.

```sh
cargo run --locked -- maps /path/to/Shadowrun.bin
cargo run --locked -- map /path/to/Shadowrun.bin 0 artifacts/map-00
```

Create the parent `artifacts` directory first. The extraction destination must
not already exist, to avoid overwriting earlier output. The `map` command writes
the 96-byte descriptor, two exact 7200-byte work-buffer copies, source-pointer
metadata, and decoded shared tile definitions when the source is compressed.
For uncompressed definitions it records pointers; their raw lengths are unknown.
These outputs contain ROM-derived data and are ignored by Git.
They do not constitute a rendered map or recovered collision data.

`src/maps.rs` implements the descriptor reader, original copy ranges, map flag,
coordinate offsets, and decompressor. `src/frame.rs` translates frame waits into
nonblocking polls for a future native scheduler; it does not execute gameplay.

For differential validation, install `unicorn==2.1.4` in a Python environment:

```sh
cargo build --release --locked
python3 tools/validate_maps.py /path/to/Shadowrun.bin
```

This checks all map entries and executes the original 68000 decompressor for
each compressed map, comparing bytes and source consumption with the Rust output.
In the prepared cloud workspace, validation dependencies are available with
`PYTHONPATH=/workspace/.reverse-tools`.


## Render a map

```sh
mkdir -p artifacts
cargo run --locked -- render-map /path/to/Shadowrun.bin 0 0 0 artifacts/background.ppm 960 640
```

Arguments: ROM, map ID (0–55), world X/Y in pixels, new PPM output path,
optional width/height (default 320×224). Renders both background planes with
the recovered graphics, palette loading, tile flips, transparency, and priority.
This command exports static background planes. Use the animation command below
for character frames and HUD composition; dynamic gameplay effects remain unimplemented.
See [rendering evidence and limitations](docs/map-rendering.md).

```sh
cargo build --release --locked
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_render.py /path/to/Shadowrun.bin
```

The validator requires Unicorn 2.1.4 and compares all 56 map crops with a separate
Python renderer using the original 68000 routine for compressed definitions.


## Characters, HUD and animation

```sh
cargo run --release --locked -- render-animation /path/to/Shadowrun.bin 0 5 8 artifacts/animation 96
python3 tools/make_preview.py artifacts/animation
```

The output directory must be new. Native output is 96 PPM frames and an auditable
frame-address/clock CSV; the optional Pillow script produces PNG/GIF previews.
The example uses sample party values and a selected animation, not live gameplay.

All 16 banks (885 sequences) can be inspected; bank/state ranges differ.
Portrait and equipment rendering APIs accept resolved asset pointers; automatic
selection from gameplay state remains outstanding. See
[formats, validation and limits](docs/characters-hud-animation.md).

```sh
cargo test --locked
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_actors.py /path/to/Shadowrun.bin
cargo run --release --locked -- actor-catalog /path/to/Shadowrun.bin artifacts/actors.csv
```

`actor-catalog` exports sequence pointers and decoded sprite pieces to a new CSV.
`hud-assets ROM NEW_DIRECTORY` exports the sample HUD window/VRAM used by the
original-routine differential validator. No ROM or extracted assets are bundled
in tracked project files.


## Stateful exploration checks

```sh
cargo run --release --locked -- replay /path/to/Shadowrun.bin 0 scenarios/exploration.csv artifacts/exploration
python3 tools/make_preview.py artifacts/exploration
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_world.py /path/to/Shadowrun.bin
```

The replay CSV specifies `ticks,buttons,slot` using the recovered controller
bitmask. This drives the same native gameplay core as the Windows executable.
It exports a state trace and images; it does not emulate or record the full game.
`world-data ROM MAP NEW_DIRECTORY` exports the native collision raster, query
results, map-animation VRAM steps, and party HUD state for differential checks.

Desktop build (with the platform's C build tools installed):

```sh
cargo build --release --locked --features desktop --bin shadowrealm-game
```

The default CLI binary remains `shadowrealm`; desktop dependencies are optional.


## Transition checks

```sh
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test transitions -- --ignored
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_transitions.py /path/to/Shadowrun.bin
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_interior.py /path/to/Shadowrun.bin
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_dialogue.py /path/to/Shadowrun.bin
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_hiring.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test followers -- --ignored
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_followers.py /path/to/Shadowrun.bin
```

The ROM-backed checks drive round trips between the connected exterior maps and
and through the Space Needle entrance, verifying source-derived behavior against
original 68000 routines. See [exterior evidence](docs/area-transitions.md) and
[interior evidence and limits](docs/interior-transitions.md). Default unit
tests leave the ROM-backed tests ignored; execute the command above explicitly.

The gradual [dialogue JSON pilot](docs/dialogue-json-pilot.md) covers this one
interaction. Rust remains the engine; the embedded JSON stores verified ROM
references and a few recovered flag effects, with text still read from the ROM.

Ordinary firearm NPCs now acquire party targets, pursue and fire. Supported street
encounter groups now spawn through the recovered district/RNG and placement rules.
Try `Shadowrealm.exe Shadowrun.bin 5 --solo` for a reproducible initial encounter.
Scripted encounters, creatures/magic and full defeat/restart remain incomplete.
See [street encounter scope and verification](docs/street-encounters.md).

Normal Joshua defeat recovery now supports companions: their equipment, stats
and contracts are archived in memory before removal, and Joshua returns through
Little Chiba. Rehiring Trent preserves those fields and uses his original
negative-opinion dialogue and fees. Insured recovery also implements both DocWagon
passages and the Seattle General arrival; insurance purchasing is still absent.
Mission-specific recovery, other runner recruitment, original save UI and broader companion
combat remain unfinished. Enter/Z acknowledges each recovery passage.

Normal defeat now archives and removes companions before Joshua's medical recovery.
Trent retains his equipment/stats on rehire, with original negative-opinion dialogue
and fees. Insured recovery has both DocWagon passages and the hospital arrival;
coverage purchasing and mission-specific recovery remain unfinished. These archives
now persist through native checkpoints. See [recovery scope and evidence](docs/defeat-and-recovery.md)
and [save/load](docs/native-save-load.md).
