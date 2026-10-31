# Resume here

## User goal and expectations

Build a **complete, working native Rust game executable for the user's Windows
PC**, reconstructing the uploaded Genesis Shadowrun ROM. The user has explicitly
said the early previews are too basic/incomplete. Prioritize connected gameplay
and authentic behavior, not more isolated asset demonstrations. The current
executable is an exploration milestone, not the finished game.

The user resumed from the saved checkpoint. The exterior seam milestone below
is now implemented; no clarification or approval is pending. On resuming again,
inspect repository status and continue the next interior-transition milestone. Do not assume that the prior runtime/processes/files outside Git persist.

## Durable checkpoint

Repository: https://github.com/okcmallrat/Shadowrealm
Branch: `main`
Uploaded implementation checkpoint before these handoff notes:
`b9b57485d83b6df4c6a430ea3831dac055c81241`.

Source, docs, scripts, validators, and the Windows package are on GitHub.
The last checked working tree before preparing this handoff was clean.
See `git log` for the subsequent handoff commit and any newer work.

Windows package: `releases/Shadowrealm-windows-x64-exploration.zip`.
SHA-256: `d4d8d39000d7360833d3e1c6bc550c3aac048eecf800a5aefa8b655df4693f22`.
The package includes EXE, required libunwind.dll, notices, file hashes and run
instructions. The ROM is excluded. The user must extract the ZIP and put their
`Shadowrun.bin` beside `Shadowrealm.exe`.

## What actually works

- ROM parser, header/checksum/vector inspection, text and raw tile extraction.
- 56 map descriptors and their layer copies; native decompression of the 15
  compressed shared tile-definition entries.
- Both map planes with palette loading/overrides, tile flips, transparency and
  priority. B low → A/window low → sprite low → B high → A/window high → sprite high.
- All 16 animation banks: 885 sequences and 987 distinct sprite frame records.
- Native sprite-piece drawing, column-major sprite tiles, animation clock,
  fixed-point projection and descending-world-Y depth sorting.
- Right-side HUD window, portraits/crops, equipment icons, health, secondary
  condition, item resource meters, and selected-party highlighting.
- Party templates with actual names/assets: Joshua's three archetypes and ten
  named runners. Current desktop test roster is Joshua/Ricky/Freya, explicitly
  not a new-game recruitment state.
- Persistent exploration state, directional input, turning, walking/idle state
  selection, static collision, controlled-runner switching and a host camera.
- Environmental tile animation on 24 map records, connected to the native VRAM.
- Native desktop executable: WASD/arrows walk, Tab switches runner, Space pauses,
  R resets, Escape exits. Default map is 0; optional second CLI argument is map ID.
- Scripted integration replay uses the same game core as the desktop binary.

## What is NOT complete

Interior doors/story transitions currently stop at a trigger. Exterior seams
0 ↔ 3/5 are implemented, as described in the latest milestone. Combat, dialogue, NPC and
follower AI, inventory/menu interactions, skills/magic, character creation,
missions/story state, audio, and save/load are not implemented. The host camera
is not the original look-ahead/hysteresis system. Original wall-sliding and other
special movement cases are not ported. The Windows GUI has **not** been run or
playtested on a Windows PC or Wine in this environment.

Do not describe any of those as complete because its graphics/animation assets
can be rendered. Full-game dynamic traces and scene comparisons remain needed.

## Latest resumed milestone: exterior seams

Read `docs/area-transitions.md` and `src/transitions.rs`. Native gameplay now links
0 ↔ 3 and 0 ↔ 5 through the original seam triggers, with party-state preservation,
destination reload, recovered spawn/camera words, palette fades and warm-up.
There are 40 passing unit tests plus two explicitly executed ROM-backed tests.
The routine-level oracle matches 54 route cases and 36 spawn applications.
The Windows exploration package is refreshed, but Windows GUI testing is still pending.

`Game::transition` blocks input and party selection during fades/reload. The
visible map width is 272 pixels with the HUD; arrival cameras can extend plane-B
fetches into the next linear grid row beneath the window. Do not restore the old
320-pixel validation/clamp in the composed game frame.

## Best next milestone

Implement one ordinary interior door AND its return path, preserving the exterior
map and formation. Current tests seed a player just outside a real exterior seam;
they are not a complete new-game traversal or an interior-door implementation.

Trace `1651e → 16608 → 167ae`: source descriptor offset 4e supplies target-location
mapping; `16e30` saves the exterior map before the location reload. Trace spawn
selection and side effects for one bounded location type, then `2f32` exterior
restore and `bc36/bc66` return-formation data. Avoid bypassing dialogue/mission
requirements by mapping every trigger byte directly to a map ID.

Existing native code still reports unsupported interior triggers. Start from
`docs/area-transitions.md`, `docs/main-loop-and-maps.md`, and the original routine
listings. Then restore NPC/companion dispatcher and interaction/dialogue logic.

## Code map

| Path | Responsibility |
| --- | --- |
| src/game.rs | Persistent exploration and transition phase integration |
| src/transitions.rs | ROM exterior route and spawn records |
| src/bin/game.rs | Native desktop window and keyboard/timing loop |
| src/party.rs | Templates, identity/assets, equipment resolution, facing |
| src/collision.rs | Rectangle raster and footprint/shape queries |
| src/environment.rs | Original map-tile animation scheduler |
| src/actors.rs | Sprite records, sequences, clock and projection |
| src/hud.rs | HUD/window assets and state-dependent cells |
| src/map_render.rs | VRAM/assets, map planes and scene pixel composition |
| src/scene.rs | Map/actor/window integration |
| src/maps.rs | Descriptors, layer data, decompressor |
| src/input.rs, src/frame.rs | Controller state and frame synchronization |
| src/main.rs | Reverse-engineering CLI, replay/data export |
| tools/validate_*.py | ROM/original-M68000 differential checks |
| scripts/build-windows.sh | Validated cross-build and packaging entry point |

