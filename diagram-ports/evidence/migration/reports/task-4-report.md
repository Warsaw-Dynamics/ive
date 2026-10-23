# Task 4 — sequence grammar cutover

Status: implementation complete, self-reviewed, committed and frozen for the controller's fresh independent SPEC/QUALITY gate. No independent review has been self-dispatched.

- Base: `f5a10a05df1cc03a5501c0235620cbac49592b3e`.
- Head: `c23afd66158758c992a0278511caf1b21cec3e88`.
- Commit: `Parse PlantUML sequence syntax with generated LALRPOP grammar`.
- Worktree: `/mnt/data/Projects/ive/.worktree/diagram-ports`.
- Product workspace: its `diagram-ports/` subdirectory. Final tracked status is clean.
- This report is in the existing ignored SDD workspace. Master progress and both controller ledgers were not edited. No merge, push, worktree removal, main IVE instance or user state changes occurred.

## Production route and preserved boundaries

The production sequence route is now `preprocess::prepare -> parser::document -> parser::preamble -> approved header selection -> parser::sequence::parse(BodySource, &mut Budget) -> preserved sequence::Sequence -> unchanged sequence layout/text/scene/SVG`.

`parser/grammar/sequence.lalrpop` owns participant declarations and aliases, lifecycle commands, message endpoints and complete arrows, optional colon labels, single/multiline positioned notes, recursive frame/end structure, else rows and EOF. It emits typed ParticipantKind, Name, Declaration, Message, Anchor, FrameKind and Lifecycle values. Frame and note-row repetitions fold into unit/actions; there is no generated raw-command list or recursive duplicated syntax tree. Arrow productions own exact supported shaft/head combinations from primitive run-boundary/operator tokens; no sequence call reaches `arrow::parse`.

The retained semantic Builder beside `Sequence` interns participants in source order, rejects duplicate declarations, preserves automatic participant identities and message direction/self ownership, validates live/dead/create/destroy and activation depth/balance, and preserves frame/else ownership. It constructs the same participants/events consumed by the existing layout. Participant/event/model order, IDs, geometry, native measurement/style/cache behavior and public APIs remain the comparison boundary.

Removed actual legacy methods: the Line-based sequence `parse`, `collect_note`, whole-command dispatch, declaration token splitting, lifecycle source parsing, frame source splitting, note positioning syntax matching and message/arrow source parsing. No old command parser is called from the generated facade or builders. The remaining `declare` and `message` methods receive typed values, not source statements. Source `Line` adapters remain only for header selection and other, still-unmigrated families. There is no parser retry, legacy fallback, partial recovery-and-render or generated Line-star forwarding.

All original sequence model/geometry assertions remain. Their setup now uses the real preprocess/document/preamble/BodySource route. The immutable 43-fixture baseline is unchanged; no fixture or baseline record was rewritten.

## Private lexical integration and origins

BodySource gained two private values: the existing shared Ownership state immediately before the first family physical row, and the conservative simultaneous Line-adapter byte allowance. Ownership now derives Copy/Clone over its existing enums/flags; it has no heap clone or new row index/vector. The document scan snapshots state before classifying a row and saves it when the generated preamble lexer first selects the family boundary. Sequence starts from that snapshot and continues the same shared classify/advance implementation over the actual remaining raw expanded bytes.

This handles a block comment opened in a consumed preamble row and closed on the first remaining row, including indentation and UTF-8 offset gaps. It adds no independent wrapper/comment pass. Row breakpoint mapping supplies truthful absolute expanded byte spans. CRLF, blank/indented raw note rows, expanded open/close markers and frozen literal definitions remain under the approved authored/current-row and expanded/next-row ownership rules. The expanded document remains immutable and is never rescanned as a preprocessor program.

Snapshot storage is included using actual `size_of::<BodySource>()` in the shared adapter allowance. Three actual Ownership sizes are charged before live/saved/transient initialization; each row snapshot and selected-state save charges before copying. Preamble admission reuses the same allowance. Sequence charges the actual Ownership size before continuation. No document/header/preamble grammar or generated output changed.

