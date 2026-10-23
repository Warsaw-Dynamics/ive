# Diagram ports — next-session handoff

Saved 2026-10-23. Work stopped at the owner's request because the weekly Codex limit is near. All implementation workers have committed their checkpoints and stopped. The overall port is unfinished.

## Start here

Use `/mnt/data/Projects/ive/.worktree/diagram-ports` as the working directory. The owner explicitly moved the session's work here; `progress.md` was moved out of user main. All subsequent commands in this session target this checkout. The tools expose no host-session relocation operation, so the application's chat location itself was not changed.

Read [progress.md](progress.md), this handoff, and [the standalone README](diagram-ports/README.md). Do not repeat completed architecture/source research or restart the migration. Detailed evidence is in the controller-owned `.superpowers/sdd/` directories, with a durable Markdown/evidence snapshot under [the handoff bundle](docs/superpowers/handoffs/2026-10-23/README.md).

Owner's binding instructions: try full Mermaid/PlantUML ports, additional dependencies allowed, standalone viewer first; use subagents for implementation and root for orchestration/verification. Replace **all current PlantUML ad-hoc structural parsers with real LALRPOP**. No generated `Line*` forwarding, whole-command reparse, legacy fallback or partial render. New-family breadth waits until current-parser migration completes. Preserve existing model/layout/API/bounds/source ownership and supported/rejected behavior, except the four assigned boundary repairs.

## Exact checkpoints

| Checkout below `/mnt/data/Projects/ive/.worktree/` | Branch | Head and status |
| --- | --- | --- |
| `diagram-ports` | `diagram-ports` | `0c5177e1f4cf50dcc23dc97ff3e0812b42d4070f`: last fully approved product; this handoff/progress/bundle are new documentation |
| `diagram-uml-parallel` | `diagram-parser-uml` | `0c2ee0e3b52d4d5a02a360fd5f4ae61931fdb089`: Task5 fix1 GREEN, scoped re-review pending; base `a23aaeeaf17dc33734874dc2e86c024121b9581e` |
| `diagram-activity-parallel` | `diagram-parser-activity` | `1b0c234b73223abe74c8fcc7e4f6e3451a671f1c`: Task8 worker-complete, initial independent review pending; base `1506eb1f14248eb6dfe109164308f6914e3421ec` |
| `diagram-data-parallel` | `diagram-parser-data` | `e71d646376c54cc7c7cb030c1d6029193109e66f`: Task10 **WIP**, latest tests/provenance/proof incomplete; base approved `0c5177e` |
| `diagram-state-parallel` | `diagram-parser-state` | `ec608400b2fb375cd899c23301bb7472c27b558e`: approved and already merged centrally |
| `diagram-tree-parallel` | `diagram-parser-tree` | `4a31cb7a3a3ceffefeaf17be2471e1dd20bee1e2`: approved and already merged centrally |

Workers have clean tracked trees. Their only intentional untracked file is root-owned `diagram-ports/.cargo/config.toml`; preserve it and never commit it. It points Merman patches to identical audited vendor copies in the central checkout for cache reuse. Central product tree was clean before handoff documents were added.

User main moved independently to `19f7b17ca289f1923685bbb837d1040f391638fd` when last inspected. Other `.claude/worktrees/` belong to unrelated work. Never reset, merge into, clean, or otherwise modify them. No push/publication was authorized.

## Verified completed work

Mermaid and the standalone GPUI viewer are implemented and reviewed. Merman0.9.2 supplies the audited broad native baseline:37 admitted variants,656 imported files/four recorded patches; this is not full Mermaid12.2 or browser/pixel parity. PlantUML native models/rendering through sequence/UML/state/activity/trees/data/styles/constants/timing/Gantt/Salt/nwdiag/files/packet/board/HCL/Chen/icon-free ArchiMate are implemented for documented subsets.

LALRPOP stages **1,2,3,4,7,9 are fully approved**: pinned generator/definitions; wrappers/header/ownership/origins; preamble/styles; sequence; state; mindmap/WBS/files/board. The central head contains nine reproducible generated outputs and six generated family routes.

Latest root independent validation at `0c5177e`:602 Rust tests, seven actual worker-fault scenarios, all16 gates,18 Node tests, three generator tests, five notice-tamper tests,67-package generator closure, source220/fixtures43/native modules43 audit, unchanged43 source/family/SVG baseline records. Root independently byte-compared23 merged family files with their reviewed heads. Fresh integration SPEC/QUALITY review approved.

Native sequence/state/mindmap/WBS/files/board captures all passed, measurements7/7/5/5/12/8 with zero fallback. Root viewed five new frames; sequence hash matched a prior inspected frame. Six empty-PATH/no-display exports passed. Anchored process checks found no surviving viewers/workers. Logs/captures use `/tmp/ive-parser-integration1-*`, and are copied into the bundle.

