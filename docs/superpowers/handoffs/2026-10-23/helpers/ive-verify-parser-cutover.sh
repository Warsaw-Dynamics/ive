#!/usr/bin/env bash
set -euo pipefail
export CARGO_TARGET_DIR=/mnt/data/Projects/ive/target
verification_mode=${1:?core or audit required}
verification_prefix=${2:?absolute temporary log prefix required}
case "$verification_prefix" in /tmp/ive-*) ;; *) exit 2 ;; esac

run_check() {
    local verification_label=$1
    shift
    "$@" > "${verification_prefix}-${verification_label}.log" 2>&1
}

verify_core() {
    run_check tests cargo test --locked --offline --workspace -- --test-threads=1
    run_check clippy cargo clippy --locked --offline --all-targets -- -D warnings
    run_check fmt cargo fmt --all --check
    run_check release cargo build --locked --offline --release
    run_check deny cargo deny --locked --offline --config ../deny.toml check licenses sources
    run_check gen-tests env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target/diagram-generator-integration cargo test --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml
    run_check gen-clippy env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target/diagram-generator-integration cargo clippy --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all-targets -- -D warnings
    run_check gen-fmt cargo fmt --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all --check
    run_check gen-deny cargo deny --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --config ../deny.toml check licenses sources
}

verify_audit() {
    run_check plantuml node tools/audit-plantuml.mjs
    run_check merman node tools/audit-merman.mjs
    run_check node node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs tools/parser-baseline.test.mjs
    run_check generation env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target/diagram-generator-integration cargo run --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml -- check
    run_check pytests python3 -B tools/audit-plantuml-generator.test.py
    run_check gen-audit python3 -B tools/audit-plantuml-generator.py check
    run_check baseline node tools/parser-baseline.mjs check
}

case "$verification_mode" in
    core) verify_core ;;
    audit) verify_audit ;;
    *) exit 2 ;;
esac
