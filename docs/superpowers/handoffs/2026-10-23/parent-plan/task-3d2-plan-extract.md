### Task 6: Shared syntax fidelity and bounded pure preprocessing

Files: `source.rs`, `style.rs`, `text.rs`, optional `preprocess/…`; family style application tests.

- [ ] Titles/legends/header/footer, explicit safe colors/fonts, supported skinparam properties and selected style rules must affect measured layout/output. Test unknown properties and selectors as errors, including unsupported declarations inside otherwise supported blocks.
- [ ] Add plain styled Creole runs with nested style/depth validation, correct escaped delimiters and explicit line breaks. Defer or reject links/images/sprites/math/tables until those specific models exist.
- [ ] Add pure literal definitions only with expansion cycles, shadowing, token-boundary, output-byte, recursion-depth and work-budget tests. Keep TIM functions/loops/conditionals unsupported unless the complete subset is implemented and tested. No general string substitution pipeline.

