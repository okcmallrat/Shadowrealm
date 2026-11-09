# Resume here

## Latest checkpoint (2026-11-09): Stoker encounter, rest and unarmed NPCs

Read `docs/stoker-ambush-and-rest.md` first. The user explicitly requested a push
after this milestone. Implementation commit `098cd14849451d2c1415d99178359c01f21a8f11`
(tree `6124589e63df6a66f44f780205b4f67e68e400e5`) contains all preceding local
Stoker/taxi, Ares, saves, hotel/Karma and new-game work on top of `3a3056f`, together
with the verified Windows ZIP. This documentation followup records that checkpoint;
the authorized publication target is `origin/main`. Preserve the supplied ROM/GZF
and private analysis; none is packaged or committed. Older local checkpoint notes
below describe their state before this cumulative publication.

Stoker rents the original 20-nuyen room before/after Michael's belongings and
reuses partial rest, per-runner Karma and checkout. Courier service retains
priority. The post-objective MORE now executes action 233 (`$249c4`): shared RNG
sample(100), <80 normal manager, otherwise the real close-placement group at
`$1d6e2a` (gunner `$1d66a4` plus two unarmed `$1d665a`). No placement/available
slot returns the original manager node; real spawns show `$f4acc` and fight.
Only this script's verified human group permits unarmed spawn admission; ordinary
street group scope otherwise retains the previous firearm restriction.

`src/unarmed.rs`, `src/stoker.rs`, `data/combat/unarmed.json`,
`data/encounters/stoker.json`, export modules and validation tools provide the
typed references and native rules. The dialogue pilot adds actions 6/233 and
manager/ambush nodes. Human unarmed NPCs now use original close-range pursuit,
attack scheduling, pose, dice/armor/focus/defense/resistance and health/condition
rules. Strikes stage actor/FX/world/RNG changes; visits also stage passage and
sprite assets before committing. Preserve D7's defender-cyberware high byte in
Melee, attacker focus ownership, byte quirks and NPC reward timing. Fresh hit FX
can have zero animation period; execute original arithmetic and terminal clamp.
Player/companion punches, melee weapons and special unarmed banks remain guarded.

Condition zero now enters source incapacitation without zeroing health. Normal
medical preparation supports condition-only defeat and all three Joshua classes,
using the original fee/restoration/archive/DocWagon core. Unsupported story/mode
contexts retain their existing guarded defeat notice. Full original selection/
service reload cadence and original motel room graphics/fades remain incomplete.

The user's Ghidra working copy supplied visit/spawn/attack/damage references;
private exports/logs/captures are `/workspace/work/stoker-encounter`. Original
comparisons pass 512 complete action-233 visits with no stubs, 2,048 unarmed
damage cases, 256 full close attacks, 256 incapacitations and 256 zero-period FX
clocks. Only audio/HUD transfer is omitted where applicable. Medical cores now
cover 1,792 cases, including 768 condition-only cases across all classes. Existing
combat, encounters, progression, rest/Karma, clinic/archive/rehire and saves
original-routine oracles also pass.

The earned development-roster route completes six jobs and Michael's belongings.
Five normal post-objective visits rent rooms; visit 5 (zero based) rolls the
actual three-NPC fight. At tick 15935 it ends with six NPC punches, eleven NPC
shots and fourteen player shots; Joshua has health 10 / condition 1. The party
survives. A subsequent 20-nuyen stay restores condition; Joshua Body 4 -> 5 costs
five Karma, leaving three. Disk save/reload at tick 16155 retains 410 nuyen,
Body/Karma, six completed jobs and story/DocWagon. No actor/stat/inventory, money,
RNG, enemies, positions, maps or job outcomes are assigned after construction.
This is an explicit development roster, not earned recruitment. A separate fresh
solo start rents before the first objective, receives the original Karma refusal
and saves with zero nuyen. Live punch, failed attack asset rollback, all-class
condition-only recovery and earlier connected routes pass.

All 76 unit/pure + 118 ROM-backed tests pass (194 total, desktop enabled).
Native Linux desktop and Windows release builds pass. Linux GUI launch failed
to create a window here; Windows GUI/input/filesystem playtesting remains pending.
Gang notice, aftermath, room/upgrade and resumed HUD frames were inspected.
The Windows ZIP is 686,473 bytes, with CRCs, all 22 hashes, exact membership,
source README/EXE identity and x64 EXE/DLL headers/imports verified. SHA-256:
`89d2c04119b446479c401fc11b2b2dd35d80b266ea86388216c210f6621418c9`.
Further story/cyberspace, recruitment, other services/shops/job types, courier
ambushes, wilderness, original full menus/cinematics and audio remain unfinished.

## Latest local continuation (2026-11-09): solo new game and opening menu

Read `docs/new-game-and-opening-menu.md` first. Preserve all prior local work
(Stoker/taxi, Ares, saves, hotel/Karma) together with this milestone. HEAD is
still `3a3056f`; no new publication was requested or performed.

