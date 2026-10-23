#!/usr/bin/env bash
set -euo pipefail
exec flock /tmp/ive-diagram-build.lock "$@"
