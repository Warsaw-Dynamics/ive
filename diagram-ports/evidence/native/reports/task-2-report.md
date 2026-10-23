# Task 2 — standalone native viewer and isolated worker

Status: implemented, verified and committed; ready for controller review. Baseline: `a8394fe2c72c299d94b70f4aab30fcd4ba774d0e`. Task-only implementation commit: **`618be6831a66a778052d2a7ee0190e8cf6460c0d`** (`Add standalone native diagram viewer and bounded render worker`). Post-commit status contains only the preserved unrelated PlantUML analysis edit. No subagent/reviewer was dispatched by this implementer.

## Delivered

- Added `diagram-ports/viewer` as the second member of the independent workspace; no engine API or IVE root workspace/product code changed.
- `diagram-viewer [FILE]`, built-in `--example NAME`, explicit `--language mermaid|plantuml`, `--render FILE --output FILE.svg`, `--background`, and native `--capture FILE.png` with automatic quit. CLI rejects ambiguous/incomplete/duplicate options. OS-string input/output paths are preserved.
- Native pgui app: language buttons, all 37 Mermaid corpus variants, read-only source, progress and visible errors, fit/100%/25–400% zoom and source toggle. Controls have stable accessible IDs, button roles, labels, keyboard focus/Tab navigation and built-in Enter/Space activation. Ctrl+0/Ctrl+1/Ctrl+=/Ctrl+-/Ctrl+B shortcuts are bound. Source preview is bounded to 4,096 lines; rendering uses the complete bounded source.
- Exact IVE pgui revision `f5d5fa2f8f07b0be68442e694d22ed6064609e73`, `gpui_platform` features `font-kit`, `wayland`, `x11`, plus macOS `runtime_shaders`. Dev profile now matches IVE (`dev` opt-level 1, dependencies 3); all final checks used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`.
- Full-color SVG display uses `SvgRenderer::parse_svg` and `render_parsed`, then `img(Arc<RenderImage>)`; monochrome `svg()` icon masks are never used. Parsing resolves fonts/text to paths; the service caches parsed SVG for later fit/zoom rasterizations.
- File reading, engine worker supervision, native shaping, SVG parsing and rasterization run off the UI thread. One dedicated service has one active render and one replaceable pending request; selection changes cancel the active worker. Generation checks suppress stale completion/raster state. Regular-file admission uses `O_NONBLOCK` on Unix, so a selected FIFO cannot hang opening or shutdown.
- Capture uses `Window::render_to_image()` after chained frame callbacks, with PNG encoding/writing off the UI thread. Background mode sets `focus: false` and skips app activation. No OS screenshot, xdotool, browser, user profile/state or IVE MCP/session mutation was used.

## Worker contract and safety limits

The same executable runs an internal exclusive `--worker` mode, initializing no GUI platform. Parent reads the selected file; source is sent in memory. The worker calls the unchanged headless engine and never opens caller paths or invokes third-party executables.

Frames: one tag byte + a big-endian `u32` payload length. Tags are config=1, source=2, measurement query=3, measurement answer=4, result metadata=5, raw SVG=6, error=7. Configuration/measurement/metadata use JSON; source and SVG are raw UTF-8 frames, avoiding JSON SVG expansion. Lengths and aggregate budget are checked before payload allocation. An SVG terminal response must be followed by EOF; trailing output is rejected.

| Boundary | Default/enforcement |
|---|---|
| Source | 256 KiB, preserving Task 1's audited stricter limit; bounded file read and worker admission |
| SVG | 16 MiB, engine and supervisor |
| Metadata/error/measurement frame | 64 KiB |
| Aggregate stdout | SVG limit + 4 MiB |
| Stderr | 64 KiB, separate concurrent bounded drain |
| Measurement RPC count | 4,096 |
| Per-query text/family/size | 16 KiB UTF-8 text, 1,024-byte family list, finite 1–256px font size |
| Wall-clock render | 10 s hard deadline; independent monitor polls cancellation/deadline and child exit |
| Worker address space | 2 GiB Unix `RLIMIT_AS` before exec, fail on setup error; Linux parent-death SIGKILL |
| Raster | max 4,096 pixels per axis and 4,194,304 pixels total, before pgui allocation |

Protocol I/O and stderr use separate threads, while the owning supervisor monitors independently. A blocked stdin write, stdout read, or slow host callback cannot hold that deadline monitor. Errors, cancellation and deadline paths kill and reap the child through an owning guard. Thread handles already completed are joined; a still-finishing pipe/native callback thread is detached after the child is reaped, so platform callback latency cannot convert the hard process deadline into a blocking wait. Native callback input is bounded. Normal app quit stops the render service and joins it in GPUI background work. No global parent environment variables are mutated.

Actual Linux memory enforcement was tested with a native test-only child: it observed the configured 128 MiB `RLIMIT_AS` and failed a 256 MiB `try_reserve_exact`; this is not an assertion about a mock or a documented constant. Fault tests record actual child PIDs and assert `kill(pid, 0)` yields `ESRCH` after each supervised failure/cancellation. A process listing after GUI captures also found no surviving viewer/fault/worker process.

These limits are process/resource boundaries, not an OS filesystem/network sandbox. The engine's audited API supplies the no-runtime-I/O guarantee. macOS address-space enforcement and abnormal-parent cleanup are not verified here; non-Unix platforms currently lack the OS memory ceiling, explicitly documented in README. Linux is the verified runtime target.

## Real native measurement and explicit fallback

The worker's named `pgui-native-unwrapped` / `f5d5fa2-v1` host profile sends bounded measurement frames to the parent's background protocol thread. Its `NativeText` owns a separate `WindowTextSystem` built from the host's `Arc<TextSystem>`; native platform text APIs are Send+Sync. The independent watchdog never waits for this callback.

Supported operations: **Measure**, **ComputedLength**, **CanvasMeasureTextWidth**. Measurements use real installed font shaping, native advance width, and native ascent/descent metrics; multiline Measure returns maximum line width, accumulated line metric height and line count. No browser ink-box parity is asserted. CSS family lists, quoting/quoted commas and case-insensitive installed names are handled; generic sans/serif/monospace choose installed concrete families, ultimately falling back to the native system font. Weight and italic style are applied. Resolved family diagnostics are retained with the result.

SVG bbox/ink extents, specialized offsets, wrapping and other unimplemented operations return `Ok(None)` and are recorded by Merman as deterministic fallback. Public GPUI typographic APIs can provide character bounds, but they do not directly expose a shaped glyph-ID ink-bounds query; generic all-font boxes or line heights were deliberately not reported as raw SVG glyph bounds. This remains an explicit compatibility limit, with no fixture measurements/constants in production.

Real app evidence:

| Corpus sample | Native calls | Recorded fallback calls |
|---|---:|---:|
| Flowchart | 12 | 9 |
| Class | 28 | 48 |
| Sequence | 0 | 31 |
| Pie | 0 | 4 |

Some families request only unsupported specialized operations and therefore retain deterministic layout throughout. Headless export/tests use deterministic metrics and need no display server. SVG painting uses pgui/usvg's installed-font resolver and converts text to paths on parse; exported SVG keeps text/font declarations. Native layout font fallback and usvg paint fallback can differ. README records these exact limitations and the distinction from upstream/browser geometry parity.

## TDD evidence

1. Added parser/worker API stubs plus behavioral tests before implementation. `rustc --edition 2024 --test diagram-ports/viewer/src/cli.rs -o /tmp/ive-viewer-cli-red-tests && /tmp/ive-viewer-cli-red-tests` showed three expected CLI failures (`CLI parsing is not implemented`); the invalid-options case already passed. The equivalent Cargo run also showed these failures.
2. Parser implementation: the same direct test command with `/tmp/ive-viewer-cli-green-tests` passed all four cases.
3. `cargo test --manifest-path diagram-ports/Cargo.toml -p diagram-viewer --test worker` against the supervisor stub failed all six expected behaviors: actual SVG render, invalid source, hard deadline, output cap, cancellation and executable-start failure. After implementation, all six passed against real same-binary workers and Merman output.
4. `cargo test --manifest-path diagram-ports/Cargo.toml -p diagram-viewer --lib viewport` showed three expected failures for fit/area cap/stale suppression before implementing those behaviors; all four viewport tests then passed.
5. Added test-only native hostile-child harness (`tests/worker_faults.rs`, `harness=false`); it exercises seven real subprocess scenarios without introducing production test hooks: blocked stdin, blocked stdout, stderr flooding, oversize declared frame, early exit, memory ceiling and active cancellation. All pass and verify PID cleanup.
6. Named-pipe source test initially failed with a 100ms receive timeout (`opening a named pipe blocked the source reader`), then passed after nonblocking regular-file admission. Headless export tests also confirm display-free SVG output and preserving an existing output on render failure.
7. Quoted-comma CSS family resolution initially failed (`Liberation Sans` vs requested installed `Font, With Comma`), then passed after quote-aware list parsing; installed-family order and generic serif cases also pass.

## Final checks and commands

All commands ran in `/mnt/data/Projects/ive/.worktree/diagram-ports`, unless noted:

| Command | Result |
|---|---|
| `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --manifest-path diagram-ports/Cargo.toml` | Pass: 58 engine + 21 ordinary viewer tests = 79, plus 7 real worker fault scenarios; doc-test targets pass |
| `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo clippy --manifest-path diagram-ports/Cargo.toml --all-targets -- -D warnings` | Pass; an earlier run found redundant async/unit binding and a large cache enum, all corrected |
| `cargo fmt --manifest-path diagram-ports/Cargo.toml --all -- --check` | Pass |
| `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo build --manifest-path diagram-ports/Cargo.toml --release` | Pass, final release build completed in 46.63s |
| `cargo deny --config ../deny.toml check licenses sources` (standalone cwd) | `licenses ok, sources ok`, unchanged root policy |
| `cargo about generate --config ../about.toml ../tools/third-party-notices.hbs --offline --locked --output-file /tmp/ive-diagram-viewer-rust-notices.md` (standalone cwd) | Pass; only expected private engine/viewer missing-license-file warnings |
| `cargo tree --manifest-path diagram-ports/Cargo.toml --edges normal,build --prefix none --format '{p}' --no-dedupe \| sort -u \| wc -l` | 485 distinct selected Linux normal/build package entries |
| Feature tree inspection | Mermaid all-diagrams/svg/layout-cytoscape retained; no selected ELK, Smetana or Graphviz feature/backend |
| `git diff --check` | Pass |

Notices were mechanically regenerated for the complete added GPUI/platform/dependency closure with the established root policy/template. The existing embedded translated-source/data and fixture legal section was preserved byte for byte: SHA-256 before and after `1e77db18cb2f3e75e112cb3136e84b4e223c341cc2b9c09c7d2cbf51b30d57b3`. No new algorithm or fixture was copied from third-party sources; built-ins reuse Task 1's attributed corpus. No selected GPL/EPL dependency or external runtime renderer was introduced.

## Native capture evidence and cleanup

Each used the real binary, `--background`, and `--capture`, and exited automatically. The implementer opened and inspected the full app PNGs (sequence additionally captured; flowchart/class/pie/error/malformed/release frame visually inspected). The controller also inspected flowchart/class/error frames independently during implementation.

- `/tmp/ive-diagram-flowchart.png`: complete colored flowchart with Review/Read changes/Tests pass?/Ship and edge labels.
- `/tmp/ive-diagram-sequence.png`: sequence corpus sample captured successfully.
- `/tmp/ive-diagram-class.png`: Animal/Duck, attributes, methods and inheritance visible.
- `/tmp/ive-diagram-pie.png`: titled colored pie with labels, percentages and legend visible.
- `/tmp/ive-diagram-error.png`: `--language plantuml` shows explicit unavailable-backend error; expected exit 1.
- `/tmp/ive-diagram-malformed.png`: actual selected Mermaid file `/tmp/ive-diagram-malformed.mmd` with an unclosed node label shows `Unterminated node label (missing ']')`; expected exit 1.
- `/tmp/ive-diagram-release-flowchart.png`: final release binary captured successfully with native/fallback count 12/9 and preserved colors/text; exit 0.

Representative exact launch:

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --example flowchart --capture /tmp/ive-diagram-release-flowchart.png
/mnt/data/Projects/ive/target/debug/diagram-viewer --background /tmp/ive-diagram-malformed.mmd --capture /tmp/ive-diagram-malformed.png
```

