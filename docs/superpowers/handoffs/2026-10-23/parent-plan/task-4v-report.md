# Task 4V — host text-paint canvas containment

Base: `d07df148ac7d82eb2ff33ba5a1ce35fbe9c03d4b` (released, reviewed Task3A checkpoint).
Head: `9cca89835b6edf773866b002061c5e693ac8c598` (`Contain native diagram text paint in bounded host canvases`). Scoped staged diff passed `git diff --cached --check`; tracked worktree clean after commit. No push or merge.
Worktree: `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`.
Scope: standalone `diagram-ports/` host only. No engine, vendored source, fixture, worker protocol, native measurement capability or deterministic export changes. One implementer; no subagents or self-dispatched review. Independent controller review remains required.

## Delivered contract

`svg::DisplaySvg` is private to the viewer crate. `parse(source, cancelled)` owns the usvg0.46 tree, original parsed viewport and derived display canvas. `dimensions()` supplies natural logical dimensions; `render(raster_size, cancelled)` validates work and rasterizes that same tree through resvg0.46; `diagnostic()` reports expansion/preservation. The service's parsed cache now owns this type, retaining it for fit/zoom. `Service::start` takes only shared `Arc<TextSystem>`; it no longer accepts a separate opaque pgui SVG renderer. `Ready.display_note` carries the host-only message to stderr and the status bar. Metadata's existing diagnostics also retain it; host/fallback measurement counts and profile identity do not change.

Containment starts at root children with identity, composes ordinary group and flattened-text local transforms, and unions visible path fill/stroke bounds and bitmap glyph rectangles. Nested SVG image trees are traversed with the renderer's local transform convention, including their paths, not their cached outer image rectangle. Text descendants remain distinguished from non-text geometry. Nonzero fill/stroke opacity, group opacity, visibility, transparent gradient stops and mixed fill/stroke paint are handled. Original viewport sides stay exact unless text overflows that side; only those derived text edges are rounded outward. The root viewBox mapping stays in the original tree and is not applied twice. Raster root mapping is scale-to-canvas followed by translation from the canvas origin; node coordinates and source SVG are not rewritten.

The actual built-in tree using its default configuration now displays **133×138** instead of **122.08×138**, at canvas origin0,0 on this machine. Its test's pre-fix text extent reached132.556. These are observations, not production constants. The prep's explicit Liberation Sans129-pixel diagnostic was not substituted for the built-in case.

## Font, color and clipping policy

- `svg_fonts::options()` mirrors pinned pgui's empty asset source: system fontdb, missing generic-family repair only, default selection then SansSerif then first available face, Emoji_Presentation recognition and the exact platform emoji fallback ordering/glyph availability/used-face rules. No fonts/assets were imported. The options/database are initialized in background parsing and reused.
- Full byte-for-byte BGRA output comparisons against actual pinned `SvgRenderer::parse_svg/render_parsed` pass for Liberation Sans, DejaVu Sans, sans-serif, serif, monospace and a missing family. Each case contains colored gradient/opacity, colored text/stroke, italics, ligatures, combining marks, CJK and emoji. Separate known pixels assert opaque `[32,64,128,255]`, translucent `[31,63,127,128]`, and transparent `[0,0,0,0]`; conversion uses pgui's public premultiplied-RGBA-to-BGRA function, not an alpha-only proxy.
- The built-in tree compares every channel at4× with a test-only equivalent root-viewBox expansion rendered by pinned pgui, in addition to text containment and unchanged left-edge assertions. Translation/rotation/skew/stroke/combining/bitmap fallback cases use a fixed oversized CPU oracle canvas independent of the bounds computation to assert every painted pixel lies inside the derived canvas.
- Explicit clip paths, masks, filters or patterns preserve the original canvas and emit a visible skip diagnostic. Likewise, if any non-text paint extends beyond an original edge in a proposed expansion direction, the whole canvas stays unchanged. This is deliberately conservative; potential bbox exposure can preserve the canvas even where exact paint would not intersect the added strip.
- The required leftward-text/left-clipped non-text path counterexample preserves original dimensions and exactly matches pinned renderer pixels. Transparent/invisible non-text does not block safe expansion. Simple clip, mask, blur and pattern cases retain full original pixels.
- This is containment for the admitted normalization subset, not universal arbitrary-SVG text containment, text fitting inside nodes, native DOM bbox support or browser geometry parity. Bitmap bounds include possible transparent borders. Linux native typographic APIs still provide advances; all existing bbox/wrap fallback semantics remain truthful.

