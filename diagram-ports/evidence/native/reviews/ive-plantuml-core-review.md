# SPEC COMPLIANCE: Issues found

Task 3A is a substantial native subset checkpoint with the requested architecture, but it does not yet satisfy the acceptance contracts for preserved/contained text, complete syntax consumption, explicit unsupported-feature rejection, and cooperative work bounds. Later families and original Graphviz/Teoz pixel parity are outside this checkpoint and are not findings.

# CODE QUALITY: Needs fixes

Review base `f16795fa96f94826b8d2dc567524d14fcf0fe06f`, head `b9815e81df754985a7bf4c1159d196251a5acb24`. References below are relative to the reviewed worktree. No checkout, index, branch, or product files were changed. Diagnostic inputs/outputs and this report are under `/tmp`.

## Strengths

- `diagram-ports/engine/src/plantuml/mod.rs:20` preserves the public render interface and routes typed sequence/UML models through a real measurement session, resvg-safe terminal processing, resource admission, and final SVG validation. The diff adds no engine filesystem, network, external-process, GPUI, or raw Mermaid-source conversion path.
- `diagram-ports/engine/src/plantuml/graph.rs:107` builds a directed compound multigraph with unique edge IDs; `graph.rs:162` uses the checked parent batch, and `graph.rs:54` runs Dagre through charged WorkControl retaining the real interruption reason. The private cyclic-parent and inside-kernel interruption tests are meaningful.
- `diagram-ports/engine/src/plantuml/text.rs:39` keys measurement caching by text and admitted decoration, requests the corresponding italic style, and checkpoints around host calls. `diagram-ports/viewer/tests/native_lifetime.rs:62` exercises actual host measurements and cache release. Baseline approximation is disclosed rather than attributed to native ink metrics.
- `diagram-ports/engine/src/plantuml/svg.rs:7` emits owned primitives with escaped attributes/text, and its writer checks output growth before append. Scene validation rejects nonfinite and negative primitive geometry.
- `diagram-ports/provenance/plantuml-mit-1.2026.9.json:1`, `provenance/LICENSE-PlantUML-MIT.txt:1`, and `tools/audit-plantuml.mjs:1` retain the authorized edition/artifact, 24 exact source mappings, fixture provenance, MIT grant and named contributor. No rejected candidate, Graphviz/Smetana/ELK translation, or extra crate version appears in the diff.
- `diagram-ports/viewer/src/cli.rs:89`, `viewer/src/examples.rs:48`, and the app changes supply real language-specific examples and extension detection while preserving the existing isolated worker architecture. Documentation clearly calls this a subset; description shape examples are not inflated into separate backend counts.

## Important findings

### 1. Sequence row allocation places multiline content over earlier events

**Location:** `diagram-ports/engine/src/plantuml/sequence.rs:390` (row allocation), `sequence.rs:494` (event drawing), `sequence.rs:426` (created participant heads).

Rows advance by the current label height *after* assigning its anchor, but message labels paint above that anchor and notes straddle it. Consequently the current event's height is not reserved before it is painted. This is a topology/readability defect in explicitly admitted multiline syntax, not a difference from Teoz's preferred spacing.

**Reproduction:** `/tmp/ive-review-multiline.puml` contains `A -> B : first`, followed by `A -> B : line1\nline2\nline3\nline4\nline5\nline6`. Headless rendering succeeds. The first message is at y=145, its label baseline is 137.889, and the second message's text baselines span 96.289–193.289 at the same x=200. Its third line baseline is 135.089: the labels visibly overlap, and later lines cross the earlier arrow. `/tmp/ive-review-note.puml` also succeeds with a six-line `note over B`; the note starts at y=142.2 and paints over the previous arrow endpoint at y=145.

Creation has the same missing occupied-height accounting: `/tmp/ive-review-created.puml` creates a four-line quoted participant, whose head spans y=187.6–289.2, but the next message is drawn through its head at y=243.

**Required fix:** Allocate rows from each event's full occupied bounds above/below its anchor, including notes, frame headings, self calls and created heads. Add geometric non-overlap tests for consecutive short/long messages, multiline notes after messages, and multiline created participants. Existing lane/event identity tests do not cover this failure.

### 2. Compound layout discards caption width, producing clipped labels

**Location:** `diagram-ports/engine/src/plantuml/graph.rs:107`, `graph.rs:204`, `graph.rs:223`; `diagram-ports/engine/src/plantuml/scene.rs:140` and `svg.rs:7` only grow positive canvas extents.

