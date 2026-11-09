# New game and opening-menu loading

Launching the desktop normally now opens **New Game / Load Game**. New Game
lets you choose Joshua's Samurai, Decker or Gator Shaman archetype, review the
original attributes, and select Begin. Each starts alone in Redmond with the
original inventory, equipment, skills, spellbook identity, 20 nuyen and first
Michael investigation lead. No companions are granted. A brief host orientation
introduces the currently connected route through Gunderson and Stoker's.

Load Game reads either native slot directly, without constructing a temporary
street game. The existing fully validated primary/backup loader is reused.
Successful backup loads identify the backup; missing/corrupt/incompatible slots
show a notice and leave any current game intact. Old native checkpoints remain
compatible, including development-roster saves.

R opens this menu during normal play, retaining the suspended game. Resume
returns to it. Back from class selection/preview cancels; Begin explicitly
replaces unsaved progress after the preview warns of that consequence. Neither
New Game nor R writes, overwrites or deletes checkpoint files. After defeat,
R -> Load Game restores a checkpoint without resetting into a temporary session.
A held confirmation cannot dismiss the new-game or loaded-save notice. The
opening menu freezes world ticks, actors, jobs, effects and RNG.

## Development mode

`--explore` launches the prior map-2 development roster (Joshua, Ricky, Freya).
An explicit MAP argument, `--solo` or `--nuyen` also selects development mode,
preserving previous command-line behavior. `--save-dir` alone still opens the
normal menu. In development mode R retains the prior reset behavior.

```text
Shadowrealm.exe
Shadowrealm.exe Shadowrun.bin
Shadowrealm.exe --save-dir "C:\Games\ShadowrealmSaves"
Shadowrealm.exe --explore
Shadowrealm.exe Shadowrun.bin 5 --solo
```

`Game::load` remains the explicit development/fixture constructor. It is not
silently redefined to initialize story flags or create an earned party.
`Game::new_game` is the source-backed solo constructor. `Session` owns the
optional live Game and opening menu; there is no placeholder world at launch.
Menu selection prepares the replacement's assets and rendered frame before
committing. Input is handed over only after held confirmation is released.

## Original references and scope

The supplied Ghidra working copy and ROM identify:

| Reference | Native use |
| --- | --- |
| `$59092`, `$1b6498` | Original New Game / Load Game options |
| `$59248..$59316`, `$1b64cf` | Three archetypes and selected template |
| `$42b0`, `$7291a + class*256` | Complete hero template copy / actor slot 0 |
| `$59318`, `$597ee` | Initial compact archives and absent companion IDs |
| `$7fc..$868` | Money, opinions/story/knowledge flags, initial lead 1 and map 2 |
| `$3084`, `$2b34`, `$a7b4` | Map descriptor/grids, party spawn and selected actor |
| `$1e68c` | Inactive job word and cleared job flags |
| `$566a2` | All five original session spellbooks |
| `$38fa` | Initial health/condition/ammo HUD normalization |
| `$1b6518/$1b6527/$1b6577` | Original slot/missing-save/success text references |

`data/newgame/references.json` is a small typed Serde/JSON reference catalog.
`src/newgame.rs` validates it and supplies class previews; `src/game.rs` builds
the solo state. `$59318` initializes the Samurai archive **before** selecting an
archetype. Native initialization retains that initial archive for all classes;
original-style saving snapshots the live chosen hero over it later. Eleven
represented archives and five spellbooks are preserved; extra original archive
spaces and unrepresented global fields remain outside the native engine's scope.

The original class templates, not host bonuses, supply starting stats and gear.
Health/condition/ammo normalize through the already recovered initial HUD rules.
The initial Michael lead is 1; this differs from the older development session's
zero story flags. Runners, contracts, money, jobs and story from an old running
Game do not carry into a newly confirmed Game.

These are host opening/preview/notice panels and an orientation, not recovered
original title graphics or intro cinematics. Original input cadence, palette
transfers, cinematic events/audio, boot RNG seeding and 31-tick warm-up are not
reconstructed. The native deterministic RNG and existing movement setup remain.
Decker creation does not implement cyberspace; Shaman creation does not add
unsupported spells. Wider story, recruitment, services, original full menus and
audio remain unfinished. Windows GUI/input/filesystem playtesting is pending.

## Connected regression and validation

A controller-only test now selects Samurai through the same Session as the
desktop, acknowledges the briefing, and completes six original courier runs.
It reaches 275 nuyen, pays Michael's 250-nuyen back rent, retrieves the 500-nuyen
credstick and retains 525 nuyen / seven Karma. It writes a checkpoint at tick
15424, drops Game/Session, constructs a new opening menu with no live Game,
and loads Slot 1. The solo roster, earnings, completed jobs and story/deck flags
survive. No actor/stat/inventory, money, map/position, RNG, enemies or job results
are assigned after normal menu construction. No encounter spawned in the trace.

That solo RNG/input path exposed an existing Halloweener courier delivery dead
end: `$110d72`'s MORE pointed to missing terminal `$111566` (text `$117868`).
The verified node is now present, and enabled courier stop continuations must
resolve within the bounded catalog. The delivery can finish and close normally;
payment is unchanged. The controller helper now bounds passage dismissal and
ignores obsolete A* queue entries so such failures report instead of spinning.

`tools/validate_newgame.py` executes all three original class-selection and
represented initialization paths. It compares every persistent hero byte,
eleven archives, five spellbooks, absent companion identities, money, story,
knowledge, job fields, map/camera and spawn. Only HUD tile transfer is omitted;
the actual resource normalization executes. Cinematics/menu rendering/warm-up
are outside that comparison. Private Ghidra exports, logs and rendered captures
are `/workspace/work/newgame`, outside Git.

Functional tests also cover every archetype's movement and disk round trip,
world freeze, preview/back/resume, failed new-game assets, missing saves, backup
loading, load after defeat, file preservation and held-key handoff. Opening,
class previews, briefing and loaded-save panels were rendered and inspected.


All 75 unit/pure and 113 ROM-backed tests pass (188 total, with the desktop
feature enabled). The new-game, courier, Stoker/taxi and native save/load
original-routine comparisons pass. Native Linux desktop and Windows release
builds pass. A native Linux GUI launch was attempted and failed to create a
window in this environment; actual Windows GUI/input/filesystem playtesting
remains pending.

The refreshed Windows ZIP is 669,648 bytes. CRCs, all 22 payload hashes, exact
membership, source README/EXE identity and x64 EXE/DLL headers/imports were
verified before replacing the local release ZIP. SHA-256:
`cc347699c425580f11c57861954fa5cd6e1f4b0e19739c7fe36bacf8bd1fd265`.
No ROM/GZF, saves or generated frames are packaged. Source/package changes
remain local and have not been published.
