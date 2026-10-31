# Stateful exploration and Windows desktop milestone

The project now has a native desktop entry point, `src/bin/game.rs`, backed by
`src/game.rs`. The Windows target is an x86-64 PE GUI executable, not a renamed
Linux binary or a launcher for Python/emulation. It reads the supplied ROM as
an asset database. No game code is emulated by the desktop runtime.

## Persistent party integration

The ROM copy routines at `42b0`/`4350` identify full 256-byte party templates at
`7291a + index*100`. Verified templates 0..12 contain names, animation banks,
palettes, portrait pointers/crops, equipment IDs, health, and other entity state:

- 0..2: Joshua, with Samurai, Decker, Gator Shaman selection order confirmed
  by menu text at `1b64cf`.
- 3..12: Ricky, Winston Marr, Trent, Petr Uvehr, Walking Bear, Phantom, Two Fists,
  Freya, Rianna, Stark.

The following record at `7361a` is the generic entity initializer, not another
named runner. `party::Runner` preserves the raw record and exposes the fields
used by rendering and exploration. The desktop roster [0,3,10] is explicitly a
test roster (Joshua/Ricky/Freya), not a reproduction of new-game recruitment.

The three spawn-coordinate pairs come from descriptor offset `42`. The original
party tile bases are 1, 19, 31 (hex); companion transfer destinations are 320/620.
HUD portrait sources are entity longword `50` plus word crop `54`. Equipment
icons resolve through the 48-byte item records at `d4540` or `d4ea0`, according
to signed entity field `56`. Item resource meters are clamped to recovered item
capacities. Health, condition, active-panel highlighting, portraits, equipment,
and resource meters are all populated from these records, not sample literals.

The original selected-panel refresh copies the highlight template **before**
reapplying portrait/equipment cells. Doing that afterward blanks the portrait.
Native `Game::refresh_hud` follows the recovered ordering. The complete window
and portrait/equipment graphics match original `334c → 3404 → 39f2` execution
for the same real-template state.

## Movement, facing and composition

Keyboard direction bits use the existing recovered input table. Selected-runner
facing turns in steps of `800` using the modular comparison at `b530`; idle and
walking states use the original octant maps at `c052/c062`. The normal player
movement vectors come from `bec2`. World coordinates remain 1/16 pixel units.
Sprite projection uses `5a56`; actor sorting follows descending world Y as in
`5b1e`. Animation phase persists when the runner moves, turns or changes state.

The desktop uses a fixed 60-tick simulation accumulator. Pausing stops simulation;
switching the selected runner changes the controlled actor and HUD highlight.
The host camera follows the controlled actor with bounds. It is an exploration
camera, not a port of the original look-ahead/hysteresis routine.

Movement currently accepts a clear candidate and blocks positive collision or
an unresolved transition. Original diagonal/cardinal wall sliding, special
movement states, companion following, collision-trigger story effects, combat,
and the wider entity AI dispatcher remain to be translated. A blocked entrance
is reported in the window title; no unimplemented room script is fabricated.

## Collision raster and queries

Descriptor offset `26` points to ten-byte rectangle entries consumed by `ee10`:
left, right, top, bottom, and a low-byte code. A negative left word terminates
the list. Rasterization writes 32-pixel cells into the 64×64 byte grid at
`ff1700`, using exact coordinate increments and inclusive bounds.

The native query translates the player footprint at `e7c8/e8a0`: corners offset
±12 horizontally and ±7 vertically. It preserves unsigned maximum-code selection,
7f hard-block precedence, sign-coded transition returns, rectangle overlap tests,
and the four diagonal shape tests at `ead2`. Transition codes are returned to
the exploration controller, which stops pending unsupported script handling.
The very outer strip where the footprint falls outside the modeled raster is
conservatively blocked, rather than reading unrelated RAM.

