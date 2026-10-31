#!/usr/bin/env bash
set -euo pipefail
# Linux cross-build. LLVM_MINGW_ROOT must point to an official verified LLVM-MinGW
# UCRT archive extraction (this workspace uses release 20261020).
: "${LLVM_MINGW_ROOT:?Set LLVM_MINGW_ROOT to the LLVM-MinGW UCRT directory}"
project_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
export CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER="$LLVM_MINGW_ROOT/bin/x86_64-w64-mingw32-clang"
test -x "$CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER"
cd "$project_root"
cargo build --release --locked --features desktop --bin shadowrealm-game --target x86_64-pc-windows-gnullvm
python3 tools/package_windows.py "$LLVM_MINGW_ROOT"
