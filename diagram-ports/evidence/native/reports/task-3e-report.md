# Task 3E — bounded native timing diagrams

Status: implemented and frozen for controller-arranged independent SPEC/QUALITY review; this is one supported timing-family increment, not full PlantUML compatibility.

- Released reviewed base: `6abc18fb22d908e2c257267850bb387341c7ea91`.
- Frozen head: `b3cef28c6f914428d8bc2e3ca07b555a3163d9bb` — `Add bounded native PlantUML timing diagrams`.
- Worktree/branch: `/mnt/data/Projects/ive/.worktree/diagram-ports`, `diagram-ports`; tracked worktree clean after commit.
- Exactly 18 committed files, all under standalone `diagram-ports/`. No root IVE product changes, dependency/lockfile/vendor edits, runtime services, other families, subagents/reviewers, merge or push.
- Binding [brief](task-3e-brief.md), [global constraints](native-global-constraints.md) and [plan extract](task-3e-plan-extract.md) read completely. Full specialized semantics/source ledger and preceding [3D](task-3d-report.md)/[3D2](task-3d2-report.md), including fix rounds, were read. Unchanged shared style/text/origin/worker contracts and prior limitations remain those reports.

## Model, parser and admission

`timing.rs` owns a dedicated frozen `Timing { tracks, marks, constraints, messages }`, stable ordered track/state IDs, `TrackKind::Robust|Concise`, `Initial::Absent|State`, sorted exact `Change { time, state }` and typed `Endpoint { track, time }`. Parser-only builders use bounded BTree registries/sets and separate selected-player/selected-time enums; no optional wrappers are stored in the model. No PlantUML-to-Mermaid rewriting or generic wrong-family boxes.

Admitted grammar inside one `@startuml`/`@enduml`:

- `robust ["display" as] CODE`, likewise `concise`; declaration selects current player. Codes use Rust alphabetic/underscore first and alphanumeric/underscore/dot thereafter, a deliberately narrower declaration subset than upstream. Empty displays and duplicate/reopened tracks reject.
- `@N`, `@+N`, signed absolute negatives and `@+-N`; the leading plus means relative. Numeric tokens are at most 12 ASCII bytes. Both offset literals and resolved exact i64 values must be in −1,000,000,000…1,000,000,000; no f64 numeric parsing/equality. Initial relative base is zero. At least one numeric mark and a real state on every declared track are required. Marks before a declaration are admitted when the document contains actual timing evidence.
- `@PLAYER` changes player only. `PLAYER is STATE` reads current global time without changing cursors. Bare `N is STATE` changes selected time and current player's state. Whitespace between tokens includes tabs/multiple spaces. A state before any selected time remains an actual untimed initial state; repeated initial assignments replace it. Timed duplicate timestamps keep the first insertion, including negative-zero/relative-zero equality. Every authored state label is retained for shared strict text/resource validation even if its change loses first-wins, so unsupported markup is never silently discarded.
- State atoms are quoted text or a letter/number/underscore/hyphen followed by letters/numbers/underscore/hyphen/dot. Quoted escapes/Creole use the existing shared decoder. Ordinary `hidden`, `-`, `...`, `?` remain literal state names; empty and brace-leading special/multistate forms reject, including quoted reserved forms. Literal arrows/colons/URLs/paths inside quoted states and grammar words/arrows inside connection captions remain data.
- `[PLAYER]@a <-> @b [: caption]` is an increasing same-lane dimension. First relative endpoint uses diagram current time; second relative endpoint uses the first endpoint. The command leaves both cursors unchanged.
- `PLAYER[@a] -> PLAYER[@b] [: caption]` is a cross-track message. Both offsets independently use diagram current time; omitted endpoint times require/read that same selected time. Both track identities and label remain owned. Same-track messages and nonplain/styled arrows reject.

Axis policy: explicit global/local numeric marks determine min/max; constraints/messages do not advance or extend the axis. Out-of-range endpoints reject with endpoint/range/line, and message endpoints before any available signal reject rather than being silently omitted. At an exact robust transition, messages attach to the new/right-hand state; concise endpoints use ribbon center. Increasing dimensions only are a documented narrower policy.

Unsupported syntax fails with offending text/location: decimals, named time definitions/references, date/hour/clock expressions, scale/ticks/time-axis formatting/hiding, clock/analog/binary/rectangle/compact tracks, state-definition aliases, hidden/unknown/tristate/multistate forms, notes/highlights, state comments, track/state colors/stereotypes/styles, reversed/zero dimensions and same-track messages. Shared prefix fonts/styles/chrome/constants apply through existing services; interleaved preambles fail. Existing preprocessing still denies generated leading `@` rows, including macro-generated numeric marks, rather than widening directive/wrapper privileges. No evaluator/resources or additional wrapper is admitted.

