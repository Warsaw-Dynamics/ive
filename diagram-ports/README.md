# Native diagram workspace

An independent Rust workspace for a headless diagram engine and a standalone native pgui viewer. IVE's root workspace is unchanged.

From this directory:

```sh
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo deny --config ../deny.toml check licenses sources
```

Run the viewer from this directory:

```sh
cargo run -p diagram-viewer                              # built-in flowchart
cargo run -p diagram-viewer -- path/to/diagram.mmd        # selected local file
cargo run -p diagram-viewer -- --example sequence
cargo run -p diagram-viewer -- --render input.mmd --output diagram.svg
cargo run -p diagram-viewer -- --background --example pie --capture /tmp/diagram-frame.png
```

The language buttons and scrollable example list are keyboard focusable (Tab/Shift+Tab, Enter/Space). The viewer shows source, render progress/errors, fit and 25–400% zoom. Ctrl+0 fits, Ctrl+1 shows 100%, Ctrl+=/Ctrl+- zoom, and Ctrl+B toggles source. All 37 Mermaid corpus variants are available; PlantUML is explicitly unavailable until its backend is added. Selected source files are read-only regular UTF-8 files; symlinks to regular files work. The source panel previews up to 4,096 lines; the complete bounded source is rendered.

`--render FILE --output FILE.svg` exports SVG with deterministic metrics and initializes no GUI/display server. A failed render leaves an existing output untouched. `--background` opens with `focus: false` and never activates the app. `--capture FILE.png` captures GPUI's real last app frame after success or error, writes PNG off the UI thread, then quits; render/capture failures exit nonzero. This is native scene readback, so another window may cover the viewer. No browser or OS screenshot tool is involved. Test instances store no profile, settings, or user state, and their workers end when the window/app closes.

For build-cache reuse with the parent IVE checkout, set `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; the standalone dev profiles match IVE's optimized dependency/dev settings. Both `gpui` and `gpui_platform` are pinned to IVE's pgui revision **f5d5fa2f8f07b0be68442e694d22ed6064609e73**, with `font-kit`, `wayland`, `x11`, and macOS `runtime_shaders` platform features.

Rendering, source reads, native text measurement, SVG parsing and rasterization run in a single background service. There is one active render and one replaceable pending request; changing selection cancels the active worker, and generation checks discard stale render/raster results. The viewer retains parsed SVG for fit/zoom and displays `img(Arc<RenderImage>)` from pgui's full-color `SvgRenderer::parse_svg`/`render_parsed`, preserving text and fills (the monochrome icon-mask `svg()` path is not used).

The host starts its own executable in an internal worker mode; the engine itself still does not spawn processes. A framed stdin/stdout protocol sends source in memory, small JSON configuration/measurement/metadata frames, and raw SVG bytes. It bounds source at **256 KiB**, SVG at **16 MiB**, metadata/error/measurement frames at **64 KiB**, aggregate stdout at SVG limit plus **4 MiB**, stderr at **64 KiB**, and measurement RPCs at **4,096** with **16 KiB** text per call. A watchdog independent of pipe I/O and host callbacks enforces a **10-second hard deadline**, kills failed/cancelled/over-limit workers and reaps them. Stdin/stdout and stderr are drained separately. Unix children receive a **2 GiB address-space ceiling** via `setrlimit` before exec; Linux also sets a parent-death kill signal. Address-space enforcement and abnormal-parent cleanup were verified on Linux; macOS enforcement is not yet verified, and non-Unix platforms currently have no OS memory ceiling. Raster requests are separately capped at **4,096 pixels per axis and 4,194,304 pixels total** before allocation. These process/resource limits are not an OS filesystem/network sandbox; the audited engine API provides that no-I/O boundary.

Interactive text metrics cross the worker boundary through bounded synchronous request/reply frames while the watchdog stays independent. The pgui host actually shapes installed fonts for Merman's unwrapped **Measure**, **ComputedLength**, and **CanvasMeasureTextWidth** operations, using native advance widths and ascent/descent line metrics. It parses CSS family lists, chooses the first installed family (including quoted names), resolves generic sans/serif/monospace to installed native families, and ultimately uses the native system font when none match. Weight and italic style are honored. Requests outside the text/font bounds decline to the deterministic fallback. Specialized SVG ink/bbox, wrapping, offsets and other operations also decline explicitly; they are not mislabeled as native glyph bounds. The status bar reports actual native and fallback call counts: some families request only unsupported operations and therefore use deterministic layout throughout. This is partial native metric support, not browser/upstream geometry parity.

For display, pgui/usvg resolves installed SVG fonts (with its own fallback policy) and converts text to paths when parsing; resvg then rasterizes at bounded sizes. Exported SVG retains its text/font declarations, so another machine can select different fonts. Native layout font fallback and usvg painting fallback can differ, as can the backend's family-specific CSS and browser behavior; deterministic headless export and interactive native metrics can therefore produce different geometry.

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

`RenderOptions.text_measurement` defaults to deterministic approximate metrics. A host can set `TextMeasurementPolicy::host_display` with a named/versioned `HostTextMeasurer` for all four phases (`Layout`, `Wrap`, `SvgBBox`, `ComputedLength`). The request specifies text, style, operation, width and wrap mode. Missing/invalid host answers fall back explicitly and are recorded in `RenderedDiagram.measurement`; callbacks must themselves be bounded. The standalone viewer supplies the partial native adapter described above.

Resource admission checks normalized semantic image/link/style fields and generated SVG resource attributes, with a CSS tokenizer distinguishing actual URL/import functions from literal strings. URLs and words such as `metadata:` in labels or comments remain ordinary text. External images, navigation targets and CSS resources are rejected; fragment references stay local. The unchanged terminal SVG validator also checks relative/encoded resources before output can reach an SVG consumer. Arbitrary source assets need a later explicit in-memory resource design.

Fixtures are adapted from MIT Mermaid documentation; attribution and full upstream/dependency legal texts are in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). Lockfile licences do not cover manually translated algorithms/data by themselves: the notices also retain Merman's embedded-source manifests and their legal files.