Immutable baseline SHA256: `a14d47f73af4f7c78c67dc746ac5c51eb96d89e54db1b52425cc1b8e08fafafe`. Never rerecord it to hide a difference.

## Resume these three tasks first

All paths in this section are relative to `.superpowers/sdd/2026-10-23-plantuml-lalrpop-plan/` in the central checkout. The same Markdown reports are backed up under the bundle's `parser-plan/`.

1. **Task5 UML fix1 scoped re-review.** Read `task-5-brief.md`, `task-5-report.md` and `/tmp/ive-plantuml-lalrpop-uml-review.md`. Initial task review found only R1: new quote decoding checked decoded length instead of old raw16KiB quote bytes. Fix base `6f1244cf87cedad7a4656ab23e249e2557d32ac3`; fixed head `0c2ee0e3b52d4d5a02a360fd5f4ae61931fdb089`. Prepared package `review-task5-6f1244c..0c2ee0e.diff` is8,624bytes. Root read the full fix appendix:322 relevant tests and strict/fmt/private-gen6/source/closure checks pass with separately named logs. **Do not redo the initial whole-task review.** Public nuance: the escaped-quote example already failed later at4096 label tokens; the fix restores the earlier raw-byte check and error boundary, without relaxing the later guard. Grammar/generated hashes and sizes32/48 remain unchanged. Root already inspected and independently ran the actual379-state/36-terminal witness; no expensive rerun needed for this five-line lexical fix.
2. **Task8 fresh initial SPEC/QUALITY review.** Read `task-8-brief.md` and full `task-8-report.md` (root already read). Prepared package `review-task8-1506eb1..1b0c234.diff` is199,788bytes. Worker passes180 units+44constants+23control+26styles, strict/fmt/private-gen6/source audit. Real recursive activity grammar replaces old recursive parser; models/activitytile production layout unchanged. Actual92states/27terminals, four reductions/12expected; `/tmp/ive-task8-stack-proof.py` and JSON use187-edge finite closure and max-plus induction for9d+6. **Root has not independently inspected/run that new proof yet.** Existing60,000-row/500ms controls retained; test setup restores its original document-only semantics rather than adding preprocessing overhead. Public full-pipeline raw/expanded action tests pass. No independent task approval or central native/baseline run yet.
3. **Task10 resume implementation, not review.** Read `task-10-brief.md` and `task-10-report.md` carefully. WIP removes old JSON/HCL recursive parsers and adds genuine structural grammars/facades returning existing models. Earlier203 units and relevant public suites passed. A later run was96pass/one assertion overcounting seven JSON cancellation checkpoints as eight; assertion corrected but **not rerun**. Newest quote/numeric/origin/storage tests are unverified. Provenance/parser source registrations and admission write-up were **not applied** after an oversized patch truncated. Run latest tests, finish actual storage/inductive proof, apply small provenance patches, regenerate/check privately, source audit, final strict checks, commit/freeze and then fresh independent review. `/tmp/ive-task10-stack-proof.py` exists; its observed JSON output was **not persisted**. JSON48states/16terminals/5d+8/eight reductions; HCL40/10/max(4d+4,6)/five reductions are reported, not yet root-verified. Do not integrate this WIP as complete.

A useful first parallel dispatch is two read-only reviewers (UML fix, activity task) plus a Task10 implementer. Old agent contexts may not be resumable across sessions: if unavailable, dispatch fresh agents with these exact reports/briefs/finding/fix-base rather than restarting the task or fix-round count.

After UML approval, Task6 Chen can start in isolation using the corrected UML interfaces; its exact brief is already extracted. Integrate approved UML/activity/data branches with one central writer, then run the full shared checks once and independent review of conflict unions. Do not integrate unapproved WIP. Keep source branches/base hashes available.

## Shared integration details

Read `global-context.md`, `parallel-context.md` and `integration-1-interface.md`. Each family facade takes BodySource+Budget and returns its preserved model. Parse before the remaining legacy layout/error closure so facade failures map through the origin document once. Continue shared Ownership; do not independently rescan wrappers/comments/evaluation.

One actual-size BodySource reborrow allowance is charged in common dispatch; adding routes must preserve it once. UML and data branches mirror Cow Syntax/Unsupported detail with static constructors+owned_syntax; FailureKind32/ParseFailure48 remain unchanged. Data also adapts a generated-state test pattern to compare the same Cow detail text. UML exposes existing `context::push` to plantuml builders, visibility only. Union these minimal common changes. After UML+state union, remove legacy `lexer::name` only when its last caller is gone; do not retain a hidden structural parser or weaken assertions.

Expect conflicts in shared dispatch/registration, generated manifest and provenance/admission lists. Union entries, never select one whole side. Mechanically regenerate outputs/manifest; do not hand-edit them. Root must not patch product findings itself: dispatch implementation/fixes and independently review them.

## Build isolation and verification

