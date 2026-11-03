# Resume here

## User goal and expectations

Build a **complete, working native Rust game executable for the user's Windows
PC**, reconstructing the uploaded Genesis Shadowrun ROM. The user has explicitly
said the early previews are too basic/incomplete. Prioritize connected gameplay
and authentic behavior, not more isolated asset demonstrations. The current
executable is an exploration milestone, not the finished game.

Latest checkpoint (2026-11-03): companion equipment conversion and post-cast
drain APIs, plus native unsafe-current-spell fallback to an inventory firearm.
Read `docs/spell-equipment.md`. 8,192 complete equipment conversions and 4,096
complete drain applications match original instructions (only HUD omitted).
The previous companion firearms and JSON spellbook milestones are included.
This checkpoint still does not enable spell casting. Effect scheduling,
resistance and impact remain unfinished. Spell conversion requires the original
incoming D5 capacity index explicitly; do not guess its caller provenance.
Next recover the effect scheduler and one complete spell action. Maintain typed
Serde/JSON content records with Rust mechanics and source-backed rules.
The user authorized publishing all completed local work after this milestone.
Verification: 66 unit tests and all 49 ROM-backed integration tests pass. The
spellbook/equipment/drain, combat and encounter original-routine checks pass.
Linux/Windows builds and Windows ZIP integrity/import checks pass. Windows GUI
playtesting is still pending. Verify remote main and git status on resume.

Latest continuation (2026-11-03): recovered session spellbooks and companion
spell selection. Read `docs/spellbooks.md` and `src/spellbooks.rs`. The embedded
Serde JSON catalog in `data/combat/spellbooks.json` drives verified source-record
references and classifications; Rust executes arithmetic and selection. All 170
initialization bytes, 4,096 drain estimates and 4,096 selection prefixes compare
against original instructions without stubs. Equipment mutation/casting remains
explicitly deferred with staged RNG/source state. This is progress toward full
companion magic, not enabled casting. Do not invent an empty book or gun preference.
Continue with original equipment conversion and one complete spell action,
including resistance and actual drain, then the verified job/contract loop.
Verification: 65 unit tests and all 46 ROM-backed integration tests pass; the
new spellbook oracle and existing combat regression oracle pass. Native Linux
and Windows builds are refreshed; actual Windows GUI testing remains pending.
The user's latest instruction reaffirms gradual JSON integration and original
behavior fidelity. Prefer JSON for reviewed content records, Rust for mechanics.

Latest milestone (2026-11-03): bounded companion firearm combat now runs through
original decision scheduling, target retention, behavior-specific range goals,
finite-reserve reloads and shared live projectiles/damage. Joshua fights as a
companion after control switches. Read `docs/companion-combat.md` first.

The original spell-selection entry is explicitly deferred when session spellbook
state is required. Default Ricky/Freya do not get an invented firearm preference.
Mundane companions and the original low-condition firearm branch are supported.
Do not label this full companion AI. A real map-5/default-roster test, with Ricky
selected and no player firing or actor/stat edits, records Joshua firing at tick
148. Alerted offscreen enemies now continue approaching even after an empty
visible target list clears combat, matching the recovered `$142ae` branch.

New combat oracle evidence: 1,024 scheduler cases, 2,304 range goals, 1,280 complete
companion decisions (including walls/reloads/pool failures) and 256 deferred
spellbook-entry prefixes. The encounter oracle adds 128 cleared-combat pursuit
decisions (256 total). Existing original-routine combat checks remain passing.
62 unit tests and all 44 ROM-backed integration tests pass. Combat, encounter
and follower original-routine oracles pass. The Windows package is rebuilt and
its ZIP hashes, x64 PE format and imports are verified. Windows GUI/input playtesting remains pending.

Session spellbook initialization and `$13a92` selection have since been recovered
as described above; casting and equipment mutation remain unfinished. Do not redo
completed navigation, firearm, archive or medical work. Changes in this milestone
are local; the previous archive/recovery checkpoint was already published in
`5041da9` with the handoff update `90bfc84`.
Do not publish environment configuration, private inputs or generated analysis.
The remaining historical sections record earlier milestones and their old counts.

