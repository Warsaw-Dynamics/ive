SPEC verdict: ❌ Issues found — the finite style/text checkpoint is substantially implemented, but admitted font sizes are not incorporated into sequence frame-kind/caption reservation. This violates Task 3D's measured frame geometry and nonoverlap contract (Important I1).

## Strengths

- `diagram-ports/engine/src/plantuml/style.rs:106`, `:297`, `:361`, `:395`: the shared preamble has explicit family/data ownership, finite selector/property/value admission, complete brace consumption, authored matching-property order, and explicit rejection of mixed skinparam/style declarations. The YAML keyed-row check precedes command extraction.
- `diagram-ports/engine/src/plantuml/creole.rs:43`, `:197`, `:220`: the byte cap precedes Creole lexing; symbolic pairing is a linear pass with literal odd markers, explicit activated-span nesting checks, and no PUA sentinel substitution. Resource-bearing/unsupported markup is rejected rather than emitted.
- `diagram-ports/engine/src/plantuml/text.rs:31`, `:114`, `:166` and `diagram-ports/engine/src/plantuml/svg.rs:121`: cache identity includes complete run style and label mode, the render shares one request counter, host callbacks have cancellation checks on both sides, and measured text/font segmentation is reused for painting. The aggregate request test at `text.rs:268` checks the actual 4096-call frontier.
- `diagram-ports/engine/src/plantuml/chrome.rs:10`: document regions compose around the completed scene with translated primitives and preserved ownership rather than entering family models as synthetic entities. `diagram-ports/engine/tests/plantuml_styles.rs:262` exercises multiple distinct families and measured canvas containment.
- `diagram-ports/viewer/src/svg/tests.rs:5` tests actual installed-font/usvg shaping, significant spaces, decoration, owning-node containment and a raster pixel for activation/note layering. The narrowly authorized stable layer correction at `diagram-ports/engine/src/plantuml/sequence.rs:429` preserves foreground events while moving activation rectangles behind them.
- `diagram-ports/provenance/plantuml-mit-1.2026.9.json:1437` and `diagram-ports/tools/audit-plantuml.mjs:18`: selected source additions have explicit paths/hashes/behavior/destinations/notices; the new stylesheet paths are narrowly enumerated rather than admitting the entire style implementation. No dependency or runtime resource-loader addition appears in the package.

## Critical

None found in this task-scoped review.

## Important

I1 — `diagram-ports/engine/src/plantuml/sequence.rs:476`, `:660`, `:670`, `:675` (integration with the new configurable measured lines at `diagram-ports/engine/src/plantuml/text.rs:166`): frame geometry still assumes the old small fixed font. Opening a frame reserves a constant 40 units above its anchor; the caption starts at anchor−20, the kind at anchor−40, and the top border at anchor−24. The kind is measured only during drawing, after row allocation, and its dimensions do not determine those positions. Both texts now receive the full admitted body font size, so their 20-unit separation is insufficient.

The existing native `styled-sequence.png` visibly has the 18px `group` word intersected by the frame's top border. This is actual text/stroke overlap, not an intentional strike decoration: its run has no strike, and the intersecting line is the rectangle border. A focused 96px probe also succeeds while emitting `group` and `Checkout` at the same x=421.08 with baselines 269.8667 and 289.8667, only 20 units apart, and the frame top at y=197.6. Their conservative measured boxes overlap by 89.6 units. Thus the problem extends beyond cosmetic frame shape fidelity to overlapping admitted labels. Existing native containment tests omit `frame` from the checked owning kinds and use no group in their styled sequence body, so they do not cover this condition.

Measure the frame kind before row allocation, reserve kind and caption boxes using their measured dimensions, and derive border/text positions from those boxes with a defined gap. Add a focused regression covering the existing 18px capture case and a large admitted font with both kind and caption; assert text/border and text/text nonoverlap in actual shaping, not only overall canvas containment. Do not fix this by a larger fixed offset or by expanding the canvas.

## Minor

M1 — `diagram-ports/vendor/merman-render-0.9.2/src/environment.rs:585`: retained test, Clippy and release logs all contain the existing deprecated `Atomic::fetch_update` warning. The report discloses it accurately and no new product warning was found. This is inherited vendor noise, not a Task 3D blocker or a request to expand this patch into vendor maintenance; track it for the next authorized vendor update. Evidence: `/tmp/ive-plantuml-3d.QOjUH3/tests-final.log:1`, `clippy-final.log:1`, `release-final.log:1`.

## Cannot verify from this diff

- The unchanged hard worker watchdog/memory enforcement, source-wrapper directive rejection, terminal resource validator and all baseline limit constants are outside the task package. The controller reports fresh passing worker/safety checks; this review does not independently re-certify those unchanged implementations.
- Source/JAR byte identity and all 110 exact header notices are not derivable from a diff of the manifest. The controller reports independently matching all entries; I checked the new trace structure and narrowed allowlist, and did not duplicate that audit. Full upstream syntax/styling/pixel parity is explicitly outside this checkpoint.

## Checks

- Read the exact task brief and complete implementer report before reviewing the prepared BASE `6ec10226db28f084d627aad53276b51794009228` → HEAD `0fe389716232d3403940f0b145f15de0a58a4bfe` package. Read its 3901 lines in bounded passes; recovered the documentation fragment truncated by the first output. No raw Git diff derivation, product edits, index/HEAD mutations or subagents.
- Named outside-hunk risk: configurable font sizes versus sequence frame reservation. The package stops inside `allocate_rows` and omits `event_extents`/frame drawing; inspected the focused `sequence.rs:440–710` region to evaluate that risk. No broad codebase crawl.
- Inspected the existing native capture `/tmp/ive-plantuml-3d.QOjUH3/styled-sequence.png`; did not regenerate captures or start a GUI instance. Consulted the prepared style semantics for the exact native escape/geometry contract.
- Ran one new doubt probe using the already-built release viewer: `diagram-viewer --render /tmp/ive-task3d-review.GYd1u0/frame96.puml --output /tmp/ive-task3d-review.GYd1u0/frame96.svg` (exit 0). Its deterministic export proves the fixed baseline separation; native 18px overlap is separately evidenced by the existing capture. Probe files are outside the checkout. No tests or broad suites were rerun.
- Inspected retained verification summaries and warnings: the log's Rust groups sum to 268 passing tests with zero failures/ignored tests. The controller additionally reports seven hostile-worker scenarios, strict fmt/release/Clippy/deny and both audits/eight Node guards passing; those suites were not repeated.

QUALITY verdict: Needs fixes. The parsing, typed measurement/paint integration and boundary tests are strong, but frame-kind measurement remains downstream of allocation and the new font contract exposes a concrete overlapping-label defect. Fix I1 before accepting Task 3D; M1 is nonblocking inherited noise.
