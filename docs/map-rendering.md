# Static map rendering

`src/map_render.rs` reconstructs a selected region of the two map planes in
native Rust. It uses no emulator at runtime and has no external Rust dependencies.
The accompanying PNG preview is a conversion of the Rust-produced PPM.

## Recovered asset loading

The descriptor longword at offset `34` points to the setup script consumed by
`1406`. Its initial layout is:

| Relative offset | Data |
| --- | --- |
| 00 | Pointer to plane addresses and 15 VDP register writes (`cec`) |
| 04 | Pointer to 64 target colors copied by `1368` |
| 08–0f | Four initial scroll words |
| 10 onward | Graphics transfers: source, VRAM destination, byte count (three longs) |
| After transfers | Zero source terminates the list; another zero ends the secondary list |

The recovered scripts all load raw graphics beginning at VRAM `4020`, then
shared graphics from ROM `e44d8` to VRAM `b000` (length `d00`). The native loader
also understands the tagged compression branch at `1452`; none of the 56
current scripts requires that branch. Nonempty secondary setup lists are rejected
explicitly because their interpretation is not implemented here.

**A later palette step matters:** `2c72` calls `566c2`, which copies 96 bytes
from ROM `18ac56` to RAM `ffd91e`. That replaces target palette banks 1, 2, 3,
leaving the first bank map-specific. The renderer applies this overwrite. Using
only the palette pointer from the initial setup script gives incorrect colors.
We render the completed target palette, not an intermediate fade frame. Channels
are expanded linearly from the three-bit Genesis range to eight-bit RGB; this
is not an analog display or CRT color simulation.

The initial VRAM reconstruction is zero-filled before scripted transfers.
Reserved areas normally supplied by sprites, HUD, or later dynamic transfers
are not reconstructed. Their omission must not be interpreted as recovered
blank graphics for every gameplay state.

## Plane rendering

The configured VDP registers and the first plane addresses at the control
structure identify layer 0 as plane B (`e000`) and layer 1 as plane A (`c000`).
For each world pixel:

1. Resolve its 32×32 block through the layer grid, using descriptor row stride.
2. Resolve its 8×8 tile-attribute word within that block's 4×4 definition.
3. Apply the layer-1 priority XOR at `3dae`, matching `ffdf58 = 8000`.
4. Decode tile index bits 0–10, horizontal/vertical flips 11/12, palette bits
   13/14, and priority bit 15.
5. Read the packed 4-bit pixel from the reconstructed VRAM tile.
6. Composite B low, A low, B high, A high; color index zero is transparent.
   If both planes are transparent, use the configured register-7 backdrop.

Coordinates supplied to the CLI are world pixels, not the 1/16-pixel stored
camera values. The command draws a crop, not a complete camera/frame simulation.
It rejects horizontal crops outside the grid row stride, accesses beyond copied
grid buffers, and invalid definition references. The fixed-copy buffer length
still does not establish logical map height; select crops inside known map areas.
A default 320-pixel crop starting at x=7 is too wide for the 320-pixel-wide maps;
start those crops at x=0.

## Reproduce

With the Rust environment from README activated:

```sh
cargo build --release --locked
mkdir -p artifacts
cargo run --locked -- render-map /path/to/Shadowrun.bin 0 0 0 artifacts/map.ppm 960 640
```

Without width/height the image is 320×224. Output paths must be new, to preserve
existing files. This workspace's preview is `artifacts/map-00-background.png`.

## Validation and boundaries

Unit tests cover tile flips, packed nibbles, palette-bank selection, transparent
pixels, priority order, and RGB expansion. `tools/validate_render.py` renders all
56 maps and compares 4,014,080 pixels with an independent Python implementation.
For compressed tile definitions that implementation executes the original ROM
routine using Unicorn's M68000 model. Narrow maps use origin (0,0); the others
use (7,11) to exercise off-tile crop handling. This checks data interpretation
and implementation consistency, not visual equivalence to full gameplay.

No running-console screenshot comparison has been performed. The output excludes
sprites, characters, HUD/window plane, animated tile updates, dynamic map edits,
scene-specific palette changes after setup, fades, raster palette changes, and
camera scrolling effects. Rebuilding those requires the entity and frame-update
routines. Static background composition is now available for that next step.