The user resumed development after cloud setup. The current local milestone
adds original fallback wall hugging and physical actor-contact separation to
companion following, plus contact-triggered travel on the verified routes.
Target acquisition and the projectile obstruction probe are now connected;
weapon cooldown ticking and source-only ranged animation setup are verified
in `src/weapons.rs` (4,096 original full-actor comparisons each for cooldown,
dispatcher tick and ranged setup, without stubs). Launch admission through the
first projectile-slot lookup and resource consumption are now implemented as
backend APIs: 18,432 resource and 6,144 launch-prefix original comparisons pass,
with only low-level HUD transfers omitted. See `docs/combat-targets.md`.
Projectile initialization, alert logic, burst setup, second FX allocation and
FX initialization are implemented in `launch_with_resolved_roll`; `launch_ranged`
now adds original `$55640` rolls for the baseline without ongoing effects.
`src/attack_roll.rs` matches 1,538 dice cases and 4,096 ranged-roll cases without
stubs. All 2,048 normal-world launch fixtures match in both supplied-roll and
original-roll variants, omitting only audio/HUD transfers in the latter.
Both allocation failures preserve the original partial state; FX failure leaves
the live projectile in place. Environment mode 2 scripts remain unsupported.
`src/projectiles.rs` now implements live movement, actor scans, impact entry,
animation holds and burst restarts/retirement: 3,264 original handler comparisons
match (only audio omitted; hits stop at the real `$55948` damage entry).
`src/damage.rs` now resolves baseline physical `$55948` damage; 4,096 original
calls match (1,231 health changes and 552 NPC defeat paths; only HUD transfer
omitted). `projectiles::complete_impact` adds health-loss impact visuals and
`$aff2` hurt/death setup, including party-defeat timer setup. All 512 reaction
cases and 588 complete live impacts match (only audio/HUD transfers omitted).
Player firearm firing is now enabled: **hold F**, optionally T to target.
`src/combat_runtime.rs` integrates the selected-player cooldown/request gate,
finite-reserve reload, actor firing/hurt poses, six projectile/FX slots, immediate
hit completion, terminal frame timing and original resident sprites into Game.
1,024 gates, 1,024 reloads, 1,024 failed-launch/reload continuations and 1,024
animation calls match original routines.
Next extend ordinary NPC combat coverage, companion combat and mission-aware death callbacks,
full defeat/game-over flow and the remaining weapon/effect systems. Ongoing
modifiers, modified burst roll contexts, environment mode 2
scripts and footprint-flag projectile collisions return explicit unsupported errors.
The earlier firing checkpoint passed 59 unit tests and 27 ROM integration tests. The 313-tick firing replay records shots, hits, ammo, reserves and pool use.
The combat oracle, including all previous timing/target/probe comparisons,
passes. The preceding checkpoint also passed avoidance,
followers, world, transitions, interior, dialogue and hiring validators.
Do not restart completed ROM/map/hiring/navigation work. No clarification is pending. Do not assume runtime files outside Git persist.

The user explicitly asked to introduce JSON **slowly** for recovered dialogue
and quest logic. Keep the Rust engine and use small verified records; do not
migrate unrelated systems or invent missing quest rules.

## Durable checkpoint

Repository: https://github.com/okcmallrat/Shadowrealm
Remote branch: `main`; this environment's local branch is `work`.
Published starting checkpoint: `5041da9` (archives, medical recovery and rehire),
with published handoff update `90bfc84`. The companion firearm follow-up is local
on top of that checkpoint. These local changes are included in the newly authorized publication.
Inspect `git status` on resume. Environment configuration/private inputs are excluded.

Windows package: `releases/Shadowrealm-windows-x64-exploration.zip`.
SHA-256: `a0a21a0da5fa77cbdc842172268d0474fef90c4f00ad95a035fccf5c3bbead01`.
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

The native reconstruction remains unfinished. General quest/story scripting,
mission-aware death/recovery, earning money and contract completion/dismissal,
other runner conversations/recruitment, full companion combat AI, melee/magic/ongoing
effects, inventory/menu interactions, insurance purchasing, character creation,
audio and disk save/load are absent or incomplete. Only the verified exterior,
Space Needle and medical routes are enabled. The current roster/funds/camera
are development fixtures, not a recovered new-game/save state. Windows GUI/input
has not been playtested here. Do not describe assets or isolated routines as a
complete game. The sections below retain historical implementation details.

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
implemented as described below. Target acquisition is now connected, while
weapon attacks and damage remain unimplemented. Do not claim all-room robust navigation.

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