Family/source integration records timing evidence only at actual syntax positions during existing raw-body preparation. Numeric `@` rows are retained for the timing parser, while raw note/action `@start`/`@end` text remains raw. Exact wrapper transitions and existing sequence/class/state/activity precedence remain covered by their original regressions. An invented `@starttiming` wrapper is not admitted.

## Projection, measurement, scene and bounds

`timingscene.rs` provides original native layout, with one monotonic linear projection `x = left + (time-min)*scale` shared by every track/connection/tick. Exact integer ordering is frozen before conversion. Robust tracks have distinct measured state levels and connected horizontal/vertical steps. Concise tracks have contiguous tapered interval polygons and measured labels. A separate measured pre-axis segment represents initial state; a measured final tail holds the last interval without inventing another timestamp.

Track titles/state text are measured before placement. Short concise intervals constrain global scale; final labels constrain tail width. Dimensions constrain horizontal interval width and each receives its own measured vertical caption/arrow band above the owning signal. Messages receive separate measured caption bands between owning tracks and real orthogonal paths ending at the actual projected signal points; transition projections are tested directly. Single-mark diagrams reserve long message caption width without an invalid clamp/panic. Sparse original 1/2/5 ticks are bounded to at most 12, skip crowded near-endpoint candidates, and never enumerate integer units across a large range. This differs intentionally from original GCD/truncation tick selection.

Same caller-memory GPUI-free render/measurement/control APIs and one shared Budget/Context/session; no engine runtime file/network/environment/clock/process/Java/JS/browser/dot access. Source256KiB, SVG16MiB, model10,000, depth64, work5,000,000/cooperative10s and independent worker10s/2GiB Unix remain unchanged. Label16KiB/query4096/finite-coordinate1,000,000 limits remain shared. Numeric arithmetic stays bounded; excessive measured width/height rejects before painting. Parser scans and string-key logarithmic lookups, state-at-time binary search, marks/state freezing, generated ticks/intervals/primitives/route vertices and layout propagation are charged. Long registry-code copies and repeated scene owner/target/name metadata charge aggregate text before retention; the 40,000-byte code/100 local-change regression now fails at the shared text limit. No dense time-range or cross-product grid is allocated. Cancellation is checked inside work loops and before/after shared host callbacks. Existing bounded writer and mandatory terminal SVG/resource validation remain in force; representative outputs parse through usvg.

## Exact source provenance

Personally inspected the actual authorized timing grammar/model/layout portions and actual special-state predicates under `/mnt/data/Projects/plantuml-mit/src` only. Full exact paths, SHA-256, destinations, behavior anchors and actual copyright/author lines are in [the committed manifest](../../../diagram-ports/provenance/plantuml-mit-1.2026.9.json). Controller independently matched all 18 additions to extracted source, exact source-JAR members, hashes and dated/author/contributor headers; selection did not change afterward.

| Authorized timingdiagram sources | Native responsibility/destination |
| --- | --- |
| TimingDiagramFactory | Registration inside UML; `source.rs`, `mod.rs` |
| command/CommandRobustConcise, CommandAtTime, CommandAtPlayer | Track/display/alias/current-player and numeric cursor grammar; `timing.rs` |
| command/CommandChangeState, CommandChangeStateByTime, CommandChangeStateByPlayerCode | Quoted/code states, separate cursor semantics and initial state; `timing.rs` |
| command/CommandConstraint, CommandTimeMessage, TimeTickBuilder | Owned endpoint grammar and distinct numeric relative bases; `timing.rs` |
| TimingDiagram, AbstractStatePlayer, ChangeState | Ordered ownership, initial replacement, timestamp-only first-wins comparison; `timing.rs` |
| TimingRuler, TimeConstraint | Linear projection and measured dimension semantics; `timingscene.rs` |
| graphic/PanelsRobust, PanelsConcise, PanelsState | Actual stepped levels/ribbons and projection semantics; `timingscene.rs` |

Every newly selected source credits `(C) Copyright 2009-2024, Arnaud Roques` / `Original Author:  Arnaud Roques`; full MIT grant remains in existing licence/notices. Generated MIT-edition files retain their historical upstream comments; no history was cleaned or relabeled. No GPL/rejected candidate/gen/h/Smetana/EPL/ELK/untraced asset was imported. Source-JAR local SHA-256 remains `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`, a measured local identity rather than authenticated Maven publication. No new JVM/original-renderer probes were run or claimed.

Exact original reference input copies and SHA-256:

- `timingbase.puml` ← `/tmp/ive-plantuml-native-oracles/timing-transitions.puml`: `b63360b1f508f465a335bece9b632db5353caf2a4614243e7eb94e4c9d326caf`.
- `timingdual.puml` ← `/tmp/ive-plantuml-specialized-probes.Kf1afs/timing-dual.puml`: `9fe1b95191e7741ab35c4c5cb3246c4acc00a3afc4b65dfc559dceb1442b0f54`.
- `timinglane.puml` ← sibling `timing-lane-mode.puml`: `f8f6daea3aa5830de880da7a96bf82bc3fb080e3077d93545ae5194444bff097`.
- Separate independently authored native viewer `timing.puml`: `c25f1e332dac7555692fb0942f36d63a02cc5316111511915b0c5ffe97e474ec`.

