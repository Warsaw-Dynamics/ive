# Task 2 — generated document/ownership/header front end

Status: implemented, verified, scoped commit made, and tracked tree frozen clean for independent controller review. No independent review was self-dispatched.

- Base: `af042d7f18713d65d46c160303aa3232d5aa2082`.
- Head: `86f56a6a9dd224af711b67dc807777eb7bb61999` (`Replace PlantUML document ownership with generated parsing`).
- Worktree: `/mnt/data/Projects/ive/.worktree/diagram-ports`; Rust workspace: its `diagram-ports/` child.
- Root progress/master/both ledgers were not edited. No main IVE product changes, merge, push, user-state changes or worktree removal occurred.

## Removed syntax and production route

Removed both legacy source wrapper state machines (`source::prepare_document`/`prepare` implementation), `TextBody`/`text_body`, `graph_declaration`, `strip_comments`, `reject_active_preprocessor`, and the duplicate preprocessing `CapabilityScan` byte/comment implementation. Removed the old string-based `mod::select` and `uml::is_graph` family-detection pass. `source.rs` now contains shared wrapper/Line values and a test-only adapter that constructs a direct origin document and calls the new facade; it cannot invoke the removed comment/wrapper logic.

Production now follows caller source → bounded authored `Ownership` plus generated document validation → unchanged eager immutable definition evaluator/registry → origin `Document` → bounded expanded `Ownership` plus generated document validation → origin-backed `BodySource` and temporary physical Lines → existing preamble and one typed header selection → one unchanged family model parser/layout. There is no parser retry or legacy source fallback. The original and expanded ownership views are intentional; original current-row raw ownership stays authoritative, while expanded note/action terminators set the next-row mode.

`grammar/document.lalrpop` owns all thirteen paired wrappers, document completeness, nonempty content, one document/EOF, definition placement inside the wrapper and outside nonliteral stylesheet blocks, and the explicit per-wrapper outside-comment rules. UML alone accepts prefix comments. UML/Gantt/nwdiag/Chen accept closed trailing comments; the other exact wrappers still reject them. Output suffix filenames, mismatched/nested/extra documents, extra content, and uppercase non-UML wrappers remain rejected. All wrappers validate lexical EOF, including the repaired Gantt trailing unterminated block comment. BOM is stripped only at document start; CRLF/physical line indices remain intact.

`grammar/header.lalrpop` uses explicit typed primitive terminals, generated first-header roles and complete EOF. Its fixed typed semantic fold handles arrow-endpoint precedence and note/pseudostate evidence without resplitting source statements. Keywords stay lexical names in other positions. Wrapper-specific families bypass UML header selection; UML headers select exactly one family. Tests cover keyword endpoints, actors/databases, quoted graph versus sequence notes, empty rows and `[*]` targets.

`BodySource<'s> { document: &'s origin::Document, range: Range<usize> }` borrows the expanded body exactly between the physical wrapper rows. `ParsedDocument` carries its wrapper and temporary Lines. Lines are materialized in the same scan into one bounded buffer; raw continuations are lexical payload for document syntax while retaining individual physical lines for unmigrated model parsers. No second retained-row buffer or conflicting wrapper/comment pass remains. Mindmap/WBS adapters preserve their original raw row text. Raw data owns style-looking literal payload; its prefix style/late-definition semantic checks remain in the existing preamble facade, pending Task 3.

Structured generated document/header failures keep a category and expanded byte span until their facades map once through `origin::Document`. Overlapping expansion uses, exact authored physical lines/UTF-8 byte columns and frozen definition lineage are retained; unrelated definitions are not attributed. A normalized header uses the truthful complete physical-row extent instead of claiming precision through removed comments. Public InvalidSource/UnsupportedFeature and cancellation/resource terminal variants remain. Legacy family/layout errors retain the old bounded compatibility bridge until their assigned cutovers.

## Observed RED and GREEN

All Cargo commands and script subprocesses used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; Cargo commands used `--locked --offline` where supported. Commands ran in the standalone child workspace with scoped sandbox escalation for the known environment restriction.

