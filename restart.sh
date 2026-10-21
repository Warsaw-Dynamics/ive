#!/usr/bin/env bash
# Start script for `ive --restart`. Run `./restart.sh` once from a terminal: it starts IVE with the Restart button.
# The button types `restart.sh --replace PID` into a terminal tab: it builds IVE and, after a good build, makes the
# IVE with PID quit (SIGUSR1: it saves and leaves the terminals running) and starts the new one. A failed build stays
# in that tab. To start IVE with other flags (--profile, --mcp, a folder), copy this script and add them to IVE_FLAGS.
set -euo pipefail
cd "$(dirname "$0")"
script="$(pwd)/$(basename "$0")"
IVE_FLAGS=(--restart="$script")

case "${1:-}" in
    "") exec target/release/ive "${IVE_FLAGS[@]}" ;;
    --replace)
        pid="${2:?--replace needs the process id of the IVE to replace}"
        cargo build --release
        # SIGUSR1 ends any other program, so make sure the process is still IVE
        if [ "$(cat "/proc/$pid/comm" 2>/dev/null)" != ive ]; then
            echo "restart.sh: process $pid is not IVE (any more); start IVE with ./restart.sh" >&2
            exit 1
        fi
        # an IVE that does not keep its terminals hangs this one up as it quits: outlive it
        trap '' HUP
        kill -USR1 "$pid"
        # the new IVE waits for the old one to let go of the terminals and the MCP port
        setsid -f target/release/ive "${IVE_FLAGS[@]}" </dev/null >/dev/null 2>&1
        ;;
    *)
        echo "restart.sh: unexpected argument $1; put IVE's flags in IVE_FLAGS (in a copy)" >&2
        exit 2
        ;;
esac
