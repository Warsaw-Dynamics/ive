### Task 4: Sequence grammar cutover

**Files:** Create `parser/sequence.rs`, `grammar/sequence.lalrpop`; modify syntax portions of `sequence.rs`, family dispatch and sequence tests. Keep its model/layout.

**Interfaces:** `parser::sequence::parse(BodySource, &mut Budget) -> Result<sequence::Sequence, RenderError>`; typed participant/message/note/frame/lifecycle values feed existing semantic construction.

- [ ] Add real generated tests for two participants/aliases, message and self-message ownership, nested frame/else, multiline note, activation lifecycle and unsupported suffix/EOF. Assert existing event/participant order, IDs and geometry through retained private tests/baselines.
- [ ] Run RED against the generated entrypoint, including tiny token/depth/work and mid-parse cancellation.
- [ ] Replace parse/collect-note/command/declaration/arrow syntax. Keep interning, alive/dead/activation validation and layout; no calls to old command parsers.
- [ ] Run sequence/constants/control/common gates, deterministic baseline and genuine native sequence/note capture.
- [ ] Self-review/commit/report and audit runtime sequence route has no legacy fallback.

