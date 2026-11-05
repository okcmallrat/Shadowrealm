SHADOWREALM — WINDOWS EXPLORATION BUILD

This is a native Rust reconstruction in progress, not the complete Shadowrun game.
It renders the original assets and runs exploration with bounded gunfire and spells.

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
F outside dialogue: fire the equipped firearm or cast a supported equipped spell.
Firing uses finite ammo, cooldowns and automatic reloads while reserves remain.
Without a selected target, shots follow the runner's facing direction.
Tab: select the next runner (also selects its HUD panel).
J: review the current run, including pickup/delivery and payment.
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
full companion combat AI, melee/remaining spell/throwable actions, ongoing combat effects,
general contact scripts and other contact-triggered travel,
general inventory/menu interactions, other missions, audio, and save/load remain
unfinished. No completion
claim is made for these systems. This ZIP is a development milestone.

Equipped firearms now use recovered ammo/reload/cooldown rules, original attack
and resistance dice, projectile collision, muzzle/impact sprites, damage and
hurt/death setup. There are no fabricated enemies or a training arena; general world
and scripted NPC spawning are still incomplete. Neutral/protected NPCs retain the original
collision filters. Audio playback and the full defeat/game-over flow are absent.

VALIDATION

The gameplay core runs and renders in the Linux build environment. Collision
rasters and queries, map animation updates, sprite frames, and HUD asset routines
are checked against execution of original Motorola 68000 routines.

The Windows executable was cross-compiled and its PE format and DLL imports were
inspected. A Windows GUI session was not available here, so window creation and
keyboard interaction have not yet been verified on an actual Windows PC.

Supported street firearm groups now spawn from the original district tables and
collision-grid placement rules, then approach and fire through native combat.
Try: Shadowrealm.exe Shadowrun.bin 5 --solo
With the host's deterministic initial seed and idle input, the solo map 5 session
spawns two attackers; other play changes the shared RNG. T selects, F fires and
R resets. Scripted encounters, creatures/magic, ambient patrol and the full
defeat/restart flow remain incomplete. Windows GUI playtesting remains pending.

Normal Joshua defeat recovery now supports companions: their equipment, stats
and contracts are archived in memory before removal, and Joshua returns through
Little Chiba. Rehiring Trent preserves those fields and uses his original
negative-opinion dialogue and fees. Insured recovery also implements both DocWagon
passages and the Seattle General arrival; insurance purchasing is still absent.
Mission-specific recovery, other runner recruitment, disk saves and companion
combat remain unfinished. Enter/Z acknowledges each recovery passage.

Uncontrolled mundane firearm companions now use original decision scheduling,
targeting, range goals, gunfire and finite-reserve reloads. Tab transfers control;
Joshua can fire as a companion when Ricky is selected. In a normal map-5 default
roster test with idle input, Joshua fired after a real encounter at tick 148.
Ricky/Freya now choose learned attack spells or Heal and fall back to inventory
firearms when the original drain gate rejects magic. These companion choices
need no M/G/F input. Melee and mission-aware combat remain unfinished.
Windows GUI/input playtesting is still pending.

Native spellcasting now supports Flame Dart/Bolt, Mana Zap/Blast and Heal with
original rolls, drain, sprites, timing, resistance/damage and recovery behavior.
For Mana Blast, Tab to Freya, M once from her starting weapon, T to target and
F to cast. Her original spellbook supplies force 3. M cycles actual learned
spells/forces; G restores an inventory weapon. For Heal, cycle M until Heal,
use P to choose a party member (including fallen active members), then F.
M/G/P are host keyboard shortcuts; original equipment/target menus, automatic
status-spell use, area/ongoing spells and full magic AI remain unfinished. Mana Zap is supported for its original learned books; the
default Samurai Joshua does not learn it. Windows GUI/input testing is pending.

Equipped attack-spell companions now use original combat positioning: blocked-path
recovery, offscreen pursuit, cooldown movement and formation when combat ends.
Tab to Freya, M once for learned force-3 Mana Blast, then Tab back to Joshua;
she can approach and cast without F input. Automatic attack/Heal equipment
selection is also connected. Heal first probes the enemy path, then redirects
to the first critically injured available party member. In a default-roster
map-5 check, idle input alone completes automatic magic at tick 157; both
companions cast and an enemy finishes defeat by tick 328. Special register
contexts and other spell actions remain unfinished.

Invisibility and Rockskin now cast with original status clocks and tile transforms.
Freya knows force-3 Invisibility; Ricky knows force-2 Rockskin. M selects the spell,
P cycles living party members, F casts, and G restores an inventory weapon without
removing the status. Original timing, allocation and combat quirks are preserved;
this does not implement general status-spell AI or all defensive behavior.

FIRST REPEATABLE JOB LOOP: GUNDERSON COURIER RUNS

Launch from the extracted folder:
Shadowrealm.exe Shadowrun.bin 2

This starts the development roster in Redmond. Enter the Jump House on the north
side, walk to the upper-left booth and ask Gunderson for work. His original
random offers and pay are retained. Courier offers can be accepted; ghoul bounty
and bodyguard objectives remain unavailable. Decline an unavailable offer, step
away and approach again. Read the route and payment, accept, leave the bar, pick
up the package and visit its recipient. J reopens the current briefing.

The recipient pays nuyen and awards Karma immediately. You can then return for
another offer. A carried package is lost on party defeat before medical recovery;
an uncollected run is retained, as in the original. Wrong locations, delivery
before pickup and repeat visits do not pay. Jobs and earnings persist in this
session; R starts a fresh session.

A controller-only check walks from the normal map-2 spawn through the booth,
Stoker's pickup, Rat's Nest delivery and the next offer at tick 3356: 20 -> 60
nuyen plus Karma. It inserts no actors or forced job outcomes. Original courier
ambushes/robberies, other job types/Johnsons, district travel, general shops, the
full Pocket Secretary UI, original new-game setup and disk saves remain pending.
The panel and J shortcut are host UI; Windows GUI/input playtesting is pending.