1. Before implementation, `cargo test --locked --offline -p diagram-engine --test plantuml_gantt should_reject_unterminated_trailing_block_comment_after_gantt_wrapper -- --nocapture` failed at the assertion requiring InvalidSource: the complete Gantt document actually accepted `/' unterminated` after `@endgantt`. Its proper closed-comment control succeeded. An earlier attempted run omitted the explicit PlantUML option and failed for a Mermaid detection error; that setup failure is **not** counted as RED. The correct initial RED is recorded here from the observed tool output; it was not originally tee'd to a durable log.
2. The new `should_borrow_body_bytes_and_preserve_physical_utf8_lines` contract compiled against the permitted explicit unavailable stub and failed with `Backend("generated document contract unavailable")`. The shipped implementation borrows the expected `"\r\n  A -> B : Ω\r\n"` body and preserves physical line 3.
3. `/tmp/ive-task2-header-red.log`: generated header EOF initially rejected the empty physical row, failing `should_derive_typed_headers_without_confusing_keyword_endpoints_or_empty_rows`. Added the generated empty-header production; the complete family ambiguity controls now pass.
4. `/tmp/ive-task2-header-budget-red.log`: source admission test observed work `8195` instead of `0` for an 8-byte source limit. Added complete source/view admission and chunked quote/word/arrow scans before token retention. The cancellation half now stops before finishing the long UTF-8 primitive.
5. `/tmp/ive-task2-ownership-red.log`: comment ownership initially returned success after its precharge despite the controlled cancellation checkpoint. The shared scanner now checks during the actual comment scan; this control passes.
6. `/tmp/ive-task2-strict-comments-red.log`: the intermediate shared lexer overaccepted closed trailing comments for strict mindmap wrappers (`left: true`, `right: false`). Generated comment productions restored the prior per-wrapper prefix/trailing policy. The named all-wrapper document/EOF test now passes with explicit accepted/rejected controls. Its earliest draft had incorrectly assumed every wrapper accepted closed trailing comments; that draft expectation was corrected, not presented as an intended new-positive change.
7. `/tmp/ive-task2-provenance-red.log`: the new exact parser/grammar destination guard failed with `unregistered Rust destination`. The audit now admits only exact authored destinations in the checked parser inventory and still rejects unregistered paths and generated outputs as authored destinations. `/tmp/ive-task2-provenance-green.log`: 13 source guards pass; combined final Node gate: 18/18.

Final focused internal GREEN is `/tmp/ive-task2-controls-green.log`: 139 passed, 0 failed. Direct controls include generated token streams with mismatched wrappers/outside definitions/extra input; actual source slices/physical CRLF lines; raw authored versus expanded terminator cases; comment-owned definition/include/style/wrapper text; quoted/raw/data literals; once-mapped frozen-definition errors; exact direct/expanded UTF-8 byte columns; tiny admission; cancellation during delivery, reductions and attempted EOF. The existing 44 constants regressions remain unchanged, including both atomic-delimiter overlap directions, inert resources, generated directive denial, eager freeze/no-rescan, and note/action ownership.

Intermediate full runs exposed and then resolved the existing state setup work bound, below-cap action work assertion and 60,000-row/500ms action deadline guard by removing duplicate retained rows/header work and raw continuation shifts. No existing assertion or limit was relaxed. Subsequent engine runs also caught and resolved the pseudostate-before-colon summary, literal files blank row and YAML literal `<style>` ownership regressions. The first final common-gate run exposed eight source-audit Node failures because new parser destinations were not registered by the old validator; the exact inventory guard fix above resolved them. No known failing test remains.

## Renewed generated allocation/control proof

Full durable proof: `diagram-ports/provenance/plantuml-parser-admission.md`. Final exact generated hashes:

| Input | Grammar SHA256 | Generated SHA256 |
| --- | --- | --- |
| definition | `0983da93c3f2a0d4920102036e4409abcf8078b0c9f4faf26e9808e7f54fb675` | `da80ae86dbb675e969dbf2be173d2536f0f5d4c2c1a121b6b6a39371809dc8ad` |
| document | `b4538391acaee6c48cbbd0e938d3200ac169c8e6617a60a87b44c44286e73e4b` | `761b856999aacafae5a744ddaed56720ccf8bd412433d69572cefe20b16bb244` |
| header | `af0b96ed412055c0f9596428a97515f675297739f9d84b7cdf4a9b068ba6e6cf` | `094a5bd7bd1d146cb25f0b7b945e0eb01d26db6104d23778d3a9ba7b156ce74c` |