## RED/GREEN and final gates

TDD/verification skills informed implementation; no self-dispatched reviewer. Public tests use should_ and given/when/then. Initial timing suite had nine tests: eight meaningful RED at the existing `@` source gate and one negative control passed; dedicated implementation made nine GREEN. Later meaningful RED→GREEN cases cover quoted-arrow/player-name ownership, caption grammar words, tab/multiple-space tokenization, single-mark long-caption panic, ordinary special-looking names, unsupported markup in losing equal-time change, empty track display, repeated long owner metadata, and missing viewer example. The first generic post-scan timing gate broke three existing raw-note constants regressions; retaining syntax-position evidence made all 31 constants and other prior PlantUML controls GREEN. Exact debug-quoted error checks/shared zero-depth option spelling and a floating-point projection equality tolerance were corrected as test assumptions, not claimed feature REDs. Seven direct model/scene tests and actual native paint checks strengthen contracts; not every later passing assertion had an independent RED cycle.

Final evidence directory: `/tmp/ive-plantuml-3e.BjavJ8/`. Every Cargo command used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` from standalone `diagram-ports/`; scoped escalation bypassed the known bwrap RTM_NEWADDR startup failure, without network access. After last semantic/budget edit:

```sh
cargo test --locked --offline --workspace
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
cargo build --locked --offline --release
cargo deny --config ../deny.toml check licenses sources
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
git diff --check
```

All exit 0. **339 Rust tests passed, zero failed/ignored**, plus all seven separate hostile-worker scenarios (blocked stdin/stdout, stderr flood, oversized frame, early exit, memory, cancel). Breakdown: engine-private71; compatibility11; Mermaid families37; Mermaid21; PlantUML33; constants31; control23; styles17; timing21; tree/data18; viewer-private42; export5; native-lifetime3; worker6. Final logs: `tests-final.log`, `clippy-final.log`, `release-final.log`; initial 336-test gates are historical and superseded. Ten Node audit tests pass. PlantUML audit:131 selected source/JAR entries,19 fixtures,28 modules. Merman:656 imported files/four recorded patches unchanged. Deny: licenses/sources OK. Inherited vendored Atomic::fetch_update deprecation persists; no new product warning. No product edits after final gates.

Actual native paint test log `native-geometry.log`: Liberation Sans resolves to Liberation Sans;16 actual painted text runs at each6/14/96px case. Concise ink fits its own ribbon borders; every text run is inside canvas. Direct private tests assert exact IDs/times, sorted first-wins changes, unchanged cursor after connections, common monotonic x, robust vertical steps at concise boundaries, measured multiline dimension/ribbon reservations, initial pre-axis positions, actual message endpoint y/x and separate caption bands. Public tests cover exact references, whole-input rejection, custom source/model/depth/work/output limits, sparse deterministic output, pre-cancel and in-flight host cancellation (exactly one callback).

Final release CLI successfully exports all three exact numeric references to evidence-directory `timingbase.svg`, `timingdual.svg`, `timinglane.svg`. Exact original named and decimal probes each exit1 at line3 with offending `@0 as :origin` / `@-2.5 as :begin` and explicit integer-only diagnostic; their logs are `named-rejected.log`, `decimal-rejected.log`. They are rejection evidence, never decimal/named-time support claims.

## Genuine background capture and handoff

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-timing --capture /tmp/ive-plantuml-3e.BjavJ8/timing.png
```

Exit0; genuine1360×880 GPUI readback inspected by implementer and controller. Actual host count **16 native measurements / 0 deterministic fallbacks**, from `capture-timing.log` and status bar. PNG SHA-256: `166d2bd3a2437a8efd4bb84876c8d754b7d73b5e3dd69d60e4985cbb81a2b909`. Closed/Open/Closed robust steps align with Waiting/Running/Done ribbon boundaries; measured6→15 dimension and V@6→C@8 message attach coherently; captions/title/footer/ticks have visible clearance. No host containment-expansion warning or fixture-padding correction. All viewer processes self-quit; final anchored debug/release diagram-viewer lookup was empty. No IVE instance, user focus/mouse/window activation, xdotool or user-process interference.

Residuals: intentionally bounded integer/narrow timing grammar; right-hand/center projection policy and original sparse ticks/orthogonal routes differ from upstream. Complex routes can cross intervening signals; general collision avoidance and original theme/pixel parity are not claimed. Shared approximate baseline/unavailable-font mismatch limitations remain. No known failing gate or additional unresolved blocker; independent frozen-head SPEC/QUALITY review is the controller's next gate. No later family was started.
