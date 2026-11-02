# Player gunfire and combat reconstruction

## Firing enabled

Hold **F** outside dialogue to fire the selected runner's equipped firearm.
Tap **T** to select a visible target, or fire in the current facing direction.
Tab switches runners; Space pauses and R resets. Enter/Z confirms dialogue and
does not fire outside it. Firearms 1..12 are supported; melee, magic, throwables,
ongoing effects and hostile/companion attack AI remain unfinished. No enemies
or practice-room spawns are added for this feature. The original protected/
neutral actor filters still apply.

`src/combat_runtime.rs` connects recovered launch, flight, damage and reaction
setup to `Game::update`. Selected-player requests use `$ad1a..$ad82` cooldown
queuing, held-A fire and `$aad2/$14226` finite-reserve reloads after failure.
Actor firing/hurt poses hold movement until their terminal sequence ends; death
poses remain visible and become incapacitated. The six projectile/FX records
advance in slot order after actor motion, resolve pending hits immediately and
use `$5fdc/$600e` frame timing, terminal flags and clamping. Rendering includes
their original resident sprites; all supported map scripts supply the shared
`$e44d8` graphics at VRAM `$b000`. The flying projectile's blank sequence is
preserved; muzzle flashes and impact graphics provide the original feedback.
Only simulation spends ammo/RNG or changes health; rendering remains read-only
with respect to actor state.

Host integration choices: F maps to Genesis A, T retains target selection,
dialogue/fades freeze the simulation, new maps clear transient projectile slots,
and switching runners clears a queued tap/status. The explicit development
roster's provisional template ammo is clamped to its ROM magazine capacities at
startup, matching the existing HUD instead of discarding excess on the first
shot. This does not reconstruct a new-game or earned inventory state. Missing
party/NPC slots are represented as inactive records for original pool scans and
reward writes. Shooting near a talkable NPC suppresses auto-contact dialogue
while the fire button/projectiles are active, so a shot is not frozen by a modal
conversation. Unsupported weapon/effect inputs report an unavailable status.

Additional original-routine checks: 1,024 controller gates, 1,024 reloads,
1,024 complete failed-launch/reload continuations and 1,024 animation-clock calls
match; only reload audio/HUD transfers are omitted. The controlled continuation
overwrites a reload timer with mode 0 and resumes direction input; a failed
reload creates no new timer and holds the idle handler for that request.
Game-level tests cover visible muzzle pixels, ammo/HUD updates, targeted damage,
reaction pose preservation, held burst fire, queued taps, reserve exhaustion and
pool retirement. All 59 unit and 27 explicitly enabled ROM integration tests pass.
The 313-tick `scenarios/firing.csv` replay exercises single/held shots and a runner
switch through the desktop core. Linux desktop and Windows cross-builds pass;
the versioned Windows ZIP is refreshed. Actual Windows GUI playtesting remains
outstanding. Audio, mission-aware death callbacks, full defeat/game-over flow,
general NPC spawning and broader combat systems are still incomplete.
The broader idle aim-charge/skill branch is not reconstructed; existing native
walking remains the host integration when no fire request is serviced.

The sections below document the recovered components and their earlier isolated
checkpoints; the runtime above now connects the supported baseline paths.

## Ordinary NPC firearm combat

`src/npc_combat.rs` ports `$13904..$13920` decision timing and the
existing-target `$14020..$1413c` firearm gates. The timer wraps as a word;
a due decision resets to 20 only when the companion-service flag permits it.
Otherwise the timer remains negative and can be retried next frame.

The fire check preserves original branch ordering: unarmed, cooldown, visibility,
negative equipment/magic and throwable paths are distinguished before the
projectile probe. A blocked probe clears `$dfde`; a clear probe can still hold
fire in mode zero when source `$b0` bit 4 is clear. A ready branch sets `$dfde`
to one and stops at launch entry. No ammunition or RNG is consumed by this
check. Missing geometry or inconsistent actor references fail without changing
the explicit firing flag or four-byte collision scratch. A trace-limit result
retains the original clear-return semantics; it is not treated as a proven hit.

