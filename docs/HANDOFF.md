# Resume here

## User goal and expectations

Build a **complete, working native Rust game executable for the user's Windows
PC**, reconstructing the uploaded Genesis Shadowrun ROM. The user has explicitly
said the early previews are too basic/incomplete. Prioritize connected gameplay
and authentic behavior, not more isolated asset demonstrations. The current
executable is an exploration milestone, not the finished game.

The user resumed development after cloud setup. The current local milestone
adds original fallback wall hugging and physical actor-contact separation to
companion following. Next, recover controlled separation/travel effects and
combat target selection; do not restart completed ROM/map/hiring/navigation work. No clarification is pending. Do not assume runtime files outside Git persist.

The user explicitly asked to introduce JSON **slowly** for recovered dialogue
and quest logic. Keep the Rust engine and use small verified records; do not
migrate unrelated systems or invent missing quest rules.

## Durable checkpoint

Repository: https://github.com/okcmallrat/Shadowrealm
Branch: `main`
Earlier published implementation checkpoint: `e5979b9` — verified Ghidra reference and
noncombat companion following. Its source, tests, tooling and refreshed Windows
package have been pushed to `main`. The subsequent documentation-only pause
checkpoint records this stopping point; see `git log` for its commit.

Source, docs, scripts, validators, and the Windows package are on GitHub.
The last checked working tree before preparing this handoff was clean.
The resumed fallback/contact milestones are recorded together by this checkpoint.
Use `git log` for its commit ID; the earlier checkpoint above describes the base.

Windows package: `releases/Shadowrealm-windows-x64-exploration.zip`.
SHA-256: `2bf8158e2243349574d47d7f5d5762b155d3d110f1f51b000d8507dda237c2bc`.
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
- Exterior seams 0 ↔ 3/5 with ROM spawn/camera data and palette fades.
- Map 0 Space Needle entrance: original text/font, acknowledgement, map 10
  arrival and fixed idle NPC, then saved exterior camera/formation on exit.
- Contact with Trent in that interior opens recovered information branches,
  including biography, magic, contacts and goodbye. A small embedded JSON file
  holds references/actions; two recovered knowledge fields persist in session.
- Trent first-hire: capacity, quoted single-run/permanent prices, payment/funds
  refusal, member/HUD initialization, room-NPC removal and contract persistence.
  Tab transfers the controlled-player flag, so hired/switched runners use doors.
- Bounded noncombat following: formation, 21-tick decision scheduling, regular
  grid corner routes, fallback wall hugging, distance/visibility speeds, gradual turning
  and static wall sliding. The old leader follows after Tab switches control.
- Native desktop executable: WASD/arrows walk, Tab switches runner, Space pauses,
  Enter/Z acknowledges the entrance passage, R resets, Escape exits. Default map
  is 0; optional second CLI argument is map ID.
- Scripted integration replay uses the same game core as the desktop binary.

## What is NOT complete

Other interior doors/story transitions stop at a trigger. The Space Needle
path implements its traced no-mission baseline; there is no quest-state VM yet.
Combat, other branching dialogue/quest scripts, other recruiting, contract
completion/dismissal/rehire history, other NPC/combat AI, general contact/travel scripts, inventory/menu interactions, skills/magic, character creation,
missions/story state, audio, and save/load are not implemented. The host camera
is not the original look-ahead/hysteresis system. Original wall-sliding and other
special movement cases are not ported. The Windows GUI has **not** been run or
playtested on a Windows PC or Wine in this environment.

Do not describe any of those as complete because its graphics/animation assets
can be rendered. Full-game dynamic traces and scene comparisons remain needed.

## Exterior milestone

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

## Space Needle interior milestone

Read `docs/interior-transitions.md`, `src/dialogue.rs` and `src/npc.rs`.
Map 0 trigger C6 resolves map 10 after acknowledging node `110b5e`. The native
panel uses exact text `113dd6` and original font, with a host layout. Entry saves
the 96-byte exterior descriptor/map ID, camera and `bc36/bc66` return formation;
room 10's `80` exit restores them. Party state survives and the fixed NPC is
cleared on return. Two successive visits pass through the same game core,
including a controller-only turning/approach route after the first return.

The fixed NPC comes from script `17018`/prototype `171a2`, using `44bc` compact
fields and byte overrides. The subsequent JSON pilot adds a bounded conversation;
AI and recruitment/payment remain absent. No original random timers are simulated.

The ordinary suite has 41 passing unit tests; all three ROM-backed integration
tests pass explicitly. `tools/validate_interior.py` matches the original baseline
entry gate/node/text/font, route/save/spawn, return formation/restore blocks and
NPC initialization. The older 54-route/36-spawn seam oracle still passes.
The Windows package is refreshed; actual Windows GUI testing remains pending.

