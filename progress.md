# Diagram ports — progress and orchestration

Updated: 2026-10-23. Owner: primary controller agent.

This is the visible master checklist for further work. The controller updates it at implementation, verification, review and handoff milestones. Detailed working evidence remains in the internal SDD ledger linked below. Checked implementation is not automatically checked compatibility: family admission, semantics, geometry and pixel parity are separate claims.

## Current status

- The owner's implement → focused tests → fix/repeat loop has finished for the defined standalone port. Two scoped independent final reviews are complete; neither found a concrete Blocker or Important product/evidence issue.

- Mermaid and the standalone native pgui viewer are implemented and independently reviewed.
- Native PlantUML checkpoints through timing, Gantt, Salt, nwdiag and files/packetdiag/board/HCL are independently approved.
- Owner required LALRPOP to replace current PlantUML ad-hoc parsers. All17 migration stages are now integrated, with current structural syntax routed through generated parsers and obsolete line adapters removed.
- Chen/ArchiMate and the lexical prerequisite are implemented and included in final validation.
- Full Mermaid/PlantUML compatibility is **not complete**. All planned native family increments and the narrow state admission correction are integrated; final verification and scoped independent review passed. Evidence archive closeout and handoff are in progress.
- Delivery estimate: the defined standalone port is implemented and tested now; local handoff follows the final archive/tracker check in this session. Complete upstream syntax or pixel parity is not claimed.
- Main IVE product code is unchanged. Work is isolated in a separate Rust workspace/worktree; IVE markdown integration is a later project.

| Item | Current value |
| --- | --- |
| Product worktree / branch | `/mnt/data/Projects/ive/.worktree/diagram-ports` / `diagram-ports` |
| Standalone Rust workspace | `diagram-ports/` |
| Latest integrated product checkpoint | `3478bbd` — all planned native families, generated PlantUML parsers and narrow state admission correction. Fresh full gates, final95-source corpus,72-frame gallery and scoped independent reviews pass; later commits add docs/evidence only. |
| Final product review | At `ffd44a5`, independent product/parser/viewer review found no concrete Blocker or Important issue; evidence/provenance review likewise found none. Reports are being archived. |
| Current implementers | Final chronological Rulings/review archive closeout in an isolated worktree; controller owns this tracker and local handoff. |
| Architecture preparation | Complete; historical builder-check artifact is a 23-byte placeholder, explicitly labeled in the durable archive |
| Pending reviews | None for the defined standalone handoff; two final scoped reviews completed with no Important/Blocker finding. Their scope does not establish full upstream parity. |
| Next implementation | Complete archive/tracker closeout, then hand off the local worktree and explicit coverage limits. IVE Markdown integration and upstream parity remain later work. |
| Last complete controller gate | At final family head `3478bbd`:858 ordinary workspace Rust tests+seven actual worker faults, required all-targets859+seven, strict Clippy/fmt/release/cargo deny, generator tests/strict/fmt/deny,75 full tooling Node tests,29 reproducible parsers,293 MIT/58 fixture/58 native module/67 dependency/656 Merman audits and unchanged43 baseline all pass. Release viewer SHA `0c43488e42fca26229b00b9fbce13511726c8786c061cef8ce4827e04943bd2d`. |
| PlantUML provenance inventory | Current root audit passes293 selected MIT source/JAR matches,58 fixtures,58 native modules; older220/43/43 count belongs to the migration checkpoint |
| Mermaid vendor audit | 656 original imported files, four explicitly recorded patches |
| Latest native compatibility captures | Final post-state release gallery `/tmp/ive-gallery-final-20261023a/captures.json` SHA `5e371805d21b29fe088cec130171c4fe2773ff7ee537e1dc894b5af3309a306a`:72/72 frames. PlantUML35/35 positive native metrics/zero fallback; Mermaid20/37 positive native,9/37 zero fallback. All37 Mermaid frames byte-identical to inspected rehearsal; old34 PlantUML diagram canvases pixel-identical; new advanced-state frame independently viewed. Both languages' visible invalid-input panes exit1; empty-PATH/no-display failed exports preserve pre-existing output. No viewer survivor. |
| Current native corpus rehearsal | At pre-state head `eb4c5b7` plus tooling/progress, developer driver exported all91 current fixtures (37 Mermaid,54 PlantUML):91 rendered, zero failed, write-once manifest closure check passes; `/tmp/ive-native-corpus-prestate-20261023a/admission.json` SHA `b8e6d131d2745470927edc0852bd2b0b6ee238a05eb28889db40ea26e2855a32`; no viewer survivor. This is a native-export rehearsal, not final original/semantic/geometry/pixel evidence. |
| Current original corpus rehearsal | Fresh original run against the same91 source identities rendered36/37 Mermaid and54/54 PlantUML; ZenUML core returned `UnknownDiagramError`. Original272/native455 artifact closure and seven shared filename stems were verified. A fresh six-case repeat found normalized-only architecture Iconify ID drift, Mermaid Git random-label/0.7788849 height drift, and byte-identical Gantt/three PlantUML controls. Historical112 corpus files stayed unchanged. This remains pre-state evidence. |
| Current viewer-gallery rehearsal | Release binary captured all71 currently registered examples:37 Mermaid and34 PlantUML; `/tmp/ive-gallery-prestate-20261023a/captures.json` SHA `4b0d6673751b9c04956139f02b0ed35756684bd3a180fbbe4ab6dcbf77aefbbc`. All34 PlantUML examples report positive native metrics and zero fallback. Mermaid reports positive native metrics in20/37 and zero fallback in9/37; deterministic fallback is retained and must be described per family, not masked. All capture processes exited. This is a pre-state UI rehearsal, not a final visual/semantic/pixel verdict. |
| Final coherent corpus | At clean head `3478bbd`, 95 exact sources (37 Mermaid/58 PlantUML) with 16,592 source bytes; original94 rendered (36+58; ZenUML core `UnknownDiagramError`), native95/95. All95 identities and284 original/476 native/18 repeat artifacts close;37 fixed-label controls+24 state endpoints/exact pseudo graph/three region owners pass; no painted errors, native foreignObject/external href0. Manifests: original `ef56cdb8e603404fb3465a7383c8072e28b7aca08414ad8bc65830574ece9742`, native `b8f66998f7728e803d8bc5560ae25b43d2ba59e759685c739f1e3d4180e67462`, repeat `43c943ea70a52f35c21d540d2c04d2ad19938039d3d35528365f03a2ba5f4f2b`. Raw architecture Iconify IDs and Mermaid Git labels/geometry vary on repeat; Gantt date/container caveats remain. Final structured evidence root `/tmp/ive-diagram-final-poststate.MH3MN8`, durable copy underway. |

