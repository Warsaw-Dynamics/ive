# Central Chen + JSON/HCL integration checkpoint

Starting central head: `e04c206` (UML/activity product `e9c1354` plus root progress commit).
Integrated product head: `435d73cf87f53d0b284535576ab7eddb14f79aa6` on `diagram-ports`.
Frozen inputs: Chen `c465f21954d998404e4aa6e1941fed98208f4327`; JSON/HCL `53dbad8` including its initial `e71d646` cutover.
Only the designated sole integration writer changed central product files during this step.
Product tree/index clean after commit; root-owned `progress.md` remains modified and unstaged.
The before/after progress diffs compare byte-identically (`/tmp/ive-integration2-progress-{before,after}.patch`).

Applied the Chen commit diff and the complete JSON/HCL base-to-tested-head diff using indexed three-way application.
Seven unique shared paths required explicit conflict resolution across the two applications:
`mod.rs`, `parser/mod.rs`, `lexer.rs`, generated manifest, parser inventory, admission appendix and state.rs test.
Dispatch keeps sequence/state/files/board/mindmap/WBS/UML/activity and adds Chen/JSON/HCL before the legacy layout bridge.
HCL lowering retains its tested placement before that bridge; YAML retains its separate legacy path.
One shared fixed BodySource view admission remains; every optional parser route uses reborrow.
Parser module registration, authored/generated inventory, MIT trace destinations and admission appendices are unioned.
The central Cow error API and preexisting state assertion are byte-identical to the starting central head.
Legacy lexical command scanning/quote helpers lose their final caller after Chen; remove them while retaining UML Token/arrow_at.
Family models/layout code and tested branch parser implementations are preserved; no fallback/recovery/golden changes.

Mechanical generation/check used cwd `/tmp`, absolute central generator manifest and private target
`/mnt/data/Projects/ive/target/diagram-generator-integration`, with the outer shared build lock.
`cargo run --locked --offline --manifest-path /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml -- generate`
Result:Generated14 parsers. The same command with `-- check` verified14 reproducible outputs without tracked writes.
All14 actual generated SHA256 values equal their corresponding frozen central/Chen/JSON/HCL branch identities.
Combined manifest SHA256: `b769477f92345ac5f1aa7891a2fffff2e4f66a29eda8a90fcbac80c87721b1f3`.
Chen generated SHA256: `506e35386ae10da006b918f33715aee6f5e5b4b54fb4b1f7c03736c2c3da393c`.
JSON generated SHA256: `5f4403d249805aac57566ae9640b2374919715daa23e4547aba3fcf4de5402b4`.
HCL generated SHA256: `dc4e03150889360397fddc0a9b8e40363fa2a0276c033d4cae785233d157c66f`.
Unchanged generated tables preserve the task6/task10 actual-table witnesses and admission derivations.

Final focused command from central diagram-ports, under the outer lock/shared engine target:
`cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_chen_archimate --test plantuml_tree_data --test plantuml_inventory --test plantuml_constants --test plantuml_control --test plantuml_styles -- --test-threads=1`
454 passed:257 engine units +43 PlantUML +13 Chen/ArchiMate +44 constants +25 control +27 inventory +26 styles +19 tree/data.
Log `/tmp/ive-integration2-focused-tests.log`; zero failures.
This includes the unchanged activity500ms/60,000-row assertions and retained family geometry/resource/origin controls.
`cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings`:exit0.
Log `/tmp/ive-integration2-clippy.log`; existing audited-vendor deprecation warnings only.
`cargo fmt --all --check`:exit0. Both working and staged `git diff --check`:exit0 before commit.
Every Cargo command used `bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=<appropriate target>` outermost.
`node tools/audit-plantuml.mjs`:Verified220 exact MIT source/ZIP hashes,43 fixture hashes,43 native modules.

No independent broad review/full16/baseline/native/viewer checks were run, per the resumed user method.
No worker instance was started; no runtime processes or held build sessions remain from this integration step.
Root owns progress/ledgers and the later combined full gates/review/native captures.