1,024 original timer prefixes and 3,264 original fire checks match without
instruction stubs, including complete actor records, probe projectile/step count,
scratch output, firing flag and unchanged RNG. Outcomes are 205 ready, 1,428
cooldown continuations, 493 hidden, 357 magic continuations, 204 throwables,
123 blocked, 46 hold-fire and 408 melee continuations, across the same six real
maps and eleven synthetic geometries used by projectile validation. Generated
Ghidra views are private under `artifacts/npc-fire-reference`.

Automatic ordinary NPC firearm dispatch is now connected to Game. The recovered
outer branch order excludes protected, ignored and special/script continuations.
Eligible engaged actors acquire party targets through the original target lists,
use `$13d56` range/strafe goals and common steering, then move through the original
engaged NPC collision tables. Ready shots share the existing launch, animation,
projectile and damage systems. Cooldown/blocked continuations preserve probe and
formation behavior. Empty weapons clear equipment before the deferred melee path.

The original `$1de00` timeout prefix and `$1dbf0` combat-link cleanup are connected;
mode, script and map guards remain explicit. Subsequent auto-target/UI continuation
is not reconstructed. 512 cleanup comparisons, 1,024 outer branches, 1,024 range
goals and 1,088 engaged movement cases match original execution without stubs.
256 complete ordinary decisions match actors, slots, world state and shared RNG,
omitting only audio/HUD transfers. Unicorn instruction patches now invalidate its
translation cache so supplied-roll and original-roll checks cannot reuse stale code.

Game-loop tests demonstrate hostile pursuit/fire/player damage and a calm NPC
returning fire after a player shot. Fixtures create test actors only: no enemy or
training spawns are added to the product. Ambient patrol, special/script actors,
companion combat, melee, magic and throwable continuations remain incomplete.
The Windows package includes this bounded automatic firearm behavior.

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
now calls this setup on successful firearm launches. General attack AI remains unfinished.

`tools/validate_combat.py` compares complete actor records in 4,096 cases each
for cooldown, dispatcher prefix and ranged setup, executing original instructions
without stubs. The dispatcher stops at its active continuation or inactive
epilogue. Cases cover enhancement priority, byte overflow, timer sign boundaries,
five modes, seventeen equipment IDs and distributed angles. A ROM integration
test verifies game ticks, inactive actors, negative timers and dialogue freeze.

## Launch admission and ammunition

`consume_launch_resource` ports `$38ac` and the actor-data changes in `$36fc`.
A zero `$4e` or item ID rejects the request. NPC slots (signed slot > 2)
subtract one byte from `$4e`. Party items 8..10 subtract three; other items
subtract one. Ordinary weapons use `$4e` with capacity from the ROM item table.
The original ADD/BGT and CMP/BPL conditions are preserved, including overflow.
Items 13..15 instead update the equipped inventory quantity; exhaustion clears
that inventory entry and equipment ID and sets the equipment slot to `$7f`.
Even these items first require nonzero `$4e`; that gate is preserved.

The original can report admission without modifying data for inactive party
actors, absent/negative equipment slots, or item IDs 16..127. This routine
does not independently enforce controller permission or weapon validity.
Malformed high item IDs and consumable slots outside the eight inventory
entries fail explicitly when their unmodeled reads would be needed. Missing
capacity data fails before changing the actor. HUD output remains the caller's
responsibility; the runtime refreshes party HUD data after resource changes.

`admit_ranged_prefix` reproduces `$1514e..$151be`, stopping before projectile
initialization. It sets the burst signal, rejects mode 3 or animation states
above `$3c` after the `$7ff` mask, sets the sticky controlled-busy signal on
the applicable state rejection, consumes resources and searches six projectile
entries in order (`$15a8c`). The search neither clears nor reserves an entry.
A full pool can consume ammunition. The later second FX allocation may also
fail; `launch_with_resolved_roll` implements that continuation below. Cooldown and
controller permission checks occur outside this recovered prefix.