A public regression exposed real double mapping: a generated error at authored column 1 passed through the old outer family/layout bridge, so `BAD` appeared twice. Sequence parsing now happens before that bridge, as the generated preamble already does. Its facade maps structured failure exactly once; the existing bounded bridge remains for unmigrated family/layout errors. A frozen BOM/CRLF expansion failure now reports one use at line 5, column 1, defined at line 2, column 1, without unrelated `UNUSED` lineage. A separate direct comment-gap failure reports line 3, column 19.

One newly authored test expectation was corrected, without changing product ownership policy. It initially expected X expansion after a generated note opener sharing a physical row with an earlier comment close. The admitted authored ownership advances from that raw comment-close prefix, so the subsequent apostrophe row is frozen/inert. The corrected test places the close plus participant on the first family row and the generated OPEN on the next physical row. It then tests current-row literal preservation, expanded `end note` replacement, blank rows, CRLF and the expanded next-row CLOSE terminator. This expectation correction is GREEN control evidence, not claimed as feature RED.

## TDD and focused verification

All Cargo commands/subprocesses use `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`; Cargo operations use locked/offline flags. The known bwrap `RTM_NEWADDR` sandbox failure was handled with scoped escalated execution. No dependency fetch/install was needed.

Initial meaningful RED command:

```
cargo test --locked --offline -p diagram-engine parser::sequence::tests -- --nocapture
```

`/tmp/ive-task4-red.log`: four new facade contracts compiled and failed (0 passed/4 failed) through the explicitly unavailable generated-sequence stub: typed aliases/messages/self-call/frame/note/lifecycle construction, cross-preamble comment continuation, typed suffix/EOF rejection, and source-derived token/depth/work/cancellation errors. The unavailable stub was replaced, never retained as a production fallback. Existing admitted cases were controls, not presented as new REDs.

Real integration-defect RED command:

```
cargo test --locked --offline -p diagram-engine should_map_sequence_failure_once_at_a_frozen_expansion_use -- --nocapture
```

`/tmp/ive-task4-origin-red.log`: 0 passed/1 failed; the correct assertion observed expansion count 2 against expected 1. After moving generated sequence parsing outside the legacy outer bridge, the same contract passes.

Focused GREEN: `/tmp/ive-task4-green.log`, nine sequence contracts pass. Additional controls directly exercise generated streams with stray end, incomplete nested frame and illegal colon suffix; seven scheduled cancellation points in actual generated delivery/actions; token rejection before a fifth append retains length/capacity four; tiny depth/work; BOM/CRLF and raw expanded notes; direct UTF-8 comment-gap columns; once-mapped frozen origins. The new full-family contract checks participant alias/order plus reversed/self-message, note text and create/destroy identities. Existing private event-order, measured geometry and all common constants/control/style/resource controls pass in the complete suite.

During final verification, one full run concurrent with the release/Clippy gate process failed the unchanged `should_scan_whitespace_only_multiline_actions_once`: 160 passed/1 failed in the engine unit group, with `Cancelled(deadline_exceeded)` at 0.52s. That exact failed run is preserved at `/tmp/ive-task4-tests-concurrent-deadline.log`. After competing builds finished, three scheduled serial repetitions of the identical 60,000-row/500ms/work assertion passed (each 0.40s; `/tmp/ive-task4-action-control-{0,1,2}.log`), and the full workspace passed. No limit, assertion or exception policy was changed. The evidence shows timing sensitivity under that competing workload; it does not establish an exclusive cause. A later self-review added explicit pre-copy charges for growing note/quote buffers, followed by a fresh complete GREEN workspace and every remaining gate below.

## Actual generated/runtime admission

Durable proof: `diagram-ports/provenance/plantuml-parser-admission.md`, Task 4 section. Witness and exact output: `/tmp/ive-task4-stack-proof.py`, `/tmp/ive-task4-stack-proof.json`.

