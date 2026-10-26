# Task 15 — tested packetdiag generated grammar checkpoint

- Worktree `/mnt/data/Projects/ive/.worktree/diagram-packet-parallel`; branch `diagram-parser-packet`.
- Base `3d3c35ae6dedf135cb096f07aa63cfc13100bb46`; tested head `a08bf48d3ef6cd068591258d0614c60d2514597e`.
- Tracked product tree clean; only identical audited Merman cache overrides in untracked `diagram-ports/.cargo/config.toml`, intentionally uncommitted.
- Port → focused tests → fix workflow followed; broad review and central integration remain deferred. Central product/progress.md untouched.

## Scope and preserved interfaces

- Added `parser/packet.rs`, `grammar/packet.lalrpop`, mechanically generated output and exact provenance/inventory registrations.
- `parse(BodySource, &mut Budget) -> Result<packet::Packet, RenderError>` parses before the remaining legacy layout/error bridge and maps structured origins once.
- Grammar owns direct/canonical bodies, complete braces/newlines/EOF, configuration assignment and optional single semicolon, inclusive/unit/auto ranges, mandatory label gap and complete attribute lists.
- Removed old production brace tracking, configure/field/attribute source splitting and numeric source parser. Remaining Line bridge is cfg(test)-only setup for preserved packet/scene tests.
- Config, Direction, Field, Fragment and Packet definitions remain unchanged; typed semantic inputs/configuration/attribute enums assemble the same models.
- Original `fragment` function byte-identical, SHA256 `ce07abf2975adcd42f6308ed3de051ebb378ca98db5b88d02391322ffb81ddb6`.
- Entire `packetscene.rs` byte-identical, SHA256 `dd07b5b7edb2c3cf82635bf80127515726b593f427a83c3ac1409dc31e09feae`.
- Retained5/8/7 payload conservation, one label per field, symbolic fragment field indices, proportional widths, RTL/ticks, variable/equal heights and row-clearance assertions unchanged.
- Final configuration applies before projection; auto positions follow prior declared ends. Sparse absolute starts never allocate gaps.
- Raw labels/comments/constant payloads remain lexical; attributes begin at the first reserved `[`. No cloned parser/label AST per fragment, retry, fallback or partial-success rendering.

## Observed RED/GREEN and verification

All Cargo commands used outer `/tmp/ive-diagram-build-locked.sh`; shared engine target `/mnt/data/Projects/ive/target`, locked/offline except fmt's supported CLI.

1. Structural RED: `cargo test --locked --offline -p diagram-engine --lib should_generate_ -- --test-threads=1` →8 passed/2 failed at explicit unavailable packet facade stub (`/tmp/ive-task15-red.log`).
   Positive contracts require direct/canonical inclusive/auto fields, complete attributes, final configuration and actual payload projection; malformed negatives were controls, not claimed RED.
2. PRIVATE generator `/mnt/data/Projects/ive/target/diagram-generator-packet`, cwd `/tmp`, absolute manifest `/mnt/data/Projects/ive/.worktree/diagram-packet-parallel/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml`.
   `cargo run --locked --offline --manifest-path ... -- generate` → generated18 parsers. Initial named/anonymous optional-symbol syntax was corrected before successful grammar generation.
3. First GREEN:3 new packet facade tests, then283 engine units +122 focused public tests passed after old source splitting was removed.
4. New controls initially16 passed/2 failed: cancellation test assumed9 checkpoints (actual8), and unknown-attribute reduction occurred too late to preserve Unsupported details at a frozen definition use.
   Corrected checkpoint range and moved unsupported-name rejection to its identifier reduction before values;18 packet-specific tests passed. This changed generated tables, whose proof was renewed mechanically.
5. Diagnostic RED: `--lib should_report_the_offending_packet_configuration_value` failed because generic generated error omitted `sideways` (`/tmp/ive-task15-diagnostic-red.log`).
   Added bounded failure-only offending-token enrichment after dropping the builder; no source slice is reparsed. GREEN included417 focused tests.
6. Contextual keyword RED: `--lib should_keep_value_keywords_unsupported_as_configuration_names` failed for `ltr=1` with InvalidSource instead of prior UnsupportedFeature (`/tmp/ive-task15-keyword-red.log`).
   Explicit key/value lexical roles restore original categories for ltr/rtl/true/false/len/height/packetdiag/unknown names without changing admitted syntax.
7. Final: `cargo test --locked --offline -p diagram-engine --lib --test plantuml_inventory --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`.
   `/tmp/ive-task15-focused-tests.log`:296 unit + constants44/control25/inventory27/styles26 =418 passed,0 failed.
8. Final strict `cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` →exit0 (`/tmp/ive-task15-final-clippy.log`), existing audited-vendor deprecation warnings only.
9. Workspace `cargo fmt --all --check` and private generator `CARGO_NET_OFFLINE=true cargo fmt --manifest-path ... --all --check` →exit0.
10. Private `cargo run --locked --offline --manifest-path ... -- check` → verified18 reproducible parsers read-only. Final grammar unchanged by last Rust-only keyword correction.
11. `node tools/audit-plantuml.mjs` → verified220 exact MIT source/ZIP hashes,43 immutable fixture hashes,43 native modules; exact parser/source inventories registered.
12. Diff whitespace check passed before commit; explicit progress.md diff empty. No worker native/viewer/baseline/full16 gates or broad review.

## Exact generated admission and boundary evidence

