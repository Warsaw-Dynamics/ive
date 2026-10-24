# DOT parity harness

This is a developer-only differential gate. It does not run in the diagram engine, viewer, or browser. The fixtures are bounded, deterministic DOT inputs under `fixtures/`; the harness records exact Graphviz 16.1.0 JSON geometry and routes, SVG output, and (when configured) PNG pixels. It writes all artifacts and `report.json` to the requested output directory.

## Oracle-only baseline

The bundled static helper accepts `-Tjson0` and `-Tsvg`, but **does not support `-Tpng`**: its PNG plugins were disabled at build time. The harness copies it to a temporary executable named `dot`, because Graphviz selects the layout engine from that name. Replace the helper path below with the local build artifact:

```sh
python3 tools/dot-parity/compare.py \
  --oracle target/debug/build/diagram-viewer-<hash>/out/ive-dot-helper \
  --output /tmp/dot-parity-oracle
```

The harness refuses an oracle that does not identify itself as Graphviz 16.1.0. It records the executable SHA-256, version, host OS, architecture, locale, timezone, available fontconfig matches, and bundled WASM font hashes. Pass `--target-platform linux|macos|wasm` to label the candidate under test and `--font-manifest FILE.json` for a reviewed platform-specific font inventory. This Linux helper alone cannot certify macOS or WASM parity.

## Candidate comparison

Pass `--candidate-command` as a JSON file containing an argument array, with `{input}` and `{output}` placeholders. The command runs three times per fixture, producing Graphviz-style JSON0, SVG, and PNG files. `{format}` is available if the command requires an explicit format argument. The checked-in `candidate-command.example.json` uses the standalone viewer. Run the harness from `diagram-ports`, or put an absolute executable path in the command file. For example:

```json
["target/release/diagram-viewer", "--language", "dot", "--render", "{input}", "--output", "{output}"]
```

The harness converts both JSON0 documents' y-up coordinates and bounding boxes to a common y-down frame before comparison. Geometry comparison includes graph/cluster bounds, node centers and sizes, edge routes, and label positions. Missing fields in the candidate are reported as differences. It matches objects by name and edges by `id` (or their array index when no ID is present), compares numbers exactly, and lists up to 500 differing fields per case. The earlier custom y-down candidate geometry schema is also accepted for historical reports. SVG byte digests and exact-byte equality are recorded separately.

For SVG pixel and PNG comparison, provide `--rasterizer-command` as another JSON argument array with `{input}` and `{output}`. The same command rasterizes both the oracle and candidate SVG. The candidate's native PNG is also compared against the oracle SVG raster. Results using an unpinned rasterizer are **diagnostic only**; a matching case is reported as `diagnostic_match`, never as certified `exact`.

For a Linux SVG raster reference, `--rasterizer-manifest` verifies an Inkscape identity covering command, version, binary, linked libraries, fontconfig and fonts. The pin script writes `reviewed: false`; a separately audited manifest may set it to `true`. The harness separately checks whether the candidate native PNG renderer is the same rasterizer. The current Rust viewer uses a GD-compatible fast path for a restricted graph subset and resvg fallback otherwise, so Inkscape can establish SVG raster equality but cannot certify its native PNG bytes. macOS and WASM require their own renderer and font identities. The [macOS runbook](MACOS.md) gives a native-oracle build and comparison procedure; no macOS results are claimed here.

Cases report `exact`, `mismatch`, `incomplete`, `diagnostic_match`, `oracle_ready`, or `error`. The process exits nonzero for every candidate status except certified `exact`.

## Linux SVG raster pixel oracle

Use the private fontconfig inventory described below and the checked-in Inkscape command template. This is separate from Graphviz's GD native PNG oracle:

```sh
python3 tools/dot-parity/prepare_linux_gd_fonts.py --output /tmp/dot-gd-fonts
export FONTCONFIG_FILE=/tmp/dot-gd-fonts/fonts.conf
python3 tools/dot-parity/pin_linux_svg_rasterizer.py \
  --command tools/dot-parity/inkscape-command.linux.json \
  --output /tmp/inkscape-svg-manifest.json
python3 tools/dot-parity/compare.py \
  --oracle target/debug/build/diagram-viewer-<hash>/out/ive-dot-helper \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --rasterizer-command tools/dot-parity/inkscape-command.linux.json \
  --rasterizer-manifest /tmp/inkscape-svg-manifest.json \
  --output /tmp/dot-parity-svg-pixels
```

The Linux probe pinned Inkscape 1.4.4, its executable SHA-256, 149 linked library hashes, the private fontconfig configuration and inventory hashes, six resolved font hashes, and the exact command. The manifest remained **unreviewed**. Both SVGs are rasterized by the same Inkscape executable; `svg.raster` gives image dimensions and changed pixels. Use `--candidate-svg-only` when candidate native PNG export is unavailable; exact SVG pixels then yield case status `incomplete`, never a native PNG parity claim.

The latest 12-case release probe found 12 SVG pixel matches, with no candidate errors. Native candidate PNG differed from the Inkscape raster in every case. All counts are diagnostic until the manifest is audited:

| Fixture | Oracle SVG raster | Candidate SVG raster | SVG pixels changed | Native PNG pixels changed |
| --- | ---: | ---: | ---: | ---: |
| chain | 277×369 | 277×369 | 0 | 3,564 |
| cluster | 491×243 | 491×243 | 0 | 4,954 |
| cycle | 357×369 | 357×369 | 0 | 4,366 |
| diamond | 277×369 | 277×369 | 0 | 4,215 |
| html | 83×155 | 83×155 | 0 | 879 |
| parallel | 289×216 | 289×216 | 0 | 2,244 |
| ports | 235×111 | 235×111 | 0 | 2,994 |
| rankdir_lr | 509×64 | 509×64 | 0 | 1,645 |
| self | 631×64 | 631×64 | 0 | 2,424 |
| strict | 101×155 | 101×155 | 0 | 1,585 |
| subgraph_edge | 221×248 | 221×248 | 0 | 5,697 |
| undirected | 119×251 | 119×251 | 0 | 2,016 |

## Browser WASM ABI comparison in Node

`run_wasm.mjs` calls the built browser ABI exports (`diagram_render_dot_json`, `diagram_render_dot_svg`, and `diagram_render_dot_png`) directly in Node. It checks source and WASM file sizes before loading, validates pointer and output ranges against WASM memory, bounds PNG dimensions, copies output bytes before freeing the result handle, and writes one artifact per invocation. This tests the same Rust WASM functions the browser worker calls, without starting a browser or editing production web code.

From `diagram-ports`, build the latest artifact and run all 12 standalone cases:

```sh
cargo build --locked --release --target wasm32-unknown-unknown -p diagram-web
python3 tools/dot-parity/prepare_linux_gd_fonts.py --output /tmp/dot-gd-fonts
export FONTCONFIG_FILE=/tmp/dot-gd-fonts/fonts.conf
python3 tools/dot-parity/pin_linux_svg_rasterizer.py \
  --command tools/dot-parity/inkscape-command.linux.json \
  --candidate-target wasm --output /tmp/inkscape-wasm-svg-manifest.json
python3 tools/dot-parity/compare.py \
  --oracle target/debug/build/diagram-viewer-<hash>/out/ive-dot-helper \
  --candidate-command tools/dot-parity/wasm-command.target.json \
  --candidate-wasm target/wasm32-unknown-unknown/release/diagram_web.wasm \
  --target-platform wasm \
  --rasterizer-command tools/dot-parity/inkscape-command.linux.json \
  --rasterizer-manifest /tmp/inkscape-wasm-svg-manifest.json \
  --output /tmp/dot-parity-wasm
node --test tools/dot-parity/test_wasm_runner.mjs
```

The report records `wasm32-unknown-unknown`, the WASM SHA-256 and byte count, Node version and binary hash, and hashes of all 12 bundled Liberation font files. The comparator verifies that the candidate command uses that exact WASM path. The checkpoint5 tested artifact was SHA-256 `235ab7c04aa76927204b5d2eb4d36a0f4073de1f93b4ceaaf937e7878dde0bb3` under Node v22.14.0. The oracle is explicitly a **Linux-hosted Graphviz 16.1.0** executable, and the SVG raster reference is Linux Inkscape; neither is a native WASM Graphviz reference. The rasterizer manifest remains `reviewed: false`, and WASM pixel certification is disabled even if a Linux host manifest is reviewed.

The latest 12-case WASM run found 12 exact geometry matches and 12 exact SVG raster pixel matches, with no parser errors. All twelve WASM PNGs differed from the Linux Inkscape raster. Case status is `mismatch` for every case because the combined gate includes PNG; no case is certified `exact`.

A separate [developer-only WASM Graphviz reference procedure](WASM-REFERENCE.md) pins the `@hpcc-js/wasm-graphviz` 1.30.0 archive, its Graphviz 16.1.0 bundle, and Node runner. Its checkpoint6 comparison against the Rust browser WASM found 11/12 seed, 7/12 feature, 12/14 extension, and 18/18 arrow/ellipse cases with exact full parsed JSON identity, geometry/routes, and Linux-rasterized SVG pixels. Five feature cases returned explicit Rust unsupported errors; the three rendered mismatches were the seed HTML table and two ranked self-loop extension cases. Source equivalence of that third-party prebuilt bundle is unreviewed, its PNG format is unavailable, and Linux Inkscape pixels do not certify a native WASM rasterizer.

| Fixture | Geometry fields | Oracle SVG raster | WASM SVG raster | SVG pixels changed | WASM PNG pixels changed |
| --- | ---: | ---: | ---: | ---: | ---: |
| chain | 0 | 277×369 | 277×369 | 0 | 3,564 |
| cluster | 0 | 491×243 | 491×243 | 0 | 4,954 |
| cycle | 0 | 357×369 | 357×369 | 0 | 4,366 |
| diamond | 0 | 277×369 | 277×369 | 0 | 4,215 |
| html | 0 | 83×155 | 83×155 | 0 | 996 |
| parallel | 0 | 289×216 | 289×216 | 0 | 2,244 |
| ports | 0 | 235×111 | 235×111 | 0 | 3,665 |
| rankdir_lr | 0 | 509×64 | 509×64 | 0 | 1,645 |
| self | 0 | 631×64 | 631×64 | 0 | 2,424 |
| strict | 0 | 101×155 | 101×155 | 0 | 1,829 |
| subgraph_edge | 0 | 221×248 | 221×248 | 0 | 5,981 |
| undirected | 0 | 119×251 | 119×251 | 0 | 2,055 |

## Linux native Graphviz PNG oracle

The static helper cannot produce PNG, but the same vendored Graphviz 16.1.0 source can be built with its GD plugin on Linux. This path remains developer-only and never enters Cargo, the viewer build, or runtime. The host needs Graphviz's ordinary C build tools and the `gdlib` development package. Build in `/tmp`, capture a pin manifest, then compare:

```sh
tools/dot-parity/build_linux_gd.sh /tmp/graphviz16-gd-prefix
python3 tools/dot-parity/prepare_linux_gd_fonts.py --output /tmp/dot-gd-fonts
export FONTCONFIG_FILE=/tmp/dot-gd-fonts/fonts.conf
python3 tools/dot-parity/pin_linux_gd.py \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --output /tmp/graphviz16-gd-manifest.json
mkdir -p /tmp/dot-parity-html-only
cp tools/dot-parity/fixtures/html.dot /tmp/dot-parity-html-only/html.dot
python3 tools/dot-parity/compare.py \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --native-gd-png --native-gd-manifest /tmp/graphviz16-gd-manifest.json \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --fixtures /tmp/dot-parity-html-only \
  --output /tmp/dot-parity-native-gd
```

Omit `--fixtures` to run all 12 standalone cases.

`pin_linux_gd.py` records the pinned source hash, Graphviz version, executable and GD plugin hashes, every dynamically linked library hash, fontconfig inventory and configuration hashes, six resolved font hashes, and fontconfig environment. It writes `reviewed: false`; a separately reviewed manifest may set it to `true`. The comparator verifies every field before recording `native_gd_png_oracle.certified`. Native PNG pixels and dimensions appear under each case's `png.native_graphviz`. This GD oracle is a separate rendering configuration from the static helper: its text metrics and JSON geometry can differ.

The private font setup copies three SHA-pinned URW OTF files and rewrites **only fontconfig's scanned family names**: modern `Nimbus Mono PS`, `Nimbus Sans`, and `Nimbus Roman` become the legacy `Nimbus Mono L`, `Nimbus Sans L`, and `Nimbus Roman No9 L` names expected by this libgd build. No system or user font files or configuration are changed. Without this setup, `chain.dot` leaves a PNG but Graphviz exits 1 on its `Courier` label; with it, Graphviz exits 0. The `FONTCONFIG_FILE` environment variable must be present for both manifest creation and comparison. Native GD PNG alone does not certify SVG pixels: a reviewed shared SVG rasterizer is still required for an `exact` case status. macOS needs its own Graphviz renderer, linked-library and font manifest; WASM needs a separately pinned rasterizer and bundled font inventory because this Linux GD executable cannot run there. Neither platform inherits the Linux certification.

The checkpoint5 Linux release run with the isolated fontconfig produced **four exact native GD PNGs** (`chain`, `cycle`, `diamond`, `self`) and eight mismatches, with no candidate errors. The WASM ABI run against the same Linux-hosted GD reference has the same four exact cases; it does not establish a native WASM Graphviz oracle. These pixel counts use a common canvas. The GD manifest remains **unreviewed**, so these are diagnostic measurements:

