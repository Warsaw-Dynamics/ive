# Original-renderer development oracles

Prepared 2026-10-22. Development references only; no IVE/standalone product files changed, no commits made, and no original JavaScript/Java/Graphviz runtime introduced into the native engine.

Harness and all inputs/SVGs/measurements are in `/tmp/ive-diagram-oracles.JwN5H5/`. The directory is temporary evidence and is not a portable dependency installation. Keep `package-lock.json` if reproducing elsewhere.

## Tools and provenance

| Tool | Observed version |
|---|---|
| Node | v22.14.0 |
| Mermaid | 12.2.0, installed npm package |
| puppeteer-core | 25.16.0 |
| Headless Chromium | Chrome/153.0.8010.12 |
| Java | OpenJDK 26.0.2+10-55, 2026-07-21 |
| PlantUML | 1.2026.9 / 18d0aae, 2026-10-17, explicitly reports `(MIT source distribution)` |
| Graphviz dot | 2.43.0 (0) |

Chromium executable: `/home/kb/.cache/ms-playwright/chromium-1243/chrome-linux64/chrome`.

MIT PlantUML jar: `/tmp/ive-diagram-oracles.JwN5H5/plantuml-mit-1.2026.9.jar`.

Verified jar SHA256: `e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd`.

Only that MIT distribution was run; no GPL PlantUML implementation was read. Java and dot were used solely to produce development reference SVGs.

## Rendering configuration and fonts

`oracles.mjs` loads the local `node_modules/mermaid/dist/mermaid.min.js` bundle into a headless page. Both render and inspection pages have offline mode enabled. No HTTP server is needed. The viewport is 1600 × 1200 at device scale 1. Each browser is closed in `finally`, including error paths. Headless launch uses `--no-sandbox --disable-dev-shm-usage --disable-background-networking`; no GUI is activated.

Mermaid settings:

```json
{
  "startOnLoad": false,
  "securityLevel": "strict",
  "theme": "default",
  "deterministicIds": true,
  "deterministicIDSeed": "ive-diagram-oracle-v1",
  "look": "classic",
  "handDrawnSeed": 42,
  "fontFamily": "Liberation Sans",
  "layout": "dagre",
  "htmlLabels": false,
  "flowchart": {
    "defaultRenderer": "dagre-wrapper",
    "htmlLabels": false,
    "useMaxWidth": false
  },
  "sequence": {
    "useMaxWidth": false,
    "actorFontFamily": "Liberation Sans",
    "noteFontFamily": "Liberation Sans",
    "messageFontFamily": "Liberation Sans"
  },
  "class": { "useMaxWidth": false, "htmlLabels": false },
  "themeVariables": { "fontFamily": "Liberation Sans" }
}
```

Every representative PlantUML input sets `skinparam defaultFontName Liberation Sans`. Java receives UTF-8 source on stdin and returns SVG on stdout; the harness bounds each Java invocation to 30 seconds and 8 MiB of output. The command is:

```sh
java -Djava.awt.headless=true -DPLANTUML_SECURITY_PROFILE=SANDBOX \
  -jar plantuml-mit-1.2026.9.jar -charset UTF-8 -tsvg -pipe
```

The actual browser paint fonts were obtained through Chromium's `CSS.getPlatformFontsForNode`, not inferred from CSS. They are Liberation Sans (regular and bold as needed), plus Noto Sans CJK JP for `世界` in the Unicode fixture. Font files:

- `/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf`
- `/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf`
- `/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc`

`FontProbe.java` confirms that Java resolves the explicit `Liberation Sans` request to that family. This explicit family matters: `fc-match Arial` and Chromium map Arial to Liberation Sans, whereas Java's `new Font("Arial", ...)` resolves to `Dialog` on this host. The final references avoid that font-metric mismatch by using Liberation Sans in both original renderers. This does not establish that a future native engine uses identical shaping or metrics.

## Reproduce and inspect

