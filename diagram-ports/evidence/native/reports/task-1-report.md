# Task 1 report: bounded native Mermaid backend

Status: implemented and verified; ready for independent controller review. Product scope is only the independent `diagram-ports/` workspace. IVE root manifests and unrelated working changes were not edited. No subagents were spawned.

Product commit: `29353b5267431c112366bfc8e62aeb8fdf164aff` (`Add bounded native Mermaid engine with broad family corpus`). The task report is intentionally not part of the product commit.

## Backend selection and audit

- Exact published `merman = 0.9.2`, source provenance `.cargo_vcs_info.json` commit `8e303a43b548c9e40e73f0bc657925d66e7d4fb5`. Stable release preferred over the candidate development HEAD because its broad family surface was adequate. Declared upstream target is **Mermaid 12.1.0**; this is explicitly not full 12.2.0 equivalence.
- `default-features = false`; selected features: `all-diagrams`, `svg`, `layout-cytoscape`. No selected ELK/EPL package, math package/font, Node/JS/Wasm runtime, system-font discovery or ambient clock/random/timezone adapter appears in the inspected feature tree. `palette_math` is unrelated colour arithmetic, not the disabled formula feature.
- `merman-core`, `merman-render`, `dugong`, `dugong-graphlib`, `manatee` and `merman-unicode-linebreak` source attribution manifests were read, with their embedded legal files; no GPL or EPL implementation was opened. Dugong tracks Dagre 2.0.2, Graphlib tracks 2.2.4, rather than asserting identity with the separate dagre-es reference checkout. Manatee's COSE/fCOSE/layout-base/Cytoscape sources have MIT notices.
- Core/renderer source searches found no implicit file/network/process path in the selected operation. An explicitly constructed `NodeKatexMathRenderer` helper exists in upstream dependency code; this wrapper never constructs/exposes it and explicitly installs no math renderer. No icon registry, font discovery, image resolver or other ambient service is installed. Native font reading/export features are disabled. Caller callbacks are trusted host services and must be bounded themselves.
- The finite Merman interactive resource profile is explicitly configured and narrowed. Deterministic fixed runtime policy, strict parse policy, strict security level, classic/default presentation, deterministic IDs and Dagre are explicitly selected. Source-requested ELK/unavailable layouts fail before rendering. Specialized non-graph families retain their own layout algorithms.
- Licence/source audit against IVE's unchanged `deny.toml` passes. Runtime normal/build tree has 122 distinct package entries including this engine. Full dependency notices were generated with cargo-about using root policy/template, then combined with 6 selected-package source manifests and 19 unique complete embedded legal texts. Notice whitespace was normalized only at two trailing spaces and final EOF. The file retains fixture provenance from Mermaid documentation revision `6d8921f94f9a12b76586eddcf45a566080d4767e`.

## Exact public interface

`render(source: &str, options: &RenderOptions) -> Result<RenderedDiagram, RenderError>` is synchronous and headless. There is no GPUI dependency.

- `Language::{Mermaid, PlantUml}`; default Mermaid. PlantUml explicitly returns `UnsupportedLanguage` until its later task.
- `RenderOptions { language: Language, limits: RenderLimits, control: RenderControl, text_measurement: TextMeasurementPolicy }`, `Clone + Default`.
- `RenderControl` is the reexported `merman::OperationControl`; clone it and call `cancel()` from the host. Each invocation uses a child control with its own relative deadline, preserving parent cancellation/deadline inheritance.
- `RenderLimits { source_bytes: usize, svg_bytes: usize, model_items: usize, nesting_depth: usize, layout_work: usize, timeout: Duration }`. Defaults respectively: 262144; 16777216; 10000; 64; 5000000; 10 seconds. Accepted positive maxima: 16 MiB source; 64 MiB output; 100000 items; depth128; 100000000 work units; 120 seconds. Upstream interactive limits additionally bound model text, SVG elements and expanded tree size/depth.
- `RenderedDiagram { svg: String, language: Language, backend: &'static str, upstream: &'static str, family: String, diagnostics: Vec<String>, measurement: TextMeasurementReport, layout_work_units: usize }`. Identities are `merman 0.9.2` and `Mermaid 12.1.0`.
- `RenderError::{UnsupportedLanguage(Language), InvalidOptions(String), SourceLimit {actual,limit}, ExternalResource(String), UnsupportedLayout(String), InvalidSource(String), Cancelled(String), Backend(String), UnsafeSvg(String)}` implements Display/Error.
- Host metrics types are reexported: `HostTextMeasurer`, its request/result/error and measurement variants, `TextMeasurementPolicy`, identity/profile id, operation/phase/source/report, `TextStyle`, `TextMetrics`, `WrapMode`. Set `TextMeasurementPolicy::host_display(identity, Arc<dyn HostTextMeasurer>, TextMeasurementPhase::ALL)`. Merman's 4 routed phases cover Layout/Wrap/SvgBBox/ComputedLength and its 22 operation variants distinguish widths, extents, offsets, wrapping and whole text dimensions. Missing/invalid callback results fall back explicitly, with actual provenance in `measurement.entries()`. Defaults use approximate deterministic metrics; **no pgui font parity is claimed**. A long synchronous callback cannot be interrupted by cooperative cancellation.

