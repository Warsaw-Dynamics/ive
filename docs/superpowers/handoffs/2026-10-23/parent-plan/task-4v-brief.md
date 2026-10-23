# Task 4V: Native viewer text-paint containment

Hold implementation until the controller explicitly releases this task after Task3A's independent review. This is a separate reviewed viewer correction, not a new engine metrics claim.

Read the existing design/parent plan, Task2 report, this brief, and `/tmp/ive-diagram-native-bounds-prep.md` completely. The owner requests standalone pgui diagram rendering, and the actual built-in Mermaid tree example visibly clips package.json. Product changes remain in this worktree's `diagram-ports` project only; use apply_patch, test-first regressions, and preserve all PlantUML and worker behavior. No subagents or concurrent implementers.

## Required behavior

- Compute a host-only display canvas from the same parsed SVG/font policy used to paint, unioning the original viewport with actual text paint bounds. Keep node positions, transforms, original deterministic exported SVG, intentional non-text clipping and layout metrics unchanged. No fixture-specific constants or blanket padding.
- A private usvg/resvg parsed-tree owner is acceptable where pinned pgui's ParsedSvg hides its tree. Match the pinned pgui font database/emoji/asset policy exactly and rasterize the same tree. Record any copied Apache-licensed glue with exact pgui revision/path and retain required attribution. Additional exact direct dependencies may reuse the audited closure.
- Traverse transformed text's actual flattened paths/images with composed transforms; don't trust incomplete cached bounds for color/SVG glyphs. Handle fill, stroke, opacity, invisibility and defined effect/clip policy explicitly. Conservatism is acceptable when justified, but don't expand arbitrary non-text connectors. Reject or preserve unsupported normalization cases explicitly rather than pretending all SVG effects have been solved.
- This fixes viewer canvas containment, not text fitting within node boxes or original geometry parity. Native glyph typographic bounds on Linux are advances; don't claim DOM RawBBox or ink measurement from that API. Existing unsupported bbox/wrap operations remain truthful recorded fallbacks.
- Keep logical dimensions independent from capped raster dimensions. Preserve 4096-axis/4,194,304-pixel bitmap limits, 1,000,000 logical-axis limit, cancellation/stale-job behavior, per-job native caches, worker hard deadlines/memory bounds, and off-UI-thread parsing/rasterization. Bound tree traversal/items/depth and finite arithmetic before allocations.

## Evidence

Start with a failing regression for the actual built-in tree (not just an explicit Liberation Sans diagnostic). Assert all admitted text paint lies in the display canvas, while the intentionally clipped connector remains clipped. Add transformed/rotated/skewed text, mixed strokes, invisible text, missing/generic/emoji fallback fonts, large/raster-capped diagrams, invalid/extreme transforms and low/cancelled bounds controls.

Compare full RGBA pixels against pinned pgui painting for representative opaque/translucent full-color cases. Test premultiplied-alpha/BGRA conversion, not merely alpha occupancy. Verify the same-font policy and preserve colored diagrams. Reuse prep probes as diagnostics, not shipped fixture answers.

Also test transformed text forcing leftward canvas expansion while an intentionally root-clipped non-text path lies left of the original viewport. That path must not become newly visible, or the case must use an explicit justified preserve/reject-normalization policy. The built-in tree retaining its left connector clip is necessary, but is not by itself proof for every expansion direction.

Run focused tests, full workspace test/clippy/fmt/release, source/license audits as relevant, and actual background native captures for the built-in tree plus a colored Mermaid and PlantUML sequence or nested UML. Inspect captures, verify no lingering test processes. Record deterministic SVG export's remaining font/viewport limitations honestly.

Commit scoped changes and write `task-4v-report.md` here with exact commit, RED/GREEN, test counts, transform/effect/font policies, provenance, capture paths and residuals. Independent review must pass before the next implementation task starts.
