# Task 3F — deterministic native Gantt

Status: implemented and frozen for controller-arranged independent SPEC/QUALITY review. This admits one bounded Gantt family, not full PlantUML compatibility.

- Released reviewed base: `b3cef28c6f914428d8bc2e3ca07b555a3163d9bb`.
- Frozen head: `d84c31464d6c85bbdb4a78e69d64d3578e63d3db` — `Add bounded native PlantUML Gantt schedules`.
- Existing worktree `/mnt/data/Projects/ive/.worktree/diagram-ports`, branch `diagram-ports`; tracked worktree clean after commit. Exactly19 scoped standalone files committed; no root IVE product, dependency, lockfile, vendor, branch/worktree, runtime-service, later-family, merge or push changes. No subagents/reviewers dispatched.
- Full [brief](task-3f-brief.md), [constraints](native-global-constraints.md), [Task7 extract](task-3e-plan-extract.md), specialized semantics/exact65-file ledger and preceding [3E report](task-3e-report.md)/[3D2 report including both fixes](task-3d2-report.md) read. Unchanged shared measured text/style/chrome/origin/worker contracts and prior fidelity limitations remain those reports and their linked [3D contract](task-3d-report.md). TDD and verification skills informed the work.

## Model, grammar and policy

`gantt.rs` owns the dedicated frozen `Gantt { project_start, tasks, dependencies }`, bounded exact `CivilDay(i32)`, source-ordered task identities/display strings, distinct Bar/Milestone kinds, start/exclusive-end ordinals and owned typed Start/End dependencies with signed offsets. Parser-only builders/registries capture dates in statement order; final layout receives the immutable schedule. No generic graph solver or PlantUML-to-Mermaid rewriting is involved.

Admitted exact wrapper: `@startgantt`/`@endgantt`, without output filenames or wrapper aliases. Prefix shared styles/fonts/chrome/literal constants use the existing services. `Project starts YYYY-MM-DD` must be the first body statement before any task; Project spelling is case-insensitive, other admitted keywords are the documented lowercase forms. ISO dates validate genuine Gregorian dates/leap centuries in years0001–9999. Both start and exclusive end must lie inside that year range; a duration occupying9999-12-31 would require an out-of-range exclusive boundary and rejects. Starts before the explicit project date reject. These are deliberate narrower native policies.

- `[display] as [id]` separates identity from display; `[id]` later retains its original label and row. Without an alias, display is identity. Display text is not an automatic second identity. Conflicting alias displays reject.
- `[task] lasts N day|days` and `requires` accept positive whole ASCII numbers of at most four digits, at most7300. Default tasks have one day at project start. An explicit predecessor duration is required before a reference captures it.
- `[task] starts [on|at] YYYY-MM-DD`; `[task] starts [at|after|with] [id]'s start|end`; `[task] starts N day|days before|after [id]'s start|end`. Complete grammar is consumed. Unsupported preposition combinations reject.
- `happens` accepts explicit dates or the admitted start/end complements, with optional on/at. It creates a real diamond at its civil day. Milestone references retain ownership but do not invent arrows: upstream happens commands do not add a drawn constraint. Ordinary start links draw owned dependencies.
- Bars are half-open `[start,end)`. A two-day task starting November2 occupies November2–3; its successor begins November4. An END milestone uses `end + offset − 1`; a START milestone uses `start + offset`. Dependencies on a milestone's own START/END reject until source one-day dependency semantics are implemented.
- References must already exist. The conservative native admission policy freezes all predecessor schedule fields after capture. Later duration/start edits, repeated specifications, unknown references and cycles fail descriptively. This preserves the admitted immediate capture behavior while explicitly rejecting broader upstream snapshot updates; it never recomputes a final reactive DAG. Cycle rejection and explicit predecessor duration are native policies, not claims about upstream detection.

The default calendar has all seven days OPEN. Dates/durations use checked bounded ordinal arithmetic, not day-by-day solving. Total schedule span is at most7300 days. Calendar/closed-day/working-day/resource rules, hours/weeks/months/mixed/fractional units, groups, percent completion, task colors, print scales, today/date builtins, additional natural-language combinations and unknown substantive suffixes reject with offending input. Shared supported prefix font/style/chrome/constants affect measured body/axis labels; interleaved preambles reject.

## Measured layout, safety and provenance

