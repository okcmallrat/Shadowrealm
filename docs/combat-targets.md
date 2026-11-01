# Combat target acquisition

## Weapon timing and source setup

`src/weapons.rs` ports `$abfc/$d7ea/$d81c`: stats `$78+$7b` add as a
wrapping byte before halving; the first set enhancement bit 12..14 supplies
the bonus. The timer uses signed word arithmetic, mode-specific doubling or
halving, then a minimum of 15. `$a8fc..$a912` decrements only active actors
with nonnegative `$dc`; zero becomes `$ffff`, and negative values hold.
The game advances timers during normal simulation and travel warmup.
Dialogue and fades retain the existing host simulation freeze.

`prepare_ranged` reproduces source-only `$b39a`: facing, rounded octant state
from `$c042`, handler `$b3ee`, equipment-dependent cooldown, animation phase
and frame reset, and refresh bit. The native animation clock resets alongside
the original fields. Missing ROM data fails before mutation. Desktop input
does not call this backend yet: admission, ammunition, projectile allocation,
damage and attack AI remain unfinished.

`tools/validate_combat.py` compares complete actor records in 4,096 cases each
for cooldown, dispatcher prefix and ranged setup, executing original instructions
without stubs. The dispatcher stops at its active continuation or inactive
epilogue. Cases cover enhancement priority, byte overflow, timer sign boundaries,
five modes, seventeen equipment IDs and distributed angles. A ROM integration
test verifies game ticks, inactive actors, negative timers and dialogue freeze.

## Targeting and obstruction

The next combat prerequisite is implemented: original target-list filters,
target retention/replacement and the close-range probe. Press **T** outside a
conversation to toggle target selection in the native window. The selected NPC
gets brackets and its name appears in the title. A blocked probe makes the
brackets red and adds "path blocked"; unknown geometry uses gray. Yellow is
selection feedback, not permission to fire. Press T again to clear;
Tab transfers the selection mode to the newly controlled runner.

Target selection does not fire a weapon, consume ammunition, classify an NPC as
hostile, or apply damage. The original candidate list can include talkable NPCs
such as Trent. There are no fabricated enemies or training-room spawns in this
build. General hostile spawning/AI, attacks and damage remain unfinished.

## Recovered rules

`13874` rebuilds the NPC list at frame phase 11 modulo 64, and the party list at
phase 15. A forced rebuild updates both. `1438c` visits slots 3..9 and `14446`
visits slots 0..2 in native slot order. Both require active, non-dead,
non-incapacitated flags and a nonzero projected visibility word `c`. NPCs also
exclude byte `b0` bit 7 and byte `9e` bits 3/5. There is no additional test for
talkable bit 4 or health byte `4c` in these routines. Eligibility for this list
is not permission to attack.

`143f6/14492` preserves a target whose pointer is still in the cached list. If
invalid, `d3a/d44` chooses a replacement using the high word of the next random
seed times the candidate count. Selection and physical contacts share the same
seed. Merely retaining a valid target consumes no randomness. The NPC-side
selector records a changed target with byte `b5 = 6`. Empty lists clear `a2`;
the original empty-NPC-list end-combat callback is reported by the core but its
combat/mission side effects are not implemented.

The `15c58` close-range probe checks active/non-dead target flags and a strict
`200` fixed-point distance threshold (32 pixels). It uses the original quantized
heading and floor-half distance estimate. This is not a line-of-sight, weapon,
ammunition, attack-permission or hit-chance test. The subsequent `14d8e`
obstruction probe is now implemented below; weapon/action/damage routines must
still be recovered before firing is enabled.

The user's Ghidra export was opened as a private working copy, rehashed against
the ROM and used alongside the instructions. Targeted references are retained
under ignored `artifacts/combat-reference/target-ghidra`. No game databases,
extracted data or ROM bytes are added to Git.

## Native integration and limits

T is a native UI binding for this checkpoint, internally mapped to a pressed
controller C bit outside dialogue. It does not claim to reproduce the original
combat/menu controls or activate the full original combat dispatcher. The target
brackets are a host UI overlay, clipped to the map viewport, not a recovered VDP
sprite. Selection uses the existing host camera/projection and simulation tick;
original camera/hardware timing remain separate work.

The mode is off by default. Enabling it, changing the controlled runner and
loading another map force a fresh list and remove old controlled-runner links.
Dialogue and travel freezes remain. The core retains cached-list timing between
refreshes; safe native pointer resolution hides a removed/dead actor instead of
dereferencing a missing entity. An empty list leaves the UI mode on with no target;
it does not pretend to execute the missing original combat-end script.

