# Spec compliance: ❌ Issues found

Task 4V, base `d07df148ac7d82eb2ff33ba5a1ce35fbe9c03d4b`, head `9cca89835b6edf773866b002061c5e693ac8c598`. The host containment behavior and explicit preservation policy meet the reviewed requirements, but the bounded raster requirement fails for an admitted `feImage` plus transformed pattern case. Quality verdict: **Needs fixes**.

## Strengths

- `diagram-ports/viewer/src/svg.rs:42`: the host owns the parsed usvg tree used for both inspection and resvg painting. Expansion changes the host canvas and root raster mapping; exported SVG and engine layout are untouched.
- `diagram-ports/viewer/src/svg.rs:241`: iterative inspection distinguishes text from ordinary shapes, composes local transforms, examines flattened text descendants, and charges points and bitmap occurrences. The explicit effect-preservation and directional non-text-overflow policies protect intentional clipping without blanket padding.
- `diagram-ports/viewer/src/svg.rs:87`: final raster dimensions are separately capped and validated before allocation, while service integration continues to obtain logical sizing through the existing viewport layout code. Parsing and painting remain on the service thread.
- `diagram-ports/viewer/src/svg_fonts.rs:29`: the copied empty-asset font policy matches the pinned pgui source's default/sans/first-face selection, missing generic-family repair, emoji order and used-face/glyph checks. Attribution and exact revision are recorded; RGBA conversion calls the existing public pgui function.
- `diagram-ports/viewer/src/svg/tests.rs:4`: tests exercise the actual built-in tree, full-channel pinned-renderer equivalence, independent oversized-canvas paint containment, transformed/stroked/fallback text, root-clipping preservation, cancellation, logical/raster separation and resource rejection. The tests materially check behavior rather than merely mirroring the bounds result.

## Important — raster guard admits an oversized pattern beneath feImage

`diagram-ports/viewer/src/svg/raster_guard.rs:241` passes the parent `Context` directly into an `feImage` subtree. Pinned resvg does something different: `resvg-0.46.0/src/filter/mod.rs:862` replaces the incoming transform's linear part with `Transform::from_row(sx, 0, 0, sy, …)`, using `ts.get_scale()`. Thus a pattern transform that cancels a parent shear in the guard does not cancel the transform actually used to paint. The pattern admission then checks the wrong tile dimensions.

Reproducer (100×100 destination, identity root raster transform):

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <defs>
    <pattern id="p" patternUnits="userSpaceOnUse" width="100" height="100"
             patternTransform="matrix(1 0 -50 1 0 0)">
      <rect width="1" height="1"/>
    </pattern>
    <rect id="art" width="1" height="1" fill="url(#p)"/>
    <filter id="f" filterUnits="userSpaceOnUse" x="0" y="0" width="1" height="1">
      <feImage href="#art" x="0" y="0" width="1" height="1"/>
    </filter>
  </defs>
  <g transform="matrix(1 0 50 1 0 0)">
    <rect width="1" height="1" filter="url(#f)"/>
  </g>
</svg>
```

The focused probe includes the production raster-guard module and parses the source with the installed usvg dependency. Its copied parent helpers have equivalent behavior for this source's opaque rectangles; it does not invoke the renderer or allocate the oversized tile. Output:

```text
guard=Ok(())
feImage guard transform Transform { sx: 1.0, kx: 50.0, ky: 0.0, sy: 1.0, tx: 0.0, ty: 0.0 }, renderer scale 50.01,1
pattern Transform { sx: 1.0, kx: -50.0, ky: 0.0, sy: 1.0, tx: 0.0, ty: 0.0 }, guard tile (100.0, 100.0), resvg tile (250100.0, 100.0)
```

Pinned `resvg-0.46.0/src/path.rs:189` rounds those dimensions and passes them directly to `Pixmap::new` at line 193, without a target-size clamp. The admitted tile is therefore 250,100×100 = 25,010,000 pixels, exceeding both the 4096-axis and 4,194,304-pixel offscreen limits. Scaling the cancellation further can amplify this gap. This contradicts the safety contract even though normal fixture captures and the existing oversized-pattern test pass.

Fix by explicitly rejecting `feImage` under the bounded policy, or by propagating resvg's exact `feImage` paint context (including its scale-only linear transform and effective surface semantics) before inspecting descendants. Add this counterexample as a regression that checks rejection before rasterization. Do not render the oversized pattern to validate the test.

Probe artifacts: `/tmp/ive-feimage-guard-probe.rs`, `/tmp/ive-feimage-guard-probe`. Compiled with the installed Rust 1.99 compiler and existing usvg rlib. No checkout edits, full-suite reruns, or oversized allocations were performed.

## Critical / minor issues

No Critical finding. Existing reported vendored `Atomic::fetch_update` deprecation noise is unchanged and is not a task regression; the controller should retain that qualification when describing validation as clean.

## Focused checks and evidence boundaries

- Read the supplied full review package once; no git rederivation or changed-file rereads. No subagents.
- Named risk: font-policy drift after replacing pgui's opaque parsed owner. Compared the exact pinned `crates/gpui/src/svg_renderer.rs` font resolver, repair, emoji and raster conversion paths with the new adapter. No discrepancy found for the empty asset source.
- Named risk: allocation bounds diverging from resvg's rendering semantics. Checked pinned `resvg` group-layer, pattern, image and filter-image allocation/transform code, plus tiny-skia integer rectangle conversion. This produced the single Important finding above. The ordinary group-layer and pattern-tile rounding formulas otherwise matched or conservatively bounded those inspected paths.
- Existing controller evidence, not reviewer runs: 166 Rust tests plus 7 hostile-worker scenarios; both source audits and 6 Node guards; formatting/clippy; all three actual native captures and no leftover processes. The implementer's report separately records release/license checks and RED/GREEN logs. No reported suite was rerun.
- ⚠️ Cannot independently establish unchanged worker hard deadlines/memory limits, stale-generation handling or native per-job cache lifetimes from this task diff. These are preserved by the limited integration changes and covered by existing controller evidence; they were not broadened into a whole-branch review.
- ⚠️ No macOS runtime/font/capture verification is available. The code and report explicitly avoid claiming it. Host-library calls remain in-process with cooperative cancellation boundaries, as documented; the review does not claim a hard host timeout or a formal process RSS ceiling.
- ⚠️ This gate is task-scoped. It does not re-establish engine geometry parity, DOM ink metrics, universal SVG normalization or deterministic export font portability; the change accurately disclaims those capabilities.

## Assessment

**Task quality: Needs fixes.** The same-tree containment, color/font compatibility and conservative clipping policy are well supported. The admitted `feImage` transform mismatch bypasses a required pre-allocation bound, so both gates remain blocked until it is fixed and the focused regression passes.
