# Working on Shadowrealm

Read `docs/HANDOFF.md` first, then the linked implementation/validation notes.

- The user's goal is a complete native Rust Shadowrun reconstruction that runs
  as a Windows PC executable. An emulator wrapper, asset viewer, GIF, or exploration
  demo does not complete that goal. Report partial milestones honestly.
- Use the existing checkout. Cloud tasks are already isolated; do not create Git
  worktrees unless the user explicitly requests them. Preserve existing changes.
- Preserve the supplied ROM as an input. Do not commit ROMs, extracted game assets,
  dependency/toolchain caches, or generated frame directories. The small Windows
  exploration ZIP is intentionally versioned in `releases/` without the ROM.
- Recover behavior from actual instruction/data references. Keep confirmed facts,
  host implementation choices, and unimplemented gameplay distinct. Do not invent
  mechanics or replace missing systems with hard-coded demonstrations unnoticed.
- Use frozen Cargo dependencies (`--locked`) and keep TLS/checksum verification.
  Do not print secret values or replace injected Git authentication.
- Run relevant unit, functional, and original-routine differential checks. A Windows
  cross-build does not establish that Windows GUI/input has been playtested.
- Follow `scripts/build-windows.sh` and `docs/exploration-and-windows.md` for the
  validated Linux-to-Windows build. Use `x86_64-pc-windows-gnullvm` with LLVM-MinGW.
- Keep handoff/status notes current after substantive milestones. Do not silently
  overwrite the versioned Windows ZIP with a stale or unverified build.
- Introduce JSON gradually for recovered dialogue and quest logic, as requested
  by the user. Start with small verified records; keep the Rust engine and avoid
  broad migrations or speculative quest rules.