| Fixture | Graphviz PNG | Rust PNG | Different pixels |
| --- | ---: | ---: | ---: |
| chain | 277×369 | 277×369 | 0 |
| cluster | 491×243 | 491×243 | 4,432 |
| cycle | 357×369 | 357×369 | 0 |
| diamond | 277×369 | 277×369 | 0 |
| html | 104×156 | 83×155 | 5,114 |
| parallel | 289×217 | 289×216 | 2,539 |
| ports | 268×110 | 235×111 | 7,082 |
| rankdir_lr | 509×64 | 509×64 | 1,886 |
| self | 631×64 | 631×64 | 0 |
| strict | 132×155 | 101×155 | 6,558 |
| subgraph_edge | 221×247 | 221×248 | 5,923 |
| undirected | 119×251 | 119×251 | 1,925 |

### Native GD pixel gap and achieved chain case

The [historical chain pixel report](artifacts/chain-gd-vs-resvg.json) and [diff mask](artifacts/chain-gd-vs-resvg-mask.png) document the previous resvg path: both canvases were 277×369 but 3,014 pixels differed. GD writes an opaque two-color palette image (black and white); Rust resvg wrote RGBA with 44 distinct colors. The differences followed node outlines, curved edge strokes, and arrow polygon boundaries, plus a translucent bottom row. The new restricted GD-compatible Rust path removes all those differences for `chain`: both desktop and WASM decoded PNG pixels now match the pinned Linux GD oracle exactly at 277×369. The historical mask remains to explain the raster rules that were implemented.

The [strict pixel report](artifacts/strict-gd-vs-resvg.json) and [diff mask](artifacts/strict-gd-vs-resvg-mask.png) show a separate text-layout cause. The Expat/GD build measures the `quoted id` node at 91 points wide, while the static helper and Rust JSON measure 68.493 points. That produces SVG widths of 99 versus 76 points and PNG widths of 132 versus 101 pixels at the same 96 DPI. Of 6,558 changed pixels, 4,805 are the 31×155 strip outside the Rust canvas. The remaining 1,753 changes include shifted text glyphs, node borders, edge strokes, and blue arrow pixels. GD's image has 10 palette colors; Rust has 60 RGBA colors. The masks are exact pixel comparisons, while attribution to SVG elements is based on their recorded coordinates and element types.

The underlying rules follow the pinned Graphviz 16.1.0 source in `vendor/graphviz/graphviz-16.1.0.tar.xz`: `lib/common/emit.c` rounds the point canvas to pixels at device DPI; `plugin/gd/gvrender_gd.c` sets 96 DPI, defaults to a palette image, rounds polygon points, draws paths as ten straight segments per Bézier, calls `gdImageLine`, and fills arrow polygons with `gdImageFilledPolygon`. `plugin/gd/gvtextlayout_gd.c` measures GD labels through FreeType/libgd. Rust desktop and WASM use the restricted in-process GD-compatible path when its predicate accepts a graph, then resvg otherwise. WASM bundles Liberation fonts, while desktop fallback font selection loads system fonts.

Visible text and HTML tables were the main native GD PNG frontier at checkpoint6. The Rust native predicate in `engine/src/dot_runtime/gd_raster.rs` then required empty node labels and space-only Courier edge labels; other inputs fell back to resvg. Graphviz's `plugin/gd/gvtextlayout_gd.c` resolves PostScript font aliases through fontconfig and calls `gdImageStringFTEx` with a null image to obtain FreeType bounding boxes at 72 DPI. `plugin/gd/gvrender_gd.c` positions spans from those widths and centerline offsets, then calls `gdImageStringFTEx` again on the image at renderer DPI. Its `lib/common/htmllex.c` uses Expat for table markup; `lib/common/htmltable.c` sizes and paints cells and text. In checkpoint6, GD `strict` differs in five geometry fields (91 versus 68.493 point graph width) and its PNG is 132×155 versus 101×155, with 6,558 changed pixels. GD `html` differs in seven fields and its PNG is 104×156 versus 83×155, with 5,114 changed pixels. `undirected` has zero GD geometry differences yet 1,925 changed PNG pixels, making it an existing text-raster probe. The practical order is GD-compatible text measurement and font resolution, then glyph pixels and paint placement, then Expat-equivalent HTML table geometry and border/text painting. These are separate from the already exact blank-label GD subset.

Four native-GD pixel milestones are achieved: `chain`, `cycle`, `diamond`, and `self`. The remaining cases add visible text, font-dependent metrics, record/port behavior, clusters, more edge routes, and other drawing rules. The strict case still differs in measured node width (132×155 GD versus 101×155 Rust) and 6,558 pixels. Achieving native GD parity on Linux, macOS, and WASM would require separately pinned platform oracles and font/render-library inventories plus validation on each target; this Linux GD oracle cannot certify the others. A shared pinned SVG rasterizer offers a different possible PNG reference, but its pixels would be **SVG-derived**, not Graphviz's native GD PNG.

## Current geometry snapshot

The checkpoint5 Linux release viewer and browser WASM runs against the pinned static helper each produced **12/12 exact JSON geometry and 12/12 exact Linux SVG raster pixels**. All 12 native candidate PNGs differ from the Linux SVG raster, because the candidate uses a restricted GD-compatible fast path and resvg fallback. Against the separate Graphviz GD native PNG oracle, four cases are exact on both desktop and WASM (4/12); the GD build has distinct Expat/font text metrics and gives only 2/12 exact JSON geometry. These are different oracle configurations and their counts must stay separate. The static helper lacks Expat HTML table parsing; the separate Expat-enabled corpus below measures actual table semantics.

Graphviz 16.1.0 source explicitly rounds graph `nodesep` and `ranksep` to integer points in `lib/common/input.c` via the `POINTS` macro (`ROUND(inches * 72)`). It does **not** support a blanket claim that node width and height inputs are rounded the same way: `common_init_node` reads them as doubles, `poly_init` uses floating-point point dimensions, and `gv_nodesize` converts them with `INCH2PS` without `ROUND`. Shape and label calculations may produce integer-point dimensions for particular nodes. The source is the pinned `vendor/graphviz/graphviz-16.1.0.tar.xz` archive.

## Expat HTML-table corpus

The static helper's `html.dot` result silently falls back to the node ID `a`; it does not render the authored HTML table. Graphviz's `lib/common/htmllex.c` requires Expat for table formatting, and the separate GD build has `HAVE_EXPAT=1` and links `libexpat.so.1`. Use the two cases in `fixtures-expat/` with that build. The `oracle-assertions.json` sidecar requires table text in the oracle SVG, so the static helper reports a capability error instead of creating a misleading baseline.