Generate captures/oracle data with `interior-data ROM NEW_OUTPUT_DIRECTORY`.
It seeds the player outside the real entrance at (528,144), then uses normal
input through entry/exit without assigning maps between legs. These are seeded
boundary checks, not a complete new-game traversal. Starting directly at map 10
does not execute the entrance or manufacture a return context.

## Gradual JSON / Trent conversation pilot

Read `docs/dialogue-json-pilot.md`. The user's JSON preference is also recorded
in `AGENTS.md`. `data/dialogue/space-needle.json` contains the entrance reference
and 12 Trent node references with three translated action records. Text/font
remain in the ROM. JSON is embedded in the EXE; no new user-side files are needed.
`Catalog::load` validates schema/links/flag widths and pointers against the ROM.

`eb92/eccc` contact with the unlinked fixed NPC schedules `21174`, selects Trent
(index 2) for the traced map/entrance, and opens `feb4c` in the zero-opinion scope.
Native code sets the original known-runner flag and 60-tick contact cooldown.
Dialogue freezes the world/selection. Up/down chooses, Enter/Z confirms,
left/right pages long bodies. The host panel is not the original portrait UI.

Actions 4 / BE / E5 resolve topic, biography and contact-information branches.
`Game::knowledge` preserves original `fffc2a` (runner-known) and `fffc2c` (contacts)
bits across reloads in memory. Pocket Secretary UI and disk saving are absent.
The subsequent first-hire milestone enables action 3 and its purchase options.
Other opinion/mission states, targeting and NPC AI are not implemented.

42 unit tests and all four explicitly enabled ROM integration tests pass.
`tools/validate_dialogue.py` matches 12 node/text records, 24 original branch/action
executions and 69 NPC contact/gating cases. The interaction fixture seeds a
player near the NPC after normal door entry; this is not a complete walking route
from the original new-game start. Earlier seam/interior oracles still pass.
The Windows package is refreshed; actual Windows GUI testing remains outstanding.

Serde/serde_json are now pinned runtime dependencies; the default CLI is no
longer dependency-free. New crate licenses are included by the Windows packager.

## Trent first-hire milestone

Read `docs/trent-hiring.md` and `src/hiring.rs`. The embedded JSON schema is now
version 2, still parsed with pinned Serde/serde_json. Only this dialogue file is
extended: three response nodes and contextual hiring/quote/source references.
There is no generic quest/payment language or arbitrary ROM execution.

Action 3 (`21226`) refuses when both companion slots are active; otherwise it
quotes fees from `fe664`, opinion multipliers and leader stat bytes. Current
zero-opinion Joshua quotes 285 / 2850 nuyen. Options at `ff958` omit a body-pointer
slot; do not parse them as a normal node. Action 0 / 1 pays for single-run /
permanent service. Insufficient funds returns `ff978` without mutation. Success
uses Trent template 5, the first free companion graphics/identity slot, full
health/condition, original contract byte `8f`, and the room NPC's coordinates.
It removes that NPC and refreshes the HUD. The contract and wallet survive area
reloads; original single-run expiry and rehire archives are not implemented.

`Game` prepares all text/member/HUD/graphics before committing payment or NPC
removal; an invalid-portrait test verifies rollback. Scene cloning preserves
current map VRAM/environment state. Party selection now transfers raw bit 1 in
entity byte `10` as `3992/39a8` do. This fixes switched/hired runners being unable
to satisfy a door's controlled-player gate.

Fresh exploration loads the original `7fc` bootstrap wallet (20 nuyen); this is
not the original map-2 new-game/earning loop. Default desktop roster is unchanged
(Joshua/Ricky/Freya), so hiring correctly gets the full-party refusal. Windows
testing can use `Shadowrealm.exe --solo --nuyen 3000`: explicitly seeded
development funds with Joshua alone. R restores the same selected fixture.

43 unit tests and all six explicitly enabled ROM integration tests pass.
`validate_hiring.py` compares 54 original fee quotes and 24 complete native
capacity/funds-boundary/contract purchase cases, including full party records,
money, recruit IDs and NPC removal. First-hire archive fixtures are explicit;
the oracle skips UI/VDP-only calls and adjusts the native host animation rate.
`hiring-data ROM NEW_DIRECTORY` exports those fixtures and selected quote/outcome
frames. Older dialogue/interior/seam and world/HUD checks still pass.
The Windows package is refreshed; actual Windows GUI playtesting remains pending.

## Earlier milestone: Ghidra reference and noncombat following

