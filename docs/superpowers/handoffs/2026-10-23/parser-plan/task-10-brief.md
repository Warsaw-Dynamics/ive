### Task 10: JSON and HCL generated structural grammars

**Files:** Create `parser/{json,hcl}.rs`, `grammar/{json,hcl}.lalrpop`; modify `data.rs` JSON syntax, `hcl.rs` and tests; preserve datascene/lowering.

**Interfaces:** generated JSON returns existing `data::Value`; generated HCL returns `hcl::Document` then existing ownership lowering. Primitive JSON string decoding and numeric validation are allowed; old value recursive descent is not.

- [ ] Add generated tests for ordered duplicate JSON keys/exact bounded number spelling and malformed separators/EOF; HCL duplicate-field order/string scalars/nested owning rows/single-block unwrap/multiple-block names. Retain exact title/style swallow negatives, valid command-named blocks, split braces and multiline-value controls.
- [ ] Run generated container/header structure RED, depth/retention-before-scan and callback-cancellation controls; unsupported HCL expressions/arrays/styles remain denied.
- [ ] Remove JSON/HCL recursive source parsers. Build ordered typed values from productions without child AST cloning or standard-HCL evaluation.
- [ ] Run tree/data/inventory/constants/control/common gates, baseline and native nested JSON/HCL compartments with actual owning-row links.
- [ ] Self-review/commit/report and audit serde_json use is primitive decoding only.

