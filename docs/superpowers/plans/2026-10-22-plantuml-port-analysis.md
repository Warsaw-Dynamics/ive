# PlantUML Rust port analysis — 2026-10-22

Status: source analysis, not an implemented engine. Covers items 1–6 of
[the handoff](../specs/2026-10-22-diagram-ports-handoff.md). The subsequent owner direction is a new Rust project,
Mermaid first, then PlantUML, attempting full ports in stages, with a standalone viewer first. Other permissively
licensed dependencies are allowed after provenance and licence review. Keep the headless engine crates
GPUI-independent; put pgui font/drawing integration at the viewer boundary. Initial syntax slices are milestones,
not a reduction of the full-port objective. This document changes no product code.

All PlantUML inspection was confined to `/mnt/data/Projects/plantuml-mit/src`. No GPL PlantUML repository or
other edition was consulted. Paths below, unless qualified, are relative to that tree; `p/` abbreviates
`net/sourceforge/plantuml/`. `p/version/CompilationInfo.java:93` identifies version `1.2026.9`, commit `18d0aae`.
The extraction and source version are verified; correspondence to a separately hashed Maven artifact has not
been reverified here.

## Feasibility and important findings

The tree contains **3,142 Java files and 659,230 physical lines**, including comments, licence headers, generated
code and platform integrations. These are inventory measurements, not estimates of Rust lines or effort.
Sequence, class, component and modern activity are sensible early families, but even these pull substantial
shared parsing, style, rich-text and drawing behavior. A first implementation can support a documented syntax
slice; it cannot honestly claim complete PlantUML translation or full visual parity after that slice works.

There is a material provenance issue inside the MIT distribution. `gen/lib/dotgen/dotinit__c.java:86–110` says
Smetana is a translation of Graphviz and is distributed under the original C program's licence, then retains
AT&T copyright and **EPL-1.0** text. `smetana/core/Globals.java:86–108` retains the same licence declaration.
Both also have an MIT distribution header above this. The outer header is insufficient evidence to treat these
imported algorithms as MIT. IVE's `deny.toml` does not allow EPL-1.0. **Do not translate `gen/`, `h/` or
`smetana/` into shipped Rust pending resolved provenance.** This is an engineering restriction based on the
observed conflicting notices, not a legal determination. The MIT provenance of caller adapters does not grant
permission to translate an underlying EPL implementation.

Recommendation: reuse the permissive layout being implemented for Mermaid for the first graph slice, document
the geometry difference, and separately scope an original implementation from published algorithm descriptions
or another verified permissive implementation for closer dot fidelity. Do not copy Smetana's implementation
under the label of an original rewrite. Sequence and modern activity do not need dot and are less exposed to this
issue. This recommendation does not change the owner's chosen Mermaid-then-PlantUML order.

## 1. Pipeline and contracts