```sh
export FONTCONFIG_FILE=/tmp/dot-gd-fonts/fonts.conf
python3 tools/dot-parity/compare.py \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --fixtures tools/dot-parity/fixtures-expat \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --native-gd-png --native-gd-manifest /tmp/graphviz16-gd-manifest.json \
  --output /tmp/dot-parity-expat
```

Both HTML cases render through the Expat-enabled oracle and candidate, but neither matches. `html_table_cells.dot` has 7 geometry differences and 10,878 native PNG pixel differences (104×191 oracle, 83×155 candidate). `html_table_port.dot` has 5 geometry differences and 5,112 pixel differences (257×59 oracle, 203×59 candidate). These are separate GD/Expat measurements; their text metrics differ from the static helper's configuration. The GD manifest remains unreviewed.

## Malformed input and admission bounds

`check_rejections.py` runs five deterministic negative cases against the desktop or WASM command template: missing closing brace, unterminated quoted ID, source above 256 KiB, 10,001 statements, and 65 nested subgraphs. The two malformed cases also require Graphviz to reject. With `--check-layout-work`, it compiles a developer-only probe against the already-built engine library, confirms valid `chain.dot` renders under default limits, then sets `layout_work=1` and requires the explicit `DOT work count 2 exceeds 1` error. Inputs, hashes, exit status, bounded output, and pass/fail results are written to `report.json`:

```sh
python3 tools/dot-parity/check_rejections.py \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --check-layout-work \
  --output /tmp/dot-parity-rejections
python3 tools/dot-parity/check_rejections.py \
  --candidate-command tools/dot-parity/wasm-command.target.json \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --check-layout-work \
  --output /tmp/dot-parity-wasm-rejections
```

The latest desktop and WASM runs both passed 6/6. The sixth check exercises the engine in-process on the Linux host for both reports; it does **not** exercise the limit through viewer CLI or browser WASM ABI, which do not expose that option. The positive comparator also caps fixture count, source bytes, command time, output bytes, and PNG pixels.

On Linux, verify the desktop DOT export path launches no worker child. This traces `execve` for SVG, PNG, and JSON and requires one executable launch each; a PlantUML export is the positive control and must launch the retained viewer worker:

```sh
cargo build --locked -p diagram-viewer
python3 tools/dot-parity/check_dot_no_worker.py \
  --viewer target/debug/diagram-viewer \
  --dot tools/dot-parity/fixtures/chain.dot \
  --plantuml fixtures/plantuml/class.puml \
  --output /tmp/dot-no-worker
```

The checkpoint6 Linux trace passed: DOT SVG/PNG/JSON each had one `execve` total; PlantUML had two. This is a desktop export check. The UI service separately has in-process DOT render, cancellation, and source-limit unit tests; other languages retain their isolated worker.

## Diagnostic DOT feature corpus

`fixtures-features/` is a separate, deterministic 12-case corpus for attributes and layout rules outside the acceptance seed. `feature-coverage.json` records the exact feature tags for each input. The pinned static Graphviz 16.1.0 helper accepts all 12 cases; its SVG raster is measured with the Linux Inkscape manifest. The comparator keeps its original strict JSON, SVG pixel, and PNG pixel measurements, while `summarize_features.py` counts explicit unsupported candidate errors separately from numerical geometry, identity/structure, SVG pixel, and PNG pixel gaps. Counts overlap when one rendered case differs in multiple ways. These results are diagnostic and do not change the acceptance-seed counts.

| Fixture | Features |
| --- | --- |
| graph_label, graph_background, graph_label_background | Graph labels, top/bottom placement, left alignment, multiline caption, solid background colors |
| node_shapes, node_styles | Box, diamond, ellipse, plaintext, rounded, filled, dashed, bold, dotted, peripheries |
| edge_arrows, edge_styles | Both-end and absent arrows, vee/diamond/box heads, arrowsize, color, penwidth, dashed/dotted/bold |
| record_nested | Nested record fields and named record ports |
| rank_same, rank_minmax, rank_direction | Same/source/sink ranks, LR direction, ranksep, nodesep |
| multiline_labels | Node `\n`, `\l`, `\r` and edge multiline labels |

```sh
export FONTCONFIG_FILE=/tmp/dot-gd-fonts-repro/fonts.conf
python3 tools/dot-parity/compare.py \
  --oracle target/debug/build/diagram-viewer-<hash>/out/ive-dot-helper \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --rasterizer-command tools/dot-parity/inkscape-command.linux.json \
  --rasterizer-manifest /tmp/inkscape-svg-manifest-final2.json \
  --fixtures tools/dot-parity/fixtures-features \
  --output /tmp/dot-parity-features
python3 tools/dot-parity/summarize_features.py \
  --report /tmp/dot-parity-features/report.json \
  --output /tmp/dot-parity-features/feature-summary.json
```

The final parser checkpoint produced six explicit unsupported errors and six rendered cases. **All six rendered cases have exact full JSON and exact Linux SVG raster pixels**; all six native PNGs differ from the Linux SVG raster. Graph label/background and rank-subgraph identity now match the static oracle. The rasterizer manifest remains unreviewed, and these are Linux-hosted measurements, not macOS or WASM evidence.

| Fixture | Candidate result | Full JSON | Numeric / identity fields | SVG raster | Native PNG vs SVG raster |
| --- | --- | ---: | ---: | ---: | ---: |
| edge_arrows | unsupported arrow mode | — | — | — | — |
| edge_styles | unsupported penwidth | — | — | — | — |
| graph_background | rendered | exact | 0 / 0 | 0 pixels, 83×155 | 1,130 pixels |
| graph_label | rendered | exact | 0 / 0 | 0 pixels, 151×188 | 1,913 pixels |
| graph_label_background | rendered | exact | 0 / 0 | 0 pixels, 87×211 | 1,878 pixels |
| multiline_labels | unsupported node/edge multiline | — | — | — | — |
| node_shapes | unsupported diamond shape | — | — | — | — |
| node_styles | unsupported rounded/filled style | — | — | — | — |
| rank_direction | rendered | exact | 0 / 0 | 0 pixels, 381×155 | 2,872 pixels |
| rank_minmax | rendered | exact | 0 / 0 | 0 pixels, 83×347 | 2,348 pixels |
| rank_same | rendered | exact | 0 / 0 | 0 pixels, 179×155 | 1,810 pixels |
| record_nested | unsupported nested record | — | — | — | — |

The source hashes, candidate error text, per-case dimensions, and mismatch fields are in `/tmp/dot-parity-features-desktop-static-checkpoint5/report.json`; `/tmp/dot-parity-features-summary-checkpoint5.json` holds the categorized counts. These two files are generated diagnostic artifacts and should be saved with the manifest when reviewing parity. The original 12-case seed results remain separate at 12/12 JSON and SVG raster matches.

