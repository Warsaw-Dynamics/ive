### Task 2: Generated document/front-end ownership and span/error bridge

**Files:** Create `parser/{document,ownership,header}.rs` and `grammar/{document,header}.lalrpop`; modify `source.rs`, `preprocess.rs`, `origin.rs`, `mod.rs` and parser tests. Update generation/provenance inventories.

**Interfaces:** Add `BodySource<'s> { document: &'s origin::Document, range: Range<usize> }` and parsed wrapper/body boundary values. Generated parse failures use expanded byte spans mapped once through the document, retaining public error variants. A temporary adapter may materialize legacy `Line` views only for unmigrated family parsers; it must not invoke the old wrapper/comment state machines.

- [ ] Write `should_consume_exactly_one_generated_document_and_validate_lexical_eof` for paired/mismatched wrappers, suffix filenames, extra documents/input, BOM/CRLF and unterminated comments after every end wrapper (including Gantt). Assert source slices/physical lines and unchanged accepted wrapper case policy.
- [ ] Add raw/expanded-terminator tests using existing constants regressions: current-row note/action ownership, next-row expanded terminator, inert definition/include/style/wrapper tokens inside ordinary comments, both atomic-delimiter overlap directions, quoted/raw/data literals, and generated directive denial. Run RED against the new front-end contract.
- [ ] Replace document/directive-placement syntax with generated productions and one shared byte-level ownership implementation. Preserve original and expanded passes. Deterministic typed header evidence selects exactly one family; test keyword endpoints, actors/databases, notes and `[*]` ambiguity. No try-parser fallback.
- [ ] Run front-end/span/origin/control and all common gates, baseline check and styled/constants native examples. Do not run a second conflicting legacy comment/wrapper pass in the transitional adapter.
- [ ] Self-review/commit/report with all remaining family parsers explicitly listed as pending.

