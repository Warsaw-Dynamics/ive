# Task17 tooling/preparation checkpoint

- Base: `e4ace61405c0cb5670df02b2879a44160f085fab`.
- Head: `d29b7468556c4d3c1d77c6d44dac89964f589eac`, branch `diagram-parser-final-tooling`.
- Checkout: `/mnt/data/Projects/ive/.worktree/diagram-final-tooling`; tracked tree clean. Only the untracked engine cache `.cargo/config.toml` remains, excluded from commit.
- Central product/progress.md untouched. This checkpoint does not complete Task17 removal/integration/review.

## Changes

- Generator CLI requires `generate|check --root ABSOLUTE_DIAGRAM_PORTS_ROOT`; validates canonical root, regular bounded workspace/engine/generator markers and grammar/output directories. Removed runtime selection by embedded `CARGO_MANIFEST_DIR`; no cache cleanup or dependency change.
- One actual compiled executable generates into two isolated fixture checkouts while invoked from the opposite checkout, preserving the other output. Missing, relative and wrong roots fail without fallback writes.
- Baseline exporter emits actual render-option configuration before fixture records, verifies the deterministic measurement route, and uses those same options to render. Driver compares this reported configuration with immutable baseline metadata. A Rust test compares real defaults with the unchanged baseline and detects changed limits; Node tests reject missing metadata/drift.
- Source audit checks exact authored/generated inventories, grammar/output hash pairs and generated module registration. Every one of 1,059 authored functions across 69 modules is individually classified, including native modules and test setup; unknown functions/files and stale output fail.
- Committed `provenance/plantuml-parser-residuals.md` gives migrated/pending matrix and precise physical-line/parse/prefix/split/cursor cleanup boundary. Eleven packet/Creole functions explicitly remain pending at this base.

## Observed RED

- `/tmp/ive-task17-generator-root-red.log`: new executable-root tests failed 0/2; explicit-root CLI rejected and missing-root generation incorrectly succeeded using embedded checkout.
- `/tmp/ive-task17-baseline-red.log`: new actual-export configuration test failed, prior two baseline contracts passed.
- `/tmp/ive-task17-inventory-red.log`: stale grammar, stale generated Rust and new function in existing native module all failed to reject; current inventory/unknown grammar controls passed (2 pass/3 fail).

## Observed GREEN

All Cargo and Cargo-spawning commands used outer `bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=...`; test harnesses were serial. Generator commands ran from `/tmp` with absolute manifest and private target `/mnt/data/Projects/ive/target/diagram-generator-tooling`.

- `cargo test --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-final-tooling/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- --test-threads=1`: 3 unit + 2 executable regressions pass; `/tmp/ive-task17-generator-final-tests.log`.
- `cargo test --locked --offline -p diagram-engine --example parser_baseline -- --test-threads=1`: 1 real-default drift test passes; `/tmp/ive-task17-exporter-test.log`. Hashed test harness only; no baseline exporter/Cargo-run example execution.
- `node --test tools/audit-plantuml.test.mjs tools/parser-inventory.test.mjs tools/parser-baseline.test.mjs`: 23 pass; `/tmp/ive-task17-node-final.log`.
- `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`: pass; `/tmp/ive-task17-engine-clippy.log`.
- Private generator `cargo clippy --locked --offline --manifest-path .../tools/plantuml-grammar-gen/Cargo.toml --all-targets -- -D warnings`: pass; `/tmp/ive-task17-generator-clippy-final.log`.
- Engine `cargo fmt --all --check` and generator `cargo fmt --manifest-path .../tools/plantuml-grammar-gen/Cargo.toml --all --check`: pass; `/tmp/ive-task17-{engine-fmt,generator-fmt-final}.log`.
- Private `cargo run --locked --offline --manifest-path .../tools/plantuml-grammar-gen/Cargo.toml -- check --root /mnt/data/Projects/ive/.worktree/diagram-final-tooling/diagram-ports`: 19 reproducible parsers, no tracked writes; `/tmp/ive-task17-generator-check.log`.
- `node tools/audit-plantuml.mjs`: 220 MIT source/ZIP hashes, 43 fresh fixture hashes, 43 native modules pass; `/tmp/ive-task17-source-audit.log`.
- Absolute `python3 .../tools/audit-plantuml-generator.py check` from `/tmp`: 67 archives/unpacked sources/licenses/features/exact notices pass; `/tmp/ive-task17-generator-closure-final.log`. Initial run from engine workspace inherited untracked engine-only patches and `--locked` rejected that graph; `/tmp/ive-task17-generator-closure.log`. Correct cwd resolved it; no lock/config/vendor changes.
- `python3 -B .../tools/audit-plantuml-generator.test.py`: 5 pass; `/tmp/ive-task17-generator-audit-tests.log`. `git diff --cached --check` passed before commit.

## Hash witnesses and residual work

- Immutable 43 baseline SHA256: `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`, unchanged.
- Generated manifest SHA256: `d2fadd1a57a17ce41fd95ee06c3b63e18bc619f6543f7a3890e4392af528d522`, unchanged; all generated grammar/Rust bytes unchanged.
- Generator manifest/lock SHA256: `7d0b09f114c9e33578d210ddb9bfe7acf5a3a35837f380b387af72a1c24347bf` / `ebce0335dc9a1c4be0bdb8ecb1f31cf3e666c1622d87430c02946b138abad165`, unchanged.
- Function responsibility ledger SHA256: `c340ad4229f3b9a161ce46b33a3ae9ee4dc8d307fe6925bafec478efc132c84e`.
- Packet production line syntax and Creole recognition/pairing/span stack remain intentionally present. After Tasks15/16 integration, update ledger/matrix on fresh base and remove obsolete physical-line/test setup adapters; preserve primitive/model/layout behavior and all public pipeline regressions.
- Existing 60,000-row/500ms action-control assertions/deadlines/data unchanged. Carried timing evidence remains `/tmp/ive-task4-tests-concurrent-deadline.log` and task-4-report.md serial passes; no exclusive cause claimed.
- Audited vendor code untouched. Inherited Atomic warnings remain in merman-core operation.rs:449 and development test-support merman-render environment.rs:585.
- Full baseline43 rendering, broad/native/full gates, representative captures, final removal and independent review were intentionally deferred to coordinated central completion. No frozen legacy comparator, new runtime dependency, default-limit edit or authorized MIT source/vendor edit was introduced.
