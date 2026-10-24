# Developer-only Graphviz WASM reference

This reference runs a third-party prebuilt Graphviz 16.1.0 WASM bundle in Node. It supplies `json0` and SVG evidence alongside the browser Rust WASM ABI. Its source provenance and equivalence to the vendored Graphviz archive have **not** been audited. The identity manifest must remain `reviewed: false`; these results do not certify source-pinned native WASM Graphviz or PNG parity.

## Pin and run the available reference

The package is `@hpcc-js/wasm-graphviz` 1.30.0, with npm archive SHA-256 `0105defc8548e241a08a22e40ee0499a851402c345e38745a5b4d521d776d57f` and JavaScript bundle SHA-256 `77d45a274928484dd1daccd55ab3e41ddda5b4262e29240e205f7188839d9d5d`. It reports embedded Graphviz version 16.1.0. The extraction script rejects a changed archive or bundle, and records the exact runner and Node binary hashes. No package enters the product build.

From `diagram-ports/`:

```sh
curl -fL 'https://registry.npmjs.org/@hpcc-js/wasm-graphviz/-/wasm-graphviz-1.30.0.tgz' \
  -o /tmp/hpcc-js-wasm-graphviz-1.30.0.tgz
python3 tools/dot-parity/prepare_hpcc_wasm.py \
  --tarball /tmp/hpcc-js-wasm-graphviz-1.30.0.tgz \
  --output /tmp/dot-parity-hpcc-wasm-pin
export DOT_WASM_PACKAGE_ROOT=/tmp/dot-parity-hpcc-wasm-pin/package
node tools/dot-parity/run_hpcc_wasm.cjs -V

python3 tools/dot-parity/prepare_linux_gd_fonts.py --output /tmp/dot-gd-fonts
export FONTCONFIG_FILE=/tmp/dot-gd-fonts/fonts.conf
python3 tools/dot-parity/pin_linux_svg_rasterizer.py \
  --command tools/dot-parity/inkscape-command.linux.json \
  --candidate-target wasm --output /tmp/inkscape-wasm-svg-manifest.json
cargo build --locked --release --target wasm32-unknown-unknown -p diagram-web
python3 tools/dot-parity/compare.py \
  --oracle tools/dot-parity/run_hpcc_wasm.cjs \
  --oracle-metadata /tmp/dot-parity-hpcc-wasm-pin/manifest.json \
  --candidate-command tools/dot-parity/wasm-command.target.json \
  --candidate-wasm target/wasm32-unknown-unknown/release/diagram_web.wasm \
  --target-platform wasm --candidate-svg-only \
  --rasterizer-command tools/dot-parity/inkscape-command.linux.json \
  --rasterizer-manifest /tmp/inkscape-wasm-svg-manifest.json \
  --output /tmp/dot-parity-wasm-reference-seed
```

Repeat the final command with `--fixtures tools/dot-parity/fixtures-features`, `fixtures-extensions`, or `fixtures-arrow-ellipse` and distinct output directories. `compare.py` exits 1 for `incomplete` because PNG was intentionally omitted, even when JSON and SVG pixels match. Inspect the case fields in `report.json`. The Inkscape rasterizer is Linux-hosted. Raw SVG bytes may differ because the Rust output omits Graphviz's DTD and generator comment; compare `svg.raster` separately.

Count full parsed JSON identity independently of the geometry comparison:

```sh
python3 tools/dot-parity/summarize_wasm_reference.py \
  /tmp/dot-parity-wasm-reference-seed \
  --output /tmp/dot-parity-wasm-reference-seed/summary.json
```

## Checkpoint8 diagnostics on this Linux host