```sh
cd /tmp/ive-diagram-oracles.JwN5H5
node oracles.mjs
node oracles.mjs --verify-repeat
node verify-oracles.mjs
java -Djava.awt.headless=true FontProbe.java
```

The first command writes eight input files (`.mmd` or `.puml`), eight original `.svg` files, per-SVG `.json` measurements, and `manifest.json`. The second independently starts the original renderers again and compares complete SVG bytes to the previous output, writing `repeatability.json`; a difference produces a nonzero exit. The third checks corpus properties and exercises positive and negative comparison/error paths, writing `verification.json`.

Individual sources can be rendered without changing the harness:

```sh
node oracles.mjs --mermaid input.mmd output.svg
node oracles.mjs --plantuml input.puml output.svg
node oracles.mjs --inspect output.svg
node oracles.mjs --compare original.svg native.svg
```

`ORACLE_CHROMIUM` can override the executable path. The script location anchors the local npm bundle and jar paths, so commands can also be invoked by absolute script path from another folder.

## Representative outputs

All eight sources rendered successfully, passed XML parsing and expected-label checks, and had nonempty geometry. Every final reference has zero `foreignObject` elements and zero external href references. They reproduced byte for byte across two independent final-settings runs.

| SVG stem | Original viewBox | Text / geometry elements |
|---|---|---|
| `mermaid-flowchart` | `0 0 196 501` | 6 / 21 |
| `mermaid-sequence` | `-50 -10 450 306` | 7 / 10 |
| `mermaid-class` | `0 0 162.3828125 449` | 9 / 24 |
| `mermaid-nested-unicode` | `0 0 986 173` | 6 / 22 |
| `plantuml-sequence` | `0 0 123 194` | 7 / 15 |
| `plantuml-class` | `0 0 131 351` | 9 / 22 |
| `plantuml-component` | `0 0 96 321` | 5 / 14 |
| `plantuml-activity` | `0 0 169 268` | 6 / 26 |

`manifest.json` records exact final output SHA256 values, source/runtime configuration and paint fonts. The `.json` beside each SVG records labels, text coordinates/font CSS, element attributes and geometry boxes in root SVG user coordinates, original width/height/viewBox and the final SVG SHA256.

## Exact matching standalone fixtures

The native fixture directory appeared during this task, so additional originals were rendered from its exact flowchart, sequence and class inputs; the original source bytes are copied into the oracle directory as `mermaid-matched-*.mmd` without changing the native fixture files.

```sh
node oracles.mjs --matched-fixtures \
  /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid
```

All three passed XML and expected-label checks. See `matched-fixtures.json` and:

| SVG stem | Original viewBox |
|---|---|
| `mermaid-matched-flowchart` | `0 0 743.8046875 253` |
| `mermaid-matched-sequence` | `-50 -10 501 303` |
| `mermaid-matched-class` | `0 0 188.80078125 312` |

Use these source copies for exact later native comparisons, or verify native fixture source hashes still match before comparing newly generated output.

## How to compare the native CLI later

Render one saved oracle input through the native CLI, with its font settings recorded, into a separate `native-*.svg` path in this temporary directory. The native CLI invocation must follow its actual completed interface; it was not yet available for this task. Then run, for example:

```sh
node oracles.mjs --compare \
  mermaid-matched-flowchart.svg native-matched-flowchart.svg
node oracles.mjs --inspect native-matched-flowchart.svg
```

The comparison returns both original width/height/viewBox values, signed content-bounds deltas, missing/extra unique text labels, both full text/position/font lists, and geometry counts. Inspect the individual measurements for shape attributes and path coordinates. For meaningful geometry comparison, match node/edge semantics first rather than assuming DOM element order or SVG IDs correspond. Multiline/compartment labels may be split differently by two renderers, so label-set differences require interpretation alongside full text lists.

Measurements are root user coordinates, with transforms applied. The root content box uses `getBBox()`; it excludes stroke extents and does not constitute a complete painted-pixel bounds calculation. Marker definitions are excluded from geometry counts. Responsive SVGs are inspected at their own viewBox size. Class originals use `width="100%"` and no explicit height, so compare viewBox/content dimensions rather than interpreting that attribute as a numeric width. Sequence originals have a negative viewBox origin.