## Published milestone: fallback wall hugging (`90fa5c4`)

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

## Published milestone: physical actor contacts (`90fa5c4`)

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
not implemented. At that published checkpoint, controlled separation onto triggers remained
blocked; the local follow-up below enables the bounded verified travel paths. Initial RNG seed zero and simulation frame parity
are explicit native host choices, not recovered hardware boot timing. Fades,
warm-up position holds and dialogue freezing remain. No JSON migration was needed
for physical motion: typed Serde/JSON dialogue/hiring records remain in use.

The native Linux desktop and Windows builds/package are refreshed. Actual
Windows GUI/input remains untested. The current local ZIP digest is recorded
above and in releases/SHA256SUMS. The fallback/contact changes are recorded in
`90fa5c4`; subsequent travel changes below are local.

## Earlier local milestone: contact-triggered travel

`avoidance::respond_with_travel` now permits controlled separation onto the
previously supported noncombat exterior seams (maps 0/3/5) and Space Needle
entry/return. It preserves the final-probe physical gate, two-negative-corner
requirement, trigger latch/release and first-negative-corner return orientation.
`TransitionPlan::space_needle_from_contact` uses the actual post-response player
position and recovered corner orientation. The existing passage acknowledgement,
fades, party preservation and saved return context still apply. Other doors and
special script/combat/mission gates remain blocked.

51 unit tests and all 14 explicitly enabled ROM integration tests pass. The
extended avoidance oracle adds 3,936 gate/motion/latch cases (2,038 original
location-dispatch requests, 55 from discarded third probes), comparing actor
records and return formations up to the `1651e` dispatch boundary. The location
label/UI update is skipped; downstream routes/spawn/passage/restore retain their
separate original-routine checks. Functional contact tests complete exterior
travel with a switched leader and an acknowledged Space Needle round trip using
real physical overlap and no directional input at activation. Warm-up zero
samples release the trigger latch, so later contact-driven travel still works.

See `docs/actor-contacts.md` for scope and commands. No new JSON migration was
needed: the typed Serde dialogue/hiring records remain unchanged. These follow-up
changes and their refreshed Windows ZIP are local; only `90fa5c4` above was pushed.
Actual Windows GUI/input playtesting remains outstanding.

## Earlier local milestone: combat target acquisition

Read `docs/combat-targets.md` and `src/combat.rs`. The user clarified that
“combat training” meant continuing to the next implementation step, not adding
a practice mode. T now toggles target selection outside dialogue, using the
original `13874/1438c/14446` list filters/cadence and `143f6/14492` target retention
and random replacement. `15c58` close-range probing is implemented separately;
it does not establish weapon readiness, line of sight or hit chance.

Targeting and physical contact share the original random recurrence. The mode is
off by default, with native host controls/brackets. Neutral talkable NPCs can be
listed; targeting does not establish hostility or attack permission. No enemies
or training-room spawns were invented. `npc::trent_contact` now honors original
`a2/ac` linked contact windows, preserving the existing conversation. Party/map
changes invalidate controlled target links; dialogue/travel freezes remain.

53 unit tests and all 16 explicitly enabled ROM integration tests pass. The new
combat oracle compares 768 acquisition cases (441 replacements, 116 retained
pointers, 147 counted/omitted empty-list callbacks), 1,024 close-range probes and
256 refresh/retention/RNG phases. Dialogue contact checks expand from 69 to 469,
covering links in both directions/fields; existing JSON branches remain unchanged.
The avoidance/travel, follower, world, transition, interior and hiring oracles pass.
The two new functional tests cover the target toggle/marker/retention/leader
switch without damage or ammo use, plus targeting Trent and opening the original
wider-range conversation. Native Linux and Windows builds/package are refreshed;
actual Windows GUI/input playtesting remains unverified.

This targeting checkpoint, the prior contact-triggered travel work and their
current Windows ZIP are local and not yet committed/pushed. The last verified
published commit remains `90fa5c4` above. Typed Serde/JSON dialogue/hiring stays
in use; no broad data migration or speculative quest records were added.

## Latest local milestone: projectile obstruction probe

The published cloud environment restored the existing target/contact/travel
changes and both private inputs. They were inspected and preserved, not reapplied.
Do not publish environment/configuration or push code without a new user request.
The last verified GitHub commit remains `90fa5c4`; subsequent changes are local.