Validation compares full actor records for **18,432 resource cases** and
**6,144 launch prefixes**, plus prefix signals, result/selected slot, unchanged
pool bytes and neighboring party records. Original outcomes were 2,379 ready,
1,020 mode-blocked, 2,195 state-blocked, 508 empty-resource and 42 pool-full.
Resource quantities cover all byte values, signed actor-slot boundaries,
equipment absence and all modeled inventory entries. Only low-level HUD/VDP
transfers `$f5c/$1232/$3648` are omitted; item removal still executes original
`$3404/$3548`. Timing/projectile checks restore those instructions afterward.
An integration sequence verifies ammo loss on pool exhaustion, a subsequent
empty-resource rejection, consumable removal and early gate/sticky-flag behavior.
The source reference was exported from the verified Ghidra working copy to
ignored `artifacts/weapon-admission-reference` and checked against disassembly.

## Projectile and muzzle-FX initialization

`initialize_projectile` implements live `$14ff8` initialization after the
separate `$55640` roll. It clears all 256 bytes, carries the original source
pointer, equipment and resolved success byte, aims at an active target (or
uses source facing), records that target's attacker, applies the octant muzzle
offset and signed velocity tables, and computes the muzzle-FX origin. Source
coordinate fractional words are not copied. No health changes occur here.

`launch_with_resolved_roll` connects admission to that initialization, `$1dbaa`
alert state, burst setup `$152fa`, and the second projectile-pool lookup. If
that second lookup fails, the original returns failure with an initialized live
projectile still in the pool; ammo, alert and supplied-roll effects also persist.
On success it copies the first 160 bytes into the FX slot, preserving that
slot's tail, and applies the original FX handler/state/position and ammo flag.
The selected sound ID is returned; audio playback is not implemented. Ammo-flag
address arithmetic wraps as a byte, including negative/absent equipment slots.
The alternate-ammo path reads the last RAM byte; that shared byte is explicit
in `LaunchWorld`, rather than replaced with an assumed constant.

This lower-level API requires an **explicit externally resolved roll**. The
`launch_ranged` wrapper below now computes the bounded original `$55640` roll.
Neither API applies damage or authorizes firing. Environment mode 2's additional
alarm/story script effects are unsupported and return an error. Invalid target
references and missing ROM data also return errors without modifying state;
this host safeguard is distinct from original gameplay allocation failures.
The desktop now uses this path through `launch_ranged` for supported firearms.

The oracle executes original `$1514e` with only `$55640` supplied with the same
explicit success byte, audio omitted and low-level HUD transfers omitted.
Both allocation searches, projectile/FX initialization, heading and alert logic
execute original instructions. Across 2,048 cases it compares every byte of all
ten actors and six projectile slots, return carry, shared flags, origins,
sound choice and alert timers. Cases include party/NPC sources, absent/inactive/
active targets, stale pool bytes, fractional coordinates, negative equipment
slots, consumable exhaustion and all pool occupancy masks. The validator models
the sign-extended `$ffff.w` alias of the Genesis bus's last RAM byte explicitly.
Generated Ghidra views are in ignored `artifacts/projectile-launch-reference`.

An integration sequence checks a surviving projectile after FX exhaustion,
subsequent successful projectile/FX allocation and retained FX tail bytes.
Host-error tests check that missing tables and unsupported script modes leave
state unchanged. Linux desktop and Windows cross-builds pass, and the current
versioned ZIP now includes the integrated runtime described above. Windows GUI
playtesting remains outstanding.

## Original ranged attack rolls

`src/attack_roll.rs` ports `$dba` and the ranged-launch `$55640` path when
source `$df` bits 0/1 are clear. The dice use the shared `$fff9f8` RNG: sixes
explode, the roll total wraps as a word, successes wrap as a byte, and even
a zero pool rolls once. Original sign-only branch behavior is preserved at
word/byte overflow boundaries.

The pool follows `$55ed6/$55f20/$12c74`: byte-wrapped physical stats, the first
inventory `$28` bonus, the original injury percentage selection, then general
versus weapon-category skill and the skill-limited bonus. Difficulty includes
`$b4`, the applicable `$90` flag, item-indexed equipment modifiers and the health/
condition penalties, with the original minimum of two. These are recovered
arithmetic rules, not a tabletop-rules substitute. On this launch path `$55740`
receives the freshly cleared projectile as A1, so its defender `$9b/$11`
penalties are zero; the selected actor is used for aiming later.

