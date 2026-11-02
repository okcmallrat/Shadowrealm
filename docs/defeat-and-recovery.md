# Defeat and bounded medical recovery

Baseline physical defeat sets `$e166` to 60 through `$aff2/$abb6`. Game services
`$1ddc6..$1ddda` after actor/projectile dispatch; the lethal-hit frame already
leaves 59. Zero holds and expiry requests the continuation once. 1,024 original
prefix comparisons cover the countdown without stubs, stopping before scheduling.

## Uninsured solo Joshua

The normal-mode, uninsured solo Joshua path now continues into original medical
recovery. `data/dialogue/clinic.json` is a small typed Serde reference record for
routine `$c64c`, passage `$c25bc`/text `$c23d7`, map 2 and spawn `$2b24`.
Rust performs the recovered logic; text remains in the user's ROM. The record's
schema, scope, addresses and original node pointer/terminator are validated.

`$c726..$c7c0` restores Joshua's active/idle flags and applies health 6 and
condition 8 floors in their normal range. The byte subtraction/sign tests are
preserved exactly, including wrapped out-of-range values. It deducts
`min(balance >> 3, 5000)` from the wallet. The native code prepares the passage
and validates destination assets before committing these changes. It then freezes
simulation on the original Little Chiba passage. Payment and restored health
are already committed, as in the original order; waiting cannot charge twice.
Enter/Z acknowledges and starts the existing fade/reload/warm-up sequence.
`$2a44/$2b78` supplies map 2, camera `(6680,2300)` and Joshua `(6f00,2780)`
(all hexadecimal fixed-point words). Movement and equipped gunfire resume there.
Equipment/ammunition, wallet remainder and existing knowledge survive. Prior NPCs,
projectiles, encounter tracking and exterior-return state are cleared.

This baseline session starts with recovery/insurance flags zero. Nonzero flags,
nonzero combat mode, other lead templates or companions defer to the host
PARTY DEFEATED notice, with no recovery charge or free heal. R reloads the chosen
development session; Enter does not dismiss that fallback notice. Reset also
clears pause/frame backlog. The fallback notice and R reset are host UI, distinct
from original recovery. Tab remains blocked during either defeat notice.

## Scope and original continuation

The solo `$c802/$59798` archive save has no additional modeled runtime effect;
there are no companion/archive/history records in this subset. Original mission
branches at `$c64c`, insurance passage `$c25a0`, companion removal/archive and
contract-history changes remain unimplemented. Insurance/story flags select the
other `$2a32` destination (map 0, spawn `$2aea`). General clinic interactions,
missions and additional scripted doors are not enabled by arriving on map 2.
The working-copy GZF views remain private under `artifacts/defeat-reference`.

## Evidence

1,024 full-actor recovery/fee cores match original `$c726..$c7c0`, with all
condition byte values, flag combinations and low/high/threshold balances. The
uninsured node, route prefix and complete arrival also match. No instruction is
stubbed; the core comparison stops before dialogue/UI calls and the route prefix
before sound shutdown. These are bounded comparisons, not the entire callback.

Four ROM integration tests cover a real spawned attack against an explicitly
wounded Joshua, the delay/frozen fallback/reset, a surviving companion, the whole
uninsured recovery loop (including one-time payment and post-arrival controls),
and deferred multi-party/insurance/mode contexts. The wounded solo fixture changes
health/condition and funds explicitly; attackers still use ordinary map 5 selection
and placement. It pays 5,000 from 40,001 and resumes in map 2 with health 6,
condition 8, balance 35,001 and its original ammunition.

```sh
cargo build --release --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test defeat -- --ignored
python tools/validate_clinic.py /path/to/Shadowrun.bin
python tools/validate_combat.py /path/to/Shadowrun.bin
cargo run --release --locked -- clinic-data /path/to/Shadowrun.bin artifacts/clinic-recovery
```

`clinic-data` captures original passage/arrival frames and recovery state.
`encounter-data` retains the street replay and captures `wounded-defeat.ppm` from
an explicitly unsupported insured fixture for the fallback screen. Windows
cross-compilation/package verification does not establish GUI playtesting.
