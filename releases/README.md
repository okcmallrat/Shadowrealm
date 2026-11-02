# Windows exploration build

Download `Shadowrealm-windows-x64-exploration.zip` and extract it. Place your
`Shadowrun.bin` beside `Shadowrealm.exe`; keep the bundled `libunwind.dll` there.
Run the EXE on Windows 10/11 x64. The ROM is not bundled.

Controls: WASD/arrows move, F fires, T targets, Tab switches runner, Space pauses, R resets,
Escape exits. This is an unfinished exploration milestone: the map 0 ↔ 3/5 exterior seams work; interior/story transitions, combat,
dialogue/AI, inventory interactions, saves, and audio remain incomplete.
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
Melee, magic, special/companion combat AI, ongoing effects, full game-over flow,
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
Mission-specific recovery, other runner recruitment, disk saves and companion
combat remain unfinished. Enter/Z acknowledges each recovery passage.
