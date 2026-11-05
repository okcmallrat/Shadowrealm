# Invisibility and Rockskin

The native game now casts Invisibility (equipped ID 4) and Rockskin (ID 11),
including their original instantaneous initializers, drain, status clocks and
tile transforms. Timed attack/Heal effects remain in the existing casting
catalog; a small typed status catalog supplies verified references for these
two instant spells. Other spells, original menus and full magic AI remain partial.

## Playing this checkpoint

Tab to Freya and press M until Invisibility appears. Her original book supplies
force 3. Selection initially targets herself; P cycles living available party
members, and F casts. Ricky's original book supplies force-2 Rockskin through
the same M/P/F controls. G restores an inventory weapon. The status remains
active after changing equipment. M/G/P remain host shortcuts, not original menus.
Use Heal for fallen active runners; these two status spells do not admit them.

A real map-5/default-roster test waits for an original street encounter, casts
Freya's learned Invisibility and restores her inventory weapon. It completes
at tick 11 with two spawned attackers and duration byte 16 under the host's
initial deterministic seed. It inserts no enemies and changes no health, stats
or spellbook records. Other input changes the shared RNG and timing.

This milestone does not enable companions to cast these spells automatically.
Their recovered `$140a0` path treats IDs 4/11 separately from attack spells;
automatic equipment conversion still waits at its incoming D5 context. Existing
attack-caster movement and bounded companion Heal remain in use.

## Casting and duration

`$1033e/$15830` use the existing roll, focus/totem rules and successful-cast
drain. `$4dca` sets Invisibility flag `$11` bit 7 and writes successes times 8,
with byte wrapping, to `$40`. `$4dfe` sets Rockskin bit 5 and writes the same
duration to `$41`; its extra `$104f6` register calculation does not replace the
success byte used for duration. Both clear `$b2` immediately. No timed impact
or fabricated effect animation is scheduled for them.

The original allocates a graphics slot before either initializer, and neither
initializer nor timer expiry releases it. Rust preserves that allocation state
until the existing map reset clears the pool. Repeated casts can exhaust it;
the original full-pool branch still applies successful-cast drain without
applying the status. This behavior is covered by full-cast comparisons.

After the visible actor's effect phase, `$46f8/$4706` call `$4e64/$4e38` for
active flags. Once per 64 phase ticks, the corresponding byte decrements. The
flag clears when that decrement is signed-negative, including zero wrapping
to `$ff`; a byte of `$80` instead decrements to positive `$7f`. Offscreen and
inactive actors hold. Invisibility also holds when `$9e` bit 6 is set. Rendering
does not advance clocks or consume RNG. Normal updates and transition warm-up
service the clocks using the existing simulation phase.

## Original visuals and combat interactions

`$3efe` masks original plane-B source tile bytes through each nonzero nibble of
the actor's sprite shape. Its tile lookup `$3f9e/$3fd8` reads the copied grid and
definitions, ignores background flip bits, and retains signed tile-source
offsets. The source is the ROM graphic bank, not an invented transparent color
or a screenshot of the composited scene. The original sprite builder suppresses
the actor palette bits while invisible.

`$3e86` maps both nibbles through the original sixteen-byte table at `$3edc`.
When both flags are set, Rockskin's graphic buffer takes precedence, while
Invisibility still suppresses the sprite palette bits. Both kernels and the
sprite attributes match original instructions. The host compositor continues
to rebuild visible frames on render; the entire original dirty-frame/DMA queue
and its resampling cadence are not claimed by this milestone.

The shared spell roll already applies the original Invisibility target penalty,
including its vision/sensor exceptions. The ordinary ranged-launch roll uses a
freshly cleared projectile context, so this milestone does not invent an
Invisibility penalty on that different path.

Rockskin has another counterintuitive source detail: `$55e62` checks A0's flag,
not A1's, on the represented spell-damage path. Thus a coated casting source
reduces that outgoing staged damage by two. A coated defender alone does not
receive invented protection there. The existing native rule is retained and
tested. Do not describe this as a newly implemented universal incoming-damage
reduction; the original instruction behavior is the acceptance criterion.

## JSON and verification

`data/combat/status-spells.json` uses typed Serde records with unknown-field
rejection. It records item/kind, initializer, effect/timer/graphics routine
pointers, flag/timer offsets, duration factor, phase mask and remap-table address.
Validation checks the exact recovered scope and original dispatch tables.
The existing schema-3 timed-effect catalog remains unchanged. Rust owns the
state transitions and tile transforms; no ROM assets are copied into JSON.

```sh
cargo build --release --locked
python tools/validate_status_spells.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The oracle compares 2,048 complete self/party casts through allocation,
initializer and drain, omitting only audio/HUD transfers. It checks every actor
byte, RNG, graphics flags and alert state. Another 4,096 status-clock phases
execute the original visible-actor dispatcher without stubs. Finally, 512 tile
transforms and sprite records match original graphics code without stubs across
maps 0, 5, 10, 21 and 31, including original decompression for tagged definitions.

Six integration tests cover learned casts, self/party targets, original graphics,
expiry and visibility holds, render stability, full-pool behavior, catalog
rejection, combat flag semantics and the real street encounter. Component
fixtures explicitly supply test opponents and state for boundaries; the street
test uses the actual map and encounter path. The verified Ghidra working copy
was reused; private views under `artifacts/status-spells-reference` remain ignored.
Windows GUI/input testing is still pending. This continuation remains local.

Final verification: 69 unit tests and all 77 ROM-backed integration tests pass.
The new status oracle and existing casting, Heal, companion-caster and physical
combat comparisons pass. Linux desktop and Windows release builds pass. The
524,046-byte Windows ZIP passes archive integrity and all 22 payload hashes;
its README/executable match the current build and its x86-64 PE/DLL imports are
verified. The ROM is excluded; the archive hash is in `releases/SHA256SUMS`.
No new publication was performed for this continuation.
