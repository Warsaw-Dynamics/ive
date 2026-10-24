#!/usr/bin/env bash
# Compile a tiny probe against the already-built engine; this does not edit product code.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 OUTPUT_DIRECTORY" >&2
  exit 2
fi
script_dir=$(cd -- "$(dirname -- "$0")" && pwd)
project_dir=$(cd -- "$script_dir/../.." && pwd)
dependencies="$project_dir/target/debug/deps"
engine_rlib=$(find "$dependencies" -maxdepth 1 -name 'libdiagram_engine-*.rlib' -printf '%T@ %p\n' | sort -nr | head -1 | cut -d ' ' -f 2-)
if [[ -z $engine_rlib ]]; then
  echo "no built diagram-engine rlib in $dependencies; build diagram-viewer first" >&2
  exit 2
fi
mkdir -p -- "$1"
rustc --edition=2024 -L "dependency=$dependencies" --extern "diagram_engine=$engine_rlib" \
  "$script_dir/work_limit_probe.rs" -o "$1/dot-work-limit-probe"
printf 'engine_rlib_sha256=%s\n' "$(sha256sum "$engine_rlib" | cut -d ' ' -f 1)"
"$1/dot-work-limit-probe"