- Grammar SHA256: `7beb1315b2e7eec56b81ec30bcfce9edd040db232a0d8112725a144b95bdfeb7`.
- Generated SHA256: `add5ecf772bb37ce05df4a68a8050a7785d1ff06f82a2c1ec390c213f2fc356e`.
- Inspected actual tables: 72 states, 25 terminals. Bounded-depth all-terminal/EOF exploration finds 89/179/269/809/5849/5939 reachable shift stacks at depths 0/1/2/8/64/65, maximum 8/10/12/24/136/138 live states, four consecutive reductions, and at most twelve expected terminals.
- Only completed frame prefixes grow the stack: repeated states 2,6 add two states per frame; a finite suffix needs at most eight states. The opening action checks depth before retaining another prefix. Admission uses min(frame-token count, caller nesting limit), reserves `2d+12`, and includes growth capacity and malformed/diagnostic paths. No nullable recursive cycle, generated recovery or dropped-token accumulation exists.
- Actual 0.23.2 runtime state/symbol initialization, generated symbol payloads, reductions, goto functions, expected-token simulations and growth transients were inspected. Token payloads move into symbols/actions; the recovery-only token-clone path is unreachable. Five units per token and bounded EOF/diagnostic work are admitted before generated parser allocation.
- Admission includes actual token capacity and literal capacities, actual concrete symbol upper bounds, simultaneous old/new state/symbol/diagnostic/expected-String vector growth, all possible static expected names, still-live adapters/BodySource/Builder, and a separate conservative bound for model text/String growth. It checks retained bytes before entry against source*256 and stack storage against caller work capacity.
- A checked aggregate retained-byte counter governs participant/event/activation/life/frame vector growth before reservation; it charges copying existing vectors and checks actual capacity. Semantic model items and depth remain separate from temporary syntax quotas. Model items are charged before participant/event retention. Note aggregate labels remain capped at 16KiB; note/quote growth charges the old buffer before copy. Primitive scans, retained-token iteration, actions, copies and attempted EOF preserve typed cancellation/work failures.

No admission constant was copied from definition/document/preamble. The four existing generated outputs and their hashes are unchanged; sequence is the fifth reproducible parser.

## Final complete gates

All sixteen final gates pass after the last product edit. Exact fifteen-command/exit-status inventory: `/tmp/ive-task4-gates.json`; the full workspace is `/tmp/ive-task4-tests.log`. Every row below exited 0.

| Gate | Command/result | Evidence |
| --- | --- | --- |
| 1 | `cargo test --locked --offline --workspace`: **553 Rust tests + 7 actual worker faults** | `/tmp/ive-task4-tests.log` |
| 2 | `cargo clippy --locked --offline --all-targets -- -D warnings` | `/tmp/ive-task4-clippy.log` |
| 3 | `cargo fmt --all --check` | `/tmp/ive-task4-fmt.log` |
| 4 | `cargo build --locked --offline --release` | `/tmp/ive-task4-release.log` |
| 5 | `cargo deny --locked --offline --config ../deny.toml check licenses sources` | `/tmp/ive-task4-deny.log` |
| 6 | `node tools/audit-plantuml.mjs`: 220 MIT source/ZIP hashes, 43 fixtures, 43 native modules | `/tmp/ive-task4-plantuml-audit.log` |
| 7 | `node tools/audit-merman.mjs`: 656 imported files, four recorded patches | `/tmp/ive-task4-merman-audit.log` |
| 8 | `node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs tools/parser-baseline.test.mjs`: 18 tests | `/tmp/ive-task4-node.log` |
| 9 | `cargo test --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml`: three tests | `/tmp/ive-task4-generator-tests.log` |
| 10 | `cargo clippy --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all-targets -- -D warnings` | `/tmp/ive-task4-generator-clippy.log` |
| 11 | `cargo fmt --manifest-path tools/plantuml-grammar-gen/Cargo.toml --all --check` | `/tmp/ive-task4-generator-fmt.log` |
| 12 | `cargo deny --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml --config ../deny.toml check licenses sources` | `/tmp/ive-task4-generator-deny.log` |
| 13 | `cargo run --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml -- check`: five reproducible outputs, no tracked writes | `/tmp/ive-task4-generation-check.log` |
| 14 | `python3 -B tools/audit-plantuml-generator.test.py`: five notice guards | `/tmp/ive-task4-notice-tests.log` |
| 15 | `python3 -B tools/audit-plantuml-generator.py check`: 67 registry archives, unpacked sources, licenses/features/exact notices | `/tmp/ive-task4-closure-audit.log` |
| 16 | `node tools/parser-baseline.mjs check`: 43 immutable source/family/SVG hashes | `/tmp/ive-task4-baseline.log` |

