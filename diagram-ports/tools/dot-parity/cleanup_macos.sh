#!/usr/bin/env bash
# Remove only a temporary macOS parity directory created by this harness.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 WORK_DIRECTORY" >&2
  exit 2
fi
temp_root=$(cd -- "${TMPDIR:-/tmp}" && pwd -P)
work_dir=$(cd -- "$1" && pwd -P)
case "$work_dir" in
  "$temp_root"/ive-dot-macos.*) ;;
  *) echo "refusing to remove non-parity directory $work_dir" >&2; exit 2 ;;
esac
if [[ ! -f "$work_dir/.ive-dot-parity-owned" ]]; then
  echo "refusing to remove $work_dir without harness ownership marker" >&2
  exit 2
fi
rm -rf -- "$work_dir"
echo "removed $work_dir"