| Stage | Verified source | Contract and Rust implication |
|---|---|---|
| Read and split blocks | `p/SourceStringReader.java`; `p/BlockUmlBuilder.java:159` | A reader becomes `List<BlockUml>`. The builder removes comments, recognizes start/end/pause/exit directives, retains located lines and gathers include dependencies. A Rust entry point should accept one block explicitly, with a separate multi-block API if needed. |
| Preprocess | `p/BlockUml.java:212`; `p/preproc/`; `p/preproc2/`; `p/tim/TimLoader.java`; `p/tim/TContext.java` | `List<StringLocated>` plus definitions and path context becomes expanded located lines and `PreprocessingArtifact`. `BlockUml` removes YAML headers, runs `TimLoader`, then `Jaws.mutateExpands1`. Preserve source maps through expansions. |
| Detect and dispatch | `p/core/DiagramType.java:117`; `p/PSystemBuilder.java:201,288` | Start directive yields candidate diagram types; factories are tried in order until one accepts. `@startuml` does not uniquely identify sequence/class/activity. Preserve deterministic disambiguation and useful errors rather than whichever parser happens to run first. |
| Parse and construct model | `p/command/PSystemCommandFactory.java:217,290,398`; per-family `*Factory` and `command/` | Registry of single/multiline commands recognizes lines and executes model mutations, with parser passes. `CommandControl` distinguishes complete/partial/nonmatching commands. There is generally no clean, independent original AST; introduce one in Rust. |
| Resolve semantic graph and style | `net/atmp/CucaDiagram.java`; `p/abel/`; `p/cucadiagram/`; `p/skin/`; `p/style/`; `p/stereo/` | Names, aliases, entity/group membership, ordered links, stereotypes, visibility rules and style cascade become draw-ready model data. Preserve declaration order for stable output. |
| Layout sequence | `p/sequencediagram/SequenceDiagram.java:375–409`; `p/sequencediagram/teoz/` | Ordered participants/events become horizontal constraints and vertical tiles. In this distribution the SVG path directly selects `SequenceDiagramFileMakerTeoz`; assuming an older default Puma renderer would be wrong. |
| Layout UML graph families | `net/atmp/CucaDiagram.java:535–556`; `p/svek/`; `p/sdot/`; `p/dot/` | Node/label dimensions become a graph. Default JVM path uses `CucaDiagramFileMakerSvek` when dot exists, Smetana when selected or dot is absent, ELK when selected. `svek` encodes DOT and interprets Graphviz geometry; it is not itself a standalone dot algorithm. |
| Layout modern activity | `p/activitydiagram3/ActivityDiagram3.java:148`; `p/activitydiagram3/InstructionList.java:192,210`; `p/activitydiagram3/ftile/Swimlanes.java:217–229,303` | Structured instructions become Ftile/Gtile blocks and connections, composed vertically and across swimlanes. VCompact and factory delegators handle branches, repeats, loops, forks, groups and notes. This is a separate layout system from UML graph layout. |
| Measure and draw | `p/klimt/shape/TextBlock.java`; `p/klimt/font/StringBounder.java:115`; `p/klimt/drawing/UGraphic.java`; `p/core/TextBlockExporter.java`; `p/decoration/` | A `TextBlock` exposes dimensions and draws using `UGraphic`; layout and drawing both consult the string bounder. Diagram chrome adds titles, legends, headers/footers and margins. Rust should finish measurement/layout before producing an immutable scene. |
| Serialize SVG | `p/klimt/drawing/svg/UGraphicSvg.java:162–200`; `p/klimt/drawing/svg/DriverTextSvg.java:188,197`; `p/svg/` | Shape drivers write SVG through the SVG builder. Retain numeric geometry and explicit styling, not Java reflection-based driver registration. `p/teavm/DiagramRendererTeaVM.java:145–164` demonstrates a DOM-free string-producing SVG path and deterministic width-table metrics. |

`SourceStringReader.outputImage` (`p/SourceStringReader.java:248`) chooses an image index across blocks and calls
`Diagram.exportDiagram`. Page splitting, `newpage`, output chrome, preprocessor failures and unsupported types
are observable behavior; they must be either supported deliberately or rejected. They cannot silently disappear.

## 2. Measured diagram and infrastructure inventory

Method: enumerate `rg --files -g '*.java' /mnt/data/Projects/plantuml-mit/src`, read each file, and count physical
lines. Package totals include all descendant packages. Numbers below are nonoverlapping **package** measurements;
they are not standalone totals for a language. Shared packages are listed separately. Some languages share one
package; separating them by file name would undercount their real dependencies.

