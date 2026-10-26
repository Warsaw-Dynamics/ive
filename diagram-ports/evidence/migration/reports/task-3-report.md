# Task 3 report — generated preamble/styles/chrome syntax

Status: IMPLEMENTED AND FROZEN, awaiting the controller's fresh independent SPEC and QUALITY reviews. This report is not review approval.

- Base: 92c9520ffbaf9d44e8dc246d66f1ed659e2ddd82.
- Head: b301d7e63187629b086bd0b97e27ec8e7779164d.
- Commit: Replace PlantUML preamble parsing with generated grammar.
- Worktree: /mnt/data/Projects/ive/.worktree/diagram-ports.
- Standalone workspace: diagram-ports/.
- Tracked worktree clean after the scoped 25-file commit. No merge, push, worktree removal, main IVE/state changes, controller progress/ledger edits, fixture changes, or vendor changes.
- Sole product implementer; no subagents or self-dispatched reviewers. Applied TDD, verification-before-completion and systematic-debugging practices. The requesting-code-review handoff goes to the controller's separately arranged reviewers.

## Actual cutover and interfaces

Added parser/preamble.rs and grammar/preamble.lalrpop, with mechanically generated generated/preamble.rs registered in generated/manifest.json. PreambleParser owns the admitted region heads/alignment/optional colon, skinparam command/key/value order, stylesheet/rule/selector braces, property/value/optional semicolon, physical row ends, first-family boundary and complete preamble-envelope EOF. There is no Line* forwarding, raw command passed to old style methods, retry, fallback, recovery or partial rendering.

Removed style::prepare, region/region_command, word, skinparam, sheet, string-key declaration/property syntax and is_preamble; removed Preamble.start and mod::has_identifier_arrow. style.rs now retains the existing Preamble/fonts/colors/regions plus typed Target/PropertyKey/Property validation/application. The generated builder receives typed targets/keys and only their explicitly owned raw label/property literals. Color, size, literal family-name and finite font-style validation remain semantic primitive/value validation. chrome.rs and scene/text/layout/native-cache code are unchanged.

Document's existing shared Ownership scan emits the preamble token buffer alongside its existing temporary family Line adapter. There is no second wrapper/comment normalization pass. UML reuses the already-computed generated header identifier-arrow fact. Other wrapper prefixes obtain that same generated evidence once when needed. New row breakpoints retain exact original byte offsets across removed inline comments; token spans use ordered binary lookup.

preamble::parse(&mut ParsedDocument, &mut Budget) returns the existing Preamble plus the generated first-family physical-row span. It also narrows the borrowed ParsedDocument.body.range to the complete remaining family source, preserving the first physical row's indentation/comment bytes and all remaining CRLF/raw rows. The legacy family Lines are filtered solely by that span's physical row. Preamble syntax/definition checks no longer depend on a normalized Line index.

Document::check_preamble accepts the generated body span. Generated stylesheet actions supply complete stylesheet spans to deny_sheet_definitions. The unchanged eager immutable literal registry/evaluator uses their mapped authored rows to preserve first-family and expanded-stylesheet definition denial. Structured preamble/value/generated failures map once through the origin document before entering any legacy family/layout diagnostic bridge.

Preserved the exact admitted/rejected subset: physical multiline/empty regions reject; escaped \n labels and physical multiline stylesheets remain admitted. Preserved case/alignment/colon policy, defaults, root versus element scope, authored property precedence, region replacement, mixed skinparam/style denial, raw/quoted command lookalikes, YAML literal-key ownership, data's exact lowercase "title " extraction policy and HCL command-named blocks/shared-preamble denial.

## Observed RED and GREEN

All Cargo invocations/subprocesses used CARGO_TARGET_DIR=/mnt/data/Projects/ive/target and locked/offline flags where supported. The environment's known bwrap RTM_NEWADDR initialization failure required scoped escalation; no automatic-review rejection or unresolved access block occurred.

