# Task 2 viewer preparation — 2026-10-22

Status: ready for implementation when the controller releases Task 2 after Task 1 review. This preparation changed no product files, installed no dependencies, ran no build, started no app, and made no commit.

## Scope and inputs read

- Worktree: `/mnt/data/Projects/ive/.worktree/diagram-ports`.
- Read full root README, AGENTS.md, `.superpowers/sdd/2026-10-22-diagram-ports-plan/task-2-brief.md`, implementation design, Task 1 brief, current headless engine API, and cached source APIs.
- Deliver standalone native `diagram-viewer`; no IVE integration, browser, JS/Java/external renderer, xdotool, window activation, or user profile writes.
- Scope includes CLI validation, isolated worker bounds, language/example selection, source visibility, progress/errors, fit/zoom, explicit SVG export, no-focus startup, native app-frame capture, tests/clippy/release build, real app verification, clean process shutdown, and task-only commit.

## Exact GPUI APIs verified

Authoritative cached source root:
`/home/kb/.asdf/installs/rust/1.99.0/git/checkouts/pgui-ec323d2a127f3321/f5d5fa2`.

- Pin both `gpui` and `gpui_platform` to root IVE revision `f5d5fa2f8f07b0be68442e694d22ed6064609e73` from `https://github.com/Warsaw-Dynamics/pgui.git`.
- Match `gpui_platform` features `font-kit`, `wayland`, `x11`; target macOS additionally `runtime_shaders`. Keep engine GPUI-independent.
- `crates/gpui/src/app.rs:2069`: `App::svg_renderer() -> SvgRenderer` clones a renderer that can be moved to background work.
- `crates/gpui/src/svg_renderer.rs:190`: `SvgRenderer::parse_svg(&[u8]) -> Result<ParsedSvg, usvg::Error>` resolves fonts and text to paths.
- Same file line 195: `render_parsed(&ParsedSvg, impl Into<SvgSize>) -> Result<Arc<RenderImage>, usvg::Error>` preserves diagram colors/text. Retain parsed SVG across zoom rasterizations where practical.
- `crates/gpui/src/elements/img.rs:107`: `ImageSource: From<Arc<RenderImage>>`, so `img(image)` displays the full-color raster. The `svg()` element is an icon alpha mask and MUST NOT display diagrams.
- `SvgSize::Size(Size<DevicePixels>)` fits width while retaining intrinsic ratio; `ExactSize` can distort; `ScaleFactor` adds GPUI's 2x smoothing factor. Bound requested width, height and total pixels before rasterization. The library caps dimensions at 8192, but an explicit app pixel-area cap is still needed.
- `crates/gpui/src/window.rs:2906`: `Window::render_to_image() -> anyhow::Result<image::RgbaImage>` captures its last rendered scene without OS capture or focus. Root `src/window_capture.rs` demonstrates PNG encoding with `image` 0.25.10.
- `Window::on_next_frame` callbacks run before the next frame draws. Chain two callbacks after Ready/Error state updates before capturing; this is the same timing pattern documented in root `src/main.rs`.
- Background launch must set `WindowOptions { focus: false, .. }` and skip `cx.activate(...)` entirely.
- Accessibility exists in this exact fork: `.id(...).role(gpui::Role::Button).aria_label(...).accessibility_id(...)`; `.tab_stop(true)`/focus handles and key bindings provide keyboard interaction. Stable accessible identifiers should be assigned to language, example, fit, zoom, status, and source controls. `gpui::Role` is re-exported; no separate AccessKit dependency is needed.

An initial exploratory read of `/mnt/data/Projects/gpui-zed-main` was discarded as authoritative because it is a different HEAD; all APIs above were checked against the exact cache.

## Current engine interface and limits

`diagram-engine::render(&str, &RenderOptions) -> Result<RenderedDiagram, RenderError>` is synchronous and cooperative; run it in the child only.

`RenderOptions` fields: `language`, `limits`, `control`, `text_measurement`.
`RenderedDiagram` fields: owned `svg`, `language`, static `backend`, static `upstream`, `family`, `diagnostics`, `measurement`, `layout_work_units`.

Current audited `RenderLimits::default()` is source 256 KiB, SVG 16 MiB, model 10,000 items, nesting 64, layout 5,000,000 units, timeout 10 s. The controller explicitly confirmed preserving 256 KiB rather than the brief's generic 1 MiB default. The viewer must enforce the same source cap while reading, before allocating the full file or spawning work. Default worker hard deadline stays 10 s (including protocol activity), with a short bounded cleanup interval.

