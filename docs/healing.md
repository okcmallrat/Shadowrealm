# Native Heal and party recovery

The later [mana milestone](mana-spells.md) expands the shared catalog to schema
version 3 and adds Mana Zap/Blast; the healing rules below remain in use.

Heal (equipped spell ID 12) now runs through the existing native casting path:
original rolls, drain, target initialization, 16 effect frames, health recovery,
condition restoration and fallen-state clearing. It supports self and available
party targets. Other spell actions, full magic AI and the original menu remain
unfinished.

## Playing this checkpoint

With the default roster, Tab to Freya and press M until Heal appears. Her original
spellbook supplies force 4; no spell or force is fabricated. Selection initially
targets herself. P cycles the available party members, including fallen members
still in the party, and F casts. T still toggles target selection; with Heal it
selects a party/self pointer instead of choosing an NPC. G restores an inventory
weapon. M/G/P are host keyboard bridges while the original menus are pending.

An already equipped Heal companion can use the recovered critical-health party
priority during combat. Automatic gun-to-spell equipment conversion still defers
at its unrecovered caller context. General spell pursuit/cooldown movement and
casting outside the represented contexts remain incomplete.

A real map-5/default-roster test waits for an original street encounter to injure
a party member, then uses Freya's learned Heal and the same party-target controls.
It restores Joshua from health 9 to 10 at tick 296 with the host's deterministic
initial RNG seed. That test does not insert enemies or write actor health/stats.
Separate component fixtures deliberately supply wounds/fallen flags to cover
self-healing, restoration of control and boundary cases.

## Recovered behavior and JSON

Casting still executes `$1033e..$1047c`, with the existing focus/totem/condition
rules and actual post-cast drain. Self-target admission follows `$ae6e`'s same-
actor probe bypass. Other targets use the represented obstruction probe. Failure
and full graphics allocation retain the original roll/drain behavior.

`$15830` initializes Heal's target effect and dispatches the original initializer
from `$4964`. For Heal that entry is `$b574`: a direct walking callback on the
target. The implementation preserves that call, including selected/unselected
speed-table differences, prior motion flags, collision sliding and lack of the
outer actor dispatch's state-change notification. It is not replaced with an
instant heal or discarded because its movement is unexpected. Triggered travel
and special walking contexts defer transactionally; they do not invent a door
or movement result. An unsupported callback commits no cast RNG, resource or
allocation changes.

The visible-target handler `$56de` starts at timer 5/frame 0, then advances every
five ticks. After frame 15 it releases the shared graphics allocation and calls
`$55c2c`. All sixteen original dynamic sprite records use the original `$6000`
palette. Offscreen targets hold the effect timer. Render frequency does not
advance the effect or consume RNG.

`$55c2c/$3674/$37ec` read the caster's current success byte at completion, with
the original minimum-one and signed-byte arithmetic. Health is restored with the
original clamping/overflow behavior; zero condition is raised to one for an active
target. Bits 4 and 6 of actor flags are cleared, the actor is made active and the
idle handler is restored. Other state, including equipment/stats/contracts, is
preserved. Native targeting admits fallen active party members, but not removed
runner archives or enemy targets. Existing medical/mission recovery is unchanged.

Schema version 2 of `data/combat/casting.json` describes the verified effect kind,
initializer/handler pointers, frame table, count, period and palette. Serde rejects
unknown fields, and references are checked against both original dispatch tables
and frame headers. Rust executes the rules and state updates. No extracted asset
payload or speculative quest scripting is added.

## Verification

```sh
cargo build --release --locked
python tools/validate_healing.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The Heal oracle checks 1,024 original initializer movement callbacks without
stubs; 2,048 full health/condition/flag completions; 1,024 complete self/party
casts through allocation, callback and drain; and 1,024 complete effect-handler
steps. Only audio, HUD and sprite transfers are omitted where reached. All actor
bytes, RNG, graphics flags and alert state are compared. Existing casting,
physical combat, followers and encounter oracles remain separate regressions.
The verified uploaded Ghidra reference's private views remain ignored under
`artifacts/healing-reference`; its program memory hash matches the supplied ROM.

Functional checks cover all sixteen rendered frames, self healing, a fallen
Joshua recovering health/condition and becoming controllable, target-pointer
retention, companion priority without player fire input, NPC rejection and
transactional rollback on unsupported callback travel.

Final milestone verification: 68 unit tests and all 60 ROM-backed integration
tests pass, including the six new healing tests. Heal, casting, physical combat,
followers and encounters pass their original-routine comparisons. Native Linux
desktop and Windows release builds pass. The refreshed 513,834-byte Windows ZIP
passes archive integrity and all 22 payload hashes; its README and executable
match this build, and its executable/DLL imports are verified as x86-64 Windows.
The ROM is not bundled. See `releases/SHA256SUMS` for the archive checksum.

This milestone is included in the spellcasting publication checkpoint. Windows GUI/input playtesting
remains outstanding. This is a bounded healing milestone, not full magic or a
completed game.
