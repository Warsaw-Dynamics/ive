# Task 3D2 implementation report

Status: implemented; independent SPEC and QUALITY review remain the controller's next gate. This is a bounded native literal-constant subset, not full PlantUML/TIM parity.

- Released base: `71cd81767477c11fcc28c97c0498c69a49f98bcf`.
- Frozen head: `dad6fb70bcd76569de7877ecdac41b63406ee719` (`Add bounded eager PlantUML literal constants`). Tracked worktree clean after commit; this ignored report is the external handoff artifact.
- Worktree/branch: `/mnt/data/Projects/ive/.worktree/diagram-ports`, `diagram-ports`.
- Scope: 16 files under the standalone `diagram-ports/` product only. No root IVE edits, dependencies, runtime engine services, assets, merge, push, helpers, or later-family work.
- Binding requirements: [brief](task-3d2-brief.md), including the controller's diagnostic-granularity clarification; [global constraints](native-global-constraints.md). Unchanged style/native-text/family contracts and outstanding parity gaps remain those in [Task 3D report, including FIX ROUND 1](task-3d-report.md).

## Contracts and implementation

`preprocess::prepare(source, &mut Budget) -> Document` runs on the raw authored source before existing wrapper/source/style/family preparation. Definitions are eagerly expanded against already frozen values; subsequent scans append stored values without rescanning them. One shared Budget and one native text session remain in force. Existing family models are unchanged. `text::Context` receives a borrowed origin Document; its measured-run/cache keys and aggregate body/chrome counter remain intact.

`origin::Document` owns expanded text, compact output `Segment`s, and definitions. `Origin` is `Direct(SourceSpan)` or `Expansion { use_span, definition_id }`. A SourceSpan contains absolute UTF-8 byte start/end and one-based physical line/UTF-8 **byte** column. Definitions retain name, frozen value, authored span, prior definition IDs, and bounded depth; no copied provenance trees. Adjacent direct spans coalesce, including across physical lines. Blank definition rows retain their original line terminator, preserving parser physical lines. BOM, LF/CRLF, raw spaces and quoted spelling survive until the existing family-specific preparation owns normalization.

Admitted grammar and deliberate restrictions:

- Exact lowercase `!define NAME literal`, inside a single admitted wrapper and before the first family statement. NAME is `[A-Za-z_][A-Za-z0-9_]{0,63}`. Literal is nonempty, same-line, ASCII-space/tab trimmed; authored quote characters remain part of the value. Definitions can accompany the admitted prefix styles/chrome but cannot occur inside authored or expanded stylesheets.
- Names are case-sensitive and immutable. Duplicate/redefinition/shadowing fail. Unknown names and self-looking names remain literal. `FIRST SECOND` then `SECOND Ready` yields `SECOND | Ready`; `A B` then `B A` yields `B | B`. This is neither recursive replacement nor lazy dependency resolution.
- ASCII maximal-name boundaries, quoted-token replacement, and the original backslash-`n` boundary exception are covered by exact controls. Ambiguous non-ASCII adjacency to a **known** constant is rejected explicitly. Unrelated Unicode text/data remains admitted. This narrow policy does not claim Java Unicode/UTF-16 boundary equivalence.
- Literal values cannot contain control characters other than tab, U+FFFE/U+FFFF, or an odd trailing backslash continuation. No physical statement newlines are synthesized. Each authored and frozen literal is at most 16 KiB.
- Parameterized defines, definelong, functions/procedures, full TIM, includes/themes, loops/conditionals, variable/builtin calls, environment/date/file/network evaluation remain denied. `$name(...)` and `%name(...)` capability forms are rejected, including within quote/data positions. Plain `$5`, `$other`, `%other`, URL/path text and inert directive-looking label data remain literal; `$X` can expand the known token X while preserving `$`.
- `##` directly adjoining a matched constant is rejected as concatenation. Inert hashes and tree `##` markers remain data. Inserted hash text is not rescanned. Generated leading directives/wrappers are rejected; all expanded input then passes existing complete wrapper/source/style/family and resource admission, so generated resources/unsupported styles gain no privilege.
- Comment classification follows the existing source rules without globally rewriting retained source. Actual inline comments do not turn inert `%getenv` text into a capability request. Shared raw note/action body ownership preserves apostrophe-looking rows, significant whitespace and blank rows. Raw Git/wire/EBNF/JSON/YAML regression controls remain passing; JSON/YAML retain their preceding prefix/keyed-payload rules.