The user supplied the corrected `ShadowRunBin.gzf` and explicitly asked to use it
going forward. Read `docs/ghidra-reference.md`: it records the upload/file ID,
checksums, official Ghidra 12.2 / Java 25 toolchain and reproduction command.
The GZF opened successfully with 607 existing functions; hashing actual program
memory confirms all 2 MB match the supplied ROM. `tools/ghidra/ExportCompanion.java`
exports targeted function views into ignored artifacts from a private working
copy. Keep GZF/project databases/decompiled game code out of Git. Ghidra's untyped
register calling conventions can yield misleading C; verify with instructions
and the original-routine oracle.

Read `docs/companion-following.md` and `src/followers.rs`. Healthy non-selected
party runners now use `ac84/138b8` scheduling, `14508` formation, original angle/
distance tables, regular `14682/149c0` corner routes and `b574/bade` walking/sliding.
At that checkpoint, the `1470e` fallback and original scans outside the collision
raster stopped safely. The fallback and physical contact response are now
implemented as described below. Combat targeting remains unimplemented. Do not claim all-room robust navigation.

Screen projection words `a/c` are maintained at tick end for the next tick's
on/offscreen velocity choice. AI does not depend on render-call frequency. Party
slots before the selected slot use its pre-movement pose; later slots use its
updated pose. The camera is still the host implementation.

Warm-up now runs follower decisions but forces idle using `ac9e/ae5e`. This is
essential: leaving decisions frozen kept stale interior waypoints after exit.
Arrival positions still remain fixed. Tests compare persistent identity/stats/
equipment/contract fields, not AI targets/timers/samples, across transitions.

That checkpoint had 45 passing unit tests and nine explicitly enabled ROM
integration tests. Its oracle compares 1024 heading/distance pairs, 768 formation goals, 762 supported
decisions, 6 explicitly withheld raster routes, 4 guarded wall-hugging requests,
1024 movement/slide records and a 900-tick
three-runner scheduler/warm-up/wall-route trace against original execution.
`follower-data ROM NEW_DIRECTORY` also captures a 540-tick seeded desktop-core
replay with leader switching, frames and a state trace. An actual hire → exit →
follow → leader-switch integration test passes. Earlier oracles still pass.
The Windows package is refreshed; actual Windows GUI testing remains pending.

## Earlier local milestone: fallback wall hugging

`src/followers.rs` now follows the original `1470e/14af4/14800` initial fallback,
`14764/14a54` corner/clear-strip continuation, and `14914` L-path exit. The
uploaded GZF was imported into a private Ghidra working copy and its actual
program bytes rehashed against the ROM before targeted exports. No generated
analysis or game assets are tracked.

The route consumer preserves the original delayed heading/range refresh and
initial tail-call/caller ordering. `Collision::map_width_tiles` stores descriptor
word `24` verbatim for the inclusive original navigation bounds check; do not
derive that value from playable right bounds. Out-of-raster reads remain guarded.
Physical contacts and combat were excluded at this checkpoint.

That checkpoint passed 47 unit tests and ten explicitly enabled ROM integration tests. The follower
oracle adds 7,200 full-actor scheduler/movement ticks across four rotated U
pockets, 192 fallback/waypoint decisions and four fully compared closed-pocket
decisions. It requires the original initial-search, corner-search, L-path exit
and clear-strip continuation branches to execute. The existing 900-tick regular
scheduler/warm-up trace, 762 supported real-map decisions, six withheld raster
routes, 1,024 movement cases, math/formation checks remain passing. Functional
checks require fallback to clear and the companion to rejoin formation without
entering blocked footprints in each pocket orientation.

World, exterior transition, interior, dialogue and hiring original-routine checks
also pass. The native Linux desktop and Windows cross-builds/package are rebuilt;
actual Windows GUI playtesting is still outstanding. See releases/SHA256SUMS for
the refreshed package digest. Rust remains the engine and the existing typed
Serde/JSON dialogue/hiring pilot remains unchanged by this navigation work.

## Latest local milestone: physical actor contacts

Read `docs/actor-contacts.md`, `src/avoidance.rs` and `tools/validate_avoidance.py`.
`eb92` physical contact flags/headings, widened existing target-link windows,
`d44` random adjustments, and `aa88/b49c` temporary motion overrides now run in
native party slot order, including selected-player deflections and the fixed NPC.
The prior handler resumes; the original discarded third probe is preserved.

49 unit tests and all 12 explicitly enabled ROM integration tests pass. The new
oracle matches 1,536 contact cases (438 contacts, 69 random adjustments), 1,024
response steps near real collision boundaries and 2,700 full-actor/RNG ordered
ticks with all three selected slots. It exercises all probe branches, including
clear third candidates that do not move. Existing follower/world/transition/
interior/dialogue/hiring checks pass. The controller-only second door approach
uses 15 rather than 11 rightward ticks after contact changes its trajectory;
all repeated-entry and persistent-state assertions remain.

