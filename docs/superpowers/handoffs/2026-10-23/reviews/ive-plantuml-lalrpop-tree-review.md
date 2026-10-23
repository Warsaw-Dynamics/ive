### Spec Compliance

- ❌ Issues found: the generated cutover adds a 16 KiB limit to marker prefixes and WBS aliases, narrowing the previously accepted subset (`diagram-ports/engine/src/plantuml/parser/outline.rs:23`; `diagram-ports/engine/src/plantuml/parser/tree.rs:219`, `:272`, `:309`). See Important I1.
- ✅ All four requested production routes are migrated: distinct Mindmap/Wbs entries plus Files and Board return the preserved models; the original production syntax routines are removed, and remaining Line adapters are test-only (`diagram-ports/engine/src/plantuml/parser/tree.rs:100`, `:124`; `parser/files.rs:24`; `parser/board.rs:21`; `tree.rs:285`; `files.rs:189`; `board.rs:108`).
- ⚠️ Cannot verify from this diff: actual 43-fixture SVG parity, native captures/paint, full 16 integration gates, release/empty-PATH behavior, or final conflict-union correctness. These are explicitly controller-owned under parallel-context.md, not missing worker scope. Run them after the fix and combined integration.

### Strengths

- Real generated productions own modifiers, aliases, required separators, file slash components/raw note blocks, board stage rows, and EOF. The generated entries disable recovery and move owned token payloads into typed actions; no production Line forwarding, fallback, or partial rendering appears (`diagram-ports/engine/src/plantuml/parser/grammar/tree.lalrpop:4`; `grammar/files.lalrpop:4`; `grammar/board.lalrpop:4`).
- Semantic construction preserves independent mindmap side stacks, WBS normalization/aliases/pseudo nodes, ordered folder reuse and duplicate files, sibling-note anchors, and nearest-lower board ancestry. Scene modules are untouched (`diagram-ports/engine/src/plantuml/tree.rs:75`, `:156`; `files.rs:87`; `board.rs:42`).
- Admission uses concrete family-specific stack bounds, actual token capacity, builder sizes, simultaneous adapter storage, diagnostic allowances, and checked model-vector growth. The extra borrowed BodySource view is explicitly charged and included in adapters (`diagram-ports/engine/src/plantuml/parser/outline.rs:95`, `:171`; `plantuml/mod.rs:80`).
- Generated family parsing occurs before the legacy error bridge; the new frozen-definition test pins exactly one expansion annotation (`diagram-ports/engine/src/plantuml/mod.rs:89`; `parser/tree.rs:487`). Shared ownership continues from the saved checkpoint and uses Row span maps (`parser/outline.rs:44`).
- No source grants/hashes, dependency locks, vendor trees, public API declarations, fixtures, or scene modules changed in the review package. Parser provenance and generated inventory are extended explicitly (`diagram-ports/provenance/plantuml-parser.json:3`; `engine/src/plantuml/generated/manifest.json:1`).

### Issues

#### Critical (Must Fix)

- None.

#### Important (Should Fix)

- **I1 — Apply the display-label bound only to labels.** `diagram-ports/engine/src/plantuml/parser/outline.rs:23` bounds every `Lexer::copy` input as a 16 KiB label. Tree lexing also calls it for the complete indentation/marker prefix (`parser/tree.rs:219`), parenthesized WBS aliases (`:272`), and quoted-label aliases (`:309`). The removed parser validated only the display label at `make_node`; prefixes and aliases remained bounded by source size, aggregate model text where retained, and work. A mindmap containing a single row of 16,385 `*` characters followed by ` Root` was accepted: the first node normalizes to depth zero and its label is only four bytes. It now fails with a label-bytes error before grammar entry. Likewise, a WBS root `*(<16,385 ASCII a characters>) Root` now fails despite its small display label and unchanged source/model/work limits. This violates the binding exact accepted/rejected-subset constraint. Separate syntax copying from label copying, retaining source/literal/model/work admission for non-label syntax and the existing 16 KiB bound for actual labels. Add focused contracts for long normalized root prefixes and both alias spellings, with an oversized display-label rejection control. No new unbounded allocation is required.

#### Minor (Nice to Have)

- Existing verification output contains two inherited vendor `Atomic::fetch_update` deprecation warnings (`/tmp/ive-task9-final-clippy.log:1`, `:14`; audited vendor origins `merman-core/src/operation.rs:449`, `merman-render/src/environment.rs:585`). They are unchanged and already assigned to final controller triage; they do not add a Task 9 fix requirement.

### Checks and Evidence

- Reviewed frozen package `review-task9-c23afd6..a60eda0.diff`, base `c23afd66158758c992a0278511caf1b21cec3e88`, head `a60eda0a5e14c0443e0da8efb3a5a971ac4866b9`. No git commands, checkout/index/HEAD mutations, Cargo commands, or suite reruns.
- Recomputed all three generated SHA256 values and all three grammar SHA256 values from the diff; every pair matches the manifest/report. Mechanically generated numeric/reduction plumbing was assessed through hashes, primitive actions, runtime interfaces, and the actual table witness rather than manual cell recounting.
- Read `/tmp/ive-task9-stack-proof.py` and `.json`; the controller independently compared its goto maps to generated code and ran it unchanged. Confirmed admission correspondence: live states 6/13/6/5; maximum four reductions; actual symbol types fit the conservative bounds; all four generated entries disable recovery. No witness rerun.
- Read `/tmp/ive-task9-red.log`, relevant failure details in parser evidence, final-tests, final-clippy, final-fmt, generated-check, source-audit, merman-audit, audit-tests, baseline-sha, and final-checks script. Evidence shows 175 unit + 138 public tests passing, engine-targeted strict Clippy passing apart from inherited dependency warnings, empty fmt output, eight reproducible grammars, 220 source hashes/43 fixtures/43 native modules, 656 vendor files/four patches, and 18 Node guard/audit tests. The unchanged 500 ms timing sensitivity remains recorded; it was not retried by this review.
- Named focused outside-diff checks: ownership/checkpoint and raw-data parity risk → `parser/ownership.rs:67`, `:109`; lexical allocation/cancellation risk → `parser/context.rs:122`; quota interpretation → `budget.rs:31`, `:46`, `:52`; possible line-1 error-origin risk → `parser/error.rs:31`, `:94` and `text.rs:234`. The label helper only checks size, so its unused line argument is not an additional origin defect. No broad code crawl.
- I1 follows directly from old/new control flow in the diff; existing tests do not exercise these long non-label atoms. Per controller direction, no redundant standalone probe was added.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The grammar/model cutover and actual resource admission are sound in the reviewed scope, with useful direct malformed-stream and cancellation controls. The new non-label 16 KiB restriction is a concrete compatibility regression and needs a same-cause fix before approval; combined baseline/native verification remains the controller's integration gate.