| Diagram family / entry package | Java files | Lines | Layout and sharing |
|---|---:|---:|---|
| Sequence — `sequencediagram` | 171 | 37,350 | Teoz constraints/tiles; `skin`, `real`, `klimt`, common commands and decoration. |
| Class — `classdiagram` | 24 | 5,800 | Shared Cuca/entity graph and svek/sdot/ELK adapters; members and relation syntax are family-specific. |
| Component, deployment, use case and description/Archimate shapes — `descdiagram` | 21 | 5,297 | One description factory and shared graph machinery; shape varieties also live in shared `svek` images and `decoration`. These are not four independent 5,297-line implementations. |
| State — `statediagram` | 13 | 3,002 | Shared graph/cluster layout; state-specific simplification and shapes in `svek`. |
| Object — `objectdiagram` | 8 | 2,083 | Shared class/entity infrastructure. No distinct object factory appears in `PSystemBuilder`; object commands participate in UML parsing. |
| Legacy activity — `activitydiagram` | 11 | 2,641 | Shared graph layout, including `CucaDiagramSimplifierActivity`. |
| Modern activity — `activitydiagram3` | 224 | 49,935 | Dedicated instruction tree, Ftile/Gtile, VCompact, swimlane and connection machinery. |
| Gantt — `gantt` | 167 | 32,635 | Task/calendar constraints, time scales and dedicated draw blocks. |
| Timing — `timingdiagram` | 66 | 13,002 | Dedicated clocks, ticks, player tracks and constraints. |
| Salt wireframe — `salt` | 62 | 10,375 | Widget/table/tile composition; shared text/shapes. |
| Network — `nwdiag` | 27 | 5,120 | Network/server grid and group layout (`GridTextBlockDecorated`), rather than dot. |
| Mind map — `mindmap` | 19 | 3,615 | Dedicated branch/finger tree layout and shared text/shapes. |
| Work breakdown structure — `wbs` | 15 | 3,169 | Dedicated tree/branch layout and shared text/shapes. |
| Chart — `chart` | 21 | 6,403 | Dedicated chart data, axes and plot drawing. |
| Packet — `packetdiag` | 13 | 2,890 | Bit ranges, rows and field blocks. |
| Wire — `wire` | 20 | 3,760 | Dedicated positioned blocks and connections. |
| JSON — `jsondiagram` | 9 | 2,041 | `SmetanaForJson` (`:145`) uses graph nodes/edges; rich JSON node text is shared drawing. |
| YAML — `yaml` | 13 | 2,666 | YAML parser/front end feeds JSON-style diagram model/layout. |
| HCL — `hcl` | 6 | 998 | HCL parser/front end feeds JSON-style model/layout. |
| Git graph — `gitlog` | 11 | 2,055 | `SmetanaForGit` (`:144`) graph layout, MagicBox labels and GitCurve drawing. |
| Files — `filesdiagram` | 6 | 935 | Dedicated file/directory tree drawing. |
| Board — `board` | 9 | 1,424 | Dedicated board/activity columns and cards. |
| EBNF — `ebnf` | 33 | 6,359 | Expression parsing and ETile railroad composition. |
| Regex diagram — `regexdiagram` | 10 | 2,065 | Regex token/shunting-yard front end reuses EBNF ETile drawing; this visualizes regex syntax and is separate from parser regex execution. |
| Chen ER — `cheneer` | 9 | 1,681 | `ChenEerDiagram` extends `AbstractEntityDiagram`; shared graph machinery. |
| BPM — `bpm` | 36 | 6,045 | Dedicated BPM branch/grid composition. |
| Flow — `flowdiagram` | 6 | 1,044 | Dedicated positioned flow blocks. |
| Ditaa — `ditaa` plus `org/stathissideris/ascii2image` | 3 + 28 | 534 + 10,436 | ASCII grid/shape recognition and AWT bitmap drawing. `PSystemDitaa.java:171–194` writes PNG, even outside the ASCII text branch; vector SVG support requires additional design. |
| Math / LaTeX — `math` | 12 | 2,964 | Equation/icon conversion, reflective TeX support, AWT/image/SVG helpers; not provided by a font measurement interface alone. |
| Raw DOT — `directdot` | 4 | 593 | External Graphviz-facing path, plus shared `dot`. Input language is out of this project's initial scope. |
| Creole standalone — `klimt/creole/legacy` | Included in `klimt` | Included | Standalone factory reuses rich-text infrastructure, not an independent chart layout. |

Additional factories in `p/PSystemBuilder.java:201–256` produce welcome/version/licence/help, colours, font lists,
emoji/OpenIconic/sprite catalogues, definitions, explanations, crash/dedication/donor/path outputs. These are
application utilities or diagnostic outputs, not missing UML languages. `DiagramType` also declares COMPOSITE,
JCCKIT, CHRONOLOGY and UNKNOWN; an enum value alone does not prove a working, independently registered renderer.
The chronology factory is commented out at `PSystemBuilder.java:240`. Do not advertise it from the enum.

| Shared infrastructure / package | Java files | Lines | Purpose |
|---|---:|---:|---|
| `tim` | 151 | 25,047 | Preprocessor expressions, functions, variables, loops, conditionals and builtins. |
| `preproc` / `preproc2` | 27 / 9 | 4,642 / 1,318 | Reader/include/definition artifacts and preprocessing helpers. |
| `command` / `regex` | 82 / 15 | 17,001 / 3,363 | Ordered command recognition, multiline handling, regex abstractions and execution. |
| `skin` / `style` / `theme` | 66 / 40 / 6 | 13,768 / 9,058 / 1,092 | Skin parameters, selectors, cascade, fonts/colours and named themes. |
| `klimt` | 396 | 77,010 | Geometry, shapes, transforms, strokes, rich text/Creole, sprites, fonts and multiple output backends. An SVG-only port does not need every backend. |
| `decoration` | 37 | 8,136 | Arrowheads, extremities and diagram decoration. |
| `abel` / `cucadiagram` | 24 / 28 | 4,812 / 5,660 | Entity/link/group models, display and shared graph semantics. `net/atmp/CucaDiagram.java` is outside these totals. |
| `svek` / `dot` / `sdot` | 171 / 22 / 10 | 35,214 / 4,145 / 2,915 | Graph shape measurement and drawing, external dot bridge, Smetana adapter. |
| `gen` / `h` / `smetana` | 83 / 101 / 23 | 40,196 / 15,312 / 6,004 | Generated Graphviz algorithms, emulated C structs/pointers and runtime. 61,512 lines total; provenance restriction above. |
| `elk` | 23 | 3,732 | Optional ELK bridge; not a permissive Rust layout implementation. |
| `real` | 16 | 2,469 | Constraint/coordinate infrastructure used by sequence. |
| `json` | 16 | 5,249 | Shared JSON values/parsing, also used by preprocessing and data diagrams. |
| `svg` | 6 | 2,713 | SVG generation helpers beyond SVG drivers in `klimt`. |
| `teavm` | 37 | 8,714 | Platform adaptations and alternate entry points; reference, not a required runtime. |