## Bounded work and cancellation

Existing worker10s/2GiB Unix limits,256KiB source/16MiB SVG framing,4096 measurement-query cap, per-render native caches, replaceable pending request and stale generation behavior remain unchanged. Existing cache-lifetime, cancellation, worker deadline and real hostile-child tests pass.

Host preflight admits at most16MiB SVG,100,000 XML nodes and depth128 before usvg conversion. It rejects malformed/nonfinite/extreme/singular transform attributes. Parsed traversal is iterative, bounded to100,000 visits, depth128,1,000,000 path points and4,194,304 cumulative bitmap decode pixels. Every bitmap is also at most4096 per axis/4,194,304 pixels. Bounds, composed transforms and display extent are finite and within1,000,000 coordinate/axis limits. Traversal checks cancellation throughout; parse/raster have before/after checks, and color conversion checks each row. No uncapped inspection raster is allocated in production.

`svg::raster_guard::check` performs another bounded traversal immediately before each actual raster allocation. It visits clip/mask/pattern/filter subtrees and repeated image occurrences. Every offscreen surface is limited to4096 per axis/4,194,304 pixels; cumulative conservative paint/surface work is limited to67,108,864 pixel operations. Counts include transformed paint rectangles plus AA fringe, per-occurrence bitmap decode, full destination scratch surfaces for embedded SVG images, actual scaled pattern tile sizes, retained group layers, four surface equivalents per clip/mask and sixteen per filter primitive plus merge inputs. No deduplication is invented where resvg repaints or redecodes. Convolution, lighting, displacement, morphology, tile and turbulence filters fail explicitly under the bounded raster policy; simple supported effects remain unchanged within the budget.

Pinned resvg inspection: `src/lib.rs` sets max_bbox to5×target axes; `src/render.rs` clamps ordinary/filter group surfaces to that region (potentially25×final pixels). `src/mask.rs`/`clip.rs` clone layer-sized surfaces. `src/path.rs::render_pattern_pixmap` allocates scaled pattern cells without the same target clamp. The new guard therefore does not mistake the final bitmap cap for an offscreen allocation bound. Oversized pattern/filter surfaces,70 nested opacity layers, and70 individually legal1px images enlarged to1024² are rejected in tests before rasterization. Portable generated tiny-PNG tests cover both aggregate decode and overdraw; actual Linux Noto Color Emoji additionally tests real glyph bitmap occurrences and states its installed-font prerequisite explicitly.

These are conservative admission/accounting bounds, not a formal process RSS ceiling or a new hard host timeout. usvg/font loading/resvg remain in-process library calls off the UI thread; cancellation cannot interrupt those calls internally. Existing engine worker hard bounds do not cover host library internals. Complex/large effect diagrams may now fail explicitly instead of allocating unchecked surfaces. Logical fit/zoom remains separate from physical raster caps and device scale; large-diagram tests preserve20,000 logical-pixel height with a capped bitmap.

## RED/GREEN evidence

All Cargo commands use `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` and standalone workspace `diagram-ports/` (or its manifest path from the worktree).