SVG validity, family admission, expected labels and nonempty geometry are separate from fidelity. No native SVG or pixel comparison was run in this task, and no upstream geometry or pixel-parity claim is made. A later pixel comparison should rasterize both SVGs at the same viewBox, scale, font set, background and rasterizer before comparing output images.

## Verification and resolved errors

`verification.json` records eight valid references; positive geometry; zero foreign objects/external href references; eight byte-identical repeat runs; zero deltas for a self-comparison; detection of an intentionally changed label; and rejection of malformed SVG XML, malformed Mermaid and malformed PlantUML. Error-path inputs remain in the temporary directory for reproduction. No invalid output SVG was accepted.

Two setup issues were found and resolved:

1. The normal command sandbox failed before executing commands with `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`. Read-only inspection and temporary-directory rendering were rerun through approved escalated execution.
2. Mermaid's `deterministicIds` alone did not produce repeatable SVG bytes: classic flowchart/class paths contained randomized collinear Bezier control points. Pinning `handDrawnSeed: 42` fixed this; all final eight outputs repeated exactly. Font mapping was also made explicit as described above.

Each browser is closed on successful and failed operations; Java subprocesses finish synchronously or are timed out. No web server, IVE instance, GUI automation, or persistent development process was started.

## Extended unchanged native Mermaid corpus admission

The original renderer was subsequently run against all **37** native Mermaid fixtures. **36 rendered successfully; 1 failed type detection.** All 36 successful outputs passed Mermaid parsing and SVG XML parsing and had positive content bounds. This is admission evidence for these exact fixtures, not proof of full family syntax, semantics, native geometry fidelity, or pixel parity.

Reproduce the scan and verify source/output integrity:

```sh
cd /tmp/ive-diagram-oracles.JwN5H5
node oracles.mjs --corpus \
  /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid
node verify-corpus.mjs
```

The scan preserves the existing representative `manifest.json`, `repeatability.json`, basic verification and exact three-fixture references. New files are under `corpus-mermaid/`:

- `admission.json`: versions/configuration, complete built-in registration metadata, per-fixture detection/parse/render status, source SHA256/size, output SVG SHA256, dimensions, text/geometry counts, fonts, foreign objects and external hrefs.
- Every unchanged `.mmd` input, copied as bytes; `verify-corpus.mjs` confirms all 37 match the current native fixture files and recorded SHA256 values.
- One original `.svg` and detailed measurement `.json` per successful fixture, with all 36 output hashes independently verified.
- `zenuml.error.txt`: the original failed detection diagnostic. No SVG exists for that failure because type detection failed before rendering.
- `verification.json`: concise integrity and admission summary.

The corpus scan exits zero when it finishes recording all cases; individual unsupported types remain explicit failures in its result data. It does not install or register any optional diagram plugins, rewrite source, or force failing input to pass.

### Core versus optional registration

Every successful fixture matched a type already registered by the installed Mermaid 12.2.0 bundle. Its metadata includes the newer agentflow, architecture, cynefin, eventmodeling, railroad variants, swimlane, treeView, usecase, venn and wardley types; these are present in this released bundle and were not supplied by additional external plugins in the harness.

The only failure was `zenuml.mmd`: `UnknownDiagramError: No diagram type detected matching given configuration`. The bundle has no registered `zenuml` detector. Mermaid's [official ZenUML integration instructions](https://mermaid.js.org/syntax/zenuml.html#integrating-with-your-librarywebsite) show importing `@mermaid-js/mermaid-zenuml` separately and registering it with `mermaid.registerExternalDiagrams`. That plugin was intentionally not installed, registered, or tested; whether its parser accepts this exact fixture remains untested.

