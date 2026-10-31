# Trent first-hire milestone

The Space Needle conversation now supports its recovered hiring path. This adds
one bounded recruitment flow; other runners, job completion, dismissal, rehire
history, companion AI and the original new-game progression remain unfinished.
Addresses below are hexadecimal unless explicitly marked decimal.

## Recovered behavior

The hire pitch `fecfc` leads to action 3 at `21226`. If both companion entities
have active bit 0 set, it returns the original full-party reply `ff988` without
quoting, charging money or removing the NPC. Otherwise it draws the quote text
at `ff9bd`, computes both prices and opens an options-only record at `ff958`.
This record has three option pointers followed by three destinations; it has no
body-pointer slot. The native offer uses that layout explicitly rather than
pretending it is an ordinary conversation node.

Trent's fee words at `fe664` are 150 and 1,500. `2127c..212ca` applies the opinion
multiplier at `12ca54`, then an integer discount from leader bytes `7a`, `8b`
and `8c`. The byte additions wrap before shifting right by two. The discount
uses five times that result times the undiscounted single-run price divided by
100 (integer truncation). The permanent price subtracts ten times that discount.
Native code preserves the byte width, truncation and wrapping arithmetic.

With opinion zero and the current Joshua template, the prices are 285 nuyen for
one run and 2,850 for permanent service. The runtime conversation still uses the
explicit zero-opinion scope. Fee arithmetic is checked for all original opinion
values -1 through 4; that does not implement their other dialogue variants.

| Choice | Action ID | Routine | Contract byte `8f` |
| --- | --- | --- | --- |
| Single run | 0 | `21326` | `00` |
| Permanent | 1 | `21332` | `ff` |
| Talk later | next node | `ff970` | unchanged |

Action 2, the separate free permanent-service path, remains unsupported. No
untraced operation is substituted for it.

Payment at `2133c` uses signed long comparison, and subtraction wraps as on the
68000. Insufficient funds return `ff978` without changing money, roster or NPC.
On success, `42e6/4350` choose the first free companion slot, install Trent's
template and reload compact state through `5966e`; `21364..21386` set health and
secondary condition to 10. `21394..213a2` deactivate the room NPC and copy its
position to the new companion. The success reply is `ff980`.

Native code prepares the member, success dialogue, HUD and scene graphics before
committing the payment/roster/NPC change. The scene is cloned so map animation,
VRAM and palette state survive this preparation. The existing party's bytes and
selected runner are preserved. The first-hire member uses the original template
and identity/graphics slots; previous dismissal/equipment history is not modeled.
The original contract byte is stored, but automatic single-run expiry awaits
the mission/job-completion implementation.

## Serde / JSON increment

The existing file `data/dialogue/space-needle.json` gains three outcome node
references, the quote/option references, fee-table references and three hiring
action sources. Serde `Deserialize` structures still define the schema, with
`serde_json` parsing and unknown-field rejection. Rust executes the contextual
operations; JSON does not execute arbitrary ROM code. The loader compares the
new references, action dispatch pointers and offer choices with the ROM.
The required hiring record advances the embedded schema to version 2; the loader
rejects other versions rather than interpreting old records as hiring-enabled.

`Dialogue::follow` remains a text/knowledge-only operation. It does not purchase
anything when it encounters a hiring action. `Game` supplies the frozen quote,
wallet, current party and actual NPC context. A quote is consumed after purchase
or leaving the offer, so it cannot silently become a repeated charge.

The host panel inserts the computed prices into the original quote's placeholders
and shows the current wallet. Arrow selection/paging and the wallet heading are
native presentation, not a pixel-perfect copy of the original portrait UI.

## Running the development build

Fresh exploration sessions now read the 20-nuyen bootstrap value from the original
instruction at `7fc`. This does not implement the original new-game start (which
uses map 2), character creation or an earning/mission loop. The default desktop
roster remains the existing Joshua/Ricky/Freya test roster, so it correctly
receives the full-party refusal.

For testing an actual hire in the Windows package:

```text
Shadowrealm.exe --solo --nuyen 3000
```

`--solo` starts the exploration fixture with Joshua alone. `--nuyen` explicitly
seeds test funds, rather than pretending they were earned. The ROM is still
located beside the EXE when its path is omitted. Optional ROM path/map arguments
remain supported. R restarts with the same chosen fixture/funds. No extra JSON
files or runtime installations are needed on the user's PC.

Tab can select newly hired Trent. This milestone also fixes control transfer:
`3992/39a8` clear the old controlled-player flag and set the new one. Previously
the native selection only changed an index/HUD, so switched runners could walk
but could not satisfy the door's controlled-player check. The hired runner can
now leave through the interior exit, retain the contract and wallet on reload,
and remain recruited on the next visit (the room NPC is suppressed).

## Verification and limits

```sh
cargo test --locked
cargo build --release --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test transitions -- --ignored
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_hiring.py /path/to/Shadowrun.bin
cargo run --release --locked -- hiring-data /path/to/Shadowrun.bin artifacts/hiring
```

The hiring oracle compares 54 fee quotes (including byte-overflow/truncation
cases) and 24 full-party/funds-boundary/contract cases. Native fixtures use the
real door/contact/dialogue state machine, with an explicit near-NPC position
seed. The original executes the quote, payment, slot allocation, compact snapshot
reload, health/condition writes, NPC removal and placement. It compares wallet,
recruit IDs and full party records. UI/VDP-only calls are skipped; original NPC
AI, rendering timing and rehire/save history are not claimed. The native member's
existing host animation rate is adjusted explicitly in the byte comparison.

There are 43 passing unit tests and six explicitly enabled ROM integration tests.
The new cases cover both contracts, exact/insufficient funds, full party,
cancellation, preserved party state, no repeated charge while holding confirm,
Trent control/exit/revisit, and an injected invalid portrait pointer proving an
asset failure cannot deduct money or remove the NPC. Existing dialogue, seam,
map/collision and HUD comparisons remain passing. Generated frames stay ignored.

The Windows executable cross-builds and its archive/imports are checked. An actual
Windows GUI playtest is still pending. Full-game functionality is not complete.
Next trace the hired companion's targeting/follower dispatch, then connect the
job/mission logic needed for earning money and resolving single-run contracts.
