# Task 10 — JSON/HCL WIP checkpoint, paused for session handoff

- Assigned worktree: `/mnt/data/Projects/ive/.worktree/diagram-data-parallel`; branch `diagram-parser-data`.
- Base: `0c5177e1f4cf50dcc23dc97ff3e0812b42d4070f`.
- WIP head: `e71d646` (`WIP: checkpoint generated JSON and HCL structural parsers`). NOT approved or ready to integrate.
- User requested freezing work because the weekly Codex limit is near. No final verification or completion claim.
- All product files are committed; root-owned untracked `diagram-ports/.cargo/` remains untouched.

## Implemented scope

- Added `parser/json.rs`, `parser/hcl.rs`, `parser/structured.rs`, two real structural grammars and mechanically generated outputs.
- JSON grammar constructs existing ordered `data::Value`: object/member/array/separator/number/EOF syntax, duplicate keys and exact numeric spelling.
- HCL grammar constructs existing `hcl::Document`: same-line header words, optional physical newlines before brace, blocks/fields/objects/limited arrays/EOF.
- Both typed facades run before the remaining legacy error bridge; HCL lowering also runs outside that bridge.
- Removed old recursive `JsonParser` from `data.rs` and old recursive `Parser` from `hcl.rs`.
- `data::Value`, `hcl::{Field,HclValue,Block,Document}`, unchanged lowering, datascene model/geometry assertions retained.
- Datascene unit setup now uses new JSON facade; model IDs, owning rows, link and private geometry assertions unchanged.
- `serde_json` in new JSON facade only decodes primitive `String`; number finite/exponent validation uses native numeric parsing, never whole JSON values.
- Lexemes borrow immutable expanded source; no child AST clones, fallback, recovery rendering, external IO or standard HCL evaluator.
- Mirrored UML `6f1244c` shared Cow refinement exactly in `parser/error.rs`, including `owned_syntax` for HCL header messages.
- Minimal state unit pattern adaptation compares Cow `detail.as_ref()` with the same `cross-ownership state endpoint` string; no state behavior/model change.

## Observed RED/GREEN (all Cargo used outer lock)

Prefix: `bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`.

1. RED: `cargo test --locked --offline -p diagram-engine --lib should_generate_ -- --test-threads=1`: 4 passed / 2 FAILED.
   New JSON/HCL structural facades returned explicit `generated JSON/HCL structure unavailable`; both expected new contracts failed at unwrap.
2. Generation from `/tmp`, PRIVATE target `/mnt/data/Projects/ive/target/diagram-generator-data`, absolute manifest:
   `cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-data-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- generate`.
   Observed `Generated 11 PlantUML parser(s) with LALRPOP 0.23.2.`
3. GREEN: `cargo test --locked --offline -p diagram-engine --lib plantuml::parser -- --test-threads=1`: 89 passed.
4. GREEN: `cargo test --locked --offline -p diagram-engine --lib --test plantuml_tree_data --test plantuml_inventory --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`.
   Observed 203 unit tests passed, plus all focused public suites; tree_data19 / inventory27 / styles26 / control25. Constants passed too.
5. Added deeper direct-generated invalid-token/EOF, model/depth-before-scan, 64/65 opener and cancellation contracts.
   Last completed focused run: 96 passed / 1 FAILED: JSON cancellation test incorrectly required checkpoint8, while the valid short path has7 checkpoints.
   Corrected cancellation range to1..=7; NOT rerun afterward. No product cancellation failure was observed.
6. After actual-depth admission correction: `cargo fmt --all` exit0 and `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` exit0.
   Audited vendor deprecation warnings remained warnings, no vendor edits.
7. Added newest raw JSON/decoded HCL quote, numeric boundary, frozen-definition origin and arena/failure-size tests AFTER the last Clippy run; these are UNVERIFIED.
8. `git diff --check` passed immediately before WIP commit. No application/native/baseline runs in this worker branch.

## Admission/proof checkpoint

- Actual witness code: `/tmp/ive-task10-stack-proof.py`; runs read generated ACTION/EOF/simulated reductions plus inspected exact goto matches.
- Observed JSON generated SHA256: `5f4403d249805aac57566ae9640b2374919715daa23e4547aba3fcf4de5402b4`; 48 states /16 terminals.
- Observed HCL generated SHA256: `dc4e03150889360397fddc0a9b8e40363fa2a0276c033d4cae785233d157c66f`; 40 states /10 terminals.
- Exhaustive depth0..4: JSON reachable64/326/1374/5566/22334, maxlive8/13/18/23/28, max8 consecutive reductions /10 expected terminals.
- HCL reachable7/29/55/113/223, maxlive5/8/12/16/20, max5 reductions /4 expected terminals.
- Weighted viable-prefix graph edges stabilize at depth3/4. Graph longest paths at depths0/1/2/8/64/65/128: JSON8/13/18/48/328/333/648; HCL6/8/12/36/260/264/516.
- Inductive structure: JSON recursive frame adds at most5 states; HCL at most4. All sibling lists fold left. Need finish written completeness argument for the graph/reduction bounds, then preserve witness stdout as a `.json` artifact.
- Current runtime admission uses OBSERVED token nesting, not unused configured depth: JSON5d+8 /16 terminals /8 reductions; HCLmax(4d+4,6) /10 /5.
- It reserves growth/transient stack capacity, diagnostic state copies/expected strings, adapters, borrowed token capacity, semantic text reserve and fallible model-vector growth.
- New unrun size contracts expect JSON symbol sum144, HCL136, FailureKind32/ParseFailure48. Review these before relying on them.
- JSON preserves old raw lexeme guard; HCL preserves old decoded label16KiB guard, including escaped raw lexemes longer than16KiB.

## Remaining before completion

- Run newest tests first; fix genuine failures, format, run fresh engine units + focused public suites and strict Clippy after final edits.
- Finish/check admission proof and storage tests; recheck opener-before-retention, tiny source/work/model and callback cancellation controls.
- IMPORTANT: authored/generated/source provenance inventory updates were prepared but NOT applied (oversized full-manifest patch truncated). `provenance/plantuml-parser.json` still needs json/hcl/structured and grammar/output registrations; MIT source traces need new parser/grammar paths and wording.
- Append exact task10 proof to `provenance/plantuml-parser-admission.md`; preserve all authorized source hashes/grants and other entries.
- Run PRIVATE-target generator read-only `check`, fmt check and exact `node tools/audit-plantuml.mjs`; root owns complete16 gates, immutable43 baseline and native nested compartments.
- Independent SPEC/QUALITY review, fixes, final scoped commit and final report still required. Do not treat WIP head as reviewed completion.
