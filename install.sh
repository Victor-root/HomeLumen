#!/usr/bin/env bash
# Registers HomeLumen with the desktop, so it gets a proper entry (and icon)
# in the applications menu and the taskbar. Only Linux needs this: Windows
# reads its icon and its entry straight off the .exe, both handled already
# at build time by crates/homelumen-app/build.rs.
set -euo pipefail
cd "$(dirname "$0")"

if [ ! -f target/release/homelumen ]; then
    echo "Compilation de HomeLumen (une seule fois)..."
    cargo build --release
fi

mkdir -p ~/.local/share/applications ~/.local/share/icons/hicolor/256x256/apps
cp assets/icon/icon.png ~/.local/share/icons/hicolor/256x256/apps/homelumen.png
sed "s|__EXEC__|$(pwd)/target/release/homelumen|" \
    packaging/linux/homelumen.desktop.in > ~/.local/share/applications/homelumen.desktop

command -v update-desktop-database > /dev/null &&
    (update-desktop-database ~/.local/share/applications || true)
command -v gtk-update-icon-cache > /dev/null &&
    (gtk-update-icon-cache ~/.local/share/icons/hicolor || true)

echo "Done. Look for \"Home Lumen\" in your applications menu."
