# Bounded street encounters

The native game now connects ordinary firearm encounter groups from the supplied
ROM to the existing NPC combat loop. These are original district selections and
compact actor records; no training arena or replacement enemies are added.

## Recovered rules

`$1e166` checks pause/player/script/combat/count guards before the frame phase
`tick & 0x3ff == 10`. A passing phase consumes one shared `$d44` random word even
when the descriptor chance is zero; it compares that word unsigned against
map descriptor offset `52`. `$59986` then adds two independent samples of five,
uses the map-to-district byte table at `3312`, and reads encounter type/group
pointers at `1d70e4`/`1d7174`. Scripted outcomes are recorded as deferred, without
rerolling or substituting a firearm group. The runtime baseline is normal mode
with no mission/script context; the general event callback VM remains absent.

`$1e302` generates ten candidate positions in order on two lines outside the
camera center. It chooses the side using descriptor extents, quantizes positions
to cell centers, and reads one collision raster byte per candidate. This is not
a footprint or actor-overlap test. `$1e40c`'s close placement is implemented for
comparison, but scripted encounters using it are not enabled. Reads beyond the
modeled 4,096-byte raster defer the attempt, preserving the existing boundary.
506 in-range placement fixtures match; six generated edge fixtures are withheld.

`$1e228/$1e25a` choose the first free actor slot 3..9 and graphics allocation
0..17. `$44bc` copies the original default actor, deserializes its compact record,
sets graphics/coordinates, consumes the decision-timer RNG word, and applies the
optional byte overrides. The map-55 timer exception is preserved. Group actors
cycle through available placement points, receive combat behavior byte `4d=4`,
flags `10` bit 5 and `9e` bit 6, and trigger the original ordinary combat alert.
A full pool stops the group with the preceding allocations intact. The native
allocator derives graphics occupancy from the represented active NPCs; unrelated
script graphics allocations are not modeled.

The runtime stages each group and enables it only when every allocated member
uses the supported ordinary firearm path. Groups needing melee, magic, creatures
or special/script handlers are deferred whole, with no partial actors or timer
RNG changes committed. The gate/selection draws remain consumed. This is an
explicit reconstruction boundary, not the original response to those encounters.

Newly spawned actors can be outside the visible play area. The normal-mode
`$1398c..$139ba` engaged branch follows the controlled party leader through the
shared formation/route steering. Visible actors then use the verified combat
range, firing and physical-damage logic. Completed encounter deaths decrement
the count once at the actor's dispatch position; the last death clears combat
links. Offscreen nonparty corpses release their actor/graphics slots. Visible
corpses remain until the camera leaves them. Map reload clears encounter tracking.
Mission callbacks, loot/interactions, ambient patrol and full game-over remain
incomplete; this is a bounded street-firefight milestone.

## Verification and replay

```sh
cargo build --release --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test encounters -- --ignored
python tools/validate_encounters.py /path/to/Shadowrun.bin
cargo run --release --locked -- encounter-data /path/to/Shadowrun.bin artifacts/street-encounters
```

The oracle executes the original instructions with no instruction stubs:
1,024 encounter gates, 1,792 district selections across all 56 map indices,
506 placement cases, 128 complete firearm-group allocations with different
occupied slots and RNG seeds, and 128 offscreen pursuit decisions. Full actor
records, RNG, allocated counts and combat alerts are checked where applicable.
Existing combat and fixed-room NPC comparisons remain separate regression checks.
The GZF working-copy views are private under `artifacts/encounter-reference`.

The exported replay loads normal map 5 with the existing solo development roster
and sends idle input. It uses the real encounter gate and shared initial seed;
there is no test actor insertion or positioning. With this deterministic host
seed, two firearm NPCs spawn at tick 10, fire twice and reduce Joshua's health to
9 at tick 215. `street-trace.csv` and spawn/impact PPMs capture that run. This
reproducibility reflects the host's fixed initial RNG seed, not original hardware
boot timing. Changing roster, movement or firing can change the shared RNG trace.

To try the Windows build, run `Shadowrealm.exe Shadowrun.bin 5 --solo` from its
extracted folder. T selects a target and F fires; R resets the session. The build
is cross-compiled and checked for package integrity; Windows GUI playtesting is
still outstanding.
