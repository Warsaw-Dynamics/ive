#!/usr/bin/env bash
# Writes the macOS app bundle APP (e.g. IVE.app) with IVE's Info.plist and icon, running EXECUTABLE: a copy of it
# becomes Contents/MacOS/IVE. install-macos.sh passes a launcher script, the release build IVE's binary.
set -euo pipefail

if [ $# -ne 2 ]; then
    echo "usage: desktop/macos-bundle.sh APP EXECUTABLE; got $# arguments" >&2
    exit 2
fi
app="$1"
executable="$2"
icons="$(cd "$(dirname "$0")" && pwd)/icons"

write_icon() {
    local iconset
    iconset="$(mktemp -d)/ive.iconset"
    mkdir "$iconset"
    cp "$icons/16.png" "$iconset/icon_16x16.png"
    cp "$icons/32.png" "$iconset/icon_16x16@2x.png"
    cp "$icons/32.png" "$iconset/icon_32x32.png"
    cp "$icons/128.png" "$iconset/icon_128x128.png"
    cp "$icons/256.png" "$iconset/icon_128x128@2x.png"
    cp "$icons/256.png" "$iconset/icon_256x256.png"
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
    cp "$executable" "$app/Contents/MacOS/IVE"
    chmod 755 "$app/Contents/MacOS/IVE"
}

write_bundle
write_icon
