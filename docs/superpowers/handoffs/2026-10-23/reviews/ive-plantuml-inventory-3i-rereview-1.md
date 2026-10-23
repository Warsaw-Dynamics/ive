# Task 3I fix round 1 — independent scoped re-review

**I1 verdict: ADDRESSED. SPEC: APPROVED. QUALITY: APPROVED.**

No Critical, Important or Minor fix-introduced findings. No blocking work remains from this review. The controller owns Task 3I approval and the next-family release.

## Scope

Reviewed only fix base `1bf1f46c762d9ff4a5082aa688c6bff3dfaf2c7a` through frozen head `3231bb195bbf183e5e961135f9889a9e872bd611`, using the sole prepared `review-1bf1f46..3231bb1.diff`, read once. Re-read the same complete Task 3I brief and binding global constraints, and read the complete FIX ROUND 1 appendix in `task-3i-report.md`.

The fix changes only `diagram-ports/engine/src/plantuml/hcl.rs` and `diagram-ports/engine/tests/plantuml_inventory.rs`: ten parser lines and five public regression/control tests. No untouched family or earlier implementation was re-reviewed. No product, progress, index, HEAD or branch changes; no subagents or worktrees. Only this report was written.

## Per-finding judgment

### I1 — HCL block-name accumulation consumes unsupported prefix statements across physical lines

**ADDRESSED.** `hcl.rs:48` records the originating header line after the initial whitespace skip. The guard inside the header-name loop rejects a next name token after `peek()` crosses a physical newline. The error points to the original header line, includes the accumulated offending header, and identifies the later physical line. It runs before reading/allocating the next token and before single-block lowering can discard the combined name.

This fixes the cause rather than filtering keywords or hiding output. A genuine opening `{` on a following line still ends the name loop and reaches ordinary object parsing. A subsequent name token on another line fails, including after a completed earlier block. Quoted command-named blocks remain legal. Shared `peek()`, field/value/container parsing, lowering, ownership, budgets and rendering are unchanged.

The added tests distinguish the relevant paths:

- `should_reject_hcl_title_statement_before_a_separate_block`: exact title-only trigger; asserts originating location, full header and block-header diagnostic.
- `should_reject_hcl_style_statement_before_a_separate_block`: independent style-only trigger with equivalent diagnostic assertions.
- `should_reject_hcl_combined_prefix_statements_without_lowering_them_away`: exact combined trigger; asserts original line, header and physical-line diagnostic.
- `should_reject_hcl_header_token_continuation_after_any_complete_block`: tests the later-header position and ordinary incomplete headers, preventing a reserved-word-only repair.
- `should_keep_hcl_command_names_split_opening_braces_and_nested_value_newlines`: positive control for title/footer names, split braces, a hash-comment row, multiline body/value containers, ordered scalar rows and five genuine compartment links.

The intentionally enforced boundary is name tokens on one physical header line; braces may occur later. That matches the documented physical-statement policy and the allowed bounded HCL subset. It does not accidentally impose a newline ban on values, objects or arrays.

## Verification evidence and limits

The appendix records a corrected meaningful baseline of 23 passing / 4 failing public inventory tests, followed by 27 passing / 0 failing after the guard. The separate corrected test hand-count is transparently excluded from the product RED claim.

Supplied final evidence is 455 ordinary Rust tests plus seven separately counted real worker faults, fourteen Node tests and all strict gates/audits, independently freshly repeated by the controller. The exact original title/style/combined inputs now exit 1 with originating line and offending header; the valid command-name input exits 0. The affected native HCL capture remains 21 native / 0 fallback with exactly the original SHA and pixels. Unaffected captures were not rerun.

These are supplied implementer/controller checks, not suites independently rerun by this reviewer. The complete scoped diff resolves the specific uncertainty without requiring another diagnostic, so no suites, audits, captures, or original-renderer probes were repeated. No additional outside-diff inspection was needed.

## Fix-introduced breakage

None found. Reviewed the new guard's ordering against initial whitespace, quoted name tokens, the next-line opening brace, blank/comment lines before that brace, later topblocks, malformed/incomplete headers and unchanged nested body/value parsing. The implementation adds no allocation-heavy traversal, recursion, I/O capability, API change or bypass of shared cancellation/budget checks.

## Out-of-scope observations and declined judgments

No new out-of-scope observation was raised. The previously documented native grammar/fidelity limits and canonical timing/Gantt/nwdiag/diagnostic follow-ups remain nonblocking ledger items owned by their existing gates. This scoped approval does not reassess untouched families or claim broader HCL/Terraform grammar, original pixel parity, cross-platform enforcement, or whole-branch completion.

## Assessment

The sole original Important finding is addressed with a narrow grammar guard and meaningful negative/positive coverage. SPEC and QUALITY are approved for this repair; combined with the original review's otherwise satisfactory checkpoint assessment, no Task 3I blocker remains from this review seat.
