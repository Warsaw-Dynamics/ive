# Native diagram workspace

An independent Rust workspace for a headless diagram engine and, next, a pgui viewer. IVE's root workspace is unchanged.

From this directory:

```sh
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo deny --config ../deny.toml check licenses sources
```

`diagram-engine` exposes `render(source, &RenderOptions) -> Result<RenderedDiagram, RenderError>`. Source is already caller-owned text: the engine reads no files, resolves no external resources and runs no subprocesses. Rendering is synchronous and belongs outside the UI thread. Cancellation/deadlines are cooperative; a host requiring a hard deadline must isolate rendering in its own worker process.

```rust
use diagram_engine::{render, RenderOptions};
let diagram = render("flowchart LR\n A[Start] --> B[Finish]", &RenderOptions::default())?;
assert!(diagram.svg.contains("<svg"));
# Ok::<(), diagram_engine::RenderError>(())
```

The backend is exact Merman **0.9.2**, derived from its published sources at commit `8e303a43b548c9e40e73f0bc657925d66e7d4fb5`. Its declared compatibility target is **Mermaid 12.1.0**, not the 12.2.0 reference checkout. Selected features are `all-diagrams`, `svg` and `layout-cytoscape`, with default features disabled. ELK, math/embedded math fonts, system-font discovery, system-clock/random/timezone adapters and binary export are disabled. Merman's optional Node/KaTeX helper is never instantiated or exposed by this adapter.

Graph layout defaults to Dagre. Source-requested unavailable layouts, including ELK, return an error. Other families retain their native specialized layouts, including COSE/fCOSE for architecture. SVG passes Merman's resvg-safe terminal validation and an additional XML/resource check. The adapter normalizes XML-escaped CSS before CSS processing (fixing usecase output) and gives info's recorded 400×100 canvas a viewBox.

All **37 admitted diagram variants** have executable, labelled, positive-size SVG fixtures, also parsed with pgui's usvg 0.46.0: agentflow, architecture, block, C4, class, Cynefin, ER, eventmodeling, flowchart, Gantt, git graph, info, Ishikawa, journey, Kanban, mindmap, packet, pie, quadrant, radar, railroad IR/EBNF/ABNF/PEG, requirement, Sankey, sequence, state, swimlane, timeline, tree view, treemap, usecase, Venn, Wardley, XY chart and ZenUML. This is family admission evidence, not proof of complete syntax support or pixel/geometry parity. The 12.2 differences and default-ELK layout equivalence remain backlog. PlantUML currently returns `UnsupportedLanguage`.

Default limits are 256 KiB source, 16 MiB SVG, 10,000 model items, depth 64, 5,000,000 layout work units and a 10-second cooperative deadline. The finite upstream interactive profile also bounds model text, SVG elements and expanded SVG tree size/depth. Caller limits are validated against hard maxima; zero or unreasonable values fail. `RenderOptions.control` can be cloned and cancelled; each invocation creates a child operation, so reusing options does not reuse a completed request's deadline.

`RenderOptions.text_measurement` defaults to deterministic approximate metrics. A host can set `TextMeasurementPolicy::host_display` with a named/versioned `HostTextMeasurer` for all four phases (`Layout`, `Wrap`, `SvgBBox`, `ComputedLength`). The request specifies text, style, operation, width and wrap mode. Missing/invalid host answers fall back explicitly and are recorded in `RenderedDiagram.measurement`; callbacks must themselves be bounded. No claim of actual pgui font parity is made until that adapter is wired and verified.

Resource admission checks normalized semantic image/link/style fields and generated SVG resource attributes, with a CSS tokenizer distinguishing actual URL/import functions from literal strings. URLs and words such as `metadata:` in labels or comments remain ordinary text. External images, navigation targets and CSS resources are rejected; fragment references stay local. The unchanged terminal SVG validator also checks relative/encoded resources before output can reach an SVG consumer. Arbitrary source assets need a later explicit in-memory resource design.

Fixtures are adapted from MIT Mermaid documentation; attribution and full upstream/dependency legal texts are in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). Lockfile licences do not cover manually translated algorithms/data by themselves: the notices also retain Merman's embedded-source manifests and their legal files.
