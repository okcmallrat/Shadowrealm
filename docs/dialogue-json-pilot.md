# Gradual JSON integration: Space Needle / Trent

The user asked to introduce JSON slowly while recovering dialogue and quest
logic. This is one small pilot, not a migration of the engine or a general quest
system. Rust still handles input, contact, rendering, state transitions and the
supported operations. `data/dialogue/space-needle.json` records the existing
entrance passage and a bounded Trent conversation.

## JSON boundary

The versioned file contains hexadecimal ROM references, choice destinations,
source action routines and two kinds of knowledge-bit effects. It contains no
copied dialogue text or graphics. Text and font still come from the user's ROM.
The file is embedded at compile time, so Windows users need no new sidecar files;
editing it currently requires rebuilding the executable.

`Catalog::load` rejects unsupported schema/scope, unknown fields/operations,
duplicates, invalid bit widths, dangling node links and references that differ
from the supplied ROM. It checks body/choice/destination pointers, the initial
conversation selector and action dispatch table. Original action execution is
checked by the differential oracle; pointer validation alone does not prove the
semantics of a newly added action. Add an action only after tracing and comparing
its state changes. Do not infer fees, mission conditions or new quest rules.

## Confirmed interaction path

Addresses below are hexadecimal. The fixed NPC from `17018` has interaction flag
bit 4 in entity byte `9e`. `eb92 → eccc` detects contact with the controlled runner.
In the unlinked case, the runner position must be within `[npc-0040,npc+0040)`
on both axes, in fixed-point units (four pixels). Inactive/dead NPCs, dead players
and non-controlled entities do not schedule this conversation. Selected-target
links use a wider original window; targeting is not implemented in this pilot.

`ecf0` schedules routine `21174` through `1e2e0/1e2ec` and sets a 60-tick scheduling
cooldown. On map 10 with entrance selector 6, `ed00..ed2a` select recruit index 2:
Trent. The dispatcher sets `fffc2a` bit 2, then chooses the first pointer from
the corresponding `fe640` record for the explicit zero-opinion fixture.
The resulting node is `feb4c`. Other opinion/history states are not reconstructed.
This is not an original new-game recruitment state.

Native contact runs before the movement candidate, matching the original update
order. It opens the conversation, sets the known-runner bit, freezes the world
and runner selection, and preserves the scheduling cooldown across the dialog.
The NPC remains idly animated when the world resumes; no AI dispatcher is added.

## Branches and recovered state changes

The JSON contains 12 node reference records. The hiring pitch is retained as
evidence, but is not offered as an enterable branch: its sole continuation calls
unimplemented hiring action 3. Both that action and the upstream hiring choice
are unavailable, leaving the conversation and world state unchanged.

Other branches cover Trent's biography, mages, Jefferson's contact information,
Vigore, switching topics and saying goodbye. Three translated action IDs are
supported by the bounded interpreter:

| Decimal ID | Original routine | Effect / next node |
| --- | --- | --- |
| 4 | `211e8` | For opinion zero, topic node `febf4` from `fe6e0` |
| 190 (`be`) | `2404c` | OR `fffc2a` bit 2; biography node `fec48` |
| 229 (`e5`) | `24964` | OR `fffc2c` bit 12; contact node `fecc4` |

`Game::knowledge.runner_known` and `.contacts` hold those recovered flag words.
They persist through area reloads for the running session and reset with R.
The Pocket Secretary UI, disk saves, recruiting/payment and quest progression
remain unimplemented; setting the contact flag does not implement those systems.
Action effects are committed only after the destination text/font is prepared.
Unknown actions leave the current page and flags intact.

`1f3c4` supplies the node layout: body, up to three option-text pointers, and
choice destinations beginning at node+16. A negative second pointer identifies
an exposition; this pilot accepts the traced `ffffffff` form. Original selection
logic `1f522..1f590` follows node pointers or dispatches IDs below 250 through
`12ccb4`. Native support is limited to the listed records/actions.

## Host presentation

Up/down selects an option; Enter/Z confirms it. Left/right pages long body text.
The original labels and text are preserved, with an asterisk and an unavailable
hint for the unsupported hiring choice. The panel, selection cursor and paging
are native host choices, not the original portrait/conversation screen. Core
input B/C also selects original slots 2/3 for replay/oracle checks; the desktop
uses the arrow/Enter controls. Saying goodbye displays its original closing
passage, then acknowledgement resumes exploration without changing maps.

## Evidence

```sh
cargo build --release --locked
cargo test --locked
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked --test transitions -- --ignored
PYTHONPATH=/workspace/.reverse-tools python3 tools/validate_dialogue.py /path/to/Shadowrun.bin
cargo run --release --locked -- dialogue-data /path/to/Shadowrun.bin artifacts/trent-dialogue
```

The oracle compares 12 body/choice reference records and decoded text, 24
original branch/action executions (including preserved pre-existing flag bits),
and 69 contact-boundary/entity-gating cases. It executes the original collision
handler up to its post-interaction point, before PRNG/avoidance, and the actual
dialogue selection/action code up to the next node. It also verifies the initial
node, known-runner bit and 60-tick scheduling cooldown. It does not emulate the
original conversation renderer, targeting, AI, payment or a full game session.

The new ROM-backed integration test enters through the real door, then seeds a
player just outside Trent's contact radius and walks into it. It checks frozen
movement/selection, unavailable hiring without state changes, biography and page
rendering, contact knowledge, topic switching, goodbye and the recontact cooldown.
The separate two-visit door test checks that knowledge flags survive reloads.
The suite has 42 passing unit tests and four ROM tests, run explicitly because
the supplied ROM is not in Git. These fixtures do not establish full navigation
from the new-game start, original AI behavior or Windows GUI playtesting.

## Next bounded step

Trace action 3 (`21226`) and its fee, party-capacity, opinion and recruitment
effects before enabling hiring. Add only the verified records/operations to this
pilot, with original-routine comparisons. Broader mission/dialogue variants,
targeting, companion behavior and a Pocket Secretary remain subsequent work.
