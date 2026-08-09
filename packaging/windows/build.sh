#!/usr/bin/env bash
# Builds HomeLumen for Windows and packages it into an installer, entirely
# from Linux: cross-compiles with the mingw-w64 toolchain, then packages
# with NSIS, neither of which needs an actual Windows machine to run.
#
# One-time setup, Debian/Ubuntu:
#   sudo apt install mingw-w64 nsis
#   rustup target add x86_64-pc-windows-gnu
set -euo pipefail
cd "$(dirname "$0")/../.."

export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
cargo build --release --target x86_64-pc-windows-gnu -p homelumen-app

mkdir -p target/installer
makensis packaging/windows/homelumen.nsi

echo "Installer: target/installer/HomeLumen-Setup.exe"