Normal desktop launch now opens New Game / Load Game with no placeholder Game.
Choose Samurai, Decker or Gator Shaman, review original attributes, and Begin.
`Game::new_game` constructs the original solo map-2 hero, class inventory/stats/
spellbook, 20 nuyen, zero contracts/knowledge/story flags except initial Michael
lead 1. Native archive initialization retains Samurai archive 0 from `$59318`
before class selection; saving later snapshots the chosen live hero over it.
`Game::load` remains the explicit development constructor and retains its earlier
semantics. `--explore` restores the prior Joshua/Ricky/Freya development roster;
an explicit MAP, --solo or --nuyen also preserves direct development launch.
--save-dir alone retains the normal opening menu.

`src/newgame.rs`, `src/start_menu.rs`, `src/session.rs`, `Game::new_game`,
`data/newgame/references.json`, `src/newgame_export.rs` and
`tools/validate_newgame.py` provide the typed references, native lifecycle and
oracle. The supplied Ghidra working copy supplied `$59092/$59248/$59318/$42b0`
references; private exports/logs/captures are `/workspace/work/newgame`.
All three original class-selection/represented bootstrap paths match: every
persistent hero byte, eleven archives, five spellbooks, absent companion IDs,
wallet/story/jobs/knowledge, selected actor, map/camera/spawn. Only HUD tile
transfer is omitted; resource normalization executes. Original cinematics,
boot seeding and warm-up timing are outside the comparison.

R opens the menu during normal play and retains the suspended game. Resume
returns; Back cancels class selection. Begin explicitly replaces unsaved Game
state. No start/reset operation writes/deletes saved files. Load Game reuses the
validated primary/backup loader and works directly at launch or after defeat;
failed loads/new-game asset preparation leave the old Game intact. Held confirm
cannot dismiss the resulting notice or spill into gameplay. In development mode
R still resets the chosen development setup. Opening UI, class preview and brief
Redmond orientation are host panels, not the original title/intro rendering.

The controller-only fresh-menu Samurai route starts alone with 20 nuyen, earns
six courier rewards (275), retrieves Michael's belongings (525), saves at tick
15424, drops Game/Session and loads Slot 1 from another opening menu with no
running Game. Solo/class identity, 525 nuyen, seven Karma, six jobs, story/deck
flags persist. No actor/stat, inventory, money, map/position, RNG, enemies or job
results are assigned after normal menu construction. No encounter spawned.
All three archetypes also move and survive disk round trips. Other checks cover
freeze, cancellation/resume, failed assets, missing/corrupt saves, backup recovery,
load after defeat, file preservation and input handoff.

The solo route exposed a previous courier dead end: Halloweener delivery node
`$110d72` has MORE -> `$111566` (terminal text `$117868`), missing from the catalog.
That verified node is now included. Enabled courier stop continuations must
resolve; payment/flags are unchanged. The test helper bounds delivery dismissal
and drops stale A* queue entries, avoiding a silent spin on future bad links.
The existing courier/Stoker/taxi/save original-routine comparisons all pass.
All 75 unit/pure + 113 ROM-backed tests pass (188 total, desktop feature enabled).
Rendered opening/class/briefing/load and solo-earned panels were inspected.

Native Linux desktop and Windows release builds pass. Linux GUI launch failed
to create a window here; Windows GUI/input/filesystem playtesting remains pending.
The verified local Windows ZIP is 669,648 bytes; CRCs, all 22 payload hashes,
exact membership, source README/EXE identity and x64 EXE/DLL headers/imports pass.
SHA-256: `cc347699c425580f11c57861954fa5cd6e1f4b0e19739c7fe36bacf8bd1fd265`.
No ROM/GZF/saves/generated frames are packaged. Further story/cyberspace,
recruitment, other services/shops/job types, courier ambushes, wilderness,
original full menus/cinematics and audio remain unfinished. Continue a connected
story/gameplay milestone without bypassing unrecovered branches.

## Latest local continuation (2026-11-09): paid hotel rest and Karma

Read `docs/hotel-and-karma.md` first. Preserve all prior local Stoker/taxi,
Ares and save/load changes together with this milestone. HEAD remains
`3a3056f`; no new publication has been requested or performed.

Downtown Gates Undersound Hotel (map 0, C3 / location 3) now opens its original
`$f4910` greeting and 75-nuyen room offer. Payment action 6 uses `$21458`.
Acknowledging success performs `$ef08` rest once, then a host room menu allows
per-runner Karma spending and checkout. World movement, effects, NPCs and
selection are frozen; street saves/loads remain guarded until checkout.
`TravelScope` now permits C3 in controlled physical separation as well as normal
walking. Matching courier service retains its existing priority.

