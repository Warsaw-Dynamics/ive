# Native PlantUML implementation design and staged plan

> For agentic workers: execute the accepted stages with `superpowers:executing-plans`, or with the controller's already-authorized subagent workflow. This document is design preparation only. Do not begin product edits until the controller releases the PlantUML stage after Mermaid verification.

**Goal:** Build an idiomatic, entirely native Rust PlantUML backend in the existing standalone `diagram-ports` workspace, advancing toward broad compatibility through honest, independently tested increments.

**Architecture:** Parse authorized MIT PlantUML syntax into located typed family models; measure and lay out those models natively; emit an owned safe SVG scene. Reuse the already-audited Merman measurement session, terminal SVG pipeline and Dugong/Dagre graph kernel below the language boundary. Keep the existing public engine API and bounded viewer worker.

**Tech stack:** Rust; existing `merman`, `merman-render`, `dugong`, `serde_json`, `roxmltree`, `cssparser`; native pgui host through its existing worker protocol. No new heavyweight runtime dependency is needed for the first stages.

**Spec:** `docs/superpowers/specs/2026-10-22-diagram-ports-implementation-design.md`, its original handoff, and `docs/superpowers/plans/2026-10-22-plantuml-port-analysis.md`. The controller's later source restriction supersedes the former candidate-evaluation suggestion.

Final standalone checkpoint (2026-10-23): the planned native increments and 29 checked-in LALRPOP grammars are integrated through product `3478bbd`; final gates, 95-source corpus, 72 viewer captures and two scoped independent reviews completed. See [COVERAGE.md](../../../diagram-ports/COVERAGE.md), [evidence](../../../diagram-ports/evidence/README.md) and [progress.md](../../../progress.md) for exact claims and remaining full-port gaps. Stage instructions and checkboxes below remain historical records.

## Global constraints and present evidence

- Translation source is exclusively `/mnt/data/Projects/plantuml-mit/src`, authorized MIT-edition 1.2026.9. No GPL repository or other edition was inspected. No rejected PlantUML candidate source was inspected or imported.
- Exclude `gen/`, `h/`, `smetana/`, the ELK implementation, and their algorithm translations. The previous inventory identifies conflicting embedded EPL Graphviz notices; use the separately audited permissive Dagre kernel instead.
- Source remains caller-owned memory. No Java, JavaScript, browser, external dot, network, filesystem resolver, environment lookup or clock access in the engine. Includes and resource-bearing directives are denied initially.
- Current engine defaults remain 256 KiB source, 16 MiB SVG, 10,000 model items, nesting depth 64, 5,000,000 work units, 10 seconds. Continue validating all options through `limits::resource_policy`.
- The viewer already supplies a hard 10-second process deadline, a Unix 2 GiB address-space ceiling and cancellation/reaping. The engine must still checkpoint and charge its own loops. These process limits are not an OS I/O sandbox.
- Model and scene values are immutable after construction; mutable local parser/layout builders are bounded and not shared. Own types use explicit enum states rather than optional-wrapper fields. Library-required `Option` fields in external API structs are confined to adapters.
- Tests follow `should_…` names and given/when/then sections. No silent skipped commands, truncated input, blank success or unrelated-family fallback.
- This document verifies API availability by reading the exact installed Rust sources; no compile probe or PlantUML implementation is claimed. Parent-directed fixture/oracle preparation is independent of this design.

## Selected approach and alternatives

**Recommended: typed PlantUML frontend plus native family scene/layout adapters.** PlantUML has semantics Mermaid does not represent: object members, many UML extremities, nested package rules, sequence lifecycle events, modern activity tiles, WBS sides, network memberships and Salt widgets. Reuse geometry and services below the syntax layer, keeping each meaning explicit.

**Conditional alternative: typed lowering to a Merman family model.** This is reasonable only when every admitted PlantUML feature maps exactly to that model and the public typed renderer accepts it. A family-specific conversion must validate its entire input and document geometry differences. The current facade `Renderer::prepare_semantic` accepts Mermaid source, not arbitrary PlantUML AST. Generating Mermaid text would add quoting, identity, dispatch and language mismatch problems; it is not the initial architecture.

**Rejected for this stage: full translation of all PlantUML infrastructure before any result.** The MIT inventory contains 3,142 Java files and about 659,230 physical lines. Reimplementing the full command framework, all rich text, TIM, Teoz/Ftile and all families before shipping a vertical slice is not feasible in a single working stage. The target remains broad/full compatibility, with an explicit ledger of supported constructs, intentional deviations and remaining work.

## Existing public interfaces to preserve

`engine/src/lib.rs` already provides:

```rust
pub fn render(source: &str, options: &RenderOptions)
    -> Result<RenderedDiagram, RenderError>;
```

Dispatch `Language::PlantUml` to `plantuml::render`. Preserve `RenderOptions`, the worker protocol and the `RenderedDiagram` fields. Set `language = PlantUml`, `backend = "native PlantUML"`, `upstream = "PlantUML MIT 1.2026.9"`; use stable families such as `sequence`, `class`, `description`, `state`, `activity`, `mindmap`, `wbs`, `json`, `yaml`, `timing`, `gantt`, `salt`, `nwdiag`. Description subtypes may be reported in diagnostics; mixed component/deployment/usecase content is one shared upstream description family, not invented independent parsers.

Correct `RenderError::InvalidSource` display from `invalid Mermaid source` to language-neutral `invalid diagram source`. Add a descriptive `UnsupportedFeature(String)` variant if that is simpler than conflating valid-but-unimplemented syntax with malformed input. Error strings contain `PlantUML <family> at line N, column M: <feature or offending value>`. Keep worker error transport as its existing string representation; inspect exhaustive matches before adding a public variant. No new transport fields are necessary.

