# Bounded companion firearm combat

The later [party unarmed milestone](party-unarmed-combat.md) connects ordinary
player/companion fists and exhausted-firearm pursuit. Unarmed boundaries below
record this earlier checkpoint. Weapon melee and special banks remain guarded.


Automatic attack/Heal selection and the bounded caller context are now connected;
see [automatic companion magic](automatic-companion-magic.md) for the current
behavior and remaining limits. The notes below describe this earlier checkpoint.

The later [native casting milestone](spell-casting.md) enables Flame Dart/Bolt.
Earlier casting limits below describe the preceding checkpoints.

The later [spellbook continuation](spellbooks.md) recovers session books and
selection decisions. Casting and equipment mutation remain deferred; the
spell-selection boundary below describes the preceding firearm checkpoint.

Uncontrolled party members now use the original companion decision entry and
ordinary firearm path in the native game loop. This includes Joshua after Tab
transfers control to another runner. The selected runner still fires through F.
This is a firearm milestone; spell selection, melee and general combat AI remain
unfinished.

## Recovered behavior

`$138b8..$138dc` decrements the companion's signed word timer, claims the shared
frame decision flag and resets the timer to 20. Only one companion can claim it
per frame. Other due companions and NPCs retain negative timers. NPC decisions
do not themselves claim that flag. Motion continues between decisions.

Behavior byte `$4d=0`, or no active combat, takes the existing `$14508` leader
formation path. Otherwise `$138f4` checks cooldown, optionally calls `$13a92`
weapon/spell selection, and enters `$139d8/$13ffe`. Target lists refresh once
before actor dispatch on their original phases; a target remains stable until
it leaves the cached list. An empty NPC list runs the recovered ordinary combat
shutdown and returns the companion to leader formation.

The shared firearm dispatcher probes obstruction, obeys the original firing
position flag, consumes finite ammunition, allocates projectiles/FX and uses
original attack/resistance/damage and animation routines. `$14160..$1419a`
lets party members reload from finite reserves instead of applying the ordinary
NPC's exhausted-weapon path. Both projectile/FX allocation failure continuations
are preserved.

`$13d7a..$13db0` selects companion range pairs from `$13e58` using behavior byte
`$4d`; it does not use the NPC weapon table. Far targets become pursuit goals;
near/in-band targets retain the original firing-position/orbit behavior. In-band
selection and reaching a goal preserve the original `$b5` clearing. The shared
steering, wall hugging and physical contact paths remain in use. Formation-query
collision scratch is retained, including the last blocked candidate when neither
candidate is usable, for the next firearm obstruction probe.

An integration test caught an existing offscreen encounter gap: after a companion
cleared combat with no visible targets, spawned attackers stopped approaching.
The recovered `$1398c/$142ae..$142be` branch sends an offscreen, alerted (`$9e`
bit 6) actor toward the controlled runner even with combat cleared. It now keeps
moving between decisions and can re-alert when visible. Other offscreen patrol
and script paths remain outside this implementation.

## Explicit boundaries

`$13a92` returns immediately for signed `$9b >= 2`. With a currently equipped
firearm it also skips spell selection when signed condition byte `$9c < 5`.
These source-backed branches are enabled. A ready spell-capable companion needing
`$13b12` is deferred before reading the session spellbook, which has not been
reconstructed. The title reports that boundary; no empty spellbook or invented
gun preference substitutes for the missing state.

This means the default Ricky/Freya companions do **not** gain normal automatic
firearm attacks while Joshua is selected. Switching control to Ricky lets the
mundane Joshua use this firearm path. Non-firearm equipment, spell actions,
ongoing effects, special world modes and weapon exhaustion into melee still
need their own recovery. The development roster/funds remain fixtures, not the
original new-game or hiring sequence. Mission-specific recovery is unchanged.

## Verification

```sh
cargo build --release --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
python tools/validate_combat.py /path/to/Shadowrun.bin
python tools/validate_encounters.py /path/to/Shadowrun.bin
python tools/validate_followers.py /path/to/Shadowrun.bin
```

Use the documented Rust and Python environments in `HANDOFF.md`. Targeted Ghidra
views were exported from the verified uploaded reference's private working copy;
analysis and ROM bytes remain untracked.

The combat oracle adds 1,024 scheduler cases, 2,304 companion range goals and
1,280 complete decisions, including obstruction walls, range behavior, target
replacement, empty-list shutdown, pool exhaustion, reserve reloads and low
condition. Another 256 cases stop at the real spellbook-selection entry and
compare the deferred prefix. Complete decisions compare all ten actor records,
projectile/FX pool, RNG, alert state, firing/collision scratch and shared flags;
only audio/HUD transfers are omitted. Timers, range goals and spellbook-entry
prefixes run without instruction stubs. Existing combat comparisons still pass.
The encounter oracle now compares 256 offscreen pursuit decisions, including
128 with combat cleared, without instruction stubs.

Functional tests cover automatic companion fire and enemy damage, finite
reloads, the spell boundary, and control handoff. A normal map-5 load with the
unchanged Joshua/Ricky/Freya roster, Ricky selected and idle input spawns a real
ROM encounter and records Joshua's first companion shot at tick 148 with the
host's fixed initial RNG seed. No enemy insertion, stat edits or player firing
are used in that test. Changing play changes the shared RNG and timing.

62 unit tests and all 44 ROM-backed integration tests pass. The combat, encounter
and follower original-routine oracles pass. The Windows package is cross-built
and inspected. Actual Windows GUI/input
playtesting remains outstanding; the native reconstruction is still unfinished.