`eb92/eccc` uses a wider 16-pixel contact window for an existing `a2/ac` link in
either direction. That window is now connected to the existing Trent dialogue
path, so selecting him still allows his real conversation to open. Unlinked
contact remains 4 pixels. Conversation/hiring continues to use the typed
Serde/JSON pilot; no quest rules or broad JSON migration were added here.

## Validation

```sh
cargo test --locked
cargo build --release --locked
SHADOWREALM_TEST_ROM=/workspace/inputs/Shadowrun.bin cargo test --locked --test combat --test avoidance --test followers --test transitions -- --ignored
/workspace/.venvs/shadowrealm/bin/python tools/validate_combat.py /workspace/inputs/Shadowrun.bin
/workspace/.venvs/shadowrealm/bin/python tools/validate_dialogue.py /workspace/inputs/Shadowrun.bin
```

The combat oracle executes the original routines and compares 768 target-list/
acquisition cases, including 441 random replacements, 116 retained targets and
147 empty-list callback requests; 1,024 close-range probes; and 256 refresh/
retention phases with complete actor records and random state. The end-combat
callback is counted and omitted; no missing shutdown behavior is substituted.

The dialogue oracle now checks 469 unlinked/linked contact cases, plus the existing
15 JSON node/text records and 26 text/flag branches. Functional tests verify the
toggle, marker, target retention, leader switching and widened Trent conversation
without applying damage or consuming ammunition. That targeting checkpoint passed 53 unit tests
and 16 explicitly executed ROM integration tests. The avoidance/travel, follower,
world, exterior, interior and hiring oracles remain passing.

The Linux desktop and Windows executable cross-build; the Windows package is
refreshed and inspected. Actual Windows window/input playtesting remains
outstanding. This is a targeting milestone, not working combat.

## Original projectile obstruction probe

`combat::line_of_fire` now translates `14d8e`, the probe-only path through
`14ff8/15118`, `14f26`, the point-query branch of `e7c8`, and intended-target
matching in `15aba`. It uses the original quantized heading, rounded muzzle
octant, ROM launch offsets and signed velocity tables. The 32-bit position
accumulator and fractional carries match the original; it is not a generic
floating-point segment/ray test. The original Ghidra references are preserved
privately in `artifacts/weapon-reference/ghidra`.

The query samples at most 27 steps. It distinguishes reaching the target,
blocking geometry, exhausting the trace and unmodeled geometry. Trace exhaustion
does not prove that the target was reached. The original carry result reports
obstruction, not a complete attack-permission decision. Actor matching checks
active/non-dead/non-incapacitated flags, nonzero health and the asymmetric
lower-inclusive/upper-exclusive `90` fixed-point window. Probe mode ignores
intervening actors and only matches the requested target pointer; real weapon
collision/damage behavior must be recovered separately.

The probe particle uses single-cell sampling rather than the party's four-corner
query. Positive bit-6 collision codes set the original projectile-passable flag,
with `7f` handled first as a blocker. Trigger codes block this query and do not
initiate travel. Rectangle and diagonal cases preserve the original conditions.
Other corner bytes remain in the original shared scratch and can affect diagonal
queries; the API therefore requires their initial values explicitly. Missing
raster/rectangle data returns `OutsideModel`, rather than pretending to be clear.

The native marker uses an independent query with cleared scratch. That is an
explicit deterministic UI choice, not a reconstruction of every original
caller's global collision history. The backend accepts retained scratch and is
compared with those states too. Rendering the indicator does not move actors,
advance simulation, consume randomness/ammunition, create a live projectile,
write incoming target links, or apply damage. No firing action is enabled.

The expanded combat oracle runs 2,176 full original probe calls without replacing
any subroutine on that query path. Six real maps (0/3/5/10/31/55) and eleven
synthetic collision bands cover hard/transparent obstacles, triggers, rectangles,
diagonal forms, fractional positions, inactive/dead/incapacitated/zero-health
targets, and three initial scratch configurations across all three party source slots.
It compares every byte of
the 256-byte projectile record, step count, scratch output and outcome, and checks
source/target/intervening actors remain unchanged. Results: 271 target reaches,
1,169 blocked traces and 736 trace-limit outcomes. The guard for missing geometry
has a separate unit check.

The new functional test uses a synthetic wall in the native game core: the
marker is red when blocked and returns to yellow after removing that collision
band, while actor records, wallet and simulation tick remain unchanged. There
are now 54 passing unit tests and 17 explicitly enabled ROM integration tests;
earlier target, contact/travel, follower, world, transition, interior, dialogue
and hiring comparisons remain passing. Linux/Windows builds and the packaged
Windows executable are refreshed. Windows GUI playtesting remains outstanding.
