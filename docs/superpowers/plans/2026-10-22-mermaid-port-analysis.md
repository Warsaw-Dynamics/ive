# Mermaid and dagre-es: Rust translation analysis

Date: 2026-10-22. Research and design only; no engine implementation is included here.

This document covers handoff items 1–6. The companion combined plan owns the full existing-Rust survey and runtime-alternative comparison. The owner's subsequent direction is to try a full port, Mermaid first and PlantUML second, in a separate Rust project with a standalone pgui viewer first. Other dependencies are allowed subject to the licence policy. The small profiles below are intermediate verification milestones, not a replacement for that full-port objective. Existing native work, especially Merman, should be evaluated before writing another implementation.

## Evidence and size conventions

The inspected local sources are:

| Source | Declared package version | Exact revision | Licence evidence |
|---|---|---|---|
| `/mnt/data/Projects/mermaid` | `packages/mermaid/package.json`: 12.2.0 | `6d8921f94f9a12b76586eddcf45a566080d4767e` | root `LICENSE`, MIT, Knut Sveidqvist, 2014–2022 |
| `/mnt/data/Projects/dagre-es` | root `package.json`: 7.0.14 | `4b95ad0a0bca6ce4cfa4cb83dfa9be19ca740e8f` | root `LICENSE.md`, MIT, original dagre/dagre-d3/graphlib and fork contributors |

These are revision-pinned findings, not a claim that all published packages labelled 12.2.0 contain this exact surface. The Mermaid checkout includes newer-looking additions such as agentflow, swimlanes, usecase and railroad variants; its last commit reports 2026-10-19. Its configuration defaults to ELK. Assuming the older familiar Dagre default would produce an incorrect parity target.

Counts below are physical newline counts, including comments and blank lines, for recursively selected `.ts`, `.js`, `.jison` and `.langium` files. Files containing `.spec.` or `.test.`, and directories `tests`/`__tests__`, are excluded. Styles encoded in TS/JS count; SCSS, YAML, JSON, docs, image assets and generated parser files absent from the checkout do not. Counts measure reference material, not the number of Rust lines required. Separate shared tables must not be added twice to diagram sizes.

Reproduction: recursively enumerate the named directory, apply the extension/exclusion rule above, and sum the number of `\n` characters in each UTF-8 file. For geometry API locations, search `packages/mermaid/src` for `\.(getBBox|getComputedTextLength|getBoundingClientRect)\s*\(` with those exclusions, discarding lines starting with `//` or `*`. This is a source audit, not an execution trace or a proof that every legacy renderer is still selected.

## 1. Pipeline and interfaces

| Stage | Original modules and interface | Native consequence |
|---|---|---|
| Public entry | `src/mermaid.ts`, `src/mermaidAPI.ts`; render/parse orchestration | A synchronous pure engine API plus caller-controlled worker execution; no browser element id is needed to parse |
| Preprocess | `src/preprocess.ts::preprocessDiagram`; `diagram-api/frontmatter.ts`, `comments.ts`, `utils.ts::detectInit/removeDirectives` | Normalize CRLF; extract title/config; legacy `%%{init:...}%%` directives; remove comments while retaining source offsets. This is not a general include/macro preprocessor |
| Configure | `config.ts`, `defaultConfig.ts`, `schemas/config.schema.yaml`, `config.type.ts`, `themes/`, `styles.ts` | Resolve defaults, caller options, diagram defaults and allowed source configuration into an immutable value |
| Detect/register | `diagram-api/diagram-orchestration.ts::addDiagrams`, `detectType.ts`, `diagramAPI.ts::registerDiagram/getDiagram` | Ordered detection matters. Closed `DiagramKind` dispatch replaces mutable registries and dynamic imports |
| Parse | `Diagram.ts::Diagram.fromText`; diagram `parser/*.jison`, adapters in `packages/parser`; usecase Chevrotain parser | Jison `parser.parser.yy = db` feeds grammar actions into diagram databases. Langium creates an AST which an adapter lowers to a DB. Rebuild these parsers from grammars instead of porting their generator runtimes |
| Semantic model | `diagrams/*/*Db.*`; e.g. `FlowDB`, `SequenceDB`, `ClassDB` | Resolve ids/aliases, preserve declaration order, validate references, construct owned models. Most original DBs are partly semantic state and partly presentation configuration |
| Common graph handoff | `flowRenderer-v3-unified.ts`, `classRenderer-v3-unified.ts`, `stateRenderer-v3-unified.ts`, `erRenderer-unified.ts`; `db.getData() -> LayoutData` from `rendering-util/types.ts` | Separate domain models from measured graph inputs rather than cloning the large optional-property `BaseNode` |
| Measure/layout | `rendering-util/render.ts` selects layout loaders; `layout-algorithms/{dagre,elk,cose-bilkent,swimlanes}`. Sequence uses `sequenceRenderer.ts` directly | Native layout requires measured label and shape dimensions before positions. Original rendering and measuring are interleaved, so extracting this boundary is substantive work |
| Draw | `rendering-elements/{nodes,edges,clusters}`, `shapes/`, intersections, markers; per-diagram `svgDraw.*` and renderers | Build a small owned scene/SVG tree with computed bounds, explicit styles and deterministic ids; replace d3 selections and DOM reads |
| SVG finalization | `setupViewPortForSVG.ts`, `setupGraphViewbox.js`, `mermaidAPI.ts`; title/accessibility, styles, viewBox, serialization, sanitization, callbacks | Serialize self-contained SVG geometry, text and attributes. Return errors instead of browser error diagrams; callbacks/interactivity belong to a later caller API |

`Diagram.fromText` also encodes entities, appends a newline, clears/initializes DBs and chooses comment-preserving parsing for DBs that request it. A Rust parser should retain original spans without reparsing an entity-encoded pseudo-source. Compatibility tests must still cover entity decoding, implicit node creation and duplicate declarations.

### Layout default: a material compatibility decision

