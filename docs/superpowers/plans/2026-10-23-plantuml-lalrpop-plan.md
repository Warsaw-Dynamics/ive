# PlantUML LALRPOP Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all current ad-hoc PlantUML structural parsing with real generated LALRPOP grammars while preserving existing model/render behavior.

**Architecture:** Shared bounded lexical/provenance infrastructure feeds generated document/common/family grammars. Typed actions build the existing models; layouts, measurement, safe SVG and viewer/worker remain. Each cutover removes its old syntax authority without runtime fallback.

**Tech Stack:** Rust, exact lalrpop/lalrpop-util0.23.2, checked-in generated Rust, existing engine/pgui viewer and Node audit tooling.

**Spec:** `docs/superpowers/specs/2026-10-23-plantuml-lalrpop-design.md`

## Global Constraints

- User explicitly directs rewriting CURRENT PlantUML parsing with LALRPOP, not just a future-family pilot. New-family breadth waits for this plan's completion.
- Keep public render/options/result/error/control/measurement APIs unchanged. Keep the engine GPUI-free and caller-memory-only; no runtime files/network/environment/clock/locale/process/JVM/JS/browser/dot.
- Preserve defaults: source256KiB, SVG16MiB, model10,000, nesting64, work5,000,000, cooperative10s; host hard10s/2GiB Unix; labels16KiB, queries4096, finite coordinate magnitude1,000,000 and existing raster limits.
- Translate PlantUML semantics only from the authorized MIT1.2026.9 tree. Preserve exact source hashes, actual credits/grants, fixtures and Merman's audited656-file/four-patch baseline. Separately audit generator development dependencies.
- Generated productions own actual syntax and EOF. No Line* forwarding, source-string reparsing in semantic builders, parser retry/fallback, partial recovery-and-render or broad golden regeneration.
- Retain explicit lexical raw/comment/quote ownership, immutable eager literal evaluation/no-rescan, bounded origins, cancellation and exact supported/rejected subset behavior.
- One product implementer at a time; fresh independent SPEC/QUALITY gate per task. Use apply_patch/TDD/AGENTS; generator output is produced mechanically, never hand-edited. Shared Cargo target is `/mnt/data/Projects/ive/target`.
- Baseline is reviewed4c197871caf992a1a852b7048311b4ee450d74ff or a documentation-only successor:488 Rust tests+seven actual worker faults, all strict audits. Existing model/layout assertions remain, even when their parser setup changes.

## Review Focus

1. Contextual keywords versus actual commands: old description archimate endpoints, timing robust/concise tracks, nwdiag title servers, Chen entity/relationship aliases/composites. Assigned Tasks5/6/12/14.
2. Original-source ownership versus expanded terminators: raw notes/actions/data, atomic comment delimiters, immutable definitions, inert resource-looking comments and active includes. Assigned Tasks1/2/4.
3. Generator-owned stacks/reductions and retained syntax before allocation: tiny limits, deep/wide input and cancellation inside lexing/iteration/actions. Assigned Task1 and every family's opener tests.
4. Significant whitespace and nonstandard data semantics: CRLF/BOM, Salt escapes/rows, YAML indentation/duplicates, HCL header lines and files trailing spaces. Assigned Tasks2/9/10/11/13.
5. Hidden residual parsers or misleading evidence: raw command tokens fed to legacy methods, changed goldens, unclassified styles/Creole/JSON/YAML parsing and stale generation. Assigned Tasks3/10/11/16/17.

## Common Verification and Report Contract

For each task: write the named failing contracts first; run focused tests and record meaningful RED; implement the stated grammar/builder cutover; run focused GREEN then the complete gates below after the last edit; self-review, scoped commit and full report. Existing already-passing corpus cases are controls, not invented RED evidence. A minimal contract stub may make a new test compile and return an explicit unavailable error before the real grammar exists; it must not ship as a fallback.

Every cutover runs baseline checking and preserves these assertions in the baseline driver:

```js
assert.equal(current.sourceSha256, baseline.sourceSha256);
assert.equal(current.family, baseline.family);
assert.equal(current.svgSha256, baseline.svgSha256);
```

The only intended new-positive differences are the four documented boundary repairs. Add their source/expected semantics explicitly; do not replace existing fixture records. Preserve exact model IDs/topology/private geometry assertions as well as SVG hashes.