The immutable browser candidate `/tmp/dot-parity-checkpoint8-bin/diagram_web.wasm` has SHA-256 `c6e060bd6e5f7235f1200fd806ad854a77efdef3fe63386da0797f432b4c4636`. Exact full parsed JSON identity, geometry/routes, and Linux SVG raster pixel counts are seed **12/12**, features **9/12** (three explicit unsupported), extensions **12/14**, and arrow/ellipse **18/18**. `node_styles.dot` now matches all three independent checks. The remaining extension differences are `ranked_self` (four geometry fields, 1,523 SVG pixels) and `ranked_self_projection` (12 fields, 8,895 pixels). Reports and summaries are `/tmp/dot-parity-{seed,features,extensions,arrow-ellipse}-wasm-expat-wasm-checkpoint8-fixed/{report,summary}.json`. The native Linux OFL GD PNG matrix is separately documented in [README.md](README.md#checkpoint8-node-styles-and-bundled-ofl-gd-oracle); it uses Graphviz on Linux and does not certify native WASM Graphviz PNG parity. The prebuilt Expat reference and Linux Inkscape raster manifests remain unreviewed.

## Historical checkpoint7 diagnostics on this Linux host

The release browser candidate SHA-256 is `bf92892b7ed312eeeeb7365b52fc5faf47e53ff6528018fb159154ae18586bec`. Full parsed JSON identity, geometry/routes, and Linux SVG raster pixels each match for seed **12/12**, features **8/12** (four explicit unsupported), extensions **12/14**, and arrow/ellipse **18/18**. The remaining extension cases are `ranked_self` (four geometry fields, 1,523 SVG pixels) and `ranked_self_projection` (12 fields, 8,895 pixels). Reports are `/tmp/dot-parity-{seed,features,extensions,arrow-ellipse}-wasm-expat-wasm-checkpoint7/{report,summary}.json`. This is an unreviewed prebuilt Graphviz WASM reference and a Linux-hosted Inkscape SVG raster comparison; it does not certify native WASM PNG or macOS output.

## Historical checkpoint6 diagnostics on this Linux host

The release browser candidate was `diagram_web.wasm` SHA-256 `64085b04a3f00d08c9814389a51ba1cb9f7af8bdf963d84aeed958c614f0019d`. Reports and structured summaries are `/tmp/dot-parity-wasm-reference-{seed,features,extensions,arrow-ellipse}-checkpoint6/{report,summary}.json`. The independent exact full JSON identity / geometry / Linux SVG pixel counts were seed **11/12**, features **7/12** (five explicit unsupported), extensions **12/14**, and arrow/ellipse **18/18**. The seed HTML table still differed in seven geometry fields and 3,647 pixels (WASM reference 88×156, Rust 83×155). The ranked self loop and its projection differed in 4 and 12 geometry fields, with 1,523 and 8,895 SVG pixels respectively. The previously unsupported multiline label feature matched all three categories. This was a third-party prebuilt reference with an unreviewed manifest and no PNG plugin.

## Expat HTML table checkpoint

The bounded one-row/one-cell `<TABLE><TR><TD>text</TD></TR></TABLE>` Rust implementation now targets the Expat-enabled Graphviz 16.1.0 behavior. On release desktop SHA-256 `b5d99c105f0fd458845b592c87d340290c69afd44dd287848c7aa3d7ca64d533` and browser WASM SHA-256 `64bc65d3d7a2258398e7f5903e66d193faf275c8b3ed7d5843903e662f7a1814`, `html.dot` has exact full JSON identity, zero differing geometry fields, and zero changed Linux SVG raster pixels against the pinned prebuilt WASM Graphviz reference. Both canvases are 88×156. The per-target reports are `/tmp/dot-parity-html-{desktop,wasm}-expat-wasm-integrated/report.json`; they remain unreviewed reference evidence.

The static helper lacks Expat and falls back to the node ID for this label. Its `/tmp/dot-parity-html-{desktop,wasm}-static-no-expat-integrated/report.json` diagnostics therefore show seven geometry fields and 3,647 SVG pixels changed, with 83×155 oracle versus 88×156 candidate dimensions. This is a known oracle capability difference. The Rust module rejects nested tables, multiple cells/rows, unknown tags and attributes, non-ASCII text, and unsupported font or shape settings explicitly; its exact result applies only to the admitted subset.

## Historical checkpoint5 diagnostics

The browser candidate was `diagram_web.wasm` SHA-256 `235ab7c04aa76927204b5d2eb4d36a0f4073de1f93b4ceaaf937e7878dde0bb3`. The prebuilt reference archive, bundle, runner, and Node binary hashes are in `/tmp/dot-parity-hpcc-wasm-pin/manifest.json`. Reports are `/tmp/dot-parity-wasm-reference-{seed,features,extensions,arrow-ellipse}-checkpoint5/report.json`.

| Independent corpus | Full parsed JSON identity | Geometry/routes | Linux SVG raster pixels | Candidate unsupported | Other mismatches |
| --- | ---: | ---: | ---: | ---: | ---: |
| Seed, 12 cases | 11/12 | 11/12 | 11/12 | 0 | HTML table: 7 geometry fields; 88×156 vs 83×155 pixels, 3,647 changed |
| Features, 12 cases | 6/12 | 6/12 | 6/12 | 6 | 0 rendered cases |
| Extensions, 14 cases | 12/14 | 12/14 | 12/14 | 0 | Ranked self loop: 4 fields/1,523 pixels; projection: 12 fields/8,895 pixels |
| Arrow and ellipse, 18 cases | 18/18 | 18/18 | 18/18 | 0 | 0 |

All 52 reference inputs rendered. `edge_arrows`, `edge_styles`, `multiline_labels`, `node_shapes`, `node_styles`, and `record_nested` are the six explicit Rust unsupported cases in the feature corpus. Every SVG pixel match above has equal dimensions and zero changed decoded pixels under this Linux Inkscape configuration. No native GD PNG result is attributed to this prebuilt WASM bundle. These numbers are for checkpoint5 before the later cluster/rank, record, and multiline work; rerun against a stable new candidate snapshot.

## Source-pinned WASM build gate

The vendored `graphviz-16.1.0.tar.xz` has SHA-256 `0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3`. On this host, `emcc`, `emconfigure`, `emmake`, a WASI C toolchain, and a WASM runtime such as Wasmtime are absent. A source-pinned Graphviz WASM build also needs WASM builds of GD, FreeType, fontconfig, and Expat for the Linux native PNG/HTML capabilities. The existing local Graphviz GD plugin and those native shared libraries cannot be linked into WASM.

For a source-pinned certification run, provide a versioned Emscripten SDK and build all dependencies for its target, then build the vendored Graphviz archive with a statically registered `dot` layout engine and JSON0/SVG/GD plugins. Pin the SDK, dependency sources, compiler flags, Graphviz source hash, generated WASM/JS hashes, Node/runtime binary, font files and fontconfig, and output capabilities in a separate manifest. Run the same corpus against that executable and include native WASM PNG output before reviewing the manifest. The prebuilt hpcc-js package does not meet this source/build provenance gate and rejects `png` format.
