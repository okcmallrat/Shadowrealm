# Companion attack-spell movement

Automatic attack/Heal selection and the bounded caller context are now connected;
see [automatic companion magic](automatic-companion-magic.md) for the current
behavior and remaining limits. The notes below describe this earlier checkpoint.

Already equipped Flame Dart/Bolt and Mana Zap/Blast companions now continue
through original combat positioning. They pursue a retained enemy while
offscreen, recover a blocked approach, reposition while cooling down, and rejoin
the leader immediately when the enemy list becomes empty. This closes the
previous native branch that could cast but returned before movement decisions.

With the default roster, Tab to Freya, press M once from her starting weapon to
equip her learned force-3 Mana Blast, then Tab back to Joshua. Freya can approach
and cast at supported street enemies without F input. Movement and target choice
still use the shared original RNG and recovered actor scheduler. A real map-5
test completes her spell at tick 116 with idle input after that equipment/control
handoff, without inserted enemies or edited actor health/stats/spellbooks.

M remains a host equipment shortcut. Automatic gun-to-spell conversion still
defers at the incoming D5 capacity context. Healing keeps its existing bounded
party-target branch; its pursuit/target-redirection path has not been expanded
by this milestone. Other spell actions, original menus and full magic AI remain
unfinished. Overwriting live target effects and special contexts retain their
previous explicit boundaries.

## Recovered execution

The existing `$138b8` companion scheduler still services one due companion per
frame and resets its timer to 20. It skips equipment reconsideration while
cooling down and retains the recovered spellbook selector and unsafe-spell
inventory-firearm fallback. No invented preference or free spell is added.

For ready visible attack casters, `$140a0` probes obstruction before calling
`$ae6e`, which probes again before rolling. A blocked or offscreen action tails
through `$144fa/$1450e` to an offset near its retained enemy. Clear actions use
the common `$13d56` behavior-dependent range goals and `$145a2` steering. The
native firearm path now shares that positioning helper; its behavior is checked
by the existing complete firearm-decision oracle.

`$14146` runs while cooling down. It reads the stored weapon-ammo field `$4e`
even for an equipped spell, and checks `$4f` reserves. Rust retains this rule;
it does not substitute condition or cast drain. An exhausted actor without
reserves clears equipment before the unimplemented melee continuation. That
prefix is reported as deferred, with no fabricated melee action.

When no valid enemies remain, the original `$1dbf0` cleanup clears combat links
and `$14508` resumes leader formation during the same serviced decision.
Target retention, allocation, actual drain and effects otherwise use the shared
casting core described in [the mana notes](mana-spells.md) and [Heal](healing.md).

Two additional instruction details surfaced in full decision comparisons:

- A nonzero cast roll uses `$afae`, entering pose handler `$afe2`; a failed roll
  uses `$af7c`, entering its `$afe0` NOP. Both now retain their actual handler
  pointer and finish through the native actor phase. This applies to player
  casts as well as companions.
- `$106d0` sign-extends the drain quotient before unsigned division by two.
  A negative 16-bit quotient overflows that division, leaving D6 unchanged.
  The drain API now preserves that result instead of halving it as an unsigned
  Rust value. Broad behavior/stat fixtures exercise this boundary, including
  its subsequent original condition subtraction and weapon fallback.

## JSON and validation

The existing schema-3 Serde casting catalog and typed spellbook catalog remain
the content boundary. This milestone reuses their verified spell references;
movement, arithmetic and scheduling stay in Rust. No general JSON AI scripting,
extracted assets or guessed content records are added.

```sh
cargo build --release --locked
python tools/validate_caster_movement.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The new oracle compares 2,040 complete original `$138b8` decisions, including
270 successful casts, plus eight exhausted-equipment prefixes stopped at the
real melee entry. It executes selection, targeting, both obstruction probes,
rolls, allocation, drain, poses and navigation. Every actor byte, RNG, graphics
flags, alert state, firing flag, collision scratch, scheduler flag and firearm
fallback FX records/signals are compared. Only audio and HUD/VDP transfers are
omitted. Both companion slots, all four supported attacks, visibility, cooldown,
retention/replacement, blocked paths, empty lists and full graphics pools vary.

Six game tests cover wall recovery followed by a cast, offscreen return,
cooldown movement without resource use, immediate formation recovery, successful
and failed pose completion, and the real street encounter. Isolated movement
fixtures explicitly supply test opponents and a bounded collision arena; they
do not claim to be original world replays. Their finite border keeps navigation
scans inside the represented raster. The real street test uses the actual map.

The uploaded Ghidra working copy was reused and its ROM memory hash reverified.
Private reference views remain ignored under `artifacts/caster-movement-reference`.
Windows GUI/input testing remains pending. This is a partial game milestone.

Final verification: 68 unit tests and all 71 ROM-backed integration tests pass.
The new companion oracle and existing casting, Heal, spellbook/equipment/drain
and physical-combat oracles pass. Native Linux desktop and Windows release
builds pass. The refreshed 516,639-byte Windows ZIP passes archive integrity
and all 22 payload hashes; its README/executable match the current build and
its Windows x86-64 PE/DLL imports are verified. The ROM is excluded. The archive
checksum is in `releases/SHA256SUMS`. This milestone is included in the spellcasting
publication checkpoint; see `HANDOFF.md` for publication state.
