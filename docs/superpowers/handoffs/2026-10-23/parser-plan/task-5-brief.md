### Task 5: UML/description/ArchiMate and common arrow/name grammar

**Files:** Create `parser/uml.rs`, `grammar/uml.lalrpop`; modify `uml.rs`, `arrow.rs`, `lexer.rs`, family dispatch and relevant tests; keep graph/shapes.

**Interfaces:** Generated typed declarations/members/groups/relations lower to existing `uml::Uml`; generated/finite lexical arrow values preserve `Arrow`/`End`. Name/alias ordering is syntax, character/escape validation remains lexical.

- [ ] Add generated tests for scoped class/object members/packages, description aliases/notes/relations/roles, exact safe ArchiMate fills and all icon rejection positions. Pin existing rectangle alias archimate and new colored alias outgoing edges, quoted arrow-looking labels and mixed-family errors.
- [ ] Run meaningful generated-structure/EOF RED and keep existing shape/ownership/baseline tests unchanged.
- [ ] Remove Parser::line and token-slice declaration/endpoint/arrow syntax; builders accept typed values and retain identity/resolution/mixing semantics.
- [ ] Run UML/Chen-control/styles/constants/common gates and native nested graph/colored description captures.
- [ ] Self-review/commit/report and verify no complete raw command reaches an old syntax parser.

