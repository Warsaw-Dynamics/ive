# Task 3F: Native deterministic Gantt

Read this first: requirements for the Gantt deliverable of detailed native-plan Task7. HOLD until explicit release from preceding reviewed commit. Only Gantt new family here; other Task7 families remain separate gates.

## Binding contracts

Owner full-port attempt, extra permissive dependencies allowed, standalone pgui viewer first; accurate subset/fidelity ledger, IVE root product unchanged. ONLY PlantUML translation source `/mnt/data/Projects/plantuml-mit/src` MIT1.2026.9; no GPL/rejected candidate/gen/h/smetana/EPL/ELK or untraced assets. Exact source SHA/destination/behavior and actual notices in current provenance. One implementer/no subagents, apply_patch/TDD, AGENTS should_ given/when/then.

Engine source-in-memory/GPUI-free, no runtime files/network/environment/clock/locale/processes/external tools. Reuse actual shared measured-label/typed-scene/style/SVG services, not raw Mermaid conversion. Preserve public API, other families, controls/measurement provenance/worker isolation. Source256KiB/SVG16MiB/model10,000/depth64/work5,000,000/cooperative10s, worker independently10s/2GiB Unix. Existing label/query/finite-geometry bounds hold; budget date spans/generated calendar ticks/cells/routes before allocation, checked arithmetic and cancellation inside loops.

Read `/tmp/ive-plantuml-specialized-semantics.md` fully with exact65-file source ledger; inspect authorized Gantt grammar/rules actually translated. Existing reference/probe artifacts remain untouched. Gantt date/task solving is not generic graph layout.

## Semantic minimum

- Exact @startgantt/@endgantt (and @startproject only if source-verified exact matching semantics). Require explicit Project starts YYYY-MM-DD before tasks; no ambient today/timezone/date lookups. Validate real civil dates/leap days, bound year/ordinal/span and numeric lexeme before conversion. Use deterministic civil-day ordinal arithmetic, not day-by-day solving for the default calendar.
- Ordered distinct task IDs vs display/aliases; durations in admitted positive bounded whole days via lasts/requires; explicit start dates, predecessor start/end links and supported bounded day offsets. Resolve references according to source declaration semantics; unknown aliases, conflicting specifications and cycles fail descriptively. Default seven days OPEN, not invented weekday closure.
- Half-open task interval [start,endboundary), actual occupied end=endboundary-1. Successor at prior end starts at exclusiveboundary. Milestone is real diamond at civil day; happens at predecessor END uses lastoccupiedday, including exact offset adjustment, not successor boundary. Model or explicitly reject using milestone start/end as another dependency until exact one-day source semantics exist.
- Original commands capture dependency dates immediately and can retain snapshots after later predecessor edits. Implement ordered snapshot semantics or the report's conservative freeze-after-reference restriction with clear errors; don't silently recompute a final reactive DAG and claim original behavior. Retain dependency ownership separately for drawn arrows and bounded cycle validation. Restriction is native policy, not upstream proof.
- Dedicated immutable schedule model, checked validation/evaluation, measured date-axis grid/bar drawing. Actual duration-width bars, month/day labels and milestone diamonds, source-order task rows and routed owned dependencies. Measured display labels and endpoint/tick reservations, finite geometry/nonoverlap/containment. No imported calendar/theme data or fixture-coordinate answers.
- Explicitly reject unsupported calendars/closed days/resources/hours/weeks/months/percent/printscale/groups/styles/today/builtins instead of dropping commands. Shared supported styles/chrome either genuinely affect body/layout or receive precise unsupported errors.

## Tests and delivery

RED tests: alias identity/source row order; exclusive end successor; end milestone lastoccupiedday and offsets; seven-day default; valid leap crossing and invalid dates; immediate-reference snapshot/freeze policy; missing/late project date; unresolved/conflicting/cyclic schedule; real bar-width/dateprojection/diamond/dependency geometry; longlabels and calendar span containment; huge numeric/sparse span custom model/work/SVG limits/cancellation/determinism under different ambient environments.

Use exact fresh base Gantt fixture and `/tmp/ive-plantuml-specialized-probes.Kf1afs/{gantt-alias-milestones,gantt-date-leap}.puml`, preserving hashes. Reference DraftNov2–3, BuildNov4–6, reviewNov6, publishNov7 must follow computed dates, never hardcoded fixture special cases. Record original geometry differences.

Add honest viewer example via existing single worker. Background real capture of alias+duration+dependency+milestones spanning dates, inspect shapes/labels/native counts and stop instances. Focused tests then full locked/offline test/clippy/fmt/release/source-license audits. Commit scoped files; `task-3f-report.md` here records base/head, admitted/rejected syntax, typeddate/schedule interfaces/source traces, RED/GREEN counts/commands, capture and residuals. Review your own diff before reporting, but do not dispatch a review agent; the controller owns the fresh independent spec+quality gate.