## Bounds, cancellation, and diagnostics

All retained expanded-source bytes **plus all frozen definition values** are charged against the configured source-byte limit before appending/copying. Definition names/model text also use the existing text budget; definitions consume model items. Origin segments plus retained prior IDs are capped at a checked `4 * model_items`. Lineage depth is checked before copying and is at most `min(32, nesting_depth)`. This is a lineage bound, not recursive evaluation.

Physical input is precharged in chunks of at most 512 bytes before classification allocations; scans/lookups/copies and diagnostic searches use the same work/cancellation budget. No per-stage budget or session reset was introduced. Existing 4096 aggregate native-measurement requests, 16 KiB label, SVG/scene and worker limits remain tested.

Seven private tests exercise actual internal state at failure, not just error strings: exact BOM/CRLF spans; retained limit 121 with a frozen 50-byte value and output at most 71 bytes; depth 2 before third-definition copying; origin quota 4 before the fifth segment; tiny work/cancellation before retention; model quota before the second definition and 16385-byte authored/frozen rejection; depth 33 rejected at 32 with no new value/prior copy. A 50000-blank-row public control verifies compact direct origins.

Known expansion use spans and definition spans/IDs are retained exactly. Existing parser errors keep their authoritative physical statement line and existing column 1; `Document::map_error` appends the expansion uses on that failing line as context, not an invented causal token. It reports at most eight uses and bounded immediate lineage with explicit omitted counts.

Late Creole errors lack source-bearing family label identity. `Document::label_error` labels their original location **label-local**, searches complete label text for bounded textual candidates, and associates candidate expansion use/definition IDs or direct-source spans. Repeated occurrences are explicitly ambiguous; at most eight occurrences/eight expansion uses per candidate are shown, with exact omitted counts. Unique, repeated-identical, inert duplicate and truncation controls pass. These are textual candidates, not a proof of family-label ownership. Existing YAML nested-flow/quote diagnostics and some legacy unterminated-style diagnostics still use line 1; this release does not claim all pre-existing diagnostics are precise.

## Translation provenance

Personally inspected actual authorized sources only under `/mnt/data/Projects/plantuml-mit/src`, including their actual MIT headers. Reused the fourteen independently observed original probes in `/tmp/ive-plantuml-style-semantics.md`; did not run or claim new JVM probes. Only scalar literal-definition/storage/token-scanning behavior was translated, not adjacent function/evaluator/capability branches.

| MIT source | Inspected/translated behavior | SHA-256 |
| --- | --- | --- |
| `tim/EaterAffectationDefine.java` | 120–133, same-line literal assignment and eager prior-value expansion; destination `preprocess.rs` | `001009a5bd744874e72233f839523eea054c07459621a137e68ab7062afb84db` |
| `tim/TContext.java` | 435–525, only scalar scan/substitution branches and no inserted-text rescan; destination `preprocess.rs` | `b914f5053b91d59c2b9578f663d531823e59f34615657fadd3562a11f529fc5d` |
| `tim/VariableManager.java` | 131–155, 216–236, name boundaries/backslash-n exception; destination `preprocess.rs` | `a9ec4da9f0ed946f97e81daa54b4a67a747b39561739c170a827237166476a36` |
| `text/TLineType.java` (previously admitted) | 132–147, 240–247, 344–365, directive classification mapping extended to bounded literal stage | `e2d8f4d06627210fc6148df329bc9d18663b53c5647927f4e456271431eaac84` |