## Completed work

### Project, design and source boundaries

- [x] Read the handoff, IVE instructions and existing APIs; prepare Mermaid/PlantUML analysis and an implementation design/plan.
- [x] Confirm scope with owner: try full ports; additional dependencies allowed; standalone viewer first.
- [x] Create an isolated local branch/worktree and an independent Rust workspace with engine and viewer crates.
- [x] Pin pgui to IVE's revision and keep the engine GPUI-independent.
- [x] Audit existing Rust candidates. Reject the PlantUML candidate with unresolved source-lineage evidence before importing its code.
- [x] Translate PlantUML only from the authorized MIT 1.2026.9 source tree; record exact files, hashes, destinations, behavior and actual credits/notices.
- [x] Exclude GPL/EPL/ELK/Smetana implementations and untraced assets. No runtime Java, JavaScript, browser, dot or remote rendering.

### Mermaid

- [x] Integrate published Merman 0.9.2 as the audited native baseline, with explicit baseline target Mermaid 12.1 and known 12.2 gaps.
- [x] Render 37 admitted variants with labelled, positive-size safe SVG fixtures and usvg parsing.
- [x] Cover agentflow, architecture, block, C4, class, Cynefin, ER, eventmodeling, flowchart, Gantt, git graph, info, Ishikawa, journey, Kanban, mindmap, packet, pie, quadrant, radar, railroad IR/EBNF/ABNF/PEG, requirement, Sankey, sequence, state, swimlane, timeline, tree view, treemap, usecase, Venn, Wardley, XY chart and ZenUML.
- [x] Replace blind resource-token filtering with semantic resource/CSS/SVG admission; preserve harmless URL-looking labels/comments.
- [x] Add bounded source/model/depth/work/SVG limits, cancellation and deterministic render configuration.
- [x] Implement released 12.2 mindmap depth coloring and coherent usecase/tree measure/paint font declarations.
- [x] Preserve audited vendor baselines, patch ledgers, legal texts and tamper tests.
- [x] Verify actual authored math requests fail explicitly when the unavailable math capability is needed.

### Standalone pgui viewer and isolation

- [x] Native full-color SVG display, read-only source, language/example selection, fit, 25–400% logical zoom and keyboard controls.
- [x] Selected-file input and headless SVG export; failed export preserves existing output.
- [x] Background/no-focus real app-frame capture with automatic quit, including visible error captures.
- [x] Off-UI-thread service: one active job, replaceable pending job, cancellation and stale-result rejection.
- [x] Isolated same-binary worker, bounded framed protocol, independent ten-second watchdog, kill/reap, stderr/output/query limits and Unix 2 GiB address-space ceiling.
- [x] Exercise seven real worker-fault scenarios: blocked stdin, blocked stdout, stderr flood, oversized frame, early exit, memory and cancellation.
- [x] Per-render native text caches and actual installed-font advance/line measurements; unsupported bbox/wrapping operations report fallback honestly.
- [x] Separate logical zoom geometry from capped raster resolution.
- [x] Same-tree native paint containment for safe text overflow, including tree `package.json`; preserve intentional clipping/effects outside that safe subset.
- [x] Bound XML/tree/path/bitmap/raster work and reject unsupported dangerous filter-image paths before rasterization.
- [x] Inspect actual native captures; stop test viewer processes. No user IVE/focus/mouse/state manipulation.

### Approved native PlantUML model/render checkpoints