`schemas/config.schema.yaml:79` declares `layout: 'elk'`. `rendering-util/render.ts` bundles ELK in large-feature builds and falls back to Dagre in the tiny build. `diagrams/flowchart/flowRenderer-elk-default.spec.ts` asserts both this default and explicit Dagre selection. Class, ER, state, requirement, agentflow and usecase also select the common registered layout. Mindmap supplies a `cose-bilkent` fallback if its requested layout is unavailable; architecture directly uses cytoscape/fcose.

The first native hierarchical implementation can target Mermaid configured with `layout: 'dagre'`. That comparison is useful and honest, but is not default-ELK visual parity. ELK/elkjs must not be translated under the handoff's licence rule; a permitted independent implementation or a separately assessed permissive implementation is a later project. Do not silently treat all layout names as Dagre.

### Dagre pipeline

`dagre-es/src/dagre/layout.js::layout` copies whitelisted graph, node and edge attributes into a layout graph, runs the algorithm and copies node centres, edge points/label positions and overall bounds back. `runLayout` performs:

1. Make room for edge labels; remove self edges; break cycles (`acyclic.js`, `greedy-fas.js`).
2. Build nesting constraints (`nesting-graph.js`), assign ranks (`rank/`), inject edge-label proxies, normalize ranks and long edges (`normalize.js`).
3. Parent dummy chains and insert borders (`parent-dummy-chains.js`, `add-border-segments.js`), minimize crossings (`order/`).
4. Restore self edges, adjust direction, position nodes (`position/bk.js`, Brandes–Köpf), route self edges, remove dummy/border nodes.
5. Restore edge labels, undo direction transforms, translate bounds, intersect node rectangles, reverse cycle-broken edge points and restore edge direction.

`rank/index.js` selects network simplex by default, with tight-tree and longest-path alternatives. `rank/network-simplex.js` includes feasible trees, cut values and edge exchange. `order/index.js` sweeps until four consecutive non-improving iterations, resetting that counter on improvement. These loops need operation budgets in Rust; merely moving them off the UI thread does not bound them.

The original algorithm expects insertion-ordered graph operations, multiedges, compound parents and temporary dummy nodes. Sorting ids differently or iterating a Rust `HashMap` changes tie-breaking and therefore coordinates. Node and edge declaration order must be represented explicitly.

## 2. Diagram and infrastructure inventory

All Mermaid paths in this section are relative to `/mnt/data/Projects/mermaid/packages/mermaid/src/diagrams/`. `G` means the shared measured graph/shape/layout pipeline; its current requested default is generally ELK, with Dagre selectable. `D` means diagram-specific layout and d3/SVG drawing rather than Dagre. A type's directory size includes legacy implementations where present.

| Directory / diagram family | Files | Lines | Parsing and layout | Sharing / extra work |
|---|---:|---:|---|---|
| `agentflow` | 13 | 4,691 | Jison; G | Flow-like graph semantics and shared shapes |
| `architecture` | 10 | 2,034 | Langium; cytoscape/fcose | Groups, junctions, port/spatial constraints; separate force engine |
| `block` | 10 | 1,766 | Jison; own `layout.ts`, graphlib data | Column spans, nested grids, custom block arrows |
| `c4` | 9 | 2,730 | Jison; D in `c4Renderer.ts` | Shared C4 shapes/label helpers; its own row/boundary placement |
| `class` | 11 | 2,777 | Jison; G in selected v3 renderer | Class compartments, member syntax, relations; includes legacy direct Dagre renderer |
| `cynefin` | 8 | 884 | Langium; D domain regions | Shared text/theme helpers |
| `er` | 9 | 2,013 | Jison; G in unified renderer | ER table measurement and crow's-foot markers; legacy direct Dagre retained |
| `eventmodeling` | 7 | 1,098 | Langium; D lane/box placement | Common text dimensions, custom semantic lowering |
| `flowchart` | 10 | 2,928 | Jison; G | Main shared node/edge/cluster consumer; graph/flowchart aliases and ELK detector |
| `gantt` | 6 | 2,319 | Jison; D date/time scales | Dayjs semantics, task dependency dates, calendar handling |
| `git` | 7 | 2,607 | Langium; D branches/commits | Custom commit placement, tags and merge geometry |
| `info` | 6 | 93 | Langium; static text | Mostly dispatch/version output |
| `ishikawa` | 7 | 774 | Jison; D fishbone | Custom rotated labels and bones |
| `kanban` | 7 | 673 | Jison; D columns/cards | Reuses insertNode/insertCluster and measured shapes |
| `mindmap` | 8 | 1,186 | Jison; shared layout selection, force fallback | Shape pipeline; optional tidy-tree package is separate |
| `packet` | 7 | 385 | Langium; D bit-field grid | Dedicated simple geometry |
| `pie` | 7 | 500 | Langium; d3 pie/arc | Legends and title measurement |
| `quadrant-chart` | 7 | 1,217 | Jison; D coordinate plot | Axis/point placement, themes |
| `radar` | 7 | 564 | Langium; D polar plot | Dedicated radial geometry |
| `railroad` | 16 | 2,140 | Langium adapters; recursive D railroad layout | Railroad, EBNF, ABNF and PEG share AST/rendering; four detectors |
| `requirement` | 7 | 871 | Jison; G | Multi-section requirements and special markers |
| `sankey` | 7 | 521 | Jison; d3-sankey | Weighted flow layout, alignment variants |
| `sequence` | 9 | 5,901 | Jison; D `sequenceRenderer.ts` | Actor spacing, message event stream, nested fragments, activation/self-call geometry |
| `state` | 11 | 3,108 | Jison; G in selected v3 renderer | Compound states, start/end/fork/join; legacy renderer/shapes included |
| `swimlanes` | 3 | 63 | Reuses flowchart parser/DB/renderer; swimlane layout | Almost all implementation lives in shared layout-algorithms/swimlanes |
| `timeline` | 8 | 1,816 | Jison; D horizontal/vertical timelines | Shared text and dedicated event geometry |
| `treeView` | 9 | 840 | Langium; D indentation/connectors | Icon packs, recursive text/tree drawing |
| `treemap` | 8 | 976 | Langium; d3 hierarchy/treemap | Label truncation and area/value formatting |
| `usecase` | 14 | 5,378 | Chevrotain; G | Substantial lexer/parser, boundary semantics, actor shapes |
| `user-journey` | 7 | 1,151 | Jison; D task/person grid | Text wrapping and person legends |
| `venn` | 7 | 811 | Jison; `@upsetjs/venn.js` | Circle/intersection optimization, rough styling optional |
| `wardley` | 8 | 1,825 | Langium; D map coordinates | Dedicated builder/renderer and label alignment |
| `xychart` | 18 | 2,288 | Jison; D scales/chart builders | d3 linear/band scales and paths; pluggable text dimension calculator |
| `common` (infrastructure) | 7 | 823 | Shared DB, sanitization and math helpers | Not a user diagram |
| `error` (infrastructure) | 2 | 179 | Browser fallback drawing | Replace with typed error return |
| Total | 298 | 59,945 | 35 directories, including infrastructure | Railroad variants and aliases mean directory count is not detector count |

