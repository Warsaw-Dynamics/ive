### Task 3: Generated preamble/styles/chrome syntax

**Files:** Create `parser/preamble.rs`, `grammar/preamble.lalrpop`; modify `style.rs`, `origin.rs`, `mod.rs`, `plantuml_styles.rs` and focused parser tests. Keep `chrome.rs` layout.

**Interfaces:** Generated typed region/style/skinparam values lower into existing `style::Preamble`/effective styles. Replace normalized-Line-index preamble validation with parsed boundary spans; keep literal evaluator semantic policy.

- [ ] Add generated tests for colon/aligned/case region forms, multiline regions/styles, exact font/color values and all active/late boundaries. Preserve existing native6/14/96 measure/paint assertions, mixed style/skinparam rejection, quoted literal lookalikes, HCL command-named blocks and YAML keys.
- [ ] Run focused RED for absent generated structures/full consumption; existing style tests remain controls.
- [ ] Replace preamble/region/selector/property syntax with actual productions. Retain typed value validation/style application; no raw line reparsing. HCL shared preambles remain denied, not silently enabled.
- [ ] Run styles/constants/control/common gates and baseline/native styled captures; inspect effective font cache/paint behavior.
- [ ] Self-review/commit/report and remove obsolete style syntax functions.

