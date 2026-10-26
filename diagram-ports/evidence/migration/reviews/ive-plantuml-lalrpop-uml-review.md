### Spec Compliance

- ❌ Issues found: the UML quote decoder changes the existing 16KiB raw quoted-token limit to a decoded-text limit, admitting previously rejected source (`diagram-ports/engine/src/plantuml/parser/uml.rs:407`; Important R1 below).
- ✅ The remaining reviewed cutover matches the brief: actual declarations/endpoints/aliases/groups/members/notes/direction/EOF are generated productions (`parser/grammar/uml.lalrpop:5`); the runtime route constructs the generated model before layout (`plantuml/mod.rs:97`). No raw command reaches the typed builder, retry, legacy UML fallback or partial recovery remains (`parser/uml.rs:175`, `uml.rs:156`, `generated/uml.rs:1895`).
- ✅ Graph implementation changes are limited to test model setup; shape/layout assertions and immutable fixture records are preserved (`plantuml/graph.rs:489`, `provenance/plantuml-mit-1.2026.9.json:2321`). No dependency/vendor/default-limit/public-API changes appear in the package.
- ⚠️ Cannot verify from this task diff: combined baseline equality, all16 integration gates and native/empty-PATH/nested-graph/colored-description captures. These are explicitly central-owned; controller must complete them after integration. Final private-generator read-only check, fmt and source/closure audit results are reported but have no separately named final logs here; `/tmp/ive-task5-generation.log` records the earlier six-parser generation.

### Strengths

- Generated grammar preserves contextual and quoted keyword names, declaration alias ordering, role positions and constrained notes while leaving finite arrow/value recognition lexical (`parser/grammar/uml.lalrpop:12`, `parser/grammar/uml.lalrpop:45`, `parser/uml.rs:466`). Builder retains scope/ID/order/mixing/implicit-class resolution without token-slice syntax (`uml.rs:156`, `uml.rs:295`).
- Concrete admission accounts for token/literal/adaptor storage, generated stack and diagnostic growth, and potential semantic arenas before generated entry (`parser/uml.rs:534`, `parser/uml.rs:614`, `uml.rs:122`). Semantic vectors use checked fallible growth; dynamic duplicate/ambiguous/undeclared errors reserve bounded text before formatting (`uml.rs:222`, `uml.rs:365`). Controller independently verified the actual379-state/36-terminal table witness, including ranged goto arms, maximum4 reductions/30 expected terminals and depth64/65 bounds.
- Tests inspect model identity/topology/roles, exact safe fills and icon rejection, generated suffix/EOF failure, origin mapping and delivery/action cancellation (`parser/uml.rs:664`, `parser/uml.rs:786`, `parser/uml.rs:814`, `parser/uml.rs:873`). These exercise real generated parsing and semantic construction.
- Exact generated/grammar hashes reconstructed from the diff match the manifest: `20ba6dca03dc4b42ae98666dd90431c132254a4dfb1bbfa80775982a5398d8ef` and `4454c4ab05bfc5ae7ff9af1af3aebb4fd2b81c6d6c5c11cfd7509a589650010c`. Source provenance adds only relevant existing authorized MIT source mappings and retains hashes/credits/grants (`provenance/plantuml-mit-1.2026.9.json:2321`).

### Issues

#### Critical (Must Fix)

- None found.

#### Important (Should Fix)

- **R1 — Preserve the raw quoted-token boundary.** `diagram-ports/engine/src/plantuml/parser/uml.rs:407` bounds `value.len()` after escape decoding. The legacy primitive bounds raw interior indices before decoding (`diagram-ports/engine/src/plantuml/lexer.rs:89`–91). Consequently a class display containing8,193 escaped double quotes has16,386 raw interior bytes but only8,193 decoded bytes: the new path passes this guard and its later decoded copy/label checks, while the previous path rejects it. Preprocessing's16KiB guards apply to definitions, and header/preamble/body admission impose no corresponding raw quote bound. This changes the exact rejected subset and weakens an existing input bound. Retain the previous raw-byte checks in the new decoder alongside decoded allocation checks, and add focused boundary controls for8,192 versus8,193 escaped quotes (including a quoted endpoint/role).

#### Minor (Nice to Have)

- Existing vendor diagnostic noise remains in both final logs: deprecated `Atomic::fetch_update` at `diagram-ports/vendor/merman-core-0.9.2/src/operation.rs:449` and `diagram-ports/vendor/merman-render-0.9.2/src/environment.rs:585`. This is pre-existing and outside this task's permitted vendor scope; carry it into central final triage.

### Checks and Scope

- Read the prepared diff once in authored chunks; mechanical generated tables were checked through exact hashes/runtime contracts and the controller's independently reproduced witness. No changed-source reread, source/index/HEAD mutation, package suite, generator run or native process was performed.
- Named final logs show319 passing tests (174+40+12+44+23+26) and successful engine-targeted strict Clippy, with the vendor warnings above (`/tmp/ive-task5-engine-final.log`, `/tmp/ive-task5-clippy-final.log`). No suite was rerun.
- Focused unchanged-code checks addressed specific risks: legacy `lexer::name`/validation for alias and endpoint parity; `lexer::quoted`, header/preamble/preprocessing for the raw quote-bound finding; `context::push` and `Budget::text` for actual allocation/admission contracts. Existing `/tmp/ive-task5-layout.rs` matches the before/after error variants; the Cow change leaves the dominant RenderError layout unchanged. Central removal of the legacy state's final `lexer::name` caller remains outside this branch.

### Assessment

**Task quality:** Needs fixes.

**Reasoning:** The generated structural cutover, typed semantic boundary, resource admission and compatibility tests are otherwise well built. Restore the raw quoted-token bound and pin it with focused regression coverage before accepting the task.