| Checkpoint | Done | Reviewed head |
| --- | --- | --- |
| 3A foundation | Strict located source/lexer, shared budgets, typed measured text/scene/SVG, safe terminal output | `d07df14` |
| 3A sequence | Participants/aliases, messages/self calls, notes, interaction frames, balanced lifecycle/activation | `d07df14` |
| 3A UML/description | Class/object members, packages/namespaces, relationships/multiplicities, actor/usecase/component/deployment-style shapes and named notes | `d07df14` |
| 3B state | Scoped initial/final nodes, ordinary/composite states, forward references, descriptions and recursive graph layout | `49838e8` |
| 3B activity | Real action/decision/loop/fork/group tiles with live/dead ports and correct continuation | `49838e8` |
| 3C trees | Dedicated mindmap sides/parent chains and WBS hierarchy/buses/aliases | `6ec1022` |
| 3C data | Ordered typed JSON and PlantUML-specific YAML semantics; duplicate policy, nested compartments and correct owning-row links | `6ec1022` |
| 3D styles/text | Finite skinparams/stylesheets, title/header/footer/legend, measured Creole runs and font/style-aware caching/painting | `71cd817` |
| 3D2 constants | Bounded immutable eager literal `!define`, no inserted-text rescan, compact origins, truthful bounded diagnostic candidates; comment/raw ownership and expansion-terminator fixes reviewed | `6abc18f` |
| 3E timing | Exact bounded integer times, separate player/time cursors, robust steps/concise ribbons, constraints/messages, sparse axis and measured reservations | `b3cef28` |
| 3F Gantt | Explicit civil dates, seven-open-day calendar, half-open bars, aliases/dependencies, milestone offsets, documented freeze-after-reference restriction | `d84c314` |
| 3G Salt | Typed nested tables/cells/spans, direct escapes, real fields/buttons/checks/radios and grid geometry; late-chrome/text boundary fixes reviewed | `f6fa5d9` |
| 3H nwdiag | Global servers, distinct interface memberships/addresses, sparse buses with foreign-crossing gaps and measured groups | `85887fa` |
| 3I small inventory | Ordered files/notes, conserved packet fragments/bit geometry, stage/branch board, ordered string HCL/owning compartments; header-boundary fix reviewed | `3231bb1` |
| 3J Chen/icon-free ArchiMate | Dedicated Chen ownership/shapes/intersections and colored DESCRIPTION; relationship/comment fixes independently reviewed | `4c19787` |

Each row is a supported subset, not complete upstream grammar or Graphviz/browser/pixel parity. Existing handwritten grammar implementations are scheduled for LALRPOP replacement; historical approvals do not mean the requested parser rewrite is complete.

## Completed Salt gate

- [x] Exact Salt wrapper and raw lexical ownership; direct escaped pipe/braces without PUA sentinels.
- [x] Typed immutable widget/table/cell arena, stable ownership, same-row colspan and sparse bottom-up layout.
- [x] Genuine fields/buttons/checkboxes/radios and nested/no-border/outer/full-grid tables.
- [x] Fifteen public Salt tests and six direct model/geometry tests; native paint at 6/14/96px and actual callback cancellation.
- [x] Exact source/fixture/notice audit, final full suite and actual native capture.
- [x] Controller independently repeat full checks, source matching, capture inspection/hash and empty-PATH/no-display export.
- [x] Read the completed independent review report: one Important and one Minor finding.
- [x] Implement the late document-command repair: colon/aligned commands inside tables reject through shared grammar-aware recognition.
- [x] Implement the tokenizer boundary repair: ordinary `{Text}` is accepted; genuine tree markers remain rejected.
- [x] Add meaningful RED/GREEN regressions, rerun checks and commit the scoped repair at `f6fa5d9`.
- [x] Independently re-review findings/fix diff: both findings ADDRESSED, no new breakage.
- [x] Mark Salt approved; nwdiag is ready for dispatch.

Review: `/tmp/ive-plantuml-salt-review.md`. Exact reproductions: `/tmp/ive-salt-review-late-chrome.puml` and `/tmp/ive-salt-review-tword.puml`. Fix round 1 is scoped to these findings, not unrelated deferred cleanup.

Final Salt milestone: meaningful RED16/18 → shared grammar17/18 → T repair18/18. Controller fresh388 Rust+seven faults/all checks pass; the native PNG is byte-identical. Full `/tmp/ive-plantuml-salt-rereview-1.md` read: both findings ADDRESSED, SPEC/QUALITY approved, no new breakage. Salt is complete for its documented subset.

## Completed nwdiag gate

- [x] Establish meaningful failing tests before implementation: seven expected failures and three negative controls passing.
- [x] Implement typed stable network/server/group IDs and memberships; one global node per shared server.
- [x] Implement separate network CIDR/interface addresses, repeated-membership updates and group ownership.
- [x] Implement consumed property grammar and explicit unsupported/resource errors.
- [x] Implement sparse bus layout, measured labels/enclosures and crossing gaps that avoid false memberships.
- [x] Semantic topology, geometry, bounds, cancellation and native paint tests.
- [x] Authorized source/fixture/notice audit and honest viewer example.
- [x] Full final checks, actual background capture, scoped commit and report.
- [x] Controller fresh verification and independent SPEC/QUALITY review; no blocking findings.
- [x] Approve nwdiag; Task 3I ready for release.