Inspected the actual generated ACTION/EOF/goto/simulated-reduction paths and exact cached `lalrpop-util 0.23.2` state-machine allocations. Final development witness `/tmp/ive-task2-stack-proof.py`, output `/tmp/ive-task2-stack-proof.log`, enumerates reachable shift stacks and every terminal/EOF simulation using the inspected goto maps. These temporary witness files are development evidence; the durable checked-in proof and exact inventory remain the review contract.

| Grammar | States/terminals | Reachable shift stacks | Max live states/symbols | Max consecutive reductions | Max expected terminals | Diagnostic capacity/growth peak |
| --- | --- | --- | --- | --- | --- | --- |
| document | 56/31 | 123 | 7/6 | 3 | 14 | 10/15 state bytes |
| header | 55/25 | 51 | 4/3 | 4 | 25 | 8/10 state bytes |

The admission uses the final grammar's 31/25 probes, conservative four reductions per token plus EOF, eight state slots, eight/four symbol slots, sixteen diagnostic slots, twice the 32-slot expected-String vector capacity and 32 bytes per terminal name. Concrete payload-size/tag/location sums include symbol-vector growth transients; forty state bytes cover generated and diagnostic growth peaks. Actual token capacity is included. Stack/diagnostic work is precharged before generated entry; syntax buffers check count/requested/current and actual capacity, source-derived bytes, cancellation and copy work before append. Reservations are fallible. Syntax does not consume semantic model-item quotas. The conservative combined syntax ceiling remains `source_bytes × 256`.

There is no recovery, dropped-token accumulation, nullable recursive cycle, recursive syntax builder or retained syntax list in either new grammar. Left-recursive body/comment/style/header folds retain fixed summaries. Iterators check a cloned control on every token and attempted EOF; fallible retaining/final actions check the single budget. Byte/comment/word/quote/arrow scans check every256 bytes (up to259 across UTF-8). Existing semantic limits/defaults/model/layout assertions are preserved. No process watchdog is substituted for internal accounting evidence.

## Complete final gates

Every command below exited 0 after the final product/source-audit edit. Logs use `/tmp/ive-task2-final-<name>.log`:

| Name | Command/result |
| --- | --- |
| workspace-test | `cargo test --locked --offline --workspace`: **520 ordinary Rust tests**, 0 failures, plus **seven actual worker fault scenarios**: blocked-stdin, blocked-stdout, stderr-flood, oversize-frame, early-exit, memory, cancel |
| clippy | `cargo clippy --locked --offline --all-targets -- -D warnings` |
| fmt | `cargo fmt --all --check` |
| release | `cargo build --locked --offline --release` |
| deny | `cargo deny --locked --offline --config ../deny.toml check licenses sources`: licenses/sources OK |
| plantuml-audit | `node tools/audit-plantuml.mjs`: 220 exact MIT source/ZIP hashes, 43 fresh fixtures, 43 native modules; checked authored/generated parser inventory |
| merman-audit | `node tools/audit-merman.mjs`: 656 imported files and four recorded patches unchanged |
| node-guards | `node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs tools/parser-baseline.test.mjs`: 18 passed |
| generator-test | separate manifest locked/offline Cargo tests: 3 passed |
| generator-clippy | separate manifest locked/offline all-target Clippy with `-D warnings` |
| generator-fmt | separate manifest `cargo fmt --all --check` |
| generator-deny | separate manifest locked/offline license/source deny: OK; existing unused license-allowance warnings only |
| generation-check | pinned separate manifest `cargo run --locked --offline ... -- check`: 3 reproducible parsers; no tracked outputs written |
| python-guards | `python3 -B tools/audit-plantuml-generator.test.py`: 5 passed |
| generator-audit | `python3 -B tools/audit-plantuml-generator.py check`: 67 exact registry archives, unpacked sources, licenses, feature closure and notice coverage |
| baseline | `node tools/parser-baseline.mjs check`: all 43 source/family/SVG records unchanged |