`ps -eo pid,ppid,args | rg '/target/(debug|release)/(diagram-viewer|deps/worker_faults)'` returned no matching process after captures. No user IVE session/window was driven, no profile/terminal was started, and no xdotool was used.

## Review notes and scope

- Remaining limitations: partial native metrics with explicit bbox/wrap fallback, source preview cap, unverified macOS memory enforcement/non-Unix no-memory-ceiling, no browser/upstream geometry parity. These are recorded in README and above.
- PlantUML stays explicitly unsupported; language-specific examples/filtering belong to Task 3. The controller explicitly asked to keep further UI additions out of this task.
- Preserved the unrelated modified `docs/superpowers/plans/2026-10-22-plantuml-port-analysis.md`; it is excluded from task staging/commit.
- Initial sandbox startup repeatedly failed with `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`; scoped read/build/test/commit commands used reviewed escalation. No automatic approval review rejection occurred.
- No publishing, pushing, IVE integration, extra dependencies outside the pinned native dependency closure, subagent, or self-dispatched reviewer.

## Fix round 1 — R1 native cache lifetime and R2 logical zoom

Accepted review: `/tmp/ive-viewer-review.md`, two Important findings only. Fix baseline `618be6831a66a778052d2a7ee0190e8cf6460c0d`; fix commit **`c3486fd0752681f18a06061468eb9794d7cfda58`** (`Bound native layout cache lifetimes and preserve logical diagram zoom`). Scope is limited to those findings, regression tests and corresponding documentation. Systematic-debugging and receiving-code-review instructions were read; root causes were checked before implementation against the exact pinned GPUI and current service, rather than inferred from RSS.