The contact oracle omits only the `eccc` script callback; the separate existing
Trent dialogue oracle remains passing. General contact/quest/combat scripts are
not implemented. Controlled separation onto triggers remains blocked; normal
player-driven travel works. Initial RNG seed zero and simulation frame parity
are explicit native host choices, not recovered hardware boot timing. Fades,
warm-up position holds and dialogue freezing remain. No JSON migration was needed
for physical motion: typed Serde/JSON dialogue/hiring records remain in use.

The native Linux desktop and Windows builds/package are refreshed. Actual
Windows GUI/input remains untested. The current local ZIP digest is recorded
above and in releases/SHA256SUMS. The fallback/contact changes are recorded by
this checkpoint; see `git log` for its commit ID.

## Best next milestone

Recover controlled separation onto triggers (`b732/ac5e/b7b8`) and its travel
side effects, then combat target acquisition before enabling attacks. Connect
one bounded source-backed job/mission flow for earning money and resolving
single-run contracts. Keep JSON increments small and verify them against original
routines. Do not claim full NPC AI or all-room robust navigation.

The ROM and corrected GZF are privately retained at `/workspace/inputs/Shadowrun.bin`
and `/workspace/inputs/ShadowRunBin.gzf`. Rust activation remains below. Python
validators now use `/workspace/.venvs/shadowrealm/bin/python`; Ghidra/Java are
under `/workspace/reverse-tools` as documented. Private analysis project:
`artifacts/ghidra-projects/FallbackReference.gpr`; exports:
`artifacts/fallback-reference/ghidra` and `artifacts/avoidance-reference/ghidra`.
Do not commit these inputs or outputs.

Trace `1651e → 16608 → 167ae`: source descriptor offset 4e supplies target-location
mapping; `16e30` saves the exterior map before the location reload. Trace spawn
selection and side effects for one bounded location type, then `2f32` exterior
restore and `bc36/bc66` return-formation data. Avoid bypassing dialogue/mission
requirements by mapping every trigger byte directly to a map ID.

Existing native code still reports other unsupported interior triggers. Start
from `docs/interior-transitions.md`, `docs/area-transitions.md`, the main-loop
notes and original routine listings. Mission/fee/conditional doors need their
own state model and script handling before entry can be enabled.

## Code map

| Path | Responsibility |
| --- | --- |
| src/game.rs | Persistent exploration and transition phase integration |
| src/transitions.rs | Exterior routing, bounded interior entry and saved return |
| src/dialogue.rs, src/npc.rs | Bounded exposition/font and fixed NPC initialization |
| src/dialogue_data.rs, data/dialogue/space-needle.json | Validated embedded dialogue pilot and flag actions |
| src/hiring.rs | Bounded Trent quote/payment/member initialization |
| src/followers.rs | Noncombat scheduler, formation, regular/fallback routes and walking |
| src/avoidance.rs | Physical actor contact flags/headings/RNG and temporary separation motion |
| tools/ghidra/ExportCompanion.java | Targeted views from the uploaded analysis reference |
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

Corrected Ghidra upload: `sediment://file_00000000181881f5a9d1370f120f2f2a`.
See `docs/ghidra-reference.md` for its verified digest and workspace/tool paths.

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
The desktop feature uses pinned minifb 0.28.0; JSON support uses pinned serde
1.0.229 and serde_json 1.0.154. All runtime code remains native Rust.

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

- 49 Rust unit tests pass; 12 ROM-backed integration tests pass explicitly.
- Physical contacts: 1536 cases, 1024 response steps and 2700 ordered party ticks
  match actor bytes and RNG state (`tools/validate_avoidance.py`).
- Noncombat followers: 1024 math pairs, 768 goals / 762 supported decisions,
  1024 movement records, 900 regular scheduler/warm-up ticks, 7200 wall-hugging
  ticks, 192 fallback/waypoint decisions and 4 closed-pocket decisions match;
  6 out-of-scope raster routes remain explicitly withheld
  (`tools/validate_followers.py`).
- Trent first-hire: 54 original fee quotes and 24 complete purchase/capacity/funds
  cases match (`tools/validate_hiring.py`).
- Trent dialogue: 15 node/text records, 26 original text/flag branch executions
  and 69 contact/gating cases match (`tools/validate_dialogue.py`).
- Space Needle entry gate/text/font, route/save/spawn/return and fixed NPC
  initialization match bounded original routines (`tools/validate_interior.py`).
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