The addressed `docs/*.m68k.txt` listings are useful evidence. A few linear
listings end at data or a deliberately bounded range; they are not proof that
all bytes in a region are instructions.

## Input and environment recovery

ROM name: `Shadowrun.bin`, 2,097,152 bytes.
SHA-256: `92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e`.
Product: `GM MK-1352 -00`; Genesis checksum `6baf`.
Original upload reference: `sediment://file_00000000093081f58a1a7292f0b452c5`.

Prior runtime path (check existence; do not assume it survives):
`/workspace/attachments/c2f645b4-e40f-434b-9eac-f2ae8fd09636/Shadowrun.bin`.
If needed, use the upload download tool with the original file ID. Ask the user
to attach it again only if it cannot be recovered. Do not obtain a different ROM
from the Internet or commit this one to GitHub.

Prior checkout: `/workspace/Shadowrealm`.
Prior Rust installation (not in Git):

```sh
export CARGO_HOME=/workspace/.rust/cargo
export RUSTUP_HOME=/workspace/.rust/rustup
export PATH="$CARGO_HOME/bin:$PATH"
```

Rust 1.99.1 was used. Fresh machines need stable Rust plus the desktop C toolchain.
Python validation dependencies in the prior runtime were installed under
`/workspace/.reverse-tools`: Capstone 5.0.9, Unicorn 2.1.4. Pillow converts
native PPM output to shareable PNG/GIF. These tools are not game runtime dependencies.
The desktop feature uses pinned minifb 0.28.0; the default CLI remains dependency-free.

Read the cloud runtime skill and network policy before network/setup work in a
managed environment. GitHub authentication was supplied by the existing HTTPS
Git proxy. Read and push access worked; no personal token was needed.

## Rebuild and verify

```sh
cd /workspace/Shadowrealm
cargo test --locked
cargo build --release --locked
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_world.py /path/to/Shadowrun.bin
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_actors.py /path/to/Shadowrun.bin
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_render.py /path/to/Shadowrun.bin
```

Run checks relevant to the change. Current completed evidence:

- 40 Rust unit tests pass; both ROM-backed transition tests also pass when explicitly enabled.
- All 56 collision rasters, 7,168 query samples, and 16 VRAM steps per map
  match original routines, including 24 maps with animations.
- Populated party HUD matches original `334c → 3404 → 39f2` output.
- 987 sprite frame geometries and a 96-tick animation timeline match original routines.
- All 56 map crops previously matched an independent renderer using original
  M68000 decompression where applicable.
- Two 156-tick exploration replays advanced all three controlled runners and
  exercised blocking; map 0 also exercised camera tracking.

Reproduce a persistent-state integration replay (output directory must be new):

```sh
cargo run --release --locked -- replay /path/to/Shadowrun.bin 0 scenarios/exploration.csv artifacts/resume-check
python3 tools/make_preview.py artifacts/resume-check
```

## Windows build specifics

The validated Linux cross-build target is `x86_64-pc-windows-gnullvm`.
The plain `x86_64-pc-windows-gnu` target failed because the portable LLVM toolchain
lacks the GCC runtime libraries expected by that Rust target. Do not paper over
that mismatch by disabling verification or renaming a Linux binary.

Official LLVM-MinGW UCRT release: `20261020`.
Compiler archive: `llvm-mingw-20261020-ucrt-ubuntu-22.04-x86_64.tar.xz`.
SHA-256 verified against its official GitHub release:
`55064b60b991a1e1e8090b8c2f84e82b22cfe5903d12c159d65064cf5e8ae335`.
Prior extraction: `/workspace/llvm-mingw-20261020-ucrt-ubuntu-22.04-x86_64`.

```sh
rustup target add x86_64-pc-windows-gnullvm
export LLVM_MINGW_ROOT=/path/to/llvm-mingw-20261020-ucrt-ubuntu-22.04-x86_64
scripts/build-windows.sh
```

This writes the package under `artifacts/windows`. The versioned package in
`releases/` is a deliberate checkpoint; after a verified new build, copy it
there and update `releases/SHA256SUMS`. Check archive hashes and DLL imports.
The cross-built EXE imports the bundled `libunwind.dll`; its other imports and
that DLL's imports are Windows system libraries. Windows 10/11 x64 is the
intended package platform. Actual Windows GUI validation remains outstanding.

## Important compatibility findings

- Collision map 31 contains a rectangle with top `1a60`, bottom `2bf`. The original
  writes outside its collision raster into RAM `ff4bcd`, aliasing map layer 1.
  Native code records and applies that alias safely. Do not silently patch the ROM.
- The outer footprint strip outside the modeled collision raster is conservatively
  blocked; the original can access other RAM there.
- Party/HUD values are derived from actual templates, but the test roster and host
  camera are not a reconstructed new-game/save state.
- Actor state tables store byte offsets in the original entity; native APIs often
  expose state indices. Preserve the factor-of-four conversion.
- Sprite tiles are column-major; background blocks are row-major.
- HUD register 3 selects window F000, while register 5 selects the sprite table.
- `1232` preserves A1; repeated HUD layout calls reread their source template.
- `566c2` overrides palette banks 1..3 after initial setup. Ignoring it produces
  incorrect bright/orange map colors.

## Collaboration preference

Continue useful authorized work autonomously. Keep updates concise and concrete.
The user does not want repeated permission prompts or a series of basic demos
presented as a completed game. State what is verified and what remains.
