#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked --release --target wasm32-unknown-unknown -p diagram-web
mkdir -p web/pkg
rm -f web/pkg/graphviz.js web/pkg/GRAPHVIZ-LICENSE web/pkg/GRAPHVIZ-WASM-LICENSE
target_dir="${CARGO_TARGET_DIR:-target}"
cp "$target_dir/wasm32-unknown-unknown/release/diagram_web.wasm" web/pkg/diagram_web.wasm
cp web/fonts/LICENSE web/pkg/FONTS-LICENSE