Exact selected-file credits are `(C) Copyright 2009-2024, Arnaud Roques` and `Original Author:  Arnaud Roques`; the existing complete MIT grant remains in THIRD-PARTY-NOTICES. Provenance records precise destinations/behavior/intentional policy differences. The audit allowlist adds only these three exact tim paths, with negative tests for TMemoryGlobal, FunctionsSet and EaterLegacyDefine. No rejected candidate, GPL/EPL/Smetana/untraced source or asset was imported. The local generated source artifact SHA-256 is `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`; this remains a local identity claim, not authenticated Maven provenance. Controller independently matched new source/JAR/hash entries and credit headers.

## TDD and verification evidence

Focused development used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml_constants`, private preprocessor tests, and focused viewer tests. Test names use `should_` and given/when/then sections.

- Initial ten public controls: meaningful RED, nine failed at existing `!define` denial and one negative control passed; initial eager/origin implementation made ten GREEN.
- Compact origins, expanded-label diagnostic and stylesheet ownership: RED 10/13; fixes made 13/13 GREEN.
- Repeated/inert late-label candidates, omitted counts and unused controls: RED 13/16; fixes made 16/16 GREEN. Exact BOM/CRLF private span assertion also failed before preserving complete direct wrapper spans.
- Comment capability classification failed before correction. Uppercase UML wrapper and inert inline comment controls were the last two RED at 18/20; both became GREEN.
- Raw note/action apostrophe row RED returned `' X` instead of `' Ready`; sharing raw-body ownership made 21/21 GREEN. Generated stylesheet opening also produced a meaningful RED before post-expansion prefix ownership validation; direct/generated/generated-with-inline-comment variants now pass.
- An initially proposed rejection of generated `##` was corrected during self-review: no-rescan semantics require inserted hashes to stay literal. The replacement test covers actual adjoining-constant concatenation versus inert hashes/tree markers and failed before narrowing that check. No contrary semantic behavior remains.
- Seven private bound tests pass; not every private assertion had an independent RED cycle. Viewer selection had an initial test-module compile-placement error, then a proper unknown-example RED, then GREEN after registration. Strict Clippy's nine local style diagnostics were corrected before the final full gates.

Final command evidence is in `/tmp/ive-plantuml-3d2.R06efT/`. Commands ran from the standalone product with the common target directory; sandbox escalation only bypassed the known bwrap RTM_NEWADDR startup failure.

```sh
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline --workspace
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo build --locked --offline --release
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo deny --config ../deny.toml check licenses sources
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
git diff --check
```

All exited 0 after the last product edit. Workspace: **299 Rust tests passed, zero failed/ignored**, plus seven hostile-worker scenarios (blocked stdin/stdout, stderr flood, oversized frame, early exit, memory and cancellation). Suite counts: engine-private 64; compatibility 11; Mermaid families 37; Mermaid 21; PlantUML 33; constants 21; control 23; styles 17; tree/data 18; viewer-private 40; export 5; native lifetime 3; worker 6. The prior 91 public PlantUML tests remain passing. Audit tests: 9/9. Source audits: PlantUML 113 selected source/JAR files, 15 fixtures, 26 modules; Merman 656 imported files/four patches. Cargo deny: `licenses ok, sources ok`.

Logs: `tests.log`, `clippy.log`, `release.log`, `deny.log`; dedicated `native-geometry.log` records the actual native styled-paint regression. Expanded sequence/class each painted eleven measured runs with Liberation Sans resolved to Liberation Sans. Existing frame kind/caption clearance, endpoint alignment, significant-space, style-run containment and activation pixel tests remain passing. Native-worker lifetime tests include both new fixtures and retain cache-release/query assertions.

Warning: inherited vendored `merman-render` `environment.rs:585` Atomic::fetch_update deprecation persists; strict product Clippy passes, and no vendor code was changed. Self-review after full gates found no further product changes.

## Actual background viewer captures

Used the actual release native viewer with `--background --language plantuml --capture`, without focus changes or xdotool. The registered examples are `plantuml-constant-sequence` and `plantuml-constant-graph`. Capture files/logs are in `/tmp/ive-plantuml-3d2.R06efT/`; each PNG is 1360×880. Implementer and controller both inspected all three real images.

