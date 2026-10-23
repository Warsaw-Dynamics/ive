### Spec Compliance

- ❌ Issues found. Task 2's generated document/ownership/span cutover is present, but the typed header cutover does not preserve the supported subset. It rejects valid sequence endpoints containing an admitted `.` or `@` after a keyword prefix, and valid implicit state labels containing an arrow. Both failures occur in the Task 2 header evidence layer, before the unchanged family parsers. See Important 1 and 2.
- ✅ Required file coverage is present: `parser/{document,ownership,header}.rs`, `parser/grammar/{document,header}.lalrpop`, `source.rs`, `preprocess.rs`, `origin.rs`, `mod.rs`, parser tests and generation/provenance inventories all have corresponding changes in the supplied diff.
- ✅ Generated document syntax has real paired-wrapper alternatives, fixed content/comment/style folds and complete EOF; wrapper-specific outside-comment rules remain explicit (`diagram-ports/engine/src/plantuml/parser/grammar/document.lalrpop:4`, `:19`, `:23`). The facade calls the generated parser, and the old production wrapper/comment state machines are removed (`diagram-ports/engine/src/plantuml/parser/document.rs:119`, `diagram-ports/engine/src/plantuml/source.rs:32`).
- ✅ The production route uses generated document parsing followed by preamble handling and one typed selection, with no parser retry or legacy wrapper fallback (`diagram-ports/engine/src/plantuml/mod.rs:63`, `:73`). Remaining family syntax is disclosed as pending in the report and README; its migration is outside this gate.
- ✅ Gantt's assigned lexical EOF repair has an explicit public render regression with a closed-comment positive control (`diagram-ports/engine/tests/plantuml_gantt.rs:470`). The all-wrapper contract covers pairing, suffixes, extra input/documents, BOM/CRLF, trailing unterminated comments and the accepted comment/case policies (`diagram-ports/engine/src/plantuml/source.rs:70`).
- ⚠️ Cannot verify from this diff alone: all common gates, exact original source/ZIP grants, full unchanged fixture/Merman inventories, mechanical generator reproduction, baseline equality and genuine native frames. The implementer provides named commands/logs and results in `task-2-report.md`; the controller is independently verifying those gates. I did not repeat suites, audits, generation, baseline checks or captures. No modified fixture, golden, baseline, dependency lock, public render/options/control API or runtime host file appears in this task diff.
- ⚠️ Admission review establishes that the checked-in proof was renewed for the new tables and that the facade precharges actual token capacity, generated/diagnostic storage and work. I read both generated tables/runtime paths and the admission additions, but did not independently rerun the implementer's exhaustive reachable-stack witness (`diagram-ports/provenance/plantuml-parser-admission.md:17`, `:25`, `:27`). This is a verification limit, not an additional defect.

### Strengths

- The authored and expanded passes now share one lexical owner. Current-row literal ownership and next-row expanded note/action transitions are explicit, and atomic two-byte delimiters have one implementation (`diagram-ports/engine/src/plantuml/parser/ownership.rs:65`, `:105`, `:137`, `:269`). Dedicated controls exercise raw/expanded terminators, inert comments and literal data (`diagram-ports/engine/src/plantuml/parser/document.rs:266`).
- `BodySource` borrows the origin document and exact body extent, while the transitional adapter materializes physical rows without a second legacy wrapper/comment pass. The CRLF/UTF-8 contract asserts the actual borrowed bytes and physical line number (`diagram-ports/engine/src/plantuml/parser/document.rs:236`, `:419`; `diagram-ports/engine/src/plantuml/parser/mod.rs:14`).
- Structured failures preserve terminal errors and map through origins at the facade. Normalized header errors use a complete physical-row extent, and tests distinguish overlapping definition uses, unrelated definitions and direct UTF-8 byte columns (`diagram-ports/engine/src/plantuml/origin.rs:64`, `:106`, `:387`; `diagram-ports/engine/src/plantuml/parser/header.rs:365`).
- Syntax buffers use fallible reservation, source-derived growth checks and actual capacity checks; generated delivery checks cancellation even on attempted EOF (`diagram-ports/engine/src/plantuml/parser/context.rs:122`, `:171`, `:242`). The new document controls cover tiny admission and cancellation across delivery/reductions/EOF (`diagram-ports/engine/src/plantuml/parser/document.rs:350`).
- Provenance adds exact authored parser/grammar destinations without allowing generated outputs as authored translations. The guard tests include registered positives and unregistered/generated/path-traversal negatives (`diagram-ports/tools/audit-plantuml.mjs:9`, `:21`; `diagram-ports/tools/audit-plantuml.test.mjs:8`).

### Issues

#### Critical (Must Fix)

- None found in this task-scoped review.

#### Important (Should Fix)