From `diagram-ports/`, with the shared target and locked/offline flags:

```sh
cargo test --locked --offline --workspace
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all --check
cargo build --locked --offline --release
cargo deny --locked --offline --config ../deny.toml check licenses sources
node tools/audit-plantuml.mjs
node tools/audit-merman.mjs
node --test tools/audit-plantuml.test.mjs tools/audit-merman.test.mjs
cargo run --locked --offline --manifest-path tools/plantuml-grammar-gen/Cargo.toml -- check
node tools/parser-baseline.mjs check
```

Task1 creates the last two tools. Audit the generator's separate lock/feature/license closure too; its checked-in outputs must rebuild offline after the initial authorized fetch. Genuine native-paint tests/captures, actual counts and cleanup remain per-cutover evidence; headless empty-PATH exports must work without the generator. Never rerun original Java/browser oracles just to reconfirm parser parity.

Reports live in this plan's controller-owned SDD workspace, one task-N-report.md per extracted brief. Include base/head, removed legacy syntax, real grammar entrypoints and model interfaces, RED/GREEN commands/output, source/generation/limit evidence, baseline/native results and residuals. Controller updates `/mnt/data/Projects/ive/progress.md`; implementers do not.

**Family interface rule (Tasks4–15):** each `parser/<family>.rs` exposes `parse(body: BodySource<'_>, budget: &mut Budget) -> Result<ExistingModel, RenderError>`, where ExistingModel is the task's named preserved model. Tree exposes separate `parse_mindmap`/`parse_wbs` entries returning the distinct tree models; files/board and timing/Gantt each have their own parse module. JSON/YAML return `data::Value`, HCL returns `hcl::Document` before its unchanged lowering. Structured internal parse failures are mapped through BodySource's origin document at these facades, not repeatedly formatted/reparsed.

### Task 1: Reproducible generator, bounded parser core and real definition cutover

**Files:** Modify `diagram-ports/{Cargo.toml,Cargo.lock}`, `engine/Cargo.toml`, `engine/src/plantuml/{mod,preprocess}.rs`, provenance/audit files. Create `tools/plantuml-grammar-gen/{Cargo.toml,Cargo.lock,src/main.rs}`, `tools/parser-baseline.mjs`, `engine/examples/parser_baseline.rs`, `engine/src/plantuml/parser/{mod,token,error,context,definition}.rs`, `parser/grammar/definition.lalrpop`, `engine/src/plantuml/generated/`, generation inventory and private tests. Do not scaffold unused future token variants.

**Interfaces:** Add borrowed `SourceView<'s> { text: &'s str, byte_base: usize }`; expanded-family bodies will pair this with the owning origin document. Add structured private `ParseFailure`, budgeted token buffering/checked iteration and per-grammar admission metadata. Add `parse_definition<'s>(raw: &'s str, authored: origin::SourceSpan, budget: &mut Budget) -> Result<DefinitionSyntax<'s>, RenderError>`; syntax contains borrowed name/literal and exact authored name/literal spans. `preprocess::Processor::define` consumes it, retaining registry/freeze/evaluation logic but removing manual declaration syntax extraction.

- [ ] Capture the current immutable baseline FIRST. The Rust development example renders each audited `.puml` through the public API with language PlantUml/default limits/default deterministic measurement and emits source path/family/SVG as bounded JSON records. The Node driver records source/SVG SHA256 and deterministic configuration in `fixtures/plantuml/parser-baseline.json`; `record` refuses an existing file, `check` never overwrites it. Keep generator independent of engine dependencies to avoid a regeneration bootstrap cycle.
- [ ] Write `should_parse_literal_definition_with_generated_grammar`: for `!define WHO World`, assert name WHO/literal World, name byte range8..11/column9 and literal12..17/column13. Add nonzero-base/UTF8 span, nonempty/ASCII64-name, literal16KiB/control/continuation, complete-EOF and tiny-limit cases. Add controlled lex/iterator/action cancellation tests returning Cancelled, plus generation drift/missing-output/version mismatch and read-only-check tests. Run focused RED without counting dependency-fetch failure as product RED.
- [ ] Acquire and inspect exact generator0.23.2/development closure using the authorized dependency workflow; audit licenses/checksums/selected features. Pin direct engine utility `=0.23.2`, default features off. Implement generator `generate` and `check` subcommands with stable logical paths, checked-in `generated/<grammar-stem>.rs` outputs and `generated/manifest.json`. Verify actual Configuration APIs, no Merman regeneration and no hand-edited/rustfmt-generated bodies.
- [ ] Implement the genuine definition production and wire it into active preprocessing. Retain original whole-declaration Definition span and use generated literal span for expansion provenance. Prove/check generated stack/reduction admission before parser entry, no nullable cycles/recovery accumulation, charged retention/actions and iterator checkpoints. Inspect actual generated code; do not assert fallible actions alone bound generator allocations. Keep authored infrastructure and semantic grammar provenance explicit rather than inventing Java source credits.
- [ ] Run GREEN, constants/preprocessor origin/bounds suites, all common gates, baseline check and a native constant example. Verify ordinary builds/exports need no generator. Self-review/commit/report; explicitly state only definition syntax has cut over, not all PlantUML.

