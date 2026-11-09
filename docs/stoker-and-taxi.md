# First objective and city taxi travel

The later [Stoker encounter and rest milestone](stoker-ambush-and-rest.md)
connects the motel room, unarmed NPC attacks and expanded normal recovery.
The notes below retain the earlier milestone’s scope and validation record.

The desktop now starts the development roster in Redmond (map 2). Courier jobs,
Michael's belongings at Stoker's and six city taxi destinations form a connected
progression route. This is native Rust gameplay, with typed Serde/JSON references
in `data/dialogue/stoker-taxi.json`; the desktop executes no emulated CPU code.

Enter the Jump House on the north side and ask Gunderson for courier work.
Complete runs until you have at least 250 nuyen. Visit Stoker's Coffin Motel,
ask for Michael's belongings and pay the back rent. Continue through the letter,
cyberdeck, wristband, holopix and hidden-credstick passages. C on a city street
opens the destination menu; Up/Down and Enter/Z choose and confirm a trip.
J still reviews the current run. [F5/F9 checkpoints](native-save-load.md) now
preserve this progress on disk; R resets the session and keeps those save files.

## Recovered objective

The user's verified Ghidra working copy supplied targeted views of `$53a78`,
`$53ab6`, `$213ca`, `$213e2`, `$213fa`, `$219b6` and `$219c8`. The actual ROM
instructions and independent original-routine comparisons establish the rules.
Private exports are under `/workspace/work/travel/stoker-ghidra` and are not
committed. The supplied ROM/GZF remain unchanged.

Stoker's is map 2's C2 service. The native courier service retains priority, so
a matching package pickup/delivery runs before the normal motel conversation.
The first story dispatcher selects `$f4afc` only while bit 0 of `$fffc1e` is clear.
Asking for Michael's belongings sets lead bit 1 in `$fffc30` and opens `$f4e8c`.
The price is 250 nuyen. Refusing or failing the signed-long funds check records
bit 0 of `$fffc27` and gives the original Jump House guidance (`$f5ab0`).

Action 5 (`$213fa`) deducts 250, sets story bit 0, adds the hidden 500 nuyen,
adds one byte Karma to each represented party record, sets Joshua's cyberdeck
ownership flag, clears lead bits 0/1 and sets 2/3/4. Long/byte wrapping and the
original early reward timing are preserved: the hidden money is applied before
its final explanatory passage. The remaining nodes read exact ROM text; actions
34 and 33 switch the holopix text context. Their portrait changes are outside the
existing host dialogue panel. The cyberdeck ownership flag does not implement
cyberspace, deck equipment or the Pocket Secretary.

This same story bit supplies the existing DocWagon recovery branch, so acquiring
the wristband now enables that previously recovered behavior through gameplay.
All fallible follow-up dialogue is prepared before committing the purchase.
Subsequent visits select the original post-objective root; the stash cannot be
repurchased. Motel rest and that root's random encounter action 233 remain
unavailable rather than bypassing the encounter roll.

## Recovered taxi

Ghidra/ROM views at `$134d2`, `$20714`, `$2076a`, `$208a8`, `$208fe` and `$136de`
establish the service. Private exports are under `/workspace/work/travel/ghidra`.
`$d4456/$d445d` supply initial selection and original destination order;
30-byte records at `$d4464` supply names and fares. Downtown/Penumbra/Renraku
cost 20, Puyallup/Redmond 40, Council Island 75, and Salish 100 nuyen.

The original traffic refusal (`$fe620`) blocks every trip until the first story
bit is set. Same-district requests use `$fe628`. Ordinary trips use the original
quote fragments/options, then `$208a8` deducts the fare if the signed-long balance
is sufficient. Insufficient funds produces `$fe610` and still permits a free
trip. Joshua's armband flag `$8f` bit 5 gives the original free-ride notice without
the paid confirmation. Visa checks and the consumed carried flag are represented
in the pure routine translation; native wilderness travel remains guarded.

The six city districts are maps 0..5. `$136de` clears interior state and selects
the destination; descriptor camera and party positions supply the arrival through
`$2b34`. Native travel uses the existing fade/reload/warm-up pipeline, clears
map-local NPCs/projectiles/targets, and preserves party resources, inventory,
spellbooks, contract archives, wallet, jobs and story state. Arrival assets and
current equipment are validated before charging. Unsupported Salish travel cannot
deduct money or consume a visa, including with the free-ride armband.

C and the compact destination list are host UI. A destination selection opens
the recovered fare confirmation directly, combining the original initial greeting
and request-for-price step. The original city-map highlight, portrait UI, exact
UI/input/fade cadence and taxi audio are not reconstructed. Calls are bounded to
city exteriors. The default-map change only selects a playable development start;
it is not original new-game creation or recruitment.

## Verification

```sh
cargo build --release --locked
python tools/validate_progression.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The new oracle compares 2,048 complete original belongings purchases/refusals,
including all party bytes, Karma, balance and lead/story/inquiry flags; 2,048
taxi story/same-district gates; 2,048 complete payment/free-ride/visa decisions;
and 1,755 armband/visa decisions. No gameplay instruction stubs are used.
Thirty city routes compare map/interior flags and descriptor party/camera arrivals;
only the leading taxi fade/sound call is omitted by starting at `$136e4`.

The connected controller test starts with map 2's normal spawn, original 20 nuyen
and the development roster `[0,3,10]`. Six courier jobs bring the balance to 280.
Paying for Michael's belongings and collecting the hidden credstick yields 530;
the Downtown taxi arrives with 510 at tick 14746. No party/stat/book/equipment,
position, map, RNG, money, enemy or job-outcome assignments occur during that run.
No street encounter spawned in this trace; it does not establish combat along
the route. Additional explicit fixtures verify no-funds refusal, repeat-claim
prevention, failed asset rollback, taxi cancellation/same-district/visa gates,
paid/free/armband trips and party/job preservation. Rendered panels were inspected.

Remaining scope includes motel rest, broader story objectives, shops/equipment
services, original new-game/menus, wilderness travel, courier ambushes, additional
job types, original save UI and audio. Native checkpoints were added afterward.
Windows GUI/input playtesting remains pending.

Final verification: 70 unit/pure tests and all 93 ROM-backed integration tests
pass. Existing courier, exterior, Space Needle and clinic/archive/rehire oracles
pass. Native Linux desktop and Windows release builds pass. The refreshed Windows
ZIP is 566,213 bytes; CRCs, all 22 payload hashes, README/executable identity,
x64 PE format and EXE/DLL imports are verified. Its SHA-256 is
`2360a9882546acd4738b28195fc49cb9e3e18c595726656f43df71046c9a1a3e`.
The ROM and Ghidra database are excluded. This continuation remains local and
has not been published.
