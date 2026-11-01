# Noncombat companion following

Healthy, non-selected party members now follow the selected runner in the native
game core, including hired Trent. Tab changes the leader; the previous leader
then follows. This is a bounded noncombat dispatcher, not full NPC/combat AI.
The corrected [Ghidra export](ghidra-reference.md) was used as a reference, with
instruction/data checks and original-routine execution validating the behavior.

## Recovered behavior

Addresses below are hexadecimal. `ac84 → 138b8` decrements each companion's word
`b6`. When it becomes negative, one companion per frame may make a decision;
the served counter resets to 20. This staggers decisions on the original
21-tick cadence. Movement continues on intervening frames.

`14508` chooses formation side using entity slot `3e` and the controlled slot.
It offsets the controlled runner's facing by `-6000` or `+6000` and projects an
`e0` fixed-point radius through table routine `15dcc`. If that footprint is
blocked, it tries the opposite offset, then the leader's own position. Trigger
footprints can be goals, but cannot be crossed by an uncontrolled companion.

The stopping gate `144e8/1460e` uses the original approximate distance and a
strict threshold of `1e0` (30 pixels). It clears route flags and switches to idle.
Heading/range selection uses `15d3c/15d4e` and the ROM's quantized angle tables,
including its different half-rounding convention. No floating-point atan or
generic steering approximation replaces those tables.

The regular `14682` route builder and `149c0` waypoint consumer can take a one-cell
step along the main axis or route around a rectangular obstruction using a pair
of corner cells. Route flags, offsets and sign-change checks mirror the traced
entity fields. The `1470e` fallback now follows the original wall-hugging path when those
strips cannot reach around an obstruction. `14af4/14800` searches up to five
candidate directions; `14764/14a54` advances subsequent corners, including the
original clear-strip continuation. `14914` clears wall hugging when either of
the two orthogonal L paths to the goal becomes clear. It does not require both.
Reaching a wall waypoint updates the next waypoint while retaining the previous
motion heading/range until a later scheduled decision, matching the original.
The initial fallback can consume a waypoint twice through its tail-call and
caller; that ordering is preserved.

Navigation retains descriptor word `24` (`ffddea`) separately from playable
bounds: the original compares the raster column directly with that tile-width
word, inclusively. It is not interchangeable with playable width or the fixed
64-cell raster stride. Reads outside the modeled collision raster remain
withheld. Closed pockets follow the original route-clearing/direct-steering
behavior and cannot teleport through walls. Physical actor contacts now use the recovered separation response described in
[actor contacts](actor-contacts.md). General contact scripts, combat AI and
controlled separation onto area triggers remain incomplete, so this does not
establish robust all-room navigation.

`b574/b6e4/bade` supplies walking, gradual turning and static collision sliding.
The distance-selected velocity tables use steps of 8, 10, 15, 20 or 31 fixed-point
units; offscreen party members use the original 40-unit table. Diagonal attempts
try cardinal alternatives, and cardinal wall sliding uses the original footprint
corner choices. Trigger results are blocked for non-controlled actors, so a
companion cannot initiate travel merely by approaching an exit.

## Integration details

Projection now records original biased screen words `a/c` at the end of each
game tick. The next follower tick uses the prior projection's visibility flag,
as the original actor update does. Headless behavior does not depend on how often
`Game::render` is called. The host exploration camera remains separate from the
original look-ahead camera; visibility therefore reflects that host camera.

The selected runner's motion handler/state is recorded for a subsequent control
handoff. Slots before the controlled slot use its pre-movement pose; later slots
use its updated pose, preserving the original party update order for this subset.

During the existing 31-tick reload warm-up, decisions/timers continue but
`ac9e/ae5e` forces movement idle. This both holds arrival positions fixed and
lets close formation goals clear navigation state from the previous room.
Skipping these decisions left stale interior waypoints active after returning
outside. Fade/input blocking and the existing ROM spawn/camera behavior remain.

Dialogue freezes party movement and AI counters. Party identity, health,
equipment, contract and knowledge/wallet state persist through area changes.
Transition tests now compare persistent fields explicitly rather than treating
navigation targets, collision samples and timers as immutable save statistics.

No JSON migration was needed for this engine work. The existing dialogue/hiring
records continue using typed Serde/serde_json. Future quest additions should
remain small and backed by original behavior.

## Verification

```sh
cargo test --locked
cargo build --release --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test transitions --test followers -- --ignored
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_followers.py /path/to/Shadowrun.bin
cargo run --release --locked -- follower-data /path/to/Shadowrun.bin artifacts/followers
```

The routine oracle checks 1,024 heading/distance pairs, 768 formation goals,
762 supported complete decisions and 1,024 movement/slide records against actual
68000 execution on maps 0, 3, 5 and 10. Six decision cases are explicitly withheld
because the original reads outside the collision raster. Four closed-pocket cases now compare the complete decision result with the
original, including the failed wall search and route clearing; they are no
longer withheld fallback cases.

A further 900-tick, three-runner trace matches full actor records for scheduler,
warm-up and regular corner routing around a synthetic wall using the real ROM
tables. It isolates these routines, holding visibility fixed and excluding
dynamic actor collision avoidance, combat, rendering timing and animation ticks.
The standalone endpoints use code hooks to handle cached Unicorn block boundaries.

A further 7,200 scheduler/movement ticks traverse four rotated U-shaped pockets,
matching every actor byte after each tick against original execution. Another
192 standalone decisions cover initial navigation, regular waypoint completion
and active wall-route completion at varied headings/positions. The oracle checks
that the original initial search, corner search, L-path exit and clear-strip
continuation branches actually execute. A functional test verifies each rotated
pocket uses fallback, leaves it, and reaches formation while every footprint
remains clear.

There are 49 passing unit tests and 12 explicitly enabled ROM integration tests
including the new physical-contact checks; see [actor contacts](actor-contacts.md).
New checks cover a regular wall detour, refusal to cross an area trigger, and an
actual hire → exterior return → following → leader switch sequence. Followers
must move normally and catch up when the leader stops, preserving payment and
contract state. The original cadence/turning can produce lag during a sharp turn.

`follower-data` also captures a 540-tick, seeded two-runner replay of `Game::update`
with frames every 30 ticks and a coordinate trace. This uses template selection
for presentation; earned hiring is covered separately by the integration test.
Generated frames/data and all Ghidra databases stay ignored under `artifacts/`.
Earlier original-routine checks remain relevant and passing. The Windows build
cross-compiles; actual Windows GUI playtesting remains outstanding.

## Next bounded work

Recover controlled separation/travel side effects and combat target selection.
Keep the six unmodeled raster cases explicit. Then connect the job/mission loop,
including single-run contract completion, earning money and broader dialogue/
opinion state. Keep JSON additions small and source-backed.
