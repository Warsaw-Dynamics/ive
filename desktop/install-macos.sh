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

write_icon() {
    local iconset
    iconset="$(mktemp -d)/ive.iconset"
    mkdir "$iconset"
    cp icons/16.png "$iconset/icon_16x16.png"
    cp icons/32.png "$iconset/icon_16x16@2x.png"
    cp icons/32.png "$iconset/icon_32x32.png"
    cp icons/128.png "$iconset/icon_128x128.png"
    cp icons/256.png "$iconset/icon_128x128@2x.png"
    cp icons/256.png "$iconset/icon_256x256.png"
    iconutil -c icns -o "$app/Contents/Resources/ive.icns" "$iconset"
    rm -r "$(dirname "$iconset")"
}

write_bundle() {
    rm -rf "$app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    cat >"$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key><string>IVE</string>
    <key>CFBundleIdentifier</key><string>com.warsawdynamics.ive</string>
    <key>CFBundleName</key><string>IVE</string>
    <key>CFBundleDisplayName</key><string>IVE</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleIconFile</key><string>ive</string>
    <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
    printf '#!/bin/bash\nexec %q\n' "$1" >"$app/Contents/MacOS/IVE"
    chmod 755 "$app/Contents/MacOS/IVE"
}

write_bundle "$1"
write_icon
# the Dock and Finder otherwise keep the icon of an IVE.app they saw before
"$lsregister" -f "$app"
echo "installed $app running $1"
