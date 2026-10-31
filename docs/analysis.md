# Initial ROM analysis

Input: user-supplied Shadowrun.bin, 2,097,152 bytes.
SHA-256: 92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e
Header: SEGA GENESIS; SHADOWRUN; GM MK-1352 -00; (C)SEGA 1994.JAN.
Stored and calculated Genesis checksums: 0x6baf.

## Observed code

- Initial stack pointer: 0x00fffff8; reset vector: 0x00000236.
- Level 4 vector: 0x228, a jump to 0x1080.
- Level 6 vector: 0x22e, a jump to 0x1070.
- Reset tests I/O at 0xa10008 and 0xa1000c. Nonzero skips the initial
  hardware setup via 0x2c2 to 0x330. This decision is translated in boot.rs.
- Initialization loads constants from 0x2c4. It tests the low nibble of the
  hardware version at 0xa10001 and conditionally writes 'SEGA' to 0xa14000.
  The signature decision is translated in boot.rs.
- Subsequent loops write video registers, coordinate with the secondary CPU,
  and clear memory. These remain assembly, not translated Rust.

The disassembly records addresses and bytes for review. No gameplay function
names, asset addresses, compression formats, or map layouts have been inferred
without evidence. ASCII scans contain accidental printable byte sequences as
well as game text; they are not a recovered dialogue database.

## Next reconstruction milestones

1. Trace code at 0x330 and interrupt handlers at 0x1070/0x1080; establish
   main-loop states and RAM ownership using static references and runtime traces.
2. Locate video transfer routines and their callers; identify actual graphics,
   palettes, tilemaps, and compression. Validate a reconstructed title scene
   against a reference capture.
3. Identify input polling, map loading, entity state, and script dispatch;
   translate one complete movement/interaction path into native Rust.
4. Add rendering and input around confirmed game data; extend to combat, audio,
   saves, and remaining content with reference behavior comparisons.

A native rebuild requires redesigning console hardware operations around native
rendering/audio services. Renaming the ROM or wrapping an emulator would not
satisfy this reconstruction goal. The current binary is an inspection tool.

## Second milestone: native video and input primitives

Implemented and tested:

- `boot::region_decision`: $330..$36a uses the hardware version's top two bits
  to index `J, 0, U, E` at $46c. It searches all 16 region-header bytes for an
  exact match. The zero table entry halts. The supplied header specifies U.
- `video::boot_font`: $39c..$3ca expands 59 glyphs from the 472 bytes at
  $4ee..$6c6. Each glyph is 8x8 monochrome, corresponding to ASCII $20..$5a.
- `video::region_screen`: reconstructs the mismatch message on a 320x224 RGB
  raster using the font, embedded string positions, and ROM region header.
  Previewing this screen does not mean a US console would reject this ROM.
  The result matched every pixel of an independent instruction-based reference
  for the supplied ROM. No reference-console screenshot has been compared.
  PPM SHA-256: af1eda6a6ed5273d2566bf5a49d1121f87a4081d475833f5a17e1190019429f2
- `video::fade_out` and `fade_in`: $1088 and $10d6 process 64 palette words
  at $ffd87e, with fade-in targets at $ffd8fe. Each RGB channel steps by one
  Genesis level. Fade-in increments differing channels rather than comparing
  magnitudes; out-of-contract inputs retain the assembly's wrap/mask behavior.
- `input::Controller`: $195c/$199a maintains held buttons, rising-edge presses,
  and the 16-entry direction table at $19d2. Contradictory directions yield no
  heading. Native inputs avoid dependence on console port timing.
- `input::decode_pad_samples`: $19f2/$1a48 combine active-low samples into
  Up, Down, Left, Right, B, C, A, Start bits, from least to most significant.

### Interrupt and entry observations (not yet translated)

Level 6 handler $1872 increments the longword at $fff9ec, clears the word at
$fff9f0, consumes pending bits 1, 0, 2, 3 at $ffef14, and conditionally calls
other routines. These observations suggest a frame count and queued video work,
but we have not traced all call sites or timed the behavior.

Level 4 handler $1910 increments $fff9f0 and alternates source palette words
between $ffd8bc and $ffd8dc according to its parity. After waiting on VDP status,
it writes two words to CRAM beginning at address $5e (control command $c05e0000).

The accepted-region branch at $6c6 reinitializes video and enters $75e.
This calls $c2c and $5d746 before reaching further game initialization at $796.
The marker $12345678 is written to $fff9e8 if absent. The meaning of routines
at $53126, $59318, $59092, and subsequent addresses remains unconfirmed.

Listings for these ranges are saved alongside the original startup listing.
Game-loop reconstruction and native scene rendering remain outstanding.


## Third milestone: main loop and map-loading trace

See [main-loop-and-maps.md](main-loop-and-maps.md) for the detailed trace and
addressed instruction listings. Main frame loop: `0x952`; descriptor loader:
`0x3084`; table at `0x3232` contains 56 pointers. Native map extraction now
preserves descriptors and both fixed-size grid copies and decodes tagged shared
tile definitions. All 56 entries and 112 grid copies were checked. All 15
compressed entries matched execution of the original 68000 decompression routine
byte for byte, including consumed source length. Frame waits are translated into
nonblocking polls with their overlapping RAM markers preserved.

This is static control-flow recovery plus routine-level differential execution;
no full-game dynamic trace, map scene rendering, or gameplay execution is claimed.


## Fourth milestone: static map-plane rendering

See [map-rendering.md](map-rendering.md). `src/map_render.rs` interprets the setup
script at descriptor offset 0x34, loads VRAM graphics, combines the map-specific
and shared palette banks, resolves both map layers, and composites tile flips,
transparency, and priority. The CLI can export arbitrary bounded world-pixel
crops as PPM. A 960×640 crop of map 0 is available in artifacts.

This renders static background planes. It does not execute the game or reproduce
sprites, HUD, animations, raster effects, or dynamic palette/map updates.


## Fifth milestone: character/HUD/animation rendering components

[Character, HUD and animation notes](characters-hud-animation.md) record 16 banks,
885 sequences, 987 distinct sprite frames, the wrapping animation clock, sprite
piece layout, and original HUD routines. Native components now compose selected
animated characters with map planes and the HUD window. Differential execution
checks cover all decoded sprite frames, one 96-tick timeline, and the sample HUD
state. A generated preview and its explicitly chosen sample inputs are retained
in `artifacts/actors-hud-animation`.

This completes a rendering-components milestone, not the game-driven entity/party
state integration. Portrait/item selection, AI/movement/combat, map-tile animation
scheduling, and a playable main loop remain unresolved.


## Sixth milestone: Windows exploration executable

[Stateful exploration and Windows details](exploration-and-windows.md) document
party initialization from real templates, resolved portrait/equipment data,
persistent movement/turning and actor depth ordering, static collision, native
map tile animations, and a keyboard-driven Windows desktop binary. Original
routine differential checks pass for all 56 collision maps, 7,168 query points,
16 animation ticks per map, and a real-template party HUD refresh. Native
integration replays render and advance all three selected runners.

A Windows x64 PE GUI executable is cross-compiled and packaged with its runtime
DLL. Windows GUI behavior remains untested here. The full-game milestone still
requires room transitions/scripts, combat, dialogue/AI, inventory and menus,
missions, save/load, audio, and Windows end-to-end playtesting.