1. **Keyword prefixes inside admitted endpoint identifiers become declaration headers.** `diagram-ports/engine/src/plantuml/parser/header.rs:283`, `:288`, `:128`. `scan_word` stops at `.` and `@`, although the unchanged identifier contract admits both characters (`diagram-ports/engine/src/plantuml/lexer.rs:180`). Consequently `class.foo -> B : ping` becomes `Graph(class), Other(.), Word(foo), Arrow, ...`; the arrow is no longer the first tail atom, so `identifier_arrow` is false and the generated Graph head selects UML. The removed selector instead considered the whole whitespace-delimited endpoint and correctly skipped declaration evidence for its following arrow. The same problem affects `class@foo` and other keyword prefixes.

   Focused public-export evidence from the reviewed release binary:
   - `/tmp/ive-task2-header-dotted.puml`, containing `@startuml\nclass.foo -> B : ping\n@enduml`, exited **1**: `invalid diagram source: PlantUML at line 2, column 1: undeclared description entity "class.foo"`.
   - `/tmp/ive-task2-header-at.puml`, containing the same message with `class@foo`, exited **1** with the corresponding undeclared description entity.
   - `/tmp/ive-task2-header-dotted-control.puml`, adding `participant class.foo` before the same dotted-endpoint message, exited **0** and exported `/tmp/ive-task2-header-dotted-control.svg`. This confirms the unchanged sequence parser accepts the endpoint when family evidence selects sequence.

   Preserve complete admitted endpoint-name ownership before assigning keyword roles, while retaining actual arrow/timing punctuation boundaries. Add public family/render regressions for keyword-prefixed dotted and `@` identifiers without an earlier participant declaration. Existing simple `class -> state` coverage (`diagram-ports/engine/src/plantuml/parser/header.rs:455`) does not catch this boundary.

2. **Arrows in state label payload suppress state-family evidence.** `diagram-ports/engine/src/plantuml/parser/header.rs:81`, `:155`. `Tail::append` sets `arrow` for every atom, including atoms after the first label colon, and `finish` requires `!tail.arrow` for a state candidate. The removed selector applied its arrow exclusion only to `lexer::split_label(line).0`; label contents did not affect header classification. A supported implicit state such as `Idle : waiting -> ready` now selects sequence and fails instead of rendering a state with the literal arrow-bearing label.

   Focused public-export evidence from the reviewed release binary:
   - `/tmp/ive-task2-header-label-arrow.puml`, containing `@startuml\nIdle : waiting -> ready\n@enduml`, exited **1**: `unsupported diagram feature: PlantUML at line 2, column 1: "Idle : waiting -> ready"`.
   - `/tmp/ive-task2-header-label-control.puml`, with `Idle : waiting`, exited **0**.
   - `/tmp/ive-task2-header-label-arrow-control.puml`, adding `state Idle` before the identical arrow-bearing label, exited **0** and exported its SVG. This isolates the failure to header selection; the unchanged state parser accepts the label.

   Freeze arrow/header facts at the first label colon, just as `last_head` already stops there, and test literal arrow text in an implicit state label through the public API. Keep quote and `::` handling intact.

#### Minor (Nice to Have)

- None added. The known later-family Minors, Task 17 baseline configuration reporting and inherited vendor warning triage were outside this task's gate and were not reopened.

### Checks Performed

- Read the exact Task 2 brief, binding global context, approved Task 1 interface and implementer report. Read the supplied `review-af042d7..86f56a6.diff` sequentially in bounded portions; recovered only the omitted segment of an initially truncated generated-output portion. No changed file was separately reread. Exact target line references were mechanically mapped from the supplied diff.
- One focused outside-diff check for the concrete **keyword endpoint lexical-boundary risk**: read `lexer::arrow_at`, `lexer::split_label` and `lexer::validate_identifier` in the single bounded `lexer.rs:105`–`:192` excerpt. The admitted identifier characters and old arrow/label boundaries establish the expected behavior for both findings. No broad repository crawl occurred.
- Inspected the existing release viewer's `--help` to select the documented headless public-export interface. Ran only the six focused public exports listed above, each with `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; no Cargo build, suite or generator subprocess was run. Command form: `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target /mnt/data/Projects/ive/target/release/diagram-viewer --language plantuml --render /tmp/PROBE.puml --output /tmp/PROBE.svg`.
- Scoped sandbox escalation was required because the sandbox launcher failed with `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`. All probes were headless and exited; no test app or process was left running. Only `/tmp` probe/report files were written; checkout, index, HEAD and branch state were not mutated.

### Assessment

**Task quality:** Needs fixes.

**Reasoning:** The generated document/ownership/span infrastructure is well separated, bounded and covered by substantive controls. The header evidence lexer loses two previously supported lexical boundaries, producing confirmed public behavior regressions that must be corrected before approving the Task 2 cutover.
