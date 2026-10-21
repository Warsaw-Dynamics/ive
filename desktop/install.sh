#!/usr/bin/env bash
# Installs IVE's icon for the current user (Linux), so the task bar shows it for IVE's window (app id `ive`).
# With COMMAND, also adds a desktop entry "IVE" that runs it, e.g. `desktop/install.sh "$PWD/restart.sh"`.
set -euo pipefail
cd "$(dirname "$0")"

if [ $# -gt 1 ]; then
    echo "usage: desktop/install.sh [COMMAND]; got $# arguments, quote a command with spaces" >&2
    exit 2
fi

data="${XDG_DATA_HOME:-$HOME/.local/share}"
for icon in icons/*.png; do
    size="$(basename "$icon" .png)"
    install -D -m 644 "$icon" "$data/icons/hicolor/${size}x${size}/apps/ive.png"
done
# a missing cache only means the desktop finds the icon a little later
gtk-update-icon-cache -q -t "$data/icons/hicolor" 2>/dev/null || true

if [ $# -eq 1 ]; then
    install -D -m 644 /dev/stdin "$data/applications/ive.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=IVE
GenericName=Integrated Vibe Editor
Exec=$1
Icon=ive
Terminal=false
Categories=Development;IDE;
StartupWMClass=ive
EOF
    echo "installed the icon and $data/applications/ive.desktop running $1"
else
    echo "installed the icon; a desktop entry with Icon=ive and StartupWMClass=ive shows it for IVE"
fi