Latest nwdiag milestone: approved85887fa after full independent review with no Critical/Important blockers. Meaningful RED/GREEN and final16 public+7 direct model/geometry/bounds tests; native paint6/14/96px. Controller independently confirms413 Rust+seven worker faults, strict checks/audits/thirteen Node pass. Actual capture at /tmp/ive-plantuml-3h.SIaRmg/nwdiag.png is controller-inspected/hash-verified,21native/zero fallback; no surviving viewer processes. Controller empty-PATH/no-display export succeeds with all nine expected labels (12,538bytes). One keyword/whitespace Minor is explicitly deferred below.

Controller source checkpoint: all ten newly selected nwdiag files match the authorized extracted MIT source and exact source-JAR entries, recorded SHA-256 and actual copyright/author/contributor headers. Final source/fixture/module audit also passes184/31/34.

## Completed Task 3I gate

- [x] Release a fresh sole implementer from approved nwdiag85887fa.
- [x] Read requirements, shared contracts and prepared source-semantic research; inspect actual authorized source implementations.
- [x] Establish meaningful failing tests: twelve public cases, eight expected behavior/admission failures and four negative controls passing.
- [x] Initial Files implementation: ordered identity-preserving trie, files/folders/notes, measured rows and native symbols.
- [x] Initial Packetdiag implementation: inclusive bit ranges, conserved fragments, bit scale/ticks and admitted configuration.
- [x] Initial Board implementation: stage/branch identities, nearest-lower parent, DFS columns, cards and no duplicate root paint.
- [x] Initial HCL implementation: PlantUML-specific ordered blocks/fields and genuine data-compartment ownership, no evaluation.
- [x] Select exact source notices/fixtures; controller independently matches22 new MIT sources/credits and four byte-identical reference inputs.
- [x] Implementer focused model/geometry/bounds and native ink checks pass; four viewer examples are registered.
- [x] Four genuine background viewer captures, controller inspected/hash-verified, zero fallbacks and no surviving viewer processes.
- [x] Implementer final checks:450 Rust tests, seven actual worker faults, strict Clippy/fmt/release/deny/audits and fourteen Node tests.
- [x] Scoped commit1bf1f46, complete report and clean tracked tree.
- [x] Controller fresh full checks450+seven faults/all gates, four empty-PATH/no-display exports, source matching and capture inspection.
- [x] Fresh independent SPEC/QUALITY review: one Important HCL boundary finding; no other findings.
- [x] Scoped HCL fix3231bb1 with meaningful RED/GREEN, preserved valid block names, fresh455-test checks/commit/report.
- [x] Independent scoped re-review: I1 addressed and no fix-introduced breakage.
- [x] Approve Task 3I; Chen/icon-free ArchiMate ready for release.

Completion: independently approved3231bb1 after one scoped HCL fix. Full initial review and /tmp/ive-plantuml-inventory-3i-rereview-1.md read: I1 ADDRESSED, SPEC/QUALITY approved, no new breakage or other findings.

Evidence: controller fresh455 Rust tests +seven actual worker faults, strict Clippy/fmt/release/deny/audits and fourteen Node tests pass. All22 new MIT source/JAR/hash/credit matches and four exact reference fixtures verified; inventory206 sources/39 fixtures/41 modules. Four genuine captures independently inspected/hash-verified: files12/packet9/board8/HCL21 native measurements, all zero fallback; no surviving viewers. HCL fix capture is byte-identical. Four empty-PATH/no-display exports succeed.

Semantics: ordered file trie and sibling notes; inclusive packet fragments conserving5/8/7 with real bit geometry; stage/branch board with single root paint; ordered string HCL with genuine owning-row compartments. Bounds/pre-retention/depth and actual callback cancellation tests cover all four.

TDD: initial8 expected failures/four controls →12 public GREEN, strengthened to22 before review. HCL fix meaningful23/27 →27/27, preserving valid command-named blocks, split braces and multiline values while rejecting cross-line header-token swallowing. The original agent recovered its WIP after a server restart; no completed task was redispatched.

Boundaries: packet full-label scale/max-height row policy deliberately differs from upstream. HCL duplicate topblocks, shared styles/chrome and evaluation explicitly reject; constants remain admitted. Header-name tokens stay on one physical line; opening braces may follow later. Broader grammar, original symbols/themes/routes and pixel parity are not claimed. Full evidence: task-3i-report.md and /tmp/ive-plantuml-3i.SWV6Rh/ plus fix1.HBrstU/.

## Completed priority — current PlantUML LALRPOP rewrite