## Diagnostic style, diamond, and ranked-loop corpus

`fixtures-extensions/` is a third, independent 14-case corpus for recently added drawing and layout paths. Its `coverage.json` lists every feature, and `projection-provenance.json` records the exact source test, PlantUML extraction, generator, and DOT hashes for the copied `ranked_self_projection.dot`. That file byte-matches `uml_parallel_cycle_self.dot` from `generate_plantuml_corpus.py`; the minimal `ranked_self.dot` adds a labeled self loop on an upper rank plus a downward edge. Both the static Graphviz 16.1.0 helper and the Expat/GD-enabled 16.1.0 oracle accept all 14 inputs. This corpus does not change the 12-case seed or 12-case feature counts.

| Group | Fixtures | Covered behavior |
| --- | --- | --- |
| Isolated diamond | `diamond_isolated`, `diamond_filled_gold`, `diamond_long_label` | Default shape, gold fill, label-driven width |
| Node styles | `box_rounded_filled`, `box_dashed`, `box_dotted`, `ellipse_dashed_bold` | Rounded/fill, dashed/dotted/bold strokes, penwidth 2 |
| Edge styles | `edge_dashed_red`, `edge_dotted_gray`, `edge_bold_gray` | Colored dashed/dotted/bold routes |
| Native GD probes | `box_colored_gd`, `edge_colored_gd` | Blue/yellow box and red edge with blank labels |
| Ranked loops | `ranked_self`, `ranked_self_projection` | Upper-rank self loop and PlantUML parallel/cycle/self topology |

Run four comparisons sequentially, because concurrent Inkscape processes can fail during DBus initialization. The first two use the static oracle plus the pinned Linux SVG rasterizer; the other two use the independently built GD oracle and native PNG. Pass each report to `summarize_extensions.py`, which checks identical fixture sets and source hashes and reports full JSON identity/geometry, raw SVG byte equality, SVG raster equality, and native GD PNG equality separately for release desktop and browser WASM. The static and GD JSON oracles can differ because the GD build has Expat and different font metrics.

```sh
export FONTCONFIG_FILE=/tmp/dot-gd-fonts-repro/fonts.conf
python3 tools/dot-parity/compare.py \
  --oracle target/debug/build/diagram-viewer-<hash>/out/ive-dot-helper \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --rasterizer-command tools/dot-parity/inkscape-command.linux.json \
  --rasterizer-manifest /tmp/inkscape-svg-manifest-final2.json \
  --fixtures tools/dot-parity/fixtures-extensions \
  --output /tmp/dot-parity-extensions-desktop-static
python3 tools/dot-parity/compare.py \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --native-gd-png --native-gd-manifest /tmp/graphviz16-gd-repro-manifest-final.json \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --fixtures tools/dot-parity/fixtures-extensions \
  --output /tmp/dot-parity-extensions-desktop-gd
python3 tools/dot-parity/compare.py \
  --oracle target/debug/build/diagram-viewer-<hash>/out/ive-dot-helper \
  --candidate-command tools/dot-parity/wasm-command.target.json \
  --candidate-wasm target/wasm32-unknown-unknown/release/diagram_web.wasm \
  --target-platform wasm \
  --rasterizer-command tools/dot-parity/inkscape-command.linux.json \
  --rasterizer-manifest /tmp/inkscape-wasm-svg-manifest-final2.json \
  --fixtures tools/dot-parity/fixtures-extensions \
  --output /tmp/dot-parity-extensions-wasm-static
python3 tools/dot-parity/compare.py \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --native-gd-png --native-gd-manifest /tmp/graphviz16-gd-repro-manifest-final.json \
  --candidate-command tools/dot-parity/wasm-command.target.json \
  --candidate-wasm target/wasm32-unknown-unknown/release/diagram_web.wasm \
  --target-platform wasm \
  --fixtures tools/dot-parity/fixtures-extensions \
  --output /tmp/dot-parity-extensions-wasm-gd
python3 tools/dot-parity/summarize_extensions.py \
  --desktop-static /tmp/dot-parity-extensions-desktop-static/report.json \
  --desktop-gd /tmp/dot-parity-extensions-desktop-gd/report.json \
  --wasm-static /tmp/dot-parity-extensions-wasm-static/report.json \
  --wasm-gd /tmp/dot-parity-extensions-wasm-gd/report.json \
  --output /tmp/dot-parity-extensions-summary.json
```

The checkpoint5 release desktop and direct WASM ABI runs, before the later cluster-gap edit, each produced **12/14 exact full static JSON cases and 12/14 exact Linux SVG raster cases**, with no candidate errors. The two ranked-loop cases differ in route/position or canvas geometry; the other twelve, including isolated diamonds, styled nodes/edges, and colored GD probes, match both static JSON and SVG pixels. Raw SVG bytes differ in all fourteen cases: the product deliberately omits Graphviz's DTD and retains its own SVG serialization, so raster equality is the relevant visual check here. Against the separate GD build, JSON is exact in 11/14 cases; the additional `diamond_long_label` difference is GD font measurement. Native GD PNG pixels match in **1/14** case on both desktop and WASM: `edge_colored_gd`. The GD and Inkscape manifests remain unreviewed.

| Fixture | Static JSON numeric fields | Static SVG pixels | GD JSON numeric fields | GD PNG pixels, desktop / WASM |
| --- | ---: | ---: | ---: | ---: |
| box_colored_gd | 0 | 0 | 0 | 248 / 248 |
| box_dashed | 0 | 0 | 0 | 397 / 401 |
| box_dotted | 0 | 0 | 0 | 209 / 213 |
| box_rounded_filled | 0 | 0 | 0 | 516 / 521 |
| diamond_filled_gold | 0 | 0 | 0 | 399 / 403 |
| diamond_isolated | 0 | 0 | 0 | 399 / 403 |
| diamond_long_label | 0 | 0 | 3 | 4,290 / 4,292 |
| edge_bold_gray | 0 | 0 | 0 | 1,269 / 1,294 |
| edge_colored_gd | 0 | 0 | 0 | 0 / 0 |
| edge_dashed_red | 0 | 0 | 0 | 1,117 / 1,142 |
| edge_dotted_gray | 0 | 0 | 0 | 1,139 / 1,164 |
| ellipse_dashed_bold | 0 | 0 | 0 | 679 / 683 |
| ranked_self | 4 | 1,523 | 6 | 4,101 / 4,149 |
| ranked_self_projection | 12 | 8,895 | 12 | 12,372 / 12,427 |

