# Task 3B implementation report — native states and structured activities

## Frozen checkpoint

- Status: implemented and locally verified; ready for fresh independent specification and quality review, not self-approved.
- Exact base: `130ca1064669f312d8883f2686f939f1bc775bfc` (controller explicitly released HOLD).
- Exact implementation HEAD: `520122dbf062c78e7529fe21cc9c83f6bdd0e927` (`Add native PlantUML state and structured activity checkpoint`). Tracked worktree clean after commit.
- Worktree: `/mnt/data/Projects/ive/.worktree/diagram-ports`; standalone product: its `diagram-ports/` directory. All committed changes are inside that standalone project. Root IVE product, Cargo manifests/lockfile, dependencies and worker protocol remain unchanged.
- This report is an ignored controller artifact, outside the product commit. No merge or push; worktree retained. One product implementer, no subagents or self-dispatched review.

## Contracts and delivered scope

Public render/options/result/measurement/control APIs are unchanged. The engine has no GPUI dependency, filesystem/network/environment/clock access or process launches. It uses shared `Budget`, text `Context`, typed `Scene`/`Shape`, bounded safe SVG serialization and checked Dagre machinery. The host retains its isolated same-binary worker and private same-tree `DisplaySvg` paint containment, without a host-metrics capability change.

Defaults remain source 256 KiB, SVG 16 MiB, model 10,000, nesting 64, work 5,000,000 and cooperative 10 s; host worker independently retains 10 s / 2 GiB Unix limits. New parsing, retained text/model growth, state identity searches, layout and emission use those controls. Text/action scanning avoids repeatedly rescanning the accumulated body. State graph work has shared cancellation checkpoints, and activity composition checks work between structured tiles.

### Admission and text preparation

Ordered complete-block family selection recognizes actual structural commands, not every word in class/note/action payloads. Existing sequence actor/database precedence, class members named `loop`, timing rejection and whole-input consumption remain tested. New regressions cover members beginning `state value`, `start`, `stop`, `:member`, note-like member headers, and literal state/action words in note bodies. Unsupported substantive commands, suffixes and decorations fail instead of creating partial diagrams.

Shared preparation now preserves multiline action and note bodies: meaningful blank rows, relative leading/trailing spaces, interior semicolons, and comment-looking literal text including the opening action row. Outside text bodies, existing blank/comment grammar remains. No JSON/YAML/grid-body stripping was introduced for future families.

### State model and layout

`state.rs` owns a distinct typed state model: root/state ownership scopes, ordinary/composite/initial/final kinds, ordered descriptions and transitions. Declarations are resolved before pending links/fields, including forward references to composites; pending descriptions are applied in source order. Each ownership scope reuses its own initial and final node, with `[ * ]` rejected and exact `[*]` roles resolved after arrow reversal. Initial and final roles are distinct even where several transitions mention `[*]`.

Admitted syntax includes plain state names, quoted displays, one quoted display plus a plain alias, ordinary directed/reversed transitions with labels, implicit ordinary endpoints, nested composites with brace or begin/end-state forms, and attached/body descriptions including entry/exit and literal hyphens. The parser does not reinterpret decorated endpoints as state identifiers.

`stategraph.rs` is a separate original recursive checked-Dagre adapter, not the UML declaration model. Child scopes are laid out bottom-up; measured composites enter their parent's graph as nodes. It emits rounded state/composite boxes, genuine initial dots and final bullseyes, attached body captions and stable semantic IDs/metadata. State SVG IDs are `s{id}` and transition IDs `t{id}`; kind and endpoint ownership remain observable for semantic tests. Same-scope links and links to a composite boundary in its parent scope are supported. Qualified state declarations and arbitrary cross-ownership transitions are explicitly outside this checkpoint.

### Activity model and layout

`activity.rs` owns bounded structured instructions for Start, Stop, End, Action, If/elseif/else, While, Repeat, Fork and Group. `activitytile.rs` composes measured tiles bottom-up with explicit entry/exit ports and live/dead exits. It does not lower activities through UML, Mermaid source, sequence layout or a generic free graph.