### Shared code outside individual types

| Source area | Files | Lines | What a native implementation needs |
|---|---:|---:|---|
| Mermaid `rendering-util/` | 192 | 37,565 | Label lowering/wrapping; shapes, intersections, markers, edge curves, clusters; layout wrappers; also testing infrastructure |
| Its `layout-algorithms/dagre/` | 2 | 1,291 | Mermaid-specific compound graph extraction, sizing, placement and routing glue, in addition to Dagre itself |
| Its `layout-algorithms/elk/` | 9 | 3,898 | MIT Mermaid adapter; depends on unported EPL elkjs algorithm |
| Its `layout-algorithms/swimlanes/` | 36 | 11,695 | Native TS lane/ordering/routing machinery; separate later translation |
| Its `layout-algorithms/cose-bilkent/` | 5 | 549 | Adapter, not the cytoscape engine |
| Its `layout-algorithms/layout-utils/` | 6 | 2,543 | Bounds, graph validation, scoring and layout checks |
| Its `layout-algorithms/ddlt/` | 14 | 1,099 | Deterministic layout fixture infrastructure; not a runtime requirement |
| Mermaid `themes/` | 14 | 5,220 | Default/dark/forest/neutral/base and other look/theme calculations |
| Mermaid `diagram-api/` | 10 | 817 | Detection, metadata/frontmatter, common interfaces |
| Mermaid `config.ts` + `defaultConfig.ts` + `config.type.ts` | 3 | 3,414 | Resolution/default values and generated type surface; schema YAML is additional |
| Mermaid `utils.ts` | 1 | 983 | Text measurements/wrapping, ids, entities, title helpers and config utilities |
| Mermaid `styles.ts` | 1 | 172 | CSS generation, mostly replaced by resolved SVG attributes |
| `packages/parser/src/` | 80 | 3,291 | Langium grammars, converters, token builders, validators, AST adapters; generated runtime absent |
| `packages/mermaid-layout-tidy-tree/src/` | 6 | 984 | Adapter to `non-layered-tidy-tree-layout`; dependency algorithm not counted |
| dagre-es `src/dagre/` | 27 | 2,915 | Hierarchical layout algorithms, excluding test files |
| dagre-es `src/graphlib/` | 17 | 2,438 | Ordered graph data and algorithms, excluding test files |
| dagre-es `src/dagre-js/` | 22 | 1,155 | d3 DOM renderer; replace most with native shape/label/edge drawing |

There are 19 production Jison grammar files in Mermaid's diagram directories. The Langium package contains 18 `.langium` files, including shared/auxiliary grammars. Usecase is a third parser family. Generated parser/runtime code is neither present nor included in the figures. Initial flowchart/sequence/class grammars alone are 646/423/440 lines respectively: 1,509 grammar lines plus semantic actions and DB behavior.

## 3. Capabilities and dependency decisions

Use established permitted Rust dependencies where they supply the capability reliably. Keep pgui at the viewer/text adapter boundary, with a headless engine below it. The original library's algorithm still needs translation or an already translated implementation when equivalent output is promised; a generic graph crate alone is not Dagre.

