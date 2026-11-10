SHADOWREALM — WINDOWS EXPLORATION BUILD

This is a native Rust reconstruction in progress, not the complete Shadowrun game.
It renders original assets and runs exploration with bounded gunfire, fists and spells.

RUN ON WINDOWS 10/11 (64-BIT)

1. Extract the ZIP to a normal folder. Do not run it from inside the ZIP.
2. Copy the Shadowrun.bin you supplied into that folder, beside Shadowrealm.exe.
3. Keep libunwind.dll beside Shadowrealm.exe.
4. Double-click Shadowrealm.exe.
5. Choose New Game, an archetype, then Begin; or Load Game for an existing slot.

No Rust, Python, web server, emulator, or separate game runtime is needed to play
this build. The game data is read from the ROM file, which is not bundled.
The supplied 2 MB Genesis ROM revision is the supported input.

CONTROLS

Arrow keys or WASD: move the selected runner.
F outside dialogue: attack with the equipped firearm, fists or a supported spell.
Firing uses finite ammo, cooldowns and automatic reloads while reserves remain.
Without a selected target, shots follow the runner's facing direction.
Tab: select the next runner (also selects its HUD panel).
J: review the current run, including pickup/delivery and payment.
I between actions: select an owned firearm; choose the equipped gun to unequip.
C on a city street: call a taxi (unlocked by collecting Michael's belongings).
T outside dialogue: toggle target selection; yellow brackets/name identify the NPC.
Red brackets/title status mean the projectile probe found a blocking path;
gray means geometry is not modeled. Yellow only identifies the selection.
Target selection does not fire, consume ammunition or apply damage.
Enter or Z: acknowledge the Space Needle entrance passage.
During dialogue: Up/Down chooses an option, Enter/Z selects it, Left/Right pages
long text.
Space: pause/resume simulation.
F5: save to Slot 1 or Slot 2 from a safe city street.
F9: load a native checkpoint from a safe city street.
R: open New Game / Load Game / Resume. Saved files remain available.
In explicit development mode, R resets the exploration session.
Escape: exit.
Resize the window to enlarge the view; aspect ratio is preserved.

OPTIONAL COMMAND LINE

Shadowrealm.exe "C:\Games\Shadowrun.bin" 5
Shadowrealm.exe --explore
Shadowrealm.exe --solo --nuyen 3000
Shadowrealm.exe --save-dir "C:\Games\ShadowrealmSaves"

Normal launch opens New Game / Load Game. Choose Samurai, Decker or Gator Shaman
and Begin to start Joshua alone in Redmond with 20 nuyen and original class gear.
--explore launches the Joshua/Ricky/Freya development roster directly.
An explicit map number (0..55), --solo or --nuyen also selects development mode.
Its default map is 2 (Redmond). --save-dir alone retains the opening menu.
Map 5 has a recovered environmental tile animation. Coordinates, room names, and transitions
are still under reconstruction; not every map is an independently playable room.
--solo starts the development roster with Joshua alone, leaving companion slots
free. --nuyen explicitly seeds development funds. Without it, fresh exploration
sessions use the original bootstrap's 20 nuyen. These options do not implement a
new-game creation flow. Courier jobs do earn money in this build. R resets to the
same chosen roster and starting funds; unsaved progress is discarded and manual
save files remain available through F9.

CURRENT BEHAVIOR

Normal new games use the original selected Joshua template and first Michael
investigation lead. The opening menus and orientation are host panels; original
title graphics, intro cinematics, boot timing and audio remain pending.

In --explore, ROM templates for Joshua, Ricky, and Freya supply names, portraits,
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
general inventory/menu interactions, other missions, audio, and original save UI remain
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
Mission-specific recovery, other runner recruitment, original save UI and broader companion
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
For Mana Blast, launch with --explore, Tab to Freya, M once from her starting weapon, T to target and
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
session and native saves. R opens the start menu during normal play.

A controller-only check walks from the normal map-2 spawn through the booth,
Stoker's pickup, Rat's Nest delivery and the next offer at tick 3356: 20 -> 60
nuyen plus Karma. It inserts no actors or forced job outcomes. Original courier
ambushes/robberies, other job types/Johnsons, wilderness travel, general shops, the
full Pocket Secretary UI, original intro/title graphics and original save UI remain pending.
The panel and J shortcut are host UI; Windows GUI/input playtesting is pending.

FIRST OBJECTIVE AND CITY TAXI TRAVEL

New Game starts Joshua alone in Redmond (map 2), with original class gear and
20 nuyen. --explore retains the prior three-runner development roster. Earn at least 250 nuyen through courier jobs,
visit Stoker's Coffin Motel and ask for Michael's belongings. Pay the back rent
and continue through the original stash passages. The recovered rules award
the hidden 500 nuyen, one Karma, cyberdeck ownership and the DocWagon wristband.
Cyberspace/deck equipment and further story objectives remain unfinished.

C on a city street opens Call Taxi. Choose a city district with Up/Down and
Enter/Z. Acknowledge the fare and arrival notice. Six city districts work;
the original traffic/story gate, same-district refusal, fares, insufficient-funds
free ride and armband free ride are retained. Salish/wilderness travel is guarded.
Jobs, party equipment, wallet, story and contract archives survive city trips.

One controller-only check completes six real courier jobs, reaches 280 nuyen,
buys the belongings (530 nuyen including the hidden credstick), then arrives
Downtown by taxi with 510 at tick 14746. It assigns no positions, stats, money,
RNG or job results. The route had no street encounter. Other shops, original menus,
audio, original intro/title sequence and the complete story flow remain unfinished.

C and the destination panel are host UI. Recovered behavior was compared with
original routines, and native rendered panels were inspected. This package is
cross-built; actual Windows GUI/input playtesting remains pending.

ARES WEAPONS AND AMMUNITION

Visit Ares Weapon Emporium in southeast Redmond, read the greeting and Browse.
Weapons lists its original stock; Equipment includes 25-nuyen ammo clips.
Up/Down and Enter/Z choose an item and confirm. Confirmation starts on No;
choose Yes to buy. Back/Leave close the shop. The currently selected runner buys
the item; use Tab before entry to shop for a companion. Matching courier pickup/
delivery service takes priority over ordinary shopping.

Firearms use the original prices/negotiation and first empty inventory slot.
Clips use the runner's shared reserve, up to the original 20-clip limit, and can
be bought even with full inventory. Buying a gun does not equip it automatically.
Use I, select the purchased gun and Enter/Z; Close returns to play. Selecting
the equipped firearm unequips it. F fires and reloads when reserves remain.

A connected check earns the money through six courier jobs and Michael's stash,
buys a 500-nuyen Warhawk and one clip from 530 nuyen, then equips it and fires
seven shots through a real reload. It ends with 5 nuyen and five loaded rounds.
No money, inventory, actor or job results are assigned during that run.

Only firearms and ammo purchases are enabled here. Asterisks mark unsupported
items/slots. Grenades, consumables, armor purchases/equipping, accessories,
selling, trading, other shops and the original full inventory UI remain pending.
The shop/I panels are host UI over native recovered rules. Actual Windows
GUI/input playtesting remains pending.

NATIVE SAVE / LOAD

F5 opens two save slots and F9 opens the load menu. Up/Down and Enter/Z choose.
Save/load on a safe city street when combat/actions have ended. Interiors,
conversations, enemies, transitions and unresolved party recovery are guarded.
R -> Load Game also restores a checkpoint directly, including after defeat.
Loading selects Joshua, as in the original load routine.

Saves preserve money, purchased gear/ammo, health/condition, skills/Karma,
contracts/companion archives, spellbooks, knowledge, story/DocWagon progress,
courier records/package flags and party positions. Temporary effects, animation,
targets and map clocks are rebuilt/reset through the original load model.

The game creates a saves folder beside the EXE. Keep it when updating the build.
The previous valid primary becomes a backup; a backup load is identified in the
notice. Invalid loads leave your current game intact. Failed writes report an
error and retain a usable previous save. --save-dir DIRECTORY chooses another
writable folder if needed. R/closing the game do not save automatically.

The native versioned file wraps the represented original save image and adds
host integrity/metadata checks. It is not an importer for arbitrary cartridge or
emulator saves. A separate-process check resumes the earned Warhawk and its
correct ammo, 5 nuyen and six completed jobs. A collected courier package can
still be delivered and paid once after loading. Full original menus/cinematics,
wilderness/script-specific saves and the wider unfinished game remain pending.
Actual Windows GUI/input and filesystem playtesting remains pending.


DOWNTOWN HOTEL REST AND KARMA

Take C: Call Taxi to Downtown, then visit Gates Undersound Hotel. Read the
original greeting, ask for a room and pay 75 nuyen. Continue through the payment
and rest notices to the room menu. Spend Karma, choose a runner, then Attributes,
Combat skills or Other skills. Select a stat to see the next value and cost.
Upgrade confirmation starts on Yes; choose No to cancel. Each runner spends
only their own Karma. Back returns to the room; Check out resumes the street.

Rest restores condition and a little health. Severe wounds still need medical
attention; a room does not fully heal everyone. Race limits, skill limits,
Sorcery eligibility and weapon specialization costs use the original rules.
There is one rest per paid stay. Return/pay again for another rest.

The hotel/runner/stat panels are host UI, with the world frozen. F5/F9 work
after checkout on a safe street. Upgrades and remaining Karma persist through
native save/load. A connected check earns all funds/Karma through six jobs and
Michael's stash, pays taxi/room, improves Joshua's Body and Ricky's Firearms,
and resumes both upgrades from disk with 435 nuyen. Other hotels, original hotel graphics/menus and the wider
unfinished game remain pending. Windows GUI/input playtesting is pending.


OPENING MENU AND ORIGINAL SOLO START

New Game offers Samurai, Decker and Gator Shaman. Review the original stats,
select Begin, then acknowledge the Redmond orientation. No companions are
awarded. Find Gunderson at the Jump House to earn nuyen/Karma and work toward
Michael's belongings at Stoker's. Existing native checkpoints still work.

R opens New Game / Load Game / Resume while retaining the running game. Back
cancels a new-game choice; Begin replaces unsaved progress. New Game and R do
not write/delete save files. Load Game works without first starting a street
session, and after defeat. Missing/corrupt saves retain any current game; backup
loads identify the backup. Held confirmation cannot skip the resulting notice.

The opening/class panels and orientation are host UI. Original intro/title
cinematics, audio and the wider unfinished story remain pending. --explore or
an explicit MAP/--solo/--nuyen retains direct development launch and R reset.
A fresh solo controller route completes six jobs, buys Michael's belongings,
and reloads from the opening menu with 525 nuyen and seven Karma. This also
fixes a missing Halloweener courier delivery continuation. Actual Windows
GUI/input/filesystem playtesting remains pending.


STOKER'S MOTEL AND UNARMED ATTACKERS

Stoker's rents a 20-nuyen room before or after Michael's belongings. Choose a
room, pay, acknowledge the rest notice, then spend Karma or check out. Rest
restores condition and only some health; severe wounds need medical care.
Matching courier pickups/deliveries retain priority over normal motel service.

After retrieving the belongings, the greeting's More rolls the original visit
encounter. Usually you reach the manager; sometimes a gunner and two unarmed
Halloweeners attack. Acknowledge the passage to resume the fight. T selects a
target; hold F to attack with a gun, fists or a supported spell. Unarmed human
NPCs approach and punch, using original defense/resistance, impact armor,
condition damage and attack timing. Melee weapons and special creature attacks
remain unfinished.

Zero condition can knock a runner out while health remains. Normal whole-party
defeat now supports medical recovery for all three Joshua archetypes, including
condition-only knockouts. Existing medical fees, companion removal/archive and
DocWagon rules still apply. R -> Load Game remains available for checkpoints.

A connected development-roster route earns the stash, survives the actual gang
fight, rents a room afterward, trains Joshua's Body, then reloads a disk save
with 410 nuyen and three Karma. No enemies, combat results, money or stats are
assigned during that route. Original room art/fades/service reload cadence and
wider story systems remain pending. Windows GUI/input/filesystem playtesting
remains pending.


PLAYER AND COMPANION FISTS

Press I between actions, then select the currently equipped firearm again to put
it away. Close inventory, select a target with T, walk into close range and hold F.
Selecting the owned firearm again restores it and converts stored ammunition back
to rounds. Fists do not spend ammunition. Normal fists damage condition; original
cyberware (including Joshua Samurai's hand razors) can instead damage health.
Swinging without a target or from too far away still uses the attack pose and
cooldown, but does not alert enemies or roll damage. Projectile obstruction
brackets do not apply to fists. Unarmed attacks still need the selected target.

Uncontrolled human companions now pursue, stop close and punch on the original
schedule. Companions with reserve clips reload; exhausting rounds and reserves
switches to fists. The recovered automatic spell choice still has priority for
magic users. Two Fists' ordinary unarmed combat also works in development rosters;
earning his recruitment is not part of this milestone.

A connected controller route earns six courier jobs and Michael's belongings,
puts Joshua's firearm away, makes 12 swings (two damaging hits) in the real Stoker
ambush, rests, trains Body to 5, then reloads a disk checkpoint with 410 nuyen,
three Karma and fists equipped. Companions use their normal supported combat.
No actor stats, enemies, money, RNG, positions or combat outcomes are assigned
after ordinary development-roster construction. Actual Windows GUI/input and
filesystem playtesting remains pending. Weapon melee, special creature attacks,
further story, cyberspace, recruitment, original full menus and audio remain
unfinished.