- [x] Owner explicitly directs full current parser replacement, superseding the earlier pilot-only recommendation.
- [x] Hold new-family implementation and dispatch read-only migration architecture preparation.
- [x] Resolve the load-bearing shared comment-tokenization regression N1; independent fix2 review passes.
- [x] Inventory current syntax and commit design/17-task plan after independent builder check.
- [x] Add pinned, reproducible LALRPOP generation, bounded spanned token/AST/error/budget/cancellation core and real definition-syntax cutover.
- [x] Migrate sequence/UML-description/Chen, state/activity and tree/data/HCL grammars to real generated productions.
- [x] Migrate timing/Gantt, Salt/nwdiag and files/packet/board grammars.
- [x] Migrate applicable shared syntax parsing (preambles/styles/supported directives/markup), explicitly separating lexical and semantic work.
- [x] Remove superseded ad-hoc syntax dispatch; no generated Line* wrapper forwarding to legacy parsers, no runtime legacy fallback after cutover.
- [x] Verify admitted grammar, identity/geometry/source-location/bounds, unchanged baseline and genuine native captures for the current documented subset.
- [ ] Perform broad independent review after the remaining port implementation.
- [ ] Resume remaining family queue using LALRPOP, then final evidence/whole-branch review.

Implemented approach: generated grammar cutovers retain existing models/layouts. Custom bounded lexing/preprocessing remains for comments/raw modes/macros and primitive lexical ownership; generated productions own current structural syntax. New family syntax still needs its own LALRPOP grammar.

### LALRPOP cutover tracking

| Task | Component | State |
| --- | --- | --- |
| 1 | Pinned generation/baseline/bounded core/real definition syntax | DONE — af042d7; all three findings addressed, independent SPEC/QUALITY approved |
| 2 | Document wrappers/lexical ownership/source-span error bridge | DONE —92c9520; both header findings addressed, independent scoped approval; Gantt EOF defect resolved |
| 3 | Preambles/styles/chrome syntax | DONE —f5a10a0; both Important findings addressed, independent scoped approval; root544/all16 gates/native/public probes pass |
| 4 | Sequence | DONE —1506eb1 integrated0c5177e; both compatibility fixes reviewed, full combined gates/native pass |
| 5 | UML/description/ArchiMate/names/arrows | INTEGRATED8a58a71 from tested0c2ee0e —362 UML-only focused tests pass; broad review deferred |
| 6 | Chen/contextual keyword repair | INTEGRATED435d73c from testedc465f21 —454 combined focused tests/private generator14/source audit, root full/baseline/native checks pass; broad review deferred |
| 7 | State | DONE —ec60840 integrated0c5177e; adjacency fix reviewed, full combined gates/native pass |
| 8 | Structured activity | INTEGRATEDe9c1354 from tested1b0c234 — combined focused suite375, strict Clippy/fmt and private generator11 pass; root full serial workspace tests and seven worker faults pass; broad review deferred |
| 9 | Mindmap/WBS/files/board | DONE —4a31cb7 integrated0c5177e; syntax/label limit fix reviewed, full combined gates/native pass |
| 10 | JSON/HCL | INTEGRATED435d73c from tested53dbad8 —454 combined focused tests/private generator14/source audit, root full/baseline/native checks pass; depth64 positive/depth65 negative and exact retained-byte controls pass; broad review deferred |
| 11 | PlantUML YAML | INTEGRATED3d3c35a from tested349ccb5 —517 combined focused tests, strict Clippy/fmt, private generator17 and source audit; root full/baseline/native checks pass; broad review deferred |
| 12 | Timing/Gantt/assigned defects | INTEGRATED6664c86 from testedf5f07f9 —505 combined focused tests, strict Clippy/fmt, private generator16 and source audit; root full/baseline/native checks pass at later3d3c35a head; broad review deferred |
| 13 | Salt | INTEGRATED86593cf from tested0b9f4a6 —507 combined focused tests, strict Clippy/fmt, private generator18 and source audit; root full/baseline/native checks pass at latere4ace61 head; broad review deferred |
| 14 | nwdiag/contextual server-name repair | INTEGRATEDe4ace61 from testeddc29c48 —489 combined focused tests, strict Clippy/fmt, private generator19 and source audit; root full/baseline/native checks pass; broad review deferred |
| 15 | Packetdiag | INTEGRATEDda590bc from testeda08bf48 —593 combined focused tests, strict Clippy/fmt, private generator21 and source audit; root full/baseline/native checks pass; broad review deferred |
| 16 | Creole nesting | INTEGRATEDa796711 from testedd398ee6 —577 combined focused tests, strict Clippy/fmt, private generator20 and source audit; root full/baseline/native checks pass at laterda590bc head; broad review deferred |
| 17 | Legacy-syntax removal/inventory/integration review | VERIFIED0f468eb from testedf714769/toolingdd7c26f —root core/audit gates, unchanged43 baseline and17 native family captures/exports pass; Line adapters removed,21 grammars/zero pending syntax; broad review deferred until port end |

Tasks1–3 are approved throughf5a10a0 after one scoped fix round each. Generated definitions, wrappers, shared ownership, typed headers, origin/body spans and preamble/style/skinparam/region grammar are active. That checkpoint passed544 Rust+seven faults/all16 gates,18Node,generator3/notice5/67-package audit/four reproducible outputs and unchanged43-fixture baseline SHA a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe. Empty-value errors are typed; repeated500 sheets pass with/without definitions and identical SVGs. Actual styled/constants20/22/12/14 and font6/14/96 each8 native,all zero fallback, are inspected/hash-verified; exports/cleanup pass. The table above tracks subsequent cutover reviews and pending combined verification.

