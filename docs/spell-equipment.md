# Companion equipment conversion and post-cast drain

This checkpoint completes recovered equipment-conversion and post-cast drain
APIs. The native companion loop now connects the original unsafe-current-spell
fallback to a supported inventory firearm. Actual spell casting, effect scheduling,
spell resistance and spell impact are still unfinished. A full spell action is
not claimed by this milestone.

## Recovered behavior

`$13af2` scans eight inventory slots for the first item ID below 13, including
zero. `$13466/$36fc` equip it and restore stored percentage ammunition with the
original capacity multiplication and half-up rounding. An empty inventory uses
`$13478`'s absolute memory read from ROM address `$7f` (value `$20` in this ROM),
not an immediate `$7f` constant. Original out-of-model capacity lookups are
rejected transactionally rather than repaired or followed into unmapped memory.

`$13b7a/$125fa` convert current ammunition before setting the equipped spell ID,
force, slot byte and cooldown. `$125fa` reads incoming register D5 as the capacity
index; this API requires that register explicitly. It does not silently replace
D5 with the current weapon ID. Its rounding test examines the packed DIVU result
shifted right eight, so a remainder or quotient >= 256 causes an increment. The
API preserves this behavior, including byte truncation. The full caller's D5
provenance must be resolved before enabling spell equipment in general gameplay.

`$10422..$1047c` apply drain after a successful effect. Force is zero-extended on
this path, whereas the earlier selection estimate sign-extends it. Both preserve
the original branch/overflow semantics. Drain foci (item `$13`) are consumed in
slots 6 through 0; slot 7 is ignored. An absorbing focus returns immediately.
Otherwise `$37ec/$3674` subtract condition and spill a negative remainder into
health. The zero-condition continuation negates D1 again at `$10466`, restoring
condition to 1. It does not subtract another point. Inactive post-cast actors
are rejected: some original inactive-state combinations loop indefinitely and
are not legitimate successful-cast inputs.

All APIs stage actor changes before committing. The native companion loop also
stages selection/RNG: an unsafe current spell may now select and equip the first
supported firearm and continue through original firearm dispatch. Unsupported
melee/casting stays deferred. No spell effect or drain is applied merely because
a spell has been selected.

## JSON, evidence and verification

The embedded Serde/JSON spellbook catalog remains responsible for source records
and classifications. Arithmetic and engine state changes remain typed Rust.
No speculative spell rules, extracted asset payloads or general scripting VM
were added. Ghidra views from the user's verified reference are private under
ignored `artifacts/spell-equipment-reference`.

`tools/validate_spellbooks.py` now compares 8,192 complete equipment conversions
and 4,096 complete drain applications against original instructions, checking
all actor bytes. Only the HUD transfer routines are omitted. The earlier 170-byte
initialization, 4,096 estimates and 4,096 selection-prefix checks still pass
without instruction stubs. Equipment fixtures include inactive/active actors,
all inventory positions, empty inventories, varied incoming D5, ammunition,
force, focus charges and condition/health boundaries.

Functional tests cover gun → spell → gun ammunition preservation, transaction
rollback, unsafe equipped spell → native firearm continuation, focus consumption,
health spillover and minimum condition. The new drain API is tested separately
from gameplay because the spell-effect pipeline remains unimplemented.

Next recover the effect scheduler and one complete spell action, including
resistance and impact, before enabling casting. Keep the explicit D5 context
boundary until its full caller has been verified. Windows GUI/input playtesting
remains outstanding.

Checkpoint validation: 66 unit tests and all 49 ROM-backed integration tests pass.
The equipment/drain, combat and encounter original-routine oracles pass. Linux
and Windows builds pass; the Windows ZIP is refreshed and verified without a ROM.