### Task 2: Generated document/front-end ownership and span/error bridge

**Files:** Create `parser/{document,ownership,header}.rs` and `grammar/{document,header}.lalrpop`; modify `source.rs`, `preprocess.rs`, `origin.rs`, `mod.rs` and parser tests. Update generation/provenance inventories.

**Interfaces:** Add `BodySource<'s> { document: &'s origin::Document, range: Range<usize> }` and parsed wrapper/body boundary values. Generated parse failures use expanded byte spans mapped once through the document, retaining public error variants. A temporary adapter may materialize legacy `Line` views only for unmigrated family parsers; it must not invoke the old wrapper/comment state machines.

- [ ] Write `should_consume_exactly_one_generated_document_and_validate_lexical_eof` for paired/mismatched wrappers, suffix filenames, extra documents/input, BOM/CRLF and unterminated comments after every end wrapper (including Gantt). Assert source slices/physical lines and unchanged accepted wrapper case policy.
- [ ] Add raw/expanded-terminator tests using existing constants regressions: current-row note/action ownership, next-row expanded terminator, inert definition/include/style/wrapper tokens inside ordinary comments, both atomic-delimiter overlap directions, quoted/raw/data literals, and generated directive denial. Run RED against the new front-end contract.
- [ ] Replace document/directive-placement syntax with generated productions and one shared byte-level ownership implementation. Preserve original and expanded passes. Deterministic typed header evidence selects exactly one family; test keyword endpoints, actors/databases, notes and `[*]` ambiguity. No try-parser fallback.
- [ ] Run front-end/span/origin/control and all common gates, baseline check and styled/constants native examples. Do not run a second conflicting legacy comment/wrapper pass in the transitional adapter.
- [ ] Self-review/commit/report with all remaining family parsers explicitly listed as pending.

### Task 3: Generated preamble/styles/chrome syntax

**Files:** Create `parser/preamble.rs`, `grammar/preamble.lalrpop`; modify `style.rs`, `origin.rs`, `mod.rs`, `plantuml_styles.rs` and focused parser tests. Keep `chrome.rs` layout.

**Interfaces:** Generated typed region/style/skinparam values lower into existing `style::Preamble`/effective styles. Replace normalized-Line-index preamble validation with parsed boundary spans; keep literal evaluator semantic policy.

- [ ] Add generated tests for colon/aligned/case region forms, multiline regions/styles, exact font/color values and all active/late boundaries. Preserve existing native6/14/96 measure/paint assertions, mixed style/skinparam rejection, quoted literal lookalikes, HCL command-named blocks and YAML keys.
- [ ] Run focused RED for absent generated structures/full consumption; existing style tests remain controls.
- [ ] Replace preamble/region/selector/property syntax with actual productions. Retain typed value validation/style application; no raw line reparsing. HCL shared preambles remain denied, not silently enabled.
- [ ] Run styles/constants/control/common gates and baseline/native styled captures; inspect effective font cache/paint behavior.
- [ ] Self-review/commit/report and remove obsolete style syntax functions.

### Task 4: Sequence grammar cutover

**Files:** Create `parser/sequence.rs`, `grammar/sequence.lalrpop`; modify syntax portions of `sequence.rs`, family dispatch and sequence tests. Keep its model/layout.

**Interfaces:** `parser::sequence::parse(BodySource, &mut Budget) -> Result<sequence::Sequence, RenderError>`; typed participant/message/note/frame/lifecycle values feed existing semantic construction.