The immutable baseline SHA256 remains exactly `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`. No fixture/golden rerecord occurred. The driver retains all three source/family/SVG equality assertions. Generator 0.23.2, its separate crate/lock/feature/notice closure and the unchanged runtime-only utility dependency remain; no new dependency fetch or runtime generator was introduced. Existing vendored Atomic deprecation warnings remain deferred, and strict workspace Clippy succeeds.

MIT traces now map actual wrapper/ownership/document grammar behavior to the retained BlockUmlBuilder/DiagramType entries and header evidence to PSystemBuilder, preserving their original exact source hashes, copyright credits and MIT notice. Authored grammar/output inventories and the README describe partial scope truthfully; no Merman source or generation was modified.

## Genuine native paint and no-generator exports

Built release `diagram-viewer` ran with `--background --language plantuml --example NAME --capture PATH`. All four real pgui frames were inspected with the image tool, showed the expected rendered diagram/source/status, exited 0 and automatically quit. No focus activation, xdotool or OS screenshot was used; no profile/user state is created by this viewer.

| Example | Native/fallback | PNG path | Bytes |
| --- | --- | --- | --- |
| plantuml-styled-sequence | 20/0 | `/tmp/ive-task2-plantuml-styled-sequence.png` | 224962 |
| plantuml-styled-graph | 22/0 | `/tmp/ive-task2-plantuml-styled-graph.png` | 195213 |
| plantuml-constant-sequence | 12/0 | `/tmp/ive-task2-plantuml-constant-sequence.png` | 217879 |
| plantuml-constant-graph | 14/0 | `/tmp/ive-task2-plantuml-constant-graph.png` | 208165 |

Matching `/tmp/ive-task2-<example>.log` files retain the actual counts. `/tmp/ive-task2-native.py` contains the exact capture/export commands. Release exports of `constantsequence.puml` and `styledgraph.puml` with `PATH=''` exited 0, producing `/tmp/ive-task2-empty-path-constantsequence.svg` and `/tmp/ive-task2-empty-path-styledgraph.svg`. These use no display or generator. A final process-ID/prefix check found no remaining release viewer or worker process.

## Pending scope and concerns

This task is the shared document/ownership/header/span bridge, not a family syntax migration. The following actual parsers remain pending: preamble/skinparam/style/chrome extraction and placement; sequence; UML class/object/description/ArchiMate; Chen; state; structured activity; mindmap; WBS; files; board; JSON; YAML; HCL; timing; Gantt; Salt; nwdiag; packetdiag; Creole nesting; remaining handwritten arrow/name/family lexical helpers. They still receive the explicit temporary Lines adapter. Existing late-definition/style checks and legacy family/layout diagnostics are disclosed above. The other three assigned boundary Minors remain with their later family tasks, untouched.

The code was self-reviewed for complete generated routing/EOF, no retry/old comment/wrapper pass, typed terminal errors, both lexical views, truthful origin extent policy, transient allocations, actual final tables, registered legal/source inputs and unchanged baseline/model/layout assertions. No unresolved implementation blocker or known failing gate is being concealed. Independent controller SPEC/QUALITY review is the next gate; implementation is frozen at the head above.

## FIX ROUND 1 — both Important header regressions

Status: both scoped review issues fixed, focused RED/GREEN observed, covering/common verification completed, scoped commit made, and tracked tree frozen clean for the controller's re-review. This section supersedes the original head/freeze above.

- Review: `/tmp/ive-plantuml-lalrpop-document-review.md`, read in full before implementing; it reported two Important issues, no Critical or new Minor issues.
- FIX_BASE: `86f56a6a9dd224af711b67dc807777eb7bb61999`.
- FIX_HEAD: `92c9520ffbaf9d44e8dc246d66f1ed659e2ddd82` (`Preserve PlantUML header identifier and label boundaries`).
- Changed only `engine/src/plantuml/parser/header.rs`, `engine/tests/plantuml.rs`, and `provenance/plantuml-parser-admission.md`. No existing assertion, model/layout test, fixture, golden, baseline, grammar, generated output, inventory, dependency or public API was changed. Root progress/master/both ledgers remain untouched. No subagent/reviewer was dispatched.

### I1: complete admitted endpoint-name ownership

