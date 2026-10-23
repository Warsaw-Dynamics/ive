# Spec compliance

- ❌ Issues found. Task 3 implements the requested generated preamble cutover, but malformed empty values can panic, and stylesheet definition checks introduce quadratic work that rejects a small, otherwise admitted document under the default budget. Both violate the preserved rejection/resource behavior required by the brief/global constraints.
- ✅ Scope is otherwise appropriate: real region/alignment/colon, skinparam, stylesheet/selector/brace/property/newline and EOF productions replace the removed style syntax (`diagram-ports/engine/src/plantuml/parser/grammar/preamble.lalrpop:6`, `:8`, `:28`, `:31`, `:46`). Typed value validation/application remains in `style.rs`; no whole-command forwarding, generated recovery, retry or fallback appears in this diff. Family grammar breadth remains deferred.
- ⚠️ Cannot independently verify unchanged native measure/paint/cache assertions, actual capture contents, the full gate outputs, fixture/vendor hashes or generator reproducibility from this task diff alone. The complete 135-line task report claims 537 Rust tests plus seven worker faults, 18 Node tests, generator3/notice5, all 16 gates, 43 immutable baseline records and native captures. I did not duplicate those suites, audits, generation, baseline checks or captures; the controller is independently verifying them.

# Strengths

- The grammar owns the relevant structures and whole-envelope EOF. Selector and declaration repetitions reduce to unit with fallible checkpoints rather than retaining an unbounded syntax tree (`diagram-ports/engine/src/plantuml/parser/grammar/preamble.lalrpop:7`, `:30`, `:39`).
- The style lowering preserves the existing target distinctions and authored property order. Region replacement, JSON/YAML's limited chrome policy, HCL's family ownership, immutable literal evaluation and the remaining family adapters remain explicit in the cutover (`diagram-ports/engine/src/plantuml/parser/preamble.rs:64`; `diagram-ports/engine/src/plantuml/style.rs:100`).
- The first-family result is a physical byte span, and the facade narrows the borrowed BodySource rather than deriving a syntax boundary from a normalized Line index. The new dedicated CRLF/BOM/definition/style test verifies the exact indented family remainder (`diagram-ports/engine/src/plantuml/parser/preamble.rs:550`).
- Comment-removal offsets are represented as ordered breakpoints with binary lookup; generated and value failures use the existing structured once-only origin mapping. The new origin test checks inline comments, UTF-8, expansion use location and absence of duplicate mapping (`diagram-ports/engine/src/plantuml/parser/ownership.rs:196`; `diagram-ports/engine/tests/plantuml_styles.rs:365`).
- Actual generated output, hashes and semantic-source destinations are registered without changing dependencies, notices, fixtures, vendors or existing generated grammars (`diagram-ports/engine/src/plantuml/generated/manifest.json:26`; `diagram-ports/provenance/plantuml-parser.json:2`). Admission explicitly includes simultaneous adapter/token/literal retention and diagnostic growth; the supplied witness's goto cases match the generated goto cases read in the diff (`diagram-ports/engine/src/plantuml/parser/preamble.rs:625`; `diagram-ports/provenance/plantuml-parser-admission.md:44`).

# Issues

## Critical

- None found.

## Important

1. **Whitespace-only values panic before generated parsing.** `diagram-ports/engine/src/plantuml/parser/preamble.rs:174` computes the value start after trimming all remaining whitespace, while `:175` computes the end after trimming the entire row. For `title   `, `skinparam defaultFontSize   `, or an indented stylesheet `FontSize   ` row, start exceeds end. The slice at `:176` panics; malformed caller input no longer returns a RenderError. Confirmed with the existing final release exporter: the skinparam case reports worker exit101 and `preamble.rs:176:30: byte range starts at 28 but ends at 25`; the region and stylesheet cases also crash their worker. A `title` row without trailing spaces returns the intended typed multiline/empty-region error. Bound the normalized end to the value start, or explicitly emit/reject an empty value before slicing, and add focused whitespace-only region/skinparam/style tests.

