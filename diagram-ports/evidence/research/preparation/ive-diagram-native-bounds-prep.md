# Native text paint bounds — read-only preparation

Date: 2026-10-23. Scope: investigate the clipped `package.json` label in `/tmp/ive-diagram-4a-tree.png`; do not modify product code, metrics semantics, fixtures, or dependencies. No subagents. Systematic-debugging was used to distinguish advance, DOM bbox, native raster ink, and SVG outline bounds before proposing a correction.

## Result / recommended scope

The pinned Linux `PlatformTextSystem::typographic_bounds` is public but **does not return glyph outlines**: it constructs a rectangle from glyph advances. Do not implement `RawBBox*`/`SvgBBox*` by relabelling that rectangle, line height, or raster ink as a browser DOM bbox.

A real, generic visible-label containment correction is feasible without fixture dimensions or node-coordinate changes: inspect the actual shaped text paths/images in the same usvg tree that will be painted, union their transformed paint bounds with the original viewport, and enlarge only the host raster canvas/logical display extent. Keep the existing native metric profile and deterministic bbox fallbacks/provenance unchanged. On this tree, a derived canvas width of 129 SVG viewport pixels (original 122.08) contains the complete label and preserves the intentionally clipped left connector. There is no browser geometry/pixel parity claim.

Preferred eventual implementation is a small host-side SVG finalizer owning its usvg options/tree and rendering that **same** tree through resvg to `Arc<RenderImage>`. It replaces the existing SVG renderer path for diagrams rather than independently parsing with a second SVG font database and then hoping the GPUI private tree agrees. GPUI's empty-asset font selection policy can be mirrored exactly at the pinned revision; the temporary equivalence probe below verifies the relevant installed-font cases. No framework patch is required. Implementation remains held until a separate controller release.

## Exact pinned APIs and their semantics

Pinned source root: `/home/kb/.asdf/installs/rust/1.99.0/git/checkouts/pgui-ec323d2a127f3321/f5d5fa2`, revision `f5d5fa2f8f07b0be68442e694d22ed6064609e73`.

- `crates/gpui/src/platform.rs:1509`: `PlatformTextSystem: Send + Sync`; public `font_id`, `font_metrics`, `typographic_bounds(FontId, GlyphId)` (1522), `glyph_raster_bounds(&RenderGlyphParams)` (1528), and `rasterize_glyph` (1530).
- `crates/gpui/src/text_system/line_layout.rs`: public `LineLayout.runs`, each `ShapedRun.font_id`, and positioned `ShapedGlyph.id/position/index/is_emoji`. These preserve shaped ligatures, combining marks, and actual fallback font IDs; measuring characters separately would not.
- `gpui_platform::current_platform(true).text_system()` supplies a real headless native platform text system. Constructing `TextSystem::new(native.clone())` proves that the native API and `WindowTextSystem` use the same font ID namespace. A UI implementation could capture this Arc from the platform before passing that platform to `Application::with_platform`; it need not construct another native font database.
- **Linux** `crates/gpui_linux/src/linux/text_system.rs` reexports `gpui_wgpu::CosmicTextSystem`. Its `crates/gpui_wgpu/src/cosmic_text_system.rs:243` implementation returns origin `(0,0)` and size `(advance_width, advance_height)`. It cannot justify an ink-bounds capability claim.
- **macOS** `crates/gpui_apple/src/text_system.rs:186` delegates to font-kit `typographic_bounds`; that is a different implementation. Its coordinate/ink behavior was inspected but not executed here. Neither implementation establishes browser `<text>.getBBox()` equivalence.
- Linux `cosmic_text_system.rs:475` really renders a hinted Swash glyph for `glyph_raster_bounds`; returned placement is `(left, -top)` in y-down device pixels. Union with each shaped glyph's position gives native **raster** ink at the selected font size/scale/subpixel variant, not scale-independent DOM bounds. Fallback/emoji use each run's own font ID.
- **Cache hazard**: Linux `raster_bounds` inserts nonempty glyph images into shared `pending_glyph_images` at line482. `rasterize_glyph` removes the entry at497. A bounds-only bridge that never consumes the image can retain entries across jobs. The probe immediately consumes every nonempty result. A future native raster-ink capability would need this disposal path, finite per-query glyph/pixel bounds, and cancellation; it must not weaken the existing per-render `WindowTextSystem` lifetime.
- `crates/gpui/src/svg_renderer.rs:103`: `ParsedSvg(usvg::Tree)` has a private field and no public tree/options accessor. `parse_svg` (190) and `render_parsed` (195) are public, but cannot expose actual outlines to this application. Rendering keeps colors and text; `svg()` icon masks remain inappropriate.
- usvg/resvg **0.46.0** are the versions used by this pinned GPUI. `Text::flattened()` exposes the real text paths/images. `Path::stroke_bounding_box()` and transformed image bounds provide conservative paint geometry. `resvg::render(tree, transform, pixmap)` publicly accepts a host root transform and does not require changing `tree.size()` or any SVG node.

