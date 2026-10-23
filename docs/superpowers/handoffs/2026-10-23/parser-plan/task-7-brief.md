### Task 7: State grammar cutover

**Files:** Create `parser/state.rs`, `grammar/state.lalrpop`; modify state syntax/dispatch/tests; preserve stategraph.

**Interfaces:** generated state syntax lowers into `state::State`; retain forward-reference resolution and scoped pseudo/composite identities.

- [ ] Add generated tests for forward references, identical local names in separate composites, initial/final scope, descriptions, supported closers/arrows and dangling/crossed/unknown suffix failures. Retain current private ID/ancestor and real geometry assertions.
- [ ] Run generated structure/EOF/opener-depth RED; advanced concurrent/history/subtype features remain unsupported until their later family task.
- [ ] Remove handwritten declaration/transition/closer parsing; retain two-phase semantic resolution and scoped construction.
- [ ] Run state/control/styles/constants/common gates, baseline and nested-state native capture.
- [ ] Self-review/commit/report and inventory residual code as semantics/layout, not hidden syntax.

