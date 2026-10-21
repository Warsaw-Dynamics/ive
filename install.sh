#!/usr/bin/env bash
# Installs IVE for the current user: builds it, writes the launcher ~/.local/bin/ive, adds IVE to the desktop (Linux:
# an "IVE" desktop entry GNOME, KDE Plasma and other freedesktop launchers find; macOS: ~/Applications/IVE.app) and
# starts IVE. IVE runs as a profile named after the user, with the Restart button: it rebuilds this checkout and
# restarts IVE. Run it again to rebuild and reinstall.
set -euo pipefail
cd "$(dirname "$0")"
source_dir="$(pwd)"
bin_dir="$HOME/.local/bin"
launcher="$bin_dir/ive"
mac_app="$HOME/Applications/IVE.app"

# `--profile` takes letters, digits, `-` and `_`, up to 64
profile_name() {
    local name
    name="$(id -un | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_\n-' '-')"
    name="${name:0:64}"
    echo "${name:-user}"
}

is_mac() {
    [ "$(uname -s)" = Darwin ]
}

# how the launcher starts the new IVE after a restart: in a session of its own, so the old IVE's terminal closing
# does not hang it up; on macOS through the app, so the Dock shows it as IVE
start_new_ive() {
    if is_mac; then
        echo "open -n \"$mac_app\""
        return
    fi
    echo 'setsid -f target/release/ive "${IVE_FLAGS[@]}" </dev/null >/dev/null 2>&1'
}

write_launcher() {
    mkdir -p "$bin_dir"
    cat >"$launcher" <<EOF
#!/usr/bin/env bash
# IVE with profile $1 and the Restart button, written by $source_dir/install.sh.
# \`ive\` starts IVE; the Restart button types \`ive --replace PID\` into a terminal tab, which builds IVE and
# swaps the running one for the new one. See restart.sh.
set -euo pipefail

SOURCE="$source_dir"
# a desktop launch has no shell profile, so cargo (asdf, rustup) is not on PATH yet
export PATH="\$HOME/.asdf/shims:\$HOME/.cargo/bin:\$PATH"
IVE_FLAGS=(--profile $1 --restart="\$(realpath "\$0")")

# the program name of process \$1, empty once it is gone (macOS \`ps\` prints the whole path)
process_name() {
    local command
    command="\$(ps -p "\$1" -o comm= 2>/dev/null)" || return 0
    basename "\$command"
}

# up to 10 s: the old IVE saves its tabs as it quits, and the new one reads them as it starts
wait_for_exit() {
    local tries=100
    while kill -0 "\$1" 2>/dev/null && [ "\$tries" -gt 0 ]; do
        sleep 0.1
        tries=\$((tries - 1))
    done
}

cd "\$SOURCE"
case "\${1:-}" in
    "")
        if [ ! -x target/release/ive ]; then
            cargo build --release
        fi
        exec target/release/ive "\${IVE_FLAGS[@]}"
        ;;
    --replace)
        pid="\${2:?--replace needs the process id of the IVE to replace}"
        cargo build --release
        # SIGUSR1 ends any other program, so make sure the process is still IVE
        if [ "\$(process_name "\$pid")" != ive ]; then
            echo "ive: process \$pid is not IVE (any more); start IVE with ive" >&2
            exit 1
        fi
        # an IVE that does not keep its terminals hangs this one up as it quits: outlive it
        trap '' HUP
        kill -USR1 "\$pid"
        wait_for_exit "\$pid"
        $(start_new_ive)
        ;;
    *)
        echo "ive: unexpected argument \$1" >&2
        exit 2
        ;;
esac
EOF
    chmod 755 "$launcher"
}

add_to_desktop() {
    if is_mac; then
        desktop/install-macos.sh "$launcher"
        return
    fi
    desktop/install.sh "$launcher"
}

start_ive() {
    if pgrep -u "$(id -u)" -f "(^|/)ive --profile $1( |$)" >/dev/null; then
        echo "IVE already runs as profile $1: press its Restart button to switch to the new build"
        return
    fi
    if is_mac; then
        open "$mac_app"
    else
        setsid -f "$launcher" </dev/null >/dev/null 2>&1
    fi
    echo "started IVE (profile $1)"
}

./check-build.sh
cargo build --release
profile="$(profile_name)"
write_launcher "$profile"
add_to_desktop
start_ive "$profile"
echo "installed $launcher; find IVE in the application launcher, or run ive if $bin_dir is on PATH"