`ganttscene.rs` provides original native measured layout: first-column display labels, source-order separated rows, common civil-date projection, true duration-width bars, four-point diamonds centered on the owning civil-day cell, and owned orthogonal start-dependency routes. Month/year and weekday/day labels use fixed English strings without locale access. At most12 sparse ticks are selected arithmetically; measured label widths reserve axis scale and endpoint clearance. Direct tests verify bar widths/boundaries, diamond projections, route endpoints/ownership, nonoverlapping large multiline rows/ticks and scene containment. Original first-column spacing, sparse ticks, bar heights/colors and routing differ from upstream. General route crossing avoidance and original pixels are not claimed.

One existing Budget/Context/session and terminal resvg-safe/resource pipeline remain in force. Source256KiB/SVG16MiB/model10,000/depth64/work5,000,000/cooperative10s, independent worker10s/2GiB Unix, labels16KiB/queries4096/coordinates1,000,000 and existing viewer raster bounds are unchanged. Input scans/string-key lookups, cycle walks, date conversions, rows/ticks, generated text/metadata and route vertices charge shared work/text/items before retention; date loops have fixed small bounds and schedule spans are never enumerated. Cancellation checkpoints stay inside growing loops and around host callbacks. No runtime IO/network/environment/clock/locale/process/JVM/JS/browser/dot access was added to the GPUI-free engine.

Personally inspected actual authorized Gantt grammar/evaluation/default-calendar/bar/diamond rules under `/mnt/data/Projects/plantuml-mit/src` only. The [manifest](../../../diagram-ports/provenance/plantuml-mit-1.2026.9.json) adds17 exact paths/hashes, Rust destinations, translated behavior and actual copyright/author lines. Source selection remained unchanged after the controller independently matched all17 against extracted files and exact JAR members/headers. No GPL/rejected/gen/h/Smetana/EPL/ELK/untraced asset was imported. All17 credit `(C) Copyright 2009-2024, Arnaud Roques` and `Original Author:  Arnaud Roques`; the existing full MIT grant is retained and the notices explicitly cover Gantt. Generated MIT-edition historical comments remain unchanged. Local source-JAR SHA remains `88e30e43639555a6963c270bc6245906b81e8cc65140645f416795d0e0bc71c3`, not authenticated Maven publication.

| Authorized source groups | Native destination/behavior |
| --- | --- |
| command/NaturalGanttCommand; lang/SubjectTask, SubjectGantt, Verbs | `gantt.rs`: ordered execution, task/project/verb grammar, aliases and milestone subtraction |
| lang/ComplementDuration, ComplementBeforeOrAfterOrAtTaskStartOrEnd, AbstractComplementTaskInstant | `gantt.rs`: admitted day durations, existing-reference lookup, endpoints and signed offsets |
| lang/DayPattern, TimeResolution | `gantt.rs`: ISO-only validated civil-date admission |
| core/TaskCode, TaskInstant, TaskImpl; solver/Solver | `gantt.rs`: identity/default load/end boundaries and explicit narrower snapshot/freeze policy |
| GanttDiagram; OpenClose | `gantt.rs`: source order/project-before-task and seven-open-day defaults |
| draw/TaskDrawRegular, TaskDrawDiamond | `ganttscene.rs`: duration bars and actual diamond semantics; layout coordinates are original native work |
| Existing core/DiagramType | `source.rs`: exact Gantt wrapper classification; shared preprocessor recognizes its authored boundaries |

Exact reference-input copies and SHA256:

- `ganttbase.puml` ← `/tmp/ive-plantuml-native-oracles/gantt-dated-dependencies.puml`: `0aa714ffdc1be9ea700553344e362415e56b5dd4891dc256f66f60bf40325a04`.
- `ganttalias.puml` ← `/tmp/ive-plantuml-specialized-probes.Kf1afs/gantt-alias-milestones.puml`: `652c8648afbf5814d01c6fde36e24c19924d51c2ab1422ea67800ec42ae6a0fe`.
- `ganttleap.puml` ← sibling `gantt-date-leap.puml`: `c2438204d02f7a3a3a9653bc973ad749170f0b667ebc0a804fa5b8568d6b0128`.
- Separate authored viewer `gantt.puml`: `cd5df287b33ecdc5e551156292fa16067f18bdc440a8a023d73739845013332c`.

Original DraftNov2–3/BuildNov4–6/ReviewNov6/PublishNov7 and leap-span/ShipMar2 expectations are computed from source rules and directly asserted, without fixture-coordinate branches. Native deterministic alias viewBox is704.43×422.8 versus original328×146; native leap517.69×342.8 versus original276×129. These differences are honest native layout evidence, not pixel parity. No new JVM probes or modifications to prior reference artifacts were made.

