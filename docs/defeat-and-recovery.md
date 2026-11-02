# Defeat, runner archives and medical recovery

Baseline physical defeat sets `$e166` to 60 through `$aff2/$abb6`. Game services
`$1ddc6..$1ddda` after actor/projectile dispatch; the lethal-hit frame leaves 59.
Expiry requests recovery once. The original 1,024 countdown comparisons remain
covered by the combat oracle.

## Normal Joshua recovery

Normal mode now supports Joshua with zero, one or two companions. `$c716` clears
both companion health bytes before `$c802/$59798` snapshots the live party. The
compact archive at `$fff1fc` has eleven 74-byte records: Joshua, then ten hireable
runners. `$597ee/$598b0` preserve bank/palette, health/condition, equipped item,
ammunition/reserves, name, stats, contract and status bytes. Position, selected
slot, navigation and animation scratch are outside this format. `src/archives.rs`
implements it in memory; disk save/load is still absent. Fresh development sessions
seed unused records from ROM templates, continuing the prior first-hire fixture
policy; this is a host initialization choice, not recovered new-game boot.

Every active companion is removed, including permanent contracts. Their archived
contract and equipment remain intact. `$c8b0` adds byte `$fe` to each runner's
opinion, then replaces a negative result with `$ff`. This preserves wraparound;
it is not ordinary signed saturation. The runtime drops their occupied slots,
refreshes the HUD and returns control to Joshua if a removed runner was selected.

`$c726..$c7c0` restores Joshua's active/idle flags, applies health 6 and condition
8 floors with the original byte subtraction/N tests, and charges
`min(balance >> 3, 5000)`. Equipment, ammunition, wallet remainder and knowledge
persist. Destination assets, HUD, JSON references and dialogue are prepared
before committing fees, healing, party removal or archives. Waiting on dialogue
cannot charge or penalize twice.

The small embedded typed Serde record `data/dialogue/clinic.json` now has schema 2.
Rust executes all state changes; original passage strings stay in the ROM.

## Uninsured and DocWagon routes

With recovery flag bit 0 clear, `$c25bc` shows the original Little Chiba passage.
Enter/Z starts the fade/reload/warm-up sequence through `$2a44/$2b78`: map 2,
camera `(6680,2300)` and Joshua `(6f00,2780)` (hex fixed-point words).

With bit 0 set, `$c25a0` shows the DocWagon rescue and its original MORE option.
The first Enter/Z advances to `$c25b4`, the Seattle General passage; a separate
acknowledgement starts `$2a32/$2b78`: map 0, camera `(6200,2d00)` and Joshua
`(6900,3a80)`. This uses the same fee formula and companion removal. The native
session defaults to uninsured; buying/renewing coverage is not implemented.
Tests explicitly seed insurance and recovery contexts.

Either arrival clears old NPCs, projectiles, encounter tracking, targeting and
exterior-return context. Movement and gunfire resume. Additional scripted clinic
services and doors remain incomplete.

## Trent after defeat

Revisiting Trent now uses `$21174`'s negative-opinion root and `$211e8`'s negative
topic record, with four additional verified JSON nodes. The original fee multiplier
uses the archived opinion: the baseline Joshua quote becomes 430 / 4300 instead
of 285 / 2850. Rehiring restores Trent's compact record through the `$42e6/$5966e`
path, preserving equipment, ammunition and stats, then restores health/condition
to 10 and sets the newly purchased contract. Both payment modes retain their
original insufficient-funds behavior. Job completion, dismissal and other runner
conversations/recruitment remain unimplemented.

## Limits and evidence

Mission-specific `$c64c` callbacks still need a quest-state model. Nonzero combat
modes, unknown recovery flag bits and unsupported lead templates retain the host
PARTY DEFEATED notice without healing/charging; R resets the development session.
No insurance purchasing, general save/load, full game-over or Windows GUI testing
is claimed.

`tools/validate_clinic.py` compares:

- 1,024 full-actor medical cores and fees, both passage chains and both arrivals.
- 1,024 exact compact pack/restore pairs and 1,024 party snapshots.
- All 256 opinion penalty inputs and 1,024 complete companion-removal prefixes.
- 256 full original rehire purchases, both contracts and negative/zero opinions.
- Original root/topic opinion selectors.

Serialization, medical core, penalty and arrival checks have no instruction stubs.
The companion-removal prefix omits HUD/UI refresh only and keeps Joshua selected;
functional tests separately exercise a removed selected runner. Rehire comparisons
omit text/UI transfers and account for the existing native `$ca=30` animation rate.
The dialogue oracle compares all 19 JSON nodes and 35 original branch executions,
including negative-opinion paths, plus 469 contact/gating cases. The older hiring
oracle still checks 54 fee quotes and 24 capacity/funds/contract purchases.

Six ROM defeat tests cover real street defeat, delay/reset fallback, a surviving
party member, uninsured solo recovery, insured/uninsured three-member recovery
with archives/control transfer, and invalid-asset rollback. A further transition
test follows insured defeat -> hospital -> real Space Needle door/contact ->
negative Trent dialogue -> permanent rehire. Its approach coordinates and initial
roster/funds/insurance are explicit development fixtures. The test verifies retained
ammunition, reserves, stats, contract and the 4300 fee.

```sh
cargo build --release --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
python tools/validate_clinic.py /path/to/Shadowrun.bin
python tools/validate_dialogue.py /path/to/Shadowrun.bin
python tools/validate_hiring.py /path/to/Shadowrun.bin
cargo run --release --locked -- clinic-data /path/to/Shadowrun.bin artifacts/clinic-recovery
```

`clinic-data` exports private archive/medical/rehire fixtures and a real street
solo defeat/passage/arrival replay. Ghidra reference views are under ignored
`artifacts/recovery-reference`, exported from the supplied verified GZF working copy.
