# Stoker's encounter and motel rest

Stoker's Coffin Motel now rents its original 20-nuyen room, both before and after
retrieving Michael's belongings. The shared hotel flow performs one original
partial rest, permits per-runner Karma spending, and returns through checkout.
Matching courier service still takes priority. Room cost is not a full-heal fee.

After the first objective, the greeting's MORE executes original action 233.
The original shared RNG rolls 100: values below 80 reach the manager; otherwise
it attempts the actual close-placement Halloweener group. Allocation failure or
no valid placement reaches the normal manager. Successful allocation shows the
original attack passage, then resumes the street fight. The group contains one
firearm user and two unarmed humans. There are no forced enemy/roll outcomes.
T selects a target; hold F to fight with an equipped supported gun or spell.

Ordinary unarmed human NPCs now pursue a target, test the original close range,
attack on the source schedule, animate, and apply defense/resistance and damage.
Normal fists reduce condition; the recovered cyberware paths can damage health.
Zero condition incapacitates a runner without replacing their health with zero.
Normal all-party defeat then admits medical recovery for all three Joshua
archetypes, including condition-only knockouts. Fees, minimum restoration,
companion archives and insured/uninsured passages use the existing source core.

## Connected play

The controller route constructs the explicit three-runner development party,
earns six courier jobs and retrieves Michael's belongings. It repeatedly visits
Stoker using ordinary movement/menu choices and the shared RNG. Five manager
visits rent rooms; visit 5 (zero based) rolls the real gang encounter. The fight
finishes at tick 15935: six NPC punches, eleven NPC shots and fourteen player
shots; companions also act through existing automatic combat. Joshua ends the
fight at health 10 / condition 1. The party survives.

The next manager visit rents another 20-nuyen room and restores condition. Joshua
raises Body 4 -> 5 for five Karma, leaving three. At tick 16155 the balance is
410 nuyen. A real disk checkpoint reload retains money, Body, remaining Karma,
six jobs and story/DocWagon progress. The route assigns no actor/stat/inventory,
money, RNG, position, map, enemies or job outcome after ordinary construction.
This is a development roster, not a claim of earned companion recruitment.

A fresh solo Samurai also rents the room before the first objective, spends the
original 20 nuyen, receives an authentic insufficient-Karma refusal and saves
with zero nuyen. Other functional cases exercise a live unarmed attacker,
condition-only incapacitation/recovery for each archetype, strict JSON and staged
attack failure. Earlier solo/new-game, Ares, taxi, hotel and save routes still pass.

## Recovered references

| Reference | Behavior |
| --- | --- |
| `$249c4`, action 233 | Roll / spawn result / normal or attack dialogue |
| `$1d6e2a` | Gunner `$1d66a4`, two unarmed `$1d665a`, zero terminator |
| `$1e1fc/$1e216/$1e40c` | Close spawn points, slot/graphics allocation, actor initialization |
| `$f4ab4/$f4acc` | Manager / gang attack passages |
| `$1f73c`, `$f4ad4` | Original 20-nuyen motel fee and offer |
| `$21458`, action 6 | Signed-long payment and original success/refusal passages |
| `$141a0/$14c7c/$15c58` | Ordinary close-attack decision and active/not-fallen range gate |
| `$b1dc/$c0a2` | Attack cooldown, aiming, pose, damage, reaction and hit FX |
| `$55c76` | Unarmed attack / armor / defense / resistance / health or condition |
| `$15a8c`, `$ac4e` | Shared hit-FX allocation and lifetime |
| `$a94a/$a9cc -> $aa4c/$b330` | Incapacitation with preserved health and party-defeat admission |
| `$c726..$c7c0` | Normal medical core, including condition-only defeat and all Joshua classes |

`src/unarmed.rs` and typed `data/combat/unarmed.json` provide the bounded rules;
`src/stoker.rs` / `data/encounters/stoker.json` verify the visit, group and fee.
The existing dialogue catalog adds only actions 6/233 and two verified nodes.
`encounters::Runtime::stoker` uses the original close-placement and human group
initialization. Regular street group admission remains limited to its earlier
firearm subset; it does not silently enable other scripted/creature groups.
`combat_runtime` connects the human NPC action, existing movement and hit FX.
`damage::incapacitate` handles the source zero-condition fall; clinic preparation
now accepts that defeat and all three Joshua templates.

Preserve the source quirks: defender Melee receives the high byte left in D7 by
its cyberware flags; unarmed armor uses impact protection; attacking armor-focus
checks use the attacker's record; cyberware priorities, byte wrapping, unusual
NPC reward timing and party mode immunity are retained. Fresh hit-FX slots may
have a zero animation period: original `$5fdc` subtracts zero and `$600e` clamps
normally. Native live FX now executes that valid path instead of rejecting it.

Dice, actor/FX changes, reactions, world alert and RNG are staged for a strike.
The visit stages actors, encounter/world state, RNG, passage and sprite assets
before committing. Rental preflights rest/HUD assets before charging. These
transactions do not partially grant a room or consume a failed attack's RNG.

## Validation and limits

The supplied Ghidra working copy provided the actual visit, spawn, close attack,
damage and medical references. Private exports, fixtures, logs and rendered
captures remain in `/workspace/work/stoker-encounter`, outside Git.

- 512 complete original action-233 visits match both outcomes, all actor bytes,
  close placement, pool/graphics exhaustion, encounter count, alert and RNG;
  no instruction stubs.
- 2,048 original unarmed damage cases match actors, armor/foci/cyberware,
  defense/resistance, health/condition, mode immunity, reward and RNG; only HUD
  tile transfer is omitted.
- 256 full original close attacks match poses, hit FX/full pools, reaction,
  defeat, alert and RNG; only audio/HUD transfer is omitted.
- 256 original incapacitations preserve health and match flags/pose/defeat;
  only audio is omitted.
- 256 original zero-period hit-FX animation steps match.
- Medical coverage expands to 1,792 original cores, including 768 condition-only
  cases across all three archetypes. Existing archive, arrival and rehire checks
  also pass.

The existing full combat, encounter, progression, rest/Karma, clinic and save/load
original-routine comparisons pass. Player punches, companion unarmed attacks,
melee weapons, special creature/unarmed poses, general scripts, further story,
other services, wilderness and audio remain unfinished. Motel rooms use the
existing host panels and retained street context; original room art/fades and
full service reload cadence are not reconstructed. The connected fight uses the
supported party; broader encounter/party combinations are not claimed complete.
Windows GUI/input/filesystem playtesting remains pending.


All 76 unit/pure and 118 ROM-backed tests pass (194 total, desktop feature
enabled). Native Linux desktop and Windows release builds pass. A Linux desktop
launch was attempted and failed to create a window in this environment; actual
Windows GUI/input/filesystem playtesting remains pending. Rendered gang notice,
battle aftermath, room/upgrade and resumed HUD panels were inspected.

The verified Windows ZIP is 686,473 bytes. CRCs, all 22 payload hashes,
exact membership, source README/EXE identity and x64 EXE/DLL headers/imports pass.
SHA-256: `89d2c04119b446479c401fc11b2b2dd35d80b266ea86388216c210f6621418c9`.
ROM/GZF, saves and generated frames are excluded. The user requested publication
after this milestone; this checkpoint includes the preceding completed local
progression, Ares, saves, hotel/Karma and new-game work.
