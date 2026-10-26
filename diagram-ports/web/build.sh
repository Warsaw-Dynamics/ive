#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked --release --target wasm32-unknown-unknown -p diagram-web
mkdir -p web/pkg
target_dir="${CARGO_TARGET_DIR:-target}"
cp "$target_dir/wasm32-unknown-unknown/release/diagram_web.wasm" web/pkg/diagram_web.wasm
cp vendor/graphviz-wasm/dist/index.js web/pkg/graphviz.js
cp vendor/graphviz-wasm/LICENSE web/pkg/GRAPHVIZ-WASM-LICENSE
cp vendor/graphviz/COPYING web/pkg/GRAPHVIZ-LICENSE
cp web/fonts/LICENSE web/pkg/FONTS-LICENSE