The remaining Java files include CLI/API, security/I/O, compression, image formats, platform UI and diagnostics.
They are part of the measured 659,230 lines but not all are needed in an SVG-only local engine. Resource inventory:
44 files under `themes/`, 200 under `stdlib/` (100 `.spm` files among them); sprites, emoji and OpenIconic assets
also live elsewhere. A source-Java total says nothing about the size or rights of those data files.

## 3. Platform capabilities and the headless engine boundary

Under the owner's accepted scope, other permissive Rust dependencies may supply regex, XML, layout, font
shaping, images or SVG capabilities after licence and provenance review. Prefer suitable existing libraries over
rewriting these capabilities. Keep headless crates independent of pgui and expose text measurement through an
interface; the standalone viewer supplies its platform adapter. External runtime tools and network access
remain excluded.

| Original capability and concrete source | Rust responsibility |
|---|---|
| AWT font discovery, fallback, widths, ascent/descent, glyph outlines — `p/FileFormat.java:345–378`, `p/klimt/font/UFont.java:247` | Host-provided `TextMeasurer`; adapter may use pgui text shaping. The engine receives measured run data, not an AWT-compatible font object. Capture font identity/fallback so a later renderer uses the same fonts. Deterministic table/stub mode is a separate test mode. |
| Java regex and PlantUML metasyntax — `p/regex/Pattern2.java:127–135,192–204`; `p/sequencediagram/command/CommandArrow.java:173–218` | Rebuild command syntax using a bounded lexer and recursive-descent/state-machine parsers. `%pLN`, `%s`, `%q`, `%g`, Java Unicode classes and case handling need explicit equivalents. `char::is_alphanumeric` is not an exact Java `\p{L}\p{N}` replacement. Broad Unicode parity needs reviewed tables/semantics. |
| Reflective factories, singleton registries and inheritance — `p/PSystemBuilder.java:195–204`; `p/command/PSystemCommandFactory.java` | Closed enums plus explicit ordered dispatch tables and immutable per-render options. A factory registry need not be a dynamic plugin mechanism. |
| AWT `Graphics2D`, paths, transforms, colours, dash patterns, paint — `p/klimt/` | Small owned geometry/scene model, numeric colour parser and SVG serializer written locally; retain draw order, transforms and clipping. Render to a string first, without requiring a GUI. |
| Graphviz process, DOT format, output geometry — `p/dot/ProcessRunner.java:140`, `p/svek/` | In-process bounded graph layout; shared permissive Mermaid-derived implementation for the first slice. No process command, DOT executable probe or external fallback in production. Dot-like constraints/ports/clusters/routing require separate milestones. |
| Smetana C pointer/struct emulation — `gen/`, `h/`, `smetana/` | Do not reproduce the Java pointer runtime. Current provenance prevents direct algorithm translation. An approved independent graph implementation uses index-based arenas and explicit adjacency/rank arrays. |
| Java images, ImageIO, remote/local image readers — `p/security/SImageIO.java`, `p/klimt/creole/atom/AtomImg.java`, `AtomImgSvg.java` | Caller supplies already approved bytes and dimensions by resource key. Initially reject image embedding not implemented by the engine. Decoding arbitrary formats and deriving SVG are extra capabilities; font measurement cannot do them. |
| DOM/XML SVG generation — `p/svg/`, `p/klimt/drawing/svg/` | Write an escaped, structured SVG subset using `std::fmt::Write`; text and attribute escaping are distinct. No raw user XML or HTML insertion. Validate numbers are finite and enforce output-size limits. |
| Java path, environment, clock, URL and include access — `p/tim/TContext.java:533,571,657,731`; `p/nio/`; `p/security/` | Immutable caller-owned resource map and explicit deterministic environment values. Disallow network and filesystem authority in the engine; builtins that reach them return descriptive unsupported errors. |
| Rich text/Creole, tables, sprites, icons, emoji — `p/klimt/creole/`; `p/klimt/sprite/`; `p/openiconic/`; `p/emoji/` | Parse into typed inline/block elements; run text metrics per styled run and compose line baselines, table cells and image boxes locally. Sprite decoders and icon asset import are separate bounded, notice-reviewed work. |
| Equation rendering — `p/math/LatexBuilder.java:143–162`, `p/math/ScientificEquationSafe.java` | Use a provenance-reviewed permissive math crate or a dedicated native implementation; report deferred syntax explicitly. Do not inherit Java reflection, Swing icons or JS-based conversion. |
| Dates/calendar/locales/compression/hash utilities — `gantt/time`, `code`, `zopfli`, `brotli` | Initially implement only required deterministic date/math helpers. Encoded URLs, QR, PNG and unrelated compression are outside SVG source rendering; no reason to translate them for the first slice. |

