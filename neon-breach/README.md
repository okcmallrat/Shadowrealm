# NEON / BREACH — v0.1

A Rust arcade prototype for Windows PCs: fast Matrix arena combat leading into ICE-breaking boss battles. You are a freelance Decker uncovering HELIX's captive-mind project through six short contracts.

## Play on Windows

Extract the Windows ZIP to a writable folder and double-click `neon-breach.exe`. No Rust installation, launcher, network connection, or external game assets are required. Targets 64-bit Windows 10/11 with an OpenGL-capable graphics driver. The executable is unsigned.

## Controls

| Action | Control |
|---|---|
| Move | WASD |
| Aim and shoot | Arrow keys (eight directions), or hold left mouse |
| Shoot along last aim | Space |
| Dash | Left Shift (1.8-second cooldown) |
| Boss payload | X or right mouse |
| Pause/resume | P or Escape |
| Accept job / continue / restart | Enter |

Close the window to quit. Resize the window as needed.

## Matrix runs

Fight security programs in 23 combat rooms across six contracts. Walls block shots. Security comes in three speed variants and fires aimed projectiles. Eliminate every program to unlock the east gate, then move through it. Move constantly, shoot while moving, and dash out of dangerous corners.

Pickups are automatic on contact:

* **S:** two shield charges, up to six. Each charge absorbs a hit.
* **W:** faster fire, then a three-shot spread. Upgrades last the whole run.
* **E:** eight exploit energy, up to 36.
* **+:** two integrity, up to six.

Weapons, shields and exploit energy carry through rooms and into the boss arena. Completing a contract restores two integrity. Death ends the run; Enter restarts the campaign.

## ICE boss fights

Normal bullets destroy ICE but cannot damage the corporate core. Move close to ICE to chew it for twice the energy. Make a horizontal corridor through the wall, then align your height with the moving core. **X fires a payload from the left edge of the arena at your current height**, costing 12 energy. The payload must cross the arena and pass through your opening to hit the core. A payload hitting ICE is consumed. The cannon holds enough energy for three shots.

The central refuge protects you from ordinary enemy projectiles. The orange hunter can enter it, so staying still is dangerous. Core contact also hurts. Bosses gain health, speed and wider attacks across the campaign; later bosses add radial bursts. Every boss becomes more aggressive below half health. Depleted ICE replenishes a column so charging always remains possible.

## Build from source

Install current stable Rust from rustup.rs. On Windows, use the default MSVC toolchain and its Microsoft C++ build prerequisites.

```powershell
cargo test --locked
cargo build --locked --release
.\target\release\neon-breach.exe
```

Or run `build-windows.ps1`, which copies the executable and this guide into `dist`.

Dependencies: Macroquad (graphics/input), Serde and serde_json (campaign data). All art is drawn in code. The JSON campaign in `data/contracts.json` is embedded during compilation; edit it and rebuild to change the story, room counts and boss health. No audio is included in this prototype.

## Verification

Three Rust unit tests cover campaign structure, collision geometry and power-up persistence. `cargo run -- --verify` opens a graphics context and simulates all 23 rooms and six bosses, checks ICE blocking and core immunity to normal bullets, and reaches the ending. This test deliberately clears enemies and places payloads for deterministic transition checks; it is not a full player playthrough or a difficulty assessment.

The executable was cross-compiled for Windows x86-64 and inspected for imports: only Windows system DLLs are required. Graphical checks and campaign simulation ran on Linux with software OpenGL. Execution on a real Windows PC remains to be confirmed.

High scores are saved to `neon-breach-score.txt` in the working directory. Saving is best-effort; the game still runs if that directory is read-only.

## Scope

This is a first playable version: one linear campaign, arcade geometric graphics, short text briefings, six escalating core bosses, two alternating arena layouts, and randomized enemy positions. No audio, mid-run save, gamepad controls or difficulty selection yet.