Mermaid uses Merman 0.9.2 / Mermaid 12.1.0, deterministic configuration, Dagre, and resvg-safe SVG. PlantUML currently returns an explicit unavailable error. Expose the language selection honestly; its selection can show this error until Task 3 supplies the implementation. Refresh this file/API reading after Task 1 review because Task 1 is still being edited.

## Proposed modules and behavior

- `cli.rs`: parse `diagram-viewer [FILE]`, `--language mermaid|plantuml`, built-in `--example NAME`, `--render FILE --output FILE.svg`, `--background`, and `--capture FILE.png` (capture then quit). Validate conflicting modes/duplicates/missing values, unknown flags, output requirements, and file/example exclusivity before opening GPUI. Keep internal `--worker` exclusive from user-facing modes. Use OS-string paths, not lossy Unicode conversion. Prefer the small existing dependency set over adding a CLI framework.
- `worker.rs`: protocol DTOs, bounded stream reads/writes, child entrypoint, parent supervisor, deadlines/cancellation, memory limit, errors, cleanup. The child consumes source over stdin and must not open caller paths. Parent alone reads exactly the selected file.
- `app.rs`: source/example/language selection and immutable request snapshots; request generation; explicit `Rendering`, `Ready`, `Failed` states; source panel; provenance/measurement diagnostics; image and fit/zoom. Keep file reading, worker management, SVG parsing and rasterization off UI thread. On completion, update only if its generation matches current state.
- Additional focused `measurement.rs` and `capture.rs` are appropriate rather than making app/worker monolithic.
- Built-in examples should use Task 1 corpus via `include_str!` and a static named registry. Initial success examples: flowchart, sequence, class, pie; all corpus samples can be listed without claiming compatibility for failed families. Source display can be read-only, scrollable monospace text; the brief does not require implementing a full editor.
- Fit uses available diagram viewport and intrinsic aspect ratio. Zoom controls should expose 25–400% (or similarly bounded range), 100%, and Fit. Use a scrollable viewport at explicit zoom. Re-rasterize in background when scale changes; reject stale raster results too.
- Native capture must run through normal UI Ready/Error state, draw the same real widget tree, save the resulting window image, and quit automatically. Capture errors should produce a nonzero process exit and a self-contained stderr message. Test error capture as well as success capture.

## Worker isolation and resource contract

- Spawn the current executable in exclusive worker mode, never a shell or third-party renderer. Pass sources only through a bounded protocol. Keep stdin/stdout/stderr piped; drain stdout and stderr concurrently so neither can deadlock a child. Bound stderr independently (for example 64 KiB).
- Prefer framed protocol with a small JSON metadata/header frame and a raw SVG frame. JSON-embedding SVG can expand data up to 6x; do not confuse the 16 MiB SVG bound with an unconstrained encoded-response bound. Validate frame lengths before allocation and aggregate protocol bytes/call counts as well as per-frame limits.
- Parent hard deadline and cancellation must be independent of engine checkpoints and blocked pipe I/O. On timeout, protocol error, output excess or cancellation, terminate the child and always reap it. Use one active render and one replaceable latest queued request (or equivalent explicit concurrency cap), not unlimited background jobs after rapid selection changes.
- Apply Unix address-space/resource limits in the child before rendering; a 1–2 GiB address-space limit is a candidate that needs a measured launch check because the linked GPUI binary maps libraries even in worker mode. Keep renderer limits independent from memory limits. Verify Linux RLIMIT_AS enforcement with an actual child; do not claim equivalent macOS enforcement until checked.
- Use child-local `Command::env` and `CommandExt::pre_exec` where needed; do not mutate the parent process's global environment. Keep only async-signal-safe calls in pre_exec. Any platform without an effective memory ceiling needs an explicit documented limitation/fallback, never a silent hard-limit claim.
- A worker guard and quit cleanup must kill/reap active children on normal close and capture exit. Linux parent-death signal can strengthen abnormal-parent shutdown; do not rely on it for the normal lifecycle. Check no worker remains after timeout tests and after screenshot capture.
- Parent validates SVG frame length and protocol success before GPUI parsing. Rasterization should have its own bounded width/height/pixel budget rather than relying solely on the SVG byte limit. Current GPUI has a per-axis 8192 cap but that alone permits large allocations.

## Host text measurement: supported bridge, exact limitation

The backend DOES support safe callback injection and explicit fallback provenance:

- Engine re-exports `HostTextMeasurer: Send + Sync`, `HostTextMeasurementRequest`, `HostTextMeasurement`, `HostMeasurementResult`, `TextMeasurementPolicy`, `MeasurementProfileId`, `TextMeasurementProfileIdentity`, `TextMeasurementPhase` and reports.
- Merman `environment.rs:680`: requests name operation/phase, text, TextStyle, optional max width and wrap mode. Responses are Metrics, Length, HorizontalExtents, or WrappedWithRawWidth. `Ok(None)` declines an operation and records deterministic fallback. The response variant must match the operation.
- `TextMeasurementPolicy::host_display(identity, Arc<dyn HostTextMeasurer>, phases)` routes selected phases with deterministic fallback; measurement reports aggregate actual Host/Profile/Fallback usage.
- `App::text_system()` supplies `Arc<TextSystem>`. `PlatformTextSystem: Send + Sync` (cached `platform.rs:1509`), and TextSystem caches are synchronized. Native shaping can run on a background host service; it does not require UI-thread callbacks.

Concrete bridge: the child's HostTextMeasurer writes a bounded measurement request to stdout, flushes, and reads a bounded reply from stdin; the parent's worker supervisor serves it from the captured Arc<TextSystem> on background work. The hard watchdog remains independent and covers all RPC activity. Bound text length per query, total queries and aggregate text bytes; cache repeated queries. Map requested family/size/weight/style into GPUI Font/TextRun and split newlines before calling single-line shaping/layout. Start with operations whose semantics can be reproduced exactly (e.g. advance/computed length and unwrapped line metrics); decline wrapping/bbox operations whose contract is not implemented and surface fallback counts. Do not label approximated wrapping or advance widths as browser bbox parity.

README should say: interactive mode uses installed native fonts for supported host measurement operations with recorded deterministic fallback for the rest; headless SVG export/tests use deterministic measurement and need no GUI session; GPUI usvg resolves installed fonts and converts SVG text to paths while parsing for display; exported SVG retains its text/font declarations, so another host may choose different fonts. Host shaping and usvg/fontdb fallback can differ, so this is not upstream/browser geometry parity.

Risk: the required bounded RPC bridge adds real work. If a simpler design is chosen, it must still wire host measurement where safe and report exactly which operations fall back; silently keeping deterministic metrics everywhere would leave the explicit brief item unfinished.

Controller clarification received after the initial preparation: deterministic worker metrics are acceptable with an explicit limitation if host metrics cannot cross the process safely. The bridge above is a supported approach to evaluate, not a requirement to compromise the isolation/deadline contract. State the final implemented behavior precisely.

## Tests and verification sequence once released

1. Write behavioral tests first: valid/invalid CLI forms; real same-binary worker success, malformed source, unsupported language; deterministic timeout using a tiny deadline and output rejection using a tiny cap; child exit/protocol failure and guaranteed reaping. Add generation/stale-result tests for state transitions. Follow `should_...` and given/when/then conventions.
2. Implement the smallest protocol/CLI to make those tests pass before UI features.
3. Verify real Mermaid labels/colors and nonempty image geometry through native success capture; inspect the PNG with image viewing. Capture an invalid-source error frame and verify the failure state is visible. Exercise at least flowchart/sequence/class plus another corpus family.
4. Run standalone `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo build --release`, using explicit standalone manifest path. Update notices/dependency audit for added GPUI packages under the controller's existing licensing workflow.
5. All launches use `--background` and `--capture` auto-exit; no xdotool/window focus. Check process cleanup after each test/launch. Commit only Task 2 changes after controller-approved implementation/checks.

## Build/cache choice

- Root `/mnt/data/Projects/ive/target` is already ~36 GiB and caches this pinned GPUI in debug/release. Standalone `diagram-ports/target` is ~1.8 GiB and currently engine-focused.
- Prefer `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo ... --manifest-path diagram-ports/Cargo.toml` for viewer work after Task 1 finishes, avoiding two concurrent Cargo jobs on the same target lock.
- Standalone dev dependencies currently use opt-level 2; root uses dependency opt-level 3 and dev opt-level 1. Match the standalone dev profile to root to increase reuse, coordinating the Cargo.toml edit with the controller. Exact package versions/features still affect reuse: root test cache includes `gpui/test-support`, and GPUI's image/futures/shared dependencies must resolve compatibly. Avoid promise of zero rebuilds.
- Pin exact GPUI revision and image version as root; inspect Cargo.lock changes and complete selected dependency closure. Do not copy unrelated root workspace dependencies or make engine depend on GPUI.

## Environment and remaining risks

- Sandbox commands began failing before execution with `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`; read-only escalated commands succeed. No approval review rejection occurred. This is an environment issue, not a product defect.
- Task 1 currently changes product files; no preparation edits overlap it.
- Native capture needs a working display/GPU platform even while unfocused; headless SVG export must bypass GPUI startup.
- Host metric semantics, aggregate protocol budgets, native raster pixel bounds, worker kill/reap behavior and macOS memory-limit support need implementation-level checks.
- Task 2 is intentionally waiting for the controller's explicit release; implementation has not started.