- [ ] Add real generated tests for two participants/aliases, message and self-message ownership, nested frame/else, multiline note, activation lifecycle and unsupported suffix/EOF. Assert existing event/participant order, IDs and geometry through retained private tests/baselines.
- [ ] Run RED against the generated entrypoint, including tiny token/depth/work and mid-parse cancellation.
- [ ] Replace parse/collect-note/command/declaration/arrow syntax. Keep interning, alive/dead/activation validation and layout; no calls to old command parsers.
- [ ] Run sequence/constants/control/common gates, deterministic baseline and genuine native sequence/note capture.
- [ ] Self-review/commit/report and audit runtime sequence route has no legacy fallback.

### Task 5: UML/description/ArchiMate and common arrow/name grammar

**Files:** Create `parser/uml.rs`, `grammar/uml.lalrpop`; modify `uml.rs`, `arrow.rs`, `lexer.rs`, family dispatch and relevant tests; keep graph/shapes.

**Interfaces:** Generated typed declarations/members/groups/relations lower to existing `uml::Uml`; generated/finite lexical arrow values preserve `Arrow`/`End`. Name/alias ordering is syntax, character/escape validation remains lexical.

- [ ] Add generated tests for scoped class/object members/packages, description aliases/notes/relations/roles, exact safe ArchiMate fills and all icon rejection positions. Pin existing rectangle alias archimate and new colored alias outgoing edges, quoted arrow-looking labels and mixed-family errors.
- [ ] Run meaningful generated-structure/EOF RED and keep existing shape/ownership/baseline tests unchanged.
- [ ] Remove Parser::line and token-slice declaration/endpoint/arrow syntax; builders accept typed values and retain identity/resolution/mixing semantics.
- [ ] Run UML/Chen-control/styles/constants/common gates and native nested graph/colored description captures.
- [ ] Self-review/commit/report and verify no complete raw command reaches an old syntax parser.

### Task 6: Chen grammar and contextual keyword repair

**Files:** Create `parser/chen.rs`, `grammar/chen.lalrpop`; modify syntax in `chen.rs` and Chen tests; keep chenscene.

**Interfaces:** `parser::chen::parse(BodySource, &mut Budget) -> Result<chen::Chen, RenderError>`; typed owners/attributes/associations feed the existing qualified-identity builder.

- [ ] Add generated tests for entity/relationship owners, composite/relation attributes, parallel/self associations and participation/cardinality. Include declared aliases entity/relationship used as endpoints and both spaced/compact composite attributes; assert identical keys/ownership, not only success. For a C→entity→leaf composite, pin keys C/entity and C/entity/leaf; actual nested entity declarations still reject.
- [ ] Run the deferred M1 reproductions as meaningful RED plus new generated-entry/span/depth contracts; keep exact shape/intersection/private tests.
- [ ] Replace keyword/association/attribute syntax dispatch. Builders receive typed nodes; preserve all five decoration/subtype rejections and normal-comment/constant ownership.
- [ ] Run Chen/UML/constants/control/common gates, baseline and genuine multi-kind native capture with6/14/96 ink checks.
- [ ] Self-review/commit/report; mark M1 resolved with exact regressions, not silently dropped.

### Task 7: State grammar cutover

**Files:** Create `parser/state.rs`, `grammar/state.lalrpop`; modify state syntax/dispatch/tests; preserve stategraph.

**Interfaces:** generated state syntax lowers into `state::State`; retain forward-reference resolution and scoped pseudo/composite identities.

- [ ] Add generated tests for forward references, identical local names in separate composites, initial/final scope, descriptions, supported closers/arrows and dangling/crossed/unknown suffix failures. Retain current private ID/ancestor and real geometry assertions.
- [ ] Run generated structure/EOF/opener-depth RED; advanced concurrent/history/subtype features remain unsupported until their later family task.
- [ ] Remove handwritten declaration/transition/closer parsing; retain two-phase semantic resolution and scoped construction.
- [ ] Run state/control/styles/constants/common gates, baseline and nested-state native capture.
- [ ] Self-review/commit/report and inventory residual code as semantics/layout, not hidden syntax.

### Task 8: Structured activity grammar cutover

**Files:** Create `parser/activity.rs`, `grammar/activity.lalrpop`; modify activity syntax/tests, preserve activitytile.

**Interfaces:** generated recursive blocks build existing `activity::Block`/`Instruction`/`Branch`/`Condition` values.