## Verified public service APIs

Source prefix for the APIs below:
`/home/kb/.asdf/installs/rust/1.99.0/registry/src/index.crates.io-1949cf8c6b5b557f/`.
The workspace may patch these same versions for Mermaid repairs; direct dependencies must resolve through those workspace patches.

### Measurement and final SVG safety

Add `merman-render = { version = "=0.9.2", default-features = false }` as a direct dependency. It is already in the selected dependency closure. `merman-render/src/lib.rs` publicly exposes `environment`, `text` and `svg`; no default diagram/layout features are added by this direct edge.

Verified in `merman-render-0.9.2/src/environment.rs:1572–1743`:

```rust
let environment = merman_render::environment::RenderEnvironment::deterministic()
    .without_math_renderer()
    .with_resource_policy(policy)
    .with_text_measurement_policy(options.text_measurement.clone());
let session = environment.begin_session_with_control(control.clone())?;
let measurer = session.text_measurer(TextMeasurementPhase::Layout);
let metrics = measurer.measure(text, &style); // import merman_render::text::TextMeasurer
let report = session.text_measurement_report();
```

`TextMetrics` has only `width`, `height`, `line_count`; there is no public native ascent, descent or ink box. Do not invent these in measurement provenance. Use explicit line splitting and a documented baseline policy, initially PlantUML `StringBounder`'s default descent (`font_size / 4.5`) with a minimum line box tall enough for the chosen baseline. Record that baseline placement is approximate even when advance/height came from native shaping. A later metric-protocol extension can carry true ascent/descent. Wrapping is initially explicit-newline only; requesting an unsupported wrapping/style feature fails until a measured implementation exists.

The public session's `text_measurer` is not the private `controlled_text_measurer`; wrap each request with caller checks and bounds. Cache only per render with finite keys containing text and every style property. The host already limits RPC calls to 4,096 and text to 16 KiB per request; a PlantUML render must not flood it with repeated labels. Apply the same limits in headless mode to keep behavior comparable, or explicitly expose a smaller effective budget in diagnostics.

Verified in `merman-render-0.9.2/src/svg/pipeline/mod.rs:523–563`:

```rust
let safe = merman::svg::SvgPipeline::resvg_safe()
    .with_postprocessor(resources::ResourceAdmission::new(control.clone()))
    .process_resvg_compatible(&svg, &session)?;
let svg = safe.into_string();
crate::svg::validate(&svg, &options.limits)?;
```

This preserves terminal resource/tree validation. Do not rely only on `crate::svg::validate`, which is an additional, deliberately narrower check. Native emission uses no CSS or external attributes, but terminal validation remains mandatory. Family metadata is optional for terminal checks; do not label a PlantUML scene as a Mermaid family to obtain a capability token.

### Bounded Dagre kernel

Add direct `dugong = "=0.9.2"`, also already in the selected closure (MIT OR Apache-2.0 plus existing upstream notices). Its public graphlib re-export avoids a third direct dependency.

Verified in `dugong-0.9.2/src/pipeline/layout.rs:34`:

```rust
pub fn layout_controlled(
    graph: &mut dugong::graphlib::Graph<dugong::NodeLabel, dugong::EdgeLabel, dugong::GraphLabel>,
    work: &mut dyn dugong::WorkControl,
) -> Result<(), dugong::LayoutError>;
```

`WorkControl::charge(&mut self, units: usize) -> Result<(), WorkError>` must accept or reject an entire tranche before mutating the counter. Use checked arithmetic; map cancellation/limit failures into a stored enum reason so `WorkError::Interrupted` retains its real cause. `layout_controlled` works transactionally on temporary graph state. Never call unbounded `layout` with `NoopWorkControl`.

Graph calls verified in `dugong-graphlib-0.9.2/src/graph/core.rs`: `Graph::new(GraphOptions)`, `set_graph`, `set_node`, `try_set_edge_named(v, w, Some(edge_id), Some(label))`, `node_ix`, `try_set_unparented_parents_ix(&[(child_ix, parent_ix)])`, `graph`, `node`, `edge_by_key`. The checked parent batch at line 2195 validates cycles before mutation; each live child must be unparented and appear once. Validate IDs and charge a conservative `O((nodes + parents) log nodes)` tranche before the batch. Do not call `set_parent` with unvalidated input: it can assert on cycles. Prefer checked mutators over wrappers that discard errors. Configure directed multigraph and compound graph; edge IDs must remain unique so parallel relations are not collapsed. Node labels provide width/height and output centers; edge labels provide label size, rank minlen and output path points. Read all `Option` geometry through checked return handling in this adapter.

Dagre is not Graphviz dot. Preserve direction, edge identity, arrow ends, package containment and text; document different ordering, routing and spacing. Forced local edge directions, ports, link-to-cluster semantics and exact rank constraints need dedicated adapters/tests. Reject them while unavailable, rather than changing their meaning to a global graph direction.

## Module ownership and concrete contracts

All proposed paths below are relative to `diagram-ports/`. Avoid changes to current Mermaid modules except the narrow language-neutral public error text, dependency declarations and common fixtures as coordinated by the controller.

