# Task 10 — tested JSON/HCL structural cutover checkpoint

- Worktree `/mnt/data/Projects/ive/.worktree/diagram-data-parallel`; branch `diagram-parser-data`.
- Base `0c5177e1f4cf50dcc23dc97ff3e0812b42d4070f`; WIP `e71d646376c54cc7c7cb030c1d6029193109e66f`; tested head `53dbad8`.
- Frozen product tree clean; only root-owned untracked `diagram-ports/.cargo/` remains untouched.
- User's resumed workflow is port → focused tests → fix; broad review and central integration gates remain deferred.

## Scope and preserved contracts

- Added `parser/{json,hcl,structured}.rs`, `grammar/{json,hcl}.lalrpop` and mechanically generated Rust/manifest entries.
- Generated JSON returns existing ordered `data::Value`; grammar owns objects/arrays/members/separators/numbers/EOF.
- Generated HCL returns existing `hcl::Document`; grammar owns same-line headers, split opening braces, blocks/fields/objects/limited arrays/EOF.
- Removed old recursive `data::JsonParser` and `hcl::Parser`; no production Line forwarding, retry/fallback, child-tree cloning or standard-HCL evaluator.
- `data::Value`, HCL models/lowering and datascene layout unchanged; geometry-test setup now uses new JSON facade, assertions unchanged.
- Dispatch parses both typed models and lowers HCL before the remaining legacy error bridge; structured failures map through frozen origins once.
- New `serde_json` use is only `from_str::<String>` for primitive JSON decoding; numeric finite/spelling checks use bounded native parsing.
- Ordered duplicates, scalar spellings, named blocks, single-block unwrap/multiple-block labels, owning rows, command-named blocks and title/style swallow negatives retained.
- JSON retains raw16KiB quote guard; HCL retains decoded16KiB labels and backslash-consumes-next-character behavior; exact128-byte numbers/exponent308 retained.
- Mirrored UML `6f1244c` shared Cow error refinement exactly. State unit pattern compares the same expected detail through `as_ref()`; no state production change.

## Observed RED/GREEN and checks

All Cargo used outer lock `/tmp/ive-diagram-build-locked.sh`; engine target `/mnt/data/Projects/ive/target`, locked/offline.

1. Original RED: `cargo test --locked --offline -p diagram-engine --lib should_generate_ -- --test-threads=1`:4 passed/2 failed at explicit unavailable JSON/HCL facade stubs.
2. Private generator from `/tmp`: `cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-data-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- generate`: generated11 parsers.
3. Earlier GREEN:203 engine units plus focused public suites before the interrupted session; saved WIP report documented newest unverified controls.
4. Resumed parser run:100 passed/3 failed. Two assertions counted both legitimate error location and used-at origin; changed to exact prefix and one expansion annotation. JSON short path has6 cancellation checkpoints, not7; corrected test range, no product cancellation failure.
5. New meaningful admission RED: both depth64 sources with source_bytes=their actual lengths failed retained-byte admission (JSON137456>37888; HCL137490>69888).
6. GREEN after symbol/reservation correction: both exact-source depth64 controls passed; depth65/opener-before-scan/model controls preserved.
7. Final: `cargo test --locked --offline -p diagram-engine --lib --test plantuml_tree_data --test plantuml_inventory --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`.
   Output `/tmp/ive-task10-focused-tests.log`:222 unit + constants44/control25/inventory27/styles26/tree_data19 =363 passed,0 failed.
8. `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`: exit0; existing audited-vendor deprecation warnings only.
9. `cargo fmt --all --check`: exit0. Generator fmt first rejected unsupported --locked/--offline flags; corrected offline-env command below passed.
10. Private target `/mnt/data/Projects/ive/target/diagram-generator-data`, `/tmp` cwd, same absolute manifest, `cargo run --locked --offline ... -- check`: verified11 reproducible parsers, no tracked output writes.
11. Private generator `CARGO_NET_OFFLINE=true cargo fmt --manifest-path ... --all --check`: exit0.
12. `node tools/audit-plantuml.mjs`: verified220 exact MIT source/ZIP hashes,43 fixture hashes,43 native modules; authored/generated inventories and source traces registered.
13. `git diff --check`: exit0 before final commit. No worker baseline/native/viewer/full16 gates or broad review.

## Actual generated admission proof

- Proof script `/tmp/ive-task10-stack-proof.py`, SHA256 `05ea91eb86cf492628ad3935acf0e998a2049826c6d10d33bb4047d1b5c04fc1`.
- Saved witness `/tmp/ive-task10-stack-proof.json`, SHA256 `6451a8c97a41695093b9f597a971276fa074d3492a25afbf0d08aa4435e2c86c`.
- JSON grammar/generated SHA256 `2a380b127a3ec90f980b8383d7509b08084c518dfd4e7e9cb1ea80e9b374fb50` / `5f4403d249805aac57566ae9640b2374919715daa23e4547aba3fcf4de5402b4`.
- HCL grammar/generated SHA256 `642ac69004b307a03f4d96193a9e9dd39c2311a86c51b8822a0bb896105d8a84` / `dc4e03150889360397fddc0a9b8e40363fa2a0276c033d4cae785233d157c66f`.
- Actual ACTION/EOF/reduction/goto closure stabilizes at JSON48states/107edges/16terminals and HCL40/69/10; ignoring production labels safely overapproximates all stacks.
- All-lookahead reduction-top graphs are acyclic: max8 JSON/5 HCL reductions; independent exhaustive depth0..4 controls agree.
- Weighted viable-prefix longest paths at0/1/2/8/64/65/128: JSON8/13/18/48/328/333/648 =5d+8; HCL6/8/12/36/260/264/516 =max(4d+4,6).
- Runtime uses actual observed nesting, symbol max-payload+tag64 bytes each; current size controls confirm FailureKind32/ParseFailure48.
- Before parser allocation it reserves state/symbol growth, diagnostic copies/epsilon growth, expected strings and actual token/adapter capacity; model vectors use fallible before-growth admission.
- Fixed generated-stack/diagnostic storage is separately admitted from the unchanged source-backed arena ceiling source_bytes*256, bounded by observed nesting (public max128).
- Tight controls admit exact remaining arena byte then reject+1; depth64 succeeds at exact source quota and depth65 fails before scanning retained contents.
- Intentional work accounting:9 units/JSON token and6/HCL token plus8/5 EOF reductions, concrete stacks and all-terminal diagnostic copy/reduction work; previous handwritten work totals naturally differ.
- Exact proof, formulas, storage rationale, unchanged scalar/comment/physical-line ownership and cancellation controls appended to own `provenance/plantuml-parser-admission.md`.

## Remaining central work / limitations

- Broad review is deferred by user instruction. Root owns shared-file integration, complete16 gates, immutable43 baseline hashes and native nested JSON/HCL owning-row checks.
- Shared unions include parser registration/dispatch, Cow errors, one state assertion adaptation, source/parser inventories and admission appendix; no dependency/vendor/license-source bytes changed.
- No known focused test failures remain. This is a tested implementation checkpoint, not a claim that central integration/native/baseline gates ran.