`launch_ranged` computes this roll after resource consumption and the first
successful allocation search, then uses the existing initialization/FX path.
It returns the roll pool, difficulty, successes and number of die faces rolled.
An early gameplay rejection consumes no RNG; second-allocation failure retains
the rolled result and RNG advance. Host errors roll back staged actors, world,
pool and RNG. Ongoing effects require saved-state data through `$55786` and
are explicitly unsupported when a roll would be needed; they are never assumed
to contribute zero. The supported baseline is now connected to desktop firing.

Validation executes **1,538 original dice cases** and **4,096 original ranged
roll cases without stubs**, comparing pool/difficulty, success bytes, actor and
projectile records, die count and final RNG seed. Dice cases include the
`$7fff/$8000/$8001/$ffff` pool boundaries, byte-success wrap, explosions and
extreme difficulty words. Ranged cases include duplicate inventory boosts,
skill/stat byte overflow and item-category/condition boundaries.
The **2,048 full launch fixtures** now run in both supplied-roll and original-roll
variants; the latter omits only audio and HUD transfers. Both variants preserve
all actor/pool bytes, alert state, sound choice, return carry and launch flags,
and original-roll cases additionally compare RNG, die count and the global
success byte. The integration test verifies that early rejection does not roll,
unsupported effects fail without mutation, and FX exhaustion retains the roll.
Generated Ghidra references are in ignored `artifacts/attack-roll-reference`.

## Live projectile handler and damage boundary

`src/projectiles.rs` implements `$14e0a/$14f26/$14f50/$15aba` up to the real
damage call. Each handler tick takes at most four fixed-point velocity steps.
Geometry is checked before lifetime decrement and position commit. A blocked
or expired step keeps the previous position; positive bit-6 collision geometry
is shoot-through. Bounds exit sets `$df68` and clears the active flag on impact.
That flag is query scratch, cleared by every query, not a persistent policy.
The existing explicit four-byte collision scratch is shared with these queries.

Eligible actors are scanned in physical slot order, irrespective of input slice
order or selected target. Active, living, non-incapacitated records with nonzero
health use the original unsigned half-open `$90` footprint. Party-origin flag
bit 3 applies the original actor `$b0/$10/$9e` filters. A zero-success contact
still moves the projectile to the actor and produces the miss/sound path.
A nonzero-success contact returns `DamagePending` at `$55948` entry, after the
original impact flags, velocity reset and positioning. It does **not** apply
damage. Callers must resolve that event before advancing the impact animation.

Impact animation holds until the caller's animation state becomes negative.
`$14f86` then decrements the burst counter: a negative result retires the entry;
otherwise it rolls again using the source actor, restores the saved origin,
changes spread angle, recomputes velocity, resets lifetime and animation fields,
and resumes flight. Fractional coordinates persist as in the original. No more
ammunition is consumed on these continuations. Modified projectile roll contexts
and ongoing source effects are explicit unsupported errors. The outer active
dispatcher, animation playback, rendering and damage are still caller-owned;
the runtime now supplies those pieces for supported player firearms.

The oracle compares **3,264 live handler executions** across six real maps and
eleven synthetic collision bands, preserving all projectile/actor bytes, collision
scratch, bounds flag, RNG, die count, substep count and loop-phase scratch.
Results are 265 moving, 1,595 miss/expiry, 588 damage-pending, 272 animation holds,
323 burst restarts and 221 retirements. It executes original movement, collision,
actor scans and burst rolls without stubs; only audio is omitted, and damage
cases stop at the original `$55948` entry. Input coverage includes stale scratch,
byte lifetime/counter boundaries, inactive/protected/overlapping actors and
fractional coordinates. Unit tests check slot-order hits, zero-success misses
and atomic model errors. A ROM integration sequence launches a burst, flies all
three shots through expiry and retires it, checking RNG timing and one-time ammo
consumption. Its caller explicitly supplies animation-end markers.