1. Initial new generated contracts, before implementation:
   cargo test --locked --offline -p diagram-engine --lib parser::preamble::tests -- --nocapture
   Observed 0 passed / 2 failed: successful typed region lowering returned the explicit "generated preamble unavailable" stub error; complete-input failure reported 0..0 rather than the expected offending 20..25 span. This output is in the implementation tool transcript, not a fabricated preexisting log. The stub was removed.

2. Region marker regression:
   /tmp/ive-task3-region-boundary-red.log
   cargo test --locked --offline -p diagram-engine --test plantuml_styles should_keep_region_keyword_boundaries -- --nocapture
   Observed failure "accepted non-keyword region title{Caption}". Exact lexical keyword ownership now precedes raw label tails, preserving rejection of title{Caption}/title;Caption and equivalent other-region lookalikes.

3. Semantic action cancellation:
   /tmp/ive-task3-region-cancel-red.log
   cargo test --locked --offline -p diagram-engine --lib should_interrupt_region_validation -- --nocapture
   Observed the long-label action return success instead of typed cancellation before copying. Its eligibility scan now checks/charges each 256 bytes before semantic retention.

4. Remaining BodySource:
   /tmp/ive-task3-body-span-red.log
   cargo test --locked --offline -p diagram-engine --lib should_borrow_the_remaining_family_body -- --nocapture
   Observed the borrowed range still contain "\r\nskinparam defaultFontSize 20\r\ntitle Top\r\n  A -> B : Ω\r\n" instead of the exact physical family remainder "  A -> B : Ω\r\n". The successful facade now narrows the body at the generated physical row boundary.

Final focused GREEN:
- /tmp/ive-task3-parser-green.log: 7 direct generated/lexical/action/body-span tests passed.
- /tmp/ive-task3-styles-green.log: 23 style tests passed.
- /tmp/ive-task3-parser-all-green.log: 35 then-current shared parser tests passed before the final additional BodySource test. Final complete workspace rerun includes all 36 shared parser tests.
- Existing constants44/control23 and all previous style/model/layout assertions remained controls, not claimed new RED.

New coverage includes typed generated region/body results, whole-envelope EOF, exact two-model-item selector/property accounting, tiny-work admission, generated delivery/reductions at seven scheduled checkpoint positions, lexical cancellation before a Value allocation, semantic-label cancellation before region retention, BOM/CRLF/expanded values, exact inline-comment/UTF-8 expansion origins, case/colon/aligned regions, physical multiline-region negatives, 6/14/96 finite fonts, every region's name/color/size/style skinparam axes, all finite plain/normal/bold/italic forms, all late region markers and supported quoted/raw lookalikes.

An intermediate new quoted-participant control used "<style>", which correctly triggered the existing unsupported rich-text label policy. The fixture was corrected to supported quoted "header Normal"; no product behavior was relaxed. An intermediate compilation error while tightening combined buffer admission exposed preprocess.rs's production use of document::validate; the approved public-internal validator remains intact, delegating with zero additional authored-preamble retention. Mechanical initial grammar generation also caught unnamed/named symbol mixing; corrected grammar generated without conflicts. None is presented as meaningful feature RED, and no failing existing tests remain.

## Actual generation/runtime/allocation evidence

The checked-in output is genuinely LALRPOP 0.23.2, no-default-features, separately locked/audited development generation. No dependency/lock change. Ordinary locked/offline builds and empty-PATH exports need no generator.

- Grammar SHA256: d133584bcd6941016ed0d055bf07e29ec919fb74ef44ff9c15522c7e9b59396b.
- Generated SHA256: a16dc15721bda29480c780df9678560765de6edcd4f550c0a984678674bff879.
- Actual stack witness: /tmp/ive-task3-stack-proof.py.
- Observed actual tables: 77 states, 23 terminals, 111 reachable shift stacks, maximum12 live states/11 symbols, maximum3 consecutive reductions, maximum12 live diagnostic states/capacity, maximum9 diagnostic growth-peak state bytes, maximum10 expected terminals.