The measured container title is passed as a node width, but the returned compound box can be narrower than that title. Drawing centers the title in that returned box without restoring a minimum caption width. Negative text coordinates are then accepted while the SVG viewBox remains rooted at (0,0), clipping actual text.

**Reproduction:** `/tmp/ive-review-container.puml` is a package named `Extremely long container caption that must stay within its box`, alias P, containing only `class A`. It succeeds. The package spans x=60–220, but its title starts at x=-43.19. The output viewBox is `0 0 347.19 247.8`, so the left portion of the caption is outside the canvas. This is plain supported package text, independent of complex edge routing or original geometry parity.

**Required fix:** Reserve measured captions in compound layout, enforce container/caption containment, and account for scene minimum coordinates when generating the canvas. Test long and multiline captions in both layout directions. The fixture-specific containment test at `engine/tests/plantuml.rs:525` checks child bounds but does not establish this invariant.

### 3. Endpoint labels are painted inside nodes without layout space

**Location:** `diagram-ports/engine/src/plantuml/graph.rs:335` and `graph.rs:338`.

Endpoint multiplicities/roles are measured only during drawing and always placed at `(endpoint.x + 10, endpoint.y + 4)`. On a downward relation the target endpoint is the target node's top boundary, so the label is placed inside that node. The placement does not depend on edge direction, node bounds, marker extent or measured text size.

**Reproduction:** `/tmp/ive-review-multiplicity.puml` contains `class A`, `class TargetName`, and `A --> "many targets" TargetName`. The target box spans y=211.4–256.4 and its title baseline is y=239.689. The role label starts inside it and has baseline y=231.689, with overlapping horizontal ranges. This happens on a single uncomplicated relation, rather than the acknowledged complex-edge/caption routing residual.

**Required fix:** Measure endpoint labels before layout, reserve their occupied space, and place them outside the owning shape using the route tangent/normal and node bounds. Add simple TB/LR and reverse-edge role/multiplicity non-overlap tests.

### 4. Parser branches accept substantive suffixes or endpoint syntax without consuming/validating them

**Location:** `diagram-ports/engine/src/plantuml/uml.rs:231`, `uml.rs:247`, `uml.rs:521`; analogous sequence splitting/command dispatch at `sequence.rs:85`, `sequence.rs:156`, and lifecycle admission at `sequence.rs:216`.

Both parsers split off an unquoted colon label before deciding the command. Declaration branches ignore that label. The right endpoint branch for a quoted multiplicity also skips identifier validation, unlike the ordinary endpoint branch. Lifecycle commands likewise do not apply endpoint identifier validation. These holes contradict the full-consumption and explicit rejection contract.

**Reproductions:** `/tmp/ive-review-suffix.puml` contains `class A : discarded member` and returns success with only A painted; the trailing substantive text disappears. Whether a suffix is supported upstream or malformed, silently accepting and deleting it is wrong for this frontend. `/tmp/ive-review-role.puml` contains `class A` and `A --> "1" B/role`; it succeeds by creating a class literally named `B/role`, despite slash-role syntax being explicitly listed as rejected. Without the quoted multiplicity the ordinary endpoint validator rejects the same token.

**Required fix:** Let each grammar consume its complete source statement, or explicitly reject remaining label/suffix tokens for commands that do not support them. Route every endpoint/lifecycle name through the same validated name contract, preserving quoted-vs-unquoted semantics. Cover both multiplicity branches and non-message commands in negative tests.

### 5. Rich-text rejection recognizes only a partial blacklist and silently accepts substantive styles

**Location:** `diagram-ports/engine/src/plantuml/text.rs:100` and `text.rs:108`.

The supported-input ledger says rich text is rejected, but the substring list omits valid styles and spelling variants. Accepted markup is then XML-escaped and painted literally, losing its authored semantics while reporting success.

**Reproduction:** `/tmp/ive-review-markup.puml` contains `A -> B : <s>strike</s>`. It succeeds and paints the entire literal `<s>strike</s>` with `text-decoration="none"`. This is not arbitrary XML-looking literal text: the authorized MIT source explicitly defines `<s>`/`</s>` as STRIKE activation/deactivation (`/mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/klimt/font/FontStyle.java:181` and `:259`), and `klimt/creole/legacy/CommandCreoleBuilder.java:163` registers that style.

**Required fix:** Recognize unsupported rich-text syntax with a bounded lexical grammar, including case/alias variants and escape rules, rather than an incomplete substring blacklist. Continue preserving genuinely literal angle text and URLs. Test unsupported style variants in each label-bearing path, including implicit names.