`src/advancement.rs`, `src/hotel_game.rs`, `data/hotels/gates.json`,
`src/advancement_export.rs` and `tools/validate_advancement.py` provide typed
references, native rules, transactional UI/HUD changes and original-routine
comparisons. Shared `$3674/$37ec` helpers preserve post-cast drain behavior.
Rest adds ten condition; health >=5 adds two, 3..4 adds one, <=2 needs medical
attention. It is not full healing. Karma uses race caps, Sorcery class gate,
ordinary skill cost 2*next, specialized gun cost ceil(1.5*next), base-Firearms
initialization of zero specializations, signed/byte quirks and original limits.
Essence and Magic cannot be trained (Magic is raw `$7f`, after two Essence bytes).
Only the chosen actor's stat and Karma change. Upgrade confirmation starts Yes.
Other hotels and Stoker's post-objective action 233 remain guarded; do not bypass
that real random encounter to expose Stoker rest.

The supplied Ghidra working copy provided hotel/quote/upgrade/payment references;
private output is `/workspace/work/karma`. The oracle passes the real hotel
entrance/fee, 13 payment boundaries, 5,120 original Karma quotes and complete
actor decisions/writes, and 1,024 full three-runner rests. Only display, wait,
confirmation input and subsequent stat-screen redraw/loop are omitted. The
existing contact/travel, casting, healing and save/load oracles also pass.

The connected controller route starts with ordinary map-2/default-roster setup,
earns six courier rewards, recovers Michael's belongings, takes the Downtown
taxi and buys the room. It cancels an upgrade, then raises Joshua Body 4 -> 5
for five Karma and Ricky Firearms 2 -> 3 for six of his own Karma. Checkout
cancellation works; a real disk save/drop/new Game reload retains both upgrades,
remaining Karma, six completed jobs and 435 nuyen at tick 15380. No actor/stat,
inventory, money, map/position, RNG, enemy or job-result assignments occur after
normal construction. No encounter spawned. Additional fixtures cover failed
font/HUD rollback, insufficient funds, held keys, physical C3 entry with switched
control, partial rest and eligibility/caps. All 74 unit/pure + 108 ROM-backed
tests pass. Rendered room/rest/upgrade/resumed HUD panels were inspected.

Native Linux desktop and Windows release builds pass. Windows GUI/input and
filesystem playtesting remains pending. The local release ZIP is 646,852 bytes;
CRCs, all 22 payload hashes, exact membership, source README/EXE identity, x64
EXE/DLL headers and imports are verified. SHA-256:
`37726aa3501eceaf22008214d0053ecebad298fa25350af7af5ec98fe30c9853`.
No ROM/GZF/save data is packaged. Room graphics, original full menus/new-game,
further story/cyberspace, other services/shops/job types, courier ambushes,
wilderness and audio remain incomplete. Next advance another connected gameplay
objective/service without enabling unsupported story/encounter branches.

## Latest local continuation (2026-11-09): native save/load checkpoints

Read `docs/native-save-load.md` first. All prior Stoker/taxi/Ares changes remain
local, together with this milestone; HEAD is still `3a3056f`. Preserve all
modified/new source, tests, catalogs, notes and the verified Windows package.
No new publication has been requested or performed.

F5/F9 open two native save/load slots from a safe city street. Files live in
`saves/` beside the executable; `--save-dir DIRECTORY` overrides that location.
The previous loadable primary becomes a backup, and fallback loads identify it.
Bad loads retain the running Game; failed writes retain a usable prior save.
R/closing do not autosave; R keeps files and F9 can restore a checkpoint.
For loading after defeat, start/reset the default Redmond session first.

`src/saves.rs`, `src/save_game.rs`, `data/saves/references.json`,
`src/save_export.rs` and `tools/validate_saves.py` provide the original image
layout/checksum, typed Serde/JSON native envelope and CRC, bounded/atomic file I/O,
staged Game reconstruction and oracle. The original street menu `$efb4` blocks
interior/combat saves. The native path adds conservative guards for active NPCs,
effects and unresolved party recovery. It is not a full emulator save state or
an importer for arbitrary SRAM/emulator saves. Motel rest remains unfinished.

Persistent money, equipment/ammo, stats/Karma, active/inactive companion archives/
contracts/opinions, spellbooks, knowledge/story/DocWagon, courier/package state,
map/camera/positions survive closing the process. Host metadata also preserves
tick/RNG, job review/acceptance and completed-job count. Loading reconstructs
actors/map clocks and clears transient effects/targets/metrics, selects Joshua,
and uses the original class-dependent portrait selection. Validate inactive
archives/assets before replacing live state. Descriptor 8/10 are not full world
dimensions; use collision and host-camera bounds. Reject incompatible precision.

The user's Ghidra working copy supplied the save/load references. Private exports,
logs/captures are `/workspace/work/saves`. The original writer/admission oracle
passes 512 complete 2,048-byte images, both odd-byte SRAM copies, overwrite/global/
coordinate/checksum behavior and corruption rejection without stubs. Another 512
persistent loads match companion initialization/compact records, positions,
camera, flags, Joshua control and portrait. Only `$3074` map/graphics/recovery
loading is omitted in that comparison; the native asset path is separately tested.