Suggested SVG subset: `<svg viewBox>`, groups, paths, lines, polylines/polygons, rounded rectangles, ellipses,
explicit fills/strokes/dashes, text/tspans, and only locally defined clipping/gradient resources when needed.
Avoid `<foreignObject>`, browser CSS layout, scripts, event attributes, remote references and externally loaded
fonts. pgui rendering compatibility must be checked in the target app; emitting valid XML alone is insufficient.
If pgui exposes no suitable SVG drawing or export surface under the chosen build features, keep SVG export in
the engine and let the preview paint the same scene with pgui primitives. Do not introduce another dependency to
bridge that gap before checking its licence, provenance and fit with the headless engine boundary.

## 4. Text, fonts and all measurement paths

Measurement is not limited to labels in diagram modules. A search for
`calculateDimension\(|getAscent\(|getDescent\(|getFontRenderContext|getFontMetrics|getStringBounds|new TextLayout|createTextLayout|createGlyphVector`
over every Java file matches **394 distinct files**. This is a lexical inventory including commented examples,
not a proof that every match is an executed metric call. It excludes pure `StringBounder` mentions. Re-run that
search against the pinned tree to enumerate every source location; the same pattern is the migration audit for
direct platform calls.

The 394-file distribution is: klimt 80, activitydiagram3 79, svek 54, decoration 22, skin 16, ebnf 13; gantt and
timingdiagram 9 each; root-level Java and salt 8 each; cucadiagram 6; chart, descdiagram, wbs and teavm 5 each;
sequence, wire and sdot 4 each; version, elk, gitlog, jsondiagram, nwdiag, packetdiag and openpdf 3 each; crash,
filesdiagram, mindmap, board, donors, svg, explain, golem, core, flowdiagram and bpm 2 each; asciiart, eggs,
annotation, definition, openiconic, posimo, swing, dedication, braille, regexdiagram, org, tikz, classdiagram,
abel and emoji 1 each. This records all packages found by the search, including non-SVG paths deliberately left
out of the port.

All observed pathways fall into the following boundaries:

| Measurement path | Source / behavior | Coverage needed |
|---|---|---|
| Normal SVG and AWT dimensions | `p/StringBounderSvg.java:132–149`; `p/StringBounderAwt.java:124–154`; `p/FileFormat.java:345–378` | SVG still calls Java font metrics. Empty text yields `(0,0)`; dimensions and descent have separate caches. SVG applies `SvgCharSizeHack` before measuring. Preserve or explicitly replace these compatibility behaviors. |
| Central interface and descent fallback | `p/klimt/font/StringBounder.java:115–128` | Width/height plus descent and output properties; default descent is font size / 4.5. A Rust interface needs actual baseline metrics, not just a scalar width. |
| Deterministic tables | `p/klimt/drawing/font/StringBounderFromWidthTable.java:140–186`; `StringBounderEmbeddedFont.java`; `EmbeddedFontMetrics.java`; `LiberationSansMetrics.java` | Unicode code-point table widths scaled from 16 px; height equals size in the width-table implementation. It does not implement full kerning, ligatures, bidi or fallback shaping, so it is not interchangeable with AWT/pgui metrics. Preserve notices if tables are copied. |
| TeaVM browser and C bridges | `p/teavm/js/StringBounderTeaVM.java:178–188`; `p/teavm/c/PngCanvas.java:272`; `p/teavm/c/demo/PngDemoC.java:160` | Platform callbacks and caches are replaced by the host measurement service. Do not import browser canvas or C runtime dependencies. |
| Rich-text layout | `p/klimt/creole/Display.java`, `SheetBlock2.java`, `SheetBuilder.java`, `atom/AtomTable.java`, `legacy/AtomText.java` | Compose measured styled runs, wrap lines and size table cells; preserve baselines and inline object dimensions. |
| Diagram geometry | sequence Teoz/skin components; activity Ftile/Gtile; graph `svek`; gantt/timing; salt; EBNF; remaining family TextBlocks | Participant heads/feet, event labels and notes, class compartments, arrow labels, package captions, swimlane titles, calendars, axes and table widgets all consume the same measured text blocks before layout. |
| Glyph outlines, not merely bounds | `p/klimt/font/UFont.java:247`; `p/klimt/shape/UText.java:169`; SVG `DriverTextAsPathSvg.java:127`, `DriverCenteredCharacterSvg.java:179` | Outlines require an additional host glyph/path capability or an explicitly deferred text-as-path feature. The measurer alone cannot satisfy them. |
| Non-SVG native/diagnostic metrics | `p/Run.java:548`; `EmptyImageBuilder.java:186–211`; `Splash.java:306`; `p/swing/FontChecker.java:188,206`; `p/openpdf/`; EPS/G2D/TikZ text drivers | Excluded output/UI code need not be translated. If watermark/chrome enters the SVG engine, route its text through the same measurement interface. |
| ASCII art image sizing | `org/stathissideris/ascii2image/graphics/FontMeasurer.java:125–169` | Independent AWT FontRenderContext, string bounds, FontMetrics and glyph-vector paths. A future Ditaa port needs the host boundary too, plus raster/vector work. |