| Capture | Actual result | SHA-256 |
| --- | --- | --- |
| `constant-sequence.png` | exit 0; 12 native measurements, 0 deterministic fallbacks | `902e7f81556a73f29e14440fa2b5b3d931c6c201ac61f43d4a1511e843aa04c5` |
| `constant-graph.png` | exit 0; 14 native measurements, 0 deterministic fallbacks | `e55f74a7bbbaae8bc0ac3f1fea6054f715cd8f55d4af649045dc17d746f399de` |
| `constant-error.png` | expected exit 1; visible error frame produced | `05914f0732f156f0c1f92e29ba8f382be262caeec82d34203f97a621f2ceeca4` |

Sequence: visible eager `SECOND | Ready`, expanded title/group/note/chrome, blank note row and significant spaces; group kind/caption and activation/note geometry clear. Class: expanded title/package/class/member/relation labels, separate `1`/`many` endpoints, chrome and containing boxes clear. No observed overlap/containment warnings.

The error input defines SIZE as `enormous`, then uses it in `  skinparam defaultFontSize SIZE`. The actual frame/log retains parser line 3, column 1 and reports definition #0, authored use line 3, column 29, definition line 2, column 1. No host-count metadata is emitted on this failure; it occurs before native-session creation, so no fabricated measured zero is claimed.

All three viewer processes self-terminated. A final anchored `pgrep` for the actual debug/release diagram-viewer paths found none. No IVE instance was started, no user process was stopped, and no capture process remains.

## Residual scope and handoff

Full TIM, parameterized macros, includes/resources, arbitrary evaluator capabilities and complete Unicode boundary parity remain intentionally unsupported. Late-label diagnostics are bounded textual candidates, not exact family ownership; old YAML/style diagnostic granularity is not repaired globally. Existing native-font fallback, baseline, original-engine layout/pixel parity and later-family gaps remain in the prior report. No known new product issue was found during self-review. Independent frozen-head SPEC/QUALITY review remains required; this report is not a claim of full-port completion.

## FIX ROUND 1 — comment/wrapper and raw-action ownership

Fix base: `dad6fb70bcd76569de7877ecdac41b63406ee719`. Fix head: `50da37e108256003388361c0b5ac5a3da76735ff` (`Fix PlantUML constant preprocessing raw row ownership`); tracked worktree clean. Full independent review read: `/tmp/ive-plantuml-constants-review.md` (two Important findings; SPEC issues / QUALITY needs fixes). Both counterexamples were reproduced before changing production code. The original self-review missed these classifier cases; its initial no-known-issue statement above is historical, not a dismissal of the subsequent findings.

R1: preprocessing now classifies wrapper transitions from comment-normalized syntax, only outside raw note/action ownership. Exact recognized end markers replace the broad raw `@end` prefix branch. Raw-data payload bypasses comment normalization; reserved exact end rows remain subject to existing complete wrapper admission. Wrapper-looking raw note/action label text goes through literal expansion without becoming a wrapper transition or triggering a false generated-wrapper rejection. The strict generated-wrapper/directive checks still apply at syntax positions; resource admission still applies to raw labels. Stylesheet marker tracking also respects literal ownership.

R2: extracted the existing raw-row transition into `source::TextBody::consume_raw(raw) -> Option<TextBody>`, shared by source preparation and preprocessing. A returned state means this row is owned by raw text, including an action's opening `:` row, before comment normalization. The existing opening/continuation/`end note`/trailing-semicolon rules and trimming behavior are preserved. Callers retain their existing inside-wrapper, graph-grammar, block-comment and raw-data eligibility guards. This replaces duplicated ordering rather than adding a third classifier; no family model, source origin, budget/storage or native paint interface changes were needed.

Only three product files changed: `engine/src/plantuml/preprocess.rs`, `engine/src/plantuml/source.rs`, `engine/tests/plantuml_constants.rs`. No new translated source, provenance entry, dependency, family or runtime capability. Existing compact-origin and before-allocation/work/cancel tests remain intact.