The immutable baseline SHA256 remains `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`. The baseline driver's source/family/SVG assertions remain intact. No Merman regeneration, vendor edits, lock/dependency changes, source-JAR/hash changes, fixture changes or notice changes occurred. Actual existing MIT sequence traces now register the new parser/grammar destinations for CommandParticipant, CommandParticipantA, CommandArrow, CommandActivate, CommandGrouping and FactorySequenceNoteCommand. Authored/generated inventories and deterministic generation hashes register their actual paths. Existing inherited vendor Atomic deprecation warnings remain; strict Clippy succeeds. Their triage remains Task 17 scope.

## Genuine native frames, empty PATH and cleanup

Final harness: `/tmp/ive-task4-native.py`; summary: `/tmp/ive-task4-native-summary.log`. It runs the final release binary with `--background --language plantuml ... --capture`, so viewer creation uses its existing focus:false contract and each capture autquits. No focus activation or input/mouse automation occurred. The standalone viewer stores no settings/profile/user state.

| Case | Actual native/fallback calls | PNG bytes | Final PID |
| --- | --- | --- | --- |
| Existing sequence fixture | 7 / 0 | 144803 | 4069604 |
| Existing styled sequence | 20 / 0 | 224962 | 4069672 |
| Existing constant sequence | 12 / 0 | 217879 | 4069746 |
| Multiline raw-note/nested-alt/loop/self-call/create/destroy case | 16 / 0 | 174249 | 4069811 |

Artifacts: `/tmp/ive-task4-native-{sequence,styledsequence,constantsequence,note}.png` and corresponding `.log` files. The note source is `/tmp/ive-task4-note.puml`. Sequence and note frames were visually inspected: note blank-line/indentation/literal text, nested frame boundaries, self-call, activation layer, event spacing, alias heads and later created/destroyed lane are intact. The final rebuilt-binary capture hashes match the inspected captures. Styled sequence PNG SHA256 `f553a1abff0e85af9d4fca63f8a94a1f74efb2eb818a3254c5c4c9de1ed6e2b7` and constant sequence PNG SHA256 `2eb245de88313502ef4f15598d95405c5014a3d3d892033c549f453dcf33bc0e` exactly match accepted Task 3 frames.

Final binary exports with empty PATH succeed for sequence and the multiline-note source: `/tmp/ive-task4-empty-path-sequence.svg` (3889 bytes) and `/tmp/ive-task4-empty-path-note.svg` (8220 bytes), both exit 0 with no stdout/stderr. No runtime generator/Java/JS/browser/external renderer is required. All four final capture PIDs exit 0, and the harness's /proc check reports no viewer/worker survivors.

## Remaining scope and review boundary

Sequence has no remaining legacy command syntax route. Shared name/arrow primitives in other family parsers remain Task 5 work; other family adapters remain until their assigned tasks, and Creole nesting remains Task 16. The later-family Minors for Tasks 6/12/14, Task 17 baseline-default reporting/vendor noise and broader upstream unsupported syntax remain separate. There is no runtime sequence fallback through those pending modules. Public resources/EOF/unsupported-subset controls and every existing assertion remain unchanged.

The implementation is frozen at the head above with clean tracked status. The controller alone will arrange independent review and update master progress/ledgers.

## FIX ROUND 1 — I1 first primitive boundaries

Status: I1 implemented, self-reviewed and frozen for root's scoped re-review. FIX_BASE `c23afd66158758c992a0278511caf1b21cec3e88`; corrected head `a23aaeeaf17dc33734874dc2e86c024121b9581e` (`Restore PlantUML sequence primitive command boundaries`). Clean tracked status. Four scoped files changed: private parser, sequence semantic metadata, public `engine/tests/plantuml.rs`, and admission evidence. No other-family, grammar/generated, fixture/baseline, dependency/vendor, ledger/master or user-state edit.

The complete review `/tmp/ive-plantuml-lalrpop-sequence-review.md` was read first. I1 was verified against unchanged native primitive lexing and the reviewed removed sequence dispatch. The whitespace-prefix classifier was wrong: it merged an adjacent quoted argument into a command head and failed to dispatch adjacent-arrow/quoted reserved spellings as commands. First-command recognition now consumes one bounded primitive and compares its decoded spelling. The same bounded primitive-end scan serves subsequent words; first quoted names reuse the existing quote decoder and move into tokens when unreserved. Actual declaration/lifecycle/message/arrow/note/frame structure and EOF remain generated. No legacy whole-command parsing, forwarding or fallback was added.