Verified the finding against the unchanged `lexer::validate_identifier` character contract and `lexer::next` arrow boundaries. The header's old `scan_word` stopped before `.` and `@`, so `class.foo`/`class@foo` exposed the `class` keyword as a generated Graph head before seeing the following arrow. The fix scans the complete admitted alphanumeric/underscore/dot/embedded-`@` name primitive, then assigns an exact keyword token only when the entire primitive matches a keyword. Constant-size slice-pattern arrow-prefix checks still stop the word at actual arrows; a leading `@` remains the separate timing primitive. The generated Header/Head/Tail productions still classify the same typed primitives. No source statement is reparsed and no parser fallback was added.

Meaningful RED, before the I1 implementation:

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --lib should_preserve_complete_keyword_prefixed_endpoint_names_in_header_evidence -- --nocapture
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml should_render_keyword_prefixed_dotted_and_at_endpoints_as_implicit_sequence -- --nocapture
```

- `/tmp/ive-task2-fix1-private-i1-red.log`: `class.foo` selected `Uml`, expected `Pending`; 0 passed, 1 failed.
- `/tmp/ive-task2-fix1-public-i1-red.log`: the forced `participant class.foo` control succeeded; the same message without a participant failed with `InvalidSource("PlantUML at line 2, column 1: undeclared description entity \"class.foo\"")`; 0 passed, 1 failed.
- Repeating those commands after the correction produced `/tmp/ive-task2-fix1-private-i1-green.log` and `/tmp/ive-task2-fix1-public-i1-green.log`: each 1 passed, 0 failed.

The final regressions cover `class.foo`, `class@foo`, `state.foo`, `state@foo`, `start.foo`, `note@foo`, `robust.foo`, and `concise@foo`, all without an earlier declaration and with an explicit participant positive control. Public assertions verify sequence family and rendered endpoint/message text. Private controls preserve actual class/state/timing/note declarations, adjacent dotted/dashed/reverse arrows, `if(...)`, quoted arrow-bearing names, and a leading timing `@42`.

### I2: label arrows cannot suppress implicit state evidence

Verified that the old comparison applied arrow exclusion only before the first label colon, while the typed Tail summary incorrectly kept accumulating arrows from label payload. `Tail::append` now freezes both `last_head` and `arrow` facts before the first real label colon. The existing quote primitive and `::` primitive continue to prevent quoted or namespace colons from becoming label separators. The generated fold and family decision receive the same fixed typed summary; this is a nonallocating flag update, not raw statement reparsing.

Meaningful RED, before the I2 implementation:

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --lib should_freeze_state_header_facts_before_literal_label_payload -- --nocapture
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml should_render_implicit_state_descriptions_with_literal_arrow_text -- --nocapture
```

- `/tmp/ive-task2-fix1-private-i2-red.log`: `Idle : waiting -> ready` had candidate `Pending`, expected `State`; 0 passed, 1 failed.
- `/tmp/ive-task2-fix1-public-i2-red.log`: the forced `state Idle` control succeeded; the identical description without that declaration failed with `UnsupportedFeature("PlantUML at line 2, column 1: \"Idle : waiting -> ready\"")`; 0 passed, 1 failed.
- Repeating those commands after the correction produced `/tmp/ive-task2-fix1-private-i2-green.log` and `/tmp/ive-task2-fix1-public-i2-green.log`: each 1 passed, 0 failed.

Final public regressions preserve literal `->`, `-->`, `<-`, `..>` and `o--` text in implicit state descriptions, check the state family and exact decoded SVG text, and repeat each with an explicit-state positive control. Private controls also cover payload pseudostates, quoted names/labels, `::` before the actual label colon, real transition arrows before the label, and actual class/participant/timing header precedence.

All RED logs remain separate and unchanged. The baseline alone was not used as evidence for either newly exposed case.

### Covering, generation and admission evidence

