# Native Flame Dart and Flame Bolt casting

The later [mana milestone](mana-spells.md) adds Mana Zap/Blast and the original
party-defense register quirk. The [Heal milestone](healing.md) adds the verified self/party healing path.
Other spell limits below describe the preceding attack-spell checkpoint.

The native game now connects two complete single-target spell paths: Flame Dart
(equipped ID 1) and Flame Bolt (ID 2). Cast admission, original rolls, totem/focus
modifiers, post-cast drain, casting pose, target-attached effect timing, resistance,
damage, reaction and original effect sprites run in Rust. Other spell actions,
area effects and ongoing magic remain explicitly unsupported.

## Trying it

Use the default roster and Tab to Ricky. M cycles his actual known spellbook
entries and forces; press it until the title says Flame Bolt (normally twice
from his starting firearm). T selects a target and F casts. G returns to the
first inventory weapon. The normal map-5/default-roster integration test uses
those equipment/input paths and a real ROM encounter, completing a spell impact
at tick 192 with the host's deterministic initial RNG seed. No enemy insertion,
actor/stat edits or fabricated spellbook are used in that test.

M/G are a host keyboard equipment bridge, not the original equipment menu. The
bridge uses original records/forces and conversion utilities, passing the actual
current weapon's capacity index explicitly and preserving the existing cooldown.
It does not claim to recover the ambiguous incoming D5 in companion spell
selection. Automatic gun-to-spell equipment selection remains deferred; an already
equipped supported companion can cast at an available target. Broader companion
spell pursuit/cooldown movement remains incomplete. Equipment changes are blocked
while projectile/spell effects are live to keep this bounded interface coherent.

## Source-backed behavior

`$ae6e` supplies heading and an obstruction probe; the supported native admission
requires a target in the represented pool. A blocked path consumes no cast RNG
or drain. Overwriting an existing target effect is an explicit unsupported exit
rather than leaking or replacing its allocation unnoticed.

`$1033e..$1047c` performs the casting roll and successful-cast drain. Condition 1
produces zero successes without consuming RNG. The roll uses the original magic
attribute/focus search, injury-selected pool, spell skill cap, force, spell focus,
totem categories/bonuses, target flags and condition penalties. Failed rolls do
not create an effect or apply drain. Successful rolls still apply drain if the
18-slot graphics allocator is full. Successful attempts enter ordinary combat
alert before effect allocation. The caller uses `$af7c`'s casting pose/cooldown.

`$15830` attaches an effect to the target, retaining the caster pointer and the
original graphics index/tile base. This is not a flying firearm projectile.
NPCs and effects share the represented 18-slot graphics allocation; encounter
allocation now respects live target effects as well as NPC graphics.

The visible-target phase uses `$521e` for Flame Dart (four frames, two ticks per
frame after the initial hold) and `$5152` for Flame Bolt (six frames, five ticks
per frame after the initial hold). Both start with timer 5/frame 0. Offscreen
effects hold their timers, matching the original visible-actor dispatch guard.
The original dynamic sprite records and palette are composed at the target.
Rendering does not advance simulation or consume RNG.

Completion releases the graphics allocation and executes `$55b78` resistance/
damage, then the applicable hurt/death setup. It reuses the original `$559bc`
staging with magic penetration; ordinary armor is bypassed on these paths. Party
resistance includes the original available-party magic contribution. Source
successes/force/equipment are read at impact, as the original does. The restored
D0 used in the reaction comparison contains the released graphics index, not a
saved health byte; this quirk is preserved. Normal NPC spell deaths use `$b0d4`
pose timing and the existing normal-encounter death/counting boundary. General
mission death callbacks remain incomplete.

## JSON, verification and limits

`data/combat/casting.json` is a small embedded Serde catalog of the two verified
handlers, frame tables, counts and periods. References and frame headers are
validated against the ROM. No extracted sprite/ROM payload is committed. Rust
owns arithmetic, scheduling and state changes. The earlier spellbook JSON remains
in use. Private Ghidra exports from the verified uploaded reference are ignored
under `artifacts/casting-reference`.

```sh
cargo build --release --locked
python tools/validate_casting.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The casting oracle compares 1,024 original pose setups, 1,024 complete casts
through effect admission/drain, 2,048 resistance/damage calls, and 1,024 complete
effect-handler steps. Actor bytes, RNG, graphics allocations, combat alert and
relevant reward/roll state are checked. Pose setup runs without stubs; complete
cast/effect comparisons omit only audio, HUD and sprite transfers. No spell rule,
RNG, allocation, damage or drain instruction is stubbed. Existing physical combat,
spellbook/equipment/drain and encounter regressions remain separate checks.

Unsupported cases include other spell IDs/actions, ongoing-effect modifiers,
area effects, special modes and the special `$9b=$17` damage branch. Targets with
an existing effect defer. Controlled casting requires an explicit represented
target. The full original equipment UI and automatic companion equipment context
remain unfinished. This is bounded native spell casting, not complete magic or
complete game reconstruction. Windows GUI/input playtesting remains outstanding.

Checkpoint: 68 unit tests and all 54 ROM-backed integration tests pass. Casting,
physical-combat, spellbook/equipment/drain and encounter original-routine oracles
pass. Native Linux/Windows builds and Windows ZIP/import checks pass. The changed
casting test also covers lethal NPC impact through the terminal corpse pose.
