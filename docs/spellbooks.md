# Spellbook state and companion selection

The later [native casting milestone](spell-casting.md) enables Flame Dart/Bolt.
Earlier casting limits below describe the preceding checkpoints.

The later [equipment/drain checkpoint](spell-equipment.md) implements equipment
conversion and actual drain APIs. Casting remains disabled. The selection-only
boundary below records the preceding milestone.

This is the next verified step toward full companion magic, not enabled spell
casting. It recovers session spellbooks, estimated drain and the original
selection decision. Equipment mutation, spell projectiles/effects, resistance,
condition drain application and spell-learning interfaces remain unfinished.

## JSON and Rust responsibilities

`data/combat/spellbooks.json` is an embedded, typed Serde catalog. It describes
five source-record addresses, record size, initialization/classification routine
references and the recovered attack/healing classifications. Runtime filtering
uses those validated JSON classifications. The catalog is parsed once when the
session loads; it is not parsed each frame. Unknown fields, altered source
addresses/classifications and unsupported versions fail validation.

Spellbook contents stay in the supplied ROM. There is no extracted spellbook
payload in the JSON and no separate configuration file needed on Windows.
`src/spellbooks.rs` reads and validates the five original records, stores mutable
session copies and executes the recovered rules in Rust. Map/combat clearing
preserves the session books. New sessions initialize them again from the ROM.
This is neither disk saving nor a reconstructed full new-game boot.

Keep JSON for reviewed content references and recovered declarative records.
Keep arithmetic, scheduling, collisions and action execution in typed Rust.
Extend the existing dialogue/quest JSON gradually when connecting actual jobs;
do not invent a general scripting system before its behavior is recovered.

## Source-backed behavior

- `$566a2` copies five 34-byte records from `$166ad8` into `$fff9fa`, preserving
  all 170 bytes, including unused entries/padding. Actor word `$a0` selects a book.
- The count word is the last index: `$13c74` executes count + 1 entries. Attack
  IDs 0, 1, 7 and 8 retain order and duplicates. ID 11 supplies the last healing
  force. These IDs are zero-based, unlike equipped item byte `$57`.
- `$10660` estimates drain using `$55f02/$55f20`, the spell table and `$166ad4`.
  The focus search visits slots 6 through 0 and ignores slot 7. Byte wrapping,
  remainder rounding, logical shifts and the distinct BPL/BGT overflow semantics
  are retained. This estimate does not consume RNG or apply condition damage.
- `$13cde` treats condition 1 as unsafe; otherwise it checks the signed drain
  estimate and compares its byte to condition. This is not a generic threshold.
- `$13c20` prioritizes Joshua, then available companion slots, at health < 4.
  Inactive actor-pool placeholders do not count as injured companions.
- `$13a92/$13b12/$13bba` preserve mundane/condition skips, current spell handling,
  healing preference, a single shared-RNG attack selection, and unsafe-drain
  fallback. There is no reroll or replacement gun preference.

The selector returns `KeepEquipped`, `Spell` or `InventoryWeapon`, stopping before
original equipment mutation. The game now loads the session books and evaluates
that result when a companion is ready. Supported keep-equipped firearm decisions
continue. Unsupported spell/equipment decisions remain explicit deferred exits;
their proposed source/RNG changes are staged and not committed. This staging is
a native implementation boundary, not the original response to casting a spell.
Default magic companions still do not cast, but the reason is now the missing
action path rather than a missing or fabricated spellbook.

## Verification

```sh
cargo build --release --locked
python tools/validate_spellbooks.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

Use the documented environment/tool paths in `HANDOFF.md`. The new oracle runs
actual original instructions without stubs: all 170 initialization bytes, 4,096
estimated drains and 4,096 selection prefixes. Selection cases cover all five
books, varied condition, current spells, injury/availability, focus inventory,
forces and seeds. They compare selected spell/force, healing target, actor
immutability and shared RNG. The prefix endpoints are `$13ae4` (keep), `$13b7a`
(spell selection before equipment mutation) and `$13ae6` (inventory fallback).
It does not claim to validate casting or equipment conversion.

Targeted Ghidra views came from the user's verified analysis reference in its
private working copy. They stay under ignored `artifacts/spellbook-reference`.

Next connect the original equipment conversion and one complete spell action,
including resistance and actual drain, before enabling magic-capable companions.
Then extend the small dialogue/quest records through a verified job/payment/
contract route. Mission and balance rules must remain source-backed.

Verification checkpoint: 65 unit tests and all 46 ROM-backed integration tests
pass. The existing combat oracle remains a separate regression check. Windows
build/package checks do not establish GUI/input playtesting on Windows.