TDD command: `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml_constants`.

- Added four exact reviewed reproductions plus two neighboring control tests. Initial RED: 22 passed / 5 failed (exit 101): block-commented end marker and inline-commented opening incorrectly rejected definitions; raw note retained `@enduml X`; action opening suppressed `' X` expansion; neighboring raw note/action ownership control also failed. The data/generated-admission negative control passed from the start.
- R1-only iteration: 25 passed / 2 failed (exit 101), with all three wrapper reproductions GREEN and only the action-opening/continuation failures remaining.
- R2 shared-ownership iteration: 27 passed / 0 failed (exit 0). Controls include raw start/end-looking note rows, action end-looking text, single-line and multiline actions with trailing whitespace after `;`, comment blocks after completed actions, literal YAML wrapper-looking data, generated start/end denial, raw-label resource denial and definitions outside a closed wrapper.

Fresh full verification after the last product edit used the same commands listed above, with logs at `/tmp/ive-plantuml-3d2-fix1.hiErUE/{tests,clippy,release}.log`. `cargo test --locked --offline --workspace`: **305 Rust tests passed**, zero failed/ignored, plus all seven hostile-worker scenarios. Constants grew from 21 to 27; all other suite counts are unchanged. Strict Clippy, fmt check, release build, cargo deny, both source audits, nine audit tests and `git diff --check` all exited 0. Audit inventories remain 113/15/26 PlantUML and 656/four Merman. The inherited vendored Atomic::fetch_update warning remains; no new warnings. Self-review found no further product edit after these gates.

The fresh release viewer successfully exported all four unchanged reviewer inputs with `--render /tmp/ive-constants-review-{comment-wrapper,inline-wrapper,raw-wrapper,action-comment}.puml --output /tmp/ive-plantuml-3d2-fix1.hiErUE/NAME.svg` (four exit-0 results). SVG text and native captures now contain the expected expanded labels.

Actual background native captures, inspected by implementer and controller:

- `raw-note.png`: visible `@enduml Ready` inside the note; 2 actual native measurements, 0 fallback; exit 0; SHA-256 `7f83c1d38c6f8f9a80415404b98a2fe057da32bd3bc0cd487460b21eafae9d2d`.
- `raw-action.png`: three visible action rows `opening /' literal`, `' Ready`, `ending`; 3 actual native measurements, 0 fallback; exit 0; SHA-256 `e733cc3cc52abcdf5b00fe30ceb742d6232abad4e25b983181d335ef3cc9c3ce`.
- Both are 1360×880 with contained text and clear connectors. Commands used `--background --language plantuml INPUT --capture PNG`; log files are `capture-note.log` and `capture-action.log` in the fix evidence directory.
- Also recaptured the unchanged registered sequence/class examples and visible SIZE error with the fresh release binary. All three PNG SHA-256 values **exactly match** the initial report above, proving unchanged pixels rather than assuming paint equivalence. Sequence/class again report 12/14 native measurements and zero fallback; expected error exit 1 retains the same line 3/column 29 use and line 2 definition. Fix-directory `capture-{sequence,graph,error}.log` records these results.
- All five viewers self-terminated; final anchored process lookup returned no diagram-viewer process. No IVE instance, xdotool or user-process interference.

Residuals are unchanged: intentional bounded literal/TIM/Unicode subset and previously documented late-label/YAML/style diagnostic granularity. Fresh scoped independent re-review is required before proceeding to the held next task.

## FIX ROUND 2 — expansion-aware next-row ownership

Fix base: `50da37e108256003388361c0b5ac5a3da76735ff`. Fix head: `6abc18fb22d908e2c257267850bb387341c7ea91` (`Reconcile PlantUML raw ownership after literal expansion`); tracked worktree clean. Read full scoped review `/tmp/ive-plantuml-constants-rereview-1.md`: original R1/R2 were addressed, but fix round 1 introduced one Important stale-ownership/generated-wrapper bypass. No other finding was reopened.

