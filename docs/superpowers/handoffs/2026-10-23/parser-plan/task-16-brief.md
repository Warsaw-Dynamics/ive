### Task 16: Generated admitted Creole nesting

**Files:** Create `parser/creole.rs`, `grammar/creole.lalrpop`; modify `creole.rs` nesting syntax and tests; keep text measurement/run/style consumers.

**Interfaces:** generated spans build the existing `Vec<Vec<Run>>` contract from text/base RunStyle/Budget. Keep primitive escape/URL handling, paired-delimiter eligibility and typed run coalescing; distinguish label-local spans from authored document spans honestly.

- [ ] Add generated nesting tests for admitted tags/symbols, same-line pairs, crossed/empty/duplicate spans, literal unmatched punctuation, hard/blank lines, escaped/URL-looking text and unsupported paired markup/resources. Preserve inline-style query/paint/cache and literal data-family behavior.
- [ ] Run nesting/complete-consumption RED, tiny run/text/depth/work/cancellation and exact style assertions. Keep all previous URL-adjacent markup regressions.
- [ ] Replace old styled-span stack/syntax recognition with generated productions. Lexical pairing may disambiguate literal unmatched symbols but may not itself build the nested run grammar. Do not activate Creole in literal JSON/YAML/HCL/file-name data.
- [ ] Run all style/text/constants/common gates, full baseline and representative6/14/96 native rich text across graph/sequence/table families.
- [ ] Self-review/commit/report and document only lexical/semantic handwritten residuals.