- [ ] Add generated tests for actions/start/stop, nested if/elseif/else, while/repeat, fork join/merge and groups, asserting existing continuation/live-dead topology. Crossed/unterminated blocks, illegal continuations and unsupported syntax must reject completely.
- [ ] Run recursive-structure RED with max-depth/max+1 before retention, tiny model/work and controlled cancellation. Preserve raw action/expanded terminator tests.
- [ ] Replace handwritten recursive command/condition/guard syntax; raw action text is payload only after a valid action production.
- [ ] Run activity/control/styles/constants/common gates, baseline and branched/loop/fork native captures with real port/ink assertions.
- [ ] Self-review/commit/report; no generic graph or legacy activity fallback.

### Task 9: Mindmap/WBS, files and board outline grammars

**Files:** Create `parser/{tree,files,board}.rs`, `grammar/{tree,files,board}.lalrpop`; modify syntax in `tree.rs`, `files.rs`, `board.rs` and relevant tests; preserve their scene modules.

**Interfaces:** separate generated entries return `tree::Mindmap`, `tree::Wbs`, `files::Files`, `board::Board`. Share lexical marker/indent machinery only, not a universal tree model.

- [ ] Add generated tests pinning independent mindmap sides/parents and WBS aliases/buses; ordered file folder reuse/duplicate files/sibling notes/empty folders/trailing spaces/dotdot; board nearest-lower parents/stage gaps/DFS columns and single root paint. Preserve literal files versus formatted notes/cards.
- [ ] Run real marker/path/note-block structure and malformed opener/indent RED; preserve unknown substantive command rejection and raw Unicode/XML data.
- [ ] Replace marker/path/row/note syntax. Keep semantic depth normalization, trie lookup/anchors and activity ancestry; no filesystem operations, dense grids or ancestor fullpath cloning.
- [ ] Run tree/data/inventory/constants/control/common gates, baseline and native captures for all four outputs, including note clearance and stage geometry.
- [ ] Self-review/commit/report each entrypoint/model explicitly; no family aliasing or skipped file in this batch.

### Task 10: JSON and HCL generated structural grammars

**Files:** Create `parser/{json,hcl}.rs`, `grammar/{json,hcl}.lalrpop`; modify `data.rs` JSON syntax, `hcl.rs` and tests; preserve datascene/lowering.

**Interfaces:** generated JSON returns existing `data::Value`; generated HCL returns `hcl::Document` then existing ownership lowering. Primitive JSON string decoding and numeric validation are allowed; old value recursive descent is not.

- [ ] Add generated tests for ordered duplicate JSON keys/exact bounded number spelling and malformed separators/EOF; HCL duplicate-field order/string scalars/nested owning rows/single-block unwrap/multiple-block names. Retain exact title/style swallow negatives, valid command-named blocks, split braces and multiline-value controls.
- [ ] Run generated container/header structure RED, depth/retention-before-scan and callback-cancellation controls; unsupported HCL expressions/arrays/styles remain denied.
- [ ] Remove JSON/HCL recursive source parsers. Build ordered typed values from productions without child AST cloning or standard-HCL evaluation.
- [ ] Run tree/data/inventory/constants/control/common gates, baseline and native nested JSON/HCL compartments with actual owning-row links.
- [ ] Self-review/commit/report and audit serde_json use is primitive decoding only.

### Task 11: PlantUML-specific YAML grammar

**Files:** Create `parser/yaml.rs`, `grammar/yaml.lalrpop`; modify YAML syntax/tests, retain data model/datascene.

**Interfaces:** lexical tab/indent/quote/literal chunks feed generated block-map/list/flow productions returning `data::Value` with existing duplicate/scalar semantics.

- [ ] Add generated tests for ordered map/list ownership, duplicate first-position replacement, all scalar strings, tabs/indentation, literal-block blanks/trimming, narrow flow lists and malformed crossed/dangling forms. Tags/anchors/aliases/folded blocks remain explicit unsupported errors before model construction.
- [ ] Run structure/indent/depth/long-flow RED; retain previous constant-time flow accounting and pre-model capability controls.
- [ ] Remove Parser block/map/list/entry/flow structural recursion. Keep primitive quote/comment/tab/literal lexing, not pre-parsed complete map entries masquerading as tokens.
- [ ] Run tree/data/constants/control/common gates, baseline and actual native ordered/nested YAML capture; verify unknown features never become partial output.
- [ ] Self-review/commit/report and classify every remaining YAML helper's lexical versus semantic responsibility.

