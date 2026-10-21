#!/usr/bin/env bash
# Writes ~/Applications/IVE.app (macOS), which runs COMMAND, e.g. `desktop/install-macos.sh "$PWD/restart.sh"`.
# Finder, Launchpad, Spotlight and the Dock find it there, with IVE's icon. COMMAND must exec IVE in the end, so the
# running IVE stays the app's process.
set -euo pipefail
cd "$(dirname "$0")"

if [ $# -ne 1 ]; then
    echo "usage: desktop/install-macos.sh COMMAND; got $# arguments, quote a command with spaces" >&2
    exit 2
fi

app="$HOME/Applications/IVE.app"
lsregister=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister

launcher="$(mktemp)"
printf '#!/bin/bash\nexec %q\n' "$1" >"$launcher"
./macos-bundle.sh "$app" "$launcher"
rm "$launcher"
# the Dock and Finder otherwise keep the icon of an IVE.app they saw before
"$lsregister" -f "$app"
echo "installed $app running $1"