### Implementation loop now in use

- Owner direction (2026-10-23): continue implementation without per-stage independent review. The broad review moves to the end. Cost if wrong: defects may accumulate across grammar stages, so each checkpoint needs focused tests, exact failing cases, and a reproducible commit before integration.
- Independent family ports use separate worktrees. A designated integration writer combines tested branches in `diagram-ports`; the controller runs shared gates on combined batches.
- Workers implement, run focused regression tests, repair failures and commit coherent checkpoints. Broad review happens at the end; native captures and baseline exports run centrally at meaningful integration milestones.
- Cargo commands share an outer build lock to avoid competing with the existing500ms test. Generator binaries use a separate target directory per checkout: their embedded checkout paths caused cross-worktree output generation when a shared executable was reused. The private targets fix that hazard; no source baseline or assertion was weakened.
- The previous10–18hour estimate described serial parser migration and is superseded. The current estimate for remaining family increments is recorded in Current status above.
- Read-only next-family preparation is complete for [Task 3K EBNF/regex](.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3k-preparation.md) and [Task 3L charts](.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3l-preparation.md). Both can now start product implementation from the verified parser checkpoint.
- New family fixtures exposed a baseline-driver scope issue; Task3M repaired `check` to select the 43 recorded baseline paths while all added fixtures remain under provenance/source audit. Combined central baseline check passes and the frozen file stays byte-identical.

## Completed prerequisite repair gate — Task 3J

- [x] Release a fresh sole implementer from approved3231bb1.
- [x] Read binding brief/shared contracts/prepared semantics and inspect actual authorized sources.
- [x] Establish initial meaningful RED: three expected missing-feature failures and three passing rejection/limit controls.
- [x] Initial Chen: typed entity/relationship/attribute identities, lexical ownership, cardinalities and true shapes/intersections.
- [x] Initial icon-free ArchiMate: colored DESCRIPTION extension, aliases/links and icon rejection.
- [x] Initial semantic/geometry/bounds RED/GREEN and actual native ink checks6/14/96px.
- [x] Finalized14 new source selections/notices and two exact reference fixtures independently matched; honest examples registered.
- [x] Two actual background captures with native metrics, controller inspection/hash verification and no surviving viewer processes.
- [x] Full strict checks, scoped commit35c108d/report/clean tracked tree and controller fresh verification.
- [x] Fresh independent SPEC/QUALITY review: two Important blockers and one deferred Minor.
- [x] Fix I1 description relationship dispatch and I2 inert Chen comment ownership, with meaningful RED/GREEN/fresh482-test checks/commit40b949a/report.
- [x] Independent fix1 re-review: I1/I2 addressed; found new Important N1 delimiter-byte overlap.
- [x] Scoped fix2 frozen4c19787: atomic delimiter steps, meaningful RED/GREEN, exact origin invariant, fresh488-test checks/commit/report.
- [x] Independent fix2 re-review confirms N1 addressed and no new breakage.
- [x] Approve Task3J baseline for LALRPOP migration; EBNF/regex remains held.

Completion: approved4c19787 after two scoped fix rounds. Full initial review/FIX1/FIX2 reports and final independent verdict read: I1/I2/N1 ADDRESSED, SPEC/QUALITY PASS, no new breakage. M1 remains explicit for migrationTask6.

Evidence: controller fresh488 Rust+seven actual faults/all strict checks/audits/fifteen Node pass; source220/43/43 and fourteen exact MIT/JAR/credit matches plus two original fixtures verified. Original Chen/ArchiMate captures11/0 and6/0, repaired alias/comment captures2/0 and1/0, and atomic-delimiter capture2/0 independently viewed/hash-verified; no survivors. Exact native headless repros preserve identities/links/active text. Full evidence remains in task-3j-report.md and /tmp/ive-plantuml-3j{,-fix1,-fix2}/.

Supported subset: lexical Chen owners, composite/relationship attributes, true rectangle/diamond/ellipse intersections and undirected cardinalities; icon-free safely colored DESCRIPTION extension. Five Chen decorations/subtypes, ArchiMate icons/package/multiline and broad named colors remain unsupported. Model/render approval is not full syntax/pixel parity or completion of the newly mandated parser rewrite.

Parser directive: replace all current PlantUML ad-hoc syntax with real LALRPOP productions, including shared syntax/data/Creole. Models/layouts stay. Tasks1–3 are approved, sequence Task4 is review-approved pending combined verification, and the live cutover table above tracks subsequent work.

## Remaining implementation queue — in order

Every increment requires source inspection/provenance, meaningful TDD, typed semantics and geometry, bounds/cancellation tests, a real viewer example/capture and controller verification. Broad independent review follows the assembled port under the owner's current direction.

