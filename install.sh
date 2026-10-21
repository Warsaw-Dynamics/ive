#!/usr/bin/env bash
# Installs IVE for the current user (Linux): builds it, writes the launcher ~/.local/bin/ive, adds an "IVE" desktop
# entry (GNOME, KDE Plasma and other freedesktop launchers find it) and starts IVE. IVE runs as a profile named after
# the user, with the Restart button: it rebuilds this checkout and restarts IVE. Run it again to rebuild and reinstall.
set -euo pipefail
cd "$(dirname "$0")"
source_dir="$(pwd)"
bin_dir="$HOME/.local/bin"
launcher="$bin_dir/ive"

# `--profile` takes letters, digits, `-` and `_`, up to 64
profile_name() {
    local name
    name="$(id -un | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_\n-' '-')"
    name="${name:0:64}"
    echo "${name:-user}"
}

write_launcher() {
    install -D -m 755 /dev/stdin "$launcher" <<EOF
#!/usr/bin/env bash
# IVE with profile $1 and the Restart button, written by $source_dir/install.sh.
# \`ive\` starts IVE; the Restart button types \`ive --replace PID\` into a terminal tab, which builds IVE and
# swaps the running one for the new one. See restart.sh.
set -euo pipefail

SOURCE="$source_dir"
# a desktop launch has no shell profile, so cargo (asdf, rustup) is not on PATH yet
export PATH="\$HOME/.asdf/shims:\$HOME/.cargo/bin:\$PATH"
IVE_FLAGS=(--profile $1 --restart="\$(realpath "\$0")")

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
        if [ "\$(cat "/proc/\$pid/comm" 2>/dev/null)" != ive ]; then
            echo "ive: process \$pid is not IVE (any more); start IVE with ive" >&2
            exit 1
        fi
        # an IVE that does not keep its terminals hangs this one up as it quits: outlive it
        trap '' HUP
        kill -USR1 "\$pid"
        setsid -f target/release/ive "\${IVE_FLAGS[@]}" </dev/null >/dev/null 2>&1
        ;;
    *)
        echo "ive: unexpected argument \$1" >&2
        exit 2
        ;;
esac
EOF
}

start_ive() {
    if pgrep -u "$(id -u)" -f "(^|/)ive --profile $1( |$)" >/dev/null; then
        echo "IVE already runs as profile $1: press its Restart button to switch to the new build"
        return
    fi
    setsid -f "$launcher" </dev/null >/dev/null 2>&1
    echo "started IVE (profile $1)"
}

./check-build.sh
cargo build --release
profile="$(profile_name)"
write_launcher "$profile"
desktop/install.sh "$launcher"
start_ive "$profile"
echo "installed $launcher; find IVE in the application launcher, or run ive if $bin_dir is on PATH"
