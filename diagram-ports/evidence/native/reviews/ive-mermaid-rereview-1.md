# Task 1 fix round 1 — scoped re-review

Reviewed `29353b5..a8394fe` in `/mnt/data/Projects/ive/.worktree/diagram-ports` against the original finding in `/tmp/ive-mermaid-review.md` and the appended RED/GREEN evidence in `task-1-report.md`. Scope was the original P2 finding and new Critical/Important breakage introduced by this fix. No product changes, commits, subagents, or suite reruns.

**Original finding: ADDRESSED.**

**Spec compliance verdict: PASS for Task 1 and this fix scope.**

**Code quality verdict: PASS for this fix scope.**

No new Critical or Important findings.

## Assessment

- The source-wide substring guard is removed. Validation now follows controlled semantic preparation (`diagram-ports/engine/src/mermaid.rs:24`–30). Resource admission inspects designated image/link/style fields rather than labels/comments or arbitrary text (`engine/src/resources.rs:9`, `:40`, `:64`). The compatibility JSON projection uses the existing operation control in pinned Merman's implementation.
- CSS tokenization distinguishes URL/import/image-set constructs from quoted text, including nested blocks, and enforces a nesting ceiling (`engine/src/resources.rs:138`–176). External semantic styles are checked before a renderer can silently discard them.
- Generated SVG resource attributes and stylesheet contents are checked in a separate pass (`engine/src/resources.rs:93`, `:109`). This remains before normalization and the unchanged resvg-safe sanitizer/finalizer (`engine/src/mermaid.rs:39`). The existing final XML/resource guard remains in `finish` (`:101`). No new resolver, external runtime, or I/O service is installed.
- `cssparser = 0.38.0` was already in the locked selected dependency closure and notices; the diff only adds its direct dependency edge. The backend feature/version selection and public API remain unchanged.
- New safe-text regressions assert preserved text beneath SVG `<text>` elements. Existing family assertions receive the same improvement (`engine/tests/mermaid.rs:13`, `engine/tests/families.rs:30`), addressing the original nonblocking assertion weakness as well.

## Verification evidence

Accepted reported verification: full locked offline suite **58 passed** (37 family fixtures plus 21 behavior tests), with clean clippy/format/license checks. The appended report records four safe-source regressions failing before the fix and passing afterward, plus the external-CSS regression that prompted semantic style validation. I did not rerun the suite.

Focused probes linked the already-built fixed engine artifact without changing product code:

| Case | Observed result |
|---|---|
| `metadata: loaded` in a node label | Render succeeds |
| URL in Mermaid comment | Render succeeds |
| Literal URL in a node label | Render succeeds |
| External image metadata | `ExternalResource` |
| Relative image metadata | `ExternalResource` |
| External navigation target | `ExternalResource` |
| External CSS paint URL | `ExternalResource` |
| Quoted relative CSS paint URL | `ExternalResource` |
| Local fragment CSS paint URL | Render succeeds |
| Quoted CSS string containing `url(example)` | Render succeeds |

Probe sources: `/tmp/ive-mermaid-review-probe.rs` and `/tmp/ive-mermaid-resource-rereview-probe.rs`.

The prior cross-task limitations remain: Task 2 must establish off-UI execution and hard worker timeout/memory behavior; actual font/painting and geometry parity are not established by this fix. Those items do not block this Task 1 review gate.
