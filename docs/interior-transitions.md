# Space Needle entrance and exterior return

The game core supports one bounded interior path: map 0 trigger `C6` enters
map 10 (Space Needle), and that room's `80` exit restores the exterior. Other
door types remain blocked until their scripts are translated. Numeric addresses
and trigger bytes in this document are hexadecimal; map IDs are decimal.

## Recovered path

The source descriptor's pointer at offset `4e` selects the destination from
`d54b2`; entry 6 is map 10. Before loading it, the original main-loop location
dispatch `1f5ca` runs an interaction script. In the explicit no-mission fixture,
this reaches `1f246` with node `110b5e`, portrait context `1220b4`, and text
pointer `113dd6`. The next node word is `ffffffff`: this is a simple passage,
not a branching conversation. It describes entering Seattle's Space Needle and
riding the elevator to the bar. `1ffd4` sets `fff0d6`; continuation `1fed8`
permits entry after the passage is acknowledged.

Native code displays that exact ROM text with the font decompressed from
`c1064` and glyph mapping at `c1320`. Enter or Z acknowledges it. The panel,
wrapping and keyboard hint are host presentation choices; the original full
portrait/conversation screen and general script VM are not implemented. The
world and runner selection remain frozen while the passage is open. Branching
nodes and unknown text controls are rejected rather than automatically bypassed.

At entry, `bc36/bc66` select return offsets at `bd34` from the first negative
footprint code. The offsets use the controlled player's position **before** the
blocked movement candidate. The current camera is saved without rounding it to
an exterior seam's `ff80` mask. `16e30` saves 49 words: map ID and the 96-byte
descriptor. The native return context preserves these values and the three
formation positions. `167ae` then chooses map 10. Its normal descriptor camera
and positions, applied by `2b34`, supply the interior arrival.

Room 10 has two adjacent `80` exit rectangles at pixel bounds
`224..255 × 224..255` and `256..287 × 224..255`. With an existing return context,
native exit restores the saved descriptor, camera and formation corresponding to
the blocks at `2f5c`, `2f82` and `2fa4`. The normal fade/reload/warm-up phases from
the [exterior milestone](area-transitions.md) surround both legs. Party identity,
health and equipment/inventory bytes survive. NPCs are cleared on exit, and a
subsequent visit requires a fresh acknowledgement and return context.

## Fixed NPC record

This entrance selects spawn script `17018`, compact prototype `171a2`, and fixed
coordinates at `1702e`. The native deserializer follows `44bc` and applies the
script's byte overrides at `45f4`: facing `4000` and entity byte `9e = 10`.
The first free allocation uses entity slot 3, graphics base `49`, and transfer
destination `0920` (`1a342`, `1a2cc`). The record is omitted if the selected test
roster already contains Trent's template, corresponding to the original recruit
check for this spawn. This is not a new recruitment/save system.

The record is rendered and idly animated. NPC AI, random behavior timers,
interactions and recruitment dialogue are still absent. The idle facing/rate
selection is a native presentation choice; those updates are not presented as
an equivalent original NPC dispatcher.

## Reproduce the evidence

```sh
cargo build --release --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test transitions -- --ignored
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_interior.py /path/to/Shadowrun.bin
cargo run --release --locked -- interior-data /path/to/Shadowrun.bin artifacts/space-needle
```

The original-routine oracle checks the entry gate/node/text/font, route and saved
descriptor, interior camera/formation, return offsets and restore blocks, and
fixed NPC initialization/overrides. It explicitly skips the original dialogue
UI, NPC random timer and AI, and unrelated location subsystem calls. Native
idle state and animation rate are adjusted explicitly in that comparison.

The integration test seeds a player at `(528,144)` outside the actual entrance,
then uses normal `Game::update` input for two successive entry/exit trips. It
checks acknowledgement, frozen movement/selection, completed fades, NPC load and
clear, saved camera/formation, and changed party-state preservation. Two existing
tests still cover exterior seam round trips and an unsupported `C0` door.
All three ROM tests pass when explicitly enabled; the default suite has 41
passing unit tests and skips those three tests because the ROM is not in Git.

`interior-data` uses that same game core to capture approach, passage, interior
and return frames plus oracle data. No map reassignment occurs between legs.
Generated frames and extracted bytes remain ignored under `artifacts/`.
These are seeded door tests, not a full new-game traversal or Windows GUI test.

## Remaining scope and next work

The runtime does not yet model mission/quest state, so this entry implements the
traced baseline only. Do not extrapolate it to other `C0..DF` doors, fees, recruit
conditions or story gates. Starting directly on map 10 does not manufacture a
saved exterior or execute this entrance script. Original camera tracking,
follower/NPC AI, dynamic world persistence and sound remain separate work.

Next trace the interaction dispatcher for this room's fixed NPC, including how
the player targets it, reaches the conversation node and processes choices.
Translate one real branching interaction with its state changes before widening
door coverage. Then connect follower dispatch and additional location types.
