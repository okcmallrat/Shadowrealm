# Gunderson courier job loop

The native game now connects Gunderson's offer dialogue, generated Redmond
courier routes, acceptance, pickup, delivery, payment/Karma and another offer.
All ten original Redmond courier locations are supported. These additions use
typed Serde/JSON references and native Rust state changes; no original CPU code
runs inside the desktop game.

Launch `Shadowrealm.exe Shadowrun.bin 2` from the extracted Windows folder.
This starts the development roster in Redmond. Enter the Jump House on the north
side and approach the upper-left booth. Ask for work, read the offer and confirm
a courier run. Leave, collect the package at the named location, then deliver it
to the recipient. J reviews the current briefing. The recipient pays immediately;
returning to Gunderson lets you ask for another offer.

The original eight-row offer table still chooses ghoul bounty, bodyguard or
courier jobs. The first two objectives are unavailable: their acceptance option
is disabled, with an availability message. Decline, step away from the booth and
approach again for another original random offer. A courier job is never forced
by changing the seed or replacing the other categories with invented work.

## Recovered state and behavior

The user's verified Ghidra working copy supplied targeted views at `$1e69c`,
`$1e9c0`, `$1eb8e`, `$1ef2c`, `$1f00a`, `$2148a`, `$20990` and the location
dispatcher. Generated listings remain in ignored `artifacts/courier-job-reference`.
Actual instructions and original-routine comparisons establish the behavior.

- `$1e69c` reads one of eight six-byte rows at `$f3ebc`, with the shared original
  RNG. It copies type and Karma, then derives negotiated pay from the row's base,
  multiplier and Joshua's byte `$8c`. The base quote is retained at `$fff0c0`.
  Review uses the original `$1060e0/$106171` pay prompts and fills both amounts.
- Gunderson's `$1e9c0` forces both districts to 2. It draws pickup and recipient
  locations from the ten entries at `$f3e80 + 20`, consumes the intervening
  twelve-way draw, and rerolls the recipient until it differs from the pickup.
  The duplicate rejection and otherwise unused random draw are preserved.
- The original job record is `$fffbec..$fffbf5`; the active word contains Johnson
  and type. Clearing a run sets only that word to `$ffff` and clears `$fffc1c`;
  the remaining record bytes are retained. Modal offer review has a host
  acceptance guard so world updates cannot complete an unaccepted offer.
- `$1ef2c` requires the matching pickup before a delivery can complete. Pickup
  sets flag bit 0 and package bit 9 of `$fffc22`. Delivery removes the package,
  adds the word reward to the long nuyen balance, adds byte Karma at `$8d` for
  all represented party records, and clears the run. Byte/long wrapping is
  retained, including Karma changes in inactive raw party slots.
- `$215ac` removes the package when cancelling, but leaves the old record until
  the forced next generation. The native modal flow follows that ordering.
  Courier completion does not invent a return-to-Johnson payment or general
  companion contract expiry. Other mission-specific contract callbacks remain
  unfinished.
- `$c6d0..$c69a` loses a collected courier package before medical recovery. The
  original `$f2510` notice precedes Little Chiba/DocWagon dialogue. An uncollected
  run survives ordinary defeat. All fallible recovery assets and notices are
  prepared before committing job loss, funds or roster changes.

The Jump House is map 2's C4 door into shared bar map 13. It uses original
entrance node `$110d1e`, descriptor routing, party spawn, saved exterior camera
and `$bc36` return formation. The enabled booth is its real `$9e` trigger and
opens Gunderson's `$105114` conversation. Other bars and story doors remain
guarded. A matching courier service runs before normal entry; it shows the
original pickup/payment passage and returns to exterior control. The prison
retains its introductory node and action-100 continuation. Wrong stops cannot
advance the run or award money.

## JSON and host UI

`data/jobs/gunderson-courier.json` records verified routine/table pointers,
dialogue node/choice references and the ten service passages. It embeds no ROM
text or graphics. Loading validates the typed schema, exact supported reference
set and every node/choice against ROM data. Unknown fields, changed links,
omitted choices and unsupported locations are rejected. Rust owns generation,
state transitions and payment, with transactional preparation.

The existing host dialogue panel formats the original briefing fragments and
pay prompts; it is not the original Pocket Secretary or portrait UI. J is a
read-only host shortcut, with pickup/delivery captions and current nuyen. A
host contact latch prevents the paused booth/service panel reopening before
the player moves away. It does not claim instruction-exact original UI timing.

## Validation

```sh
cargo build --release --locked
python tools/validate_jobs.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The oracle compares 2,048 complete Gunderson generations (including all three
offer categories), RNG, base quotes, all ten job bytes and flags; 2,048 courier
visits with 246 pickups and 431 payments; and 2,048 carried-package defeat
prefixes. It compares every party byte, balance and package flag, including
wrong district/stop, no active run, completed flags, overflow and inactive slots.
Only failure-notice UI is omitted in defeat prefixes, which stop before medical
recovery. Offer and visit checks use no instruction stubs.

Twenty original service selections execute the real `$1f5ca` mission dispatch
to the first dialogue, including the prison continuation, without gameplay
stubs. Jump House entry permission and 64 save/route/return-formation/spawn
cases also match original instructions without stubs.

The controller-only integration test uses map 2's ordinary spawn and development
roster `[0,3,10]`. It navigates actual collision, declines unsupported generated
offers, accepts a courier run, leaves the bar, collects at Stoker's, delivers at
the Rat's Nest and asks for another offer. At tick 3356, nuyen is 20 -> 60 and
the original Karma is awarded. No positions, stats, money, enemies or job results
are assigned during that run. No encounter spawned in this trace; it establishes
the courier travel/payment loop, not courier combat or ambush behavior.

Other functional cases check duplicate/wrong-order rewards, strict JSON and
generation rollback, and package loss followed by native medical recovery.

## Remaining scope

This is the first bounded repeatable job loop. Scripted courier ambushes and
robberies, other objectives/Johnsons, general shops, district travel, original
new-game/recruitment, full menus and disk saving remain unfinished. Starting in
Redmond is a development entry point; travel there from the current map-0 start
has not been implemented. Jobs, nuyen and Karma persist only within the session.
Linux/Windows builds do not establish Windows GUI/input playtesting.

Final verification: 69 unit tests and all 87 ROM-backed integration tests pass.
Existing dialogue, exterior routing, Space Needle and clinic/archives/rehire
original-routine checks pass. Native Linux and Windows release builds pass.
The 548,800-byte local Windows ZIP is refreshed and verified for CRC, all 22
payload hashes, README/executable identity and x64 PE/DLL imports; it excludes
the ROM and Ghidra database. This continuation is included in the courier-loop publication checkpoint.
Windows GUI/input playtesting is pending.
