# Task17 final implementation checkpoint

- Base: `da590bc60089535f19649e825aeaa38e9636d6fb` (stable central 21-parser product).
- Additive tooling: `dd7c26f4b347267ff96ec6364c1a99e8100e93c9`, cherry-pick of tested `d29b7468556c4d3c1d77c6d44dac89964f589eac`; no conflicts.
- Final head: `f714769c7ebe6a9c3a4627468d09bd9f4c0feb89`, branch `diagram-parser-final`, checkout `/mnt/data/Projects/ive/.worktree/diagram-final`.
- Tracked tree clean; untracked engine-cache `.cargo/config.toml` remains excluded. No central product/progress.md edits or central integration performed.
- Current structural migration/removal is implemented; central broad/native/full gates, baseline execution and independent final review remain pending.

## Final behavior and residual matrix

- All 21 grammar inputs/generated outputs/module registrations remain intact with unchanged bytes. Definitions/documents/headers/preamble, all current families and Creole have one generated structural authority; no pending-syntax exemptions remain.
- Removed `source::Line`, `ParsedDocument.lines`, `document::append_line`, the render body-line slice, header `PhysicalRows::locate`, and Gantt's dummy diagnostic line. Header selection streams bounded borrowed body bytes through shared ownership/generated evidence and maps failures to truthful expanded row spans.
- Retired 20 obsolete source/native/parser setup functions. Five shared generated-entry test helpers plus one source test inspector replace them: final ledger covers 72 authored modules and 1,097 function declarations (base: 71/1,111), with no native-directory exemption.
- Retained lexical primitives: finite arrows/dates/colors/escapes, ownership/comment/literal handling, marker/path/number/scalar primitives, bounded cursors and Creole same-line pair eligibility. Typed builders, immutable definition evaluation, origin/error mapping, semantic identities/constraints/models and layouts remain.
- Complete committed matrix: `diagram-ports/provenance/plantuml-parser-residuals.md`; explicit function ledger: `plantuml-parser-responsibilities.json`. Source audit rejects unknown functions/files, pending syntax, obsolete physical-line types, stale grammar/output bytes and registration drift.
- Actual document retention is the body-view size plus preamble/token/generated-stack retention; obsolete row-vector/string storage and its work charges are removed. Render still admits/charges the additional family view before entry. All worker token/literal/table/depth/model controls and defaults remain unchanged. Some work totals decrease because removed copies/locators and duplicate test setup no longer run; no work golden was rewritten.
- Gantt's finite date primitive takes a line number directly; public generated date failure spans/details remain unchanged. The impossible private locator-outside-document error disappears with its synthetic input interface.

## Meaningful feature RED/GREEN

- `/tmp/ive-task17-final-audit-red.log`: final audit accepted a deliberately inserted pending-syntax role (7 pass/1 fail). Enforcement now rejects that exemption.
- `/tmp/ive-task17-final-line-audit-red.log`: audit accepted a restored old `Line` input type inside an existing registered source module (8 pass/1 fail). Enforcement now rejects that obsolete interface.
- Final inventory tests are GREEN within `/tmp/ive-task17-final-node.log`: 25 Node tests across source/inventory/baseline contracts, including both new controls and prior unknown-input/stale-output/default-drift controls.

## Setup migration, separately from feature RED

- Line-based test inputs became ordinary source/body strings; shared cfg(test) helpers invoke generated entries. Styled/model tests keep their source data and assertions. Activity uses document-only construction with no new preprocessing, matching its original setup semantics.
- Existing 60,000-row/500ms activity data/deadlines/assertions and worker fault/control assertions remain unchanged. Document tests now assert exact borrowed expanded bytes (including inert comments/raw rows), source line position and actual retained view size instead of deleted line-copy fields.
- Mechanical type/import/range/cfg errors during setup migration were corrected; they are not counted as feature REDs. Initial compile evidence: `/tmp/ive-task17-final-units-first.log`, `/tmp/ive-task17-final-public.log` (dangling test-import cfg corrected). Final 593-test run below passes both test and production library builds.