2. **Each stylesheet now rescans/charges the whole document, making the preamble quadratic.** `diagram-ports/engine/src/plantuml/origin.rs:147` maps both sheet boundaries via `authored_line`; `:162` charges `self.text.len() + self.segments.len()` on each lookup, and the lookup linearly searches all segments. Every sheet then scans every definition at `:149`. The generated builder invokes this path for every completed sheet, including a document with no definitions. A novel focused final-release probe with 500 repetitions of `<style>\nroot {\nFontSize 20\n}\n</style>\n`, followed by `A -> B : x`, has only 19,029 source bytes and 1,000 style model items yet fails the unchanged default 5,000,000-work budget at **5,014,014**. One and 100 identical sheets succeed. The removed style parser traversed these declarations once; this new path breaks the binding linear-work/admission requirement. Use ordered segment/definition lookups or a single advancing validation pass, charge the work actually required, and preserve a focused repeated-sheet control (also with definitions).

## Minor

- None raised beyond the separately assigned baseline/configuration, vendor-warning and later-family items.

# Named checks and limits

- **Diff scope:** Read the supplied 6,527-line package once, in sequential bounded portions, including generated tables/actions, all adapter/test changes and provenance updates. No git command, checkout/index/HEAD change, source edit, suite, audit, generation or baseline rerun was performed.
- **Raw whitespace reaching the new lexer:** The document scan hunk omitted the middle of its physical-row/ownership loop. Inspected only that missing context in `parser/document.rs:170–240`; raw trailing spaces survive into Ownership::Row and `preamble.row` runs before append_line. This confirms that the empty-value panic is reachable through production normalization.
- **Public failure behavior / empty values:** Read the existing `/tmp/ive-task3-native.py` for the already authorized headless export invocation and the unchanged public engine result contract in `engine/src/lib.rs`. Used `/mnt/data/Projects/ive/target/release/diagram-viewer --language plantuml --render /tmp/ive-task3-review-<case>.puml --output /tmp/ive-task3-review-<case>.svg` for exactly four focused cases: empty region spaces, empty skinparam spaces, empty stylesheet property spaces, and the no-spaces empty-region control. Only /tmp probe artifacts were written; there was no GUI launch.
- **Definition-span mapping / once-only errors:** Inspected the unchanged SourceSpan/Origin/Segment contract and preprocess source-span construction, plus `parser/error.rs`, to check that physical boundary bytes can map to authored rows and Terminal failures bypass remapping. No additional issue found in that path. This was one focused check for that concrete cross-component risk, not a broader preprocess review.
- **Generated allocation proof:** Read the supplied `/tmp/ive-task3-stack-proof.py` and matched its goto cases against the generated diff. The witness is an actual table exploration, not a universal copied bound. Did not rerun the witness or independently repeat the controller's runtime/table proof; actual numerical maxima and complete allocation proof remain controller verification limits.
- **Repeated-sheet work:** Ran only the novel 1/100/500-sheet exporter probe above. Inputs remain `/tmp/ive-task3-review-repeated-sheets-{1,100,500}.puml`. Outcomes were exit0 / exit0 / exit1, with the precise budget error recorded in Important2. No former-head baseline was generated or executed; the former single traversal and admitted sheet repetition are visible in the removed code in the supplied diff.
- **Environment:** Every probe subprocess inherited `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; no Cargo invocation was needed. The known bwrap RTM_NEWADDR startup failure required scoped read/probe escalation. There was no approval rejection or unresolved access block.

# Assessment

**Spec compliance:** Needs fixes.

**Task quality:** Needs fixes.

**Reasoning:** The generated structural cutover is clear and appropriately bounded in scope, with useful behavioral and cancellation tests. The unchecked empty slice and repeated whole-document stylesheet checks are concrete regressions on the newly introduced production path and should be fixed before Task 3 approval.