### Task 12: Timing/Gantt grammar cutovers and assigned boundary repairs

**Files:** Create `parser/{timing,gantt}.rs`, `grammar/{timing,gantt}.lalrpop`; modify their syntax/tests; preserve scenes/calendar/cursor semantics.

**Interfaces:** generated commands build existing `timing::Timing` and `gantt::Gantt`; checked arithmetic/reference semantics remain builders, not rewritten source phrases.

- [ ] Add generated tests pinning timing exact integers/separate cursors/duplicate changes/constraint and message bases; Gantt explicit dates/duration/task aliases/dependencies/half-open bars/milestones/freeze-after-reference. Add timing robust/concise name regressions and terminated-versus-unterminated trailing Gantt comments.
- [ ] Run assigned bug RED if not already fixed by shared front end, plus new generated command/endpoint/full-EOF RED. Do not invent a second RED for an already repaired common EOF check.
- [ ] Remove keyword dispatch, bracket/verb/time/complement source splitting; generated typed endpoints feed existing checked arithmetic. No ambient calendar/clock/resource expansion.
- [ ] Run timing/Gantt/constants/control/common gates, baseline and genuine native tracks/calendar captures. Record the two assigned Minors resolved with exact controls.
- [ ] Self-review/commit/report with all unchanged integer/date/range/grammar exclusions explicit.

### Task 13: Salt grammar cutover

**Files:** Create `parser/salt.rs`, `grammar/salt.lalrpop`; modify Salt syntax/tests, preserve saltscene.

**Interfaces:** generated table/row/cell/widget productions build the existing postorder Salt arenas; direct escapes are lexical and same-row span ownership is semantic.

- [ ] Add generated nested/no-border/full-grid/widget/span tests and malformed/extra/missing close cases. Preserve checkbox/button precedence, direct escaped pipe/braces, blank rows, ordinary Text versus reserved tree markers and late-chrome controls.
- [ ] Run structural widget/table RED plus depth/item/retained-text/cancellation before allocation; retain real grid/span geometry tests.
- [ ] Remove recursive table source parsing/widget classification. No complete raw widget command forwarded to the old classifier; textual leaf content stays a bounded payload.
- [ ] Run Salt/styles/constants/control/common gates, baseline and6/14/96 native paint/capture.
- [ ] Self-review/commit/report and verify all unsupported widget/border/region shapes still reject.

### Task 14: nwdiag grammar and contextual server names

**Files:** Create `parser/nwdiag.rs`, `grammar/nwdiag.lalrpop`; modify nwdiag syntax/tests, preserve nwscene.

**Interfaces:** typed network/group/server/property statements build existing stable global server and membership registries.

- [ ] Add generated tests for complete property lists/terminators/nonnesting, global shared server identity/separate addresses, repeat policy and group-only references. Add equivalent title-with-space/property and compact forms, while genuine late title/chrome remains rejected.
- [ ] Run the deferred keyword bug RED and generated block/property/full-EOF RED, retaining sparse crossing/group/endpoint geometry and limits.
- [ ] Remove expect/peek/block/statement/property syntax; preserve typed property admission/registries and sparse layout. No server/network/group identity inference from display text.
- [ ] Run nwdiag/styles/constants/control/common gates, baseline and real shared-router/group capture. Record the assigned Minor resolved.
- [ ] Self-review/commit/report, including all keyword/late-command and foreign-crossing assertions.

### Task 15: Packetdiag grammar cutover

**Files:** Create `parser/packet.rs`, `grammar/packet.lalrpop`; modify packet syntax/tests, preserve packetscene/fragment projection.

**Interfaces:** generated wrapper/config/range/label/attribute values build existing Config/Field models, then existing conserved Fragment projection.

- [ ] Add generated tests for direct/canonical body, inclusive and auto ranges, admitted configuration/attributes, complete delimiters and exact rejection of repeated colons/unknown rotation. Retain payload5/8/7, proportional widths, RTL/ticks and variable/equal row-height geometry.
- [ ] Run generated range/config/list/full-EOF RED plus huge sparse numeric/token/tick/depth/work/output/cancellation limits.
- [ ] Remove brace/config/field/attribute source splitting. Preserve original numeric caps, final configuration effects and symbolic field identities; no cloned label AST per fragment.
- [ ] Run inventory/constants/control/common gates, baseline and actual split-packet native capture/ink tests.
- [ ] Self-review/commit/report and confirm no ignored configuration or partial-success suffix remains.

