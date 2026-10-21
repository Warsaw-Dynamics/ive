#!/usr/bin/env bash
# Start script for `ive --restart`. Run `./restart.sh` once from a terminal: it starts IVE with the Restart button.
# The button types `restart.sh --replace PID` into a terminal tab: it builds IVE and, after a good build, makes the
# IVE with PID quit (SIGUSR1: it saves and leaves the terminals running) and starts the new one. A failed build stays
# in that tab. To start IVE with other flags (--profile, --mcp, a folder), copy this script and add them to IVE_FLAGS.
set -euo pipefail
cd "$(dirname "$0")"
script="$(pwd)/$(basename "$0")"
IVE_FLAGS=(--restart="$script")

# the program name of process $1, empty once it is gone (macOS `ps` prints the whole path)
process_name() {
    local command
    command="$(ps -p "$1" -o comm= 2>/dev/null)" || return 0
    basename "$command"
}

# up to 10 s: the old IVE saves its tabs as it quits, and the new one reads them as it starts
wait_for_exit() {
    local tries=100
    while kill -0 "$1" 2>/dev/null && [ "$tries" -gt 0 ]; do
        sleep 0.1
        tries=$((tries - 1))
    done
}

# in a session of its own, so the old IVE's terminal closing does not hang it up
start_detached() {
    if command -v setsid >/dev/null; then
        setsid -f "$@" </dev/null >/dev/null 2>&1
        return
    fi
    # macOS has no setsid(1)
    perl -MPOSIX -e '$SIG{HUP} = "DEFAULT"; fork and exit; POSIX::setsid(); exec @ARGV or die "cannot start $ARGV[0]: $!"' \
        "$@" </dev/null >/dev/null 2>&1
}

case "${1:-}" in
    "") exec target/release/ive "${IVE_FLAGS[@]}" ;;
    --replace)
        pid="${2:?--replace needs the process id of the IVE to replace}"
        cargo build --release
        # SIGUSR1 ends any other program, so make sure the process is still IVE
        if [ "$(process_name "$pid")" != ive ]; then
            echo "restart.sh: process $pid is not IVE (any more); start IVE with ./restart.sh" >&2
            exit 1
        fi
        # an IVE that does not keep its terminals hangs this one up as it quits: outlive it
        trap '' HUP
        kill -USR1 "$pid"
        wait_for_exit "$pid"
        start_detached target/release/ive "${IVE_FLAGS[@]}"
        ;;
    *)
        echo "restart.sh: unexpected argument $1; put IVE's flags in IVE_FLAGS (in a copy)" >&2
        exit 2
        ;;
esac