| Original capability | Native implementation | Initial scope consequence |
|---|---|---|
| JS regex lexer, Jison/Langium/Chevrotain | Grammar-derived tokenizer with spans, explicit lexical modes, recursive-descent/precedence or suitable table-driven productions | Preserve quoted labels, comments, escapes, arrow longest-match rules, semicolon/newline distinctions and keyword context. Splitting source by lines or `-->` is insufficient |
| JS `Map`, graphlib compound multigraph, object mutation | `Vec` arenas plus typed ids; ordered adjacency and explicit parent enum; local construction/layout scratch state | Port only used graph operations; retain stable insertion order and graph invariants |
| d3 selection/DOM construction and bounds | Owned scene items and owned SVG elements; analytical bounds for primitives/paths and unions for groups | No browser, HTML tree or repeated draw-measure-redraw loop |
| d3 curves and shape intersections | Translate selected curve formulas and shape intersection math | Straight-line routing is a scoped visual deviation; it does not implement Mermaid's curves |
| d3 scales/pie/sankey/hierarchy | Translate the relevant permissively licensed algorithms when their diagram family lands | Chart families cannot all be made equivalent by a generic graph layout |
| Browser text shaping/fallback/fonts | Caller-supplied text metrics; pgui text-system adapter for production | A character-count approximation is for deterministic tests or explicitly approximate preview only |
| CSS cascade, stylis, theme functions, khroma | Resolve supported theme/style properties to typed colours, stroke/fill/font values and explicit SVG attributes | Full arbitrary CSS is a separate language surface; do not accept and silently ignore it |
| `marked`, HTML labels, entity decoding | Bounded label AST for plain text, line breaks and later emphasis; escape XML during serialization | Default browser HTML labels must be lowered to SVG `<text>/<tspan>`; unsupported HTML gets an explicit diagnostic |
| KaTeX/MathML | Later native math layout producing geometry/glyph runs | MIT does not make KaTeX small. Reject math in the first slice rather than emit invisible MathML/foreignObject |
| roughjs | Later seeded rough path generation | Start with `classic`; seeded hand-drawn parity is additional work |
| Dayjs | Explicit date parser/calendar functions for Gantt | Full date grammar, exclusions, locale/timezone behavior are later scope |
| DOMPurify/sanitize-url, links/click callbacks | Build escaped SVG from typed data; reject executable attributes/URLs; explicit caller-approved link metadata later | No JS callbacks, dynamic icon fetching, URL/file fetch or embedded script |
| Iconify images/icons, browser image dimensions | Caller-supplied immutable asset table with bytes, media kind and dimensions | No filesystem/network access in engine; initial image/icon syntax can be unsupported |
| Async dynamic imports/global config/DOM scheduling | Static dispatch and immutable `RenderOptions`; bounded work functions | No asynchronous loading is needed in the core |
| Numeric JS behavior/ordering | `f64` geometry, checked integer counters, finite/bounded values and explicit rounding | Preserve upstream numeric defaults/tie breaks where meaningful; replace NaN/sentinel behavior with descriptive errors |

Candidate infrastructure crates, if not already supplied by the selected engine, include `indexmap` for insertion ordering, `lalrpop`/`lalrpop-util` or a grammar-derived handwritten parser, `regex` for bounded regular-expression needs, `quick-xml` for serialization, `serde`/`serde_json` for fixture/config data, `unicode-segmentation`/`unicode-linebreak` for text boundaries, and `usvg`/`resvg` for development raster comparison. `rustybuzz`/`ttf-parser` are alternatives when a caller cannot provide pgui metrics. These are proposals, not a completed dependency licence audit: pin versions/features and run `cargo deny` against the actual graph before adoption. Avoid adding a second shaping stack if the pgui adapter meets the contract.

IVE's `Cargo.toml` already pins the gpui crate to `https://github.com/Warsaw-Dynamics/pgui.git` revision `f5d5fa2f8f07b0be68442e694d22ed6064609e73`. The checkout at `/mnt/data/Projects/gpui-zed-main/crates/gpui/src/text_system.rs` exposes `shape_line`, `layout_line`, `bounding_box` and `typographic_bounds`; this identifies useful capability, not a guarantee that each API has identical availability in the pinned revision. Verify the actual dependency version when implementing. The requirement “depends only on pgui” concerns direct project dependencies; pgui itself has a large transitive dependency graph.

## 4. Text, fonts and every direct browser sizing site

The source audit found direct geometry API calls in 51 Mermaid source files, one optional tidy-tree renderer, and four dagre-es DOM renderer files. Some are text, some are whole-shape/group bounds and some are interaction hit rectangles. They should not all become font calls.

### Complete direct geometry call-site inventory

Paths here are relative to Mermaid `packages/mermaid/src/`. Numbers are source lines at the pinned revision; repeated profiler branches count as distinct sites.

