#!/usr/bin/env bash
# Build a developer-only native PNG oracle from the pinned Graphviz 16.1.0 source.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 /tmp/graphviz16-gd-prefix" >&2
  exit 2
fi
if [[ $(uname -s) != Linux ]]; then
  echo "the GD PNG oracle build is supported only on Linux" >&2
  exit 2
fi

script_dir=$(cd -- "$(dirname -- "$0")" && pwd)
project_dir=$(cd -- "$script_dir/../.." && pwd)
archive="$project_dir/vendor/graphviz/graphviz-16.1.0.tar.xz"
expected=0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3
actual=$(sha256sum "$archive" | cut -d ' ' -f 1)
if [[ $actual != "$expected" ]]; then
  echo "Graphviz source hash differs: expected $expected, found $actual" >&2
  exit 2
fi
pkg-config --exists gdlib || { echo "gdlib development headers are required" >&2; exit 2; }

prefix=$(realpath -m -- "$1")
build_root="${prefix}-build"
source_root="${prefix}-source"
mkdir -p -- "$prefix" "$build_root" "$source_root"
tar -xf "$archive" -C "$source_root"
source_dir="$source_root/graphviz-16.1.0"

(
  cd -- "$build_root"
  "$source_dir/configure" \
    --prefix="$prefix" --with-libgd=yes --with-pangocairo=no \
    --with-gdk=no --with-gdk-pixbuf=no --with-rsvg=no \
    --with-ghostscript=no --with-poppler=no --with-gtk=no \
    --with-smyrna=no --without-x > configure.log 2>&1
  make -j "$(nproc)" > make.log 2>&1
  make install > install.log 2>&1
)

"$prefix/bin/dot" -Tpng:gd:gd -o "$prefix/html-probe.png" "$script_dir/fixtures/html.dot"
"$prefix/bin/dot" -V
echo "native GD PNG probe: $prefix/html-probe.png"