| File(s) | Ownership and contract |
|---|---|
| `engine/src/plantuml/mod.rs` | Whole operation: validate source/options, make child control/session, preprocess, parse, resolve, render, terminally validate, populate result. `render(&str, &RenderOptions) -> Result<RenderedDiagram, RenderError>`. |
| `engine/src/plantuml/source.rs` | Located lines, wrapper scan, literal-aware comments and preprocessing. `prepare_source(&str, &mut Budget) -> Result<PreparedSource, PumlError>`. Exactly one diagram block. |
| `engine/src/plantuml/lexer.rs` | Bounded UTF-8 cursor, escaped quoted identifiers/text, arrows, braces and punctuation. `lex(&PreparedSource, &mut Budget) -> Result<Vec<Token>, PumlError>`. Tokens retain source spans; no regex over arbitrary nested source. |
| `engine/src/plantuml/diagnostic.rs` | `SourceSpan { start, end, line, column }`, located unsupported/syntax/limit errors; conversion into existing public error types. |
| `engine/src/plantuml/budget.rs` | `Budget::new(&RenderOptions)`, `checkpoint()`, `charge(usize)`, `admit_items(usize)`, `admit_depth(usize)`, checked byte/geometry checks; implements/backs Dugong `WorkControl`. Shared aggregate counters per render, no resetting per family/parser attempt. |
| `engine/src/plantuml/ast.rs` | Common IDs, declaration order, generic located wrapper, `DiagramAst` closed family enum, document chrome. Family-specific structs live with each family. |
| `engine/src/plantuml/parse.rs` | `parse(&PreparedSource, &mut Budget) -> Result<DiagramAst, PumlError>`. Deterministic wrapper/factory admission; consumes every substantive token before success. |
| `engine/src/plantuml/text.rs` | `measure_label(&Label, &mut RenderContext) -> Result<MeasuredLabel, PumlError>`; own `Label`/`TextRun`/`FontSpec`, newline decoding, simple explicitly supported Creole, cache and baseline policy. |
| `engine/src/plantuml/style.rs` | Typed colors/strokes/font settings and supported skinparam/style properties. Unknown properties have located errors; no silently ignored block. |
| `engine/src/plantuml/scene.rs` | Immutable `Scene { bounds, primitives }`, finite `Point/Rect`, typed path operations, fill/stroke, text runs, named local IDs. Primitive/node/item quotas charged before allocation. |
| `engine/src/plantuml/svg.rs` | `serialize_scene(&Scene, &mut Budget) -> Result<String, PumlError>`. Bounded writer and XML escaping; no raw attributes/CSS/markup from source. Stable semantic `data-*` IDs for tests. |
| `engine/src/plantuml/graph.rs` | `layout_graph(&MeasuredGraph, &mut Budget) -> Result<GraphLayout, PumlError>`; only shared node/edge/group geometry and Dagre conversion. Own graph semantics stay in family modules. |
| `engine/src/plantuml/sequence/{mod,parse,layout}.rs` | Ordered participants/events, activation/lifecycle stack, frames and dedicated native timeline scene. |
| `engine/src/plantuml/uml/{mod,parse,resolve,draw}.rs` | Class/object/description semantic declarations, aliases, nesting, compartments and UML decorations over measured graph layout. |
| `engine/src/plantuml/state/{mod,parse,layout}.rs` | Hierarchical state machine/pseudostates/regions and transitions; reuse graph adapter, preserve state semantics. |
| `engine/src/plantuml/activity/{mod,parse,layout}.rs` | Structured instructions and ports; direct bottom-up tile composition for decisions/loops/forks; no generic sequence coercion. |
| `engine/src/plantuml/tree/{mod,parse,layout}.rs` | Mindmap/WBS typed trees; preserve explicit sides, root and parentage; separate family drawing policies. |
| `engine/src/plantuml/data/{mod,json,yaml,layout}.rs` | Ordered data values and object/array compartments with links; strict bounded parsing before allocation/expansion. |
| `engine/src/plantuml/{timing,gantt,salt,nwdiag}/…` | Later independent native models/layouts, created only when their stage starts. |
| `engine/tests/plantuml.rs`, `engine/tests/plantuml/…`, `fixtures/plantuml/…` | Public black-box security/family tests plus semantic/geometry fixtures. Use one test harness with per-family data to avoid duplicate assertions. |
| `THIRD-PARTY-NOTICES.md`, `provenance/plantuml-mit-1.2026.9.json`, `README.md` | Exact source/copyright manifest and executable support ledger. No source path may point at rejected candidates or another PlantUML edition. |
| `viewer/src/…` existing language/example detection | Only integrate after backend admission: examples, family output/status, file-language detection for every admitted `@start…` wrapper and `.puml`/`.plantuml`; no second renderer or new worker. |

`RenderContext<'a>` groups `&RenderSession`, `&mut Budget` and bounded measurement cache, keeping functions at three or fewer arguments. `parse`, semantic resolution, layout and SVG emission stay separate. Use stable index-based `EntityId`, `GroupId`, `EventId`, `TextId`, never visible text as the storage key. Own enums encode `Parent::Root/Group(GroupId)`, `Label::Empty/Runs(Vec<TextRun>)`, `Endpoint::Entity/Port/Boundary`, `ArrowEnd::None/Open/Filled/Triangle/HollowDiamond/FilledDiamond/…`.

Expected implementation size for the shared foundation is approximately 1,500–3,000 Rust production lines plus 500–1,000 lines of meaningful tests: source/lexer/diagnostics/budgets 500–1,000; label/style/scene/SVG 700–1,300; Dagre adapter 200–500. These are planning estimates, not progress evidence. Rich Creole, full styles, true native baseline metrics, TIM and graph cluster/port fidelity are additional work. Avoid a 200-line universal renderer whose apparent breadth comes from discarding semantics.

## Preprocessing, admission and errors