## Runtime/output boundary

The engine accepts caller text and never loads external resources. Source guard rejects URL-like tokens (`://`, `file:`, `data:`, `javascript:`), image/attribute/import tokens and whitespace-obfuscated `img:` metadata. This is deliberately conservative: even URL text in a comment/label is rejected. Relative or encoded resources must also pass the terminal SVG resource validator before output is returned. No images or external navigation links are admitted.

Output uses Merman's **resvg-safe** pipeline and an extra XML check rejecting active/foreign/image elements, event/src attributes and non-fragment hrefs. CSS is not a browser execution surface; upstream safe finalization checks it and structural references. XML is parsed with DTD disabled by roxmltree defaults. Source/output/model/work bounds limit processing; a hard wall-clock/memory isolation boundary remains the viewer worker's responsibility, not a claim of this direct Rust function.

Two compatibility fixes were required and are implemented as a small custom preprocessor **before** the unchanged resvg-safe sanitizer/final validation:

1. Merman usecase emits XML-escaped stylesheet quotes. Its CSS sanitizer treats the serialized `&quot;` tokens as CSS and produced invalid output. The adapter decodes XML character-data quotes while retaining XML escaping for `&`/`<`; all safety passes still run afterward. The exact usecase fixture failed before this fix and passes afterward.
2. Info's upstream model defines a 400×100 canvas but its SVG intentionally omits a viewBox. Native output adds `viewBox="0 0 400 100"` so its version text has stable native dimensions. This is an explicit host adaptation, not a geometry-parity assertion.

## RED/GREEN and verification evidence

Commands were run from `/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports` unless otherwise noted.

| Command / stage | Actual result |
|---|---|
| First `cargo test --workspace` against a minimal returning-error stub | **RED**, all 10 initial behavioral tests failed for the missing implementation; compile succeeded first |
| Initial native wrapper `cargo test --workspace` | 10/10 behavioral tests passed |
| First 37-family corpus | 34 passed; info lacked viewBox; Ishikawa split a label across tspans; usecase CSS validation failed |
| Label-extraction correction | Normalize text-node whitespace; Ishikawa passes without product behavior changes |
| Compatibility preprocessors | All 37 family fixtures pass; info and usecase production failures fixed |
| Relative image regression, `cargo test --test mermaid` | **RED**, relative/whitespace-obfuscated `img : "private.png"` did not classify as ExternalResource; 14 others passed |
| Resource guard correction, final `cargo test --workspace` | **52 passed**: 37 family fixtures + 15 wrapper/security/limit/cancellation/determinism tests; 0 failures; no ignored tests |
| `cargo clippy --all-targets -- -D warnings` | Pass, clean. Earlier bool-comparison lint was corrected before this final run |
| `cargo fmt --all -- --check` | Pass |
| `cargo build --release` | Pass, including final guard; final rebuild 0.69 seconds after initial optimized build |
| `cargo deny --config ../deny.toml check licenses sources` | Pass; only unmatched allowances from reusing IVE's larger policy are warnings |
| `cargo tree -e features --prefix none` | Inspected; no selected ELK, math/native-font/system-adapter feature or runtime engine |
| `cargo about generate --config ../about.toml ../tools/third-party-notices.hbs --offline --locked` | Pass; expected warning that private diagram-engine has no separate license file; dependency texts preserved |
| `git diff --cached --check` from worktree root | Pass after formatting generated notices |

All admitted variants have expected rendered text, finite positive viewBox, SVG text elements, well-formed XML and successful **usvg 0.46.0** parsing (the pgui version), not merely a nonempty string. Native metrics/font painting were not compared to Chromium. No GUI/runtime integration or IVE window was started for this headless task.