After the final edits/formatting:

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --lib plantuml::parser::header::tests -- --nocapture
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml --test plantuml_control --test plantuml_constants --test plantuml_timing
python3 -B /tmp/ive-task2-stack-proof.py
```

- `/tmp/ive-task2-fix1-header-covering.log`: all four header tests pass, including source admission and controlled cancellation.
- `/tmp/ive-task2-fix1-public-covering.log`: 36 PlantUML public tests + 23 control tests + 44 constants tests + 21 timing tests pass; no existing assertion was changed.
- Original `/tmp/ive-task2-stack-proof.py` and `/tmp/ive-task2-stack-proof.log` are preserved. New `/tmp/ive-task2-fix1-stack-proof.log` reproduces exactly: document 56 states/31 terminals/123 reachable shift stacks/max seven live states/three consecutive reductions/diagnostic capacity ten and growth peak fifteen; header 55/25/51/max four/four/capacity eight and peak ten.
- `/tmp/ive-task2-fix1-generated-hashes.log` confirms both grammars, generated Rust and `generated/manifest.json` are byte-identical. Document/header generated hashes remain `761b856999aacafae5a744ddaed56720ccf8bd412433d69572cefe20b16bb244` and `094a5bd7bd1d146cb25f0b7b945e0eb01d26db6104d23778d3a9ba7b156ce74c`; inventory hash is `06474b7289807f6daf4eb9013f34a1c7f46f87d3dfbb808c92dfb7b8b52832e3`.

The checked-in admission proof now explains why those exact bounds still apply: no token variant, payload field, generated production, retained structure or allocation path changed; the witness explores every stream over the same terminal alphabets, independently of earlier lexical grouping. Complete names generally reduce token count, still at most one token per source byte. Word scanning still charges/checks every256 bytes (at most259 across UTF-8), with a fixed-size arrow-prefix predicate at each cursor; the label-flag correction allocates nothing. Existing token-capacity, generated stack/reduction/diagnostic and semantic model-item quotas remain intact. No generation was performed or hand editing of generated output attempted.

### Final common gates and public/native checks

`/tmp/ive-task2-fix1-gates.py` ran the exact sixteen common commands listed in the original report after the final edits, with `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` inherited by every Cargo subprocess and locked/offline flags where supported. All sixteen exited 0; per-command logs are `/tmp/ive-task2-fix1-final-<name>.log` using the same names as the original table.

- Workspace tests: **524 ordinary Rust tests** (including 141 internal engine tests and 36 tests in `plantuml.rs`), zero failures, plus all **seven actual worker fault scenarios**.
- Strict workspace Clippy, fmt, release build and cargo-deny license/source checks: pass. Existing vendored Atomic deprecation noise remains unchanged.
- Source audit: 220 MIT source/ZIP hashes, 43 fixtures, 43 native modules. Merman: 656 imported files/four patches unchanged. Node behavior guards: 18 passed.
- Separate generator tests: 3 passed; Clippy/fmt/deny/drift check: pass. Reproduction: three parsers, no tracked writes. Python notice guards: 5 passed; generator closure audit: all 67 exact archives/source/license/feature/notice records pass.
- Immutable baseline: all 43 source/family/SVG equality records pass. File SHA256 remains exactly `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`; no rerecord.

The fixed release binary reran the reviewer's six exact input files (`dotted`, `at`, `dotted-control`, `label-arrow`, `label-control`, `label-arrow-control`) through the documented headless PlantUML export interface. **All six exited 0**; evidence is `/tmp/ive-task2-fix1-public-export.log`, with SVG outputs `/tmp/ive-task2-fix1-header-<probe>.svg`. This independently checks the public release path as well as the new API tests.

Fresh genuine native captures ran via `/tmp/ive-task2-fix1-native.py`, with `--background`, automatic capture/quit and no focus manipulation. `/tmp/ive-task2-fix1-native-summary.log` records styled sequence **20/0**, styled graph **22/0**, constants sequence **12/0** and constants graph **14/0** native/fallback measurements, each exit 0. PNGs are `/tmp/ive-task2-fix1-plantuml-{styled,constant}-{sequence,graph}.png`; all four are byte-identical to the previously inspected review-base frames, and the fresh constants graph was additionally viewed directly. New empty-PATH constants-sequence/styled-graph exports exited 0. Final process-prefix inspection found no release viewer or worker left running.

Self-review confirmed the diff is limited to the two exact subset-preservation fixes, private/public controls and the unchanged-table proof explanation. Existing family/style/Creole migration residuals remain unchanged. No unresolved failure or additional scope request remains; controller-owned scoped re-review is next.
