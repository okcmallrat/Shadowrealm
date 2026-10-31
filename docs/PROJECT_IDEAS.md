# Enhancement ideas

The first priority is completing faithful gameplay and verifying a Windows build.
These ideas are options for later discussion, not commitments or implemented features.

## Highest-value next work

1. **One connected playable route.** Walk out of a room, enter the street, speak
   to an NPC, return, and save. This tests the game's systems together and makes
   progress more useful than a larger collection of disconnected renderers.
2. **Windows playtest notes.** Keep a short record of launch errors, controls,
   visual differences, and crashes, with screenshots where useful. Validate
   keyboard focus, resizing, pause/resume and multiple Windows display scales.
3. **Repeatable gameplay scenarios.** Expand the existing input replays to cover
   doors, dialogue choices, a fight, equipping an item, and save/load. Compare
   state and selected frames with original behavior.
4. **Readable system status.** Maintain a completion checklist with concrete
   acceptance tests. Distinguish recovered assets from functional mechanics.

## Optional player improvements after fidelity

| Idea | Benefit | Constraint |
| --- | --- | --- |
| Rebindable keys and controller support | Comfortable Windows play | Preserve original action mappings as defaults |
| Integer scaling and borderless fullscreen | Clear pixel art on modern displays | Keep aspect ratio and original-resolution rendering |
| Separate music/effects volume controls | Practical modern audio controls | Reconstruct authentic audio before changing its mix |
| Multiple manual save slots and optional quicksave | Easier session management | Validate original state and version saves before extending them |
| Optional larger dialogue text | Accessibility on high-resolution monitors | Retain original text/layout mode and all dialogue content |
| Contrast and flashing-effect options | Broader accessibility | Make changes optional and document departures from original visuals |
| Optional control reminders | Easier onboarding | Avoid cluttering the original HUD |

## Development improvements

- A map/trigger inspector that shows decoded room links and unresolved scripts.
- An entity inspector for party state, animation, equipment and script dispatch.
- Automated Windows builds with attached package checksums once packaging and
  runtime behavior are verified; keep ROM-dependent checks private/local.
- A reproducible release checklist including asset revision, runtime DLLs,
  licenses, fresh-folder launch, and Windows playtesting.
- Lightweight frame-time profiling before choosing rendering optimizations.

Avoid redesigning game balance, quests, progression or core combat during the
reconstruction. First recover the original rules; put any later changes behind
an explicit optional mode so fidelity and enhancements can be assessed separately.