The earned-money/Ares controller route now writes a checkpoint at tick 15373 and
drops the old Game. A separate CLI process reads the slot with six completed jobs,
the purchased Warhawk, five loaded rounds, story flag and 5 nuyen. A new Game
loads it and fires, consuming one round. There are no actor/inventory/stat, money,
RNG, enemy, position, map or job-result assignments during that route. No encounter
spawned. An explicit collected-package fixture delivers/pays once after loading.
Backup/overwrite, independent slots, corrupt/unsupported input, failed writes,
cleanup, menu freezing and held-key notices also pass. All 73 unit/pure and 103
ROM-backed integration tests pass. Rendered save and resumed HUD panels were
inspected. Actual Windows GUI/input/filesystem playtesting remains pending.

Native builds/package verification are recorded in `docs/native-save-load.md`
and `releases/SHA256SUMS`. Native Linux desktop and Windows release builds pass.
The Windows ZIP is 628,574 bytes; CRCs, all 22 payload hashes, README/executable
identity and x64 PE/DLL imports are verified. SHA-256:
`357281f64060ba2fcddb37e03f8c8f7923d2522e5b2f38d193676d42de084331`.
The existing medical/archive/rehire oracle also passes. Original menus/new-game, wilderness/script-specific
saves, the wider story, other services/shops/job types, courier ambushes and audio
remain unfinished. The ROM/GZF remain private inputs and are not packaged.

## Latest local continuation (2026-11-08): Ares equipment and ammunition

Read `docs/ares-equipment.md` first. The preceding Stoker/taxi changes are still
present, together with this new local milestone; local HEAD remains `3a3056f`.
No publication has been requested. Preserve all existing modified/new files.

Redmond C-B (location 11) now opens Ares' original `$110dd2` passage and a bounded
shop. Matching courier service retains priority. Ares' actual five firearms and
ammo clips can be bought with original stock, price multipliers, selected-runner
negotiation, first-empty inventory allocation, funds checks and clip limits.
Buying a gun does not auto-equip it. I opens the selected runner's firearm
inventory; selection includes original ammo conversion and same-slot unequip.
Other merchandise, armor, selling, accessories and full inventory actions remain
guarded. The stock/confirmation/I panels are host UI, with world/selection freeze
and transactional actor/wallet/HUD preparation.

`src/shops.rs`, `data/shops/ares.json`, `src/shop_export.rs` and
`tools/validate_shops.py` contain the typed reference pilot, Rust rules and oracle.
The user's Ghidra working copy supplied the original shop/equipment views;
private exports/captures/logs remain in `/workspace/work/shops`, outside Git.
All 4,096 purchases and 4,096 explicit firearm selections match original actor
bytes, money and prices. Only display/wait/confirmation UI or HUD/menu redraw is
omitted. Preserve `$121fe`: an empty old weapon equips directly, even if its
stale slot byte equals the selected slot. The same-slot toggle applies to an
actual equipped weapon. Do not invent incoming D1 for old non-firearm contexts.

The connected test uses only ordinary controls/host menus after the normal
map-2/default-roster construction. Six jobs and Michael's belongings leave 530
nuyen; Joshua buys a 500-nuyen Warhawk and 25-nuyen clip, equips it and fires
seven shots through a real reload. At tick 15339 the balance is 5, reserve clips
are 10 (10 -> 11 purchased -> 10 reloaded) and five rounds remain. No actor,
equipment, money, RNG, enemy, position, map or job outcomes are assigned. No
encounter spawned in that trace. The prior earnings -> belongings -> Downtown
route still passes. Additional fixtures cover refusal/cap/full-inventory order,
unsupported items, cancellation, freezing and failed purchase/equipment HUD
rollback. All 71 unit/pure and 98 ROM-backed integration tests pass. Existing
courier and Stoker/taxi original-routine checks pass. Rendered shop, inventory and
firing HUD panels were inspected; Windows GUI/input playtesting is pending.

Rebuild/package details are recorded in `docs/ares-equipment.md` and
`releases/SHA256SUMS`. Native Linux desktop and Windows release builds pass.
The Windows ZIP is 580,651 bytes; CRCs, all 22 payload hashes, README/executable
identity and x64 PE/DLL imports are verified. SHA-256:
`4092db2ce63f42c48aaead732629b980ebc8d2ee83a21f5d6c3d9f44d30ba27f`.
Source and package updates remain local. Next extend a
connected gameplay service or objective; saving, broader story/new-game, full
menus, other shops, courier ambushes and other job types still need work.

## Latest local continuation (2026-11-08): first objective and city travel

