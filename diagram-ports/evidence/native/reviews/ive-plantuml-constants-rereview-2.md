**"A constant-generated action terminator leaves preprocessing in raw ownership and permits a generated closing wrapper." — ADDRESSED.** `diagram-ports/engine/src/plantuml/preprocess.rs:57` now determines only current-row raw ownership before normalization, without prematurely committing the next body state. After literal expansion, `preprocess.rs:133` charges the additional scan to the same Budget and updates body state through `consume_raw(expanded)`. Consequently, the exact `FIN`/`END` reproduction closes the action when `FIN` inserts `;`; the following `END` is a syntax row and reaches generated-wrapper denial at `preprocess.rs:124`. The fix report records the exact release export failing with `generated directive or wrapper`, authored use line 7, column 1 and definition line 3, column 1. The regression `should_reject_generated_wrapper_after_expanded_action_terminator` explicitly asserts both denial and those origins, and its fresh test-log result is passing.

### New Breakage in the Fix Diff

**None found.** The change preserves current-row raw ownership before comment handling while reconciling next-row ownership with the expanded input consumed by final source parsing. The same reconciliation handles action-opening and action-continuation semicolons, generated `end note`, and an authored `end note` changed into literal text by expansion. It does not rescan inserted values for constants or change origin/storage representations. The new scan is charged before `consume_raw` against the existing shared Budget. The generated-wrapper guard remains active for syntax rows, while actual raw wrapper-looking label rows retain their literal ownership.

### Out-of-Scope Observations

**None.** Original R1 and R2 remain closed. No unchanged-code issue or broader compatibility claim was added to this gate.

### Checks and Evidence

- **Scope checked:** fix base `50da37e108256003388361c0b5ac5a3da76735ff`, head `6abc18fb22d908e2c257267850bb387341c7ea91`; only `engine/src/plantuml/preprocess.rs` and `engine/tests/plantuml_constants.rs` changed. Read the complete new FIX ROUND 2 appendix of the same implementation report and the prepared `review-50da37e..6abc18f.diff` once. The previously read complete brief, binding constraints and prior review remain governing evidence. No git commands, changed-file rereads, product/index/HEAD writes, or subagents were used.
- **Recorded coverage checked:** `/tmp/ive-plantuml-3d2-fix2.TOSycC/tests.log` records all four new tests passing: `should_reject_generated_wrapper_after_expanded_action_terminator`, `should_close_expanded_opening_and_continuation_actions_before_normal_syntax`, `should_close_expanded_note_terminators_before_normal_syntax`, and `should_keep_note_rows_raw_when_expansion_removes_authored_terminator`. Constants are 31/31; suite result lines total 309 Rust tests, zero failures/ignored. The exact bypass test includes useful diagnostic-origin assertions rather than merely accepting any error.
- **Reported verification distinguished:** the report records meaningful RED 27 passed / 4 failed, GREEN 31/31, seven worker scenarios, nine audit tests, strict Clippy/fmt/release/deny/source gates and the exact release-export rejection. The controller independently verified that all five fresh native capture hashes match the previously inspected references. These suites, release export and captures were not rerun during this scoped re-review.
- **Focused probes:** none required. The prepared diff and named recorded tests answer the specific ownership/terminator doubt; no additional unanswered fix concern justified another probe.

### Verdict

**Fix round: All findings addressed, no new Critical/Important breakage.**

**SPEC verdict: ✅ Pass for this scoped fix at `6abc18fb22d908e2c257267850bb387341c7ea91`.** The generated-wrapper bypass is closed, with ordinary generated action/note terminators and the inverse note transition covered. Original R1/R2 remain addressed; the prior subset and diagnostic limitations remain unchanged.

**QUALITY verdict: Approved for this scoped fix.** Current-row classification and next-row state now agree with their distinct roles, reuse the shared transition helper, and retain shared budget/origin/public API contracts. This verdict does not constitute a fresh whole-branch review or full PlantUML/TIM parity claim.
