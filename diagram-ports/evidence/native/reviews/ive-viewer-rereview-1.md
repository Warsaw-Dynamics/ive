# Task 2 scoped re-review — fix round 1

Range: `618be6831a66a778052d2a7ee0190e8cf6460c0d..c3486fd0752681f18a06061468eb9794d7cfda58`.
Scope: original review findings R1/R2 and regressions introduced by their fixes. Read all six changed files and the appended task report. No broader review was reopened.

## Findings status

**R1 — native layout cache lifetime: resolved.**

`native_text.rs:15` now separates reusable `Arc<TextSystem>` and installed-family resources from `NativeText`'s `WindowTextSystem` and resolution diagnostics. `service.rs:183` creates the latter inside each uncached engine render. Its local owner disappears after result/report processing or an early error; the worker protocol thread owns its own Arc, so a cancelled in-flight callback retains exactly its own old cache until callback completion. The parsed-SVG cache stores no native measasurer. Fit/zoom cache hits do not allocate additional measurement sessions.

The new real-native tests meaningfully cover the original ownership defect: two distinct successful diagrams release their native sessions while retaining shared font resources; a paused, already-shaped callback survives cancellation safely, another render completes independently, and releasing the paused callback releases its old session. The assertions use Weak ownership rather than unstable RSS measurements. The production service uses the same resource/session factory exercised by those tests.

**R2 — raster limits disabling logical zoom: resolved.**

`viewport.rs:16` computes finite, bounded logical geometry independently of device-scaled bitmap dimensions. `service.rs:245` forwards that logical size unchanged to the image. `app.rs:444` gives the scroll surface the same explicit non-shrinking dimensions, keeping enlarged content inside scrollable positive coordinates. Raster axis/area caps remain in place. The added one-million-logical-pixel axis and maximum device-scale limits fail explicitly and are documented, rather than silently changing the requested zoom.

Independently repeated the original targeted reproduction using the actual corrected module:

```text
100% -> bitmap (409, 4096), displayed logical size (1000.0, 10000.0)
200% -> bitmap (409, 4096), displayed logical size (2000.0, 20000.0)
400% -> bitmap (409, 4096), displayed logical size (4000.0, 40000.0)
```

Reproducer: `/tmp/ive-viewer-review-zoom.rs`. The focused tests additionally cover scale-two logical invariance, scale-two Fit, invalid dimensions/device scales, and preserved bitmap caps. README accurately describes reduced sharpness when a capped raster is displayed at large logical zoom.

## New findings within scope

None. No Critical, Important, or Minor issue requiring another fix round was found in the R1/R2 changes.

## Verification limits

This reviewer compiled and ran only the original focused zoom reproducer. The controller supplied fresh passing results for all 27 ordinary viewer tests and seven hostile-worker scenarios, including both real-native ownership tests. The implementer report records full-workspace 85 ordinary tests plus seven fault scenarios, clippy, formatting, release build and the native tall-diagram Fit capture. I inspected the regression source and production wiring rather than repeating that full suite. The native capture demonstrates Fit, not interaction at every percentage; the actual geometry regression establishes percent zoom behavior.

No reviewed checkout/index/branch files were changed, and no app or subagent was started. The existing unrelated PlantUML analysis edit remains untouched.

## Declined to judge

- Unrelated Task 2 functionality and the original review's documented out-of-scope items: this round is explicitly limited to R1/R2 and their regressions.
- New end-to-end UI interaction testing at every zoom level: no unresolved code-path concern remained after geometry tests and inspection of the image/scroll-surface wiring; the supplied native capture only claims Fit.

## SPEC COMPLIANCE verdict

**Pass for the scoped fixes; Task 2 ready to accept.** Both original Important findings are corrected while preserving bounded raster allocation, worker isolation, and documented compatibility limitations.

## CODE QUALITY verdict

**Pass.** Per-render cache ownership is explicit and safe across cancellation, logical geometry is separated cleanly from raster allocation, and focused behavioral regressions cover the failure modes. No further fix round is indicated by this review.
