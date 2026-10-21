#!/usr/bin/env bash
# Builds and starts IVE. Extra arguments go to cargo, e.g. ./run.sh --release
set -euo pipefail

cd "$(dirname "$0")"
./check-build.sh
exec cargo run "$@"
