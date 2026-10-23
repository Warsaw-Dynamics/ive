### Task 13: Salt grammar cutover

**Files:** Create `parser/salt.rs`, `grammar/salt.lalrpop`; modify Salt syntax/tests, preserve saltscene.

**Interfaces:** generated table/row/cell/widget productions build the existing postorder Salt arenas; direct escapes are lexical and same-row span ownership is semantic.

- [ ] Add generated nested/no-border/full-grid/widget/span tests and malformed/extra/missing close cases. Preserve checkbox/button precedence, direct escaped pipe/braces, blank rows, ordinary Text versus reserved tree markers and late-chrome controls.
- [ ] Run structural widget/table RED plus depth/item/retained-text/cancellation before allocation; retain real grid/span geometry tests.
- [ ] Remove recursive table source parsing/widget classification. No complete raw widget command forwarded to the old classifier; textual leaf content stays a bounded payload.
- [ ] Run Salt/styles/constants/control/common gates, baseline and6/14/96 native paint/capture.
- [ ] Self-review/commit/report and verify all unsupported widget/border/region shapes still reject.