1. `cargo test --offline --manifest-path diagram-ports/Cargo.toml -p diagram-viewer --lib svg::tests -- --nocapture`: after correcting the initial usvg-default-font test setup to the pinned font policy, the actual built-in tree failed with **text paint ends at132.556, canvas ends at122.08**. Log `/tmp/ive-4v-red.log`. Canvas was still the original viewport; containment implementation followed that failure.
2. The first9 containment/color/policy tests became green. During this wave, explicit zero-determinant validation fixed the observed all-zero matrix failure (tiny-skia's simple inverse branch can return nonfinite components). Two test expectations were corrected: short `stroke` did not actually overflow the right side; separate translated raster canvases can differ by a few RGB units in bitmap resampling, so transformed tests assert paint containment while the unchanged-transform full-color comparisons retain exact byte equality.
3. `/tmp/ive-4v-guards-red.log`:12 tests,10 pass/2 fail before the fractional-side and bitmap-size corrections. Fractional original right edge incorrectly rounded60.25→61 when only left text overflowed; oversized bitmap validation was still a no-op. Both are green.
4. `/tmp/ive-4v-raster-red.log`, `cargo test --offline --locked -p diagram-viewer --lib before_`:4 tests,2 pass/2 fail with the pre-raster guard stub. Oversized/cumulative effects and repeated enlarged legal bitmaps were admitted. Implemented guard then yields **18/18 SVG tests green** in `/tmp/ive-4v-raster-green.log` (later full run includes the additional convolution case and Noto prerequisite assertion).
5. Additional tests around already-correct preserved behavior passed immediately: no independent RED is claimed for every safety/compatibility assertion. A transparent non-text fixture initially retained an implicit black fill; declaring `fill=none` corrected that test setup.

## Final verification

After the final production/test edits:

| Command | Result |
| --- | --- |
| `cargo test --offline --locked --workspace` | exit0;166 Rust tests +7 separate hostile worker scenarios |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | exit0 |
| `cargo fmt --all -- --check` | exit0 |
| `cargo build --offline --locked --release` | exit0; final incremental2.85s |
| `cargo deny --config ../deny.toml check licenses sources` | licenses/sources OK |
| `node tools/audit-plantuml.mjs` |32 MIT source/ZIP hashes,5 fixtures,12 modules verified |
| `node tools/audit-merman.mjs` |656 imported files,4 recorded patches unchanged |
| `node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs` |6 audit guards pass |
| `git diff --check` | pass |

Rust counts: engine-private15, compatibility11, Mermaid-family37, Mermaid/general21, PlantUML-public33, viewer-unit35 (18 added), export5, native-lifetime3, worker6. Hostile worker7 and Node6 are separate. Logs: `/tmp/ive-4v-tests.log`, `-clippy.log`, `-release.log`, `-deny.log`. Only the preexisting vendored Atomic::fetch_update deprecation warning remains. No root IVE product changes, therefore no root IVE test/build claim.

## Provenance and licences

Copied/adapted glue is explicitly attributed in `viewer/src/svg_fonts.rs` and standalone notices to pgui `crates/gpui/src/svg_renderer.rs`, revision `f5d5fa2f8f07b0be68442e694d22ed6064609e73`, original file SHA256 `fb12dae87c0cd4c5f5a1fbbd2f9548d52fde9440c21e3d576dfeb0233eda88a6`. Copyright2022–2025 Zed Industries, Inc.; Apache2.0 full original grant/licence remains in the existing gpui notices section. The source comment identifies the modifications: empty-asset specialization and shared system fontdb/resolver extraction.

Five exact direct viewer edges reuse the audited closure: usvg0.46.0, resvg0.46.0, regex1.13.1, ttf-parser0.25.1, svgtypes0.16.1. Cargo.lock adds only those viewer dependency edges, no package/version. `cargo about generate --config ../about.toml ../tools/third-party-notices.hbs --offline --locked --output-file /tmp/ive-4v-rust-notices.md` passes with only existing private-package warnings. Generated dependency legal text matches the retained section after normalizing line endings/trailing whitespace; original dependency/embedded legal content is unchanged. Only the new adapted-source prelude is added. No new external source assets, fonts, forbidden renderer, browser/JVM or engine dependency.

## Actual native captures and cleanup

Final release binary commands all used `--background ... --capture /tmp/ive-diagram-4v.PKYhgx/<name>.png`, and autoquit. All three real1360×880 app-frame PNGs were opened and inspected:

| Name / arguments | Inspection | Actual native/fallback counts | SHA256 |
| --- | --- | --- | --- |
| `tree.png` / `--example tree` | Complete package.json, original left connector clip retained, visible133×138 containment diagnostic |0/10|`5e9141535f79f774c9a2e6b7cbee7d563fd6473036b34dec26358b979f00a84e`|
| `pie.png` / `--example pie` | Colored slices, percentages, title and legend intact |0/4|`56e06138444ea456c39e22f8705142398913f01a30dc7be036b32f3bebecefce`|
| `plantuml-sequence.png` / `--language plantuml --example plantuml-sequence` | Heads/lifelines, messages, note, activation and frame intact |8/0|`c52c846a44a0279ad75c596b7faed996f287eecd7d18be77c4ac50c282ccafd4`|

All exit0; stderr logs are beside the captures. Final `pgrep -a -x diagram-viewer` and process-pattern inspection returned no matching viewer or worker/fault process. No user IVE instance/profile/terminal was driven; no focus activation, OS screenshot or xdotool was used. Scoped commands needed escalation because sandbox bwrap failed to create loopback; no auto-review rejection occurred.

Residuals: conservative preserved/rejected effect cases may retain label clipping or reject large valid SVG; native-font results are host dependent; deterministic export's text/font/viewport limitations remain; glyph bitmap rectangles may overestimate ink; text overflow inside node backgrounds/layout collisions are not fixed. No macOS runtime/font/capture verification is claimed. This checkpoint is ready for independent review, not an approval of its own implementation.

## Fix round 1 — reject filter-image transform context

Fix base: `9cca89835b6edf773866b002061c5e693ac8c598`.
Fix head: `130ca1064669f312d8883f2686f939f1bc775bfc` (`Reject filter images before bounded SVG rasterization`). Tracked worktree clean after commit; no push or merge.

Read the complete independent finding in `/tmp/ive-viewer-containment-review.md` and checked pinned resvg 0.46.0 `src/filter/mod.rs::apply_image`: it reconstructs a scale-only linear transform, whereas the guard passed the parent's complete shear into the filter-image subtree. The reported opposite-shear pattern therefore passed the old guard despite resvg requesting a 250,100×100 tile. The original checkpoint's filter-image admission was unsafe; this fix supersedes that part of the bounded-policy description above.

Chosen conservative policy: all encountered `feImage` filter primitives now fail the existing unsupported-bounded-raster-work branch, with `feImage` named explicitly. This happens in the pre-raster guard, before rendering or pixmap allocation. Ordinary SVG `<image>` nodes, embedded SVG glyph images and the supported filter/effect paths are unchanged. No attempt is made to add or approximate resvg's special filter-image coordinate system. README now documents this rejection alongside the other unsupported filter kinds.

Changed files: `diagram-ports/viewer/src/svg/raster_guard.rs`, `diagram-ports/viewer/src/svg/tests.rs`, `diagram-ports/README.md`. Added test `should_reject_filter_image_with_cancelling_pattern_shear_before_rasterization` uses the review's complete SVG counterexample, parses the real usvg tree through `DisplaySvg`, and calls only the actual raster guard with identity transform and a 100×100 destination. It requires an explicit `feImage` rejection. The oversized pattern was never rendered or allocated, including during RED.

All commands below ran from the standalone `diagram-ports/` workspace; Cargo used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`:

| Command / stage | Observed output |
| --- | --- |
| `cargo test --offline --locked -p diagram-viewer --lib svg::tests` / RED before production change | exit 101; 18 passed, 1 failed; new regression panicked because guard returned `Ok(())`; `/tmp/ive-4v-fix1-red.log` |
| Same command / GREEN after rejection | exit 0; 19 passed, 0 failed; `/tmp/ive-4v-fix1-green.log` |
| `cargo test --offline --locked -p diagram-viewer` | exit 0; 50 Rust tests (36 unit, 5 export, 3 native lifetime, 6 worker) plus 7 hostile worker scenarios; `/tmp/ive-4v-fix1-viewer.log` |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | exit 0; `/tmp/ive-4v-fix1-clippy.log` |
| `cargo fmt --all -- --check` | exit 0 |
| `git diff --check`, staged `git diff --cached --check` | exit 0 |

The targeted SVG suite includes existing supported effects/clips/patterns, full-channel font/color comparisons, transformed text, exact built-in tree rendering, root clipping, cancellation and allocation bounds. Only the preexisting vendored `Atomic::fetch_update` warning remains. Full engine-workspace tests, release/native captures and source/licence audits were not repeated for this three-file rejection-only fix; their prior evidence remains attached to the original checkpoint, not misrepresented as fresh fix-head runs. No dependencies, assets, engine or export behavior changed. Independent scoped re-review remains required.