The same-cause frame/else/note differences were covered before freezing. Typed `FrameKind::Literal` preserves the earlier raw display prefix before ASCII space, so a quoted/attached alt head still cannot own else. Quoted else retains the earlier raw byte-4 caption. Fixed-size row/suffix metadata opens the generated note-row stream for a quoted first Note primitive; existing Row.literal/opening_action metadata preserves its earlier normalized continuation rows while ordinary raw notes retain their literal rows. Every note-row visit, including discarded blank/comment rows, charges/checks. Shared ownership/comments, expansion, origins and other families are unchanged. These are compatibility values from the reviewed removed native code, not upstream-oracle expectations.

### Meaningful RED / GREEN

Before the primary fix, each command below failed its new real contract (0 passed/1 failed), in both private and public admission directions:

```
cargo test --locked --offline -p diagram-engine --lib sequence_command_primitives -- --nocapture
cargo test --locked --offline -p diagram-engine --lib reserved_first_primitives -- --nocapture
cargo test --locked --offline -p diagram-engine --test plantuml sequence_command_primitives -- --nocapture
cargo test --locked --offline -p diagram-engine --test plantuml reserved_first_primitives -- --nocapture
```

Evidence: `/tmp/ive-task4-fix1-{private,public}-{admit,reject}-red.log`. Admitted `participant"A"` failed at line2 column12, while rejected `participant->B : hello` produced a model/SVG. GREEN matrices also cover adjacent activate/deactivate, quoted first participant/lifecycle spellings, aliases, and adjacent-arrow/quoted endpoints across participant/actor/database/lifecycle/note/end commands.

After restoring the primitive head, these same-cause controls observed further RED before corresponding metadata fixes:

```
cargo test --locked --offline -p diagram-engine --lib primitive_command_caption -- --nocapture
cargo test --locked --offline -p diagram-engine --lib noncanonical_frame_spelling -- --nocapture
cargo test --locked --offline -p diagram-engine --test plantuml primitive_command_caption -- --nocapture
cargo test --locked --offline -p diagram-engine --test plantuml noncanonical_frame_spelling -- --nocapture
```

`/tmp/ive-task4-fix1-{private,public}-caption-red.log` shows quoted frame kind incorrectly rendered without its quotes; `...-frame-red.log` shows else incorrectly admitted under quoted alt. After fixing those frame/else values, rerunning the caption contract exposed quoted multiline-note rejection at line4 column3 (`...-{private,public}-note-red.log`). All are now GREEN; prior admitted controls were not relabeled RED.

Final serial verification after the last product edit:

```
cargo test --locked --offline -p diagram-engine --lib parser::sequence::tests -- --nocapture
cargo test --locked --offline -p diagram-engine --test plantuml --test plantuml_constants --test plantuml_control --test plantuml_styles -- --nocapture
cargo test --locked --offline --workspace
```

Results: private **13/13** (`/tmp/ive-task4-fix1-private-green.log`); public/common covering **133/133** =40+44+23+26 (`...-covering-green.log`); full workspace **561 Rust tests + all7 actual worker faults** (`...-tests.log`). Existing quote/alias/lifecycle/frame/note/operator, source-origin, budget/cancellation, native/cache/geometry and unchanged 60,000-row/500ms/work assertions pass. No common gate failure or limit/assertion change occurred in this round.

### Allocation/table evidence and complete gates

Admission evidence now documents the first Name temporary, fixed local row/suffix enums and literal-frame ownership. All lexical copies, including discarded command spellings, count under the existing source*2 syntax quota; a command's decoded spelling is at most eleven bytes. Frame-literal actual String capacities are included in pre-entry retained syntax. Token/symbol data moves into the checked frame arena; its actual enlarged element size is used before reservation. The event's separate display copy charges model text before allocation. No input-sized container/origin pass or generated parser storage was added. The existing concrete-symbol upper bound includes FrameKind through Token and is evaluated with actual size_of values.

