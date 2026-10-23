### Task 8: Structured activity grammar cutover

**Files:** Create `parser/activity.rs`, `grammar/activity.lalrpop`; modify activity syntax/tests, preserve activitytile.

**Interfaces:** generated recursive blocks build existing `activity::Block`/`Instruction`/`Branch`/`Condition` values.

- [ ] Add generated tests for actions/start/stop, nested if/elseif/else, while/repeat, fork join/merge and groups, asserting existing continuation/live-dead topology. Crossed/unterminated blocks, illegal continuations and unsupported syntax must reject completely.
- [ ] Run recursive-structure RED with max-depth/max+1 before retention, tiny model/work and controlled cancellation. Preserve raw action/expanded terminator tests.
- [ ] Replace handwritten recursive command/condition/guard syntax; raw action text is payload only after a valid action production.
- [ ] Run activity/control/styles/constants/common gates, baseline and branched/loop/fork native captures with real port/ink assertions.
- [ ] Self-review/commit/report; no generic graph or legacy activity fallback.