### R1 root cause and correction

Pinned `WindowTextSystem` owns `LineLayoutCache`; `text_system/line_layout.rs` inserts every new shaped text/font/size into current-frame maps and used-line vectors. Their frame cleanup is inaccessible through the public `WindowTextSystem` API. The service's original single `NativeText` consequently retained every distinct label for the whole viewer session and mixed font-resolution diagnostics across diagrams.

`NativeTextResources` now retains only the reusable `Arc<TextSystem>` and immutable installed-family list. Each uncached engine render obtains a fresh `Arc<NativeText>` containing its own `WindowTextSystem` and resolution diagnostics. That instance drops after rendering/report collection; a detached in-flight callback holds its own Arc safely until it finishes. Fit/zoom reuse the parsed SVG and do not create or retain native text caches.

New `tests/native_lifetime.rs` runs actual headless Linux platform shaping and real same-binary render workers, without new dependencies or an RSS threshold. The normal test renders two disjoint label sets and proves via Weak references that each completed native cache is gone, shared font resources survive between jobs, and those font resources disappear when their owning resources are dropped. The cancellation test shapes real text, pauses that callback, cancels/reaps the worker, verifies the old cache is still alive while the callback holds it, completes an independent current render/cache, then releases the old callback and proves its cache disappears. The paused test adapter lives only in the test utility.