The witness reads actual ACTION/EOF/reduction tables and matches the emitted goto function. It and the cached exact runtime initialization/growth/error paths were inspected before parser entry admission was enabled, then rerun after final mechanical regeneration. Prior document/header witness /tmp/ive-task2-stack-proof.py independently reruns unchanged56/55 states,123/51 shift stacks,7/4 live states and3/4 reduction maxima.

Full proof is committed in provenance/plantuml-parser-admission.md. Before generated allocation, preamble admission checks128 stack/diagnostic slots, actual token capacity, actual literal capacities, actual simultaneous Line-vector capacity plus conservative twice-source string allowance, generated symbol growth/transients, copied states and expected-string vector/content transients. It reserves four work units per token, three EOF reductions,128 storage units and23×(12+3) diagnostic simulation units. Document admission additionally includes the still-live preamble buffer/adapter allowance; earlier authored validation passes zero extra preamble bytes.

There is no recursive syntax AST/declaration vector or recovery accumulation. Strings move from lexer tokens into generated symbols/actions. Source-derived checked token/literal storage remains separate from semantic item/text counters. Direct generated root+property parsing succeeds at exactly model_items2 and rejects1. Existing 60,000-row/500ms, below-cap action-work and100,000-work state controls were kept unchanged and pass.

Actual lexical scans/delivery/action tests exercise cancellation; Checked checks every token and attempted EOF and preserves Cancelled terminals. Raw literal and label scans checkpoint in256-byte chunks, finite primitive recognition is allocation-free/bounded, and name/color/size/style validation avoids an unbounded lowercase value copy. No worker deadline is used as parser admission proof.

## Final complete gates

The final full run occurred after the last product/provenance edit. Runner /tmp/ive-task3-gates.py; exact complete outputs /tmp/ive-task3-gates/01.log through16.log. All exit0:

| Log | Command/result |
| --- | --- |
| 01 | cargo test --locked --offline --workspace — 537 ordinary Rust tests, seven actual worker faults |
| 02 | cargo clippy --locked --offline --all-targets -- -D warnings |
| 03 | cargo fmt --all --check |
| 04 | cargo build --locked --offline --release |
| 05 | cargo deny --locked --offline --config ../deny.toml check licenses sources |
| 06 | node tools/audit-plantuml.mjs —220 authorized MIT source/ZIP hashes,43 fixture hashes,43 native modules |
| 07 | node tools/audit-merman.mjs —656 files, four recorded source patches |
| 08 | node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs tools/parser-baseline.test.mjs —18 tests |
| 09 | cargo test --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml —3 tests |
| 10 | cargo clippy --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all-targets -- -D warnings |
| 11 | cargo fmt --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all --check |
| 12 | cargo deny --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --config ../deny.toml check licenses sources |
| 13 | cargo run --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml -- check —4 reproducible parsers, no tracked writes |
| 14 | python3 -B tools/audit-plantuml-generator.test.py —5 behavior/notice tests |
| 15 | python3 -B tools/audit-plantuml-generator.py check —67 exact generator dependency archives/source/licenses/features/notice coverage |
| 16 | node tools/parser-baseline.mjs check —43 immutable source/family/SVG equalities |

Updated real parser/grammar/output registrations and original authorized MIT destination traces. Kept all source hashes, exact notices/credits, generator dependency closure and all fixture records intact. Prior definition/document/header grammar/generated files are byte-identical. Baseline file SHA256 remains a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe. No rerecording; the three source/family/SVG assertions in the baseline driver are unchanged. No Java/browser/original oracle rerun.

The only observed warning noise is the previously assigned vendor Atomic::fetch_update deprecation; vendor sources remain untouched and strict workspace/generator Clippy gates pass.

## Genuine native verification and cleanup