## Actual coverage and limitations

37/37 native fixture variants pass: agentflow; architecture; block; C4; class; Cynefin; ER; eventmodeling; flowchart; Gantt; git graph; info; Ishikawa; journey; Kanban; mindmap; packet; pie; quadrant; radar; railroad IR/EBNF/ABNF/PEG; requirement; Sankey; sequence; state; swimlane; timeline; tree view; treemap; usecase; Venn; Wardley; XY chart; ZenUML. These correspond to 34 Cargo family selectors with four railroad dialects counted separately. All are admitted examples, not claims that every grammar feature is implemented.

Controller-provided original-renderer evidence, read at `/tmp/ive-diagram-oracle-report.md`, additionally records 36/37 of these exact inputs accepted by the original Mermaid12.2 core; ZenUML needs its optional original plugin and was not admitted by that oracle. The originals are in `/tmp/ive-diagram-oracles.JwN5H5/corpus-mermaid/`. Original eventmodeling/journey retain foreignObject despite htmlLabels:false and Wardley uses an additional serif font. No native geometry or raster diff has run; none is claimed.

Remaining gaps: 12.1→12.2 behavior changes; missing ELK/default-ELK equivalence; math/font-data solution; real pgui metrics and painting; full syntax/geometry corpus coverage; explicit in-memory assets/links; hard worker timeout/memory boundary. No full-port completion claim is made. The engine boundary and broad native foundation are ready for the viewer task and independent review.

## Review fix round 1: resource-bearing constructs, not source substrings

Fix commit: `a8394fe2c72c299d94b70f4aab30fcd4ba774d0e`.

The review correctly identified an unapproved restriction: the original global substring guard rejected harmless `metadata: loaded`, literal URLs and URL-bearing comments. That historical restriction in the first report/README is superseded by this fix.

Changes:

- Removed the global source-token guard entirely. After controlled parsing, inspect normalized semantic resource/style fields on nodes, classes, actors, edges, groups and entities; no label/comment or arbitrary user-payload traversal is performed. Explicit image metadata and external navigation fields fail before layout.
- Added a generated-SVG resource-admission pass before existing normalization and the **unchanged** resvg-safe sanitizer/finalizer. It examines actual href/src/CSS-bearing attributes and stylesheet content. `cssparser 0.38.0` tokenizes actual URL/import/image functions, including nested component values; ordinary quoted CSS strings and SVG text are not treated as resource references. Local fragment references remain admitted. Normalized style fields are also checked because upstream may silently strip an external paint declaration before emitting SVG.
- `cssparser 0.38.0` was already in the audited selected closure/notices; it is now a direct dependency. Cargo.lock only adds that existing package to diagram-engine's dependency list, with no new package or licence selection. Public engine API and existing terminal SVG validation remain unchanged.
- Label assertions in both test files now extract text only beneath SVG `<text>`, strengthening the safe-label regressions and existing family checks. README documents the corrected boundary.

Covering evidence, from the standalone directory:

| Command / stage | Result |
|---|---|
| `cargo test --locked --offline --test mermaid`, before implementation | **RED:** 17 passed, 4 failed. The four safe metadata/URL/comment/resource-word label regressions failed with `ExternalResource`; existing actual-resource checks still passed |
| First resource-aware implementation | Safe regressions passed; external CSS test exposed upstream silently stripping the paint declaration before SVG emission |
| `cargo test --locked --offline --test mermaid` after adding parsed style-field admission | **GREEN:** 21 passed, 0 failed, including all 4 safe regressions and real external image, relative image, navigation and CSS resource rejection |
| `cargo test --workspace --locked --offline` (full suite, once after final changes) | **58 passed:** 37 family fixtures + 21 behavior tests; 0 failures/ignored tests |
| `cargo clippy -p diagram-engine --all-targets --locked --offline -- -D warnings` | Pass, clean |
| `cargo fmt --all -- --check` | Pass |
| `cargo deny --config ../deny.toml check licenses sources` | Pass; only the prior unmatched IVE policy allowances warn |
| `git diff --check` scoped to standalone files | Pass |

One CSS-parser type-inference compile error was corrected during implementation; no unresolved failure remains. No GUI process was started. Unrelated uncommitted PlantUML analysis changes remain untouched. Existing cooperative-execution, approximate-font, version and fidelity limitations remain as previously documented.