1. Validate the caller limits before scanning. Accept UTF-8 text with BOM/CRLF normalization and preserve original line/column mapping. One balanced start/end wrapper; comments outside are allowed, substantive text outside or multiple diagrams is an explicit error. Initially require wrappers; unwrapped-snippet convenience would be an explicit later API behavior.
2. Handle apostrophe line comments and `/' … '/` block comments with lexical state so quoted strings/JSON/Salt text are not altered. Never implement comments by splitting every line at the first apostrophe.
3. `@startuml` allows multiple families. Upstream order is sequence, class, legacy activity, description, state, modern activity, then timing later in `PSystemBuilder`. `A -> B` alone is normally sequence; a class/object/state declaration changes which complete grammar can accept the block. Parse explicit syntax features and require whole-block consumption. Parser trials use `NoMatch/Matched/Error`, not exceptions, and share work counters. A distinctive unsupported timing command must not be reinterpreted as an actor name.
4. Exact wrappers such as `@startmindmap`, `@startwbs`, `@startjson`, `@startyaml`, `@startgantt`/`@startproject`, `@startsalt`, `@startnwdiag`, `@startchen`, `@startpacketdiag` choose those languages only. Timing belongs to `@startuml`; do not invent `@starttiming` as upstream syntax. Unsupported wrappers return a named error, never a graph.
5. Deny include families (`!include`, `!include_once`, `!include_many`, `!includeurl`, `!includesub`, local/URL/stdlib forms), `!theme` resolution, file/environment/date/network builtins and image/sprite resource reads. Resource-looking text inside ordinary labels remains text; explicit URL/link/image syntax is a capability request and is rejected.
6. Initially reject every substantive `!` directive except any individually implemented pure directive. Stage a small literal constant expansion later, with immutable definitions, token-aware expansion, source maps, cycles/depth/step/output limits. Do not accept full TIM by string replacement. `!if`, procedures/functions, loops, JSON loading and `%…` builtins each need semantics and budget tests before admission.
7. `newpage`, multiple blocks, `@pause`, `@unpause`, `@exit`, page splitting and output filenames are explicit unsupported features in the single-output API. An optional wrapper name may be retained as metadata only if documented; it must never name an output file. Do not silently return only page one.
8. Titles/legends/header/footer and common colors/fonts can be implemented incrementally. Until implemented, reject them. Parse multiline `skinparam`/`<style>` as a bounded construct and validate each property/selective rule; do not strip the block and call the source supported.
9. Empty source, empty postprocessed block, unknown commands, dangling group/frame delimiters, alias conflicts and unresolved references receive located errors. Some PlantUML families intentionally create implicit referenced entities: model that family-specific rule explicitly rather than rejecting all undeclared endpoints.

Expansion output initially may not exceed `limits.source_bytes` (same default 256 KiB); all source copies/strings and macro-produced items count toward the total. Model-item accounting includes entities, relations, members, group/frame markers, tree/data nodes and events, not just visible boxes. Nesting applies to wrapper constructs, groups, ASTs, label markup and data. Long token/label bounds are checked before host calls. Use checked arithmetic and bounds before capacity reservations.

The PlantUML work counter covers parsing/expansion, layout adapters, graph construction, measured-run composition and SVG emission, including charged pairwise algorithms. Terminal SVG validation has its own Merman session budget; `layout_work_units` reports native PlantUML charged work, with documentation that terminal validator internals are not added to that counter. Both subsystems observe the same operation control and configured limits. No hidden unlimited worker stage.

## Semantic family lowerings and fidelity boundaries

