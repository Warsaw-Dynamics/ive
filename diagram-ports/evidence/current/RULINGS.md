# Diagram ports — chronological directions and rulings

Prepared 2026-10-23 for the controller's final evidence index at integrated head `ffd44a5`. Product behavior and final execution evidence are at `3478bbdf5d11865414817e5f6a01eb81cd2336da`; the current evidence addendum is integrated at `8d41271`, and final COVERAGE/README updates are integrated at `ffd44a5`. Both scoped independent final reviews of `ffd44a569850d05e0359bff8ea6267982342c6fc` are complete with no concrete Important or Blocker finding; no fix wave is required. This archive closeout retains the Rulings and exact review reports; central integration and local handoff remain controller-owned. The consolidation itself is not an independent product review or full-upstream acceptance. Links are relative to the intended durable location, `diagram-ports/evidence/current/RULINGS.md`.

**Attribution:** an **owner directive** below is explicitly attributed to the owner in a source specification/ledger; where only the ledger survives, that attribution is reported rather than presented as a verbatim chat transcript. A **controller ruling** is an implementation decision made on the owner's behalf. A **recommendation** remains a recommendation unless subsequently adopted. Dates are recorded document/event dates; commit timestamps identify checkpoints, not invented decision-message times. Untimestamped decisions retain their recorded relative order.

**Current authority:** the live master is [progress.md][master], inside the `diagram-ports` worktree. The original `/mnt/data/Projects/ive/progress.md` path no longer exists. Both controller ledgers explicitly record moving the master out of the user's main checkout at the owner's request. Do not recreate a second competing checklist. The old [eleven-ruling handoff][old-rulings] and [archived ledgers][catalog] are historical; the current master requires **twelve** rulings: ten parent, one parallel acceleration, one review deferral. Keep old payloads unchanged and add this as a new dated record.

## Controller digest

- **Owner scope:** analysis-only on2026-10-22 was superseded by explicit full-port attempts, extra suitable dependencies and standalone pgui-first implementation. IVE integration/push/publication remain outside this local task.
- **Owner source/runtime boundary:** only authorized MIT PlantUML translation; reject unresolved puml lineage and EPL/ELK/Smetana implementation imports. No runtime Java/JS/browser/dot/network/resolver; original tools are developer oracles. Hash matches prove local identity, not external authentication.
- **Changed parser approach:** the controller's future-family-only LALRPOP pilot recommendation was superseded by the owner's explicit rewrite of **all current** PlantUML structural syntax. No Line* forwarding or legacy runtime fallback. Models/layouts preserved; all17 cutovers and subsequent29 grammars are integrated.
- **Changed delivery approach:** inventory3C–3Q precedes advanced state3B2; parser migration was inserted ahead of remaining breadth. Owner-requested acceleration led to isolated worktrees/one central writer. Weekly-limit handoff froze work; later owner resume replaced per-stage broad review with implement/test/fix and **broad independent review at the end**.
- **Location:** live master `/mnt/data/Projects/ive/.worktree/diagram-ports/progress.md`; standalone workspace is its `diagram-ports/` subdirectory. Historical main-checkout progress paths and old child checkboxes are not current authority.
- **Current result:** product3478bbd has recorded green final full gates,95-source original/native corpus and72-frame gallery. These establish their stated checks, not full upstream syntax/geometry/pixel parity. Native fallbacks, ZenUML plugin/reference limits and reference nondeterminism remain explicit.
- **Final review and remaining handoff:** current evidence is integrated at8d41271 and final COVERAGE/README atffd44a5. Both [product review](reviews/ive-final-whole-branch-review.md) and [evidence review](reviews/ive-final-evidence-review.md) completed on that reviewed head with no concrete Important or Blocker; no fix wave is required. Their scopes and runtime/platform/compatibility limits remain binding. The controller owns central archive integration and the residual/platform and local launch/evidence handoff. Do not reopen completed tooling/corpus work because an old checkbox remains unchecked.

The longer sections retain exact ruling/cost wording and dated sources for archival use.

## 1. Owner directions and supersession chronology