## Observed final verification

All Cargo/Cargo-spawning commands used outer `bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=...`; Rust harnesses used `-- --test-threads=1`. Workloads were serial. Generator commands used cwd `/tmp`, an absolute manifest/root and private target `/mnt/data/Projects/ive/target/diagram-generator-final`.

- `cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_constants --test plantuml_control --test plantuml_styles --test plantuml_tree_data --test plantuml_gantt --test plantuml_timing --test plantuml_nwdiag --test plantuml_salt --test plantuml_chen_archimate --test plantuml_inventory -- --test-threads=1`: **593 pass**, 12 suites; `/tmp/ive-task17-final-focused.log`. Counts: 320,43,13,45,25,17,27,17,18,26,23,19.
- `cargo test ... -p diagram-engine --example parser_baseline -- --test-threads=1`: real default drift contract **1 pass**; `/tmp/ive-task17-final-exporter-test.log`. Hashed test harness only, no baseline exporter/Cargo-run example execution.
- `node --test tools/audit-plantuml.test.mjs tools/parser-inventory.test.mjs tools/parser-baseline.test.mjs`: **25 pass**; `/tmp/ive-task17-final-node.log`.
- Private generator `cargo test --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-final/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- --test-threads=1`: **5 pass**, including same-binary two-checkout/wrong-root isolation; `/tmp/ive-task17-final-generator-tests.log`.
- Strict engine `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` and corresponding private generator Clippy: pass; `/tmp/ive-task17-final-{engine,generator}-clippy.log`.
- Engine `cargo fmt --all --check`, private generator manifest `cargo fmt ... --all --check`, `git diff --cached --check`: pass; `/tmp/ive-task17-final-{engine,generator}-fmt.log`.
- Private generator `cargo run --locked --offline --manifest-path .../tools/plantuml-grammar-gen/Cargo.toml -- check --root /mnt/data/Projects/ive/.worktree/diagram-final/diagram-ports`: **21 reproducible parsers**, no tracked writes; `/tmp/ive-task17-final-generator-check.log`.
- Source audit: **220 MIT source/ZIP hashes, 43 fresh fixture hashes, 43 native modules**, plus final parser inventory checks; `/tmp/ive-task17-final-source-audit.log`.
- Absolute generator dependency audit from `/tmp`: **67 archives/unpacked sources/licenses/features/exact notices**; `/tmp/ive-task17-final-closure.log`. Audit unit tests **5 pass**; `/tmp/ive-task17-final-closure-tests.log`.

## Hash witnesses and final handoff

- Immutable baseline SHA256 unchanged: `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`.
- Actual 21-parser manifest SHA256 unchanged from da590bc: `59e87d2612efee01e077484704af2ff835ad408435dd988b46940b5ee1f60d7f`; all grammar/generated Rust bytes unchanged and independently regenerated.
- Generator manifest/lock SHA256 unchanged: `7d0b09f114c9e33578d210ddb9bfe7acf5a3a35837f380b387af72a1c24347bf` / `ebce0335dc9a1c4be0bdb8ecb1f31cf3e666c1622d87430c02946b138abad165`.
- Final function ledger SHA256: `ac98c9ff348e9de99613ddb6faab9b2738ef5570f2e3761c0b134b275b89f6e5`.
- No fixture/golden/authorized-source/vendor/limit/public API edits. Audited Atomic warnings remain in merman-core operation.rs:449 and development test-support merman-render environment.rs:585.
- Carried timing evidence remains `/tmp/ive-task4-tests-concurrent-deadline.log` and task-4-report.md serial passes; sensitivity observed, no exclusive cause claimed or deadlines relaxed.
- Root to integrate both isolated commits additively, then run coordinated full/baseline43/native/capture gates and independent review. Paused before central integration.