Read `docs/stoker-and-taxi.md` first. Work continues on top of clean local HEAD
`3a3056f` (courier/automatic magic checkpoint); these new changes are local.
The desktop now defaults to map 2, a Redmond development entry point with the
same roster and original 20 nuyen, rather than map 0. This is not new-game
creation or recruitment. C calls the recovered taxi service from a city street.

Six courier jobs can fund Michael's 250-nuyen back rent at Stoker's. The original
purchase grants the hidden 500 nuyen, byte Karma, cyberdeck ownership and story/
DocWagon bit 0, updates the lead flags and shows the original stash passages.
The wristband enables the existing DocWagon recovery path. The same bit unlocks
taxi travel among maps 0..5, with original selectors, fares, same-district/story
refusals, insufficient-funds free rides, armband rules and descriptor arrivals.
Do not replace the original free ride with an insufficient-funds block.
Courier stop service still takes priority over the ordinary motel interaction.

`src/progression.rs` and `data/dialogue/stoker-taxi.json` provide typed verified
references, native decisions and transactional purchase/payment preparation.
`Story` stores `$fffc30` leads and the represented `$fffc27` inquiry byte;
`Game::recovery_flags` remains the shared `$fffc1e` story/DocWagon word. The
catalog includes a distinct options-only taxi record: `$fe5ec` has no leading
exposition pointer. Do not parse it with the ordinary dialogue node layout.

The controller-only route uses ordinary map-2 spawn/default roster and starts
with 20 nuyen: six jobs -> 280 balance -> belongings -> 530 -> Downtown taxi ->
510 at tick 14746. It assigns no actor/stat/book/equipment, position, map, RNG,
money, enemy or job outcome during the run. No encounter spawned in this trace.
The new oracle passes 2,048 complete purchases/refusals, 2,048 taxi gates, 2,048
payments/free rides/visa checks, 1,755 armband decisions and 30 city route/arrival
cases. Relevant existing courier, interior, exterior and recovery oracles pass.
Seventy unit/pure tests and all 93 ROM-backed integration tests pass (the full
pre-existing suite plus the final jobs/progression checks). Native rendered menu,
quote, stash and arrival panels were inspected. Native Linux release builds pass;
GUI launch failed to create a window because this instance has no X11 display or
callable desktop-control tool. Actual Windows GUI/input playtesting is pending.

The Windows package is rebuilt and verified as recorded in `releases/SHA256SUMS`.
It is 566,213 bytes; CRCs, all 22 payload hashes, README/executable identity and
x64 PE/DLL imports are verified. SHA-256:
`2360a9882546acd4738b28195fc49cb9e3e18c595726656f43df71046c9a1a3e`.
No new publication has been requested or performed in this continuation.
Generated Ghidra references/logs/captures stay in `/workspace/work/travel`, outside
Git. The user's ROM/GZF are still `/workspace/inputs/Shadowrun.bin` and
`/workspace/inputs/ShadowRunBin.gzf`.

Motel rest, the post-objective motel random encounter action, cyberspace/deck
equipment, further story/shops, original menus/new-game, courier ambushes, other
job objectives, wilderness travel, saves and audio remain incomplete. The C menu
combines the original greeting/price request into a host fare confirmation;
original map highlight/portrait/input/fade/audio cadence is not reconstructed.
Prefer the next concrete gameplay milestone to more isolated renderers.

## User goal and expectations

Build a **complete, working native Rust game executable for the user's Windows
PC**, reconstructing the uploaded Genesis Shadowrun ROM. The user has explicitly
said the early previews are too basic/incomplete. Prioritize connected gameplay
and authentic behavior, not more isolated asset demonstrations. The current
executable is an exploration milestone, not the finished game.

Publication checkpoint (2026-11-05): the user requested pushing all completed
local work and stopping for now. This checkpoint includes Invisibility/Rockskin,
automatic companion magic and the first repeatable Gunderson courier loop,
alongside the verified 548,800-byte Windows ZIP. Validation remains 69 unit + 87
ROM-backed integration tests, relevant original-routine comparisons, and native
Linux/Windows release builds. Windows GUI/input playtesting is pending.

The prior remote main commit is `eed31ff6f36b13cad6a1d493d3cbb3bf85305e20`
(tree `a374b3cfe6c613ebce0e613753fbf02c322ddb9b`). Publish on top of that checkpoint,
verify main and the archive hash, then record the new commit/tree in
`/workspace/work/published-shadowrealm-checkpoint.json` if retained. Connector
publication may leave local HEAD at `90bfc84`; the index records the latest
published tree. Preserve staged work and compare worktree/index on resume.

Latest local milestone (2026-11-05): Gunderson's repeatable Redmond courier
loop now connects original offer generation, acceptance, pickup, delivery,
one-time nuyen/Karma reward, collected-package loss on defeat and another offer.
Read `docs/courier-job-loop.md` first. Launch `Shadowrealm.exe Shadowrun.bin 2`;
enter the Jump House (north side), approach its upper-left booth and ask for work.
J reviews the run. Ghoul/bodyguard offers still use the original random table,
but their objectives cannot be accepted yet. Decline and reapproach for a courier.