| File | Lines | Native replacement |
|---|---|---|
| `utils.ts` | 726, 822 | Text metrics; title/group bounds |
| `setupGraphViewbox.js` | 50 | Scene/group bounds |
| `diagrams/agentflow/agentflowDb.ts` | 1114 | Tooltip event rectangle; omit from pure engine |
| `diagrams/architecture/svgDraw.ts` | 168, 176, 356, 384 | Label metrics; service/junction bounds |
| `diagrams/block/blockRenderer.ts` | 62 | Final scene bounds |
| `diagrams/block/renderHelpers.ts` | 150 | Shape bounds |
| `diagrams/class/classDb.ts` | 540 | Tooltip event rectangle |
| `diagrams/class/classRenderer.js` | 263 | Legacy scene bounds |
| `diagrams/class/shapeUtil.ts` | 43, 49, 58, 70, 74, 81, 87, 93, 168, 181 | Compartment text and shape unions |
| `diagrams/class/svgDraw.js` | 110, 203, 229, 255, 270, 275, 341, 353, 358 | Legacy labels, title/members/notes, class bounds |
| `diagrams/cynefin/cynefinRenderer.ts` | 280, 325 | Text badges |
| `diagrams/er/erRenderer.js` | 49, 107, 108, 131, 150, 346, 542, 645 | Legacy entity/attribute/key/comment text and scene bounds |
| `diagrams/flowchart/flowDb.ts` | 629 | Tooltip event rectangle |
| `diagrams/gantt/ganttRenderer.js` | 416, 449 | Task/section text |
| `diagrams/git/gitGraphRenderer.ts` | 351, 424, 929, 1065 | Commit/tag/branch labels |
| `diagrams/ishikawa/ishikawaRenderer.ts` | 29, 139, 190, 284 | Scene, rotated text and label bounds |
| `diagrams/kanban/kanbanRenderer.ts` | 81 | Card bounds |
| `diagrams/mindmap/svgDraw.ts` | 217 | Label metrics |
| `diagrams/pie/pieRenderer.ts` | 186, 254 | Legend/title metrics |
| `diagrams/railroad/railroadRenderer.ts` | 86 | Terminal/nonterminal/rule text |
| `diagrams/sequence/sequenceRenderer.ts` | 275 | Text line height |
| `diagrams/sequence/svgDraw.js` | 159, 167, 314, 498, 625, 765, 867, 950, 1116, 1211, 1609 | HTML/text metrics; actors, notes and fragment bounds |
| `diagrams/state/shapes.js` | 54, 90, 121, 149, 161, 176, 332, 340, 403, 475, 482, 497 | Legacy title/description/divider/cluster text and shape bounds |
| `diagrams/state/stateDb.ts` | 672 | Tooltip event rectangle |
| `diagrams/state/stateRenderer.js` | 71, 162, 168, 254, 269, 278 | Legacy group/scene bounds |
| `diagrams/timeline/svgDraw.js` | 478, 522, 580 | Word wrapping and label bounds |
| `diagrams/timeline/timelineRenderer.ts` | 206 | Scene bounds |
| `diagrams/timeline/timelineRendererVertical.ts` | 201, 215 | Scene bounds |
| `diagrams/treeView/renderer.ts` | 87, 197 | Label/description metrics |
| `diagrams/treemap/renderer.ts` | 265, 273, 279, 402, 448, 515 | Repeated fit/truncation width checks |
| `diagrams/user-journey/journeyRenderer.ts` | 39, 57, 68, 73, 105 | Full labels, words, hyphenated fragments and line widths |
| `diagrams/wardley/wardleyRenderer.ts` | 918, 920 | Text advance and ink box |
| `rendering-util/createGraph.ts` | 67 | Shape bounds |
| `rendering-util/createText.ts` | 73, 122, 135, 189, 190 | HTML box; styled-run advance; text ink bounds |
| `rendering-util/layout-algorithms/cose-bilkent/render.ts` | 76 | Measured node bounds |
| `rendering-util/rendering-elements/clusters/swimlane.js` | 53, 58 | Cluster title label |
| `rendering-util/rendering-elements/clusters.js` | 53, 58, 123, 164, 209, 214, 295, 336, 341, 406, 494, 565, 569, 632, 719, 723, 851 | Cluster labels and analytical container bounds |
| `rendering-util/rendering-elements/edges.js` | 114, 122, 124, 152, 156, 179, 183, 207, 211, 235, 239 | Main/start/end edge label boxes and transforms |
| `rendering-util/rendering-elements/shapes/c4LabelHelper.ts` | 79, 94 | Multi-section C4 label boxes |
| `rendering-util/rendering-elements/shapes/classBox.ts` | 94, 98, 101, 104, 192 | Compartments and shape bounds |
| `rendering-util/rendering-elements/shapes/datastore.ts` | 33 | Path bounds |
| `rendering-util/rendering-elements/shapes/erBox.ts` | 76, 170, 388, 393 | Attribute text and box bounds |
| `rendering-util/rendering-elements/shapes/icon.ts` | 68 | Asset geometry |
| `rendering-util/rendering-elements/shapes/iconCircle.ts` | 52 | Asset geometry |
| `rendering-util/rendering-elements/shapes/iconRounded.ts` | 75 | Asset geometry |
| `rendering-util/rendering-elements/shapes/iconSquare.ts` | 75 | Asset geometry |
| `rendering-util/rendering-elements/shapes/rectWithTitle.ts` | 48, 54, 66, 91 | Title/body text boxes |
| `rendering-util/rendering-elements/shapes/requirementBox.ts` | 112, 113, 245, 255 | Requirement compartments and whole-box bounds |
| `rendering-util/rendering-elements/shapes/usecaseActorIcon.ts` | 55 | Actor icon bounds |
| `rendering-util/rendering-elements/shapes/util.ts` | 128, 129, 137, 138, 195, 196, 203, 204, 243, 244 | Shared label and shape bounds, with profiler branches |
| `rendering-util/setupViewPortForSVG.ts` | 29 | Final scene bounds |

Additional size reads not captured by that three-method search:

- `diagrams/common/common.ts:321`: MathML HTML `clientWidth/clientHeight`; requires native math box layout, not just font width.
- `diagrams/gantt/ganttRenderer.js:103`: parent `offsetWidth`; replace with explicit viewport/diagram width in options.
- `rendering-elements/shapes/imageSquare.ts:18–19`: image `naturalWidth/naturalHeight`; replace with caller-provided asset dimensions.
- `packages/mermaid-layout-tidy-tree/src/render.ts:81`: node `getBBox`; measured node/shape bounds.
- dagre-es `src/dagre-js/create-nodes.js:38,72`, `create-edge-labels.js:25`, `label/add-label.js:21`, `create-edge-paths.js:103`: label, shape and edge-label bounds; the pure Dagre algorithm itself does not call the DOM.

Indirect text measurement must also retain behavior: `utils.ts::calculateTextDimensions` is used by C4, sequence, eventmodeling, class `shapeUtil`, ER/requirement boxes and sequence `actorBands.ts`; `calculateTextWidth/Height`, `wrapLabel` and `wrapLabelWithStyle` build on it. Sequence call sites are `sequenceRenderer.ts:414,470,927,1518,1593,1631,1687,1710,1742,2013` and `actorBands.ts:66`. Eventmodeling uses `db.ts:385`. `xychart/chartBuilder/textDimensionCalculator.ts:37` delegates to `createText.ts::computeDimensionOfText`, which also measures styled runs for wrapping. Railroad's private `measureText` is used at lines 99, 137, 474 and 542. These consumers must use the same resolved font and metrics as drawing.

### Measurement contract

Use `TextMeasurer::measure(&TextRequest) -> Result<TextMetrics, TextError>`. `TextRequest` contains borrowed text or styled runs, a resolved font descriptor (family/fallback identity, size, weight, slant, spacing) and line direction/language information where supported. `TextMetrics` must expose advance width, ink bounds with offsets, ascent, descent and line advance. Width/height alone cannot represent overhanging italics or distinguish SVG ink bounds from CSS line boxes.

Line splitting/wrapping is engine logic using this measurement interface. Break only at valid Unicode boundaries, preserve `<br>`/newline semantics, shape complete candidate runs rather than adding per-character widths, and cache bounded requests by text, font identity and font-generation id. For rich labels the engine measures styled runs and computes line boxes/baselines; a text metrics service is not a general HTML/CSS box engine.