Grammar/output SHA256 remain `7beb1315b2e7eec56b81ec30bcfce9edd040db232a0d8112725a144b95bdfeb7` / `add5ecf772bb37ce05df4a68a8050a7785d1ff06f82a2c1ec390c213f2fc356e`. `/tmp/ive-task4-fix1-stack-proof.json` reproduces the actual72-state/25-terminal witness:5849/5939 stacks at depth64/65,136/138 live states,max4 reductions,max12 expected. Same grammar/alphabet means unchanged stack/reduction/diagnostic/cancellation bounds; actual literal-capacity admission covers changed payload ownership. All five checked-in outputs are byte-identical and reproducible.

All sixteen final common gates pass. The first/full workspace gate is recorded above; the remaining fifteen exact commands, wrapped argv, exits and log paths are in `/tmp/ive-task4-fix1-gates.json`. They are the same fifteen remaining common commands listed in the initial report's gate table, with logs `/tmp/ive-task4-fix1-{clippy,fmt,release,deny,plantuml-audit,merman-audit,node,generator-tests,generator-clippy,generator-fmt,generator-deny,generation-check,notice-tests,closure-audit,baseline}.log`. Outcomes: strict Clippy/fmt/release/both deny checks pass; MIT220/43/43 and Merman656/four pass; Node18, generator3, notice5 and exact67-package closure pass; generation verifies five unchanged outputs; immutable baseline43 passes. Baseline SHA remains `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`; no rerecord.

All Cargo/subprocesses inherit `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target`. The serial private/covering/full run had already completed when root introduced parallel worktree coordination. Every remaining Cargo or Cargo-spawning outer gate then used exactly one outer `bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target ...` lock. No nested locks or competing test/build processes were started by this implementer.

### Native / empty PATH / freeze

Final native harness `/tmp/ive-task4-fix1-native.py` ran under the same outer lock across this checkout's release build and captures, preventing shared-target binary replacement by other worktrees. Summary `/tmp/ive-task4-fix1-native-summary.log`: five genuine background/autquit frames, sequence7/styled20/constant12/raw-note16/primitive-command6 native calls, every fallback0. Final PIDs38081,38158,38225,38303,38372 exited0; /proc cleanup found no viewer/worker survivors. Artifacts use `/tmp/ive-task4-fix1-native-*.png/.log`; the extra primitive source is `/tmp/ive-task4-fix1-primitive.puml`.

Primitive-command and raw-note frames were visually inspected. The extra frame demonstrates adjacent aliases/lifecycle, quoted group display, quoted note normalization and quoted end. Standard sequence/styled/constant frame hashes match initial Task4 captures byte-for-byte. The raw-note SVG exported with empty PATH is byte-identical to its initial Task4 export (SHA256 `77214e453b1db09fbcea36e678152188a01bd0b4bc070a1701f872acb2d001f2`); its full UI frame now displays the fix-round source pathname. Both final empty-PATH sequence/note exports pass (3889/8220bytes). No focus/input automation, main IVE/user-state change, external renderer or surviving process.

Frozen at corrected head `a23aaeeaf17dc33734874dc2e86c024121b9581e`, tracked-clean. No known remaining I1 issue. Root alone arranges scoped re-review and integration/progress/ledger updates; none was self-dispatched.

## FIX ROUND 2 — R1 malformed quoted frame headers

FIX_BASE `a23aaeeaf17dc33734874dc2e86c024121b9581e`. Read the complete `/tmp/ive-plantuml-lalrpop-sequence-rereview-1.md` and updated `parallel-context.md`. I1 remains addressed. R1 was confirmed against the reviewed removed entrypoint: it tokenized the complete pre-colon head before dispatch, while the raw frame/else branch bypassed suffix quote validation. Three scoped files change: private sequence parser, public PlantUML tests and admission proof. No grammar/generated/model/public API/dependency/vendor/other-family or ledger/master edit.

Frame/else rows now validate remaining quoted primitives before copying raw frame display/caption values. The allocation-free scan rejects unterminated quotes and unsupported quoted escapes, preserves the native raw 16KiB quoted-token bound, and stops at the first unquoted single colon (`::` stays in the header). Valid attached/quoted frame display, raw quoted-else captions and canonical Alt/Par else eligibility are unchanged. Post-colon quotes remain literal caption data; the existing Creole layer still decides its own escape support. All actual frame/nesting/else/end/EOF syntax remains generated; no legacy command parsing/forwarding/fallback was introduced.