Final release binary rerun:
- Harness /tmp/ive-task3-native.py.
- Summary /tmp/ive-task3-native-summary.log.
- Every launch used --background and --capture; existing viewer background contract opens with focus:false, and capture autquits. No window activation, mouse/input automation, main IVE instance or saved-state edits.
- styled sequence:20 native /0 fallback, PNG224962 bytes.
- styled graph:22 native /0 fallback, PNG195213 bytes.
- constant sequence:12 native /0 fallback, PNG217879 bytes.
- constant graph:14 native /0 fallback, PNG208165 bytes.
- Additional actual font6/14/96 captures: each8 native /0 fallback; PNG135229/140387/147029 bytes.
- Final capture PIDs3728741,3728834,3728899,3729018,3729084,3729156,3729221 exited0. Harness /proc check found no viewer/worker survivors.
- Empty-PATH final binary exports of constantsequence and styledgraph both exit0, saved under /tmp/ive-task3-empty-path-*.svg.

Real GPUI image artifacts:
- /tmp/ive-task3-plantuml-styled-sequence.png
- /tmp/ive-task3-plantuml-styled-graph.png
- /tmp/ive-task3-plantuml-constant-sequence.png
- /tmp/ive-task3-plantuml-constant-graph.png
- /tmp/ive-task3-font-6.png
- /tmp/ive-task3-font-14.png
- /tmp/ive-task3-font-96.png

Inspected styled sequence/graph and font6/96 real frames: expected effective fonts, hierarchy, chrome containment and separation, nested graph/frame geometry, raw-note spacing and styled paint are intact. Existing native measure/paint/containment tests including6/14/96 cases and all three native cache-lifetime tests remain unchanged and pass in the final workspace run.

## Honest residuals and review handoff

No known new product blocker after self-review. Fresh independent SPEC and QUALITY review is still required and arranged only by the controller.

Family command parsers remain the assigned temporary Lines implementations pending Tasks4–15. Creole nesting remains Task16. Legacy family/layout diagnostic bridging remains until those cutovers. YAML's existing unquoted-colon lexical eligibility primitive is intentionally reused pending its family migration; this task does not claim a YAML structural cutover. Font/color/quote primitive decoding and typed semantic validation intentionally remain handwritten.

Physical multiline document regions, unsupported stylesheet selectors/properties, general CSS, broader skinparams and HCL shared preambles remain rejected. No incidental supported-syntax expansion is intended. The later-family boundary Minors and Task17 baseline-default reporting/vendor-noise triage remain with their assigned tasks.


## FIX ROUND 1 — both Important findings repaired

Status: IMPLEMENTED, COMMITTED AND FROZEN for the controller's scoped independent re-review. This section supersedes the initial implementation's readiness assessment for the two findings in /tmp/ive-plantuml-lalrpop-preamble-review.md.

- FIX_BASE: b301d7e63187629b086bd0b97e27ec8e7779164d.
- FIX_HEAD: f5a10a05df1cc03a5501c0235620cbac49592b3e.
- Commit: Fix empty PlantUML values and linear preamble origin checks.
- Four tracked files changed: parser/preamble.rs, origin.rs, engine/tests/plantuml_styles.rs, provenance/plantuml-parser-admission.md.
- Self-reviewed and tracked-clean after commit. No subagent, self-dispatched reviewer, root progress/ledger edit, unrelated family change, limit increase, merge, push or worktree deletion.

### Important 1: whitespace-only values

Cause: trimming the tail's start independently of the complete row's end could produce start>end before the raw-value slice. The lexer now bounds that end to at least the trimmed start. An empty literal therefore reaches the existing generated/semantic typed rejection path, with a valid zero-width span. No panic catching or exception control flow was introduced; the generated grammar/admission and rejection policy remain intact.

Meaningful focused RED:
- Command: cargo test --locked --offline -p diagram-engine --test plantuml_styles should_return_typed_errors_for_whitespace -- --nocapture
- /tmp/ive-task3-fix1-empty-red.log: 0passed/1failed, production panic "byte range starts at6 but ends at5" in preamble.rs.