The implementation needs one measured-run contract covering font family/fallback, size, weight, style, text,
language/direction policy; returned advance, ascent, descent, line gap and ink bounds. Cache per render with a
bounded key including all those properties and a font-database generation. Build multiline and styled blocks
from measured runs, accounting for trailing spaces, explicit newlines, links, superscripts, inline icons,
underline/strike and table alignment. Do not derive baseline from the glyph ink box.

An example conceptual boundary (API design, not product code):

```rust
trait TextMeasurer {
    fn measure(&self, request: &TextRequest<'_>) -> Result<TextMetrics, MeasureError>;
}
```

Store enum states such as `FontChoice::Default` / `FontChoice::Named(String)` and `TextDirection::Auto` rather than
optional-wrapper fields, following this repository's guideline. `Option` remains suitable for lookup return
values. The test stub returns known deterministic widths/baselines; it verifies algorithms, not real typography.

Off-UI execution requires a verified thread-safe host measurer. If pgui's font service is UI-thread-bound, use
staged measurement requests: the worker produces a deduplicated batch, the UI/host resolves it, then the worker
continues. Additional wrap/sub-label requests may require bounded rounds. Do not hold a GPUI window or entity
borrow on a background thread. Verify the concrete pgui API before choosing this adapter implementation.

## 5. Idiomatic Rust translation and runtime policy

The core pipeline should be explicit values:

```text
source + source name + options + resource values
  → expanded source with source map
  → typed syntax tree
  → resolved family model + computed styles
  → measured text / boxes
  → layout result (boxes, ports, edge paths, labels)
  → scene (ordered primitives and text runs)
  → SVG string + diagnostics
```

Keep `PlantUmlSyntax`, `DiagramModel`, `LayoutResult` and `Scene` distinct. Sequence events can be
`Message`, `Activation`, `Note`, `GroupStart`, `GroupElse`, `GroupEnd`, `Divider`, `Delay` and lifecycle events.
Activity uses a structured `Instruction` enum (`Action`, `Decision`, `Loop`, `Fork`, `Stop`, `Group` and later
swimlane routing). UML graph entities/edges live in `Vec`s addressed by `EntityId`, `EdgeId` and `GroupId`, with
declaration order retained. Share typed geometry and styling, not a universal untyped graph with string-keyed
attributes for every language.