All Cargo or Cargo-spawning commands take one OUTERMOST lock:

```sh
bash /tmp/ive-diagram-build-locked.sh env CARGO_TARGET_DIR=/mnt/data/Projects/ive/target cargo test --locked --offline -p diagram-engine --lib -- --test-threads=1
```

Run that from the assigned checkout's `diagram-ports/` directory. Code edits/read-only inspection can overlap. Do not nest locks or cargo-clean shared caches. The existing500ms test has observed load sensitivity; full central Rust tests now use testthreads1 with unchanged assertions/deadlines. Preserve failure/serial evidence for final triage.

**Generator exception:** separate target per checkout under `/mnt/data/Projects/ive/target/diagram-generator-{integration,uml,activity,data,state,tree}`. Run from `/tmp` with the absolute assigned `tools/plantuml-grammar-gen/Cargo.toml` manifest. A shared unhashed binary embeds `CARGO_MANIFEST_DIR` and actually generated into another checkout despite Cargo Fresh. Private targets prevent that. Never use shared generator binaries, shared cargo-clean or metadata hacks. Cargo-run examples/baselines/viewer release/native are CENTRAL ONLY for the same alias risk. Task17 carries a durable explicit invocation-root/two-checkout regression fix.

Root's `/tmp/ive-verify-parser-cutover.sh` runs nine core+seven audit gates; use core THEN audit under one lock. It has the private integration generator exception and testthreads1. Helpers are backed up in the bundle. All workers use focused suites/strict checks/private generation/source audits; root performs full16/baseline/native once per combined batch. No repeated original Java/browser oracle runs merely to reconfirm parser changes.

Sandbox `bwrap: loopback: Failed RTM_NEWADDR` often prevents even reads. Scoped `require_escalated` calls were auto-approved; no approval rejection occurred. Do not waste repeated retries or treat that setup failure as a product defect.

## Remaining queue and follow-ups

Migration pending beyond current tasks:6Chen,11YAML,12timing/Gantt,13Salt,14nwdiag,15packet,16Creole,17removal/inventory/integration. Exact task briefs are prepared. Task17 brief has extra carried requirements; **do not regenerate/overwrite it**.

Assigned defects: Chen aliases entity/relationship and spaced composites(Task6); timing tracks robust/concise(Task12); nwdiag title server whitespace(Task14). Gantt unterminated trailing comment is already fixed by Task2. Task17 also reconciles baseline config with actual defaults, fixes generator invocation-root isolation, classifies/removes remaining structural helpers, preserves test timing evidence and triages two inherited Merman deprecation warnings without silently editing audited vendor code.

After all current parsers migrate, resume parent native queue:3K EBNF/regex →3Lcharts →3MASCII Git →3Nwire →3Olegacy BPM →3Pdirectional flow →3Qstandalone Creole →3B2advanced state →4Bfinal evidence/tooling/docs. Require LALRPOP for new syntax. Full upstream grammar/Graphviz/browser/pixel parity is not established; keep residual coverage honest.

Final4B still needs coherent fresh original/native corpus and tool/config/hash/semantics/geometry evidence, oracle output-directory repair, Merman import authentication hardening, COVERAGE/docs/notices, durable reports/rulings archive, final native/empty-PATH/audit checks and most-capable whole-branch review. Do not delete owned ignored SDD metadata until evidence is archived and links updated.

## Source and hard constraints

Only authorized PlantUML MIT1.2026.9 `/mnt/data/Projects/plantuml-mit/src`; selected220 source hashes/notices/credits already audited against local source JAR. Do not recrawl all sources or import GPL/EPL/ELK/Smetana/untraced assets. Merman0.9.2 vendor baseline656/four immutable. The sources-JAR hash is locally observed, not externally Maven-authenticated. No runtime JVM/JS/browser/dot/network/process/file/env/clock in engine; caller-memory pure Rust, GPUI-free engine.

Defaults remain source256KiB, SVG16MiB, model10k, nesting64, work5m, cooperative10s; host independent10s/Unix2GiB; labels16KiB, queries4096, finite coordinates1m; raster4096axis/4,194,304pixels/scale8/logicalaxis1m. Keep actual stack/reduction/diagnostic/arena admission before allocation. Hard worker bounds do not sandbox host font/raster stages; macOS runtime remains unverified.

## Decisions retained

Eleven chronological rulings and exact cost-if-wrong text are preserved in bundle `RULINGS.md`: audited Rust reuse; unattended authorization; gpui+platform dependency; Merman0.9.2 baseline; reject untraced puml; native typed Plant models/scenes; inventory before advanced state; unmatched symbolic Creole stays literal; truthful ambiguous bounded origins; full current LALRPOP before breadth; isolated parallel families with central verification. None is silently discarded.

No implementation/review/build worker remains active at handoff. Only this requested documentation move/handoff is being finalized. Resume from the exact checkpoints above.