Admitted syntax includes `start`, `stop`, `end`, raw `:action;`, nested `if (test) then (guard)` (also optional then), `elseif`/`else if`/`else`/`endif`, while/is-or-equals/endwhile guards, repeat entry actions and repeat-while positive/negative guards, fork/fork-again/end-fork or fork-end, end-merge, and brace-delimited group/partition. Full fork join specifications such as `{all}` are retained as label text, matching the original MIT oracle.

Only live branches reach convergence and following actions. An omitted else supplies the implicit false route. Elseif has an actual condition chain. While has a live back edge and false exit; repeat has a positive back edge and negative exit. Fork lanes have separately measured widths; synchronization bars span all lanes, while end-merge uses a merge diamond. Stop is a bullseye, end a crossed circle; neither has fallthrough. Partition/group boxes contain their nested shapes and labels.

### Explicit exclusions

- State concurrency/regions, history, choice/fork/join stereotypes, ports, arbitrary cross-ownership links, qualified declarations and styled/directional transition semantics remain rejected. Future concurrency/history research was not implemented.
- Activity swimlanes, switch, break, kill, detach, backward/goto, legacy activity syntax and unsupported decorators remain rejected. Next-arrow lane labels, repeat backward-route suffixes and alternate `if ... is ... then` forms are not supported. Group/partition bodies require `{`.
- No new themes, math/ELK/runtime capability, wrapper families or filesystem include/preprocessor capability.
- This is a useful native subset, not a full PlantUML port or exact font/geometry parity claim. Diagnostic token columns remain 1.

## Source authority, translation trace and notices

Every newly translated behavior was inspected in actual `/mnt/data/Projects/plantuml-mit/src`, MIT edition 1.2026.9; supplementary research notes were not treated as source authority. No GPL repository, rejected puml candidate, gen/, h/, Smetana, Graphviz or ELK implementation translation was used. Dagre composition and Rust tile geometry are original implementation choices, not a claim to reproduce the original layout backend.

The manifest now records 69 exact selected source files (37 new), 7 fresh fixture hashes (2 new) and all 16 native modules (4 new). Its per-file entries contain SHA-256, Rust destinations, source-specific behavioral notes and notices. New traces are:

| Authorized source under `net/sourceforge/plantuml/` | Translated contract / destination |
| --- | --- |
| `statediagram/StateDiagram.java` | Scoped role-specific pseudostate identities / `state.rs` |
| `statediagram/command/CommandLinkStateCommon.java`, `CommandLinkState.java`, `CommandLinkStateReverse.java` | Endpoint grammar, direction and label attachment / `state.rs` |
| `statediagram/command/CommandCreateState.java`, `CommandCreatePackageState.java`, `CommandAddField.java`, `CommandEndState.java` | Display/alias forms, composites, fields and closers / `state.rs` |
| `activitydiagram3/command/CommandActivity3.java`, `CommandActivityLong3.java` | Single/multiline action framing / `activity.rs`, shared `source.rs` |
| `activitydiagram3/command/CommandIf2.java`, `CommandElseIf2.java`, `CommandElse3.java`, `CommandEndif3.java`; `InstructionIf.java` | Ordered branch guards and branch construction / `activity.rs`, `activitytile.rs` |
| `activitydiagram3/command/CommandWhile3.java`, `CommandWhileEnd3.java`, `CommandRepeat3.java`, `CommandRepeatWhile3.java`; `InstructionRepeat.java` | Loop grammar, repeat entry, guards / `activity.rs`, `activitytile.rs` |
| `activitydiagram3/command/CommandFork3.java`, `CommandForkAgain3.java`, `CommandForkEnd3.java` | Separate fork arms, join/merge and complete join specification / `activity.rs`, `activitytile.rs` |
| `activitydiagram3/command/CommandPartition3.java`, `CommandCloseGroup3.java` | Structured group ownership / `activity.rs`, `activitytile.rs` |
| `activitydiagram3/command/CommandStart3.java`, `CommandStop3.java`, `CommandEnd3.java`; `ActivityDiagram3.java` | Lifecycle and terminal instructions / `activity.rs`, `activitytile.rs` |
| `activitydiagram3/ftile/vcompact/FtileFactoryDelegatorAssembly.java`, `VCompactFactory.java`, `FtileWhile.java`, `FtileRepeat.java`, `ParallelBuilderFork.java`, `FtileIfLongHorizontal.java` | Live/dead assembly, structured branch/loop/fork topology / original Rust tile composition |
| `activitydiagram3/ftile/vertical/FtileCircleStop.java`, `FtileCircleEndCross.java` | Distinct stop/end terminal symbols / `activitytile.rs` |