| Family | Native meaning and minimum useful admission | Layout and explicit remaining differences |
|---|---|---|
| Sequence | Declared and implicit ordered participants; aliases and participant shapes; both message directions; solid/dashed and open/filled arrow semantics; self calls; multiline labels; notes; nested `alt/else`, `opt`, `loop`, `group`; balanced activation and create/destroy events. | Dedicated participant lanes and vertically ordered events/frame spans. Width constraints account for messages, notes and participant heads. Teoz exact constraints, parallel `&`, anchors, slanted messages, complex autoactivation/autonumber formatting and page splitting remain separate work. Do not map sequence to a free graph. |
| Class/object | Class/interface/abstract/enum/annotation and object declarations; display vs alias; members/compartment separators; visible access/static/abstract markers when admitted; inheritance/realization/association/dependency/aggregation/composition; endpoint multiplicities; packages/namespaces and notes. | Measured UML node compartments + Dagre. Object headers remain underlined; an object is not a class renamed in metadata. Distinct link ends stay distinct. Source member order and cross-package references survive. Dot edge routes/ports/together/visibility selectors require their own work. |
| Component/deployment/usecase/description | `component`, `interface`, `node`, `database`, `artifact`, `folder`, `cloud`, `rectangle`, `actor`, `usecase` and their shorthand forms, only as their actual shape drawing is ready. Display/aliases, nested containers, stereotypes, direction, relations and notes. | Shared description model over compound Dagre. Actor must be a stick figure, usecase an ellipse, database a cylinder, node a deployment box, component its actual component decoration. Unsupported shape symbols return an error. Avoid family counts based solely on alternative keywords. |
| ArchiMate | Native `archimate #color "Display" as id` is an upstream description command, including package/multiline forms. | Basic native shape can reuse description geometry. `<<icon>>` is translated upstream to `$archimate/<icon>` sprite lookup; do not substitute literal stereotype text and claim icon support. Import/trace permitted Archi sprites separately or reject icon-bearing syntax. Stdlib ArchiMate macros are not built-in parser syntax and remain unavailable while includes are denied. |
| State | Named/aliased states, `[*]` initial/final role, transitions with labels, nested composite states and state-body descriptions; unique scoped pseudostates. | Hierarchical state scenes over graph layout. State entry/exit/do labels remain attached to the state. Concurrent regions, history `[H]`/`[H*]`, choice/fork/join and cross-region transitions require corresponding model semantics, not ordinary circles and boxes. |
| Modern activity | Start/stop/end distinctions; `:action;`; if/elseif/else/endif and guards; while/endwhile, repeat/repeat while; fork/fork again/end fork with proper split/join; groups/partitions. | Structured tiles with entry/exit ports, guard labels, real branch convergence and loop-back edges. Dead paths after stop are not connected to later actions. Swimlanes, break/kill/detach, switch, backward labels and legacy activity are separate grammar/layout increments. |
| Mindmap | Root and ordered children, `*`/`+`/`-` levels, left/right sides, multiline text, boxless `_` nodes and colors once implemented. | Native tree layout balances branch extents around root and keeps explicit sides. `CommandMindMapPlus` computes level as marker-count minus one; any `-` selects left. No Mermaid-indent approximation. Exact PlantUML finger curves and style cascade remain fidelity gaps. |
| WBS | Depth, parentage, sides/directions, named nodes, boxless/pseudo nodes and admitted extra links. | Own WBS layout/drawing policy over shared measured tree utilities. WBS old/new indentation/marker syntax must follow its `getSmartLevel`, not mindmap rules blindly. Unsupported crosslinks fail until emitted. |
| JSON | Ordered objects/arrays and scalar values with keys/types, escaped text, nested structures, empty containers and duplicate-key policy matching the inspected parser. | Native compartment/tree graph replaces SmetanaForJson. Keep keys, scalar values, array indices and connection origins. `serde_json::Value` default maps sort keys, so use a bounded custom `DeserializeSeed`/visitor into ordered vectors, or deliberate checked ordered-map configuration; do not silently reorder. |
| YAML | PlantUML's own YAML subset translates into ordered data values, including nested maps/lists and quoted scalars/block values as individually implemented. | Same data renderer. Reject aliases, anchors, tags and merge keys before any expansion. Avoid whole-document generic YAML loading followed by a post-hoc depth check. A bounded translation of the authorized `yaml/parser` grammar can avoid a new dependency; do not advertise general YAML specification compliance. |
| Timing | `robust`/`concise` tracks, named states, absolute/relative timestamps, state changes, constraints/messages and notes; clocks/binary/analog as separate typed track kinds. | Dedicated common time axis; monotonic event coordinates, per-track state heights, horizontal intervals and true cross-track messages. Not Mermaid sequence or timeline. Date formats, compact modes and calendar conversion are later. |
| Gantt | Task aliases/labels, duration, explicit dates, precedence, milestone, completion and resource/calendar constraints as individually admitted. | Dedicated deterministic date/task solver and time-scaled bars. English `NaturalGanttCommand` uses Subject/Sentence grammar; it is not an arrow graph. Working vs calendar days, closed dates, cycles, scheduling and locale are substantive; reject those not implemented. No host date/time lookup. |
| Salt | Nested cell/table layout, text, buttons, text fields, checkboxes, radio, dropdown, separators, table borders and spans as admitted. | Widget/table composition with measured minimum cell sizes. `CommandAnything` is just the upstream collection stage; strict widget parsing is still needed. Trees/tabs/menus/scrolling/image references require dedicated model cases. No generic label grid that discards widget kind. |
| Nwdiag | Named networks and ordered server identities, address/description properties, multi-network membership and groups. | Dedicated network buses and server columns with membership edges. The same named server on two networks is one entity, not two boxes. Address labels belong to their network attachment. Standalone links and group overlays need actual routing rules. |

## Legal source map and notices

All Java paths in the following table are relative to the authorized source tree. `p/` abbreviates `net/sourceforge/plantuml/`. These are grammar/reference anchors; the implementation must list every additional file whose logic/constants it translates.

| Rust area | Authorized reference anchors |
|---|---|
| Wrappers/dispatch | `p/core/DiagramType.java`, `p/PSystemBuilder.java`, `p/BlockUmlBuilder.java`, `p/BlockUml.java`. |
| Preprocessing | `p/tim/TimLoader.java`, `p/tim/TContext.java`, selected `p/tim/` command/expression classes only as their pure semantics are implemented. |
| Sequence | `p/sequencediagram/command/CommandArrow.java`, `CommandParticipant*.java`, `CommandGrouping.java`, `CommandActivate*.java`; `p/sequencediagram/SequenceDiagram.java`; selected `p/sequencediagram/teoz/CommunicationTile*.java`, `GroupingTile.java`, `NoteTile.java`, `LifeEventTile.java`; `p/real/` if constraints are translated. |
| Class/object | `p/classdiagram/ClassDiagramFactory.java`; `p/classdiagram/command/CommandCreateClass*.java`, `CommandLinkClass.java`; `p/objectdiagram/command/CommandCreateEntityObject*.java`; `p/command/NameAndCodeParser.java`; `p/abel/Entity.java`, `Link.java`. |
| Description/ArchiMate | `p/descdiagram/DescriptionDiagramFactory.java`; `p/descdiagram/command/CommandCreateElementFull.java`, `CommandLinkElement.java`, `CommandPackageWithUSymbol.java`, `CommandArchimate*.java`; selected `p/decoration/symbol/USymbol*.java` only when shapes are implemented. |
| State | `p/statediagram/StateDiagramFactory.java`; `p/statediagram/command/CommandCreateState.java`, `CommandCreatePackageState.java`, `CommandLinkState*.java`, `CommandEndState.java`. |
| Activity | `p/activitydiagram3/ActivityDiagramFactory3.java`, `Instruction*.java`; `p/activitydiagram3/command/CommandStart3.java`, `CommandStop3.java`, `CommandIf*.java`, `CommandElse*.java`, `CommandEndif3.java`, `CommandWhile*.java`, `CommandRepeat*.java`, `CommandFork*.java`; selected `ftile`/`gtile` classes for each implemented composition rule. |
| Trees | `p/mindmap/CommandMindMapOrgmode.java`, `CommandMindMapPlus.java`, `CommandMindMapDirection.java`, `MindMap.java`; `p/wbs/CommandWBSItemNew.java`, `CommandWBSItemOld.java`, `CommandWBSLink.java`, `WBSDiagram.java`, `WBSTextBlock.java`. |
| Data | `p/jsondiagram/JsonDiagramFactory.java`, `JsonDiagram.java`, `StyleExtractor.java`; `p/yaml/YamlDiagramFactory.java`, `p/yaml/parser/YamlParser.java`, `MonomorphToJson.java`. Do not translate SmetanaForJson layout. |
| Timing | `p/timingdiagram/TimingDiagramFactory.java`, `command/CommandRobustConcise.java`, `CommandClock.java`, `CommandAnalog.java`, `CommandBinary.java`, `CommandAtTime.java`, `CommandAtPlayer.java`, `CommandChangeState*.java`, `CommandConstraint.java`, `CommandTimeMessage.java`, player/time classes as needed. |
| Gantt | `p/gantt/GanttDiagramFactory.java`, `command/NaturalGanttCommand.java`, `lang/Subject*.java`, selected `lang/Sentence*.java`, `time/`, `core/`, and `draw/` as each feature is translated. |
| Salt/network | `p/salt/PSystemSaltFactory.java`, `PSystemSalt.java`, `factory/ElementFactory*.java`, associated element classes; `p/nwdiag/NwDiagramFactory.java`, `CommandNetwork.java`, `CommandElement.java`, `CommandGroup.java`, `CommandProperty.java`, `NwDiagram.java`, `next/GridTextBlockDecorated.java`. |
| Shared text/geometry | `p/klimt/font/StringBounder.java`, `p/klimt/creole/Display.java`, selected Creole/shape/color/decoration classes. No wholesale `klimt` import or external asset import. |