| Date / recorded position | Direction and attribution | What it supersedes / what remains binding | Sources |
| --- | --- | --- | --- |
| 2026-10-22 — original handoff | **Owner: analysis and planning only**, with documents reviewed before port code. Target native, idiomatic Rust; preserve logic/algorithms/constants/edge cases and source traces rather than Java/JS class shapes. | Initial authority did not permit product implementation. Later explicit implementation direction superseded that hold, while technical/licensing constraints remained. | [Original handoff][handoff] |
| 2026-10-22 — same handoff | **Owner: built in, offline runtime.** No JVM/Node/browser/JAR/mmdc/dot/remote renderer at runtime; no network or filesystem access beyond caller-supplied data. GPUI-free engines, SVG usable by resvg, host measurement interface, off-UI-thread bounded work. Other diagram languages, Markdown integration and the pan/zoom project were outside this engine task. | Original Java/Node/Graphviz may be developer/test oracles only. A standalone host reading a selected source is distinct from an engine resolver. Later worker limits never became an OS filesystem/network sandbox. | [Handoff runtime/source constraints][handoff], [current runtime limits][coverage] |
| 2026-10-22 — same handoff | **Owner: hard translation-source/license gate.** PlantUML only from the authorized MIT1.2026.9 tree; no GPL repository/editions or EPL ELK translation. Crates must pass project deny policy; preserve original code/data/notices and inspect embedded-source lineage. | A package MIT label is insufficient provenance. The handoff describes Maven-origin metadata; later SHA/source-JAR checks establish local byte identity, not independent Maven authentication. | [Handoff license rule][handoff], [source audit][source-gate] |
| 2026-10-22 — three scope answers, implementation design | **Owner: try full Mermaid then PlantUML ports, permit suitable extra dependencies, start a new Rust project using IVE's pgui and deliver the standalone viewer first.** Local implementation/subagent execution was authorized. | Supersedes analysis-only hold. It is a full-port **attempt**, not permission to rename an admitted subset as complete compatibility. IVE product integration/push/publication were not authorized. Treating those answers as sufficient routine execution authority is controller R2 below. | [Implementation design][implementation], [parent ledger preflight][native-ledger] |
| 2026-10-23 — after viewer/Mermaid audits | **No new owner permission to ship the candidate.** Controller rejected the audited `puml` candidate because unresolved implementation lineage failed the owner's source gate; fresh MIT-only PlantUML implementation selected. | Supersedes earlier conditional candidate-adoption recommendations. This is not an infringement finding; no rejected candidate implementation/examples were imported. Preserve the main-controller analysis correction rather than recreating a stale pending patch. | R5/R6 below; [source audit][source-gate], [native plan][native-plan] |
| 2026-10-23 — state/activity stage | **Owner's full-port/overnight-readiness request is recorded as motivation**, not a guarantee of overnight parity. Controller R7 moved advanced state3B2 behind family inventory3C–3Q and before final4B. | Supersedes the earlier state-extension-before3C preparation note. It postponed, never cancelled3B2. Later R10 inserted parser migration before the remaining breadth; R11 allowed independent integrations to occur in parallel. | R7 below; [native ledger][native-ledger] |
| 2026-10-23 — visible tracker request | **Owner: keep a visible progress.md with all done/remaining work and ongoing orchestration.** Initially placed in the user's main checkout; later moved into the central worktree at owner request. | Controller alone edits the master. Keep it after any cleanup and update durable evidence links before removing only controller-owned ignored SDD artifacts. File movement is recorded; no available tool retargeted the chat UI itself. | [Native ledger:256][progress-request], [migration handoff][migration-ledger], [live master][master] |
| 2026-10-23 — parser assessment question | **Owner asked about the parser stack/LALRPOP development speed.** Controller's initial response was an assessment and recommendation for a future-family EBNF/regex pilot, with custom lexing/bounds/models/layout still required; no runtime benchmark claim. | The question itself authorized no migration/dependency/product changes. That pilot recommendation was later explicitly superseded, not silently treated as owner direction. | [Assessment event][parser-assessment] |
| 2026-10-23 — explicit repeated rewrite request | **Owner: “use LALRPOP for parsing plantuml. rewrite current adhoc parser.”** Applies to every current family and shared/data/Creole structure. | Supersedes pilot-only advice. Preserve models/layouts/API, exact supported/rejected subset and resource/origin/cancellation rules. A Line* forwarder, selector above a legacy parser, runtime fallback or partial-model recovery is not completion. New-family breadth held until migration finished; subsequent families also use LALRPOP. | [Explicit amendment][rewrite-direction], [migration design][migration-design] |
| 2026-10-23 — before first parallel batch | **Owner asked why it was slow and how to accelerate.** Controller selected isolated family worktrees, one writer per checkout and one serialized central integrator, with focused worker checks and combined gates (R11). | Supersedes the controller's all-plan serial writer/per-worker-full-gate schedule; not the source/semantic/resource contracts. Shared-interface unions remain coordinated. | [Acceleration event/ruling][parallel-ruling] |
| 2026-10-23 — after first reviewed parallel batch | **Owner requested next-session handoff because the weekly Codex limit was near.** Finish running focused checks, record honest WIP, freeze/stop workers; do not start more implementation/review/research. | Preserve approved0c5177e separately from frozen UML/activity and unfinished data work. A WIP commit was not approval. Resume only on the later owner continuation. | [Migration handoff freeze][migration-ledger], [historical handoff][old-handoff] |
| 2026-10-23 — resumed session | **Owner resumed execution and changed method:** implement → focused tests → fixes/repeat; broad independent review at the assembled-port end. | R12 supersedes per-stage broad review requirements in old plans/briefs and the task-review clause of R11. It does not erase earlier reviews, permit unsupported syntax, cancel final independent review, or waive focused proofs/source/baseline/native gates. Old serial10–18hour ETA is superseded; later estimates remain agent estimates, not owner deadlines. | [Master policy/cost][review-deferral] |
| 2026-10-23 — final families/tooling | Controller released only two independent reproduced Task4B tooling defects while breadth continued: authenticated archive-member/extraction comparison before import-baseline recording, and explicit fresh coherent oracle output. Later bounded native corpus/deadline repairs followed. | This scoped release did not authorize final coverage/corpus claims early or changing existing IMPORTS/PATCHES/vendor bytes. Fresh-output/hash/process cleanup rules preserve old evidence; repairs are developer-only. | [Task4B implementation report][tooling-report] |
| 2026-10-23 — post3Q | Advanced state3B2 released after clean465b9c8, integrated95908d1, then the concrete unquoted `state history*` admission leak repaired at3478bbd with quoted/stereotype controls. | Fulfils the postponed bounded state increment. This is not full state-machine execution/routing/parity. Late repair is a tested correction, not a new thirteenth ruling or a broad-review verdict. | [State report][state-report], [integration/follow-up supplement][state-supplement], [current master][master] |
| 2026-10-23 — current handoff | Controller commissions this read-only chronological consolidation for final evidence. Full core/audit,95-source corpus and72-frame gallery are recorded passed at3478bbd. | Current evidence is integrated at8d41271 and final COVERAGE/README atffd44a5. Both scoped independent final reviews atffd44a5 are now complete with no Important/Blocker and no fix wave required. This archive closeout retains their exact reports and twelve-ruling record; central integration and final handoff remain controller-owned. No full-upstream approval is inferred. | [Current master][master], [final corpus report][final-corpus], [closing gates below](#5-closing-gates-and-evidence-dispositions) |

## 2. Twelve controller rulings, in their recorded chronological order

R1–R11 below preserve the ruling and cost-if-wrong wording from the [old chronological record][old-rulings], cross-checked against both ledgers. Dates are bounded by the associated stages, not fabricated chat timestamps. R12 is the later review policy/cost recorded in the live master; it was not present in the eleven-ruling snapshot.

### R1 — reuse audited Rust work (2026-10-22 planning; active)

> Ruling: Reuse audited native Rust ports when they preserve broader behavior than fresh wrappers — owner permits additional dependencies and handoff prefers existing usable Rust work — cost if wrong: backend replacement and compatibility rework.

Controller choice under the owner's scope/source conditions, not unconditional adoption of any port. [Parent line28][r1]

### R2 — execute the accepted plan without routine approval pauses (2026-10-22 planning; active)

> Ruling: Treat the owner's three scope answers as authorization to execute the written design/plan without further routine approval gates — owner explicitly requests unattended subagent implementation — cost if wrong: reversible local branch work may need redesign.

Does not authorize publishing, user-main product changes or abandoning an explicit later HOLD/freeze. [Parent line29][r2]

### R3 — use both pgui framework packages (2026-10-22 planning; active)

> Ruling: gpui and gpui_platform together are the pgui framework dependency — the pinned split requires both for desktop startup and other dependencies are allowed — cost if wrong: manifest reshaping only.

The selected pin is IVE's `f5d5fa2f8f07b0be68442e694d22ed6064609e73`; the engine remains GPUI-free. [Parent line30][r3], [implementation design][implementation]

### R4 — published Merman0.9.2 baseline (2026-10-22 Mermaid stage; active)

> Ruling: Prefer published Merman =0.9.2 as the audited broad baseline while retaining explicit Mermaid 12.2 residuals — it already has the broad family surface, host metrics and cooperative limits — cost if wrong: upgrade/fork work for new 12.2 syntax and geometry.

Baseline target is12.1, with specifically tested12.2 depth behavior and four audited patches;37 native variants versus36 original-core accepts plus the ZenUML plugin boundary do not establish full12.2 parity. Its parent-ledger line appears late because that section contains historical audit entries; the recorded chronological record puts it fourth. [Parent line421][r4], [coverage][coverage]

### R5 — exclude the current puml candidate (2026-10-23, before fresh native plan; active)

> Ruling: Do not adopt the current puml candidate after concrete unresolved implementation-lineage evidence — the owner's MIT-only PlantUML translation-source rule is hard, and package MIT metadata does not resolve that evidence — cost if wrong: unnecessary fresh implementation and reduced short-term coverage; the candidate remains available only for a future rights-resolved audit, not shipping code.

Supersedes the earlier limited candidate audit/conditional fork recommendation; no infringement finding. The source-gate docs checkpoint is f16795f (2026-10-23T00:44:03+02:00). [Parent line92][r5], [source-lineage audit][source-gate]

### R6 — distinct typed PlantUML models/scenes (2026-10-23 fresh native design; active)

> Ruling: Use fresh typed PlantUML family models and native scenes, sharing audited measurement/SVG/Dagre services below the language boundary — PlantUML sequence/UML/container semantics do not map completely to Mermaid source — cost if wrong: more frontend/scene code and layout maintenance than a thinner compatibility layer, with no automatic Graphviz geometry parity.

Supersedes the thin language-rewriting approach; later parser migration preserves these verified models/layouts. [Parent line104][r6], [native plan][native-plan]

### R7 — inventory breadth before advanced state (2026-10-23, after initial state/activity; fulfilled)

> Ruling: Complete the remaining native family-inventory increments3C–3Q before the separate advanced-state3B2 extension, then final4B evidence — the owner asked for a full-port attempt and overnight readiness; the state/activity checkpoint's substantial fix/review cost makes broad genuine-family coverage the next priority, while shared styles/data infrastructure can stabilize before the state extension — cost if wrong: advanced state support arrives later and its implementer must adapt to later shared-interface changes. This reorders, does not cancel3B2 or weaken any review gate. After current3Bfix/review, nextproductreleaseis3C;3B2brief/researchremainHOLDuntilafter3Q, before4B. Earlier before3C ordering notes are superseded.

The initial state fix checkpoint is49838e8 (03:28:25+02:00);3C checkpoint6ec1022 is04:12:01+02:00. R10 later interposed migration and R12 later changed review timing. Broad delivery remained required; parallel integration order need not equal the queue's printed ordinal. [Parent line388][r7]

### R8 — unmatched symbolic punctuation stays literal (2026-10-23 shared Creole stage; active)

> Ruling: Preserve already-admitted unmatched symbolic punctuation when adding shared Creole styling, rather than the prepared 3D brief's blanket unmatched-delimiter rejection — that sentence contradicted its own all-regressions requirement and the existing `left -- right`/empty-slot test, while inspected original behavior permits unmatched markers as text — cost if wrong: malformed symbolic emphasis may remain literal where a stricter subset would error, requiring a documented activation-policy revision. Activated spans must still nest correctly, recognized tags must close, resources remain denied, and pairing must be linear/bounded. Task 3D brief amended before release; no product edits or completed-task repetition.

Preserved through generated Creole nesting and standalone Creole. This is a controller resolution of contradictory prepared wording, not an owner request to accept arbitrary markup. [Parent line374][r8]

### R9 — truthful bounded late-label diagnostics (2026-10-23 constants/origin stage; active)

> Ruling: Preserve authoritative parser-line origin mapping and permit explicitly ambiguous bounded text/use candidates for late identical-label diagnostics, rather than inventing a precise occurrence or broadly replacing all family models with source-bearing label types — current parsers often retain only statement line/column1, and late Creole Context calls have text but no semantic source identity; honest conservative context satisfies this checkpoint without misrepresenting precision — cost if wrong: some errors need manual disambiguation and a later narrow family/token-origin metadata migration. Unique mappings remain exact where known; candidates/omitted counts and definition IDs must be labeled truthfully, inert duplicates tested, and all bounds remain mandatory. Task 3D2 brief amended before review; no runtime/grammar capability expansion.

Not a blanket waiver for parser origins or a claim that every current error has coarse columns. Direct/generated spans remain exact where known; current YAML/late-label limitations require explicit final triage. [Parent line340][r9]

### R10 — full current LALRPOP rewrite before more breadth (2026-10-23; fulfilled, architecture remains active)

> Ruling: Insert full current PlantUML LALRPOP grammar migration before remaining family breadth, using staged real generated-grammar cutovers while retaining models/layouts and explicit bounded lexical/preprocessing semantics — the owner's repeated explicit rewrite direction requires replacing current ad-hoc syntax, and existing rendering/regression contracts remain load-bearing — cost if wrong: generation/dependency complexity, syntax-parity rework and delayed remaining-family breadth; staged local changes remain reviewable and reversible.

Owner chose LALRPOP/the rewrite; controller chose separate generated family/front-matter grammars, checked-in generation and token/budget decomposition. Migration design committed5d7706e at10:06:34+02:00; all17 stages later verified at0f468eb at16:43:08+02:00. The21 initial grammars grew to29 with later inventory; this count is not upstream-syntax completeness. [Parent explicit direction][rewrite-direction], [migration design][migration-design]

### R11 — isolated parallel implementation, serialized integration (2026-10-23, recorded before12:01UTC; active except review clause)

> Ruling: Run independent remaining family ports in isolated parallel worktrees, retaining one integration writer, focused worker regression/proof/source checks, independent task reviews and complete combined integration gates — owner asks to speed up the serial workflow; shared frontend is implemented and filesystem isolation removes concurrent source edits — cost if wrong: shared registration/dispatch/provenance merge conflicts and private-interface rework; scoped branches/base hashes remain available for review/revert. This supersedes the self-imposed all-plan single-writer/per-worker-full-gate schedule. Parent retains10 historical Rulings; this child adds1, final chronological handoff TOTAL11.

R12 supersedes the independent-per-stage-review portion, not one writer per isolated worktree/one central writer, tested checkpoints, coordination or shared-source unions. The reviewed first combined batch0c5177e is recorded12:26UTC (commit14:22:42+02:00). [Migration line65][parallel-ruling]

### R12 — focused implementation loop, broad review at end (2026-10-23 resumed session; active)

> Owner direction (2026-10-23): continue implementation without per-stage independent review. The broad review moves to the end. Cost if wrong: defects may accumulate across grammar stages, so each checkpoint needs focused tests, exact failing cases, and a reproducible commit before integration.

This is the current master's recorded direction/cost; it is the twelfth decision counted there, not a line retroactively present in the eleven-ruling archive. Earlier plan/task demands for a fresh broad review after every stage are superseded. Final whole-branch independent review and concrete-finding fixes/re-review remain required. [Master line206][review-deferral], [master ruling count][ruling-count]

## 3. Implementation decisions that must not become extra owner directives or invented rulings

These qualify execution/evidence under R1–R12; they do not change the twelve-ruling count.

- **Resources versus literal text:** the early Mermaid resource-token blacklist was repaired to use semantic/CSS/SVG admission, preserving inert URL/metadata-looking text. Includes/assets/full TIM remain denied or explicitly bounded caller-memory-only policy. This was a reproduced defect repair, not a user request to allow network access. [Native ledger][native-ledger], [README][readme]
- **Runtime bounds/host metrics:** the controller selected the isolated same-binary worker, per-render cache lifetime and logical/raster split. Current engine defaults256KiB/16MiB/model10,000/depth64/work5,000,000/cooperative10s and Unix host10s/2GiB are measured implementation policies. The initial plan's conditional1MiB source suggestion is superseded by audited stricter limits. Host stages remain cooperative; macOS/non-Unix equivalence is unverified. Native advances/line metrics, deterministic fallbacks, approximate baselines, bbox/wrap gaps and viewer-only paint containment must remain distinct from original pixel/ink parity. [Design][implementation], [coverage runtime][coverage]
- **Generated does not mean automatically bounded:** retaining actions/typed cancellation plus actual generated-table stack/reduction/diagnostic/allocation admission are required. No fake EOF/recovery-and-render. Checked-in Rust, pinned developer-only LALRPOP closure and explicit write/check commands keep Java/Node/generator out of runtime. [Migration design][migration-design], [current admission][admission]
- **Cache/scheduling corrections:** a shared unhashed generator embedded another checkout path; workers moved to private targets and absolute manifests from/tmp, then Task17 added explicit validated roots/two-checkout controls. Shared engine tests use the outer build lock; serial full-test ordering avoids the existing500ms action-test load sensitivity. Failed logs are retained; assertions/deadlines were not weakened. These are controller operational safeguards. [Migration cache incident][cache], [master implementation loop][master]
- **Immutable baseline:**43 source/family/SVG identities were recorded before parser changes. Task17 corrected actual exporter-config reporting; Task3M repaired check-selection to those recorded paths while new fixture hashes stay independently audited. No bulk re-record to conceal drift. [Frozen baseline][baseline], [master tracking][master]
- **Research is not authorization/coverage:** prepared state research's possible homogeneous-only region subset was superseded by the released3B2 requirement to accept mixed separators with the first axis. Error SVGs, intentionally activity-detected synchronization and degraded formula images are not state/math positives. Existing scoped source inspections/credits are required for actual translations; research does not authorize wholesale import. [State implementation report][state-report], [catalog][catalog]
- **Evidence categories remain separate:** reported implementation checks, historical independent review, source/original research, native capture, corpus export, selected semantic/geometry checks and actual pixel comparisons establish different facts. The23-byte `nothing important left\n` builder-check file is explicitly a placeholder, not substantive verification or final approval. [Archive explanation][archive], [historical verdict index][verdicts], [coverage evidence types][coverage]
- **Preserve provenance and history:** source-JAR/registry/archive identities and actual authors/notices must remain exact; local artifact SHA is not external authentication. Candidate exclusion is source-lineage policy, not infringement adjudication. Historical manifest overwrite and three rerendered original SVG differences are acknowledged; fresh outputs never overwrite old artifacts. Keep archived payload bytes, including old paths/stale statuses/blank EOF lines, unchanged and label their scope. [Archive][archive], [Task4B implementation report][tooling-report]

## 4. Checkpoint chronology — dates of commits, not fabricated decision times

| Recorded commit time (+02:00, 2026-10-23 unless marked) | Checkpoint | Meaning / limit |
| --- | --- | --- |
| 2026-10-22 23:48:26 | a8394fe | Mermaid semantic resource repair; reviewed subset baseline. |
| 00:21:11 /00:37:51 | c3486fd /4f80a73 | Worker/cache/zoom repair and four scoped Mermaid depth/font patches. |
| 00:44:03 /01:50:53 | f16795f /d07df14 | Fresh MIT-only plan/source gate; reviewed first native PlantUML spine/sequence/UML. |
| 02:29:38 /03:28:25 |130ca10 /49838e8 | Viewer containment/feImage rejection; initial state/activity ownership and painted-flow fixes. |
| 04:12:01 /05:17:36 /06:23:19 |6ec1022 /71cd817 /6abc18f | Trees/data, measured styles/Creole and eager literal/origin stages. |
| 09:39:00 /10:06:34 |4c19787 /5d7706e | Atomic-comment prerequisite and full migration design/plan. |
| 12:48:55 /14:22:42 |f5a10a0 /0c5177e | Shared generated preambles reviewed; first independently approved parallel integration.0c5177e remains the master's last fully reviewed product checkpoint, not approval of later work. |
| 16:43:08 |0f468eb | All17 original migration stages verified,21 grammars; later-family broad review deferred. |
| By18:22:45 |465b9c8 | Remaining inventory through standalone Creole centrally integrated. Parallel integrations need their exact report ranges; printed queue order is not commit-time order. |
| 19:02:25 |95908d1 | Advanced-state integration. |
| 19:16:08–19:17:39 |9434e2b /73126d8 /882cac3 | Bounded coverage docs and historical archive/tracker integration; not final corpus/review acceptance. |
| 19:19:11 |3478bbd | Finite-value/unquoted-state-identity repair. |
| 19:31:06 |ff67547 tracker | Records final full core/audit,95-source corpus and72-frame gallery at product3478bbd; current evidence/coverage integration and review were pending at that checkpoint. |
| Later same-day integration |8d41271 | Current evidence addendum integrated:17 current payloads join114 unchanged historical payloads. |
| Subsequent same-day integration |ffd44a5 | Final COVERAGE/README updates integrated. Both subsequent scoped independent reviews inspected this head and found no Important/Blocker; no fix wave is required. Final archive integration/local handoff remain controller-owned. |

Dates/subjects were read from local Git. Full ranges/worker versus controller evidence and scoped verdicts remain in the linked reports/catalog; this table does not re-run or approve them.

## 5. Closing gates and evidence dispositions

At this archive closeout the following are **recorded complete by the current controller master**, not independently re-executed by this read-only task:

- Full product3478bbd core/audit:858 ordinary workspace Rust tests+seven actual worker faults; required all-targets859+seven; strict Clippy/fmt/release/deny; generator tests/strict/fmt/deny;75 full Node tests;29 reproducible parsers;293 selected MIT/58 fixture/58 native module/67 generator dependency/656 Merman audits; unchanged43 baseline. Release viewer SHA `0c43488e42fca26229b00b9fbce13511726c8786c061cef8ce4827e04943bd2d`.
- Fresh final corpus:95 exact Git-aligned sources (37 Mermaid/58 PlantUML), native95 rendered, original94 rendered; only original-core ZenUML registration failed.284 original/476 native/18 repeat artifacts close. Selected37 Mermaid label controls,24 state endpoints/pseudo topology and regional-owner controls pass. No universal all-label/semantic/geometry/pixel verdict follows. Full report/manifest hashes are in [final corpus report][final-corpus]; the durable current corpus reports/manifests are integrated; raw SVGs/PNGs/logs remain external with recorded paths and hashes.
- Final72-frame gallery:35/35 PlantUML examples have positive native metrics/zero fallback; Mermaid20/37 have positive native and9/37 zero fallback. Old native frames/canvases compare to rehearsals, not original-renderer pixels. Invalid-input panes, empty-PATH failed-export preservation and process cleanup are recorded. [Live master][master]
- Fresh repeat caveats remain: raw architecture Iconify IDs vary; Mermaid Git labels and geometry vary; Gantt date/container policy and bbox caveats remain. Original foreignObject presence and text-only metrics limit label comparison; native terminal resource safety is a separate test/audit claim. The final corpus report preserves both raw runs without normalization.

**Current integration and remaining handoff:**

1. **Current evidence complete:**8d41271 integrates17 current payloads (411,317 bytes), including the final corpus structured results, gallery/controller evidence and advanced-state reports. The114 historical payloads (1,844,096 bytes) remain unchanged. See the [archive scope][archive] and [catalog][catalog]. Raw render/capture/log artifacts remain external with recorded identities; structured preservation does not create final independent approval.
2. **Controller handoff packaging:** final COVERAGE/README updates are integrated atffd44a5. This archive closeout adds the twelve-ruling record and two exact final review reports with navigation/manifests/checksums, preserving all114 historical and17 earlier current payloads. The controller owns central integration and live-checklist reconciliation; no archive closeout commit is invented before it exists.
3. **Final independent review complete:** the [whole-branch product review](reviews/ive-final-whole-branch-review.md) and [evidence/provenance review](reviews/ive-final-evidence-review.md) both inspected `ffd44a569850d05e0359bff8ea6267982342c6fc` and found no concrete Important or Blocker in their stated scopes. No fix wave is required. The product reviewer's optional four-case binary probe was rejected by automatic approval review and was not retried; existing controller gates/corpus and committed tests supply executable evidence. The evidence reviewer independently checked hashes/audits/selected external artifacts, while broad Cargo gates, negative-pane inspection and34 PlantUML crop comparisons remain attributed controller facts. Neither report claims complete upstream compatibility, universal labels/topology/geometry/pixels, or macOS runtime verification.
4. **Final disposition and launch handoff:** explicitly retain diagnostics (coarse/late-candidate/YAML positions), large-unit/vendor Atomic warnings,500ms verification-load sensitivity, platform limits, and compatibility/layout/font/assets/TIM/math residuals. Distinguish the completed defined standalone port from full upstream syntax/style/geometry/assets/pixel equivalence. Give launch/coverage/evidence paths, local branch/head and process-cleanup status. No unrequested publish/push/IVE integration.

**Reconcile stale checkboxes without repeating completed work:** historical ledgers, old archive snapshots and earlier pending text predate the current evidence integration at8d41271 and final docs integration atffd44a5. Owner R12 and the controller's latest recorded releases govern. Final full gates/corpus/gallery, current evidence, final COVERAGE/README and both scoped independent reviews are complete. This archive closeout retains the durable Rulings/review entries, with no fix wave required; central integration and local handoff remain controller-owned. The controller owns checklist updates.

## Source map

Links below resolve from `diagram-ports/evidence/current/RULINGS.md`. They use integrated state/corpus reports, historical ledgers/catalog and tracked specifications. Historical line numbers are descriptive labels only, never appended to file paths. The canonical live progress location remains `/mnt/data/Projects/ive/.worktree/diagram-ports/progress.md`; the retired user-main path is preserved above as relocation history.
[master]: ../../../progress.md
[handoff]: ../../../docs/superpowers/specs/2026-10-22-diagram-ports-handoff.md
[implementation]: ../../../docs/superpowers/specs/2026-10-22-diagram-ports-implementation-design.md
[native-plan]: ../../../docs/superpowers/plans/2026-10-23-plantuml-native-plan.md
[native-ledger]: ../history/native-controller-ledger.md
[migration-ledger]: ../history/migration-controller-ledger.md
[old-rulings]: ../../../docs/superpowers/handoffs/2026-10-23/RULINGS.md
[old-handoff]: ../../../diagram-ports-handoff.md
[migration-design]: ../../../docs/superpowers/specs/2026-10-23-plantuml-lalrpop-design.md
[rewrite-direction]: ../history/native-controller-ledger.md
[parser-assessment]: ../history/native-controller-ledger.md
[progress-request]: ../history/native-controller-ledger.md
[parallel-ruling]: ../history/migration-controller-ledger.md
[cache]: ../history/migration-controller-ledger.md
[review-deferral]: ../../../progress.md
[ruling-count]: ../../../progress.md
[r1]: ../history/native-controller-ledger.md
[r2]: ../history/native-controller-ledger.md
[r3]: ../history/native-controller-ledger.md
[r4]: ../history/native-controller-ledger.md
[r5]: ../history/native-controller-ledger.md
[r6]: ../history/native-controller-ledger.md
[r7]: ../history/native-controller-ledger.md
[r8]: ../history/native-controller-ledger.md
[r9]: ../history/native-controller-ledger.md
[source-gate]: ../../../docs/superpowers/plans/2026-10-23-puml-provenance-audit.md
[tooling-report]: ../tooling/reports/task-4b-import-oracle-report.md
[state-report]: state/task-3b2-report.md
[state-supplement]: state/task-3b2-integration-followup-report.md
[archive]: ../README.md
[catalog]: ../catalog.md
[verdicts]: ../verdict-index.md
[coverage]: ../../COVERAGE.md
[readme]: ../../README.md
[admission]: ../../provenance/plantuml-parser-admission.md
[baseline]: ../../fixtures/plantuml/parser-baseline.json
[final-corpus]: final-corpus-report.md
