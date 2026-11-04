# Native Mana Zap and Mana Blast

The later [companion movement milestone](companion-casting-movement.md) connects
attack casters' pursuit/cooldown positioning and corrects the cast-result pose
entry and drain-division overflow boundary. Older movement limits below describe
the preceding checkpoint.

Mana Zap (equipped ID 8) and Mana Blast (ID 9) now connect original casting,
drain, timed effects, resistance, health damage, reactions and defeat animation
to the native game. Together with Flame Dart/Bolt, this covers all four attack
IDs recognized by the recovered companion spellbook classifier. Heal remains
supported. Area spells, ongoing modifiers, other spell actions, original menus
and full magic AI remain unfinished.

## Playing this checkpoint

With the default Joshua/Ricky/Freya roster, Tab to Freya and press M once from
her starting weapon to equip Mana Blast. Her original spellbook supplies force 3.
T selects an available target and F casts; G restores an inventory weapon.
M/G remain host equipment shortcuts while the original menus are pending.
For Heal, M cycles to Heal, P cycles party targets and F casts.

The normal map-5/default-roster integration test completes Mana Blast against
a real street encounter at tick 90 with the host's initial deterministic seed.
It uses the equipment and input paths above, without inserting enemies or
editing actor health, stats or spellbooks. Other input changes the shared RNG.

Mana Zap is learned by the original Gator Shaman Joshua and Walking Bear.
Its functional fixture uses Joshua template 2 and his learned force 3; the
desktop's default Joshua remains the Samurai template. This milestone does not
add character creation or Walking Bear recruitment to expose new roster choices.
The isolated fixtures supply test opponents and wounds explicitly.

An already equipped supported companion can cast through the existing bounded
combat path. Automatic gun-to-spell conversion still waits for recovery of the
incoming D5 capacity context. General pursuit/cooldown movement, other spell
actions and mission-aware AI remain partial.

## Original behavior

`$1033e/$15830` retain the existing roll, focus/totem modifiers, allocation and
post-success drain. Blocked admission consumes no cast RNG or drain. Equipment
changes remain blocked while an effect/projectile is live through the host
shortcut. Unsupported contexts retain the previous explicit boundaries.

| Spell | Handler | Frames | Frame table | Reset timer | Palette |
| --- | --- | --- | --- | --- | --- |
| Mana Zap | `$508a` | 3 | `$5142` | 1 | `$4000` |
| Mana Blast | `$4fe2` | 6 | `$512a` | 4 | `$4000` |

Both start with timer 5, hold while offscreen and release the shared graphics
slot on completion. Mana Blast also clears the target's `$de` marker. Original
dynamic sprite records are read from the supplied ROM at render time.

`$55b78/$55c08` select the resistance attribute using the signed spell-record
byte at offset `$27`. Mana spells normally use willpower (`$7c`), while the flame
spells use body (`$77`). Party defense first runs `$1047e`, which adds active
party magicians' original rounded magic contribution. That routine saves D5/A0
but leaves D7 overwritten by `$55f02`'s final focus search. The subsequent
attribute read therefore uses the final focus index or -1 in that context. In
this ROM all those resulting bytes select body. Rust preserves this original
register side effect instead of applying an invented universal willpower rule.

Mana Blast saves pre-impact health before damage and reacts only if it changes.
Mana Zap, like the existing flame handlers, compares health against the restored
graphics-allocation value in D0. This can produce a hurt pose even when resisted
damage leaves health unchanged. Both behaviors are retained and tested.

For a defeated ordinary NPC, mana effects call `$b0e4`, selecting handler `$b10c`
and animation state `$e4`. Flame effects retain `$b0ac/$b0d4/$e0`. Party actors
and bank-12 actors follow the existing generic reaction. The native actor phase
now completes `$b10c` and sets the original finished/fallen flag, allowing the
existing encounter cleanup path to run.

## JSON and verification

Schema version 3 of `data/combat/casting.json` adds the two verified records and
explicit marker-clearing, reaction-comparison and defeat-routine metadata.
Serde rejects unknown fields. The catalog checks the exact recovered values,
both ROM dispatch tables and dynamic frame headers. Rust executes the mechanics;
no extracted assets or speculative quest scripting are embedded in the JSON.

```sh
cargo build --release --locked
python tools/validate_casting.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The expanded casting oracle covers all four attacks: 1,024 pose setups, 2,048
complete casts, 4,096 resistance/damage resolutions and 2,048 effect steps.
It compares every actor byte, RNG, graphics flags, alert state and defeat-reward
count. Fixtures vary foci, party-defense participants, body/willpower, allocation
exhaustion, health, effect frames/timers and allocation slots including zero.
Only audio/HUD/sprite transfers are omitted where reached. Separate Heal and
physical-combat oracles provide regressions.

Five functional tests cover the learned spell paths, all rendered frames,
surviving/defeated targets, completion of the original mana defeat animation,
body/willpower and party-defense differences, unchanged-health reactions,
equipped companion casting, catalog rejection and the real street encounter.
The uploaded Ghidra reference was reused in its private working copy; generated
views under ignored `artifacts/mana-reference` have the verified ROM memory hash.
Windows GUI/input playtesting remains pending. This is a partial game milestone.

Final verification: 68 unit tests and all 65 ROM-backed integration tests pass.
The expanded casting oracle, Heal oracle and physical-combat regression oracle
all pass. Native Linux desktop and Windows release builds pass. The refreshed
515,782-byte Windows ZIP passes archive integrity and all 22 payload hashes;
its README/executable match this build and its x86-64 Windows PE/DLL imports
are verified. The ROM is excluded. See `releases/SHA256SUMS` for the checksum.
This milestone is included in the spellcasting publication checkpoint; see `HANDOFF.md`.
