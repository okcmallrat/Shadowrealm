# Shadowrealm

Native Rust reconstruction of the Genesis version of Shadowrun, targeting a
complete Windows PC game. The current milestone is a **Windows exploration
build**, with a desktop window, persistent party state, collision-aware movement,
portraits/equipment/meters, character animation, and environmental tile updates.
Story transitions, combat, dialogue, AI, saves and audio are still unfinished.

## Run the Windows build

Download and extract [`Shadowrealm-windows-x64-exploration.zip`](releases/Shadowrealm-windows-x64-exploration.zip), put your
`Shadowrun.bin` beside `Shadowrealm.exe`, keep `libunwind.dll` in that folder,
and launch the EXE on Windows 10/11 x64. No Rust/Python/emulator installation is
needed for the package. The ROM is not bundled.

Arrow keys/WASD move, Tab switches runner, Space pauses, R resets, Escape exits.
The roster is an explicit test setup using original Joshua/Ricky/Freya templates,
not new-game recruitment. Unfinished entrances stop movement.

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

Game hardware initialization, interrupt scheduling, audio, scripts, combat,
UI state machines, saves, and playable scenes remain unimplemented. Native map,
character, and HUD rendering components are available below. No title screen or gameplay
has been reconstructed. See `docs/analysis.md` for evidence and next steps.

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