### 6. Comment scanning has uncharged quadratic work and delays cancellation

**Location:** `diagram-ports/engine/src/plantuml/source.rs:54` and `source.rs:84`.

For each unquoted apostrophe after non-comment content, `out.trim().is_empty()` rescans all preceding leading whitespace. A line containing N leading spaces, one nonspace character and N apostrophes therefore performs quadratic work. Only the source-byte count is charged up front; the scanner has no inner checkpoint and cannot observe cancellation during this work.

**Reproduction:** `/tmp/ive-review-comment-work.puml` contains 65,536 spaces followed by A and 65,536 apostrophes inside a wrapper (about 128 KiB, below the default source ceiling). One frozen-release headless run took 2.40 seconds before the worker rejected the oversized eventual diagnostic. A direct-engine probe `/tmp/ive-review-budget.rs`, linked against the current built engine, sets `timeout=10ms` and `layout_work=200_000`; it took **2.414949398 seconds** to return deadline cancellation. That is one probe, not a repeated benchmark. The independent hard worker deadline still provides an outer ceiling, but it does not satisfy the required cooperative engine work contract.

**Required fix:** Track whether non-whitespace output has occurred in constant time, and charge/checkpoint bounded scanner chunks. Add an adversarial long-line regression that verifies cancellation/work limits are reached within scanning rather than only after the whole line.

## Minor findings and scoped gaps

- **Located diagnostics remain incomplete:** `diagram-ports/engine/src/plantuml/lexer.rs:9` stores no source spans; `budget.rs:78` hardcodes column 1, and `sequence.rs:273` reports note errors at line 1 regardless of the real statement. The detailed Task 1 plan required token spans and located diagnostics. The report discloses the column limitation, but that disclosure does not implement the requirement. At minimum preserve the actual note line; token columns can be a clearly tracked follow-up.
- **Large new mixed-responsibility units:** `sequence.rs:156`, `sequence.rs:494`, `uml.rs:247`, and `graph.rs:196` contain broad parsing/drawing matches; the newly added sequence and UML files are 727 and 528 lines. This deviates from the planned parse/layout/resolve separation and several helpers take 4–7 arguments despite the provided context abstraction. Not a separate blocking defect, but extraction around the fixes above would make validation invariants easier to share.
- **Warning noise:** the implementation report records a preexisting vendored Merman deprecated `Atomic::fetch_update` warning. This is a Minor test-output finding under the reviewer instructions, not evidence of a new PlantUML runtime regression. Do not alter audited vendor sources merely to silence it without the normal provenance update.

## Checks and verification limits

- Read the task brief, implementation report, complete assigned diff in bounded chunks, detailed native plan, parent design and source-analysis plan. Only the initially truncated diff section was reread; later lookups extracted line numbers for report references. No changed source file was separately reread.
- Ran only focused new-doubt probes: eight small headless syntax/geometry inputs, one bounded hostile-line CLI probe, and one direct-engine tiny-deadline probe. All processes terminated; no GUI window was started. All successful SVG probes were inspected as XML with numeric geometry. The temporary files named above retain the exact reproductions.
- The one focused source check outside the diff was the named risk that strike markup might be substantive PlantUML syntax; only authorized MIT Creole/font source paths were consulted. The generated outputs and current compiled artifact were inspected only to substantiate the named geometry/parser/deadline risks.
- Did not rerun the controller-evidenced 131 Rust tests, seven worker fault scenarios, source audits, clippy, format, release build, licence checks, or Mermaid suite. Their passing results are useful evidence but do not cover the counterexamples above.
- **Cannot independently verify from this diff:** unchanged worker hard ceilings/reaping on every platform, actual font/capture appearance, and all source-JAR byte matches. The controller independently supplied seven fault passes, inspected three real captures, and matched all 24 source entries; those are acknowledged rather than claimed as this reviewer's own reruns. Original/native five-fixture visual differences are not treated as parity failures: this task never promised original pixel equivalence.
- The exact public render API, preserved Mermaid path, changed dependency edges, source notices, typed sequence/UML architecture, graph safety calls, fixed-font measurement/painting agreement, and viewer admission are verifiable in this diff. No IVE product integration or later-family implementation was added.

**Assessment:** Needs fixes before Task 3A is trusted as its declared subset. The foundation and provenance work are sound, but ordinary admitted diagrams can lose visible text or overlap semantic content, unsupported syntax can silently succeed, and a bounded-size source defeats the intended cooperative deadline granularity.
