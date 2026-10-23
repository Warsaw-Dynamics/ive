# Native Diagram Ports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Attempt broad native Mermaid and PlantUML compatibility and make the engines inspectable in a standalone pgui viewer.

**Architecture:** Independent workspace `diagram-ports/`: headless `diagram-engine`, standalone `diagram-viewer`. Audit existing Rust ports before reuse, with source and feature provenance preserved. Select native backend modules through a bounded request API and return resvg-compatible SVG.

**Tech Stack:** Rust 2024, owner's pgui (`gpui` + `gpui_platform`, revision `f5d5fa2f8f07b0be68442e694d22ed6064609e73`), audited permissive Rust rendering crates, SVG/resvg.

**Spec:** `docs/superpowers/specs/2026-10-22-diagram-ports-implementation-design.md`; upstream-analysis reference `docs/superpowers/specs/2026-10-22-diagram-ports-handoff.md`.

## Owner-directed parser amendment — 2026-10-23

The owner explicitly requires LALRPOP for PlantUML and rewriting the current ad-hoc parsers. Execute `docs/superpowers/plans/2026-10-23-plantuml-lalrpop-plan.md`, designed in `docs/superpowers/specs/2026-10-23-plantuml-lalrpop-design.md`, before remaining PlantUML-family breadth. Preserve already verified models/layouts/viewer and all safety/source constraints; historical parser implementation choices below do not override this direction. The canonical orchestration checklist is `/mnt/data/Projects/ive/progress.md`.

## Global Constraints

- Attempt full coverage, Mermaid first then PlantUML; report actual compatibility and remaining gaps honestly.
- Standalone viewer first; no IVE product changes or integration in this plan.
- Other permissive dependencies are allowed. No GPL PlantUML sources or EPL layout translations.
- No network, no filesystem access beyond what the caller passes in, no external tools at runtime in the engine.
- SVG output compatible with resvg/usvg; no foreignObject or external resources.
- Rendering runs off the UI thread and must be bounded; hostile inputs fail descriptively.
- Follow `AGENTS.md`; tests named `should_…`, with given/when/then sections.
- Preserve original copyright/notices and module/backend source provenance.

## Review Focus

- Feature unification must not re-enable excluded ELK dependencies.
- Valid-looking SVG must not conceal missing labels, unsupported syntax, foreignObject or external resources.
- Input preprocessor directives must not open local files, access the network or loop without a bound.
- Rendering failures, excessive output, stale asynchronous results and worker timeouts must remain recoverable.
- Family support counts must be backed by executed fixtures; upstream pixel parity needs separate evidence.

## Analysis findings and alternatives

See `2026-10-22-mermaid-port-analysis.md` and `2026-10-22-plantuml-port-analysis.md` for measured upstream pipelines/inventories. The research survey is maintained separately as `2026-10-22-diagram-rust-survey.md`. Existing native Rust ports may avoid years of fresh implementation; they must be audited rather than assumed compatible. Merman is the leading Mermaid candidate; `puml` is a PlantUML candidate pending provenance and runtime audit. TeaVM and embedded JavaScript remain comparisons only, since the selected direction is native Rust.

### Task 1: Bounded headless Mermaid renderer with broad family coverage

**Files:** Create `diagram-ports/Cargo.toml`, `diagram-ports/engine/{Cargo.toml,src/lib.rs,src/mermaid.rs,src/limits.rs,tests/mermaid.rs}`, `diagram-ports/fixtures/mermaid/`, `diagram-ports/{README.md,THIRD-PARTY-NOTICES.md}`. Additional focused modules are allowed.

**Interfaces:** Produces `Language::{Mermaid,PlantUml}`, `RenderOptions`, `RenderedDiagram` with owned SVG and language/backend identity, `RenderError` with Display/Error, `render(source: &str, options: &RenderOptions) -> Result<RenderedDiagram, RenderError>`. PlantUml initially returns an explicit unsupported-language error. Preserve this public interface for subsequent tasks; publish exact fields/defaults in the task report. Do not add a GPUI dependency to the engine.

- [ ] Audit and pin Merman's selected release/revision, licenses, native API, metrics hooks, runtime I/O and cooperative budgets; use `default-features = false` and the widest verified permissive family/SVG features. Prefer a stable published version if it preserves the required coverage; identify its upstream Mermaid target.
- [ ] Write failing behavioral tests for rendering a flowchart with both node labels, sequence messages, class members, invalid source, XML metacharacters, Unicode, source limits and rejected external resources; observe red before implementing wrappers.
- [ ] Implement the native rendering wrapper, explicit deterministic configuration, descriptive errors, safe SVG checks and limits. Use a built-in Dagre path, reject explicitly requested unavailable layouts rather than silently promising ELK.
- [ ] Add and execute one meaningful fixture per admitted family, using existing permissive source examples with attribution. Record families that fail and the reason; exercise broad coverage rather than stopping at the initial three.
- [ ] Run engine tests and clippy; inspect selected Cargo feature tree for any ELK/EPL package. Record commands/results and backend API/metrics/cancellation facts in the report. Commit only task files after checks.

### Task 2: Standalone pgui viewer and isolated worker

**Files:** Create `diagram-ports/viewer/{Cargo.toml,src/main.rs,src/app.rs,src/worker.rs,src/cli.rs}`, viewer tests; modify standalone workspace membership and README. Additional focused modules allowed.

**Interfaces:** Consumes Task 1's `render` API. Produces `diagram-viewer [FILE]`, `diagram-viewer --render FILE --output FILE.svg`, and a bounded worker protocol. Viewer can start with built-in corpus samples. A worker process of the same binary is acceptable as an isolation mechanism; no third-party external executable is allowed. Public headless engine remains GPUI-independent.