`utils.ts::calculateTextDimensions` unusually measures both `sans-serif` and the requested family, rounds each line's box and chooses using its dimension comparison. That compatibility policy and its empty-line zero-width-space behavior are distinct from native font shaping. Preserve them when matching this path, or record an intentional difference. HTML `line-height: 1.5` in `createText.ts` is likewise different from glyph ink height.

A deterministic test measurer makes layout repeatable, but cannot establish real-font visual parity. Differential layout tests can feed captured upstream dimensions to isolate layout. Production rendering must shape with pgui using the same fonts used to draw. If SVG is rasterized using a different font resolver, match resolved fonts or provide glyph outlines through an explicit text-to-path capability; matching metrics alone cannot force SVG's eventual font choice. Validate the pgui adapter's worker-thread capabilities at the pinned revision; otherwise obtain bounded batches of metrics through the host before worker layout.

Scene bounds are computed from primitives and transformed unions, including text ink boxes and marker extents. Do not send rectangle/path/group queries to `TextMeasurer`. Test coordinate transforms carefully: browser `getBoundingClientRect` is in screen coordinates, whereas SVG `getBBox` is local. Native geometry stays in diagram units independent of display zoom.

## 5. Idiomatic Rust translation

### Data and stage boundaries

Use a `DiagramKind` enum and diagram-specific syntax enums, not a universal string-keyed object. The stages are `preprocess(source, options) -> PreparedSource`, `parse(&PreparedSource) -> DiagramAst`, `resolve(ast, options) -> DiagramModel`, `measure(&model, context) -> MeasuredDiagram`, `layout(measured, budget) -> PositionedDiagram`, `draw(&positioned, style) -> SvgDocument`, then bounded serialization. Group related arguments in context/options structs to keep functions small. The public convenience entry can be `render(source, &RenderContext) -> Result<SvgDocument, RenderError>`.

Useful owned models include `FlowGraph { nodes, edges, groups }`, `SequenceModel { participants, events }`, and `ClassModel { classes, relations, namespaces }`. Typed ids index vectors; maps resolve source names while vectors preserve order. `MeasuredNode` and `PlacedNode` are different types so width/position cannot be accidentally used before construction. An explicit root/parent enum represents group membership. Sequence events are semantic variants such as message, activation, note and fragment boundaries/tree nodes rather than numeric line-type flags.

The AGENTS.md field rule says not to store `Option` wrappers in fields. Represent meaningful alternatives as domain enums (`Parent::Root | Group(GroupId)`, `Label::Empty | Text(TextBlock)`, a `NodePosition` stage-specific type); return `Option` from lookups where natural. Do not replace missing values with ambiguous empty ids or sentinel coordinates.

Construction and layout algorithms need temporary working state. Keep it owned within a builder or a layout pass and publish immutable results; do not expose mutable DBs, shared object graphs or global config. This retains the original algorithms without replicating JS object architecture.

| Original pattern and concrete source | Rust form |
|---|---|
| `Diagram.fromText` mutates Jison `parser.yy` and calls DB methods | Tokenize/parse into AST, then resolve to owned domain model; parser instances hold no global DB |
| `FlowDB::getData()` lowers diagram state to broad `LayoutData` optional properties | Explicit `GraphLayoutInput` with typed node shape, dimensions, edge style and group hierarchy |
| `SequenceDB` uses `ImperativeState`, numeric `LINETYPE` constants and mutable actor/message maps | `SequenceEvent` enum and indexed participants; scoped construction followed by validation and immutable events |
| `ClassDB::addClass` creates nodes with global `classCounter` and later pushes members/relations | Per-document ids and a builder that resolves duplicate/implicit classes, then emits `ClassModel` |
| `railroad/parser/railroadParser.ts::transformExpression` switches on Langium `$type` strings with casts | Exhaustive `RailroadExpression` enum matching; no generated AST inheritance/casts |
| d3 `append(...).attr(...).style(...)` in `createText.ts` and sequence `svgDraw.js` | Constructors for text/path/group elements with typed paint and computed geometry |
| Node `intersect` function fields installed by shapes | `Shape::intersect(&self, bounds, point)` exhaustive matching; traits only for a deliberate external extension boundary |
| dagre `Graph` dynamically stores rank/order/border/dummy fields | Private typed layout arena and stage-specific scratch arrays indexed by ids |
| dagre `buildLayoutGraph/updateInputGraph` whitelist attributes | Validated immutable input and explicit layout output, retaining defaults, ranks and tie-breaking |
| `getConfig/setConfig`, module registries, memoized global text measurements | Immutable resolved config plus render-scoped bounded cache and explicit measurer |
| Thrown JS errors plus browser fallback SVG | Typed `RenderError` with phase, source span and offending value, returned to caller |

### Rewrite infrastructure, translate behavior

Rebuild grammar parsing from Jison/Langium/Chevrotain source definitions and their tests. Do not translate generated parser runtimes, JS module loaders, d3 selections, fastdom scheduling, browser sanitizers or CSS cascade wholesale. A typed owned SVG writer needs only XML escaping, deterministic ids, stable numeric formatting and a bounded output buffer; it does not need a general XML parser. Style resolution should emit fill/stroke/font attributes directly. Use `<text>/<tspan>`, shapes, groups, paths, clip paths and markers compatible with IVE's SVG renderer; avoid scripts, external resources, `<foreignObject>` and browser-only CSS/MathML.

Translate Dagre's algorithm behavior, including compound nesting, label dummy nodes, self edges, cycle reversal, ordering and network simplex. A single topological-sort grid is useful for a prototype but is not a translation of Dagre. The 5,353 lines of Dagre+graphlib and 1,291 lines of Mermaid Dagre glue are a concrete lower bound on the reference surface for this layout family, before shapes and labels. Port only necessary graphlib capabilities rather than recreating its entire public API.