## Why raw DOM bbox must stay distinct

Vendored `merman-render-0.9.2/src/text/measure.rs:156–195` explicitly distinguishes direct `<text>.getBBox()` width/height/baseline offsets from helper/tspan/title probes. The tree renderer requests RawBBoxWidth/RawBBoxHeight. A glyph-ink union is useful, but is not those browser operations.

At 16px Liberation Sans on this host (baseline y=0):

| Text | Chromium direct SVG bbox | Native shaped advance | Native raster ink [left,top,right,bottom] |
| --- | --- | --- | --- |
| `package.json` | x0,y−14,width94.296875,height17 | 94.2890625 | [1,−12,93.390625,3] |
| italic `jfy` | x−2,y−14,width19,height17 | 16 | [−2,−12,17,3] |
| three spaces | x0,y−14,width13.34375,height17 | 13.3359375 | empty |

The public Linux “typographic” union for `package.json` is `[0,−17.875,94.2890625,0]`; actual usvg outline ink is `[1.03125,−11.59375,93.25,3.3203125]`. These are observably different quantities, not rounding variants of one exact operation. Actual browser bbox height17 is not the native raster ink height15. Whitespace has DOM extent and no paint. This rules out an honest generic RawBBox implementation from the suggested Linux API, without invoking browser runtime or explicitly defining a different compatibility profile.

## Same-font proof against pinned GPUI

Temporary probe `/tmp/ive-native-bounds-probe/src/main.rs` builds a usvg font database using the same installed system fonts and mirrors the pinned viewer's **empty asset source** policy:

1. System fontdb; no bundled fonts because `SvgRenderer::new(Arc::new(()))` supplies none.
2. Preserve existing generic-family mappings; only repair missing mappings to IBM Plex Sans/Lilex as GPUI does.
3. Default usvg font selection, then SansSerif, then the first available face.
4. Same Emoji_Presentation test; Linux emoji priority Noto Color Emoji, Emoji One, Twitter Color Emoji, JoyPixels; reject previously used faces and require the requested glyph; otherwise default fallback selection.

Comparing transparent 400×80 output from actual pinned `SvgRenderer::parse_svg/render_parsed` with direct usvg/resvg gives **0 differing alpha pixels in every one of 13 cases**:

- Liberation Sans: `package.json` normal; `jfy` italic; `Ág`; `office ffi`; `x` + combining acute; `A 世界 🐈`; three spaces; `src` bold.
- DejaVu Sans: `office ffi` (six shaped glyphs for ten Unicode characters proves actual ligatures, not character advances).
- sans-serif, serif, monospace, and Missing Font Family: `Fallback family`.

The multilingual native line uses distinct fallback font IDs (0,13,14). This is concrete installed-font equivalence evidence for these cases, not an assertion about every font or future GPUI revision. A naive default usvg resolver originally differed from pinned GPUI by156 alpha pixels on emoji; matching the emoji fallback policy removed that discrepancy. Do not drop the policy when implementing the adapter.

## Bounds and transform proof

Traversal must mirror resvg, not rely on cached absolute bounds inside flattened text:

- Begin at tree.root children with identity; resvg does not separately apply tree.root().transform().
- For an ordinary group, recurse with `parent.pre_concat(group.transform())`.
- For a Text node, recurse through `text.flattened()` with that flattened group's transform composed too.
- Transform each local path stroke bbox / image bbox by the composed affine transform, then union. A transformed rectangle is conservative under rotation/skew. Skip hidden/zero-opacity groups; production must also skip paths with neither nonzero-opacity fill nor stroke and explicitly handle clip/mask/filter policy.
- Do not blindly use flattened paths' `abs_stroke_bounding_box()`: usvg text/flatten.rs:81/95 has unresolved absolute-transform TODOs for color/SVG glyph children. This initially omitted ancestor transforms in a probe. Following the renderer's local transform stack fixes it, including bitmap emoji.