## RED/GREEN and final evidence

Initial public suite:10 meaningful failures at missing Gantt wrapper/constant admission. Initial native implementation made9/10 GREEN; the remaining test expected SVG exhaustion at1024 bytes but reached the intentionally shared expanded-text bound first. Raising that test's SVG bound to4096 made it specifically reach the SVG writer. Later exact references/date/cancellation/limits and direct typed-model/scene tests strengthen coverage; not every later passing assertion had an independent RED cycle. Viewer registration had an actual unknown-example RED before GREEN. The exact-source allowlist had an unauthorized-SubjectTask RED before GREEN. Self-review caught over-admitted start prepositions; a new meaningful RED accepted `starts on [A]'s end`, then strict complement admission made it GREEN. No known failing gate remains.

Focused commands included `cargo test --locked --offline -p diagram-engine --test plantuml_gantt` (final16/16), engine private Gantt tests (5/5), viewer example selection, and actual native Gantt paint at6/14/96px. Native paint resolved Liberation Sans to Liberation Sans and contained15 actual painted runs at each size. Shared worker/cache lifetime coverage now includes the Gantt viewer fixture. All tests use should_ and given/when/then.

Final evidence directory: `/tmp/ive-plantuml-3f.ZNROdJ/`. After the last product edit, from standalone `diagram-ports/`, every Cargo command used `CARGO_TARGET_DIR=/mnt/data/Projects/ive/target` and scoped escalation only bypassed the known bwrap RTM_NEWADDR startup failure:

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

All exit0. **362 Rust tests passed, zero failed/ignored**, plus all seven separately reported hostile-worker scenarios: blocked stdin/stdout, stderr flood, oversized frame, early exit, memory, cancellation. Breakdown: engine-private76; compatibility11; Mermaid families37; Mermaid21; PlantUML33; constants31; control23; Gantt16; styles17; timing21; tree/data18; viewer-private44; export5; native-lifetime3; worker6. Eleven Node audit tests pass. PlantUML audit148 source/JAR entries/23 fixtures/30 modules; Merman656/four recorded patches. Deny licenses/sources OK. Logs: `tests-final.log`, `clippy-final.log`, `release-final.log`, `deny-final.log`, `audit-tests-final.log`. Only inherited vendored Atomic::fetch_update deprecation remains. Self-review found no product edits needed after these final gates.

Final release exports of all three exact fixtures succeeded. `alias-east.svg` with `TZ=Pacific/Kiritimati LC_ALL=C` and `alias-west.svg` with `TZ=America/Los_Angeles LC_ALL=C.UTF-8` are byte-identical, SHA256 `0281b046374a38eb36de42c8251f7aed9f8c0466d780b1590ecf832055421501`. They verify timezone/locale independence, not a simulated ambient-clock change. Engine source inspection confirms no ambient clock access.

## Actual background capture and handoff

```sh
/mnt/data/Projects/ive/target/release/diagram-viewer --background --language plantuml --example plantuml-gantt --capture /tmp/ive-plantuml-3f.ZNROdJ/gantt.png
```

Exit0, genuine1360×880 GPUI readback; implementer and controller inspected the actual image. SHA256 `a3298930ca0121f5d46327b322cee05fcb3b48d6378b78f3ec61c710b993cac5`. `capture-gantt.log` and status bar report **18 actual native measurements / 0 deterministic fallbacks**. Draft's two-day bar and Build's three-day bar share the correct exclusive boundary; the owned dependency arrow is clear; ReviewNov6 and PublishNov7 diamonds align with their civil-day ticks. Display labels, bold draft, title/footer, pale node fills and dark strokes are contained with visible clearance. No containment-expansion warning or fixture-padding correction occurred. The viewer self-quit; final anchored debug/release viewer process lookup was empty. No IVE instance, xdotool, focus activation, real mouse use or user-state/process interference.

Residuals: bounded day-only/ISO grammar, conservative freeze policy, explicit predecessor-duration requirement, milestone-as-predecessor rejection, sparse English calendar labels, simplified routes/themes and original baseline/font-fallback limitations. Full calendar/resource/natural-language Gantt and full PlantUML remain incomplete. Timing's already logged keyword-shaped identifier Minor was not changed. Independent frozen-head SPEC/QUALITY review is the controller's next gate; no later family was started.