RED: `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --manifest-path diagram-ports/Cargo.toml -p diagram-viewer --test native_lifetime` failed both ownership expectations against the original shared-cache lifetime (`completed native layout cache was retained by shared resources`; `current cache was retained after completion`). An initial full-label substring assertion was corrected to a token assertion because SVG text spans split the label, before confirming the intended second ownership failure. GREEN: both ownership regressions pass with per-render native sessions, including the strengthened shared-font drop assertions.

### R2 root cause and correction

The previous service computed `Ready.logical_size` from the already capped raster bitmap. For a 1000×10000 diagram, 100/200/400% all displayed 409×4096 at device scale 1; device scale 2 reduced the displayed size further. The toolbar still advertised the requested zoom.

`viewport::layout` now returns separate requested logical geometry and bounded raster dimensions. Percent zoom applies to the SVG's natural logical size; Fit applies to the logical viewport. Device scale affects only raster resolution. Existing axis/area raster bounds remain unchanged. The service forwards the requested logical size to `Ready`, and the canvas's non-shrinking scroll surface explicitly adopts those logical dimensions so centered overflow does not place enlarged content at inaccessible negative coordinates.

Regression results, all with raster at most 4096 per axis / 4,194,304 pixels:

| Natural diagram | Mode/device factor | Logical display | Raster |
|---|---|---|---|
| 1000×10000 | 100% / 1 | 1000×10000 | 409×4096 |
| 1000×10000 | 200% / 1 | 2000×20000 | 409×4096 |
| 1000×10000 | 400% / 1 | 4000×40000 | 409×4096 |
| 1000×10000 | 200% / 2 | 2000×20000 | 409×4096 |
| 1000×10000 | Fit in 800×600 / 2 | 60×600 | 120×1200 |