| Result | Fixtures |
|---|---|
| Rendered (36) | agentflow, architecture, block, c4, class, cynefin, er, eventmodeling, flowchart, gantt, git, info, ishikawa, journey, kanban, mindmap, packet, pie, quadrant, radar, railroad-abnf, railroad-ebnf, railroad-peg, railroad, requirement, sankey, sequence, state, swimlane, timeline, tree, treemap, usecase, venn, wardley, xychart |
| Unregistered optional type (1) | zenuml |

### Additional original-output limitations exposed by the corpus

The same pinned configuration and fonts were requested; no family-specific workaround was added. The actual outputs nevertheless show:

- `eventmodeling.mmd` emits **3** `foreignObject` elements and `journey.mmd` emits **4**, despite `htmlLabels: false`. These original reference SVGs need not be compatible with a `resvg/usvg` rendering path; their admission must not be confused with native runtime compatibility. The original output is retained unchanged as evidence.
- Wardley paints some text in **Liberation Serif**, in addition to Liberation Sans. That actual font is recorded in its measurement and admission entries; the global font setting does not make every upstream family use one font.
- `info.mmd` is intentionally text-only (`v12.2.0`): it has a positive content box but no counted shape geometry. It is not a blank output.
- All 36 successful original SVGs have **zero external href references**. This check is separate from the broader SVG resource-safety requirements of the native engine.

The broad corpus was not rerun for byte repeatability; only the eight representative references have that verified property. Family-specific layouts can have their own internal algorithms even though the harness explicitly requests global `layout: dagre`; the registration metadata also lists `flowchart-elk`, but the compared flowchart source was detected as `flowchart-v2` under the pinned Dagre configuration. No native or pixel comparison was performed in this extension; the subsequent native comparison is documented below.

## Independent public native API comparison

The native engine subsequently rendered all **37/37** unchanged inputs successfully through its public `render(&source, &RenderOptions::default())` API. All 37 returned valid XML, had positive SVG content bounds, and had zero `foreignObject` elements or external href references. Native outputs and exact copies of the input bytes are under `native-mermaid/`.

### Baseline and reproducible diagnostic host

The audit host is an independent temporary Rust crate at `audit-native/`, with a path dependency on the standalone `diagram-engine`; it does not patch the product. It selects fixture files as a diagnostic host, reads each source, passes the source text to the public API with default options, records diagnostics/measurement provenance and elapsed time, and saves the returned SVG. The engine remains responsible only for caller-owned text, as its API specifies.

Baseline commit: **`29353b5267431c112366bfc8e62aeb8fdf164aff`**. The native backend reports Merman 0.9.2 with declared Mermaid 12.1.0 compatibility; original references are Mermaid 12.2.0. `native-build-evidence.json` records the audit executable/host/lockfile hashes, Rust/Cargo versions, output manifest modification times, initial/current dirty status and engine source hashes. The engine source hashes were captured after native rendering while the engine source was still clean and independently match committed baseline content:

| Source under `diagram-ports/engine/src/` | SHA256 |
|---|---|
| `lib.rs` | `a6b68b7b53a1716e8340f0504ca59e64cf6d3d3360e1bdbf67152424403a85f3` |
| `limits.rs` | `6bc181fe6791d000095ad78deda339765758b9a7e1bcbe8bc73710206a678375` |
| `mermaid.rs` | `2c880f42591e1be91eaeedf2c2dd22ae56de0636a8b23647f21429d24a4ea4c0` |
| `svg.rs` | `b8529073fabf67091b98d27506793da729a15a93c700c09de0470a77734a51d3` |

The linked diagnostic executable has SHA256 `7f82ac327ca5ad037c3395230c09e7e4a9dba14ba5ca0363bd0fc4781bdb0cc1`, modified `2026-10-22T21:41:11.205Z`. Initial dirty status contained only the unrelated PlantUML analysis plan. During the later evidence snapshot, another agent had started modifying engine tests for a resource-guard fix; those later edits are outside this output baseline. Do not attribute these SVGs to an unverified later build merely because the dependency path remains the same.

