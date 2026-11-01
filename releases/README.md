# Windows exploration build

Download `Shadowrealm-windows-x64-exploration.zip` and extract it. Place your
`Shadowrun.bin` beside `Shadowrealm.exe`; keep the bundled `libunwind.dll` there.
Run the EXE on Windows 10/11 x64. The ROM is not bundled.

Controls: WASD/arrows move, Tab switches runner, Space pauses, R resets,
Escape exits. This is an unfinished exploration milestone: the map 0 ↔ 3/5 exterior seams work; interior/story transitions, combat,
dialogue/AI, inventory interactions, saves, and audio remain incomplete.
Companions now use recovered fallback wall hugging; four rotated U-shaped
pockets pass functional and original-routine comparisons. Party/NPC contact now
uses recovered temporary separation, including player deflections. General
contact scripts, contact-triggered travel and combat remain incomplete.
The executable was cross-compiled; Windows GUI playtesting remains outstanding.

The ZIP includes runtime licenses and per-file hashes. `SHA256SUMS` records the
archive hash. Build and validation details are in
[the project notes](../docs/exploration-and-windows.md).