One source anomaly was found in map 31: rectangle 32 has top `1a60` and bottom
`2bf`. The original rasterizer still writes one byte outside the collision grid,
at RAM `ff4bcd`, aliasing the second copied map grid. Native loading records
out-of-grid writes and applies the observed alias safely to the matching grid.
Unknown RAM aliases are rejected; the ROM is not patched. The all-map original
routine checks include this case.

## Environmental tile animation

Descriptor `3c` points to base/spec pairs registered by `164a8`, into up to ten
scheduler slots. Native `environment.rs` implements their delay/frame progression
at `1641a`, index lists, and 32-byte graphics transfers. Frame selection advances
before the first transfer and the delay period includes the following zero tick.
The graphics source is descriptor `38`; source/destination offsets preserve
16-bit shift and signed address-add behavior.

The ROM supplies these animations on 24 map records (map 5 and maps 21..43).
The desktop ticks them with actor updates. They modify native VRAM, so the
existing map renderer displays their changes without redrawing fake overlays.

## Validation performed

- 37 Rust unit tests pass.
- Two 156-tick scripted exploration runs (maps 0 and 5) render persistent state.
  Each selected runner moves; blocked movement is observed; camera changes are
  observed on map 0. These are integration checks, not full-game replays.
- All 56 collision rasters and 7,168 query samples match original M68000 routines.
- Sixteen consecutive VRAM updates for each map match the original scheduler,
  including all 24 animated maps.
- Real-template party HUD window/portrait/equipment output matches the original
  refresh, including health, condition, and resource meter tables.
- The Windows executable cross-builds; its PE GUI architecture and DLL imports
  are inspected. Only Windows system DLLs and the bundled libunwind.dll are
  imported; that DLL's imports are also inspected.

No Windows desktop runtime or Wine is installed here. Actual Windows window
creation, keyboard input, resize behavior and performance remain unverified.
Cross-compilation is not being reported as a Windows playtest.

## Windows build and package

The ready-to-try ZIP is generated under
`artifacts/windows/Shadowrealm-windows-x64-exploration.zip`. It includes
Shadowrealm.exe, its LLVM unwind runtime, licenses, hashes, and run instructions.
It excludes the ROM. Put the supplied `Shadowrun.bin` beside the extracted EXE.

On Windows, build the `shadowrealm-game` binary with a Rust Windows toolchain
and the corresponding C build tools:

```
cargo build --release --locked --features desktop --bin shadowrealm-game
```

That native Windows build command is documented, not verified in this Linux
session. The validated cross-build uses Rust target
`x86_64-pc-windows-gnullvm` and official LLVM-MinGW UCRT release 20261020.
The compiler archive was verified against the SHA-256 published on its official
GitHub release: `55064b60b991a1e1e8090b8c2f84e82b22cfe5903d12c159d65064cf5e8ae335`.
The ordinary GNU Rust target requires GCC runtime libraries absent from this
LLVM toolchain; the matching gnullvm target builds successfully.

With Rust/cargo activated, reproduce the Linux cross-build with:

```sh
rustup target add x86_64-pc-windows-gnullvm
export LLVM_MINGW_ROOT=/path/to/llvm-mingw-20261020-ucrt-ubuntu-22.04-x86_64
scripts/build-windows.sh
```

The package's runtime DLL is required with this cross-build; it is not assumed
to exist on the user's PC. The app opens a message box on startup errors and
locates `Shadowrun.bin` beside itself if no command-line ROM path is given.

## Next acceptance milestones toward the complete game

1. Translate door/area transitions and their spawn/state changes; verify a
   continuous trip between two rooms instead of choosing maps manually.
2. Restore NPC/companion update dispatch, interactions and dialogue scripts.
3. Connect inventory/equipment actions, combat, damage, skills and magic to
   persistent party/world state and the existing HUD/animation components.
4. Restore character creation, menus, missions and world progression.
5. Implement audio sequencing/synthesis and the original save/load format.
6. Run end-to-end gameplay and packaging tests on Windows, then compare scene,
   timing and interaction traces with the original game.

This milestone supplies a Windows exploration executable. It is not the complete
working game requested as the final goal; the list above remains required.
