# Gates Undersound rest and Karma

The later [Stoker encounter and rest milestone](stoker-ambush-and-rest.md)
connects the motel room, unarmed NPC attacks and expanded normal recovery.
The notes below retain the earlier milestone’s scope and validation record.

Downtown's Gates Undersound Hotel (map 0, location 3 / C3) now offers its
original greeting and 75-nuyen room. Pay, acknowledge the proprietor, rest,
then choose a runner and spend that runner's Karma. Check out to resume the
street. Hotel menus freeze the world and block street save/load shortcuts.
After checkout, F5 preserves the improved stats and remaining Karma.

The connected path starts in the default Redmond development roster with 20
nuyen, completes six real courier jobs, pays Michael's back rent and retrieves
his belongings. A Downtown taxi leaves 510 nuyen; the room leaves 435. Joshua
raises Body 4 -> 5 for five Karma; Ricky raises Firearms 2 -> 3 for six of his
own Karma. A canceled upgrade spends nothing. Checkout cancellation stays in
the room. The test writes a disk checkpoint, drops Game and constructs a new
Game from the file: both upgrades, remaining Karma, 435 nuyen and six completed
jobs survive. The checkpoint is at tick 15380. No actor/stat/equipment, money,
position, map, RNG, enemy or job outcome is assigned after ordinary construction.
No encounter spawned along this route. This remains an explicit development
roster, not the original new-game/recruitment flow.

## Recovered behavior

The supplied Ghidra working copy and ROM identify these references:

| Reference | Behavior |
| --- | --- |
| `$1f65a` | Hotel dispatcher, Downtown location 3, fee 75, `$f4910` root |
| `$f4910/$f4924/$f493c` | Original introduction, greeting and room offer |
| `$21458`, action 6 | SUB.L fee/funds decision, `$f5ab8` success / `$f5af8` refusal |
| `$cf38`, `$ef08` | Paid hotel flow and three-slot rest |
| `$3674/$37ec` | Health/condition adjustment and signed byte clamps |
| `$d82a` | Karma quotes, caps, Sorcery restriction, specialization rules |
| `$d33a..$d474` | Karma check, confirmation and two actor-byte writes |
| `$d53c` | Upgrade confirmation starts on Yes |
| `$c22a6` | Five races' six attribute maxima |
| `$c1a2a/$c1a4a` | Original attribute/skill names |

Rest restores ten condition using the original helper. Health >= 5 adds two;
health 3..4 adds one; health <= 2 adds no health and shows the original medical
attention notice. Eligible health branches clear the original wounded bit.
Inactive slots are skipped. Rest does not refill ammunition or grant Karma.
Byte overflow, condition spill into health and the original flag behavior are
preserved, including unusual inputs in the differential fixtures.

The six trainable attributes cost their next value and obey racial caps.
Essence and Magic cannot be raised with Karma. Ordinary skills cost twice the
next value and normally cap at 12. Pistols/SMGs/Shotguns cost the ceiling of 1.5
times their next value; zero specialization starts from base Firearms + 1.
The original cap check precedes that substitution. Sorcery is limited by the
source class-byte gate. Costs, stat increments and Karma use the original byte
operations; unsupported races are rejected before indexing the cap table.
Only the chosen stat and that runner's Karma change. Existing combat/shop code
reads those actor fields directly.

`src/advancement.rs` owns the rules and staged actor decisions;
`src/hotel_game.rs` connects them to Game. The small typed Serde/JSON catalog
`data/hotels/gates.json` verifies the exact passages, action, labels and cap
references. Unknown fields, changed scope and mismatched ROM records fail.
Health/condition helpers are shared with post-cast drain, preserving those rules.
Payment preflights rest/HUD assets. Rest and advancement prepare replacement
actors, scene and HUD before committing. Failed upgrade assets leave the
confirmation, actor bytes, wallet and rendered frame intact.

## Host presentation and limits

The hotel room, runner selector, category lists, quotes and checkout confirmation
are host panels over the recovered rules. Original room graphics, stat-screen
layout, palette/fade/input cadence and audio are not reproduced. World control
selection stays unchanged while training another runner. The original generic
payment passages are retained, even though the hotel offer has its own wording.
There is one rest per paid admission; returning to the room menu does not heal
again. Check out and enter/pay again for another stay.

Only Gates is connected. Other hotels, Stoker's post-objective random thug
encounter/action 233, further story/new-game, wider services/shops/missions,
cyberspace, wilderness and audio remain unfinished. Street stats do not grant
free training. Saving inside the room is guarded. Actual Windows GUI/input and
filesystem playtesting remains pending.

## Validation

`tools/validate_advancement.py` executes the supplied original M68000 routines:

- Real Gates location dispatch, opening and fee, with no gameplay stubs.
- 13 original payment/funds/signed-overflow boundary cases.
- 5,120 original quotes and Karma decisions/actor writes, all 20 cells, all
  current byte values, five races, class gates, cost boundaries and cancellation.
- 1,024 complete three-runner rests with active/inactive/wounded flags and
  health/condition byte boundaries.

Only font/tile display, waiting and confirmation input are omitted. Karma
comparisons stop at the menu redraw/loop boundary after all actor writes.
Generated Ghidra references, oracle output, logs and rendered panels stay in
`/workspace/work/karma`, outside Git. Functional tests cover the earned route,
physical-contact entry with a switched leader, insufficient funds, cancellations,
world freezing, held confirmation keys, failed assets and disk persistence.
Rendered room quote, rest, Joshua and companion upgrade panels were inspected.


All 74 unit/pure and 108 ROM-backed integration tests pass. Existing physical
contact/travel (including 4,704 gates after adding Gates C3), casting, healing and
save/load original-routine comparisons pass. Native Linux desktop and Windows
release builds pass. The verified Windows ZIP is 646,852 bytes. Its CRCs, all 22
payload hashes, exact membership, source README/EXE identity and x64 EXE/DLL
headers/imports were checked before replacing the local release ZIP. SHA-256:
`37726aa3501eceaf22008214d0053ecebad298fa25350af7af5ec98fe30c9853`.
No ROM/GZF, generated frames or saves are packaged. Source/package changes
remain local; no publication was requested or performed.
