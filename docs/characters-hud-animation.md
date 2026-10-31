# Character, HUD and animation reconstruction

Native rendering components are in `src/actors.rs`, `src/hud.rs`, and
`src/scene.rs`. They render supplied actor/party state. They do not execute AI,
combat, save-game loading, or the game's complete main loop.

## Recovered character formats

Sixteen bank pointers at ROM `62ac` select animation tables. State counts, from
confirmed table boundaries, are 16, 6, 5 for banks 0, 1, 2 and 66 for banks
3..15: **885 sequences**. Their actor names are not yet established. Each state
selects a pointer at `table + state*4`; this corresponds to entity offset `16`
containing a byte offset, masked with `7fff`, in the original selector.

Each sequence begins with:

| Field | Meaning |
| --- | --- |
| word 0, low 15 bits | Frame count |
| word 0, high bit | Terminal/clamped sequence instead of looping |
| word 2 | Clock increment for terminal sequences |
| longwords from 4 | Frame-record addresses |

The selector at `600e` clamps terminal sequences to the last frame and sets
entity state bit `8000`; looping sequences wrap to frame zero. For looping
sequences, speed comes from entity `ca`: multiply by 28 below 22, otherwise by
16. Bank 13 is special: increment `180`, except frame 2 uses `90`. The native
`Sequence::increment` preserves that behavior.

The clock at `5fdc` adds entity word `18` to phase `1a`, compares as signed
words against period `1c`, and advances at most once before subtracting the
period. The native clock retains wrapping-word behavior rather than dividing
by the period. The default entity template at `7361a` has period `1000` at
relative offset `1c`. Other initialized entity fields and speeds depend on
live state. Native `ended` exposes the terminal event for a future state machine;
it does not implement every gameplay response to the original state bit.

### Sprite frames

The builder `622c` consumes two frame forms:

- Plain list of pieces terminated by word `8000`.
- Prefix `8002`, graphics source long, graphics length word, then pieces.
  The original queues this graphics transfer and offsets relative tile indices
  by entity word `20`. The native renderer applies the transfer before rendering.

A piece contains signed x offset, signed y offset, size word, and tile attribute
word. Size uses two bits each for tile width/height minus one. Sprite tile data
is **column-major**, unlike background block definitions. Horizontal/vertical
flip flags reverse the whole piece, including its tile ordering. Entity palette
flags are ORed into tile attributes before the optional base-tile addition.

The native sprite renderer chooses the earliest nontransparent sprite at an
overlap. Scene compositing follows B-low, A/window-low, sprite-low, B-high,
A/window-high, sprite-high. A high-priority later sprite cannot bypass an earlier
sprite solely because its priority bit is higher.

`actors::project` translates `5a56`: wrapping subtraction of stored 1/16-pixel
world/camera coordinates, arithmetic shift, carry-based rounding, height offset,
and the two recovered culling modes. Results omit the hardware's 128 bias.
Scene placement takes screen-pixel anchors; a future entity update loop can
supply projected anchors through this function.

## HUD window and party state

VDP register `833c` selects the window tilemap at `f000`. The map registers
`9191/929f` expose its rightmost 48 pixels at screen x=272..319. This window
**replaces plane A** in that region; it is not simply a bitmap pasted over both
background layers. The native compositor preserves its palette, transparency,
and priority bits.

Recovered routines and native operations:

| ROM routine | Native support |
| --- | --- |
| 334c | Fixed HUD graphics, borders and three party-slot templates |
| 3404/3548 | Portrait tile transfers and 4x4 portrait layout |
| 34c8/35f0 | Six-tile equipment icon and 3x2 layout |
| 3674 | Health-meter table, explicit level 0..10 |
| 36fc | Two-cell item-resource table, explicit bounded table index |

`1232` saves and restores A1. Consequently repeated layout calls use the same
source template rather than advancing to successive templates. The native HUD
was checked against execution of the original routine, including this behavior.

`Hud::portrait` accepts the resolved portrait-definition address (plus any crop
offset selected by gameplay). `Hud::equipment` accepts the six-word icon source
and palette flags. These APIs implement the rendering operations; automatic
actor-to-portrait, item-to-icon, damage-to-meter, and item-resource resolution
still require the party/equipment logic. They are not inferred from animation
bank numbers. Palette values and VRAM allocations are explicit inputs.

## Runnable animation preview

```sh
cargo build --release --locked
mkdir -p artifacts
cargo run --release --locked -- render-animation /path/to/Shadowrun.bin 0 5 8 artifacts/animation 96
python3 tools/make_preview.py artifacts/animation
```

Arguments: ROM, map ID, bank, state, **new** output directory, optional tick count
(default 96, maximum 600). Raw output is native PPM frames and a CSV containing
sequence frame index, ROM frame address, and clock phase. `make_preview.py`
requires Pillow and converts those frames to PNG/GIF for viewing. GIF presentation
at approximately 60 Hz is a preview choice, not measured game timing.

The preview uses an explicitly chosen actor at screen (160,184), camera (0,0),
VRAM base tile `100`, palette 1, sample speed 8, and clock period `1000`.
HUD health levels are sample inputs [10,7,4]. Portraits use the blank definition;
equipment icons use the ROM's unarmed definition at `d1860`, with zero item
resource. It does not claim those values describe a real party or a real save.
The 96-tick example renders four distinct character-frame images.

`scene::render` also accepts multiple placed actors and an optional HUD. The
caller must allocate nonoverlapping VRAM ranges and supply ordering; live entity
sorting and dynamic allocation are not reconstructed here. Sprite scanline
limits/masking, raster effects, shadow/highlight mode, animated map tiles, and
full game-driven camera/party state are not emulated.

## Verification

`tools/validate_actors.py` (Unicorn 2.1.4) performs reproducible routine-level
checks against the original uploaded ROM:

- Native catalog: all 885 sequence headers and frame pointer lists match ROM.
- All **987 distinct sprite frames**: native geometry and attribute results match
  execution of `622c`, including dynamic-frame base addition and palette flags.
- A 96-tick native preview timeline matches original `5fdc` and `600e` execution
  for the same explicitly supplied entity state.
- HUD graphics, window words, health meters, blank portraits, unarmed equipment
  icons, and empty item meters match execution of `334c`, `3404`, `3674`, `36fc`
  for the same fixture inputs.

Unit tests cover sprite column ordering, flips, transparency/overlap order,
clock wrap/clamp behavior, projection and clipping, in addition to existing map
and input tests. This is not validation of all animation timing states, arbitrary
portrait/equipment inputs, or a comparison with a running-console screenshot.

```sh
cargo test --locked
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_actors.py /path/to/Shadowrun.bin
```

The rendering/asset milestone is functional; full gameplay-driven characters,
party portraits/equipment selection, and animated map scheduling remain unfinished.