A controller-only check uses the ordinary map-2 spawn and development roster,
walks the booth -> Stoker's pickup -> Rat's Nest payment -> next-offer route at
tick 3356, with 20 -> 60 nuyen plus original Karma. No positions, stats, nuyen,
enemies or job outcomes are assigned during the run. No encounter spawned in
that trace. Original courier ambushes/robberies, other job types/Johnsons, shops,
district travel, new-game progression, full menus and disk saves remain pending.
Starting in Redmond is a development entry point, not a new-game travel feature.

`src/jobs.rs` and `data/jobs/gunderson-courier.json` retain typed Serde references
and native Rust state changes. The job record preserves uncleared trailing bytes;
pickup precedes delivery; byte Karma/long nuyen wrap; an uncollected run survives
defeat. A collected run is lost before the existing Little Chiba/DocWagon flow.
Pay prompts retain base/negotiated amounts. Matching courier service precedes
normal building entry, and the prison keeps its action-100 continuation. The
Jump House uses verified C4/map13 routing, saved camera/formation and bar return.
Its enabling does not authorize other bars or story doors. UI and J remain host
bridges. This continuation is included in this publication checkpoint; preserve
the published index tree described above. Final verification: 69 unit tests and all 87
ROM-backed integration tests pass. The job oracle matches 2,048 offers, 2,048
visits, 2,048 defeat prefixes, 20 service selections and 64 entry/save/route/
return-formation/spawn cases plus entry permission. Existing dialogue, exterior,
Space Needle and clinic/archives/rehire oracles pass. Native Linux and Windows
release builds pass; the 548,800-byte Windows ZIP is refreshed and verified for
CRC, all 22 payload hashes, README/executable identity and PE/DLL imports. The
ROM is excluded. Actual Windows GUI/input playtesting remains pending.

Previous local milestone (2026-11-05): automatic companion attack spells and Heal
now connect learned selection, equipment conversion, target acquisition, original
movement and unsafe-spell inventory-firearm fallback without M/G/F input. Read
`docs/automatic-companion-magic.md` first. A real map-5/default-roster test reaches
its first automatic impact at tick 157; by tick 328 both Ricky and Freya have
completed magic and an enemy finishes defeat, with the party alive. No inserted
enemies or actor health/stat/book/equipment edits are used in that test.

The missing D5 is prior-actor register residue: normal idle/action collision
leaves pixel X, walking restores fixed-point X. Preserve that unexpected ROM
capacity lookup and spell-to-spell reconversion; do not substitute weapon IDs.
`data/combat/companion-magic.json` is a small typed, ROM-checked reference catalog.
Special handlers/triggers/banks, first-slot incoming registers and outside-ROM
lookups still defer transactionally. Heal acquires/probes an NPC enemy before
redirecting to shared `$ffe190`; its initializer movement preserves collision
scratch and its post-cast positioning uses the updated party/self pointer.

The new oracle passes 1,024 prior-handler D5 cases, 3,031 complete automatic
companion decisions with 205 equipment changes, 154 successful casts (27 Heal
effects), and 41 exhausted-equipment prefixes stopped before melee. Earlier
spellbook/equipment/drain, Heal and companion-caster oracles also pass. Functional
tests now use per-actor cast/impact metrics so another companion cannot satisfy
a caster-specific assertion. Automatic status spells, other spell actions,
original menus and full magic AI remain incomplete. This milestone and the
preceding status-spell work are included in this publication checkpoint; preserve
the published index tree described above. Final verification: 69 unit tests and all 83
ROM-backed integration tests pass. Automatic decisions, spellbook/equipment/drain,
Heal, caster movement, casting/damage/effects, physical combat and status-spell
original-routine comparisons pass. Native Linux and Windows release builds pass.
The local 531,747-byte Windows ZIP is refreshed and verified for archive integrity,
all 22 payload hashes, README/executable identity and PE/DLL imports. Windows
GUI/input playtesting remains pending.

Previous local milestone (2026-11-05): Invisibility and Rockskin now cast through
original instantaneous initializers, drain, visible-actor status clocks and
original tile transforms. Read `docs/status-spells.md` first. Freya knows force-3
Invisibility and Ricky knows force-2 Rockskin: select with M, P cycles living
party targets, F casts, and G restores an inventory weapon while retaining the
status. These remain host shortcuts. A real map-5/default-roster test completes
Invisibility during a real encounter at tick 11, without inserted enemies or
health/stat/book edits. Automatic status-spell companion casting, other spell
actions, original menus and full magic AI remain incomplete.

