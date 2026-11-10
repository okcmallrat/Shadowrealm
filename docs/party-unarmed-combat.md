# Player and companion unarmed combat

Ordinary human fists now work for the selected player and uncontrolled companions.
Open **I** between actions and select the equipped firearm again to put it away.
Close inventory, select a target with **T**, walk close and hold **F**. Selecting
the owned firearm again restores it using the existing ammunition conversion.
Normal fists reduce condition; original cyberware can make the attack damage
health. Joshua Samurai and Two Fists retain their recovered hand-razor behavior.
Fists consume no ammunition, and stored ammunition survives native checkpoints.

A swing always takes the original mode-1 cooldown and pose. Missing, inactive,
fallen or distant targets do not cause damage, alert or RNG consumption. A distant
target still controls the swing's heading. With no target, the current desired
heading is retained. The player does not automatically walk into range. The
projectile obstruction diagnostic is omitted for fists, which use the original
close-target query. Target brackets remain a host selection aid.

Companions retain the original decision scheduler, target acquisition and automatic
spell selection. Ordinary fists enter close pursuit, stop at range, wait through
cooldown and strike. Firearm users with reserves reload; exhausted ammunition and
reserves convert equipment to fists and enter the same pursuit path. The existing
NPC unarmed path shares that close action; ordinary firearm NPCs also complete
their recovered exhausted-gun continuation. Special banks/effects remain guarded.

## Original references and implementation

The supplied verified Ghidra working copy provided targeted exports, with actual
ROM memory hash checked by the exporter. Private exports/logs/captures are under
`/workspace/work/party-unarmed`; the supplied ROM/GZF are preserved outside Git.
The desktop runtime executes Rust rules and reads assets; it does not run emulated
instructions or invoke Ghidra.

| Reference | Behavior |
| --- | --- |
| `$ad1a..$ae16` | Controlled cooldown queue, held attack, unarmed dispatch |
| `$b1dc..$b2f4`, `$c0a2` | Full swing, optional target heading, pose, hit reaction/FX |
| `$15c84/$15c58` | Stored target pointer, active/not-fallen query, range below `$200` |
| `$55c76` | Impact armor, defense/resistance, condition/health, source byte quirks |
| `$138b8/$139d8/$13ffe` | Companion schedule and target/equipment dispatch |
| `$14146..$141a0` | Finite reload or equipment clearing, then close decision |
| `$141a0/$13d0c/$13a40/$145a2` | Stop, attack/wait, or pursue target through navigation |
| `$12168..$12292` | Existing equipped-item toggle and ammunition percentage conversion |

The typed unarmed catalog adds validated controlled-entry and target-query
references. `unarmed::swing` stages actors, dice, world, reactions and hit FX and
preflights the resulting pose before committing. `strike` retains the close-only
API used by acquired AI targets. `combat_runtime` connects controlled attacks,
companion fists and exhausted-gun continuations; the inventory panel explains
its existing toggle. Player swing/damaging-hit and companion swing counters are
session diagnostics, not save data or invented game mechanics.

## Validation

- All **76 unit/pure + 123 ROM-backed tests pass: 199 total**, desktop enabled.
- **768 complete original controlled entries** match all actor bytes, queued/held
  input, cooldown, every party slot/template, targetless/distant/inactive swings,
  damage/rewards, hit FX/full pools, alert and RNG. Idle-walking entry behavior is
  outside these fixtures; only audio and HUD tile transfer are omitted.
- **768 complete original companion decisions** match both slots, scheduler,
  target acquisition, pursuit, attacks, cooldown, exhausted guns, finite reloads,
  actor/FX state and RNG. Only audio/HUD are omitted.
- The **2,048 unarmed damage comparisons** now include player/companion sources,
  preserving impact armor, foci/cyberware, high-byte Melee quirks, defense,
  resistance, condition/health, rewards and RNG. The earlier **256 full attacks,
  256 incapacitations and 256 zero-period FX clocks** also match.
- Existing combat and automatic-magic oracles pass, including 3,033 complete
  automatic decisions plus 39 deliberately guarded equipment prefixes. No
  broader magic/melee support is inferred from a passing prefix.
- Live tests cover all solo classes toggling their original firearm, repeated
  swings, ammunition preservation and checkpoint restoration; actual controlled
  and Two Fists companion damage; exhausted Stark firearm pursuit; and failed
  controlled-pose assets leaving actors, FX, world, rewards and RNG unchanged.
  Low-defense explicit fixtures isolate nonlethal damage for Decker/Shaman;
  these tests do not claim those archetypes win arbitrary fights untrained.

### Connected earning, combat, rest, training and reload

The controller route constructs the existing Joshua/Ricky/Freya development roster,
completes six courier jobs and retrieves Michael's belongings. Post-objective
visits use the ordinary shared RNG; visit 5 (zero based) rolls the actual Stoker
group. Joshua puts away his firearm through the inventory panel and makes **12
swings, two damaging hits, zero player shots**. Companions use their normal supported
combat. Enemies make eight punches and seven gunshots. The fight ends at tick
15750 with Joshua/Ricky health 10 / condition 10 and Freya health 10 / condition 9.

The party rents another 20-nuyen room, trains Joshua Body 4 -> 5 for five Karma,
and checks out. A real disk checkpoint at tick **15978** reloads **410 nuyen,
three Karma, Body 5, six jobs, story/DocWagon, fists and 100% stored ammunition**.
No actor/stat/inventory, money, RNG, enemy, position, map or job outcome is assigned
after construction. It is a development roster; companion recruitment is not
claimed earned. The original firearm version of the route also still passes.
Swing, aftermath, hotel/Karma and resumed HUD frames were inspected.

## Package and remaining limits

Native Linux desktop and Windows x64 release builds pass. A Linux GUI launch was
attempted and failed to create a window in this environment. Actual Windows
GUI/input/filesystem playtesting remains pending.

The refreshed Windows ZIP is **689,598 bytes**, SHA-256
`8e581601385a985aaa2d237d1ebdb583ca0e232cf2f995f71809aec1ae55f165`. CRCs,
all 22 payload hashes, exact 23-member archive contents, source EXE/DLL/README
identity, x64 PE headers and imports are verified. ROM/GZF, saves and generated
frames are excluded.

This is an ordinary human fists milestone. Weapon melee, throwables, special
creature banks and ongoing-effect combinations remain incomplete. Further story,
cyberspace, broader recruitment/services/jobs, wilderness, original full menus,
cinematics, room art/fades and audio remain unfinished.