`combat::line_of_fire` now follows `14d8e/14ff8/15118/14f26` and the point-query
branch of `e7c8`, plus intended-target matching in `15aba`. It preserves launch
rounding/offsets, signed velocity, 32-bit fractions, the 27-step cap and original
projectile-passable geometry. This is an obstruction probe, not a full attack.
TargetReached, Blocked, TraceLimit and OutsideModel remain distinct. Only the
requested actor can satisfy the probe; intervening actors do not intercept it.
The source, target and other actors are not mutated.

Initial collision scratch is explicit because the original point/diagonal query
retains other corner bytes. The UI deliberately uses cleared scratch for an
independent deterministic query. Red target brackets/title status report a
blocked probe, gray reports unmodeled geometry, and yellow still only means
selection. Rendering does not advance simulation, consume RNG/ammo, create live
projectiles, change target links, or apply damage. Firing was disabled at that
checkpoint; the current runtime at the top of this handoff now enables it.

54 unit tests and all 17 explicitly enabled ROM integration tests pass. The
combat oracle adds 2,176 complete original probe calls: 271 target reaches,
1,169 blockers and 736 trace-limit outcomes, on six real maps and eleven synthetic
collision bands with three initial scratch configurations. All 256 scratch actor
bytes, steps, collision scratch and outcome match; source/target/intervening actor
records remain unchanged. No subroutine on the projectile-query path is stubbed.
The functional wall/marker test checks red-to-yellow feedback with unchanged
actor records, wallet and tick. Earlier target/contact/travel/follower/world/
transition/interior/dialogue/hiring checks remain passing. Linux/Windows builds
and the versioned ZIP are refreshed; actual Windows GUI/input remains untested.

See `docs/combat-targets.md` for scope and commands. New Ghidra working-copy
exports are under ignored `artifacts/weapon-reference/ghidra`. No previous changes
were reinstalled/reapplied, no environment draft was edited, and nothing was
published or pushed during this milestone. The Rust engine and existing typed
Serde/JSON dialogue/hiring records remain in use.

## Best next milestone

Firing, street firearm encounters and normal party medical recovery are enabled.
Recover companion spellbook/selection and mission-specific defeat callbacks next, followed by melee/magic and
ongoing effects. Connect earning money and contract resolution through a verified job
flow, introducing JSON only in small source-backed records. Full combat and
additional scripted travel remain unfinished.

The ROM and corrected GZF are privately retained at `/workspace/inputs/Shadowrun.bin`
and `/workspace/inputs/ShadowRunBin.gzf`. Rust activation remains below. Python
validators now use `/workspace/.venvs/shadowrealm/bin/python`; Ghidra/Java are
under `/workspace/reverse-tools` as documented. Private analysis project:
`artifacts/ghidra-projects/FallbackReference.gpr`; exports:
`artifacts/fallback-reference/ghidra`, `artifacts/avoidance-reference/ghidra`,
`artifacts/contact-travel-reference/ghidra` and `artifacts/combat-reference/target-ghidra`.
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
| src/avoidance.rs | Physical actor contact flags/headings/shared RNG and temporary separation motion |
| src/combat.rs | Target lists/retention, close range and projectile obstruction probe |
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

- 54 Rust unit tests pass; 17 ROM-backed integration tests pass explicitly.
- Combat targeting: 768 acquisition cases, 1024 close-range probes and 256
  refresh/retention phases and 2176 complete projectile probes match original
  routines (`tools/validate_combat.py`).
- Physical contacts: 1536 cases, 1024 response steps and 2700 ordered party ticks
  match actor bytes and RNG state; 3936 contact/travel gate cases compare the
  dispatch boundary and return formation (`tools/validate_avoidance.py`).
- Noncombat followers: 1024 math pairs, 768 goals / 762 supported decisions,
  1024 movement records, 900 regular scheduler/warm-up ticks, 7200 wall-hugging
  ticks, 192 fallback/waypoint decisions and 4 closed-pocket decisions match;
  6 out-of-scope raster routes remain explicitly withheld
  (`tools/validate_followers.py`).
- Trent first-hire: 54 original fee quotes and 24 complete purchase/capacity/funds
  cases match (`tools/validate_hiring.py`).
- Trent dialogue: 15 node/text records, 26 original text/flag branch executions
  and 469 unlinked/linked contact cases match (`tools/validate_dialogue.py`).
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