Original root viewport for the saved tree is `W=122.08,H=138`, root viewBox `V=(−0.5,0,122.08,138)`, default xMidYMid meet. Therefore its viewBox-to-canvas mapping is exactly `M(x,y)=(x+0.5,y)`; usvg inserts this transform as a child group (parser/converter.rs:404–425). Text paint union in these original **canvas coordinates** is `[5.65625,6.4328117,128.75,131.74687]`. Right128.75 exceeds canvas width122.08, reproducing clipping independently of the screenshot.

Union the original viewport `[0,0,W,H]` with the text paint bbox rounded outward in original viewport pixels: `C=[0,0,129,138]`. Rounding is geometry-derived quantization, not a font/fixture padding constant. The canvas grew only rightward, so the existing connector at canvas x−9.5 remains clipped. By contrast, expanding for all diagram paths produces `[−9.5,...]` and reveals the artificial root connector; do not use whole-graph bounds for this correction.

At raster size `(rw,rh)`, paint the **unchanged original tree** with:

`R = Scale(rw/(C.right−C.left), rh/(C.bottom−C.top)) × Translate(−C.left,−C.top)`.

The original viewBox matrix M and all original SVG ancestor transforms remain in the tree; do not apply them twice. Logical 100% dimensions become `(C.width,C.height)`, not the capped raster dimensions. This avoids SVG XML rewriting, percent-size reparsing, and changing node coordinates.

For the unit-scale tree only, the equivalent root-only viewBox is `M⁻¹(C)=(−0.5,0,129,138)`. Rendering that diagnostic reparse and rendering the original tree into the expanded canvas at4× produce **0 differing RGBA pixels, with identical dimensions**. Both label-extent containment assertions pass. The before/after images were visually inspected: the final `n` is complete and the connector clipping remains as before.

For general root viewport `W,H` and viewBox `(vx,vy,vw,vh)`, calculate `sx=W/vw, sy=H/vh`. With preserveAspectRatio=none use both; with meet/slice choose min/max respectively for uniform scale s, then alignment offsets `(ax,ay)` from remaining width/height. `M=(sx,0,0,sy,−vx*sx+ax,−vy*sy+ay)`. This is usvg tree/geom.rs:58–84. **The host-canvas approach needs no inverse or root rewrite**: traversal already yields paint bounds after M, and R only expands the resulting viewport. Root transforms, nested svg viewports, and explicit clip paths must continue to follow the parsed renderer tree.

Additional temporary assertions rasterized at4× and checked every nonzero-alpha pixel against the conservative transformed text bounds:

| Case | Contained painted pixels |
| --- | ---: |
| translated/scaled italic `jfy` | 1706 |
| rotated center-anchored DejaVu ligatures | 4213 |
| nested skew/scaled combining/stroked text/tspan | 3706 |
| scaled CJK + emoji fallback | 11058 |

All assertions passed. These cases cover positive/negative overhang, ancestor transforms, ligatures, combining marks, text stroke, tspan offsets and bitmap/color fallback. Clip/mask/filter interactions and transparent-paint pruning are not claimed proven by this matrix.

## Proposed scoped implementation / safeguards

1. Keep `NativeTextResources` shared `Arc<TextSystem>` and a fresh `NativeText/WindowTextSystem` per engine render. No metric RPC opcode or advertised capability changes are needed for canvas containment. Preserve fallback diagnostics for bbox/wrapping/unsupported operations. Native raster-ink API work is optional future work, not a prerequisite for this fix.
2. Add one internal parsed-SVG/finalizer adapter on the existing background service thread: own pinned-version usvg options/fontdb and parsed tree; inspect actual Text flattened descendants; store original viewport plus derived expanded canvas; rasterize that same tree with R. Convert premultiplied RGBA to the GPUI RenderImage format exactly as the pinned renderer does; preserve full colors and img(Arc<RenderImage>) display. Attribute any reused pinned Apache-2.0 font-policy/conversion glue and update standalone notices if direct deps are added.
3. Keep separate logical and physical sizes. Validate every bbox/affine/canvas coordinate and arithmetic result as finite, reject nonpositive/overflow/unsupported extreme dimensions before allocation, and apply the existing max1,000,000 logical pixels per axis. Existing zoom25–400%, device factor positive/finite≤8 and Fit remain honest after expansion. Raster independently remains≤4096 per axis and≤4,194,304 pixels; never allocate an uncapped “inspection bitmap”. Bounds inspection is path geometry, not supersampling the entire uncapped canvas.
4. Use an iterative bounded node/path traversal with cancellation checks; define and test an explicit visit/depth budget rather than recursing unboundedly. Existing source256KiB, SVG16MiB, stderr64KiB and worker10s/memory limits remain unchanged. Check for stale/canceled generation before and after parsing/bounds/rasterization. The same-binary engine watchdog stays independent of font callbacks. Host parsing/rasterization already occurs in-process off UI: do not claim a new killable hard deadline for that in-process work.
5. Only enlarge from **text paint geometry**, never connector/shape bbox, fixture names, node positions or fixed padding. Ignore empty/hidden/fully transparent paint. Bitmap glyph rectangle bounds may conservatively include transparent border pixels; call the result conservative painted-label containment, not exact outline/DOM metrics. Explicit clip/mask/filter effects require a deliberate conservative policy: preserve explicit clipping, do not silently “unclip” text, and record unsupported effects/skip normalization where containment cannot be justified. Filter paint can extend outside ordinary path bounds. Avoid promising all arbitrary SVG effects are handled by this minimal traversal.
6. Emit a host diagnostic when the canvas is expanded, with original/derived dimensions and a `native SVG paint containment` description. Keep engine measurement metadata unchanged: this is a host display finalization, not native RawBBox success, browser layout parity, or corrected node-box layout. Canvas expansion prevents viewport clipping; it does not fix labels overflowing their own node backgrounds or all possible layout collisions.
7. Test first: saved tree clipping before/containment after with bounds computed from real installed font outlines; original connector clip unchanged; no mutation of text/coordinates; transformed/italic/ligature/combining/emoji cases; no-op contained/empty/invisible text; percent roots and nontrivial preserveAspectRatio; bbox extremes/nonfinite/budget rejection; raster/deviceFactor2/Fit/zoom guards; existing Weak current/canceled-job regressions; real background native capture and cleanup after release.