```sh
cd /tmp/ive-diagram-oracles.JwN5H5
CARGO_TARGET_DIR=/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/target \
  cargo run --offline --manifest-path audit-native/Cargo.toml -- \
  /mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid \
  /tmp/ive-diagram-oracles.JwN5H5/native-mermaid \
  29353b5267431c112366bfc8e62aeb8fdf164aff
node oracles.mjs --compare-corpus /tmp/ive-diagram-oracles.JwN5H5/native-mermaid
node verify-native.mjs
node record-build-evidence.mjs
```

The last command checks the recorded audited source hashes against the specified commit and records any current-tree differences; it does not establish that a future rebuilt binary still uses that commit. Restore/select the intended engine baseline independently before reproducing a committed-baseline build.

### Quantified results and comparison method

`native-comparison.json` contains all per-file results; `native-verification.json` independently verifies input/output hashes and summarizes the measurements. `native-mermaid/native-admission.json` records public-API diagnostics and text measurement reports. The native policy is the named deterministic approximate profile, not a real host font-measurement callback.

The paired scan reuses **one** offline browser for all 37 native SVGs and the 36 available original SVGs. It pairs unique labels by normalized text, never by DOM element order. It includes visible foreign-object labels, follows single-label local `<use>` references, excludes definition/hidden fallback text, and inserts spaces between wrapped text spans. This avoids falsely reporting lost text when SVG lowering changes its representation. Full text positions, font CSS, actual platform fonts, content bounds, viewBoxes and shape counts remain available for inspection.

| Check | Result |
|---|---|
| Native API admission / valid XML / positive content box | 37 / 37 |
| Exact source-byte matches to original corpus | 37 / 37 |
| Available original/native pairs | 36 |
| Exact normalized painted-label sets | 34 / 36 |
| Exact word-token multisets | 34 / 36 |
| All four content-box coordinates within 1 SVG unit | 14 / 36 |
| All four explicit viewBox coordinates within 1 SVG unit | 13 / 35; original info has no explicit viewBox |
| Equal counted shape-element totals | 23 / 36 |

The two label-set differences are generated data, not missing fixture source labels: Git graph generates `0-07bc4fd` / `1-08ea043` upstream versus `0-93ba65d` / `1-f55760d` natively, and info displays `v12.2.0` versus `v12.1.0`. All source-authored labels present in these 36 originals were retained by the measured native outputs. This limited corpus result does not prove broader semantic or relationship correctness. ZenUML rendered natively but has no original reference in this setup because its optional upstream plugin was not registered.

Shape counts are structural evidence only: a native single path can replace several upstream fill/stroke paths. Unequal totals alone do not establish a missing node or edge, and equal totals do not establish matching geometry.

### Concrete geometry/style differences

The following outputs preserve their label content but visibly encode different sizing, wrapping or typography. Dimensions below are SVG user units; values are rounded. The report identifies observed output differences without assigning an unverified internal cause.

| Source fixture | Original viewBox size → native viewBox size | Content-box difference / concrete example |
|---|---|---|
| [class.mmd](/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid/class.mmd:2) | `188.8008 × 312` → `174.3144 × 315.6` | Content width `172.8008` → `158.3144` (−14.4864), height `296` → `299.6` (+3.6). Actual browser paint family is Liberation Sans on both sides. |
| [eventmodeling.mmd](/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid/eventmodeling.mmd:2) | `693 × 470` → `655 × 470` | Content width `633` → `595` (−38). Card boxes for CartUI/AddItem/ItemAdded are `129/145/164` wide upstream versus `122/132/146` native; later cards move left by 7 and 20 units. Original foreign objects are safely lowered to referenced SVG labels natively. |
| [swimlane.mmd](/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid/swimlane.mmd:3) | `986 × 150.6` → `986 × 133.6` | `Browse catalogue` wraps onto two lines upstream (text height ≈34.6007) but stays on one native line (≈17.0008). Overall content height decreases by 17 units. This is a wrapping difference, not missing words. |
| [usecase.mmd](/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid/usecase.mmd:5) | `392.5234 × 167` → `376 × 163.2` | `Place order` uses 16px Liberation Sans upstream versus 12px Open Sans/sans-serif CSS natively (both resolve to Liberation Sans here). Its left edge shifts about −105.9242 units; its painted text width falls from ≈82.4192 to ≈61.3051. The actor label also shifts ≈−30.0348 units. |
| [tree.mmd](/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid/tree.mmd:2) | `134.2969 × 135.0781` → `122.08 × 138` | Original labels paint in Liberation Sans; native labels inherit Times New Roman CSS and paint in Liberation Serif. `package.json` text width is ≈94.2969 upstream versus ≈83.0061 native. This font/style difference remains with an explicit font init directive. |
| [flowchart.mmd](/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid/flowchart.mmd:2) | `743.8047 × 253` → `737.92 × 253.6` | Content width decreases by ≈5.8847 and height grows by ≈0.6. This is a smaller measured sizing difference, with the same actual Liberation Sans paint family. |

