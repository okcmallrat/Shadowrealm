SHADOWREALM — WINDOWS EXPLORATION BUILD

This is a native Rust reconstruction in progress, not the complete Shadowrun game.
It renders the original assets and runs exploration with bounded player gunfire.

RUN ON WINDOWS 10/11 (64-BIT)

1. Extract the ZIP to a normal folder. Do not run it from inside the ZIP.
2. Copy the Shadowrun.bin you supplied into that folder, beside Shadowrealm.exe.
3. Keep libunwind.dll beside Shadowrealm.exe.
4. Double-click Shadowrealm.exe.

No Rust, Python, web server, emulator, or separate game runtime is needed to play
this build. The game data is read from the ROM file, which is not bundled.
The supplied 2 MB Genesis ROM revision is the supported input.

CONTROLS

Arrow keys or WASD: move the selected runner.
F outside dialogue: hold to fire the equipped firearm; tap for one request.
Firing uses finite ammo, cooldowns and automatic reloads while reserves remain.
Without a selected target, shots follow the runner's facing direction.
Tab: select the next runner (also selects its HUD panel).
T outside dialogue: toggle target selection; yellow brackets/name identify the NPC.
Red brackets/title status mean the projectile probe found a blocking path;
gray means geometry is not modeled. Yellow only identifies the selection.
Target selection does not fire, consume ammunition or apply damage.
Enter or Z: acknowledge the Space Needle entrance passage.
During dialogue: Up/Down chooses an option, Enter/Z selects it, Left/Right pages
long text.
Space: pause/resume simulation.
R: reset this exploration session.
Escape: exit.
Resize the window to enlarge the view; aspect ratio is preserved.

OPTIONAL COMMAND LINE

Shadowrealm.exe "C:\Games\Shadowrun.bin" 5
Shadowrealm.exe --solo --nuyen 3000

The final number selects a map record (0..55); the default is 0. Map 5 has a
recovered environmental tile animation. Coordinates, room names, and transitions
are still under reconstruction; not every map is an independently playable room.
--solo starts the development roster with Joshua alone, leaving companion slots
free. --nuyen explicitly seeds development funds. Without it, fresh exploration
sessions use the original bootstrap's 20 nuyen. These options do not implement a
new-game/earning loop. R resets to the same chosen roster and starting funds.

CURRENT BEHAVIOR

Real ROM party templates for Joshua, Ricky, and Freya supply names, portraits,
health and equipment. This test roster does not represent a new-game recruitment
state or a saved game. Spawns come from the selected map descriptor. You control
one runner; other healthy party members follow in the recovered noncombat subset.
Tab changes the leader, with the old leader becoming a follower. Companions now
use the recovered wall-hugging fallback to escape tested U-shaped obstructions
and rejoin formation. Party and NPC contact now uses the original temporary
separation response; nearby actors can deflect the controlled player as well.
Those contact steps can activate the supported exterior seams or Space Needle
entry/return. The entrance still requires acknowledging its original passage.

Static map collision, turning, sprite animation, sprite depth sorting, camera
tracking, HUD portraits/equipment/meters, environmental tile animations, and the
recovered map 0 ↔ 3/5 exterior seams are connected. Those seams include fades and
party spawning. Map 0's Space Needle entrance displays its original text, waits
for acknowledgement, loads the bar and a fixed NPC, and restores the saved
exterior camera/party formation when leaving through the room's lower exit.
Walking into Trent's contact area opens a limited recovered conversation about
his background, magic and local contacts. The knowledge flags persist during the
session. Saying goodbye and acknowledging the closing passage resumes walking.
Trent's hire pitch checks capacity, quotes single-run/permanent prices, and checks
funds before charging and joining the party. At the current Joshua stats, the
prices are 285 and 2850 nuyen. A full party or insufficient funds causes no loss.
On success, his HUD panel appears, the room NPC disappears, and Tab can select
Trent to walk or leave the room. Money and the contract persist across area
reloads for this session. Single-run expiry requires the unfinished mission loop.
Companions use formation, distance/visibility-based speed, gradual turning,
regular corner routes, static wall sliding and recovered fallback wall hugging.
Unmodeled raster boundaries and general contact/travel scripts remain incomplete. Followers
cannot cross area triggers independently. Sharp turns can create lag; stopping
the leader gives them time to catch up. The room NPC is calm before combat is triggered.
The host camera follows the controlled runner; exact original
camera look-ahead behavior is not yet reconstructed.

NOT YET IMPLEMENTED

Other interior doors and story transitions still stop at their trigger. Full combat,
other conversations/quest scripts, other recruitment, contract completion,
special/companion combat AI, melee/magic/throwable actions, ongoing combat effects,
general contact scripts and other contact-triggered travel,
inventory/menu interactions, missions, audio, and save/load are not functional. No completion
claim is made for these systems. This ZIP is a development milestone.

Equipped firearms now use recovered ammo/reload/cooldown rules, original attack
and resistance dice, projectile collision, muzzle/impact sprites, damage and
hurt/death setup. There are no fabricated enemies or a training arena; the world
and NPC spawning are still incomplete. Neutral/protected NPCs retain the original
collision filters. Audio playback and the full defeat/game-over flow are absent.

VALIDATION

The gameplay core runs and renders in the Linux build environment. Collision
rasters and queries, map animation updates, sprite frames, and HUD asset routines
are checked against execution of original Motorola 68000 routines.

The Windows executable was cross-compiled and its PE format and DLL imports were
inspected. A Windows GUI session was not available here, so window creation and
keyboard interaction have not yet been verified on an actual Windows PC.

Ordinary firearm NPCs now acquire party targets, pursue into range and fire once
the recovered hostile/alarm conditions engage them. Alarm expiry clears combat
links. General encounter spawning, ambient patrol and special combat AI remain
incomplete; no enemies or training arena have been fabricated for this checkpoint.