The current row still obtains raw ownership from its incoming body state/opening spelling **before** comment normalization. Preprocessing no longer updates the next body state from that unexpanded row. After eager literal expansion it precharges the additional scan against the same Budget and applies the existing shared `TextBody::consume_raw` to the expanded row consumed by final source parsing. Thus generated semicolons/end-note rows close ownership; an authored end-note changed into literal text keeps ownership. Generated wrapper denial applies to the following syntax row, while genuinely raw `@enduml X` label text retains expansion. No inserted value is rescanned for constants. No new classifier, family syntax, origin model, allocation policy, capability, dependency or translated source was added.

Only `engine/src/plantuml/preprocess.rs` and `engine/tests/plantuml_constants.rs` changed. Four new public tests use literal hand-derived expectations and the existing real render path:

- `should_reject_generated_wrapper_after_expanded_action_terminator`: exact reviewed FIN/END input without authored end must reject the generated wrapper and identify END use line 7, column 1 / definition line 3, column 1.
- `should_close_expanded_opening_and_continuation_actions_before_normal_syntax`: both single opening and multiline/comment-looking action payloads end with expanded FIN, then an actual inert comment and ordinary action syntax are correctly classified; authored final wrapper succeeds.
- `should_close_expanded_note_terminators_before_normal_syntax`: expanded end-note closes raw ownership; following normal syntax succeeds, but generated END is denied. The preceding raw `@enduml X` label still paints `@enduml Ready`.
- `should_keep_note_rows_raw_when_expansion_removes_authored_terminator`: eager/no-rescan opening and closing constants plus `note -> body` turn authored `end note` into literal `end body`; subsequent apostrophe row must still expand to `' Ready` until the actual expanded close.

Focused command: `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml_constants`. Meaningful RED: 27 passed / 4 failed, exit 101. The exact bypass incorrectly returned a rendered activity; generated action/note terminators misclassified an inert comment as `%getenv` capability; the inverse note transition left `' X` unexpanded. After the minimal state-timing correction: 31 passed / 0 failed, exit 0.

Fresh full gates after the final product edit used the previously listed locked/offline workspace test, strict Clippy, fmt, release, deny and source-audit commands. Evidence directory: `/tmp/ive-plantuml-3d2-fix2.TOSycC/` (`tests.log`, `clippy.log`, `release.log`). **309 Rust tests passed, zero failed/ignored**, plus all seven hostile-worker scenarios. Constants are 31; other suite counts remain unchanged. Strict Clippy/fmt/release/deny/source audits and nine audit tests all exit 0. PlantUML inventory remains 113/15/26; Merman 656/four. Inherited vendored Atomic::fetch_update warning only. `git diff --check` is clean; no product edits followed these gates.

Actual release repro: `/mnt/data/Projects/ive/target/release/diagram-viewer --render /tmp/ive-constants-rereview-generated-end.puml --output /tmp/ive-plantuml-3d2-fix2.TOSycC/rejected.svg` now exits **1**, reporting `PlantUML at line 7, column 1: "generated directive or wrapper"; expansion "END" [definition #1] used at line 7, column 1, defined at line 3, column 1`.

All five existing background native captures were regenerated with the fresh release binary into the fix2 evidence directory: `constant-sequence.png`, `constant-graph.png`, `constant-error.png`, `raw-note.png`, `raw-action.png`. Their SHA-256 values exactly match the initial/FIX ROUND 1 values above, proving identical pixels for all five prior controls. Actual successful native measurement counts remain 12, 14, 2 and 3 respectively, with zero fallbacks; the unchanged error has expected exit 1 and unchanged SIZE origin text. Each capture log is `capture-NAME.log`. All viewer processes self-terminated; anchored process lookup found none. No IVE instance or user input interference.

Residual subset/diagnostic limitations are unchanged. This correction awaits scoped independent re-review; it does not authorize the held timing task or claim full TIM/PlantUML parity.