Retain the original MIT grant and copyright notice (Arnaud Roques and any other named authors on the actual translated file) in `THIRD-PARTY-NOTICES.md` and the appropriate local module notice. Authorized preprocessed MIT files may retain older upstream comment headers beneath the MIT distribution grant: provenance records must preserve the actual file and edition rather than inventing a cleaned history. Embedded third-party algorithm/data notices require their own review; aggregate MIT branding does not override those.

The manifest records artifact identity/hash already established by the controller, version/commit, exact relative file, file SHA-256, Rust destination, translated behavior, and applicable notice paths. For fixtures record fresh author/provenance and oracle identity separately. Do not claim Maven hash verification based merely on `CompilationInfo.java`.

Do not import themes, Archi sprites, stdlib/C4/AWS libraries, OpenIconic, Twemoji, font tables/files, Ditaa or mathematical assets until their distinct provenance and licences are traced. Already-audited Merman/Dagre embedded notices continue to apply to the reused services. Re-run `cargo deny` on the selected dependency graph even though both direct dependencies already exist transitively.

## Full inventory backlog beyond the first families

The accepted source inventory, not a marketing count, defines remaining work. `PSystemBuilder.java` additionally registers chart, packet, wire, Git, files, board, HCL, EBNF, regex, Chen ER, BPM and flow. Recommended later order is files/packet/board (small distinct models), HCL over the data renderer, Chen over the UML graph primitives, EBNF/regex sharing typed railroad expressions, then chart/Git/wire/BPM/flow. Each needs its own syntax and actual output contract; no empty placeholder scenes.

Ditaa requires an independent ASCII-grid/vector design because its current MIT path is raster-oriented. Math/LaTeX requires a reviewed native math renderer, not font measurement. Raw DOT is excluded from the original initial scope; future native DOT support would still need a grammar and semantics. Standalone Creole can reuse rich text when implemented.

Factory diagnostics/catalogues (version/license/help/color/font/icon/sprite lists, crash/dedication/donor output) are application utilities. They should receive explicit unsupported responses until intentionally implemented, not count as successfully ported diagram families. `CHRONOLOGY` appears as an enum but its factory registration is commented out; do not advertise it as a functioning source family.

## Prioritized implementation tasks

Every task starts with its listed failing tests, implements the specified contracts, runs the focused test target and then the complete workspace checks when it changes shared behavior. New admitted syntax has both positive and negative/unsupported coverage. These are ordered deliverables, not a promise of full parity within this session.

### Task 1: Complete bounded service spine and strict frontend

Files: `plantuml/{mod,source,lexer,diagnostic,budget,ast,parse,text,style,scene,svg}.rs`, narrow `lib.rs`/Cargo edits, notices/manifest, shared tests. Produces the contracts above; initially no public success for unimplemented families. Integrates measurement and SVG validation directly, tested through a private controlled scene fixture.

- [ ] Add `should_reject_multiple_blocks`, `should_keep_apostrophes_inside_quoted_labels`, `should_reject_unterminated_comment`, `should_reject_include_even_in_unsupported_macro_body`, `should_reject_unknown_substantive_commands`, `should_reject_newpage`, `should_report_plantuml_line_and_feature`, and `should_respect_custom_model_and_depth_limits`.
- [ ] Add `should_record_native_and_fallback_measurements`, `should_escape_xml_label_and_attribute_characters`, `should_bound_svg_before_allocation`, `should_reject_nonfinite_geometry`, `should_observe_cancel_before_and_after_measurement`, and `should_pass_terminal_svg_and_usvg_validation` using the private scene fixture. A blocked host callback is already tested at the worker boundary; engine callbacks remain caller-bounded.
- [ ] Implement strict common parsing, bounded writer, shared scene and session integration. No permissive catch-all command branch. Unknown styles/directives are errors.
- [ ] Verify focused tests, all existing Mermaid tests, and dependency licence/source checks. Do not change viewer admission yet.