A new small typed Serde catalog `data/combat/status-spells.json` records verified
references and status metadata; the timed-effect schema-3 catalog is unchanged.
The oracle passes 2,048 full status casts, 4,096 clock phases and 512 original
graphics/sprite comparisons across five maps. Preserve original quirks: instant
initializers/expiry do not release their allocated slot; clocks hold offscreen
and use signed-byte underflow; Rockskin reads the source flag on the represented
spell-damage path. Do not invent defender protection. See the status notes for
details and visual/DMA cadence limits. This continuation is included in this
publication checkpoint.
Verification: 69 unit tests and all 77 ROM-backed integration tests pass. Status,
casting, Heal, companion-caster and physical-combat original-routine comparisons
pass. Native Linux desktop and Windows builds pass. The local 524,046-byte Windows
ZIP is refreshed and verified for archive integrity, 22 payload hashes, README/
executable identity and PE/DLL imports. No new publication was performed.
The published base is `eed31ff6f36b13cad6a1d493d3cbb3bf85305e20`, tree
`a374b3cfe6c613ebce0e613753fbf02c322ddb9b`. Local HEAD may still be `90bfc84` while
the index holds that published tree: preserve it and compare worktree/index for
new work. Windows GUI/input testing remains pending.

Previous milestone (2026-11-04): equipped attack-spell companions now use the
original post-action positioning path, blocked/offscreen approach, cooldown
movement and immediate leader formation when targets disappear. Read
`docs/companion-casting-movement.md` first. Tab to Freya, M once for her learned
force-3 Mana Blast, then Tab back to Joshua: she can approach and cast without F.
A real map-5 test completes her spell at tick 116 with idle input after handoff,
without inserted enemies or actor/stat edits. Automatic gun-to-spell conversion
still defers at its incoming D5 context; Heal pursuit, other spell actions,
original menus and full magic AI remain partial.

Full original-routine comparisons pass 2,040 complete companion decisions with
270 successful casts plus eight exhausted-equipment prefixes stopped at the
melee entry. The common positioning helper is shared with firearms. Successful
casts now retain the original `$afe2` pose entry; failed rolls use `$afe0`.
The broad cases also recovered the `$106d0` EXT/DIVU-overflow drain boundary.
Typed casting/spellbook JSON stays at its existing schema and reference scope.
Verification: 68 unit tests and all 71 ROM-backed integration tests pass. The
new companion oracle and casting, Heal, spellbook/equipment/drain and physical-
combat regressions pass. Native Linux desktop and Windows builds pass. The
516,639-byte local Windows ZIP is refreshed and verified for archive integrity,
all 22 payload hashes, README/build identity and PE/DLL imports.
This snapshot is the user-authorized publication checkpoint for native attack
spells, Heal, mana attacks and companion caster movement. Verify `main` after
publication and record its commit/tree in the local publication record. Preserve
the staged work; connector publication can leave local HEAD at an older commit.
Windows GUI/input testing remains pending.

Previous local milestone (2026-11-04): Mana Zap and Mana Blast now connect original
roll/drain, willpower resistance with the party-defense D7 quirk, original effects
and the mana-specific NPC defeat animation/cleanup. Read `docs/mana-spells.md`
first. With the default roster, Tab to Freya, M once from her starting weapon
for learned force-3 Mana Blast, T to target and F to cast. A real map-5 encounter
test completes the effect at tick 90 without inserted enemies or actor/stat edits.
Mana Zap uses original Gator Shaman Joshua/Walking Bear books; desktop character
creation and those new roster choices remain unfinished.

Verification: 68 unit tests and all 65 ROM-backed integration tests pass. The
expanded attack-spell oracle passes 2,048 full casts, 4,096 damage/resistance
cases and 2,048 effect steps, alongside the Heal and physical-combat regressions.
Linux desktop/Windows release builds pass; the 515,782-byte Windows ZIP is
refreshed and verified for payload hashes, README/build identity and PE/DLL imports.

Casting JSON is now schema version 3 with five verified records, including
explicit completion metadata. All four classified attack spells and Heal are
supported in the represented contexts. Automatic companion spell equipment
conversion still defers at its incoming D5 context; original menus, other spells
and full magic AI remain unfinished. Preserve the existing staged work. This
milestone is included in the spellcasting publication checkpoint; Windows GUI/input testing is pending.

Previous local milestone (2026-11-04): Heal now connects self/party targeting,
original roll/drain and initializer movement, the sixteen-frame effect, health/
condition restoration and fallen-state clearing. Read `docs/healing.md` first.
Tab to Freya, M until Heal, P to cycle party targets and F to cast. Her original
book supplies force 4. M/G/P are host keyboard bridges, not original menus.
A real map-5/default-roster test heals an encounter injury (Joshua 9 -> 10) at
tick 296 without inserted enemies or actor/stat edits. Already equipped Heal
companions can prioritize a critically wounded party member during combat.
Automatic gun-to-spell equipment conversion and broader magic AI remain partial.