Meaningful RED before product changes, both exit101 / 0passed1failed, showed `group"unterminated` incorrectly producing a private model/public SVG:

```
cargo test --locked --offline -p diagram-engine --lib malformed_quoted_sequence_frame_headers -- --nocapture
cargo test --locked --offline -p diagram-engine --test plantuml malformed_quoted_sequence_frame_headers -- --nocapture
```

Logs `/tmp/ive-task4-fix2-{private,public}-reject-red.log`. New controls cover all seven frame commands plus else with attached unclosed/unsupported-escape, spaced quoted-colon and double-colon suffixes; existing valid attached/quoted frame and I1 matrices remain. Five colon-literal controls pass both privately/publicly. Private controls retain post-colon `\q` in the model. The initial public control also included `\q`; it failed in existing Creole (`label escape \\q`), so that fixture assertion was corrected before product edits to assert literal unterminated quote text without that independently unsupported escape. This was a control-expectation correction, not feature RED or a product policy change: `/tmp/ive-task4-fix2-public-colon-control-red.log` and corrected pre-edit GREEN `...-public-colon-control.log`; private pre-edit GREEN `...-private-colon-control-red.log`.

Final covering commands after the last product edit, all exit0:

```
cargo test --locked --offline -p diagram-engine --lib --test plantuml --test plantuml_constants --test plantuml_control --test plantuml_styles -- --nocapture
cargo clippy --locked --offline -p diagram-engine --all-targets -- -D warnings
cargo fmt --all --check
node tools/audit-plantuml.mjs
```

Covering **302/302** =167engine units+42PlantUML+44constants+23controls+26styles, including the existing I1/frame/else/note/quote/alias/lifecycle/operator/source-origin/resource/cancellation contracts and unchanged 60,000-row/500ms/work assertions. Logs `/tmp/ive-task4-fix2-{covering-green,clippy,fmt,source-audit}.log`; strict Clippy/fmt pass with only inherited Merman deprecation warnings, source audit verifies220MIT hashes/43fixtures/43native modules. Earlier focused private GREEN15/15 is in `...-private-green.log`.

Every Cargo invocation used the outer `bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target ...`. Generator exception ran from `/tmp` with private target `/mnt/data/Projects/ive/target/diagram-generator-integration`, absolute manifest `/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/tools/plantuml-grammar-gen/Cargo.toml`, and `cargo run --locked --offline --manifest-path ... -- check` under that same outer lock: five outputs verified (`/tmp/ive-task4-fix2-generation-check.log`). No shared-cache clean/metadata tricks, viewer rebuild or native/baseline executable was run in this round.

Actual allocation/control evidence: the new header scan borrows Row, has only fixed-size cursor/escape locals and allocates nothing. Each visited UTF8 character charges/checks before processing; quote/escape/colon delimiters are included. One extra linear pre-colon scan is paid by existing work, before any raw display copy. Existing Row.span and once-mapped origin facade handle failures. No retained payload/arena or parser table changes: prior actual72states/25terminals, four reductions, twelve expected terminals, depth64/65 stack witness and transient admission remain applicable without changed constants. Durable proof appended to `provenance/plantuml-parser-admission.md`.

Grammar/generated SHA256 unchanged: `7beb1315b2e7eec56b81ec30bcfce9edd040db232a0d8112725a144b95bdfeb7` / `add5ecf772bb37ce05df4a68a8050a7785d1ff06f82a2c1ec390c213f2fc356e`. Immutable baseline SHA remains `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`. Per the updated parallel context/root direction, broad16gates, native/empty-PATH and baseline execution are central-only for the integration batch; this round does not claim they were rerun. Self-review found no additional R1 issue; root arranges scoped re-review. Freeze head is recorded below after commit.

Frozen head `1506eb1f14248eb6dfe109164308f6914e3421ec` (`Validate PlantUML sequence frame header quotes`), tracked-clean. R1 implemented and ready for root's scoped re-review; no known remaining issue in this finding.