| Order | Task | Remaining deliverable |
| --- | --- | --- |
| 1 | Full current PlantUML LALRPOP migration — DONE | All17 stages verified at `0f468eb`;21 generated parsers, zero pending syntax roles |
| 2 | 3K EBNF / regex — INTEGRATEDd3baf26 | 761 Rust+seven worker faults, strict Clippy/fmt, private generator24 and source252MIT/46fixtures/48modules+23audit/closure67 pass; document union37terms/65states; root native EBNF4/regex9 zero-fallback captures/empty-PATH exports pass; immutable43 baseline selector later repaired in 3M |
| 3 | 3L charts — INTEGRATED3c90bb1 | 606 engine/public tests, viewer example, strict Clippy/fmt, private generator22, source232MIT/44fixtures/45modules+14audit and closure67 pass; root chart native14/zero-fallback capture and headless export pass; immutable43 baseline selector later repaired in 3M |
| 4 | 3M ASCII Git — INTEGRATED5a2f1ec | 549 focused Rust+11 viewer+29 Node tests, strict engine/generator Clippy/fmt, private generator26, source264MIT/49fixtures/52modules/closure67 and immutable43 baseline pass. Exact padded original merge verifies four endpoint-mapped edges; native unpadded robustness documented, `*-.` rejected. Root native4/zero-fallback capture visually checked, empty-PATH SVG3266 bytes; full combined gate pending |
| 5 | 3N wire — INTEGRATED819cd9e | 674 focused Rust+12 viewer+21 Node tests, strict Clippy/fmt, private generator27, source275MIT/51fixtures/54modules/closure67 and immutable43 baseline pass; combined document43terms/74states. Root native7/zero-fallback capture visually checked, empty-PATH SVG3740 bytes; full combined gate pending |
| 6 | 3O legacy BPM — INTEGRATED3071513 | 777 Rust+seven worker faults, strict Clippy/fmt, private generator25, source261MIT/47fixtures/50modules+23audit and closure67 pass; generated graph preserves split/merge/continuation; root native4/zero-fallback capture/headless export passes; immutable43 baseline selector later repaired in 3M |
| 7 | 3P directional flow — INTEGRATED8ebb34e | Combined828 Rust+seven worker faults,30 Node, strict/generator/source/closure,28 parsers and unchanged43 baseline pass; native Flow/cursor3/3 zero-fallback captures and empty-PATH exports pass |
| 8 | 3Q standalone Creole — INTEGRATED465b9c8; PAINT FIXeb4c5b7 | Combined595 focused Rust+13 viewer+30 Node, strict/generator/source/closure,29 parsers and unchanged43 baseline pass. Real capture exposed serif measurement/paint mismatch; scoped correction passed568 focused+three actual glyph tests+30 Node/strict/source audits. Root release recapture native7/zero fallback and empty-PATH SVG passes, styled runs visibly separated |
| 9 | 3B2 advanced state — INTEGRATED95908d1; FIX3478bbd | Combined717 focused Rust+14 viewer+75 Node, strict/generator29/closure67/source293/58/58/Merman656 and frozen43 baseline pass. Root inspected concurrent/history/pseudostate native9/zero-fallback capture. Unquoted `state history*` admission fixed with focused RED/GREEN while quoted identity/stereotype stay positive; full final gates and95-source corpus pass |
| 10 | 4B final evidence/tooling/docs — CLOSEOUT | Authenticated import/fresh-oracle/native corpus driver/deadline fix, bounded COVERAGE/README,114-payload historical archive and17-payload current addendum integrated. Full gates,95-source corpus,72-frame gallery and two visible error captures pass. Two scoped independent final reviews found no Important/Blocker. Chronological Rulings/review archive and local handoff are closing. |

Existing inventory briefs/source-semantic research are prepared and read; the parser migration and planned standalone implementation are complete. Do not restart completed source research or redispatch prior model/render work. New family work requires LALRPOP and current generated interfaces.

## Known review follow-ups

- [x] Timing: robust/concise contextual track names corrected in Task12 generated parser with positive/negative regressions.
- [x] Gantt: reject an unterminated block comment attached after `@endgantt`. Generated document cutover92c9520 resolves it with all-wrapper EOF and closed-comment controls; independently reviewed.
- [x] Nwdiag: title-with-spaces/tab contextual server names corrected in Task14 generated parser; genuine late chrome still rejects.
- [x] Chen: entity/relationship alias endpoints and spaced/compact composite attributes corrected in Task6 generated parser with exact scoped ownership tests.
- [x] Triage existing coarse source columns, YAML nested-flow line1/missing-close diagnostics and large parser/layout units without unrelated rewrites; record them as residuals in COVERAGE.
- [x] Record inherited vendored Atomic deprecation in COVERAGE; preserve the audited vendor baseline.
- [x] Migration Task17: baseline driver now reports actual exporter defaults, with a drift contract; immutable43 outputs remain unchanged.
- [x] Migration Task17: generator takes an explicit validated root; same-binary two-checkout/wrong-root regressions pass.
- [x] Record development test-support's second inherited Merman Atomic deprecation in COVERAGE and preserve vendor audit policy. Review: `/tmp/ive-plantuml-lalrpop-core-review.md`.
- [x] Record Task4's 500ms action-test load sensitivity, concurrent-build failure and serial passing controls/full workspace in COVERAGE; preserve its assertion/deadline.

