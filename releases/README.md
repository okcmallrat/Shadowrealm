# Windows exploration build

Download `Shadowrealm-windows-x64-exploration.zip` and extract it. Place your
`Shadowrun.bin` beside `Shadowrealm.exe`; keep the bundled `libunwind.dll` there.
Run the EXE on Windows 10/11 x64. The ROM is not bundled.

Normal launch now opens **New Game / Load Game**. Choose Samurai, Decker or
Gator Shaman to start Joshua alone in Redmond with original gear and 20 nuyen.
R opens that menu during play; Resume retains the suspended game. `--explore`
launches the previous three-runner development roster. An explicit map, `--solo`
or `--nuyen` also keeps direct development launch/reset behavior. See
[new-game scope and validation](../docs/new-game-and-opening-menu.md).

Controls: WASD/arrows move, F fires, T targets, Tab switches runner, Space pauses, R opens the start menu,
Escape exits. This is an unfinished exploration milestone: the map 0 ↔ 3/5 exterior seams work; interior/story transitions, combat,
dialogue/AI, inventory interactions, original menus, and audio remain incomplete.
Companions now use recovered fallback wall hugging; four rotated U-shaped
pockets pass functional and original-routine comparisons. Party/NPC contact now
uses recovered temporary separation, including player deflections. General
contact scripts, other travel paths and combat remain incomplete. Contact steps
can now activate the verified exterior seams and Space Needle entry/return.
T toggles target selection outside dialogue; targets can include neutral NPCs.
A blocked projectile probe shows red brackets and a title status; yellow is
selection feedback. Hold F to fire the equipped firearm, with or without a target.
Ammo, finite-reserve reloads, cooldowns, flashes/impacts, burst cycling, hit/resistance
rolls, physical damage and hurt/death setup are connected to the native game loop.
Melee, other spell actions, full companion combat AI, ongoing effects, full game-over flow,
audio and general missions remain unfinished.
The executable was cross-compiled; Windows GUI playtesting remains outstanding.

The ZIP includes runtime licenses and per-file hashes. `SHA256SUMS` records the
archive hash. Build and validation details are in
[the project notes](../docs/exploration-and-windows.md).

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
For Mana Blast, launch with `--explore`, Tab to Freya, M once from her starting weapon, T to target and
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

The first repeatable job loop now supports Gunderson's Redmond courier runs:
launch `Shadowrealm.exe Shadowrun.bin 2`, enter the Jump House on the north side
and approach the upper-left booth. Accept a courier offer, collect its package,
deliver it for nuyen/Karma, then return for another offer. J reviews the run.
Other offered job types are unavailable; decline and reapproach for another
original random offer. Package loss on defeat and one-time reward checks are
connected. This is a bounded courier loop; original ambushes, other jobs, shops,
wilderness travel, further story progression and original save UI remain unfinished. See
[the courier notes](../docs/courier-job-loop.md).

The default launch now starts in Redmond. Courier earnings can pay the 250-nuyen
back rent at Stoker's to recover Michael's belongings, the hidden 500 nuyen,
Karma and the DocWagon wristband. This unlocks **C: Call Taxi** on city streets,
with six city districts, original fares/refusals and free-ride rules. The tested
route completes six jobs, retrieves the stash and arrives Downtown with 510
nuyen, using ordinary controls and no game-state assignments. Original intro/title cinematics, broader story, wilderness travel, other shops, original menus and
audio remain unfinished. See [objective and travel notes](../docs/stoker-and-taxi.md).

Redmond Ares now sells firearms and ammunition. Select merchandise with Up/Down
and Enter/Z; confirmation defaults to No. Press I between actions to equip an
owned gun, then F fires and reloads from reserves. The connected earnings test
buys a Warhawk and ammo and fires through a reload. Other shops, armor/services,
selling and full inventory actions remain unfinished. See
[equipment scope and checks](../docs/ares-equipment.md).

**F5 saves / F9 loads** one of two native checkpoints from a safe city street.
Money, gear/ammo, companion archives, spellbooks, courier/package and story state,
map/camera/positions survive closing the game. Files live in `saves/` beside the
EXE; keep that folder when updating. Previous valid saves have a backup and failed
loads retain the running game. R opens the start menu without deleting files. `--save-dir`
chooses another writable folder. See [save/load checks and limits](../docs/native-save-load.md).
Original save UI, arbitrary SRAM import and wilderness/script-specific saves are
unfinished. Windows GUI/input/filesystem playtesting remains pending.


Downtown's Gates Undersound Hotel now rents a 75-nuyen room for partial rest
and Karma spending. Choose a runner and stat, review its cost, confirm, then
check out before F5 saves. Earned-route tests improve Joshua and Ricky and
reload both upgrades from disk with 435 nuyen. The room/stat panels are host UI;
other hotels and original hotel graphics remain unfinished. See
[hotel rules and checks](../docs/hotel-and-karma.md).


Stoker's now rents a **20-nuyen room** and executes the actual post-objective
Halloweener encounter. Its two unarmed attackers pursue and punch; condition-only
knockouts and normal medical recovery work for all three Joshua classes. An
earned route survives the fight, rests, raises Body and reloads with 410 nuyen
and three Karma. Player/companion punches and melee weapons remain unfinished.
See [Stoker combat and checks](../docs/stoker-ambush-and-rest.md).
