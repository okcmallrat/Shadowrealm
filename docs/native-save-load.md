# Native save/load checkpoints

F5 opens the save menu and F9 opens the load menu. Choose Slot 1 or Slot 2 with
Up/Down and Enter/Z. Save on a safe city street after combat/actions finish;
interiors, conversations, transitions, enemies and unresolved party recovery are
guarded. If necessary, start/reset the default Redmond session, then F9 loads a
previous checkpoint. Loading restores Joshua's control, as in the original.

The game creates a `saves` folder beside the executable. Each slot has a primary
file and a backup of its previous valid save. A successful load from the backup
is identified in the notice. Failed loads retain the running game. Failed writes
report an error and retain a valid previous save. F5 is manual: closing the game
or pressing R does not save automatically. R resets the session and keeps files;
F9 can restore the last checkpoint.

For a different writable folder:

```text
Shadowrealm.exe --save-dir "C:\Games\ShadowrealmSaves"
```

Keep this folder when updating the executable. No save files, ROM or Ghidra
database are bundled in the release ZIP.

## Persistent state and original reconstruction

Checkpoints preserve party equipment, inventory, ammo, health/condition, skills,
Karma, contracts, inactive companion archives/opinions, money, spellbooks,
knowledge, Michael's belongings/DocWagon progress, courier records/package flags,
completed-job count, map, camera and party positions. A collected package can be
delivered and paid once after loading. A purchased gun can resume firing with its
stored magazine and reserve clips after restarting the process.

Original loading reconstructs actors and reloads map assets. Transient animation,
NPCs, projectiles, targets, temporary status effects, environment clocks and combat
metrics are reconstructed/reset rather than serialized. The native guards require
a quiet, living party so loading does not discard a live encounter or recovery.
This is a persistent checkpoint, not an exact mid-frame emulator state.

The user's Ghidra working copy supplied targeted views of `$efb4`, `$596c4`,
`$594e0`, `$59786`, `$5966e` and `$59798`. Private exports/captures/logs remain
under `/workspace/work/saves`. The supplied ROM/GZF are preserved. Native Rust
implements the rules; the original CPU is executed only by validation tools.

The original menu at `$efb4` checks interior and combat state. The save writer
at `$596c4` updates active compact archives, copies the global block, stores the
map/camera/party coordinates, calculates an additive checksum, then writes two
odd-byte SRAM copies for the selected slot. `$594e0/$5966e` restore globals,
initialize companions from their saved identities, unpack persistent records,
restore positions and select Joshua's control and class-dependent portrait.

The native file embeds the represented original 2,048-byte image:

| Offset | Bytes | Contents |
| --- | ---: | --- |
| `000` | 20 | Original header, including `GAME` at offset 4 |
| `014` | 962 | Thirteen 74-byte compact record spaces |
| `3d6` | 584 | Original `$fff9fa..$fffc41` persistent block |
| `61e` | 18 | Map, camera and three position pairs |
| `630` | 4 | Sum of bytes `014..62f` |
| `634` | 460 | Zero-filled native padding |

The engine represents eleven compact archives and its recovered global fields;
unimplemented original fields/extra record spaces begin at zero. This file is
for native checkpoints, not an importer for arbitrary cartridge SRAM or emulator
saves. Wilderness, mission-specific scripted encounters, original title/save UI,
motel rest and the wider game's unimplemented systems remain outside this scope.

`data/saves/references.json` is a small typed Serde reference catalog.
`src/saves.rs` implements image packing/checking, a versioned Serde/JSON envelope,
CRC32 and bounded slot I/O. The original additive checksum is retained; a separate
host CRC covers the image and native metadata. Metadata preserves the host tick,
shared RNG, original job review scratch/acceptance, completed-job count and hero
template identity. ROM length/product/Genesis checksum checks select the supported
revision; these checks are not a cryptographic ROM-identity proof.

`src/save_game.rs` stages reconstruction into a separate Game. It validates format,
version, both checksums, image size, party/job references, city/position/camera
bounds, spellbooks, inactive archives, HUD/sprite assets and a rendered frame
before replacing live state. It uses collision/host-camera bounds; descriptor
offsets 8/10 are not full map dimensions. Saved camera precision must fit the
native pixel camera. Invalid/incompatible input never partially updates a game.

Writing stages and syncs a temporary file in the save directory, retains the
previous loadable primary as `.bak`, then replaces the primary. An invalid primary
cannot overwrite a usable backup. File handles close before replacement; errors
clean up only the operation's own temporary files. Backup fallback fully validates
and constructs its Game before use. Loading does not automatically rewrite a
corrupt primary; the next successful save replaces it.

## Verification

```sh
cargo build --release --locked
python tools/validate_saves.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The new oracle compares 512 complete original save buffers and both odd-byte SRAM
copies, including compact overwrite order, all raw global bytes, map/camera/party
coordinates and checksums. The real original admission routine rejects corrupted
payloads. These writer/admission checks use no stubs. Another 512 complete
persistent-load cases compare companion initialization and compact records,
coordinates, actor flags, Joshua control and portrait selection. Only the
map/graphics/recovery loader `$3074` is omitted there; its native map/asset path
is independently covered by existing map and integration checks.

The controller-only earnings/Ares test now waits for a safe checkpoint, chooses a
slot, writes to disk and drops the old Game at tick 15373. A separate native CLI
process opens the file and confirms the six jobs, story flag, Warhawk, five loaded
rounds and 5 nuyen. A newly loaded Game fires that weapon and consumes a round.
No actor/stat/inventory, money, RNG, position, map or job results are assigned after
normal session construction. No encounter spawned in this route. A separate
explicit collected-job fixture verifies delivery/payment after loading and no
duplicate reward.

Additional tests cover two-slot independence, overwrite/backup fallback, corrupt
and semantically unsupported primaries, bad writes, cleanup, invalid versions/
checksums/bounds, no partial load, save restrictions, menu freezing and held-key
notice handling. The standard CRC vector passes. Rendered save and resumed HUD
panels were inspected. Actual Windows GUI/input and filesystem playtesting remains
pending; cross-builds do not establish that testing.

Final verification: all 73 unit/pure and 103 ROM-backed integration tests pass.
The existing medical/archive/rehire original-routine oracle also passes. Native
Linux desktop and Windows release builds pass. The refreshed Windows ZIP is
628,574 bytes; CRCs, all 22 payload hashes, README/executable identity, x64 PE
format and EXE/DLL imports are verified. SHA-256:
`357281f64060ba2fcddb37e03f8c8f7923d2522e5b2f38d193676d42de084331`.
It excludes ROM/GZF and all save files. Source/package changes remain local and
have not been published.