## Export distinction

Keep headless engine rendering and the existing deterministic SVG export distinct. This investigation does **not** make deterministic exported SVG use native fonts/metrics or eliminate its possible host-font-dependent viewport clipping. A viewer-only canvas correction changes the displayed raster extent, not the exported source SVG. Document this explicitly; do not silently change --render provenance or claim export geometry parity. A later separately named host-finalized export could use the same host adapter and record environment-dependent font selection, but that would be a separate authorized scope. No browser/JVM/runtime tooling belongs in production.

## Reproduction / evidence / cleanup

- Temporary crate and exact probe source: `/tmp/ive-native-bounds-probe/{Cargo.toml,src/main.rs}`. It now consumes the saved SVG snapshot, so concurrent PlantUML engine implementation cannot affect reruns. Dependencies are pinned GPUI plus already cached usvg/resvg0.46; run offline:

  `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo run --offline --manifest-path /tmp/ive-native-bounds-probe/Cargo.toml > /tmp/ive-native-bounds-probe/probe-output.txt`

  Final result exit0; all equivalence/containment assertions pass. Last incremental build1.27s. Removing the temporary engine dependency caused a25.19s feature-cache rebuild; no packages installed or network accessed. One attempted rerun while the concurrent engine module was incomplete failed with missing `plantuml` module; the probe was then decoupled to the saved SVG, without touching engine files.
- Original SVG snapshot: `/tmp/ive-native-bounds-probe/output/tree-before.svg`, SHA256 `73a5c6fe03368da0f65b1fc1361df976c1dd5a7ad01959ced1d526ce057c27c0`. It was generated from the builtin tree source with coherent Liberation Sans config after stageA; the values are observations, not constants in the bounds algorithm.
- Detailed metrics: `/tmp/ive-native-bounds-probe/output/text-metrics.json`; final assertions `/tmp/ive-native-bounds-probe/probe-output.txt`.
- Offline DOM-only comparison: `node /tmp/ive-diagram-oracles.JwN5H5/native-bounds.mjs`, using existing puppeteer-core / cached Chromium, headless+offline, browser closed in finally; result `/tmp/ive-native-bounds-probe/output/browser-text-metrics.json`. This is a temporary semantic oracle, not a product dependency. It measures fonts only, not diagram rendering.
- Visually inspected PNGs: `/tmp/ive-native-bounds-probe/output/tree-before.png`, `tree-after.png` (rejected whole-paint expansion), and `tree-text-only-after.png` (preferred, complete label). Corresponding SVGs are alongside them. These are temporary CPU raster probes, not a new native app capture.
- Process check shows no remaining probe or cached-Chromium process; only the inspection command itself matched. No native window/focus was created by this prep. No product files edited, no tests/commits claimed for this read-only task. Git status changes belong to the concurrent PlantUML implementation and were left untouched.

Ready for a separately released, test-first native display containment fix after Plant3A gate. Main remaining risks are intentional SVG effects, drift from pinned font policy, and confusing display-only containment with engine/export/native DOM metric support.