- Grammar SHA256 `152452e7dffd4fc08f1e374fdfdbe8b8e80d9302ceabe4b2c778c9032d9b5e85`; generated SHA256 `7eaf6090bb00d92e18ba37b74c9890c2cf81458d748afb9947865ac1634625dd`.
- Proof `/tmp/ive-task15-stack-proof.py`, SHA256 `8b441d2751bffb6d5e849a91cddac383fa186b4d760916a0257a12ba9cfd4b69`.
- Saved witness `/tmp/ive-task15-stack-proof.json`, SHA256 `83d6ffc51af755cfe0cfe7ca02bb713cea311f8a5c26c29c80260b319e70db15`.
- Actual ACTION/EOF/reductions and mechanically extracted goto arms exhaustively enumerate161 shifted stacks:69 states/28 terminals/max15 live/four reductions/nine expected terminals; diagnostic capacity18, growth peak27 i8 states.
- Before parser entry:24 runtime state/symbol growth slots each,48 two-byte diagnostic-copy slots, all-terminal expected String capacities, actual token/adapters/Builder and20 source-byte text/temporary reserve.
- Concrete generated symbol payload+tag bound40 bytes; Field48/Fragment32 measured. Independent three-capacity model storage allowance uses existing model_items quota for both fields and fragment projection.
- Model allowance is added to unchanged source-backed source_bytes*256 ceiling; actual item checks still precede every retained field/fragment and preserve original flat nesting policy.
- Intentional work accounting:5 units/token,4 EOF reductions, concrete stack slots plus28*(15+4) diagnostic copy/reduction work, alongside lexical/action/projection Budget checks. Public limits unchanged.
- Controls pass exact source quotas/nesting_depth1, sparse9,999,999 identity, eighth-digit refusal before unseen-label scanning, fifth-token refusal with capacity4 unchanged, pre-entry work bound and huge projection item bound.
- Tick/model, work/output limits, generated malformed primitive streams, lexical/generated/projection cancellation, unknown rotation/duplicate attrs/repeated colons/semicolons and suffix/EOF rejection all pass.
- Unknown feature and bounded generated syntax failures map once through exact raw Row spans; frozen definition-use lineage is retained. Full formulas and storage reasoning appended to own `provenance/plantuml-parser-admission.md`.

## Remaining work / limitations

- Parent/root owns central union, deferred broad review, complete16 gates, immutable43 baseline SVG parity and actual split-packet native/ink capture.
- Shared integration surface: parser dispatch/registration, generated manifest and provenance/admission appendices. Existing family code, dependencies/vendor/source grants, fixtures/models/layout assertions unchanged.
- No known focused failures remain. No claim that central integration/native/baseline gates ran. No app instances or external runtime processes started; paused before central merge.

## Central integration checkpoint

Worker `a08bf48` integrated into central base `fd0367f449ec48983d3a81a7a2fdf440037abe6c`.
Stable central commit `da590bc60089535f19649e825aeaa38e9636d6fb`; clean worktree.
Five textual conflicts resolved by additive union: mod.rs retains Salt/nwdiag preparse
and adds packet_model; parser/mod.rs keeps Salt/Creole/nwdiag and adds packet registration;
manifest retains nwdiag and adds packet entry; parser.json retains current scope/authorship,
all inventory entries and packet additions; admission.md retains both existing appendices
and packet appendix. MIT destination lists merge mechanically. No whole-side selection.
progress.md and main checkout untouched. Current scene/layout/source-limit code preserved.

Final combined engine command under outer lock and shared engine target, engine cwd:
`cargo test --locked --offline -p diagram-engine --lib --test plantuml
--test plantuml_chen_archimate --test plantuml_constants --test plantuml_control
--test plantuml_gantt --test plantuml_inventory --test plantuml_nwdiag --test plantuml_salt
--test plantuml_styles --test plantuml_timing --test plantuml_tree_data
-- --test-threads=1`:593pass,0fail (320units+43/13/45/25/17/27/17/18/26/23/19 public).
Log `/tmp/ive-task15-integration-tests.log` includes all contextual key/value controls,
Creole/literal data exclusions and current packet/Salt/nwdiag/model/geometry assertions.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings` passes,
`/tmp/ive-task15-integration-clippy.log`; existing Merman dependency warnings only.
`cargo fmt --all -- --check` passes empty `/tmp/ive-task15-integration-fmt.log`.
Private integration generator, cwd `/tmp`, outer lock, target
`/mnt/data/Projects/ive/target/diagram-generator-packet-integration`:
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- check`
passes21 reproducible parsers, no writes, `/tmp/ive-task15-integration-generator.log`.
`node tools/audit-plantuml.mjs` passes220MIT/43fixtures/43modules,
`/tmp/ive-task15-integration-source.log`. Outer-locked cwd `/tmp`
`python3 /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/tools/audit-plantuml-generator.py check`
passes67 archives/licenses/notices `/tmp/ive-task15-integration-closure.log`.
`git diff --cached --check` passes empty `/tmp/ive-task15-integration-diff.log`.

Read-only assertion comparison `/tmp/ive-task15-integration-preservation.log` verifies:
all20 prior generated manifest entries and actual Rust outputs unchanged; packet21st;
all47 prior authored-file and21 generated-inventory entries preserved; four packet worker
implementation/grammar/output files byte-identical to tested a08bf48; Creole/text/literal
data/Salt/nwdiag production unchanged; exact fragment function and scene equal central base.
Scene SHA256 `dd07b5b7edb2c3cf82635bf80127515726b593f427a83c3ac1409dc31e09feae`.
Fragment slice SHA256 `1412bbf896bed83eeb6a8824e969e52d23d5646bb8232f0a0d3bd83b3c773ad5`
includes trailing separator whitespace through the next cfg(test), identical on both sides.
Packet grammar/generated hashes remain the worker hashes above. Combined manifest SHA256
`59e87d2612efee01e077484704af2ff835ad408435dd988b46940b5ee1f60d7f`.
All final commands exit0; no integration test failure or further product fix required.
No broad review/full16/baseline/native run; root owns these remaining gates. Paused.