The typed casting JSON catalog is schema version 2: effect kind, initializer,
handler, frames/count/period and palette are validated against ROM tables. Rust
owns execution. Heal's initializer really calls `$b574`; do not remove it or
replace it with a fabricated immediate heal. Travel/special movement callbacks
defer with staged state/RNG. The new oracle covers 1,024 callbacks, 2,048
completions, 1,024 full casts and 1,024 effect steps. Final verification/package
results are recorded in the healing document. Windows GUI testing is pending.
All 68 unit tests and 60 ROM-backed integration tests pass, along with Heal,
casting, physical-combat, follower and encounter differential checks. Native
Linux desktop and Windows release builds pass. The local Windows ZIP is refreshed
and its payload hashes, README/executable identity and PE/DLL imports verified.
This milestone and the earlier Flame Dart/Bolt continuation are included in the
spellcasting publication checkpoint. Preserve the staged published tree described below.

Latest local continuation (2026-11-03): native Flame Dart/Bolt casting now connects
admission, roll/focus/totem modifiers, casting pose, actual drain, visible-target
effect timing, original sprites, resistance/damage and normal reactions/deaths.
Read `docs/spell-casting.md`, `src/casting.rs` and `data/combat/casting.json` first.
M cycles the selected magic runner's actual book entries/forces; G restores an
inventory weapon; T targets and F attacks. This is an explicit host equipment
bridge, not the original equipment menu or a recovered companion D5 caller.
Other spell actions/ongoing/area effects and general spell AI remain incomplete.
Do not claim full magic or complete game reconstruction.

A real map-5/default-roster test selects Ricky and a known Flame Bolt through
that bridge, then completes a real encounter spell effect at tick 192 without
actor/stat edits or inserted enemies. New differential checks compare complete
casts, poses, target-effect handlers and magic resistance/damage. See the casting
document. Verification: 68 unit tests and all 54 ROM-backed integration tests pass.
The casting, physical-combat, spellbook/equipment/drain and encounter original-
routine oracles pass; Linux and Windows builds pass. The Windows ZIP and PE/DLL
imports are verified. Windows GUI testing remains pending.
This continuation is included in the spellcasting publication checkpoint. Its base is
`950f3d2c2c721fb8ab94ba6f9f842497eea88833` (tree `4d13586304eeedf6e582752ae435fc2fe2da4680`).
The cloud shell Git proxy was unavailable during that publication, so local HEAD
remained `90bfc84` and the index holds the published tree. Preserve that staged
work; current unstaged/new files are the casting follow-up. Publication evidence
is also in `/workspace/work/published-shadowrealm-checkpoint.json` if retained.

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
Publication baseline before this checkpoint: `950f3d2c2c721fb8ab94ba6f9f842497eea88833`
(companion firearms, JSON spellbooks, equipment conversion and drain). This
checkpoint adds native attack spells, Heal and companion caster movement.
The GitHub connector can publish while local HEAD retains `90bfc84`; after
publication, the index records the published tree. Staged changes against that
older HEAD do not by themselves mean unpublished work. Consult remote `main`,
`/workspace/work/published-shadowrealm-checkpoint.json` when retained, and the
worktree/index diff before continuing. Do not reset away the staged work.
Environment configuration and private inputs are excluded.

Windows package: `releases/Shadowrealm-windows-x64-exploration.zip`.
SHA-256: `3667470bd428340990c52cd8cc9fe915bac27bcd0bab562797268dbafa3d4d53`.
The package includes EXE, required libunwind.dll, notices, file hashes and run
instructions. The ROM is excluded. The user must extract the ZIP and put their
`Shadowrun.bin` beside `Shadowrealm.exe`.

## What actually works

- Gunderson's bounded Redmond courier loop: real random offers and negotiating
  pay, acceptance, ten pickup/recipient locations, nuyen/Karma reward, repeat
  offers and collected-package loss before medical recovery. J reviews a run;
  map 2's Jump House entrance/booth/return is enabled. Other jobs and original
  courier ambushes remain unfinished.
- Automatic supported companion attack spells and Heal from original learned books,
  including bounded prior-actor equipment context, enemy-first Heal path, movement,
  drain and inventory-firearm fallback. Default-roster real encounter test uses
  idle input and completes both casters' magic plus an enemy defeat by tick 328.
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

Courier earnings now connect to the first Stoker objective and six city districts.
Extend this through verified equipment/services, original courier ambushes and
another original job objective. Original new-game setup, menu/save systems
and broader story progression remain incomplete. Keep JSON additions small and
source-backed, with Rust owning mechanics and differential checks covering
real routine behavior. Do not force supported job types by changing RNG.

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
| src/jobs.rs, data/jobs/gunderson-courier.json | Typed job/dialogue references, Gunderson offers, courier lifecycle and payment |
| src/job_export.rs, tools/validate_jobs.py, tests/jobs.rs | Original job comparisons and controller-only courier loop |
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