### Task 2: Real sequence vertical slice

Files: `sequence/…`, fixtures and tests, parser dispatch. Consumes prepared source, measured labels and scene; produces frozen `SequenceModel`, `layout_sequence(&SequenceModel, &mut RenderContext) -> Result<Scene, PumlError>`.

- [ ] Add fixtures/assertions for ordered declarations and implicit participants, quoted aliases, Unicode/XML text, reverse/dashed/open arrows, self-message, multiline note, nested `alt/else/loop`, activation spans, create/destroy, and malformed/unbalanced frame/activation inputs. Assert lane order, event order, arrow endpoint ownership, note text and frame containment, not merely SVG success.
- [ ] Implement participant and event parsing with unique IDs and full token consumption; add horizontal spacing constraints and typed vertical event rows. Draw genuine lifelines and participant kinds.
- [ ] Verify `should_preserve_reverse_message_direction`, `should_keep_self_call_and_note_geometry`, `should_end_destroyed_lifeline`, `should_reject_unimplemented_parallel_anchor_syntax`, custom low work budget and deterministic output. Compare newly authored cases to MIT oracle semantics; record geometry differences.
- [ ] Expose the first honest PlantUML viewer example only after render success, host metric evidence and background capture pass.

### Task 3: Shared UML graphs, then class/object and description breadth

Files: `graph.rs`, `uml/…`, fixtures/tests. Consumes typed declarations/relations and measured boxes; produces `GraphLayout` and `layout_uml(&UmlModel, &mut RenderContext) -> Result<Scene, PumlError>`.

- [ ] First test Dagre graph semantics directly: two parallel edges stay two; self-loop remains; cyclic/disconnected graphs get finite geometry; compound child boxes lie inside parent bounds; custom low work budget interrupts inside the kernel; labels reserve space. Fail unsupported cross-cluster endpoint constraints explicitly.
- [ ] Implement checked graph adapter and primitive shape intersections; separately render each relation's start/end marker, dash pattern, role label and multiplicity. Use unique graph IDs independent of user aliases.
- [ ] Add class/object tests for aliases, compartment order, visibility/static/abstract text, aggregation vs composition vs inheritance, quoted multiplicities, package nesting and same short name in separate namespaces. Verify the exact resolved endpoint identities.
- [ ] Add description tests for component → database inside deployment node; actor → usecase inside system rectangle; nested packages, notes and mixed shape kinds. Verify each actual shape kind and container ownership, not a family string.
- [ ] Keep unsupported decorations, ports, layout constraints, visibility selectors and sprite-bearing ArchiMate constructs as named errors. Admit icon-free native ArchiMate only once its actual shape/color semantics are implemented.

### Task 4: State and structured activity

Files: `state/…`, `activity/…`, fixtures/tests. Share drawing/text/graph utilities, not ASTs.

- [ ] State tests: initial/terminal nodes are distinct, aliases and transition labels, nested composite ownership, state body lines; reject concurrent/history syntax until its dedicated implementation. Add the concurrent-region feature as a separate reviewable extension.
- [ ] Activity tests: if/elseif/else branches converge correctly, while and repeat edges point back, fork has separate lanes and join, guards remain on edges, action text retains semicolons/newlines as grammar permits, stop does not create a fallthrough path. Test malformed stack endings and depth budget.
- [ ] Implement immutable models, bottom-up activity tiles with explicit input/output ports, and a distinct state graph adapter. Expand swimlanes/break/kill/detach only with route/lifecycle tests.
- [ ] Compare branch/loop topology to oracle fixtures before considering geometry polish.

### Task 5: Mindmap/WBS and bounded JSON/YAML

Files: `tree/…`, `data/…`, fixtures/tests.

- [ ] Tree tests: marker-defined parentage, left/right branches, depth changes, boxless nodes, multiple roots/invalid jump policy against source, WBS directions/aliases and crosslinks if supported. Assert subtree bounds and side of root.
- [ ] Implement distinct mindmap/WBS policies using shared ordered tree storage and bounded iterative or depth-limited layout.
- [ ] Data tests: ordered keys, escaped/Unicode values, nested arrays/objects, null/bool/number/string type labels, empty containers, very deep input rejected before construction, model-item budget, duplicate-key semantics. YAML aliases/tags/merge keys must fail before expansion; test exponential alias source specifically.
- [ ] Implement a bounded ordered JSON deserialization visitor and authorized PlantUML YAML subset; preserve source/line diagnostics and data semantics. Render object compartments and true child connections. Record different native geometry from upstream Smetana.

### Task 6: Shared syntax fidelity and bounded pure preprocessing

Files: `source.rs`, `style.rs`, `text.rs`, optional `preprocess/…`; family style application tests.

- [ ] Titles/legends/header/footer, explicit safe colors/fonts, supported skinparam properties and selected style rules must affect measured layout/output. Test unknown properties and selectors as errors, including unsupported declarations inside otherwise supported blocks.
- [ ] Add plain styled Creole runs with nested style/depth validation, correct escaped delimiters and explicit line breaks. Defer or reject links/images/sprites/math/tables until those specific models exist.
- [ ] Add pure literal definitions only with expansion cycles, shadowing, token-boundary, output-byte, recursion-depth and work-budget tests. Keep TIM functions/loops/conditionals unsupported unless the complete subset is implemented and tested. No general string substitution pipeline.

### Task 7: Timing, Gantt, Salt and Nwdiag as separate deliverables

