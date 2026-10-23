# Independent MIT PlantUML native-port references

Prepared 2026-10-23 Europe/Warsaw; final coherent original run recorded at `2026-10-22T22:34:59.308Z`.

Thirty small, freshly authored source fixtures were tested against the verified MIT PlantUML jar. **28 produced meaningful original diagrams with all declared painted labels; two produced degraded formula text fallbacks.** All 30 outputs parse as SVG XML and have positive finite content bounds. Those structural properties alone are deliberately not counted as formula support.

Artifacts are isolated at `/tmp/ive-plantuml-native-oracles/`. No historical Mermaid/oracle corpus was overwritten. No product source, dependency, implementation, or commit was changed, and no candidate implementation or candidate examples were read.

## Provenance and tools

Only the authorized grammar tree `/mnt/data/Projects/plantuml-mit/src/net/sourceforge/plantuml/` was consulted. The fixtures use independently chosen domain labels and small structures rather than copied examples. Every result records the exact authorized grammar paths and file SHA256 values that guided its syntax; `verify.mjs` independently rechecks those hashes. No implementation code was imported into a native backend or product.

| Tool | Observed version |
|---|---|
| PlantUML original | 1.2026.9 / 18d0aae, explicitly reports `(MIT source distribution)` |
| Original build date | 2026-10-17 08:34:41 UTC |
| Java | OpenJDK 26.0.2+10-55 |
| Graphviz dot | 2.43.0 (0) |
| Node | v22.14.0 |
| Headless Chromium | Chrome/153.0.8010.12 |

Jar: `/tmp/ive-diagram-oracles.JwN5H5/plantuml-mit-1.2026.9.jar`; SHA256 **`e84e45b37ad3030cf802e7be18a687d209eca55733a5679b1f1f1a515f7f70dd`**. The harness checks this hash before running. The authorized extracted source tree has no Git checkout metadata; the jar supplies the reported revision, while per-file hashes identify the grammar material actually read.

Browser executable: `/home/kb/.cache/ms-playwright/chromium-1243/chrome-linux64/chrome`. The new harness reads the already-installed Puppeteer 25.16.0 package from the historical temporary dependency directory without modifying it.

## Reproduce and inspect

```sh
cd /tmp/ive-plantuml-native-oracles
node render.mjs
node verify.mjs
# Optional exact-name or family selection; writes a separate selected manifest:
node render.mjs sequence class state activity
node render.mjs flow math latex
```

`fixtures.mjs` and `extended-fixtures.mjs` are the authored source catalog and expected-label contracts. `render.mjs` writes one `.puml` input, one original `.svg`, one detailed measurement `.json`, and one `.result.json` per case. `manifest.json` is the coherent 30-case run; `core-manifest.json` preserves the early 16-case handoff. `verification.json` is the independent final integrity summary. Existing selected manifests are incremental probe records, not substitutes for the final full manifest.

Every result includes source and SVG SHA256 values, exact source bytes, original subprocess status/stderr, original arguments, grammar-file paths/hashes, expected and painted labels, positive/finite geometry state, original width/height/viewBox, content bounds, shape count, actual browser fonts and external href/foreign-object counts. The source inputs are at most **215 bytes** each.

Java is run headlessly with `-DPLANTUML_SECURITY_PROFILE=SANDBOX`, UTF-8 input on stdin and SVG output on stdout, bounded to **20 seconds** and **8 MiB** per case. The general invocation is:

```sh
java -Djava.awt.headless=true -DPLANTUML_SECURITY_PROFILE=SANDBOX \
  -jar /tmp/ive-diagram-oracles.JwN5H5/plantuml-mit-1.2026.9.jar \
  -charset UTF-8 -SdefaultFontName='Liberation Sans' -tsvg -pipe
```

The exact per-fixture arguments in the manifest take precedence: some data/experimental families intentionally omit the CLI font flag, as explained below. This command is a development reference, never a proposed native runtime dependency.

One headless browser measures the full run. It has offline mode enabled before rendering SVGs and uses no HTTP server. Browsers close in `finally`; every Java process completed synchronously. No window was activated, no real mouse/keyboard was used, and no IVE instance or persistent process was started.

## Meaningful admitted cases

All cases in this table have `original-rendered` status, all their declared expected labels, at least one shape and positive finite content bounds. The listed viewBox sizes are original SVG user units; complete bounds and labels are in the per-case JSON.