All listed JSON differences are numeric geometry/routes; there are **zero identity/structure differences** in this corpus. For `ranked_self`, the static SVG canvas is 133×156 oracle versus 133×155 candidate; GD PNG is 149×156 versus 133×155. For `ranked_self_projection`, static SVG is 253×177 versus 229×177, and GD PNG is 275×177 versus 229×177. For `diamond_long_label`, GD PNG is 227×59 versus 185×59 because its font metrics differ from the static helper. Every other GD PNG pair has matching dimensions despite its pixel differences. The four raw reports and the categorized report are under `/tmp/dot-parity-extensions-*-checkpoint5/report.json` and `/tmp/dot-parity-extensions-summary-checkpoint5.json`. This is Linux-hosted evidence; WASM comparisons exercise the browser ABI against Linux Graphviz, not a native WASM Graphviz reference.

## Diagnostic arrow and ellipse corpus

`fixtures-arrow-ellipse/` is a fourth independent corpus, copied byte-for-byte from source-backed production goldens (`engine/src/dot_runtime/goldens/`). Its twelve arrow cases cover `dir=back/both/none`, absent/default/vee/diamond/box heads and tails, and `arrowsize=2`. Its six ellipse cases cover blank fixed-size ellipse chain/cycle/diamond, filled and mixed-color chains, and a self-loop fallback. The `coverage.json` sidecar names each behavior. Both pinned Graphviz 16.1.0 oracle configurations accept all eighteen inputs.

Run the same four `compare.py` commands shown for the extension corpus, replacing `--fixtures tools/dot-parity/fixtures-extensions` with `--fixtures tools/dot-parity/fixtures-arrow-ellipse`, and pass `--coverage tools/dot-parity/fixtures-arrow-ellipse/coverage.json` to `summarize_extensions.py`. Keep output directories distinct. The checkpoint5 reports are `/tmp/dot-parity-arrow-ellipse-{desktop,wasm}-{static,gd}-checkpoint5/report.json`; the categorized report is `/tmp/dot-parity-arrow-ellipse-summary-checkpoint5.json`.

Checkpoint5 release desktop and direct WASM ABI runs, before the later cluster-gap edit, each have **18/18 exact full static JSON and 18/18 exact Linux SVG raster cases**. Shape-aware clipping removed the prior `ellipse-self` route and 305-pixel SVG difference. Raw SVG bytes differ in all eighteen cases because the production output omits Graphviz's DTD and generator comment; an `arrow_head_vee` diff shows these header lines with identical SVG body. The separate GD build gives 12/18 exact JSON (the arrow cases) because it uses different font/raster configuration, while **15/18 native GD PNGs are pixel exact on both desktop and WASM**: nine arrow cases plus all six ellipse cases, including `ellipse-self`. The remaining three arrow modes are `both`, `head_none`, and `none`. The native GD oracle is Linux-hosted and its manifest remains unreviewed; this does not certify macOS or native WASM Graphviz parity.

| Fixture | Static JSON | Static SVG pixels | GD JSON | GD PNG pixels, desktop / WASM |
| --- | ---: | ---: | ---: | ---: |
| arrow_back | exact | 0 | exact | 0 / 0 |
| arrow_both | exact | 0 | exact | 702 / 702 |
| arrow_head_box | exact | 0 | exact | 0 / 0 |
| arrow_head_diamond | exact | 0 | exact | 0 / 0 |
| arrow_head_diamond_size2 | exact | 0 | exact | 0 / 0 |
| arrow_head_none | exact | 0 | exact | 678 / 678 |
| arrow_head_size2 | exact | 0 | exact | 0 / 0 |
| arrow_head_vee | exact | 0 | exact | 0 / 0 |
| arrow_none | exact | 0 | exact | 678 / 678 |
| arrow_tail_box | exact | 0 | exact | 0 / 0 |
| arrow_tail_diamond | exact | 0 | exact | 0 / 0 |
| arrow_tail_vee | exact | 0 | exact | 0 / 0 |
| ellipse-chain | exact | 0 | mismatch | 0 / 0 |
| ellipse-cycle | exact | 0 | mismatch | 0 / 0 |
| ellipse-diamond | exact | 0 | mismatch | 0 / 0 |
| ellipse-filled-chain | exact | 0 | mismatch | 0 / 0 |
| ellipse-mixed-chain | exact | 0 | mismatch | 0 / 0 |
| ellipse-self | exact | 0 | mismatch | 0 / 0 |

The exact GD ellipse chain variants are 117×217 pixels in the native GD PNGs, while their SVG raster canvases are 117×216; each pair is compared only within its matching oracle format. `ellipse-self` has exact visible PNG pixels despite GD JSON differences from the static build's font/renderer configuration. The existing twelve-case seed, twelve-case feature, and fourteen-case extension counts remain separate.

## Checkpoint6 release matrix

The stable checkpoint6 desktop executable SHA-256 is `f48f907264036245089f44c49f3c95178637f7d37ab3d259f968931bd31ff4f3`; browser WASM SHA-256 is `64085b04a3f00d08c9814389a51ba1cb9f7af8bdf963d84aeed958c614f0019d`. Both targets have identical independent counts below. Each row uses its own deterministic fixture set. Static JSON and SVG reference the vendored Graphviz 16.1.0 helper; PNG references the separate, unreviewed Linux GD build.

| Corpus | Full static JSON identity | Static geometry/routes | Linux SVG raster pixels | Native GD PNG pixels | Explicit unsupported |
| --- | ---: | ---: | ---: | ---: | ---: |
| Seed | 12/12 | 12/12 | 12/12 | 4/12 | 0 |
| Features | 7/12 | 7/12 | 7/12 | 0/12 | 5 |
| Extensions | 12/14 | 12/14 | 12/14 | 1/14 | 0 |
| Arrow and ellipse | 18/18 | 18/18 | 18/18 | 18/18 | 0 |

The five unsupported feature cases are `edge_arrows`, `edge_styles`, `node_shapes`, `node_styles`, and `record_nested`; `multiline_labels` now matches static JSON and SVG. The two extension gaps are `ranked_self` (4 geometry fields, 1,523 SVG pixels) and `ranked_self_projection` (12 fields, 8,895 pixels). All eighteen arrow/ellipse native GD PNGs have equal dimensions and zero changed decoded pixels on both targets. The remaining seed GD pixel mismatches are `cluster`, `html`, `parallel`, `ports`, `rankdir_lr`, `strict`, `subgraph_edge`, and `undirected`. GD JSON is a different oracle configuration: its exact identity counts are seed 1/12, features 4/12, extensions 11/14, arrow/ellipse 12/18. Those GD JSON differences do not negate static JSON parity, and PNG exactness does not imply GD JSON identity.

The reports are `/tmp/dot-parity-{seed,features,extensions,arrow-ellipse}-{desktop,wasm}-{static,gd}-checkpoint6/report.json`. Both malformed-input and admission-bound guard suites passed 6/6; the sixth `layout_work` check runs in-process on the Linux host for both reports. All raster manifests remain `reviewed: false`, and no macOS or source-pinned native WASM Graphviz result is claimed.