These are concrete fidelity gaps in the tested output/configuration, not evidence that the native adapter implements only a small subset: admission and label preservation are substantially broader than geometric equality. Host metrics may help measurement-driven sizing/wrapping, but this audit does not establish that a callback alone fixes family-specific CSS, source configuration handling or version differences.

### Supplementary explicit-font study

`prepare-font-study.mjs` creates **six separate derived inputs** by prefixing class, eventmodeling, flowchart, swimlane, tree and usecase with exactly:

```text
%%{init: {"fontFamily":"Liberation Sans","themeVariables":{"fontFamily":"Liberation Sans"}}}%%
```

The original fixture bytes remain unchanged after that prefix, and both renderers receive identical derived input. The unchanged baseline corpus and its SVGs are preserved. Run:

```sh
node prepare-font-study.mjs
CARGO_TARGET_DIR=/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/target \
  cargo run --offline --manifest-path audit-native/Cargo.toml -- \
  /tmp/ive-diagram-oracles.JwN5H5/font-study/inputs \
  /tmp/ive-diagram-oracles.JwN5H5/font-study/native \
  29353b5267431c112366bfc8e62aeb8fdf164aff
node oracles.mjs --font-study /tmp/ive-diagram-oracles.JwN5H5/font-study/native
node verify-native.mjs
```

All six rendered in both engines. **None of the six measured native content-box deltas improved:** all four coordinate deltas stayed identical to the baseline within 0.001 unit. The class, card, flowchart, swimlane and usecase differences remain even though the ordinary actual paint family already matches. Tree still paints in Liberation Serif after the explicit Liberation Sans directive. `font-study-comparison.json` stores detailed paired results; `font-study/inputs.json` records the source transformation and before/after SHA256 values. This study changes only the requested font configuration, not the native deterministic measurement policy.

### Environmental/measurement limits that must not be mistaken for bugs

- [Gantt's dated fixture](/mnt/data/Projects/ive/.worktree/diagram-ports/diagram-ports/fixtures/mermaid/gantt.mmd:5) is container/clock dependent. The browser original uses a `1584 × 148` viewBox while the headless native default uses `800 × 148`; those are different host widths. Its today marker is at x=`134238` upstream and x=`−208848` native, outside either viewBox. That marker dominates raw `getBBox()` width. Do not interpret the resulting huge content-box delta as a chart-layout defect or compare it with font-only deltas. Container width and current-date policy were not matched in this baseline.
- Content boxes exclude stroke extents and may include geometry outside the viewBox. Foreign-object/reference boxes are not always the same as text ink bounds. Text-based placement pairs describe the returned SVG representation, not pixel equality.
- Merman declares Mermaid 12.1.0 compatibility; these original references are 12.2.0. The audit measures the gap but does not distinguish every intentional version change from a translation defect.
- No PNGs or other raster outputs were compared. No pixel-parity claim is made, and equal bounds/labels/counts are insufficient for one.

All audit browsers were closed, synchronous Rust/Java processes finished, and no server or GUI instance was started. The only writes outside the temporary oracle tree were Cargo's explicitly authorized reusable target cache. No engine/backend/product source changes or commits were made by this audit.