- [ ] Timing: native common time axis with at least robust/concise state changes, relative/absolute time and a measured constraint; assert time-to-x monotonicity and lane association; reject clock/analog syntax until actual waveforms exist.
- [ ] Gantt: deterministic date/duration/dependency solver, task bars and a milestone; verify closed-day semantics before admitting them, cycle detection and large ranges. Reject source requesting unsupported calendar/resource rules.
- [ ] Salt: nested measured table with button, field, checked/unchecked controls, radio and border; assert widget kinds/cell/span ownership and errors for malformed nested blocks or unsupported images.
- [ ] Nwdiag: two buses, one shared server identity, separate address labels and a group; assert every membership connection and no duplicate shared server node. Add properties only when they affect output.
- [ ] Each has its own fresh source manifest entries and reference cases; a family is added to the viewer only after its substantive fixture passes.

### Task 8: Broad backlog and parity iteration

Work through the inventory backlog in the order above, taking each family through parse/model, topology, geometry, SVG and viewer gates. Concurrently close shared style/Creole/TIM/arrow/port/page gaps based on real documents. Full compatibility cannot be declared while known substantive syntax is intentionally rejected. Future multi-page API design should be explicit rather than changing single-SVG output silently.

## Practical release sequence for this session

The first useful session checkpoint is Tasks 1–3: bounded common infrastructure, a nontrivial sequence renderer and one shared UML graph renderer covering class/object plus component/deployment/usecase shapes. This is meaningfully broader than three unrelated toy parsers because it shares typed aliases, relations, nesting, text and safety while preserving distinct semantics. It still must be labelled a subset.

If these gates pass, Tasks 4–5 broaden the backend to state/activity/tree/data. Timing/Gantt/Salt/Nwdiag follow as their own independently reviewable tasks, not as renamed generic scenes. A sustained session may complete several stages; source scale and the oracle results make a fixed promise of all-family completion irresponsible. Report actual admitted constructs and test evidence at each checkpoint. Do not stop at the first slice if more safe authorized work remains and resources allow continued progress.

## Review focus and verification gates

Five especially important cross-feature inputs, assigned to tasks above:

1. Quoted labels containing apostrophes, colons, braces, URLs, XML metacharacters or arrow text: retain text and location; never treat it as syntax or I/O. Tasks 1–3/5.
2. Reused aliases and nested scopes plus parallel/self edges: stable entity identity, correct relation ownership, no dropped links. Tasks 2–4.
3. Unsupported syntax in an otherwise valid diagram (style block, page, macro, sprite, concurrent region): report the unsupported construct rather than partial success. Tasks 1–7.
4. Custom tiny limits, cancellation during parser/layout, long labels and YAML expansion bombs: reject before excessive allocation/expansion and preserve the worker's hard deadline. Tasks 1/3/5/6.
5. Native measurements that decline or return invalid values: honest fallback counts, deterministic fallback, finite non-overlapping layout and no claim of native baseline/bbox parity. Tasks 1–3 and viewer gate.

For each family fixture assert selected family, all expected visible labels, model identities/topology, positive bounded viewBox, shape/decorations, containment and finite geometry. Parse the generated SVG with the existing `usvg 0.46.0` path. Source traces and golden images complement semantic assertions; merely finding `<svg>` is not acceptance evidence.

MIT oracle fixture generation records exact jar/version/hash, fonts, dot version/layout setting, limits and source provenance. Java/dot remain developer-only. Graph comparisons prioritize semantic topology, shape kind, label containment and direction because Dagre/dot geometry is deliberately different. Dedicated sequence/activity/tree/table layouts still need a documented mismatch ledger until upstream geometry comparisons pass.

An independent original-renderer baseline is now available at `/tmp/ive-plantuml-native-oracles-report.md`, with sources and manifest under `/tmp/ive-plantuml-native-oracles/`. Its 30 freshly authored cases have matching source/SVG/authorized grammar hashes; 28 contain meaningful original diagrams and expected painted labels. The two math/LaTeX cases are explicitly degraded raw-expression fallbacks because the MIT jar lacks `org.scilab.forge.jlatexmath.TeXFormula`; do not count them as equation rendering. This is original-side evidence only, not native acceptance. The report includes the exact MIT jar hash `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1a515f7f70dd` and per-family arguments. JSON/YAML reference runs must omit the original CLI `-SdefaultFontName` injection, which otherwise changes their parsing into an error placeholder. The native API should not emulate that CLI bug. The source stems map directly to Tasks 2–8; future tests should read their actual `.puml` bytes and assert semantics beyond label counts.

Integration gate from `diagram-ports/`: `cargo test --workspace`; `cargo clippy --all-targets -- -D warnings`; `cargo build --release`; `cargo deny --config ../deny.toml check licenses sources`. Reuse the configured parent target directory if the controller requests it; avoid contention with an active Mermaid build. Capture a background pgui viewer frame for representative sequence, nested graph, activity and data cases with `--capture`; ensure every test process exits. This standalone viewer has its own capture path and no IVE state/profile; no need to manipulate the owner's IVE window.

## Design self-review and remaining decisions

All public API claims above were checked against exact cached 0.9.2 sources; product source was read only. The document does not promise that available APIs automatically provide PlantUML semantics. It explicitly accounts for measurement baseline limits, private Merman budget state, parallel edge IDs, wrapper dispatch, YAML pre-expansion bounds, source provenance, external asset denial and unsupported-feature errors.

Before implementation begins, the controller should confirm the Mermaid verification gate and the first staged ownership boundary. There is no new user authorization requirement introduced by this document. The owner already authorized local implementation and coordinated agent execution. No publishing/pushing or modification of the user's running IVE is proposed.
