SHADOWREALM — WINDOWS EXPLORATION BUILD

This is a native Rust reconstruction in progress, not the complete Shadowrun game.
It renders the original assets and runs a limited stateful exploration simulation.

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
Tab: select the next runner (also selects its HUD panel).
Enter or Z: acknowledge the Space Needle entrance passage.
Space: pause/resume simulation.
R: reset this exploration session.
Escape: exit.
Resize the window to enlarge the view; aspect ratio is preserved.

OPTIONAL COMMAND LINE

Shadowrealm.exe "C:\Games\Shadowrun.bin" 5

The final number selects a map record (0..55); the default is 0. Map 5 has a
recovered environmental tile animation. Coordinates, room names, and transitions
are still under reconstruction; not every map is an independently playable room.

CURRENT BEHAVIOR

Real ROM party templates for Joshua, Ricky, and Freya supply names, portraits,
health and equipment. This test roster does not represent a new-game recruitment
state or a saved game. Spawns come from the selected map descriptor. You can move
and switch runners independently. There is no follower AI yet.

Static map collision, turning, sprite animation, sprite depth sorting, camera
tracking, HUD portraits/equipment/meters, environmental tile animations, and the
recovered map 0 ↔ 3/5 exterior seams are connected. Those seams include fades and
party spawning. Map 0's Space Needle entrance displays its original text, waits
for acknowledgement, loads the bar and a fixed NPC, and restores the saved
exterior camera/party formation when leaving through the room's lower exit.
The NPC has idle animation; AI and interaction are not implemented.
The host camera follows the controlled runner; exact original
camera look-ahead behavior is not yet reconstructed.

NOT YET IMPLEMENTED

Other interior doors and story transitions still stop at their trigger. Combat,
branching conversations, NPC/companion AI,
inventory/menu interactions, missions, audio, and save/load are not functional. No completion
claim is made for these systems. This ZIP is a development milestone.

VALIDATION

The gameplay core runs and renders in the Linux build environment. Collision
rasters and queries, map animation updates, sprite frames, and HUD asset routines
are checked against execution of original Motorola 68000 routines.

The Windows executable was cross-compiled and its PE format and DLL imports were
inspected. A Windows GUI session was not available here, so window creation and
keyboard interaction have not yet been verified on an actual Windows PC.