| Java pattern with concrete example | Rust mapping |
|---|---|
| `PSystemBuilder` singleton and mutable factory list (`:195–204`) | Explicit `DiagramKind` and stable dispatch; `RenderOptions` contains theme, limits, resources and compatibility profile. No cross-render global state. |
| `CommandArrow extends SingleLineCommand2<SequenceDiagram>` and creates/adds `Message` (`CommandArrow.java:154,378,486,509`) | `parse_arrow` returns a located syntax value, then `resolve_sequence` resolves aliases and participant IDs in a private builder. Return the frozen `SequenceModel`; parsing does not draw. |
| Java object relationships `Entity` / `Link` / Cuca group references | Index-based arenas, validated IDs and parent group indices. A label is an owned styled block or an ID into an owned text arena. No `Rc<RefCell<_>>` graph by default. |
| `Swimlanes.getFtileFactory` wraps a chain of delegating factories (`:217–229`) | Recursive enum composition and pure layout helpers returning geometry plus ports; decorators such as notes/margins become explicit parameters/values, not a corresponding trait/class hierarchy. |
| `InstructionList.createFtile` accumulates and wraps tiles (`:210–237`) | Build structured instructions once, then compose immutable layout blocks bottom-up with bounded recursion/arena stacks. Internal short-lived mutable local builders are appropriate; shared mutable render state is not. |
| `StringBounderSvg` plus global font caches (`FileFormat.java:345–378`) | Host measurement trait plus bounded per-render cache; output-profile/font identity is a value. No permanently growing synchronized singleton cache. |
| `UGraphicSvg` registers drivers by shape class (`:169–200`) | Match on `Primitive` enum; apply immutable transform/style values and write SVG. Separate scene construction from low-level XML emission. |
| `PSystemRegex` mutable ETile stack and shunting yard (`:177,213`) | Preserve grammar and precedence in typed tokens/AST; use a local stack, then freeze the expression and lay out its railroad scene. |
| Java nulls, sentinel indices and exception-catching factory trials | `Result` for real errors; enums for match/no-match/partial and absent semantic states; `Option` only at return boundaries. Expected parser disambiguation is not an exception. |

Rewrite platform backends, parser-framework mechanics and singleton registries. Translate tested command
semantics, layout constraints, geometry constants, ordering and edge-case handling. SVG-only output needs neither
the CLI nor AWT raster/PDF/ASCII/XMI exporters. Preserve source provenance in Rust module documentation as
`PlantUML MIT 1.2026.9: <package/file>`, and record intentional deviations in a compatibility ledger. A different
shape for the code is idiomatic translation; omitting behaviors without a diagnostic is not.

### Bounded execution and resources

The following are proposed initial defaults, not observed PlantUML limits or measured performance guarantees.
Tune them after representative corpus benchmarks. All limits are explicit options with an engine-enforced hard
ceiling; a caller cannot turn a budget into infinity.

| Resource | Proposed initial bound |
|---|---:|
| UTF-8 input | 2 MiB and 50,000 lines |
| Preprocessed output | 8 MiB / 200,000 lines |
| Preprocessor call/include depth | 32 frames |
| Expanded resources | 128 includes, 16 MiB aggregate supplied bytes |
| Preprocessor evaluation | 1,000,000 charged operations and 100,000 loop iterations total |
| AST/model | 10,000 nodes/events, 20,000 edges, nesting depth 128 |
| Layout | 128 crossing/order sweeps and 10,000,000 charged layout operations |
| Cached measurement | 100,000 runs / 16 MiB text keys total |
| Scene/SVG | 250,000 primitives / 32 MiB serialized output |
| Geometry | finite coordinates; absolute coordinate/extent at most 1,000,000 px |

An operation budget and cooperative cancellation are required inside every potentially long loop and graph
traversal, not only between stages. Add a caller deadline as a secondary guard; a deadline cannot safely interrupt
an uncooperative native call. Bound image pixels and compressed sprite expansion before allocation. Return
`LimitExceeded { stage, resource, observed, limit, location }`, `Cancelled`, `Syntax`, `Unsupported`,
`UnknownResource`, `InvalidStyle`, `InvalidGeometry` or `TextMeasurement`, with the offending value and location.
For example: `PlantUML include at design.puml:12: resource "../secret.puml" was not supplied by the caller`.
No silent truncation or diagram-shaped success after an unsupported feature.

Resource policy: engine code has no network client, process spawning or filesystem resolver. `!include name`,
`!includesub`, themes and sprites can resolve only to canonical logical IDs in an immutable caller-supplied map.
Keep include stack/origin and cycle detection; selectors and include-once/include-many semantics need explicit
tests. Reject absolute paths, path traversal and URL schemes before lookup. `!includeurl`, URL images and external
SVG references return `Unsupported`. A caller can separately fetch/preload an approved resource under a local
logical ID, but the engine never fetches a URL. Initially absent stdlib returns `UnknownResource`; later a
notice-reviewed embedded stdlib is just another explicit resource pack. No automatic system stdlib lookup.
Environment/time/random builtins use supplied deterministic values/seed or are unsupported. File probing,
filesystem writes and shell-backed helpers stay unsupported.

### First practical PlantUML slice after Mermaid

Reuse the already tested text/scene/SVG/limits/resource foundation, then implement sequence participants,
aliases, messages, self messages, explicit activation and plain notes. Add nested `alt/else/loop` only with their
layout tests. Then class/component entities, compartments, plain associations/inheritance and simple groups use
the shared graph layout. Add modern activity start/action/stop and structured if/else, followed by loops/forks and
swimlanes. Full preprocessor compatibility, rich Creole, every arrow/shape/style and complex graph routing are
later milestones. Reject deferred syntax explicitly. The first slice is an implementation milestone, not evidence
that four entire PlantUML languages have been ported.