### Task 16: Generated admitted Creole nesting

**Files:** Create `parser/creole.rs`, `grammar/creole.lalrpop`; modify `creole.rs` nesting syntax and tests; keep text measurement/run/style consumers.

**Interfaces:** generated spans build the existing `Vec<Vec<Run>>` contract from text/base RunStyle/Budget. Keep primitive escape/URL handling, paired-delimiter eligibility and typed run coalescing; distinguish label-local spans from authored document spans honestly.

- [ ] Add generated nesting tests for admitted tags/symbols, same-line pairs, crossed/empty/duplicate spans, literal unmatched punctuation, hard/blank lines, escaped/URL-looking text and unsupported paired markup/resources. Preserve inline-style query/paint/cache and literal data-family behavior.
- [ ] Run nesting/complete-consumption RED, tiny run/text/depth/work/cancellation and exact style assertions. Keep all previous URL-adjacent markup regressions.
- [ ] Replace old styled-span stack/syntax recognition with generated productions. Lexical pairing may disambiguate literal unmatched symbols but may not itself build the nested run grammar. Do not activate Creole in literal JSON/YAML/HCL/file-name data.
- [ ] Run all style/text/constants/common gates, full baseline and representative6/14/96 native rich text across graph/sequence/table families.
- [ ] Self-review/commit/report and document only lexical/semantic handwritten residuals.

### Task 17: Full migration removal, inventory and integration gate

**Files:** Remove remaining legacy syntax/transitional adapters; update parser integration/tests, generation/provenance audits, README/COVERAGE and migration inventory. Do not alter unrelated later-family work or audited Merman sources.

**Interfaces:** one generated structural syntax authority per current component; production render contains no old parser retry/alternate route. Handwritten residues are explicit lexical decoding/ownership or semantic/model/layout work.

- [ ] Add an audit test rejecting an unclassified current parser/grammar input/generated output and a deliberately stale generated file. Inventory remaining parse/prefix/split/cursor helpers by responsibility; no blanket native-directory exemption or renamed raw command parser.
- [ ] Run meaningful audit RED, then remove duplicate/obsolete syntax code and test-only frozen comparators. Do not delete still-required models/layout/helpers just because their files contain old Parser names.
- [ ] Verify every original family/common syntax entry is generated, all four assigned defects have positive/negative tests, no Line* forwarding exists, all source/fixture hashes remain truthful and generation is reproducible from the separate audited lockfile.
- [ ] Run every common gate/baseline/native suite, representative actual captures for all cutovers and shared style/constants, empty-PATH/no-display exports, process cleanup and dependency/source audit. Record any deliberate diagnostic/work-accounting differences; no silent golden updates.
- [ ] Self-review/commit/report with a migrated-versus-pending matrix showing zero current-parser omissions. Dispatch the independent final migration review under SDD; after its gate, resume parent family inventory with LALRPOP-required briefs.

## Controller self-review and handoff

Coverage: Tasks1–3 own document/definitions/ownership/styles;4 sequence;5 UML/description/ArchiMate/common names/arrows;6 Chen;7 state;8 activity;9 mindmap/WBS/files/board;10 JSON/HCL;11 YAML;12 timing/Gantt;13 Salt;14 nwdiag;15 packet;16 Creole;17 complete removal/integration. Every current syntax component has an owner, while render models/layouts remain.

Type/interface consistency: SourceView/structured failure/token/context foundation is Task1, expanded BodySource/typed document mapping Task2, existing model return types remain the family boundary. Primitive decoders are not alternate structural parsers. Generated-header mode decisions must be explicit before raw payload emission.

Review-focus cases are assigned above. The first baseline is immutable; meaningful RED comes from new generated contracts/known defects, not rerunning old positive fixtures. Generator acquisition/reproducibility and internal stack admission are explicit first-task deliverables rather than assumed from cached runtime availability.

The owner already selected subagent-driven execution and explicitly directed this rewrite. Continue task-by-task with fresh implementers/reviewers, canonical progress updates and recorded rulings. Do not pause for routine execution-method approval. A real authority/security/destructive blocker still requires normal escalation.
