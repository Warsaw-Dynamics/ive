# Task 2 review

Reviewed range: `a8394fe2c72c299d94b70f4aab30fcd4ba774d0e..618be6831a66a778052d2a7ee0190e8cf6460c0d` in `/mnt/data/Projects/ive/.worktree/diagram-ports`.

## Strengths

- Worker supervision separates protocol I/O, stderr draining, and the deadline/cancellation monitor. The owning child guard kills and waits on every return path; bounded frames are admitted before allocation. The real hostile-child tests cover meaningful failures rather than only mocking return values.
- The native viewer uses full-color SVG parsing/rasterization, background work, replaceable pending requests, generation checks, and explicit failure presentation. The engine stays independent of GPUI.
- Native measurement support and bbox/wrapping fallback are accurately distinguished. Platform enforcement limitations and lack of upstream geometry parity are documented explicitly.
- CLI export stays display-independent, preserves OS-string paths, and does not replace an existing output when rendering fails. The dependency revision/platform features match the requested host framework.

## Findings

### Important — R1: Native text layouts accumulate for the lifetime of the viewer

Locations: `diagram-ports/viewer/src/service.rs:124`, `diagram-ports/viewer/src/native_text.rs:19`, `diagram-ports/viewer/src/native_text.rs:128`.

`work` creates one `NativeText` before its request loop and passes that same object to every worker. Its `WindowTextSystem` is a frame cache, not a bounded application-wide cache. In the exact pinned GPUI revision, `text_system/line_layout.rs:716` inserts each new text/font/size layout into `current_frame.lines`, and `:717` retains its key in `used_lines`. These collections are cleared by `finish_frame` (`:586`), which is crate-private through `WindowTextSystem` and is never called for this independently owned system. Dropping the previous parsed SVG or completing/killing a worker therefore does not release the native layouts.

Reproduction: launch the viewer on a regular Mermaid file; repeatedly replace its labels with new text and request a render (for example, toggle Source after each external edit). Every distinct successfully measured label remains in the same native layout cache, even after the old source/result has been replaced. Repeatedly visiting built-ins also retains the distinct labels of all visited examples. The worker's address-space limit does not cover this parent-owned allocation. The unbounded lifetime is verified from the actual pinned cache implementation, not an RSS experiment.

Fix direction: scope `NativeText`/`WindowTextSystem` to one engine-render job (retaining the underlying shared `TextSystem`), or introduce an explicitly bounded/resettable measurement cache. Ensure a cancelled job's callback owns its old instance safely. A per-render instance also makes font-resolution diagnostics describe the current result rather than previous diagrams.

Suggested focused regression: exercise two or more renders with disjoint measured labels and prove that the previous per-job native measurer/cache can be dropped after completion, including cancellation, while shared font infrastructure remains reusable. An allocation/drop counter or `Weak` ownership assertion is more reliable than an RSS threshold.

### Important — R2: Raster limits silently disable zoom on large diagrams

Locations: `diagram-ports/viewer/src/viewport.rs:31`, `diagram-ports/viewer/src/service.rs:234`, `diagram-ports/viewer/src/app.rs:451`.

The raster cap correctly bounds bitmap allocation, but `Ready.logical_size` is derived directly from those capped bitmap dimensions. Thus the cap also reduces the requested display size. The toolbar continues to report the requested percentage. Once the natural diagram reaches the cap, increasing zoom no longer enlarges it; for sufficiently tall/wide diagrams even “100%” displays reduced-size text that the user cannot enlarge to inspect.

Executed reproduction against the actual `viewport.rs`, with device scale 1 and a 1,000 × 10,000 diagram:

```text
100% -> bitmap (409, 4096), displayed logical size (409, 4096)
200% -> bitmap (409, 4096), displayed logical size (409, 4096)
400% -> bitmap (409, 4096), displayed logical size (409, 4096)
```

Reproducer: `/tmp/ive-viewer-review-zoom.rs`; executable: `/tmp/ive-viewer-review-zoom`. The logical-size statement follows `service.rs:234` exactly at scale 1. No GUI interaction is needed to demonstrate the defect.

Fix direction: calculate requested logical display dimensions separately from bounded raster dimensions. Keep the bitmap limits, but display the bounded bitmap at the requested logical size (or use tiled rendering). If the product instead intentionally limits zoom, the effective zoom must be explicit and controls must not advertise a size they do not produce; that still needs a way to inspect large diagrams adequately.

Suggested focused regression: a tall/wide input should have increasing logical dimensions at 100%, 200%, and 400%, while each corresponding raster remains within the axis and area limits; also cover scale factor 2 and Fit.

## Declined to judge

- Full browser/upstream geometry parity and unsupported native bbox/wrap operations: explicitly documented compatibility limitations, not claimed Task 2 deliverables.
- PlantUML support and language-specific example filtering: explicitly deferred to Task 3.
- Verified macOS memory enforcement and non-Unix OS memory ceilings: explicitly documented platform limitations; Linux is the verified runtime target.
- IVE integration and IVE MCP verification: root product is unchanged; this task is the standalone viewer with native captures.
- Full legal/source-provenance audit of the regenerated approximately 12,000-line notices: outside this focused implementation review; controller supplied successful license/source checks and preserved-notice evidence.
- Mandatory atomicity for an output write that fails after rendering: the task promises preservation on render failure, which the implementation provides; no additional atomic-export requirement was assumed.

## Verification and scope

Read the task brief/report, design, implementation plan, applicable AGENTS/README and requesting-code-review template; inspected all new viewer implementation modules, tests, and the exact pinned GPUI text/raster implementations relevant to the findings. No reviewed checkout file, index, HEAD, or branch state was modified. The existing unrelated PlantUML analysis edit remains untouched. No app instances or subagents were started. Only the focused zoom reproduction was compiled/run; the full suite was not repeated because the controller freshly ran 79 ordinary tests and 7 fault scenarios and inspected native captures.

## SPEC COMPLIANCE verdict

**With fixes.** Task 2's architecture, UI, CLI, isolation, native measurement and verification deliverables are substantially present. R2 prevents the required zoom controls from working meaningfully for large diagrams; R1 defeats bounded parent-side resource lifetime across renders.

## CODE QUALITY verdict

**With fixes; not ready to accept unchanged.** The supervisor and separation of concerns are strong. Correct the two Important findings before proceeding; no Critical finding was identified. Both fixes can remain scoped to native-measurer lifetime and logical/raster sizing, without expanding the task.
