# Exterior area transitions

The native Windows/game core now traverses the ROM's exterior seams linking map
0 with maps 3 and 5. These are source-derived area links, not newly authored doors.
The subsequent [Space Needle milestone](interior-transitions.md) adds one
bounded interior path. Other interior/story scripts remain unfinished.

## Recovered behavior

`b7fa..b830` requires at least two negative footprint-corner samples before
transition handling. The previous maximum-code-only gate was insufficient.
`Collision::trigger_corners` adds that check; hard-block precedence still comes
from the existing collision query. Only the controlled-player entity flag can
start this transition, matching `b7ce`.

The main-loop call at `99c` reaches `3168` before general location scripts. For
trigger low bits `10`, that routine chooses:

| Source | Condition | Destination | Spawn record |
| --- | --- | --- | --- |
| 0 | fixed-point camera X < 1400 | 3 | 3222 |
| 0 | fixed-point camera X >= 1400 | 5 | 3212 |
| 3 | same seam trigger | 0 | 3202 |
| 5 | same seam trigger | 0 | 31f2 |

All numeric addresses/thresholds above are hexadecimal. The native implementation
supports these observed exterior maps; it does not extend the routine's generic
nonzero fallback to other maps without tracing their use.

Spawn application at `2b78` consumes two camera words and three party coordinate
pairs. It masks the camera words with `ff80`. `TransitionPlan` reads these records
from the supplied ROM, rather than baking camera/coordinate values into the game.
Party health, identity, palette, equipment and inventory bytes survive the reload.
Map grids/graphics, collision, environmental animation registration and HUD are
rebuilt for the destination. Fallible asset preparation completes before the
active scene is replaced.

## Native phase integration

The transition is handled by `Game::update`, so the desktop and headless checks
run the same state machine. Input cannot move actors or switch the party selection
during the transition.

- Fade out: 17 palette steps, one step after each two ticks (`131c/1342`).
- Reload at black: apply the ROM route/spawn and prepare destination assets.
- Warm up: 31 actor/environment update ticks, corresponding to loop `8c0`.
- Fade in: 17 ascending palette steps, one after each two ticks (`12f6/1342`).

This preserves the recovered palette loops and warm-up count. It does not claim
the original full dispatcher/AI or instruction-exact timing runs during warm-up;
the native subset advances animation and freezes world positions. Sound and all
other scripted transition side effects remain unimplemented.

Some original arrival cameras put the map's right edge underneath the 48-pixel
HUD window. Frame validation now checks the visible 272-pixel map width, and
plane B can fetch the following linear grid row beneath the window, as the
original tile resolver does. Static background-export bounds stay unchanged.
The host exploration camera's right clamp now reserves the HUD width as well.

## Evidence and reproducible tests

`tools/validate_transitions.py` executes original `3168` and `2b78` using Unicorn's
M68000 model. All 54 native route/eligibility cases and 36 camera/party spawn
applications match, including both sides of the camera threshold and unsupported
trigger codes. This validates routing and spawn effects, not the full game.

Two ROM-backed integration tests are ignored by default because the copyrighted
input is not in Git. Run them explicitly:

```sh
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test transitions -- --ignored
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_transitions.py /path/to/Shadowrun.bin
```

They seed the player just outside an actual seam rectangle and then drive normal
input through 0 → 3 → 0 and 0 → 5 → 0 without changing map IDs between legs.
Both routes finish their fades, load/render destination assets, match arrival
camera/party coordinates, and preserve changed health/equipment-state bytes.
The second test enters an actual C0 footprint and verifies that its unresolved
interior script leaves the scene unchanged. These are seeded boundary tests,
not proof of a complete trip from the game's original new-game start.

The ordinary suite now has 40 passing unit tests. Its two ignored ROM tests were
also run separately and both pass. The Windows package cross-builds; an actual
Windows GUI/playtest is still outstanding.

## Subsequent investigation

Ordinary doors proceed through `1651e → 16608` and the main-loop dispatch to
`167ae`. That path saves the exterior map via `16e30`, resolves target location
from descriptor offset `4e`, selects location mode and scripts, then reloads.
Some room types use `ffe114` spawn records; other types select procedural room
state. The exit path uses the saved exterior descriptor and return formation
(`bc36/bc66`, `2f32`). The [Space Needle implementation](interior-transitions.md)
now supplies a bounded entrance and return; general location scripts remain next.
