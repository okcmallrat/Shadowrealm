# Noncombat actor contact and separation

The native game now applies the physical contact response from `eb92/b49c` to
the party and the currently supported fixed NPC. Nearby actors can deflect the
player as well as companions. This is a reactive separation step after overlap,
not predictive path planning or complete NPC AI. The subsequent
[target-selection milestone](combat-targets.md) creates the verified target links.

The uploaded Ghidra reference was opened as a private working copy and the
program memory hash was checked against the supplied ROM. Targeted instruction
and decompiler views are under ignored `artifacts/avoidance-reference/ghidra`.
Actual 68000 execution supplies the differential oracle; untyped decompiled C
does not define the Rust behavior.

## Recovered behavior

`eb92` scans later actor slots, skipping inactive/dead targets. Unlinked contact
uses an unsigned 4-pixel window, lower inclusive and upper exclusive on each
axis. An existing `a2/ac` actor link widens that window to 16 pixels. The window
uses wrapped word bounds, rather than a toroidal distance calculation. Link
comparison is supported here; the subsequent target-selection milestone creates
those links without enabling attacks.

Contact sets byte `11` bit 1 on both actors and derives opposite separation
headings in word `92` from the actual quantized heading tables. Word `94` stores
the other actor's motion word `38`. The original random recurrence at `d44`
updates the session seed: `seed = seed * 20e5 + 1`, modulo 65536. When bits
`7000` are clear, the headings rotate again and the stored motion words double.
The native session starts with a deterministic zero seed; this does not recreate
hardware/input-dependent boot seeding or the random consumption of missing game
systems.

`aa88/aa92` consumes the pending bit and saves the normal motion handler to `d4`.
For one dispatch, `b49c` tries the ROM's fixed separation vectors. Odd frame phases
try the actor's normal desired heading first; even phases try its contact heading.
A blocked first candidate sets motion word `38` to 2 and tries a perpendicular
candidate. The third probe at `b504` updates collision samples but never commits
its position: the `b518/b51e` conditions discard it for every status, including
clear. This observed branch is preserved. The previous motion handler resumes;
the response does not turn the actor or select a new animation state.

`Game::update` now schedules, scans, and moves party actors in slot order. An early
actor sees later actors' old positions; later actors see earlier motion. Selecting
the middle or last party member preserves that ordering. Pending responses on
the fixed NPC are consumed after the party. Existing fade/warm-up position holds
and dialogue freezes remain; no background thread or render-call timing drives
the contact system. Frame alternation uses the native simulation tick, not a
recreated hardware clock.

## Scope and validation

Non-controlled actors still cannot cross area triggers. The final trigger sample
also records the original `ac5e/b7b8/baae` blocked-route flag. Controlled separation can now activate the already verified exterior seams
on maps 0/3/5 and the map 0 Space Needle entrance / map 10 return, with the
required saved return context. Other locations remain blocked. Normal directional
player movement continues to use those same bounded travel paths.

The contact-only oracle omits the `eccc` dialogue/quest callback to isolate
physical effects; it executes the real heading, RNG, response and static-collision
routines. The existing Trent contact/dialogue oracle separately checks the
implemented interaction. General contact scripts and combat remain absent.

```sh
export SHADOWREALM_TEST_ROM=/workspace/inputs/Shadowrun.bin
cargo test --locked
cargo build --release --locked
cargo test --locked --test avoidance --test followers --test transitions -- --ignored
/workspace/.venvs/shadowrealm/bin/python tools/validate_avoidance.py "$SHADOWREALM_TEST_ROM"
```

The new oracle compares all actor bytes and RNG state for 1,536 boundary/link/
eligibility/seed cases (438 actual contacts, 69 random adjustments), 1,024 response
steps near real collision boundaries on maps 0/3/5/10, and 2,700 scheduler/contact/
movement ticks with each of the three selected slots. It requires first, second
and third probe branches, including a clear third candidate that is discarded.

Functional tests drive overlapping parties apart with every selected slot and
verify clear footprints, unchanged health/equipment/wallet, and restored idle-NPC
behavior. The repeated Space Needle approach now uses four more rightward input
ticks to account for actual contact deflections; it still uses controller input
only and preserves the entry/return assertions. That checkpoint passed 49 unit
tests and 12 explicitly executed ROM integration tests. The earlier follower, world,
transition, interior, dialogue and hiring oracles also pass.

The Windows package is cross-built and inspected. Windows GUI/input playtesting
remains outstanding. These checks do not establish full-game navigation or combat.

## Contact-triggered travel

The follow-up port connects the final collision samples from `b49c` to the
physical `ac5e/b7b8` gate. It permits only the noncombat routes already supported
by `TransitionPlan`. Unsupported doors, special trigger scripts, combat and
mission-state conditions remain guarded.

Two negative footprint corners are required to request travel. Byte `b0` bit 5
latches a dispatched trigger; a raw zero collision releases it, including the
idle reload warm-up. One negative corner and non-controlled actors retain the
original blocked-route outcome. The maximum collision code supplies word `c6`,
but `bc36` takes orientation for word `cc` and return formation from the first
negative corner. Mixed-corner fixtures check that distinction.

The first or second accepted separation candidate commits its position before
the gate runs. The third candidate remains discarded, but its final samples can
still activate travel. Space Needle return formation therefore uses the actual
post-response position, which can differ from a blocked normal walking step's
pre-candidate position. Entering still shows the original passage and requires
acknowledgement; leaving still restores the saved exterior camera and formation.
No quest permission is inferred from an arbitrary trigger-to-map mapping.

The oracle now adds 3,936 motion/gate/latch cases against original execution,
including 2,038 location-dispatch requests and 55 requests from discarded third
probes. It stops at `1651e` before location scripts and omits only the `423c`
location-label/UI update. Actor bytes, eligibility, trigger orientation and Space
Needle return formations are compared. The existing separate route, entry-gate,
spawn and restore oracles validate the downstream supported paths.

Two added functional tests use real contacts with no movement input to activate
all three exterior seam maps (after selecting another leader), and to enter,
acknowledge and leave the Space Needle. The latter verifies trigger-latch release
and exact saved return context. Only boundary positions are seeded; these are
not full-game traversal recordings. The full checkpoint passes 51 unit tests and
14 explicitly enabled ROM integration tests. JSON dialogue/hiring remains typed
Serde data and unchanged by this physical travel work.
