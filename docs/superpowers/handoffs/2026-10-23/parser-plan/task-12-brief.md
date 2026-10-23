### Task 12: Timing/Gantt grammar cutovers and assigned boundary repairs

**Files:** Create `parser/{timing,gantt}.rs`, `grammar/{timing,gantt}.lalrpop`; modify their syntax/tests; preserve scenes/calendar/cursor semantics.

**Interfaces:** generated commands build existing `timing::Timing` and `gantt::Gantt`; checked arithmetic/reference semantics remain builders, not rewritten source phrases.

- [ ] Add generated tests pinning timing exact integers/separate cursors/duplicate changes/constraint and message bases; Gantt explicit dates/duration/task aliases/dependencies/half-open bars/milestones/freeze-after-reference. Add timing robust/concise name regressions and terminated-versus-unterminated trailing Gantt comments.
- [ ] Run assigned bug RED if not already fixed by shared front end, plus new generated command/endpoint/full-EOF RED. Do not invent a second RED for an already repaired common EOF check.
- [ ] Remove keyword dispatch, bracket/verb/time/complement source splitting; generated typed endpoints feed existing checked arithmetic. No ambient calendar/clock/resource expansion.
- [ ] Run timing/Gantt/constants/control/common gates, baseline and genuine native tracks/calendar captures. Record the two assigned Minors resolved with exact controls.
- [ ] Self-review/commit/report with all unchanged integer/date/range/grammar exclusions explicit.

