#!/usr/bin/env bash
# Build a developer-only native Graphviz 16.1.0 GD oracle on macOS.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 WORK_DIRECTORY" >&2
  exit 2
fi
if [[ $(uname -s) != Darwin ]]; then
  echo "the macOS GD oracle build requires a macOS host" >&2
  exit 2
fi
mkdir -p -- "$1"
work_dir=$(cd -- "$1" && pwd -P)
touch "$work_dir/.ive-dot-parity-owned"

script_dir=$(cd -- "$(dirname -- "$0")" && pwd)
project_dir=$(cd -- "$script_dir/../.." && pwd)
archive="$project_dir/vendor/graphviz/graphviz-16.1.0.tar.xz"
expected=0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3
actual=$(shasum -a 256 "$archive" | cut -d ' ' -f 1)
if [[ $actual != "$expected" ]]; then
  echo "Graphviz source hash differs: expected $expected, found $actual" >&2
  exit 2
fi
pkg-config --exists gdlib || { echo "gdlib development headers are required" >&2; exit 2; }
pkg-config --exists expat || { echo "Expat development headers are required" >&2; exit 2; }

prefix="$work_dir/oracle"
source_root="$work_dir/source"
build_root="$work_dir/build"
if [[ -e "$prefix/bin/dot" ]]; then
  echo "oracle already exists at $prefix/bin/dot; use a fresh work directory" >&2
  exit 2
fi
mkdir -p -- "$prefix" "$source_root" "$build_root"
tar -xf "$archive" -C "$source_root"
source_dir="$source_root/graphviz-16.1.0"

(
  cd -- "$build_root"
  "$source_dir/configure" \
    --prefix="$prefix" --with-expat=yes --with-libgd=yes \
    --with-pangocairo=no --with-gdk=no --with-gdk-pixbuf=no \
    --with-rsvg=no --with-ghostscript=no --with-poppler=no \
    --with-gtk=no --with-smyrna=no --without-x > configure.log 2>&1
  make -j "$(sysctl -n hw.ncpu)" > make.log 2>&1
  make install > install.log 2>&1
)

export DYLD_LIBRARY_PATH="$prefix/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
"$prefix/bin/dot" -V
"$prefix/bin/dot" -Tjson0 -o "$work_dir/chain.json" "$script_dir/fixtures/chain.dot"
"$prefix/bin/dot" -Tsvg -o "$work_dir/html.svg" "$script_dir/fixtures-expat/html_table_cells.dot"
python3 - "$work_dir/html.svg" <<'PY'
from pathlib import Path
import sys
svg = Path(sys.argv[1]).read_text()
if "one" not in svg or "two" not in svg:
    raise SystemExit("Expat table SVG probe lacks cell text")
PY
"$prefix/bin/dot" -Tpng:gd:gd -o "$work_dir/chain-gd.png" "$script_dir/fixtures/chain.dot"
python3 - "$work_dir/chain-gd.png" <<'PY'
from PIL import Image
import sys
with Image.open(sys.argv[1]) as image:
    image.verify()
print("verified native GD PNG")
PY
echo "oracle=$prefix/bin/dot"
echo "export DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH"