| Fixture stem | Actual structure tested | Original viewBox size |
|---|---|---|
| `sequence-group-activation-note` | Browser/Service messages inside Checkout group, activation/deactivation and Check stock note | 296 × 238 |
| `class-fields-inheritance-association` | Account/Premium inheritance, fields/method, cardinality and labelled Ledger association | 238 × 242 |
| `object-slots-relation` | Cart/Item objects, total/sku slots and composition-style contains relation | 118 × 186 |
| `component-deployment-groups` | Cluster node, nested Worker pool package, components/database and two labelled relations | 207 × 399 |
| `usecase-boundary-relations` | Shopper actor, Store boundary, two usecases and includes relation | 159 × 327 |
| `nested-state` | Fulfilment/Packed nesting, initial/final nodes and labelled transitions | 174 × 627 |
| `activity-branch-merge` | Start/read, Valid? yes/no branch, Persist/Reject, merge, Reply and stop | 166 × 322 |
| `mindmap-hierarchy` | Three-level Release/Tests/Unit and Delivery/Package hierarchy | 347 × 133 |
| `wbs-hierarchy` | Migration/Design/Schema and Rollout/Monitor hierarchy | 262 × 197 |
| `gantt-dated-dependencies` | Explicit 2026-10-01 project start, Plan/Build/Verify durations and end-based dependencies | 262 × 129 |
| `timing-transitions` | Robust Pump signal, Idle/Running states and transitions at 0/10/25 | 395 × 118 |
| `json-nested` | Nested service object, ports array and enabled boolean; all keys/values painted | 358 × 80 |
| `yaml-nested` | Nested service mapping, replica scalar and zones list; all keys/values painted | 338 × 102 |
| `salt-form` | Login label, user text field and Cancel/Sign in buttons | 137 × 77 |
| `nwdiag-network` | edge network/CIDR, gateway/portal addresses and attached nodes | 251 × 139 |
| `archimate-authentic-uml` | Actual `archimate #color` declarations and reads relation inside `@startuml` | 145 × 169 |
| `chen-entities-relationship` | CUSTOMER/ORDER entities with attributes, PLACES relationship and cardinalities | 237 × 540 |
| `regex-branch-repeat` | INV- literal, A/B alternation and repeated digit range | 236 × 106 |
| `ebnf-optional-repeat` | invoice rule with literal, named digit, repetition and optional -X suffix | 402 × 130 |
| `chart-bars` | Week category axis, Mon/Tue/Wed labels, Orders series with value labels and legend | 593 × 451 |
| `wire-components-link` | Sized Input/Output components in separate columns and signal link | 261 × 101 |
| `board-columns-cards` | Backlog/Done columns and Add search/Fix login cards | 372 × 193 |
| `files-tree` | app/src/main.rs and app/README.md nested paths | 144 × 120 |
| `packetdiag-ranges` | Explicit Version/Flags/Length/Payload bit ranges in a packetdiag block | 735 × 150 |
| `hcl-nested-data` | service block, ledger name, zones array and nested tags object | 234 × 109 |
| `git-linear-history` | Authentic ASCII Git log containing c3/b2/a1 connected commit rows | 47 × 155 |
| `flow-directed-tiles` | Exact MIT ID/quoted-label syntax with south/east placement of Read/Check/Store | 401 × 141 |
| `bpm-events-branch` | Exact MIT colon-event syntax, new branch/else/end branch and Archive merge continuation | 527 × 153 |

The source start tags and grammar matter. Archimate is tested through real declarations in the description-diagram grammar under `@startuml`, not an invented family alias. Packet uses `@startpacketdiag`; Git uses ASCII log input; flow uses the MIT n/s/e/w tile grammar; BPM uses its own branch/event commands. No candidate aliases or example corpus were used.

## Genuine original limitations: formula fallback

Both `math-equation.puml` and `latex-fraction.puml` use real MIT factory-recognized syntax. The first requests `x^2+y^2=z^2` under `@startmath`; the second requests `\frac{1}{2}+x` under `@startlatex`. Both subprocesses exit zero and produce valid, positive-size SVG, but stderr reports:

```text
java.lang.ClassNotFoundException: org.scilab.forge.jlatexmath.TeXFormula
```

The actual painted output is only the raw expression in DejaVu Sans Mono, with zero counted shapes. These cases have **`original-degraded-formula-fallback`** status and are excluded from the 28 meaningful admitted diagrams. Their original viewBoxes are 103 × 27 and 120 × 27 respectively. This exact MIT jar does not establish successful typeset-formula support; literal fallback must not be reported as mathematical layout parity. No missing optional formula library was installed to force success.

## Font/configuration pitfall and actual fonts

The UML-oriented core fixtures, mindmap/WBS/Gantt/timing/Salt/nwdiag/Archimate request Liberation Sans through the CLI font option. JSON/YAML and extended data/experimental fixtures use the renderer defaults; the actual fonts are recorded rather than assumed to obey a global setting. Observed paint families across this corpus are Liberation Sans, DejaVu Sans (a JSON boolean glyph fallback), DejaVu Sans Mono (Git and raw formula fallback), and Liberation Serif (flow tiles).

The original jar has an important CLI behavior: applying `-SdefaultFontName=Liberation Sans` to valid JSON/YAML input injects a skinparam into their data parsing path. The result is an exit-zero placeholder, `Your data does not sound like JSON data` or `Your data does not sound like YAML data`, despite valid input. The independent check reran the exact input bytes with and without that option; omitting it restores every expected key/value label. The final references use the appropriate per-family arguments. Those earlier placeholders are not counted as admission.

Expected-label validation catches this type of false positive. The explicit formula-fallback classification separately catches a missing rendering dependency even when raw source text remains painted.

## Verification and practical limits

`node verify.mjs` passes for the coherent final run:

- 30 exact source hashes and 30 exact SVG hashes match the current artifacts.
- All authorized grammar source hashes still match the recorded paths.
- All 30 outputs are XML-valid and have positive finite content bounds; all Java exits are zero.
- All 28 meaningful admitted cases retain expected painted labels and have nonempty shape geometry.
- The two formula fallbacks retain explicit degraded status and missing-engine diagnostics.
- No output contains a `foreignObject` or external href reference.

The checks are intentionally distinct: a valid SVG containing an error/fallback message is not meaningful diagram support. Label retention and shape counts are still not complete semantic validation of every relation, containment rule or style. Measurements use browser `getBBox()` in root SVG user coordinates, exclude stroke extents, and may include content outside the viewBox. Font CSS and actual platform-font evidence are recorded separately.

No native backend outputs were compared during this task, and no raster or pixel comparison was performed. This corpus is an independent original-renderer test baseline for the new native designer. Later native tests should consume the exact `.puml` bytes and use the original metadata/labels/geometry as separate layers of evidence, with unsupported/degraded original behavior preserved explicitly.