The three initial type directories contain 11,606 lines. Sequence's renderer/drawing alone are 4,328 lines; label and actor quirks are part of its layout semantics. That does not justify a line-for-line Rust copy, but it rules out describing a small arrow parser as complete sequence compatibility.

### Practical first milestones within the full-port objective

The owner chose to try the full port. The following profiles identify useful early differential test gates, not permission to stop at a syntax subset. With an existing broad engine, enable and audit its actual coverage rather than deliberately removing grammar features. During any incomplete translation, publish exact coverage and reject unsupported constructs with their location. Do not silently skip unknown lines or reclassify valid unsupported syntax as successful rendering.

| Family | Practical first end-to-end profile | Follow-up work before claiming broad fidelity |
|---|---|---|
| Flowchart | `flowchart`/`graph`, TB/TD/BT/LR/RL; quoted/plain labels; rectangle/rounded/diamond; basic directed/undirected/dashed/thick links and text labels; stable declared/implicit nodes; cycle/self-loop cases when Dagre stages exist | Complete shape registry and metadata syntax, chaining/multi-target edges, nested subgraphs and their direction rules, group crossings, edge ids/styles/animations, markdown labels, CSS/theming, icons/images, ELK parity |
| Sequence | Participant/actor declarations and aliases; implicit participants; solid/dashed arrow messages; self messages; explicit line breaks; notes; basic activation when validated | Full arrow vocabulary, create/destroy, boxes, autonumber options, nested loop/alt/opt/par/critical/break/rect fragments, participant glyph variants and look differences, wrapping and all placement quirks |
| Class | Class declarations with attribute/method compartments; basic visibility; labelled associations and inheritance/dependency/aggregation/composition; direction and multiplicities | Generic/member parsing details, interfaces/lollipops, namespaces/nesting, annotations and styling, notes/links, complete terminal labels and layout variants |

Delivery can start with one path per family and grow fixture by fixture. A profile with correct declared scope is a first milestone; it does not discharge the request for all current grammar productions and behavior. Port or adopt complete individual layers/types in stages and label intermediate builds incomplete instead of changing the target to a subset.

Fresh-translation effort estimate, not a commitment: a useful three-family subset is work measured in focused weeks once project/tooling decisions are fixed; broad tested fidelity across those families and shared layout is a multi-month engineering effort. All diagram families plus native text/math/rough/force/ELK alternatives is a larger programme. Adoption of mature existing Rust work can reduce this substantially, but only a real integration/audit can quantify the savings. Confidence is limited until differential fixtures exercise the real font stack and compound layout. Counting types or producing three sample SVGs is not a credible completion criterion.

### Existing broad implementation changes the starting point