- [ ] Test CLI argument validation and worker failure/timeout/output-limit behavior before implementation. Use deterministic small test limits; actual defaults source 1 MiB, SVG 16 MiB, render timeout 10 seconds unless audited backend limits warrant stricter values.
- [ ] Implement the pgui host using the exact revision, platform features as IVE, and real SVG rendering; show language/example selection, source, progress/error, fit/zoom. Keep resource work off UI thread and drop stale results.
- [ ] Wire host text measurement if backend can accept it safely. State exact metric behavior and SVG font handling in README.
- [ ] Add explicit CLI SVG export and a background/no-focus launch option, plus app-frame capture or deterministic GPUI test capture for verification. Do not use xdotool unless no native alternative exists, and never activate windows.
- [ ] Run standalone tests, clippy and release build; launch the viewer against corpus samples, inspect a captured frame, confirm errors visible and shut down all test processes. Commit task files after checks.

### Task 3: PlantUML native backend, broad corpus, strict include boundary

**Files:** Create `diagram-ports/engine/src/plantuml.rs`, `diagram-ports/engine/tests/plantuml.rs`, `diagram-ports/fixtures/plantuml/`; modify engine dispatch, manifests, notices, viewer example selection and README.

**Interfaces:** Consumes shared `render` API and uses `Language::PlantUml`. Preserve caller API and viewer isolation. Input includes/resources can only come from explicit caller-owned data; initially reject includes if dependency cannot enforce an in-memory resolver.

Audit outcome: the current `puml` candidate failed the traceable permitted-source gate before any import. Follow [the fresh MIT-only native plan](2026-10-23-plantuml-native-plan.md). Reviewed stage 3A combines its common service spine, real sequence renderer and shared UML graph deliverables (detailed Tasks 1–3); subsequent stages add state/activity/tree/data, pure preprocessing/styles and the remaining distinct families. Each stage receives a scoped brief, source manifest, meaningful oracle tests and independent review. Candidate-only aliases and source are not reused.

- [ ] Audit `puml` provenance and runtime API. Disable default CLI/network/include features. If the library reads files or lacks safe resolver control, adapt a minimal local permissive fork or implement safe native layers from the authorized MIT source; do not enable unsafe behavior to gain a family count.
- [ ] Write failing tests for sequence, class, component and activity examples, Unicode/escaping, malformed wrappers and local/URL/preprocessor include attacks. Test output dimension/resource limits and invalid source diagnostics.
- [ ] Implement native rendering and preprocessing restrictions with descriptive errors, preserve source provenance and notices. Expose backend coverage accurately.
- [ ] Add and execute one meaningful fixture per candidate-supported PlantUML family and adversarial fixtures; document unsupported or semantically wrong families. Make viewer select and render both languages.
- [ ] Run tests/clippy, verify dependency closure/licensing, render samples in viewer and inspect capture. Commit task files after checks.

### Task 4: Fidelity evidence, coverage ledger, final integration verification

**Files:** Create `diagram-ports/COVERAGE.md`, a corpus report and comparison tooling under `diagram-ports/tools/`; modify engine/tests/docs/notices as necessary for verified compatibility defects. A narrowly vendored exact-source backend crate is allowed when the public API cannot repair the consumed typed model; retain per-file source hashes and a local patch ledger.

**Interfaces:** Consumes all existing corpus fixtures and native CLI. Produces reproducible family/behavior evidence, explicit reference-engine/version/config/font details, and documented residuals. Original tools are optional development oracles, never runtime requirements.

Execute in two reviewed stages: compatibility repairs first, final corpus/tooling/documentation second. Do not overlap product implementers. Read-only investigation found that released Mermaid 12.2's `mindmap.colorBy: depth` is silently ignored by the 12.1-targeted baseline; its public API cannot modify the consumed built-in semantic model. Font investigations also isolated usecase effective size and omitted tree paint-family differences. These are concrete follow-ups, not reasons to replace geometry with fixture constants.

- [ ] Repair reproduced `mindmap.colorBy: depth` before layout, retaining default branch coloring, structural rather than indentation depth, palette cycling, child-edge section semantics and cancellation checkpoints. Test exact public rendered output and typed semantics; preserve the exact imported-source provenance if a small core fork is required.
- [ ] Fix reproduced font-style defects only where measurement and painting can use coherent effective configuration; cover non-default configuration. Keep unsupported glyph-bbox/wrapping metrics explicit and never hardcode oracle fixture measurements.
- [ ] Test all fixtures through public rendering and resvg parsing, checking finite nonzero geometry, labels and no forbidden SVG content. Include negative and bounded-execution tests across both backends.
- [ ] Compare representative output against original Mermaid and PlantUML MIT oracles if the installed sources/toolchain can produce them without GPL input. Record missing reference infrastructure explicitly; do not invent geometry/pixel parity claims.
- [ ] Produce coverage matrix per upstream family, separating tested family admission from syntax/semantic coverage and layout fidelity, with commands and counts. Preserve exact pinned versions, source traces and third-party notices.
- [ ] Run standalone `cargo test`, `cargo clippy --all-targets`, `cargo build --release`, selected license checks and real viewer capture checks. Root IVE checks are required only if root product code/dependencies changed. Stop test processes.
- [ ] Obtain a fresh whole-change review, route concrete findings through one implementation fix wave and scoped re-review. Leave local commits/branch and a concise final status with the app launch command, evidence and outstanding full-port gaps; do not push/publish.