Focused GREEN:
- Same command, /tmp/ive-task3-fix1-empty-green.log:1passed.
- cargo test --locked --offline -p diagram-engine --lib should_emit_empty_literals -- --nocapture
- /tmp/ive-task3-fix1-lex-empty-green.log:1passed.

The public matrix covers no whitespace, spaces, tabs, Unicode em-space and mixed whitespace/CR; all four regions, aligned/colon region variants, default size/name/color, region style, background and indented stylesheet font/background properties return InvalidSource or UnsupportedFeature. The private lexer control verifies genuinely empty Value tokens and ordered zero-width spans. The private control is GREEN evidence; the public matrix supplied the observed RED.

### Important 2: repeated-sheet origin work

Cause: both boundary mappings for every stylesheet rescanned/charged the entire document, and every stylesheet scanned every definition again. A binary segment search alone would still repeatedly count newlines in merged direct segments.

Repair: origin::PreambleCursor borrows the existing document and advances one segment cursor, expanded-byte cursor, authored-row counter and definition cursor across all ordered stylesheet boundaries and the final first-family boundary. It scans only previously unvisited direct-byte prefixes; expansion rows come from their existing use spans. It charges/checks every actual segment/definition visit and direct-byte chunks of at most256 bytes. It allocates no index/vector/retained sheet list. Total origin-validation work is O(source bytes + segments + definitions + sheets), including constant boundary peeks.

The same cursor preserves first-family definition denial and definitions inside expanded stylesheets. Unrelated definitions before or between sheets are consumed once; an inside-sheet definition is still rejected at its authored row. The new direct/expanded BOM/CRLF/UTF-8 mapping control and later expanded-sheet negative check pass. No denial was bypassed.

Meaningful focused RED:
- cargo test --locked --offline -p diagram-engine --test plantuml_styles should_keep_repeated_stylesheets_linear -- --nocapture
- /tmp/ive-task3-fix1-sheets-red.log: default-budget failure at5,013,811work against5,000,000.
- cargo test --locked --offline -p diagram-engine --lib should_validate_repeated_sheet_origins -- --nocapture
- /tmp/ive-task3-fix1-origin-red.log: origin-only failure at116,974work against100,000, with200 unrelated definitions.

Focused GREEN:
- Same repeated-sheet command, /tmp/ive-task3-fix1-sheets-green.log:1passed;100/500 repeated sheets use246,207/1,222,703 total public-render work without definitions,319,622/1,346,006 with200 unrelated definitions. Defaults unchanged. The500-sheet public sources have19,028/24,318bytes respectively.
- Same private origin command, /tmp/ive-task3-fix1-origin-green.log:1passed.
- cargo test --locked --offline -p diagram-engine --lib origin::tests -- --nocapture
- /tmp/ive-task3-fix1-origin-controls.log:4passed. The500-sheet/200-definition private pass uses20,899work for19,228expanded bytes,201segments and200definitions; all200 definitions advance once. Ordered authored-row controls, cancellation during direct-byte scanning and definition advancement, and a two-unit work failure pass.
- cargo test --locked --offline -p diagram-engine --test plantuml_styles --test plantuml_constants --test plantuml_control -- --nocapture
- /tmp/ive-task3-fix1-covering.log:26styles +44constants +23control tests passed, retaining all existing assertions. The final complete workspace run repeats these after the last implementation/proof edits.

### Renewed actual admission/storage/cancellation proof

No grammar, generated file, token variant or generated state/symbol payload changed. The actual witness /tmp/ive-task3-stack-proof.py was rerun against generated SHA256 a16dc15721bda29480c780df9678560765de6edcd4f550c0a984678674bff879 and again reports77states,23terminals,111reachable shift stacks,max12live states,max3consecutive reductions,max12diagnostic capacity,max9diagnostic growth-peak bytes and max10expected terminals. The actual goto mapping remains unchanged.