After checkpoint6, bounded Expat-compatible HTML table support changed `html.dot` deliberately. [The WASM reference runbook](WASM-REFERENCE.md#expat-html-table-checkpoint) records exact full JSON identity, geometry, and Linux SVG raster pixels for that case on release desktop and browser WASM. The static helper lacks Expat, so its prior HTML fallback and its 12/12 seed count are historical for that older product snapshot. Future seed matrices should report Expat-enabled and no-Expat oracle configurations separately.

## Checkpoint7 full oracle matrix

The stable checkpoint7 release viewer SHA-256 is `d666190bb5160a6f112f838404d2a0822064eabe19da79fcd0f9fc0eeff2d4f8`; browser WASM SHA-256 is `bf92892b7ed312eeeeb7365b52fc5faf47e53ff6528018fb159154ae18586bec`. Both targets have identical counts. The Expat-enabled reference is the pinned, unreviewed `@hpcc-js/wasm-graphviz` Graphviz 16.1.0 WASM bundle under Node; the native GD PNG reference is the separately built Linux Graphviz 16.1.0 GD configuration. The legacy static helper has no Expat.

| Corpus | Expat full JSON identity, geometry, SVG pixels | Static no-Expat full JSON, geometry, SVG pixels | Native Linux GD PNG pixels | Explicit unsupported |
| --- | ---: | ---: | ---: | ---: |
| Seed | 12/12 | 11/12 | 4/12 | 0 |
| Features | 8/12 | 8/12 | 0/12 | 4 |
| Extensions | 12/14 | 12/14 | 1/14 | 0 |
| Arrow and ellipse | 18/18 | 18/18 | 18/18 | 0 |

The sole seed static mismatch is the expected no-Expat `html.dot` fallback: seven geometry fields and 3,647 Linux SVG pixels, with an 83×155 static canvas versus 88×156 Expat candidate. Expat-enabled seed HTML is full JSON identical and pixel exact. Feature errors are `edge_arrows`, `edge_styles`, `node_styles`, and `record_nested`; `node_shapes` now renders exactly. Extension gaps remain `ranked_self` (4 geometry fields, 1,523 SVG pixels) and `ranked_self_projection` (12 fields, 8,895 pixels). GD JSON identity is a separate font/renderer configuration: seed 1/12, features 5/12, extensions 11/14, arrow/ellipse 12/18. Native GD PNG exact cases are unchanged from checkpoint6: seed 4, features 0, extensions 1, arrow/ellipse 18. Desktop and browser rejection/work-limit suites each passed 6/6. Release DOT SVG/PNG/JSON exports each made one `execve` in the Linux trace; the PlantUML worker control made two.

Reports and Expat structured summaries are `/tmp/dot-parity-{seed,features,extensions,arrow-ellipse}-{desktop,wasm}-{expat-wasm,static-no-expat,gd}-checkpoint7/{report,summary}.json` (`summary.json` exists for Expat reports only). The process trace is `/tmp/dot-no-worker-checkpoint7/report.json`. The Expat package source provenance, native GD manifest, and Linux Inkscape raster manifest remain `reviewed: false`; no macOS or source-pinned native WASM PNG claim follows from these Linux measurements.

## Checkpoint8 node styles and bundled OFL GD oracle

The immutable release candidates are `/tmp/dot-parity-checkpoint8-bin/diagram-viewer` (SHA-256 `6f25b2381623864f80c7ceb9add6d51c62e3ed972c40950316dba579aebfd672`) and `diagram_web.wasm` (SHA-256 `c6e060bd6e5f7235f1200fd806ad854a77efdef3fe63386da0797f432b4c4636`). Both use the same stable source snapshot. The six pinned Expat-enabled JSON0/SVG goldens under `engine/src/dot_runtime/goldens/node_periphery_*` and `node_styles_feature.*` establish exact `peripheries=2` box/ellipse sizing, inner-first SVG painting, combined styles, and the original `fixtures-features/node_styles.dot` chain. Their source hashes and unreviewed package provenance are in `node-peripheries-provenance.json`. The raw WASM SVG generator stamp `(0)` is the only normalized byte sequence in the Rust raw SVG tests.

| Corpus | Full Expat JSON identity | Expat geometry/routes | Linux SVG raster pixels | Native Linux OFL GD PNG pixels | Candidate unsupported |
| --- | ---: | ---: | ---: | ---: | ---: |
| Seed | 12/12 | 12/12 | 12/12 | 5/12 | 0 |
| Features | 9/12 | 9/12 | 9/12 | 0/12 | 3 |
| Extensions | 12/14 | 12/14 | 12/14 | 1/14 | 0 |
| Arrow and ellipse | 18/18 | 18/18 | 18/18 | 18/18 | 0 |

Desktop and browser WASM have identical exact counts. The remaining feature errors are `edge_arrows`, `edge_styles`, and `record_nested`; `node_styles.dot` is now exact in full JSON identity and Linux SVG pixels. The two extension route gaps remain `ranked_self` (four geometry fields, 1,523 SVG pixels) and `ranked_self_projection` (12 fields, 8,895 pixels). Native GD `undirected.dot` joins chain, cycle, diamond, and self at 0 changed decoded PNG pixels on an equal 119×251 canvas. `node_styles.dot` has equal 93×261 native GD canvases but 1,917 changed pixels on desktop and 1,854 on browser WASM. This is a separate raster gap despite exact Expat JSON/SVG. The OFL GD reference runs on Linux for both candidates; it is not a native WASM or macOS Graphviz oracle.

The OFL GD reference uses the Graphviz 16.1.0 binary SHA-256 `61b78ded6e765b8a73d183360355029e51e5c18f9ba36e870f3cce16ee06ad85`, GD plugin SHA-256 `ce681ee83c7cc8555ee05f5e1a8e43746a92a0e2b5406cb669f3bfd48ce72169`, and isolated `FONTCONFIG_FILE` SHA-256 `28e8a2709b9ce02b4c6c352d31a9dafe98b69c6b884a060c99107b0b98a2b447`. Its unreviewed full dependency and font inventory is `/tmp/dot-parity-ofl-gd-manifest-checkpoint8-final.json`. `prepare_linux_ofl_gd_fonts.py` checks the bundled SIL-OFL Liberation font hashes and creates the private configuration. The config adds strong `Times`, `Courier`, and `Helvetica` aliases and presents the scanned OFL files as the Nimbus families required by this libgd PostScript compatibility table. Without both rules, GD can either reject Courier or silently select Liberation Mono for Times. An `strace` check confirmed the corrected undirected oracle opens `LiberationSerif-Regular.ttf` SHA-256 `705903ae1382f0150f115d6f10a30f5b2f3d6ade649c474afe6366c39174fb65`; its PNG is byte-identical to the focused OFL golden. [The libgd source](https://sources.debian.org/src/libgd2/2.2.5-5.2/src/gdft.c/) shows the hard-coded Nimbus family check. The earlier URW GD matrices remain historical diagnostics with a distinct manifest.

Reproduce the OFL PNG reference from `diagram-ports/` after building the developer-only Graphviz GD oracle:

```sh
python3 tools/dot-parity/prepare_linux_ofl_gd_fonts.py --output /tmp/dot-ofl-gd-fonts
export FONTCONFIG_FILE=/tmp/dot-ofl-gd-fonts/fonts.conf
python3 tools/dot-parity/pin_linux_gd.py \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --output /tmp/dot-parity-ofl-gd-manifest-checkpoint8-final.json
python3 tools/dot-parity/compare.py \
  --oracle /tmp/graphviz16-gd-prefix/bin/dot \
  --native-gd-png --native-gd-manifest /tmp/dot-parity-ofl-gd-manifest-checkpoint8-final.json \
  --candidate-command /tmp/dot-parity-checkpoint8-bin/candidate-desktop.json \
  --output /tmp/dot-parity-seed-desktop-ofl-gd-checkpoint8-final
```

Use the corresponding `candidate-wasm.json` and `--candidate-wasm /tmp/dot-parity-checkpoint8-bin/diagram_web.wasm --target-platform wasm` for the browser candidate, and `--fixtures tools/dot-parity/fixtures-{features,extensions,arrow-ellipse}` for the other corpora. Expat report and summary paths are `/tmp/dot-parity-{seed,features,extensions,arrow-ellipse}-{desktop,wasm}-expat-wasm-checkpoint8-fixed/{report,summary}.json`; final OFL reports are `/tmp/dot-parity-{seed,features,extensions,arrow-ellipse}-{desktop,wasm}-ofl-gd-checkpoint8-final/report.json`. All oracle/raster manifests remain `reviewed: false`. Both malformed-input/work-limit suites passed 6/6; `/tmp/dot-no-worker-checkpoint8/report.json` records one `execve` for each DOT SVG/PNG/JSON export and two for the PlantUML worker control. The five semantic PlantUML DOT projections have geometry field differences `class 0`, `deployment 1`, `object 0`, `parallel/cycle/self 12`, and `usecase 0` on both candidates; `/tmp/dot-parity-projections-{desktop,wasm}-expat-wasm-checkpoint8*/report.json` contains the source-provenance records.

### Bounded multi-edge arrow path probe after checkpoint8

The original feature fixture `fixtures-features/edge_arrows.dot` and three directed-path variants now have active pinned Expat Graphviz JSON0 and raw SVG goldens in `engine/src/dot_runtime/goldens/edge_arrow*` with hashes in `edge-arrow-path-provenance.json`. The cases cover `dir=both`, `back`, and `none`; diamond, vee, and box ends; an inactive unsupported `crow` head; and `arrowsize=1.5`/`2`. Graphviz `arrows.c` supports up to four composed, modified shapes per end, but the bounded Rust admission still rejects compositions such as `diamondvee` and branched multi-edge routes. The vendored Graphviz 16.1.0 `lib/common/arrows.c` documents its matching and `dir` behavior; goldens come from the pinned 16.1.0 WASM oracle and remain unreviewed provenance evidence.

The post-checkpoint8 **debug desktop** one-case comparison at `/tmp/dot-edge-arrows-debug-expat-report/{report,summary}.json` reports 1/1 full JSON identity, 1/1 geometry/routes, and 1/1 Linux SVG raster pixels. Its `incomplete` case status reflects the Expat WASM oracle's missing PNG output. A release desktop/browser full feature-corpus count requires a fresh integrated snapshot; checkpoint8's 9/12 feature count above is historical for its immutable binaries.

### Styled shortcut triangle diagnostic

`fixtures-edge-style-triangle/` pins three related 16.1.0 Expat JSON0/SVG oracle inputs: the original `edge_styles.dot`, the same a→b/a→c/b→c topology without style attributes, and a styled version without explicit `penwidth`. Its `provenance.json` checks all DOT source hashes and records each JSON/SVG hash with unreviewed prebuilt-oracle provenance. This isolates multi-rank shortcut routing from stroke painting.

The preguard release diagnostic `/tmp/dot-edge-style-triangle-release-diagnostic/report.json` found that the plain triangle rendered with **8 differing geometry fields and 12,329 changed Linux SVG pixels**: Graphviz's canvas is 81×180 points and the Rust candidate used 54×181. The original penwidth/style input and style-only variant already returned explicit `UnsupportedFeature` errors. A narrow guard now also returns an explicit error for the plain ordered, attribute-free default-ellipse triangle until the source-backed multi-rank route is implemented. The pinned oracle accepts all three; this corpus is diagnostic and contributes no new exact parity count.

## PlantUML-derived graph corpus

PlantUML does not emit internal DOT: its UML graph adapter builds a `dugong::Graph` and calls the in-process layered layout directly. `generate_plantuml_corpus.py` therefore projects a bounded subset of existing PlantUML fixture topology into standalone DOT. It reads the four source fixtures and one named test literal listed in `plantuml-sources.json`, preserving nodes, relations, parallel edges, self loops, relation labels, and nested containers. It does not recreate measured PlantUML node sizes, arrowhead semantics, scene styling, or a historical Graphviz invocation. These cases exercise DOT features reached by PlantUML graph models; they are **semantic projections**, not exact internal DOT dumps or an end-to-end PlantUML parity claim.

From `diagram-ports`, generate and compare them with:

```sh
python3 tools/dot-parity/generate_plantuml_corpus.py --output /tmp/dot-parity-plantuml-fixtures
python3 tools/dot-parity/compare.py \
  --oracle target/debug/build/diagram-viewer-<hash>/out/ive-dot-helper \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --fixtures /tmp/dot-parity-plantuml-fixtures \
  --output /tmp/dot-parity-plantuml-report
```

The generated `provenance.json` records every source path, named test selector, source and extracted PlantUML hashes, generated DOT hash, node/edge counts, and generator hash. The comparator automatically attaches this provenance to each case and rejects stale DOT hashes. The 2026-10-24 Linux debug run returned **five geometry mismatches**, totaling 57 differing fields: graph size 7, node positions 14, node sizes 10, routes 11, and other geometry or identity 15. Case counts were class 10, deployment 15, object 7, parallel/cycle/self 12, and usecase 13. Cluster bounds and generated cluster identity account for several larger differences. PNG parity remains uncertified without the platform rasterizer manifest described above.

Run comparator unit tests with `python3 -m unittest discover -s tools/dot-parity -p 'test_*.py'` from `diagram-ports`.