The traced next damage path is `$55948`: weapon power/type data, defense pool,
ongoing modifiers, exploding resistance dice and success staging precede the
health/condition changes. `$14e88` then selects impact visuals from actual health
loss and invokes target reaction `$aff2`. These are now implemented through
`complete_impact` below; a returned `DamagePending` still requires that explicit
completion call. Generated Ghidra views are under
ignored `artifacts/projectile-update-reference`.

## Damage, impact visuals and reaction setup

`src/damage.rs` ports physical `$55948` damage for defenders without `$df`
ongoing-effect bits 0/1. It reads weapon difficulty/power and staging data,
computes the defender's wrapped physical/body pool and first enhancement bonus,
rolls the original exploding resistance dice, stages against projectile `$9f`,
then applies armor and the original projectile-side reduction. Byte arithmetic,
sign-only tests, signed-overflow branches and minimum one-die behavior are
preserved. Item ID zero retains the original preceding-record table read, which
can follow consumable exhaustion; malformed high item IDs fail explicitly.

Party health changes use `$3674`'s actor-data rules and mode-3 immunity. NPCs
retain the `$9a` negative-byte one-point damage cap and defeat bookkeeping:
the `$42/$13` counters, wrapping reward countdown and all three party `$8d`
increments. `DamageResult.amount` reports staged damage before mode immunity or
the NPC cap; before/after health records the applied result. `DamageWorld` holds
the affected shared counters and success scratch explicitly. HUD output is not
generated by these backend functions. Ongoing effects could change damage kind/
penetration; this supported baseline clears those scratch values as the original
does and does not invent stun rules for the missing effect path.

`projectiles::complete_impact` consumes a just-returned pending hit, resolves
damage, selects `$14e88` impact visuals from actual health loss and runs `$aff2`
reaction setup. A no-health-loss hit emits the original miss sound ID. Living
targets receive the facing-specific hurt state, handler and eight-tick sound
delay. Dead targets receive the facing-specific death state and handler, dead
flag and sound ID; `$abb6`'s all-party incapacitation check sets its original
60-tick defeat timer. Only setup is implemented: later reaction movement/sound
updates and the defeat-timer/game-over flow are separate. Native actor clocks
reset alongside the original phase/frame fields. Callers must consume each
pending event once; stale health/state and unsupported model inputs fail
without changing actors, projectile, counters or RNG.

Validation executes **4,096 original damage calls**, with 1,231 health changes
and 552 NPC defeat paths, omitting only the low-level HUD transfer. Full actor
records, resistance pool/difficulty/face count, final seed and global counters
match. **512 original reaction calls** match, including 200 immediate death
sounds; only audio playback is omitted. All **588 pending-hit fixtures** also
replay the complete original live handler through damage, impact selection and
reaction setup, comparing projectile/actor records, counters, sounds and RNG;
only audio/HUD transfers are omitted. A connected backend integration test
launches, flies, applies resistance, defeats an NPC and verifies reward/counter
updates, death setup and impact hold without additional ammo use. Its supplied
attack-success byte isolates a guaranteed hit; original roll generation has
separate full-path oracle coverage.

These APIs now support desktop firing. Ongoing modifiers, mode-2 scripts and
mission-aware reaction/death/game-over behavior remain unfinished.
Generated Ghidra views are under ignored `artifacts/damage-reference`.

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
build. General hostile spawning/AI and playable combat remain unfinished; the
target selection alone does not activate the firing path.

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

## Street encounters follow-up

The subsequent [street encounter milestone](street-encounters.md) connects real
ordinary firearm groups, original placement/allocation and offscreen pursuit.
It supersedes earlier statements that world spawning is entirely absent; scripted,
creature and magical encounter continuations remain deferred. No training actors
are inserted into the product. Full defeat/restart remains unfinished.

The [defeat follow-up](defeat-and-recovery.md) connects the original countdown
to a frozen host notice and manual session reset. Full medical recovery remains
unimplemented; 1,024 additional original prefix comparisons cover delay/expiry.

The uninsured solo clinic follow-up now enables one bounded original medical
recovery path, including passage, fee and playable arrival. It supersedes the
earlier statement that all medical recovery is deferred. Other recovery contexts
still use the host notice; see the linked defeat/recovery scope.