PreambleCursor is one borrowed Document reference plus four machine words:40bytes on this64-bit target. It replaces the builder's former8-byte Document reference. Before builder construction/parser entry, actual sizeof(PreambleCursor) is conservatively added in full to the retained-byte admission, and its five machine-word initialization units are charged. No model item/text quota is spent. The generated128-slot/state/symbol/expected-string/transient reservations remain unchanged. The proof amendment is committed in provenance/plantuml-parser-admission.md; no copied reduction constant or uncharged input-sized retention was added.

The new cursor checks cancellation every segment/definition visit and every256 direct bytes. Existing generated delivery/reduction/EOF, lexical/raw-value, tiny source/work/model/depth/output, once-only origin and expanded-sheet definition controls remain unchanged and pass. The final helper extraction keeps the new cursor within the repository's nesting guideline without changing its accounting.

### Exact reviewer release probes

Harness /tmp/ive-task3-fix1-review-probes.py, complete output /tmp/ive-task3-fix1-review-probes.log. Uses the original /tmp/ive-task3-review-*.puml inputs and the final release exporter, without modifying the reviewer inputs:

- Empty-region no-spaces control: exit1, typed "multiline/empty document region", line2column6.
- Empty-region spaces: exit1, same typed error, line2column9.
- Empty-skinparam spaces: exit1, typed finite-size error, line2column29.
- Empty-style-property spaces: exit1, typed finite-size error, line4column12.
- None panics or reports an abnormal worker exit.
- Exact1/100/500-sheet inputs, including the literal space in FontSize 20: all exit0.
- Their successful SVGs are byte-identical, SHA256 e290420a78de0b23539d99b01b4951c19681839c1479ea954665dba42c3f4594. The reviewer's500-sheet source retains its original19,029bytes.

### Final complete gates, provenance and native checks

Final runner /tmp/ive-task3-fix1-gates.py executed all sixteen exact commands listed in the original "Final complete gates" table above, with the same flags, after the last edit. New complete outputs are /tmp/ive-task3-fix1-gates/01.log through16.log; every command exits0:

- 01:544ordinary Rust tests +7actual worker faults.
- 02/03/04: strict all-target Clippy, formatting and locked/offline release build.
- 05: workspace license/source gate.
- 06:220authorized MIT source/ZIP hashes,43fixtures,43native modules.
- 07:656Merman files/four recorded patches.
- 08:18Node behavior guards.
- 09/10/11/12: generator3tests, strict Clippy, format and separate lock/feature license/source gate.
- 13:4reproducible parsers, drift check writes no tracked output.
- 14:5generator-notice behavior tests.
- 15:67exact generator dependency archive/source/license/feature/notice coverage.
- 16: all43immutable source/family/SVG equalities.

All Cargo commands and subprocesses used CARGO_TARGET_DIR=/mnt/data/Projects/ive/target, locked/offline where supported. Fixture and baseline files, source hashes/notices, vendor files, all generated parsers/inventory, public APIs, native/font/cache/layout code and resource limits are untouched. Baseline SHA256 remains a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe; no rerecording. Known inherited vendor Atomic deprecation noise remains assigned elsewhere; strict gates pass.

Final native harness /tmp/ive-task3-fix1-native.py, summary /tmp/ive-task3-fix1-native-summary.log. Seven background real-frame/autquit captures pass: styled/constants native counts20/22/12/14 and font6/14/96 each8, every fallback0. Both empty-PATH exports pass. Final captured viewer PIDs3877690,3877835,3877968,3878079,3878175,3878268,3878421 exit0; /proc cleanup check reports no viewer/worker survivors. Artifacts use /tmp/ive-task3-fix1-*.png/.log/.svg, preserving initial-round artifacts. Inspected the fix-round styled graph and96px native frames; expected fonts/chrome/containment remain intact. Existing6/14/96 paint/measurement/cache assertions pass unchanged in the complete workspace run. No focus/input automation or main IVE/state changes.

Both reviewed Important causes are addressed with observed RED/GREEN and direct release reproduction checks. No known new blocker after self-review. Scoped independent re-review remains the controller's responsibility; no approval is claimed. All previously disclosed family/Creole/Task17 residuals remain assigned and unchanged.
