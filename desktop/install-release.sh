#!/usr/bin/env bash
# Installs the latest IVE release for the current user, or updates it:
#   curl -fsSL https://raw.githubusercontent.com/Warsaw-Dynamics/ive/release/desktop/install-release.sh | bash
# Linux: IVE in ~/.local/share/ive, the command ~/.local/bin/ive, and an "IVE" desktop entry with IVE's icon.
# macOS: ~/Applications/IVE.app and the command ~/.local/bin/ive.
# IVE updates itself from there; running this again also brings it to the latest release.
set -euo pipefail

repo="https://github.com/Warsaw-Dynamics/ive"
bin_dir="$HOME/.local/bin"
data="${XDG_DATA_HOME:-$HOME/.local/share}"

fail() {
    echo "install-release.sh: $1" >&2
    exit 1
}

# the asset name of this machine's release, without the commit and extension: ive-<os>-<arch>
asset_prefix() {
    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64) echo ive-linux-x86_64 ;;
        Darwin-arm64) echo ive-macos-arm64 ;;
        *) fail "no IVE release for $(uname -s) $(uname -m); build it from source, see $repo" ;;
    esac
}

# the latest release's tag, ive-<version>-<commit>, from where github.com/.../releases/latest redirects
latest_tag() {
    local url
    url="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "$repo/releases/latest")"
    case "$url" in
        */tag/ive-*) echo "${url##*/tag/}" ;;
        *) fail "found no IVE release at $repo/releases/latest (it led to $url)" ;;
    esac
}

sha256_of() {
    if command -v sha256sum >/dev/null; then
        sha256sum "$1" | cut -d ' ' -f 1
        return
    fi
    shasum -a 256 "$1" | cut -d ' ' -f 1
}

# downloads asset $2 of release $1 into folder $3 and checks it against the release's SHA256SUMS
download() {
    local expected
    echo "downloading $2"
    curl -fsSL -o "$3/$2" "$repo/releases/download/$1/$2"
    curl -fsSL -o "$3/SHA256SUMS" "$repo/releases/download/$1/SHA256SUMS"
    expected="$(awk -v name="$2" '$2 == name || $2 == "*" name { print $1 }' "$3/SHA256SUMS")"
    [ -n "$expected" ] || fail "SHA256SUMS of $1 has no line for $2"
    [ "$(sha256_of "$3/$2")" = "$expected" ] || fail "$2 does not match its checksum in SHA256SUMS of $1"
}

# moves everything in folder $1 into folder $2, replacing what is there; renaming keeps a running binary working
move_into() {
    local entry target
    mkdir -p "$2"
    for entry in "$1"/*; do
        target="$2/$(basename "$entry")"
        rm -rf "$target.new"
        mv "$entry" "$target.new"
        rm -rf "$target"
        mv "$target.new" "$target"
    done
}

install_linux_icons() {
    local icon size
    for icon in "$1"/icons/*.png; do
        size="$(basename "$icon" .png)"
        install -D -m 644 "$icon" "$data/icons/hicolor/${size}x${size}/apps/ive.png"
    done
    # a missing cache only means the desktop finds the icon a little later
    gtk-update-icon-cache -q -t "$data/icons/hicolor" 2>/dev/null || true
}

# IVE's window has the app id `ive`: StartupWMClass ties it to this entry, and so to the icon
write_desktop_entry() {
    install -D -m 644 /dev/stdin "$data/applications/ive.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=IVE
GenericName=Integrated Vibe Editor
Exec="$1" %f
Icon=ive
Terminal=false
Categories=Development;IDE;
StartupWMClass=ive
EOF
    update-desktop-database -q "$data/applications" 2>/dev/null || true
}

# replaces ~/.local/bin/ive (a symlink, or the launcher of an install from source) with $1
link_command() {
    mkdir -p "$bin_dir"
    rm -f "$bin_dir/ive"
    ln -s "$1" "$bin_dir/ive"
}

install_linux() {
    local home="$data/ive"
    tar -xzf "$1/$2" -C "$1"
    move_into "$1/${2%.tar.gz}" "$home"
    install_linux_icons "$home"
    write_desktop_entry "$home/ive"
    link_command "$home/ive"
    echo "installed $home, $bin_dir/ive and the desktop entry IVE"
}

install_macos() {
    local app="$HOME/Applications/IVE.app"
    local lsregister=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
    ditto -x -k "$1/$2" "$1"
    mkdir -p "$(dirname "$app")"
    rm -rf "$app.new"
    mv "$1/${2%.zip}/IVE.app" "$app.new"
    rm -rf "$app"
    mv "$app.new" "$app"
    # a script, not a symlink: IVE finds the app it updates from the path it was started by
    mkdir -p "$bin_dir"
    rm -f "$bin_dir/ive"
    printf '#!/bin/sh\nexec %q "$@"\n' "$app/Contents/MacOS/IVE" >"$bin_dir/ive"
    chmod 755 "$bin_dir/ive"
    # the Dock and Finder otherwise keep the icon of an IVE.app they saw before
    "$lsregister" -f "$app"
    echo "installed $app and $bin_dir/ive"
}

report_path() {
    case ":$PATH:" in
        *":$bin_dir:"*) echo "run ive, or find IVE in the application launcher" ;;
        *) echo "find IVE in the application launcher; to run ive in a shell, add $bin_dir to PATH" ;;
    esac
}

main() {
    local prefix tag commit work
    prefix="$(asset_prefix)"
    tag="$(latest_tag)"
    commit="${tag##*-}"
    work="$(mktemp -d)"
    # expanded now: the trap runs after main, when its local work is gone
    trap "rm -rf '$work'" EXIT
    if [ "$prefix" = ive-macos-arm64 ]; then
        download "$tag" "$prefix-$commit.zip" "$work"
        install_macos "$work" "$prefix-$commit.zip"
    else
        download "$tag" "$prefix-$commit.tar.gz" "$work"
        install_linux "$work" "$prefix-$commit.tar.gz"
    fi
    echo "IVE $tag is installed; a running IVE switches to it when it restarts"
    report_path
}

# called last, so a download cut short runs nothing
main "$@"
