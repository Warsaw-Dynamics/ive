#!/usr/bin/env bash
# Checks that IVE can be built here: cargo, and on Linux the system libraries GPUI links against.
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not found: install Rust with 'asdf plugin add rust && asdf install' (version from .tool-versions)" >&2
    exit 1
fi

# macOS links against system frameworks only
if [ "$(uname -s)" != Linux ]; then
    exit 0
fi

# cargo's linker error does not name the package to install
missing=()
for lib in xkbcommon xkbcommon-x11 fontconfig freetype2 wayland-client xcb libpipewire-0.3 egl gbm; do
    pkg-config --exists "$lib" || missing+=("$lib")
done
if [ ${#missing[@]} -gt 0 ]; then
    echo "missing system libraries: ${missing[*]}" >&2
    echo "Debian/Ubuntu: sudo apt install libxkbcommon-dev libxkbcommon-x11-dev libfontconfig-dev libfreetype-dev libwayland-dev libxcb1-dev libpipewire-0.3-dev libegl-dev libgbm-dev libclang-dev" >&2
    exit 1
fi
