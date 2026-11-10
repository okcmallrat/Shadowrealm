# Automatic companion magic

The later [party unarmed milestone](party-unarmed-combat.md) connects ordinary
player/companion fists and exhausted-firearm pursuit. Unarmed boundaries below
record this earlier checkpoint. Weapon melee and special banks remain guarded.


Ricky and Freya now choose supported learned attack spells or Heal, convert
their equipment, target and move, cast, and restore inventory firearms when the
original drain estimate rejects magic. The normal companion scheduler makes
these decisions without M/G/F input. Typed Serde/JSON remains the content and
reference boundary; Rust executes the recovered rules.

The default map-5 roster `[0, 3, 10]` completes an automatic Ricky spell at tick
157 with idle input, two automatic spell equipment changes and a real street
encounter. That test inserts no enemies and changes no actor health, stats,
books or equipment. By tick 328 both Ricky and Freya have completed magic and
an enemy has finished its defeat animation; the party remains alive. Other
component fixtures separately cover Freya's automatic Heal for each party slot, blocked enemy paths, unsafe-spell firearm fallback,
and transactional deferral of unavailable conversion context.

## Equipment caller context

The previously missing D5 argument is register residue from the preceding active
actor, not the current weapon ID. The actor loop at `$a8f0`, dispatcher at
`$aaa0`, companion call at `$ac8e` and selector at `$13a92` do not initialize it
for the next companion. The selector and drain estimator preserve it.

Normal idle/action handlers load fixed-point X and call `$b48c`; collision
routine `$e7c8` shifts D5 right four and leaves pixel X. Walking `$b574/$b77e`
saves and restores D5 around that collision call, and `$bbe4` commits it as the
new fixed-point X. A stopped walk falls through idle and leaves pixel X.
The native loop tracks this verified residue from the preceding actor's
completed normal handler. Inactive slots do not overwrite it.

`$13b7a` calls `$125fa` for a nonzero old equipped ID signed below 13, including
spell-to-spell changes. `$125fa` multiplies incoming D5.w by 48 and reads a byte
at `$d4540 + index*48 + $24`. In these contexts that can be an unrelated ROM
byte. Native conversion preserves that lookup, the original DIVU rounding and
byte truncation; it does not substitute a plausible weapon capacity.

The small `data/combat/companion-magic.json` catalog validates exact caller,
handler and table references against the supported ROM. It contains no extracted
asset payloads. Unknown/contact handlers, trigger scripts, special banks/flags,
the first party slot's unverified incoming register, and outside-ROM conversion
reads remain deferred with selection/source/RNG staged. This bounded handler
contract is not a full-frame CPU register simulation.

## Heal decision and target order

`$13c20` checks Joshua, then available slot 1, then slot 2 for health below four.
It writes shared `$ffe190` only when that scan finds a critical party member.
The hint survives a scan that returns early at the condition gate, and an
unsuccessful scan leaves the old hint in place.

The companion first acquires or retains an NPC enemy at `$13ffe`. Without one,
it ends ordinary combat and resumes formation, even with an injured party member.
A ready visible spell probes the enemy path at `$140b4`. Heal redirects `$a2`
to the shared party hint only after that probe clears; `$ae6e` then probes and
rolls against the recipient. A blocked enemy path causes the original approach
movement instead of an immediate heal.

Heal's initializer still calls `$b574` on the recipient. Its final collision
scratch is retained for subsequent decisions. Shared `$13d56` positioning reads
the current target after casting, including party or self pointers; self-goals
can immediately replace the casting pose with idle. Cooldown movement still
reads stored weapon ammunition `$4e` and reserves `$4f`, even with a spell equipped.

## Validation and limits

```sh
cargo build --release --locked
python tools/validate_automatic_magic.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The new oracle executes the original preceding actor dispatcher/collision path
to obtain D5, then compares complete `$138b8` decisions using explicit incoming
world and scratch state. It checks all ten raw actors, RNG, Heal hint, scheduler,
combat alert, firing state, collision scratch, graphics occupancy, projectile
pool and launch signals. The oracle passes 1,024 preceding-handler D5 cases
(including stopped walks), 3,031 complete decisions with 205 spell equipment
changes and 154 successful casts (27 Heal effects),
plus 41 exhausted-equipment prefixes stopped at the unimplemented melee entry.
Only audio/HUD/VDP transfers are omitted. These are complete decision-boundary
comparisons, not an end-to-end original frame replay.

Automatic status spells, unsupported register/trigger contexts, target-effect
overwrites, other spell actions, original menus and general magic AI remain
unfinished. The supplied roster and deterministic host seed are still development
choices; new-game creation/recruitment and complete game progression are pending.
Windows cross-compilation does not establish Windows GUI/input playtesting.

Final verification: 69 unit tests and all 83 ROM-backed integration tests pass.
Earlier spellbook/equipment/drain, Heal, caster movement, casting/damage/effects,
physical-combat and status-spell original-routine comparisons pass. Native Linux
and Windows release builds pass. The 531,747-byte local Windows ZIP is refreshed
and verified for CRC, all 22 payload hashes, README/executable identity and
PE/DLL imports. It includes no ROM or Ghidra database. This continuation is included in the courier-loop
publication checkpoint; Windows GUI/input playtesting is pending.
