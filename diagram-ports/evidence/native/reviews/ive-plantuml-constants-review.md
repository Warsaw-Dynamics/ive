SPEC verdict: ❌ Issues found. The eager constant model, origin representation, bounds, diagnostic clarification, provenance additions, and viewer integration meet the scoped brief, but the new preprocessing classifier mishandles existing comment/wrapper and raw-action ownership. Two Important corrections are required.

QUALITY verdict: Needs fixes. The core implementation is bounded and well tested; the duplicated source-state classification still produces observable wrong results on valid raw input.

## Strengths

- `diagram-ports/engine/src/plantuml/preprocess.rs:1`: definition-time expansion stores frozen values and insertion appends those values without rescanning. The insertion path checks lineage, individual literal size, origin quota and retained bytes before copying. Definitions and final parsing share the same Budget.
- `diagram-ports/engine/src/plantuml/origin.rs:1`: direct/expansion segments retain authored byte spans and compact definition IDs, coalesce contiguous direct ranges, and bound diagnostic candidates and omitted counts. The explicit label-local and ambiguous-candidate descriptions honor the brief's diagnostic-granularity clarification.
- `diagram-ports/engine/src/plantuml/mod.rs:43`: expanded input returns through wrapper/source/style/family admission and the existing terminal SVG/resource checks, using one native measurement session. No runtime resource/evaluator dependency was introduced.
- `diagram-ports/engine/tests/plantuml_constants.rs:20`: public cases cover eager freeze, authored quotes, boundaries, mutual-looking values, capability rejection, retained raw text, deterministic options, and unique/repeated/inert late-label candidates. Private preprocessor cases inspect state at allocation-bound failures rather than asserting only error text.
- `diagram-ports/viewer/src/svg/tests.rs:29` and `diagram-ports/viewer/tests/native_lifetime.rs:65`: both new examples enter existing actual native paint and per-render cache-release checks. Provenance and audit changes restrict the new TIM allowlist to the three selected files (`diagram-ports/tools/audit-plantuml.mjs:18`).

## Critical

None found.

## Important

1. `diagram-ports/engine/src/plantuml/preprocess.rs:34` (`Processor::scan`, raw `trimmed` wrapper branches): wrapper detection runs against the raw row regardless of comment or raw-body ownership, while the existing source parser recognizes wrappers after comment stripping and lets raw note/action bodies own their text. This both rejects valid definitions and skips legitimate substitutions.

   Confirmed with the frozen implementation's existing release viewer, using only headless export:

   - `/tmp/ive-constants-review-comment-wrapper.puml`: `@startuml`, a block comment containing `@enduml`, then `!define X Ready` and `A -> B : X` fails at line 5 with `definition must be inside a single wrapper`. The commented end marker incorrectly clears `inside`.
   - `/tmp/ive-constants-review-inline-wrapper.puml`: `@startuml /' wrapper comment '/` followed by the same definition fails at line 2 with the same error. The existing parser admits the comment-normalized opening wrapper, but preprocessing never sets `inside`.
   - `/tmp/ive-constants-review-raw-wrapper.puml`: a raw note containing `@enduml X` successfully exports, but its SVG contains `@enduml X`, not the required expanded `@enduml Ready`. The broad `starts_with("@end")` branch copies the raw label row unchanged.

   Classify wrapper transitions only in non-raw, non-comment syntax, using the same comment-normalized spelling as source admission. Raw note/action rows must still go through literal constant expansion without changing wrapper state. Add focused regressions for all three controls.

2. `diagram-ports/engine/src/plantuml/preprocess.rs:34` (`Processor::scan`, classification before action-opening ownership): an activity action's opening row enters `strip_comments` before preprocessing has established that it belongs to a raw action. The existing source parser deliberately recognizes a leading `:` before comment stripping (`diagram-ports/engine/src/plantuml/source.rs:159`). An apostrophe-block-looking substring in that opening payload therefore changes the preprocessor's comment state, prevents body detection, and suppresses expansion on following raw rows.

   `/tmp/ive-constants-review-action-comment.puml` defines `X Ready` and contains `start`, `:opening /' literal`, `' X`, `ending;`, `stop`. Headless export exits 0, but its SVG contains the raw label `' X`; the constant should have expanded to `' Ready`. This is the same kind of raw apostrophe row the new test explicitly supports, with a valid action-opening payload that the test omits.

   Recognize raw action ownership before running comment normalization, consistent with the existing source parser. Preserve that ownership until the existing action terminator rule closes it, and add this multiline action regression.

## Minor

No new Minor finding. The reported vendored `environment.rs:585` deprecation is the controller-identified inherited, deferred issue.

## Checks and scope

- Read the complete task brief, binding native constraints, implementation report, and prepared 16-file diff. The diff was read in sequential chunks; the notice/module region was recovered once after tool-output truncation. No git commands, product/index/HEAD edits, subagents, or broad repository crawl were used.
- Focused outside-diff risk check: source ownership mismatch. Read the cut-off `source::prepare`/wrapper context to compare wrapper, comment and raw-action ordering; it establishes the existing behavior implicated by both findings.
- Focused outside-diff risk check: new `check_preamble` indexing. Checked `style::prepare` through its return; it rejects `start == lines.len()`, so the new index is protected.
- Focused outside-diff risk check: storage accounting and shared work. Checked `Budget::text`/`charge`; retained copies consume the existing text/work budget and observe cancellation. No reset was found.
- Focused authorized-source risk check: backslash-n boundary ambiguity. Inspected only `VariableManager.java:216` onward under `/mnt/data/Projects/plantuml-mit/src`; the known-name scan and documented narrower Unicode boundary policy are consistent with the scoped contract. No excluded source was accessed.
- Ran exactly four focused headless exports for previously untested classifier doubts, all using `/mnt/data/Projects/ive/target/release/diagram-viewer --render INPUT --output OUTPUT` with temporary files under `/tmp`. Two returned the erroneous outside-wrapper diagnostic; two returned success with unexpanded raw labels, which were read from their SVG text nodes. No GUI instance was started.
- Did not rerun the reported 299 Rust tests, seven worker scenarios, nine audit regressions, captures, source hashes or credit checks. Their successful results and controller-verified capture/provenance evidence remain reported evidence, not newly reproduced evidence. These classifier counterexamples are not covered by the existing tests in the diff.
- ⚠️ This is the requested task-scoped gate. Whole-branch compatibility, all unchanged geometry/service guarantees, and full original-engine parity are not independently established by this diff; the latter is explicitly outside this release's claim.