## Final evidence, audits and handoff still required

- [x] Build a fresh coherent original/native corpus with source/SVG hashes, tool versions, configs, selected labels/topology/geometry and honest nondeterminism. Do not reuse the historically overwritten oracle manifest as final proof.
- [x] Fix developer oracle output-directory handling so explicit fresh output paths are respected and existing artifacts remain untouched.
- [x] Harden Merman import initialization to compare extracted files with authenticated archive members before recording a baseline; add mismatch/tamper tests without regenerating/loosening existing hashes.
- [x] Produce durable `COVERAGE.md` and structured corpus results distinguishing syntax admission, semantics, geometry and actual pixel comparisons.
- [x] Update README, analyses, survey, parent/native plan state and source/license notices to final standalone reality, including the rejected candidate and deliberate deviations. Historical plan task boxes remain as original instructions; the final checkpoint is linked from the plan.
- [ ] Archive final independent review reports and chronological rulings; update this file's links. Preserve the master checklist and durable SDD evidence.
- [x] Fresh final tests, Clippy/fmt/release, license/feature/source audits, headless empty-PATH exports, negative cases, both-language native captures and no surviving test processes.
- [x] Fresh scoped independent whole-branch product and evidence reviews found no concrete Blocker or Important issue; no fix wave was required. Scope limits are recorded in the reports.
- [ ] Keep local branch/worktree/commits; no unrequested merge/push/publication. Read finishing workflow and provide launch/coverage/evidence handoff.
- [ ] Preserve all recorded controller rulings (twelve: ten parent, one parallel acceleration, one review deferral), chronologically with cost-if-wrong, in final handoff archive.

## Compatibility work not completed by family admission

These remain explicit gaps, not hidden success claims. Final coverage must refine them from actual results:

- Mermaid: complete 12.2 syntax/theme/layout parity beyond the audited patches; default ELK equivalence; browser bbox/wrapping/offset metrics; math/typesetting/assets; remaining geometry/font differences.
- PlantUML: full sequence/Teoz features and participant/arrow/activation/reference variants; broader UML/description/state/activity grammar, ports/routing/styles; broader tree/data grammar; full timing/calendar/resource/natural-language Gantt; advanced Salt widgets/borders/regions.
- Shared PlantUML: full TIM/macros/functions/conditionals/loops, complete styling/theming/Creole, named colors and separately licensed icons/sprites/fonts/math assets. External files/network/includes remain denied unless a separately designed caller-memory resource API is implemented.
- Rendering fidelity: original Graphviz/Teoz/theme/symbol/route geometry, exact ink/baseline/browser metrics, general collision avoidance and cross-platform font equivalence are not established.
- Host limits: hard worker limits are not an OS filesystem/network sandbox; host parsing/font/raster stages have bounded admission/cooperative checks, not a separate hard killable deadline. macOS enforcement remains unverified; non-Unix has no OS address-space ceiling.
- Formula/LaTeX reference fallbacks are not typesetting support. Ditaa/Chronology registration exclusions are inventory facts, not implemented families. Other languages and IVE markdown/toggle/canvas integration are outside this standalone-first stage.

## Orchestration rules

1. Controller owns this file and the evidence ledger; update both at each real milestone. Give agents a pointer here, but keep report/diff details in their scoped handoffs.
2. One product implementer per isolated worktree; independent family tasks run concurrently. One central integration writer; use focused tests and combined gates as the implementation loop.
3. Mark tested implementation milestones separately from final compatibility approval. Defer broad review findings and final triage until the assembled port is ready.
4. Preserve source/license/runtime-I/O restrictions, public APIs, existing regressions, immutable typed models and shared resource budgets.
5. Continue the ordered queue after the current gate; do not stop at a status report, replan already completed work or claim full parity from family counts.

## Useful locations and running

- [Standalone README](diagram-ports/README.md)
- [Implementation design](docs/superpowers/specs/2026-10-22-diagram-ports-implementation-design.md)
- [Native staged plan](docs/superpowers/plans/2026-10-23-plantuml-native-plan.md)
- [Detailed controller ledger](.superpowers/sdd/2026-10-22-diagram-ports-plan/progress.md)
- [Current Salt report](.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3g-report.md)
- [Latest approved nwdiag report](.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3h-report.md)
- [Approved inventory report including HCL fix](.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3i-report.md)
- [Approved Chen/ArchiMate report including fixes](.superpowers/sdd/2026-10-22-diagram-ports-plan/task-3j-report.md)
- [LALRPOP migration design](docs/superpowers/specs/2026-10-23-plantuml-lalrpop-design.md)
- [LALRPOP migration plan](docs/superpowers/plans/2026-10-23-plantuml-lalrpop-plan.md)
- [Active migration ledger](.superpowers/sdd/2026-10-23-plantuml-lalrpop-plan/progress.md)

```sh
cd /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo run --locked --offline -p diagram-viewer
# Or select a built-in explicitly:
CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo run --locked --offline -p diagram-viewer -- --language plantuml --example plantuml-nwdiag
```