[Merman](https://github.com/Latias94/merman) reports a browserless Rust parser/layout/SVG pipeline. Its compatibility notes distinguish published `0.9.1` targeting Mermaid 12.1.0 from the development branch targeting 12.2.0, and explicitly acknowledge browser-font/HTML/hand-drawn differences. Treat those as upstream claims to verify, not proof of parity. Its root code is MIT OR Apache-2.0, but feature-dependent third-party notices matter. [Compatibility and attribution](https://github.com/Latias94/merman#compatibility).

The [facade manifest](https://raw.githubusercontent.com/Latias94/merman/main/crates/merman/Cargo.toml) enables all diagrams plus ELK by default. A candidate evaluation should disable defaults and initially select `all-diagrams` plus `svg`; optional Cytoscape/math can be audited separately. `complete-svg` adds Cytoscape/math without ELK, while `complete-svg-elk` adds ELK. Selecting Dagre at runtime does not exclude a compiled ELK implementation. The [render manifest](https://raw.githubusercontent.com/Latias94/merman/main/crates/merman-render/Cargo.toml) makes `merman-layout-elk` optional and exposes separate Cytoscape/math features; it also uses existing graph, geometry, XML and Unicode crates.

Recommendation: perform the first standalone-viewer integration against a pinned permitted Merman feature graph, inspect its text/environment adapter contracts, prohibit ambient I/O, and run the corpus above. Adopt or fork permitted layers where the audit succeeds; translate identified gaps rather than start a second narrow parser. The parent observed candidate revision `83e3ea74bf4da30089a8012ec2d3418d40638e52`; verify downloaded content matches that revision before using it. Current-source API/feature documentation may differ from a published release. This analysis has inspected public documentation/manifests only, not built Merman, verified its dependency graph or inspected its EPL implementation. Full-port status must continue to identify missing default-ELK equivalence under the project's explicit exclusion rule.

### Resource limits and errors

Use explicit budgets for original source bytes, tokens, nesting depth, nodes/edges, expanded dummy nodes, text measures/cache bytes, label length, total output bytes, rank/order/network-simplex operations, force iterations and cancellation. Illustrative initial desktop limits to tune with corpus measurements: 1 MiB source, 5,000 nodes, 10,000 edges, nesting depth 64, 100,000 expanded layout nodes and 32 MiB SVG output. These are design estimates, not inherited Mermaid defaults; its schema currently has 50,000 `maxTextSize` and 500 `maxEdges`.

Check limits before expensive allocations and inside every potentially repeated pass. Bound dummy expansion as well as source node count; long edges/rank spans can multiply work. Keep `f64` values finite and bounded before serialization; validate nonnegative dimensions, valid endpoints and acyclic parent relationships. Return `UnsupportedFeature`, `Parse`, `InvalidModel`, `Text`, `LimitExceeded`, `Cancelled` or `Layout` errors with source span and offending input/value. Return an explicit bounded-work error rather than silently switching algorithms after a budget runs out. The core has no filesystem/network callbacks; any supplied assets are already resolved caller data.

## 6. Fidelity and verification

### Oracle and corpus

Pin the upstream revision, configuration, layout engine, look, seed and font files in every fixture manifest. Start graph comparisons with explicit `layout: 'dagre'`, `htmlLabels: false` and `look: 'classic'`, then add rich-label and other profiles separately. The manifest must distinguish the chosen comparison configuration from the source default; changing to Dagre is a visible compatibility decision.

Useful existing sources include:

- `packages/mermaid/src/docs/syntax/{flowchart,sequenceDiagram,classDiagram}.md` and other family syntax pages; avoid double-counting mirrored root `docs/` pages.
- Grammar/DB tests in `packages/mermaid/src/diagrams/{flowchart,sequence,class}/`, including `sequence/lifelineStart.spec.ts` and sequence actor-band behavior.
- `e2e/platform/dev-diagrams/layout-tests/` including `class-diagram*.mmd` and `elk-edge-cases/`; the latter should not be claimed as Dagre goldens without explicit re-rendering.
- `e2e/platform/dev-diagrams/diagrams/sequence-fixes/01-actor-vs-database.mmd` through `07-multiline-labels.mmd` and `knsv/knsv2-01-sequence.mmd` plus Dagre-labelled flowchart cases.
- `e2e/platform/dev-diagrams/performance/flowcharts/` for size/budget tests, separate from visual goldens.
- dagre-es `src/dagre/**/*.test.js`, `src/graphlib/**/*.test.js` and `test/graphlib/alg/`; these isolate graph/layout behavior from text.
- Mermaid `rendering-util/layout-algorithms/ddlt/` plus `.mmd`/`.sizes.json` fixtures: its fixed content/label sizes and invariant checks are a useful native-test pattern. Some DOMUS backends in this revision are explicitly unavailable stubs; do not assume every fixture runner works.

Oracle setup is not complete in the supplied checkout: `node_modules`, Mermaid `dist/` and parser generated sources are absent. Node alone is not a browser text layout engine. The developer-only baseline job needs pinned build dependencies and an actual browser for real measurements, or a previously generated trusted fixture bundle. `jsdom` with mocked `getBBox` is useful for logic tests but is not a visual oracle. No dependencies were installed and no original SVGs were generated during this analysis. Browser/Node use is confined to development/test tooling, never the shipped runtime.

### Compare separate layers

1. Parse/semantic fixtures: compare normalized nodes, edges, participants, event nesting, class members, source positions and defaults. Include invalid input diagnostics and valid-but-unsupported cases.
2. Dagre fixtures: feed identical measured graph inputs to original and native algorithms; compare rank/order where exposed, node centres/sizes, label centres and edge points. Preserve insertion order. Proposed starting tolerance: absolute `1e-6` diagram units for pure numeric fixtures, widened only for explained floating-point differences.
3. Full diagram geometry: capture upstream text/shape metrics and use them in native tests. Proposed initial tolerance: 0.5 diagram units for boxes/points/baselines. Match ids semantically, normalize transforms and ids, compare marker/edge topology and label content. Byte-for-byte SVG comparison is inappropriate because tree grouping and attribute order are deliberately different.
4. Real-font integration: freeze font files and fallback order, render both SVGs through the same renderer where possible, and compare raster output at fixed dimensions and several scales. Record antialiasing tolerances, bounding-box deltas, changed-pixel count and a saved heatmap. Establish thresholds from reviewed fixtures; a broad global pixel allowance can hide missing labels or arrows.
5. Invariants and hostile inputs: verify finite geometry, valid bounds/endpoints, label containment where promised, no duplicate document ids, no dangling references, deterministic output, budget termination and safe XML escaping. Do not impose an obstacle-routing invariant on Dagre that its original curved routing does not guarantee.

Use a small mandatory corpus per supported feature, then expand from upstream regressions. A proposed first gate is 20–30 reviewed cases per initial family plus isolated layout/escaping/budget cases, not an assertion that this number establishes full compatibility. Maintain a coverage matrix of upstream grammar constructs, supported profiles, expected unsupported diagnostics and known visual deviations. A fixture only counts as visual parity after an oracle was actually run and reviewed.

Tests in the Rust project should follow `should_…` names and `// given`, `// when`, `// then` sections. Keep deterministic fixture tests independent of a window, and isolate pgui metric/render tests. The parent's implementation plan should specify project-local `cargo test`, `cargo clippy --all-targets` and release build gates; running IVE's whole suite is not needed merely to add this research document. Later IVE integration must use its isolated-profile MCP workflow.

## Licensing and provenance to preserve

Mermaid's MIT root notice is `Copyright (c) 2014 - 2022 Knut Sveidqvist`. dagre-es `LICENSE.md` separately names original dagre-d3 (2013 Chris Pettitt), original dagre/graphlib (2012–2014 Chris Pettitt), and the fork's 2022–2024 contributors Thibaut Lassalle, David Newell, Alois Klink, Sidharth Vinod and dagre-es contributors. Preserve the complete notices verbatim with translated source and distributed binaries; do not replace them with the new project's copyright alone.

Each translated module should identify the original relative path(s), source repository and revision. Maintain a provenance manifest for translated algorithms, constants, theme palettes, example corpora and copied/generated data. A grammar-derived parser and a rewritten SVG emitter can still be derived works; changing language or structure does not remove attribution duties. Record which code is translated versus independently written.

The handoff identifies d3 as ISC, cytoscape/KaTeX/roughjs as MIT, and elkjs as EPL-2.0. The Mermaid root MIT licence does not relicense those dependencies. Before translating any additional dependency, inspect the exact dependency version's licence and file notices, including transitive algorithms, fonts and icon packs. Dependency sources were not installed here, so those individual packages have not been independently audited in this analysis. Do not infer icon/font/data licences from an adapter's MIT licence. Keep ELK implementation code out of this translation under the stated project rule.

When these engines ship in IVE, `NOTICE`, `THIRD-PARTY-NOTICES.md` and the Settings third-party data must include translated-source/data notices as well as pgui and all actual linked dependencies. Cargo lockfile-based notice generators do not discover manually translated TS/JS: add an explicit source/data-notice input and verify generated distribution output. Run the repository's licence checks when dependencies/integration change; this document itself changes neither dependency declarations nor licences.