Source artifact SHA-256 remains `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`. The audit compares each local source byte-for-byte against its exact source-JAR entry as well as the manifest hash and MIT grant. This verifies local provenance consistency, not independent external authentication.

Original author/contributor metadata is now audited with flexible spacing before `:`. Controller caught the four `Contribution   :  Serge Wenger` omissions in StateDiagram, CommandLinkStateCommon, CommandLinkState and CommandLinkStateReverse; all four exact credits are now retained. README, THIRD-PARTY-NOTICES and the MIT notice appendix identify the newly used state/activity source and Serge Wenger, preserving prior Arnaud Roques/PlantUML contributors/Hisashi Miyashita credits. No other selected-source author/contribution omission was found in the controller's independent 69-file scan.

The audit's source allowlist includes state/activity subdirectories while explicitly rejecting nested gen/h/smetana/elk entries. Negative Node tests cover unauthorized sources and stripped contributor credit.

## RED/GREEN and semantic evidence

Focused commands used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --test plantuml_control` and named private/unit filters as appropriate. Viewer example tests used `-p diagram-viewer --lib` (an initial mistaken bin-only invocation ran zero tests and is not counted as evidence). Node audit tests used `node --test tools/audit-plantuml.test.mjs`.

Observed RED then GREEN during implementation:

1. Initial five control tests rejected missing state/activity families; state's three tests became green first, then expanded activity tests.
2. Class member and note/action keyword admission regressions failed before structural family selection. Existing tests caught shared preparation swallowing graph-note content; corrected preparation restored all 33 prior public PlantUML tests. The obsolete unsupported-state/activity cases were replaced with genuinely unsupported choice/break.
3. Ordered state descriptions and literal hyphens failed before pending source-order attachment; class `:member`/note-like header payloads failed before graph-body-aware preparation.
4. First raw action row trailing whitespace/comment-looking content failed before preservation ahead of comment stripping.
5. Unequal fork lane geometry failed before measured column widths and spanning join bars.
6. New viewer example lookup failed before adding both examples. Real capture then exposed the old hardcoded sidebar count; it now derives the count from the examples.
7. Original MIT oracle showed `{all}` including braces; the corrected expectation failed against stripped text, then passed with full join specification preservation.
8. Removed-contributor metadata failed the negative audit test before exact credit enforcement and notices were fixed.
9. A 1,000-state declaration identity scan incorrectly fit a 100,000-work budget before comparison work was charged; the regression now rejects it.
10. A 60,000-row whitespace action exceeded the test's 500 ms parser deadline due to repeated accumulated-body trimming; scanning only the current row made it pass (final complete private suite ~0.30 s).
11. Quoted-to-quoted alias forms were mistakenly admitted; rejecting the unsupported alias/composite forms made the focused test green.

Additional topology, geometry, cancellation and containment tests were added against working implementation and passed immediately; those are not claimed as independent RED observations. Public control tests cover ordered transitions, reverse pseudostate roles, per-scope reuse, forward composites, nested ownership, exact fixture admission/determinism, if/elseif convergence, implicit false paths, dead branches, both loops, fork join versus merge, nested partitions, malformed stacks/endpoints, unsupported features, raw text, tiny model/depth/work/output limits and cancellation during layout measurement. Private tests inspect child/label containment, distinct fork lanes, actual loop routing and cancellation inside checked state layout.

## Final verification

All commands ran from the standalone project with `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` and locked/offline Cargo. Final product edits preceded this complete gate. Scoped elevated execution was required for the environment's ordinary bwrap loopback failure; no escalation was rejected.

| Gate | Result |
| --- | --- |
| `cargo test --locked --offline --workspace` | 194 Rust tests passed, zero failures |
| Engine private | 24 |
| Compatibility | 11 |
| Mermaid families / general | 37 / 21 |
| Prior PlantUML public / new control public | 33 / 17 |
| Viewer unit / export / native lifetime / worker integration | 37 / 5 / 3 / 6 |
| Actual hostile-worker scenarios (separate executable) | 7 passed: blocked stdin, blocked stdout, stderr flood, oversize frame, early exit, memory, cancel |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | Exit 0 |
| `cargo build --locked --offline --release` | Exit 0 |
| `cargo fmt --all -- --check`; `git diff --check` | Exit 0 |
| `cargo deny --config ../deny.toml check licenses sources` | Exit 0, licenses and sources OK |
| `node --test tools/audit-merman.test.mjs tools/audit-plantuml.test.mjs` | 7 passed (3 existing + 4 PlantUML) |
| `node tools/audit-merman.mjs` | 656 imported files, 4 existing patches verified |
| `node tools/audit-plantuml.mjs` | 69 exact source/ZIP hashes, 7 fixtures, 16 modules, contributor notices verified |

Final logs: `/tmp/ive-plantuml-3b.WmiVul/tests-final.log`, `clippy-final.log`, `release-final.log`, `deny.log`. Only the unchanged vendored merman `Atomic::fetch_update` deprecation warning remains; strict project Clippy passes. Controller independently reran 194 Rust tests, seven fault scenarios, both source audits and seven Node guards on the current product state and reported success. Root IVE builds/tests were not run or claimed.

## Original oracle and real native captures

Exact independently authored fixture copies are admitted unchanged:

- `fixtures/plantuml/state.puml` = original `nested-state.puml`, SHA-256 `081bdcec241cbc1b3cb035cfe0d7e42dcee2efa45813e0de058c0df8533b24bf`.
- `fixtures/plantuml/activity.puml` = original `activity-branch-merge.puml`, SHA-256 `5f2ccae11dc644b2227663247a92936b17fe6d0b8f280e4a7d9aa813579d2979`.

Original SVG and probe JSON/result JSON in `/tmp/ive-plantuml-native-oracles` were compared for admitted semantics. State has three scoped initials, local final, nested Packed with Sealed/Checked and directed inspect transition. Activity's yes/Persist and no/Reject branches converge before Reply. Both native exact-fixture exports are deterministic and parse with usvg.

Original nested state viewBox is `0 0 174 627` with 29 shapes; native export is `0 0 296 1089.4`. Original activity is `0 0 166 322` with 29 shapes; native is `0 0 374.84000000000003 651.2`. Original state uses Liberation Sans 14/13 px and activity 12/11 px, versus the native uniform 14 px measurement contract. Spacing, padding, routing and text metrics differ; no pixel parity is claimed. Native exports are `state-native.svg` and `activity-native.svg` in the evidence directory.

Additional original runtime probes used only the verified MIT jar `/tmp/ive-diagram-oracles.JwN5H5/plantuml-mit-1.2026.9.jar` (SHA-256 `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd`), a 20 s timeout, headless Java, SANDBOX security profile, UTF-8, fixed Liberation Sans and `-tsvg -pipe`. `fork-join-original.svg` confirms literal `{all}`; `activity-control-original.svg` admits the custom branch/dead-end/while/partition source with all intended labels. Java is development-only oracle evidence, never part of production rendering.

Final release captures were actually opened and visually inspected, each 1360×880, background capture with automatic quit:

| Capture under `/tmp/ive-plantuml-3b.WmiVul/` | Native / fallback | Inspected contents | SHA-256 |
| --- | --- | --- | --- |
| `state-final.png` | 8 / 0 | Exact nested fixture, three scoped initial dots, Packed children, pack/inspect labels, local bullseye and rounded nested boxes; sidebar shows 7 | `ae79bc418d75dfe0a0e8550ad3ed5519968a2dd82588a844d17d7187bc3419be` |
| `activity-final.png` | 11 / 0 | `activity-control.puml`: Order service partition, Valid? branches, crossed end on Reject with no fallthrough, Persist convergence, More items? back edge/false exit, Reply then stop bullseye | `8714049af85288a373336b0f0d8bf0c5521574b2b199b1d3604bace55f32910c` |
| `fork-final.png` | 4 / 0 | `fork-join.puml`: start, distinct Left/Right lanes, spanning split/join bars, retained `{all}`, stop | `4fcd723567319ebafb654f2464df238bc794fb04bc5b05ef3c23fba7733ceb5b` |

Matching `state-final.log`, `activity-final.log`, `fork-final.log` retain actual native/fallback counts. The focused actual-worker lifetime command `cargo test --locked --offline -p diagram-viewer --test native_lifetime should_measure_plantuml -- --nocapture` passed and logged sequence 8/0, description 8/0, state 8/0 and activity 8/0 with cache release (`native-final.log`).

All viewer test instances exited; `pgrep -a -x diagram-viewer` returned no processes. No IVE instance was started, no user profile was touched, and no xdotool/window activation or real-mouse input was used.

## Remaining risks and handoff

Semantic exclusions above are intentional hard rejection, not silently approximated support. Geometry remains conservative original Rust layout rather than PlantUML geometry parity; arbitrary edge-label collision freedom is not proven. Host baseline metrics remain the existing approximation. Long composition functions do not uniformly meet the preferred <20-statement guideline, though the model/parser/layout responsibilities are separated. Source columns remain coarse. The action deadline regression is a performance sentinel rather than a machine-independent complexity proof.

Independent review must evaluate this exact base-to-head diff, unsupported-substantive admission, raw-text handling, bounded work and live/dead control topology before later family work or state concurrency/history extensions. No additional product task was started.

## Fix round 1 — six Important review findings

### Scope and frozen revision

- Review read completely: `/tmp/ive-plantuml-state-activity-review.md`, findings R1–R6. Controller explicitly authorized one scoped repair wave; no next feature task, subagents, or broad refactoring.
- Exact fix base: `520122dbf062c78e7529fe21cc9c83f6bdd0e927`.
- Exact fix HEAD: `49838e86eea67155710bf29d07179935e45009bc` (`Fix PlantUML control admission and painted flow continuity`). Tracked worktree clean after commit.
- Changed product files: `engine/src/plantuml/{source,state,mod,uml,activitytile}.rs` and `engine/tests/plantuml_control.rs`. No root IVE, viewer, worker/protocol, dependency, metrics capability or provenance inventory changes.
- Compact-colon and repeat-positive-guard suspicions were resolved by review and deliberately left unchanged.

### Repairs and contracts

**R1 — raw-body preprocessing boundary.** `source::prepare` now rejects a leading `!` directive (including leading whitespace) before retaining an action/note body row. It still retains literal non-leading exclamation marks and all legitimate blank rows, indentation, trailing spaces, semicolons and comment-looking text. Public regressions cover missing include and define/conditional directives in both body types and assert the actual offending source line. The private source test confirms literal `!` and surrounding raw whitespace are preserved.

**R2 — ancestor identities.** Before creating an implicit state endpoint, lookup now checks ancestor scopes with precharged comparison work. An existing ancestor produces an explicit cross-ownership rejection, never a new local copy. Declared identities are also checked after declaration collection, so ancestor-name redeclarations cannot introduce false shadow nodes regardless of declaration order. Source line is retained on state nodes for accurate errors. Ordinary explicitly nested distinct states, same-scope identities and per-scope initial/final pseudostates retain their prior contracts. No cross-ownership routing was added. The private model test directly verifies a rejected resolution leaves the original three nodes and the single root A unchanged.

This repair was checked against the already inventoried authorized `CommandLinkStateCommon.java` contextual lookup and `CommandCreateState.java` declaration grammar, plus the exact MIT oracle. Root `state A` followed by `state Outer { state A; state B; A --> B }` has only root A and Outer.B, not a new Outer.A (`shadow-original.svg`). The reverse declaration order, Outer.A before a later root `state A`, also produces one A identity (Outer.A, `shadow-later-original.svg`). Both native cases now reject explicitly rather than fabricate a second identity. No new upstream source file was translated; no provenance entry or source credit was removed or invented.

**R3 — command words versus identifiers.** Family selection, UML graph fallback and shared preparation now distinguish an identifier followed by an arrow from a structural command. The public regression renders 27 cases: nine state/activity/UML-like names (`start`, `stop`, `state`, `repeat`, `fork`, `partition`, `class`, `if`, `while`) as implicit, actor-declared and database-declared sequence participants. A separate regression preserves raw note rows after `state -> Bob`. Existing class-member and note/action payload regressions remain green. Exact pseudostate evidence is excluded from the identifier-arrow shortcut; `A --> [*] : finished` still selects the implicit state family.

**R4 — attached descriptions on quoted displays.** Pending state declarations dispatch as declarations before scanning for transition arrow punctuation. Quoted display strings containing hyphens, `<`, or `->` now retain their attached descriptions. Declaration/alias validation and unsupported decoration rejection remain unchanged.

**R5 — visible group input.** Activity container backgrounds are emitted before content/flows, retaining parent-before-child order and charging the extra linear partition pass. Groups/partitions now expose a boundary entry port with their own identity and an explicit internal connector to the first child. That connector uses a right-hand header gutter before turning toward the child, avoiding title text. Long captions receive additional horizontal routing clearance. Incoming arrowheads are therefore visible through nested composition; stop/end live/dead behavior is unchanged. Private tests assert arrowhead coordinates at the action boundary, opaque-container paint order, and every flow segment's separation from nested long captions plus their routing clearance. A branch-to-partition input case is included, not just a sequential block.

**R6 — labeled join continuity.** The synchronizing join's output port is at the painted bar's bottom (8 px), independently of reserved label height. Its label is placed to the right of the outgoing stem below the bar. The next connector therefore begins on the bar and reaches the following action without a gap or crossing `{all}`. The private test compares the actual first/last path points against painted bar/action rectangles and verifies label/stem separation; it does not rely solely on owner IDs.

### Observed RED → GREEN

Evidence directory: `/tmp/ive-plantuml-3b-fix1.40YnXd`.

1. Before fixes, `cargo test --locked --offline -p diagram-engine --test plantuml_control` produced **17 pass / 4 fail**: raw directive painted as text, ancestor copy accepted, implicit `start` selected as activity, quoted `well-known` mistaken for arrow. Saved as `red-public.log`.
2. Before geometry fixes, `cargo test --locked --offline -p diagram-engine --lib activitytile::tests` produced **5 pass / 2 fail**: opaque partition after arrowhead, and detached join flow start y=280.8 versus bar bottom y=251.4. Saved as `red-geometry.log`.
3. Each of the six fixes passed its corresponding focused test before moving to the next repair.
4. The controller's long-caption requirement exposed a genuine extra R5 failure after the paint-order-only repair: `flow f5 crosses caption ...` (`red-caption.log`). Boundary entry plus right-gutter routing made it green. Real native capture then showed insufficient visual clearance at the inner caption; a stricter 24 px measured routing-clearance assertion failed (`red-caption-clearance.log`) before increasing caption-side padding. Final captures visibly separate both long captions from the route.
5. During R3 regression checking, `A --> [*] : finished` failed after the initial identifier-arrow shortcut (`red-pseudostate.log`). Keeping exact pseudostate evidence ahead of that shortcut restored admission. The final test suite includes this case.
6. Additional private preservation/no-mutation and public raw-note tests passed against the working fix; these are supplemental checks, not claimed as independent pre-fix RED observations.

All Cargo commands use `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` from the standalone project. Focused final logs `green-public.log` and `green-private.log` show **23 public control tests** and **29 private engine tests**, respectively, with zero failures.

### Final gates on the final repaired product tree

The complete gate was rerun after the last routing-clearance edit, not merely reused from the earlier passing attempt:

| Command / gate | Final observed result |
| --- | --- |
| `cargo test --locked --offline --workspace` | **205 Rust tests: 176 non-private + 29 engine-private**, zero failures |
| Test suite breakdown | 29 engine-private; 11 compatibility; 37 Mermaid families; 21 general/Mermaid; 33 prior PlantUML public; 23 state/activity public; 37 viewer unit; 5 export; 3 native lifetime; 6 worker integration |
| Separate actual hostile-worker scenarios | **7 passed**: blocked stdin/stdout, stderr flood, oversize frame, early exit, memory, cancel |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | Exit 0 |
| `cargo build --locked --offline --release` | Exit 0 |
| `cargo fmt --all -- --check`; `git diff --check` | Exit 0 |
| `node tools/audit-plantuml.mjs` | 69 exact MIT source/ZIP hashes, 7 fixtures, 16 modules, source credits preserved |
| `node tools/audit-merman.mjs` | 656 imported files, 4 recorded patches |
| `node --test tools/audit-merman.test.mjs tools/audit-plantuml.test.mjs` | 7 passed |
| `cargo deny --config ../deny.toml check licenses sources` | Exit 0: licenses ok, sources ok |

Logs `tests-final.log`, `clippy-final.log`, `release-final.log` contain final command output. The unchanged vendored Atomic::fetch_update deprecation warning remains disclosed. Root IVE tests/builds were not run or claimed. No current failure is omitted.

### Final native captures and inspection

Each final capture used `/mnt/data/Projects/ive/target/release/diagram-viewer <source.puml> --background --capture <final.png>`, exited 0, and was actually opened and inspected at 1360×880. Captures were regenerated after the final product edit. Matching `.puml` inputs and `-final.log` outputs are in the evidence directory.

| Final capture | Native/fallback | Visually inspected result | SHA-256 |
| --- | --- | --- | --- |
| `group-flow-final.png` | 3/0 | Start connector reaches partition boundary; visible internal line and arrowhead reach Inside; outgoing line reaches stop | `ea404ffb5c1f78dcf9b0236ab3020fb4bbb5874e74eb18f865cab20a6b8f3e90` |
| `nested-caption-final.png` | 4/0 | Outer and inner long captions fully readable; both boundary entries/internal connectors are visible and clear of caption text; continuous route to Inside and stop | `e9c462606352c832d492fd01b01f68440398ad3775307cf3678b21e41b9870c3` |
| `fork-following-final.png` | 5/0 | Two separate lanes join at a real bar; outgoing stem touches the bar, `{all}` sits beside it, and After then stop are visibly connected | `bf8206d5b270a60916c908bc0601fc9c50a7af27ce1a68d3843ab0d437c90339` |

`group-flow-final.svg` and `fork-following-final.svg` are deterministic headless exports for direct paint-order/path inspection. Older non-final capture names preserve the intermediate inspection evidence and are not the final handoff images.

The actual-worker command `cargo test --locked --offline -p diagram-viewer --test native_lifetime should_measure_plantuml -- --nocapture` passed again: sequence 8/0, description 8/0, state 8/0 and activity 8/0, caches released (`native-final.log`). All test viewer instances auto-quit; final `pgrep -a -x diagram-viewer` found none. No IVE instance, user-profile changes, xdotool, focus activation or mouse input.

### Remaining boundaries / review handoff

All previously explicit subset exclusions remain. Ancestor-name declarations/references now reject where upstream contextual identity would cross the native ownership boundary; this is a stricter truthful subset, not cross-ownership support. Caption routing has measured clearance plus actual native capture evidence, not a universal font/glyph-bound guarantee. Geometry/pixel parity remains unclaimed. No future concurrency/history or other family work was added. This fix checkpoint is ready for the controller's scoped re-review, not self-approved for further implementation.
