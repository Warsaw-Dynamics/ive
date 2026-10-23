## Global Constraints

- User explicitly directs rewriting CURRENT PlantUML parsing with LALRPOP, not just a future-family pilot. New-family breadth waits for this plan's completion.
- Keep public render/options/result/error/control/measurement APIs unchanged. Keep the engine GPUI-free and caller-memory-only; no runtime files/network/environment/clock/locale/process/JVM/JS/browser/dot.
- Preserve defaults: source256KiB, SVG16MiB, model10,000, nesting64, work5,000,000, cooperative10s; host hard10s/2GiB Unix; labels16KiB, queries4096, finite coordinate magnitude1,000,000 and existing raster limits.
- Translate PlantUML semantics only from the authorized MIT1.2026.9 tree. Preserve exact source hashes, actual credits/grants, fixtures and Merman's audited656-file/four-patch baseline. Separately audit generator development dependencies.
- Generated productions own actual syntax and EOF. No Line* forwarding, source-string reparsing in semantic builders, parser retry/fallback, partial recovery-and-render or broad golden regeneration.
- Retain explicit lexical raw/comment/quote ownership, immutable eager literal evaluation/no-rescan, bounded origins, cancellation and exact supported/rejected subset behavior.
- Owner-directed acceleration: one product implementer per isolated worktree; independent families run concurrently under parallel-context.md, with one integration writer and fresh independent SPEC/QUALITY gate per task. Use apply_patch/TDD/AGENTS; generator output is mechanical, never hand-edited. Shared engine Cargo target `/mnt/data/Projects/ive/target`, outer build/test lock `/tmp/ive-diagram-build-locked.sh`; generator targets are PRIVATE per checkout as specified in parallel-context.md to prevent embedded-path executable aliasing.
- Baseline is reviewed4c197871caf992a1a852b7048311b4ee450d74ff or a documentation-only successor:488 Rust tests+seven actual worker faults, all strict audits. Existing model/layout assertions remain, even when their parser setup changes.

## Review Focus

1. Contextual keywords versus actual commands: old description archimate endpoints, timing robust/concise tracks, nwdiag title servers, Chen entity/relationship aliases/composites. Assigned Tasks5/6/12/14.
2. Original-source ownership versus expanded terminators: raw notes/actions/data, atomic comment delimiters, immutable definitions, inert resource-looking comments and active includes. Assigned Tasks1/2/4.
3. Generator-owned stacks/reductions and retained syntax before allocation: tiny limits, deep/wide input and cancellation inside lexing/iteration/actions. Assigned Task1 and every family's opener tests.
4. Significant whitespace and nonstandard data semantics: CRLF/BOM, Salt escapes/rows, YAML indentation/duplicates, HCL header lines and files trailing spaces. Assigned Tasks2/9/10/11/13.
5. Hidden residual parsers or misleading evidence: raw command tokens fed to legacy methods, changed goldens, unclassified styles/Creole/JSON/YAML parsing and stale generation. Assigned Tasks3/10/11/16/17.

## Common Verification and Report Contract

**Current execution override:** after the owner's speed request, parallel-context.md defines lean worker verification and concise reports. Complete gates below run once per combined integration batch; focused worker tests/private-generator/source checks and independent task reviews remain required. Baseline/Cargo-run examples/viewer/native are CENTRAL ONLY because unhashed executables can alias another checkout. This supersedes the earlier serial/per-worker-full-gate scheduling, not semantic/resource/legal/compatibility requirements.

For each task: write the named failing contracts first; run focused tests and record meaningful RED; implement the stated grammar/builder cutover; run focused GREEN then the complete gates below after the last edit; self-review, scoped commit and full report. Existing already-passing corpus cases are controls, not invented RED evidence. A minimal contract stub may make a new test compile and return an explicit unavailable error before the real grammar exists; it must not ship as a fallback.

Every cutover runs baseline checking and preserves these assertions in the baseline driver:

```js
assert.equal(current.sourceSha256, baseline.sourceSha256);
assert.equal(current.family, baseline.family);
assert.equal(current.svgSha256, baseline.svgSha256);
```

The only intended new-positive differences are the four documented boundary repairs. Add their source/expected semantics explicitly; do not replace existing fixture records. Preserve exact model IDs/topology/private geometry assertions as well as SVG hashes.

From `diagram-ports/`, with the shared target and locked/offline flags:

```sh
cargo test --locked --offline --workspace
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
cargo build --locked --offline --release
cargo deny --locked --offline --config ../deny.toml check licenses sources
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs tools/parser-baseline.test.mjs
cargo test --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml
cargo clippy --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all --check
cargo deny --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --config ../deny.toml check licenses sources
cargo run --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml -- check
python3 -B tools/audit-plantuml-generator.test.py
python3 -B tools/audit-plantuml-generator.py check
node tools/parser-baseline.mjs check
```

Task1 creates the last two tools. Audit the generator's separate lock/feature/license closure too; its checked-in outputs must rebuild offline after the initial authorized fetch. Genuine native-paint tests/captures, actual counts and cleanup remain per-cutover evidence; headless empty-PATH exports must work without the generator. Never rerun original Java/browser oracles just to reconfirm parser parity.

Reports live in this plan's controller-owned SDD workspace, one task-N-report.md per extracted brief. Include base/head, removed legacy syntax, real grammar entrypoints and model interfaces, RED/GREEN commands/output, source/generation/limit evidence, baseline/native results and residuals. Controller updates `/mnt/data/Projects/ive/.worktree/diagram-ports/progress.md`; implementers do not.

**Family interface rule (Tasks4–15):** each `parser/<family>.rs` exposes `parse(body: BodySource<'_>, budget: &mut Budget) -> Result<ExistingModel, RenderError>`, where ExistingModel is the task's named preserved model. Tree exposes separate `parse_mindmap`/`parse_wbs` entries returning the distinct tree models; files/board and timing/Gantt each have their own parse module. JSON/YAML return `data::Value`, HCL returns `hcl::Document` before its unchanged lowering. Structured internal parse failures are mapped through BodySource's origin document at these facades, not repeatedly formatted/reparsed.