## 6. Fidelity, corpus, verification and notices

Use the **MIT 1.2026.9 binary** as a development/test oracle only. Do not accidentally obtain a default GPL jar
for reference generation. The sources inspected here contain no `test/`, `tests/`, `docs/` or `examples/`
directories, so an upstream test/doc corpus is not already present. Import fixtures only after locating an
approved MIT-edition corpus or checking each example's provenance; keep original notices. Start immediately
with owner-authored small fixtures covering every implemented syntax path and source errors. Bundled themes
can supply theme data after review; they are not a comprehensive diagram test suite.

Record oracle jar hash/version, Java version, Graphviz version, layout pragma, installed/pinned font hashes,
theme/options and generated SVG for each fixture. Use Java and `/usr/bin/dot` only in the developer fixture
generator. Run it with a timeout, output/memory bounds and no network; production render tests verify no external
process invocation. Permissive library dependencies are allowed in production; Java, Node and Graphviz remain
development/test tools only. Additions to reference-generation tooling must remain explicit.

| Test level | Comparison / exit condition |
|---|---|
| Syntax and preprocessing | Expanded source with source map, selected family, AST/model values and located diagnostics. Include CRLF, Unicode, escaped quotes, aliases, comments, multiline bodies and malformed/incomplete inputs. |
| Measurement | Stub tests assert exact dimensions/baselines and wrapping; host tests pin fonts and cover combining marks, emoji, CJK, Arabic/bidi, ligatures, bold/italic, trailing whitespace and missing fonts. Record known shaping differences. |
| Exact deterministic geometry | Pin metrics and algorithm/options, compare entity boxes, label baselines, edge endpoints/control points and bounds. Proposed numeric tolerance: 0.25 px for equivalent algorithms; establish the value empirically. |
| SVG structure | Normalize generated IDs, whitespace, attribute order and irrelevant metadata; compare draw order, primitive types, text content, colours/strokes, transforms and local references. Byte equality is not required. Normalization must not erase misplaced labels or missing elements. |
| Visual output | Rasterize both SVGs using the same dev renderer/fonts at 1× and 2×; retain originals, diff image and geometry report. Establish antialiasing/colour tolerance from known equivalent examples, with a separate strict check for missing text/primitives. No blanket percentage threshold hiding absent small labels. |
| Alternative graph layout | Assert semantic fidelity, no overlaps, label containment, edge/node associations, ranks/direction and determinism. Track geometry divergence from dot openly; passing these properties does not establish dot visual parity. |
| Bounds/security | Oversized input, expansion bombs, include cycles, recursion, long tokens, dense/cyclic graphs, deep groups, huge numeric styles, NaN/Inf and cancelled work return bounded errors. Instrument budgets; verify no network/file/process capability is invoked. |
| Host preview | Compare exported SVG and in-app scene/SVG preview with pgui. For later IVE integration, launch an isolated `agent-*` profile and inspect via its MCP as AGENTS.md requires, then stop it. This research task starts no app. |

Suggested initial corpus: at least one minimal success and one malformed fixture per supported construct, then
cross-feature pairs (notes + activations, aliases + self messages, nested groups + multiline labels, package
membership + inheritance, branches + styles). Add 10/100/1,000-node cases, dense edges, disconnected components
and nested groups for each relevant layout. Test order dependence intentionally; store an explicit list of
unsupported features and intentional deviations. Do not count an unsupported fixture as a successful render.

`THIRD-PARTY-NOTICES.md` and an import manifest must record the MIT PlantUML version, source artifact/hash,
copyright holder/year lines and full MIT text for translated code. Keep source notices on copied/adapted files
and data. Review separately any imported themes (the source header credits Brett Schwarz), OpenIconic,
Archimate sprites, stdlib libraries, Twemoji, font files/metric tables, Ditaa, AsciiMath and test/doc examples;
the distribution's aggregate MIT label does not settle every embedded asset's notice requirements. Until rights
are verified, exclude the asset rather than inventing a notice. Derived algorithms copied from Mermaid/dagre
need their own notices in the shared engine, as the Mermaid analysis covers. `gen/`, `h/` and `smetana/` remain
excluded under the observed EPL notice issue. Notices for translated local source must be maintained explicitly;
Cargo's dependency notice generator cannot discover their provenance automatically.

No build, runtime performance or parity result is claimed by this analysis. Those claims require the
implementation, pinned reference outputs and the exit checks above.