Logical dimensions must be finite, positive and at most 1,000,000 pixels per axis; an oversized request fails visibly instead of silently altering the zoom. Device scale must be finite, positive and at most 8. README records these scroll/layout limits and that capped bitmaps can look soft when displayed at large logical zoom. No tiled renderer, new CLI/UI feature, engine change or metric capability expansion was added.

RED: `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --manifest-path diagram-ports/Cargo.toml -p diagram-viewer --lib viewport` showed the tall logical-size failure `(409,4096)` vs `(1000,10000)`, scale-two failure `(204.5,2048)` vs `(2000,20000)`, and missing finite/logical-bound admission. GREEN: all eight viewport tests pass, including original bitmap caps/stale suppression, percentages, scale two, Fit and invalid/unsafe logical sizes.

### Fix-round verification and remaining limits

- Full standalone `cargo test` with the root target cache: **85 ordinary tests** (58 engine + 27 viewer) and the **7 hostile-worker scenarios** pass. This includes 15 viewer unit tests, 4 export tests, 2 actual-native cache-lifetime tests and 6 ordinary worker tests.
- Full `cargo clippy --all-targets -- -D warnings` passes; `cargo fmt --all -- --check` and `git diff --check` pass.
- Native release build passes. Generated tall source `/tmp/ive-diagram-tall.mmd` has a 140-edge downward chain. The real release viewer ran `--background /tmp/ive-diagram-tall.mmd --capture /tmp/ive-diagram-fixround1-tall.png`, exited 0, and reported **1963 native measurements / 141 fallback calls**. The tall Fit app frame was opened and inspected: the full chain fits vertically with source and controls visible. Percent zoom is established by the targeted geometry regressions; this capture does not claim manual pointer/keyboard interaction at each zoom level.
- Dependencies, selected features, lockfile and legal notices are unchanged; the passing Task 2 audit and preserved embedded-notice checksum remain applicable, so legal generation was not repeated.
- Existing explicit bbox/wrap fallback, soft capped raster at high zoom, source-preview cap and platform memory-enforcement limitations remain. Linux owns the new actual-native lifetime regressions; no claim of macOS native test verification was added.
- No subagent, external renderer, xdotool, focus activation, IVE profile/session mutation or unrelated analysis edit was introduced.
- Final `ps -C diagram-viewer -o pid=,ppid=,args=` returned no process. Post-commit status contains only the intentionally preserved unrelated PlantUML analysis edit; all six fix-round product files are committed. The final release rebuild completed in 2.91s; its final 1360×880 tall capture was inspected. No dependency/notice regeneration or unrelated extra checks were needed.
