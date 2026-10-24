# macOS DOT parity runbook

This is an executable **macOS-host procedure**, not a macOS parity result. Run it on each macOS architecture and OS build that needs a claim. It builds the pinned Graphviz 16.1.0 source as a developer-only oracle, then compares the release desktop candidate against that host's JSON0, SVG, and native GD PNG. Nothing here enters the product build or runtime.

The vendored archive has SHA-256 `0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3`. [Graphviz's source build guide](https://graphviz.org/doc/build.html) describes the `configure`, `make`, `make install` flow and the GD plugin. The package names below are the current [Homebrew gd](https://formulae.brew.sh/formula/gd), [fontconfig](https://formulae.brew.sh/formula/fontconfig), [Expat](https://formulae.brew.sh/formula/expat), [pkgconf](https://formulae.brew.sh/formula/pkgconf), and [Inkscape](https://formulae.brew.sh/cask/inkscape) entries; capture their versions in the run artifacts.

## GitHub-hosted Apple Silicon run

The repository includes [the manual/branch-push workflow](../../../.github/workflows/dot-macos-parity.yml), which runs this procedure on GitHub's `macos-15` arm64 runner. Its job builds the pinned Graphviz source and release viewer, compares seed, feature, extension, and arrow/ellipse corpora, and uploads `dot-macos-arm64-parity-<run-id>` even when known parity mismatches make the job red. The uploaded artifact contains four reports, unreviewed oracle/rasterizer manifests, private font inventory, tool versions, binary hashes, and Graphviz build logs. `tools/dot-parity/run_macos_ci.sh` is the reusable script behind the job.

A GitHub run is evidence only after its artifact is downloaded and the font, dylib, Inkscape, Graphviz, and OS identities are reviewed. The workflow has not yet run in this worktree; its shell syntax and YAML structure were checked on Linux.

## Prepare one isolated run

From `diagram-ports/` on the macOS host, with Xcode Command Line Tools, Homebrew, Python 3 plus Pillow, and Rust installed:

```sh
xcode-select -p
brew install pkgconf gd fontconfig expat
brew install --cask inkscape
python3 -c 'from PIL import Image; import PIL; print(PIL.__version__)'

mac_parity_work=$(mktemp -d "${TMPDIR:-/tmp}/ive-dot-macos.XXXXXX")
export PKG_CONFIG_PATH="$(brew --prefix gd)/lib/pkgconfig:$(brew --prefix expat)/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
pkg-config --modversion gdlib expat
brew list --versions pkgconf gd fontconfig expat > "$mac_parity_work/brew-versions.txt"
sw_vers > "$mac_parity_work/macos-version.txt"

python3 tools/dot-parity/prepare_macos_fonts.py --output "$mac_parity_work/fonts"
export FONTCONFIG_FILE="$mac_parity_work/fonts/fonts.conf"
fc-match -f '%{file}\n' Courier
fc-match -f '%{file}\n' Helvetica
fc-match -f '%{file}\n' Times-Roman
fc-match -f '%{file}\n' 'Nimbus Mono L'
fc-match -f '%{file}\n' 'Nimbus Sans L'
fc-match -f '%{file}\n' 'Nimbus Roman No9 L'

bash tools/dot-parity/build_macos_gd.sh "$mac_parity_work"
export DYLD_LIBRARY_PATH="$mac_parity_work/oracle/lib:$(brew --prefix)/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
```

The font script verifies all twelve bundled Liberation font hashes, copies them under the temporary directory, and writes an isolated fontconfig file. Check that all six `fc-match` paths resolve under `$mac_parity_work/fonts`; otherwise stop and fix fontconfig before pinning an oracle. The Rust desktop fallback renderer may still resolve fonts through its own platform font database; any resulting text differences are parity gaps to report, not reasons to change the oracle inventory. The build script checks the vendored source SHA, requires GD and Expat headers, and probes `-Tjson0`, HTML-table `-Tsvg`, and `-Tpng:gd:gd`. Build and install logs stay under `$mac_parity_work/build/`.

The command template assumes the Homebrew cask installed `/Applications/Inkscape.app`. Edit a copy of `tools/dot-parity/inkscape-command.macos.example.json` if the app lives elsewhere; retain `{input}` and `{output}`. Pin the *exact* command used:

```sh
cp tools/dot-parity/inkscape-command.macos.example.json "$mac_parity_work/inkscape-command.json"
python3 tools/dot-parity/pin_macos_oracles.py \
  --oracle "$mac_parity_work/oracle/bin/dot" \
  --rasterizer-command "$mac_parity_work/inkscape-command.json" \
  --output "$mac_parity_work/manifests"

cargo build --locked --release -p diagram-viewer
```

The manifests pin the Graphviz executable, GD plugin, Inkscape executable, resolvable transitive Mach-O libraries, private fontconfig file and selected font hashes, and macOS build. macOS shared-cache install names are tied to the recorded OS build because their individual files may not exist on disk. An unresolved `@rpath` dependency is an error; extend `DYLD_LIBRARY_PATH` with its actual directory and regenerate manifests rather than suppressing it. Both manifests start with `reviewed: false`.

## Compare and preserve evidence

Run from `diagram-ports/`. Keep the Graphviz binary at its installed path: its `@executable_path`/`@rpath` dependencies can fail if copied into a temporary command directory. `compare.py` now preserves a binary already named `dot` in place.

```sh
python3 tools/dot-parity/compare.py \
  --oracle "$mac_parity_work/oracle/bin/dot" \
  --candidate-command tools/dot-parity/candidate-command.example.json \
  --target-platform macos \
  --rasterizer-command "$mac_parity_work/inkscape-command.json" \
  --rasterizer-manifest "$mac_parity_work/manifests/inkscape-svg.json" \
  --native-gd-png \
  --native-gd-manifest "$mac_parity_work/manifests/graphviz-gd.json" \
  --font-manifest "$mac_parity_work/fonts/fonts-manifest.json" \
  --output "$mac_parity_work/seed-report"
```

The comparator exits 1 while mismatches remain, even when every command succeeds. Inspect `$mac_parity_work/seed-report/report.json`: each case records full JSON geometry and identity differences, raw SVG byte hashes, SVG raster dimensions and changed pixels, and candidate PNG versus native Graphviz GD dimensions and changed pixels. Count those fields independently; a case `status` alone combines several gates. Repeat with `--fixtures tools/dot-parity/fixtures-features`, `fixtures-extensions`, or `fixtures-arrow-ellipse` and a fresh output directory to extend coverage.

Keep the work directory until the report, both manifests, font inventory, build logs, `brew-versions.txt`, `macos-version.txt`, `python3 --version`, and Pillow version are archived together. The report records Python/Pillow and host metadata. A reviewer may set `reviewed: true` only after auditing that exact platform identity. Linux or WASM manifests cannot certify macOS, and this macOS host does not establish native WASM Graphviz parity.

After preserving the evidence elsewhere, remove only the marked temporary directory:

```sh
bash tools/dot-parity/cleanup_macos.sh "$mac_parity_work"
```

The cleanup script refuses paths outside the host's temporary directory or without this harness's ownership marker. No system or user fonts are modified.
