### Task 11: PlantUML-specific YAML grammar

**Files:** Create `parser/yaml.rs`, `grammar/yaml.lalrpop`; modify YAML syntax/tests, retain data model/datascene.

**Interfaces:** lexical tab/indent/quote/literal chunks feed generated block-map/list/flow productions returning `data::Value` with existing duplicate/scalar semantics.

- [ ] Add generated tests for ordered map/list ownership, duplicate first-position replacement, all scalar strings, tabs/indentation, literal-block blanks/trimming, narrow flow lists and malformed crossed/dangling forms. Tags/anchors/aliases/folded blocks remain explicit unsupported errors before model construction.
- [ ] Run structure/indent/depth/long-flow RED; retain previous constant-time flow accounting and pre-model capability controls.
- [ ] Remove Parser block/map/list/entry/flow structural recursion. Keep primitive quote/comment/tab/literal lexing, not pre-parsed complete map entries masquerading as tokens.
- [ ] Run tree/data/constants/control/common gates, baseline and actual native ordered/nested YAML capture; verify unknown features never become partial output.
- [ ] Self-review/commit/report and classify every remaining YAML helper's lexical versus semantic responsibility.

