# Handoff: analyse PlantUML and Mermaid and plan their idiomatic Rust translation

For: Codex. From: a brainstorming session with the IVE owner, 2026-10-22.

This task is **analysis and planning only**. Do not write port code yet. The deliverables are written documents
(listed under "What to produce"). The owner reviews them before any porting starts.

## Why

IVE (this repo, Rust, Zed's gpui) is getting a rendered view for markdown files: a toggle between text and
rendered view, with images and diagrams. The trigger is reading design docs, such as a security feature's
documentation with large diagrams. Large diagrams will be shown on a pan-and-zoom canvas (like Google Maps).
The markdown view and the canvas are separate projects that come later. This one covers only the diagram engines.

The owner wants as many diagram languages as possible, and **everything built in**:

- no network: diagram source must never leave the machine (no Kroki, no PlantUML server);
- no external tools at runtime: no JVM, no Node or browser, no `plantuml.jar`, no `mmdc`, no Graphviz `dot`;
- where no usable Rust library exists, **translate the original into idiomatic Rust**.

PlantUML and Mermaid come first.

### Translate into Rust idioms, not line by line

The original code is translated: its logic, algorithms, constants and edge-case handling carry over, so the Rust
engine behaves like the original and its output matches. Its **shape does not carry over**. The Rust code is written
the way a Rust programmer would write it:

- Java class hierarchies and JS duck typing become enums with pattern matching, or traits only where open
  extension is really needed;
- mutable objects that build themselves up step by step become plain data passed through pure functions (source →
  AST → model → layout → SVG), with builders where construction needs many inputs;
- shared references and object graphs become owned data, borrows, or index-based arenas (graphs as `Vec` plus ids);
- nulls and sentinel values become `Option`, exceptions become `Result` with descriptive errors;
- global config, singletons and static registries become values passed in explicitly;
- d3's DOM chaining becomes building an SVG tree directly;
- hand-rolled utilities become crates where good ones exist.

Keep a trace back to the original: each Rust module names the Java package or TS module it comes from, so a
behaviour difference can be checked against the source.

## Sources (already checked out)

| What | Where | Version | Licence |
|---|---|---|---|
| PlantUML, MIT edition (sources jar from Maven Central) | `/mnt/data/Projects/plantuml-mit/src` | 1.2026.9 (2026-10-17) | MIT |
| Mermaid monorepo | `/mnt/data/Projects/mermaid` | 12.2.0, commit `6d8921f` (shallow) | MIT |
| dagre-d3-es, the dagre/graphlib fork Mermaid uses for layout | `/mnt/data/Projects/dagre-es` | 7.0.14, commit `4b95ad0` | MIT |

Sizes measured so far. PlantUML has about 3,100 Java files and 660k lines. The biggest packages are `klimt` (77k,
drawing primitives and output), `activitydiagram3` (50k), `sequencediagram` (37k), `svek` (35k, the bridge to
Graphviz layout), `gantt` (33k) and `tim` (25k, the preprocessor). It also carries **smetana**, a Java port of
Graphviz's `dot` layout (`gen/` 40k, `h/` 15k, `smetana/` 6k). Together with Graphviz's papers and docs, it is a
readable reference for `dot` layout. Mermaid's `packages/` hold about 120k lines of TS/JS excluding tests, across 30+
diagram types in `packages/mermaid/src/diagrams`. Its layout relies on dagre-d3-es, plus optional ELK
(`mermaid-layout-elk`, elkjs) and cytoscape (cose-bilkent, fcose) for some diagram types. It uses d3 for SVG
building, khroma for colours, KaTeX for math and roughjs for the hand-drawn look. Text sizes come from the browser
(`getBBox` / `getComputedTextLength` in about 38 files).

### Licence rule (hard)

IVE ships closed source. `deny.toml` allows only permissive licences (MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0 …).

A translation is a derived work, so the source licence decides what can be translated:

- Translate PlantUML **only from the MIT sources above**. Do not read or translate the GPL repo
  (github.com/plantuml/plantuml) or the GPL/LGPL/EPL editions; a translation of them could not ship.
- Mermaid and dagre-es are MIT and can be translated. elkjs is EPL-2.0: do not translate it; plan either a
  permissive alternative or a fresh implementation from ELK's published algorithm descriptions. Check every other
  dependency you plan to translate (KaTeX MIT, cytoscape MIT, d3 ISC, roughjs MIT).
- Any Rust crate you propose must pass `deny.toml`.
- The translated crates keep the original copyright and MIT notices, and so does any data taken over (themes,
  colour palettes, the PlantUML stdlib and sprites, test corpora). List what THIRD-PARTY-NOTICES.md will need.

## Constraints from IVE

- Read `AGENTS.md` (coding guideline) and `README.md`. The ports must follow the guideline: immutable state,
  small functions, no boolean parameters, `should_…` tests in given/when/then sections, self-contained error
  messages.
- Engines are Rust crates with no gpui dependency, so they can be tested alone. `codeview/` is the precedent for a
  workspace crate.
- Output is **SVG** (a string or a small owned tree). gpui already draws SVG (resvg/usvg). Plan for what
  resvg supports; avoid relying on CSS or `<foreignObject>` HTML that it cannot render.
- Text measurement must not need a browser. Propose a measuring interface the engine calls (font, size, text →
  width/height), so IVE can supply real font metrics through gpui and tests can supply a deterministic stub.
- Rendering runs off the UI thread and must be bounded. Huge or hostile input must fail with an error, not
  hang or exhaust memory (limits on input size, node count, layout iterations, preprocessor recursion and loops).
- No network, no filesystem access beyond what the caller passes in. PlantUML's `!include`, `!includeurl`, sprites
  from URLs and the stdlib need an explicit decision: what is supported, and how the caller provides it.

## What to analyse

For **each** of PlantUML and Mermaid:

1. **Pipeline.** Source text → preprocessing → parsing → model → layout → drawing → SVG. Name the packages or modules
   for each stage and the interfaces between them.
2. **Inventory.** Each diagram type with its code size, its layout engine, and how much it shares with others.
   Also list the shared infrastructure every type needs (PlantUML: `tim` preprocessor, `command` parsing, `skin` and
   `style` theming, `klimt` drawing, `svek` + smetana layout. Mermaid: the parsers (jison grammars and langium in
   `packages/parser`), config and theming, `rendering-util`, dagre layout).
3. **Capabilities the Rust engine needs.** What the original gets from its platform and libraries (Java AWT fonts
   and images, regex dialects, d3 selections, DOM APIs, graph libraries). For each, name the Rust crate or the part
   IVE writes itself. This is about capabilities, not API-for-API replacements.
4. **Text and fonts.** Every place a size is measured, and how the measuring interface covers it.
5. **Translation design.** Per layer, how the original maps onto idiomatic Rust: the core types (AST, model,
   layout result), the stages as functions between them, the errors, and the crates used. For the main Java/TS
   patterns found in each layer (class hierarchies, mutable builders, visitors, global config, d3 chaining), give
   the Rust form they become, with one concrete example from the source. Call out layers that should be rewritten
   rather than translated, because a crate or a cleaner Rust design does the job better. Candidates are parsers
   generated by jison or langium, which are better rebuilt from their grammar, and smetana's generated C-style Java.
6. **Fidelity.** How the Rust engine's output is compared with the original's. The original tools may be used at **dev
   and test time only**: `java` and `node` are installed, `pnpm` is not, and Graphviz `dot` is at `/usr/bin/dot`.
   Propose a golden-test corpus (Mermaid's and PlantUML's own test and doc examples) and how SVGs are compared
   (structure and geometry within tolerance, or rasterise and diff).

Also:

7. **Existing Rust work.** Search the web for Rust ports or partial ports of Mermaid, PlantUML, dagre, Graphviz
   `dot` layout and ELK. For each, record coverage, maintenance and licence, and whether IVE should use it, fork it,
   or ignore it. Prefer building on a good existing crate over a fresh translation.
8. **Alternatives to a full port, for comparison only.** PlantUML ships a TeaVM target
   (`net/sourceforge/plantuml/teavm`) that compiles it to JS/wasm. `/mnt/data/Projects/mquickjs` is an embeddable
   JS engine the owner has looked at. Estimate briefly what embedding these would cost (binary size, speed, the
   DOM Mermaid needs) so the owner can see the port's value against them. The owner's direction is the Rust
   port, so don't recommend switching without strong evidence.

## What to produce

Write to `docs/superpowers/plans/` in this repo:

1. `2026-10-22-plantuml-port-analysis.md`: findings for items 1–6 for PlantUML.
2. `2026-10-22-mermaid-port-analysis.md`: the same for Mermaid (include dagre-es).
3. `2026-10-22-diagram-ports-plan.md`: the combined plan:
   - the survey (item 7) and the alternatives estimate (item 8);
   - crate layout (e.g. shared `diagram-svg` / text measuring / layout crates, then `plantuml` and `mermaid`), and
     the public API, e.g. `render(source, &Options) -> Result<Svg, RenderError>`;
   - the first milestone: the smallest end-to-end slice that produces a correct SVG for the most-used diagram types.
     Propose which ones; a likely start is sequence, class, component and activity for PlantUML, and flowchart,
     sequence and class for Mermaid. Then later milestones in order, each with its diagram types, what it reuses,
     a size estimate and its exit test;
   - risks, open questions for the owner, and the decisions you made yourself, marked as yours.

Keep the documents factual: file paths, line counts, names of classes and functions. Where you estimate, say so.

## Out of scope here

- The markdown rendered view, the text/render toggle, images, and the pan-and-zoom canvas (separate projects).
- Other diagram languages (Graphviz DOT input, D2, Excalidraw, BPMN …). You may note where the PlantUML/Mermaid
  work makes one of them cheap, for example DOT input once the `dot`-style layout exists.
