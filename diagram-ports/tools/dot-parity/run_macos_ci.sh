#!/usr/bin/env bash
# Run the developer-only Graphviz 16.1.0 parity gate on a clean macOS host.
set -euo pipefail

if [[ $# -ne 1 || $(uname -s) != Darwin ]]; then
  echo "usage: run_macos_ci.sh FRESH_WORK_DIRECTORY (on macOS)" >&2
  exit 2
fi
script_dir=$(cd -- "$(dirname -- "$0")" && pwd -P)
project_dir=$(cd -- "$script_dir/../.." && pwd -P)
mkdir -p -- "$1"
work_dir=$(cd -- "$1" && pwd -P)
evidence="$work_dir/evidence"
mkdir -p -- "$evidence/reports" "$evidence/manifests"
cd -- "$project_dir"

export PKG_CONFIG_PATH="$(brew --prefix gd)/lib/pkgconfig:$(brew --prefix expat)/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
python3 "$script_dir/prepare_macos_fonts.py" --output "$evidence/fonts"
export FONTCONFIG_FILE="$evidence/fonts/fonts.conf"
python3 - "$FONTCONFIG_FILE" "$evidence/fonts/fonts-manifest.json" <<'PY_FONTS'
import hashlib
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET

config = Path(sys.argv[1])
manifest_path = Path(sys.argv[2])
tree = ET.parse(config)
root = tree.getroot()
for alias in root.findall("alias"):
    root.remove(alias)
for source, family in (
    ("Liberation Serif", "Nimbus Roman No9 L"),
    ("Liberation Mono", "Nimbus Mono L"),
    ("Liberation Sans", "Nimbus Sans L"),
):
    match = ET.SubElement(root, "match", target="scan")
    ET.SubElement(ET.SubElement(match, "test", name="family", compare="eq"), "string").text = source
    ET.SubElement(ET.SubElement(match, "edit", name="family", mode="assign"), "string").text = family
for requested, family in (
    ("Times", "Nimbus Roman No9 L"),
    ("Times-Roman", "Nimbus Roman No9 L"),
    ("Courier", "Nimbus Mono L"),
    ("Helvetica", "Nimbus Sans L"),
):
    alias = ET.SubElement(root, "alias", binding="strong")
    ET.SubElement(alias, "family").text = requested
    ET.SubElement(ET.SubElement(alias, "prefer"), "family").text = family
tree.write(config, encoding="utf-8", xml_declaration=True)
manifest = json.loads(manifest_path.read_text())
manifest["fontconfig_sha256"] = hashlib.sha256(config.read_bytes()).hexdigest()
manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
PY_FONTS
fc-cache -f "$evidence/fonts/fonts"
for entry in 'Times:Nimbus Roman No9 L' 'Times-Roman:Nimbus Roman No9 L' 'Courier:Nimbus Mono L' 'Helvetica:Nimbus Sans L'; do
  family=${entry%%:*}
  expected=${entry#*:}
  match=$(fc-match -f '%{family}|%{file}' "$family")
  case "$match" in
    "$expected|$evidence/fonts/"*) ;;
    *) echo "font $family resolved outside pinned $expected family: $match" >&2; exit 2 ;;
  esac
done

bash "$script_dir/build_macos_gd.sh" "$work_dir"
export DYLD_LIBRARY_PATH="$work_dir/oracle/lib:$(brew --prefix)/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
cp "$script_dir/inkscape-command.macos.example.json" "$evidence/inkscape-command.json"
python3 "$script_dir/pin_macos_oracles.py" \
  --oracle "$work_dir/oracle/bin/dot" \
  --rasterizer-command "$evidence/inkscape-command.json" \
  --output "$evidence/manifests"

cargo build --locked --release -p diagram-viewer
shasum -a 256 "$work_dir/oracle/bin/dot" target/release/diagram-viewer > "$evidence/binary-sha256.txt"
python3 -c 'from PIL import Image; import PIL; print(PIL.__version__)' > "$evidence/pillow-version.txt"
python3 --version > "$evidence/python-version.txt"
rustc -Vv > "$evidence/rust-version.txt"
sw_vers > "$evidence/macos-version.txt"
brew list --versions pkgconf gd fontconfig expat > "$evidence/brew-versions.txt"

mismatches=0
for corpus in seed features extensions arrow-ellipse; do
  case "$corpus" in
    seed) fixtures="$script_dir/fixtures" ;;
    features) fixtures="$script_dir/fixtures-features" ;;
    extensions) fixtures="$script_dir/fixtures-extensions" ;;
    arrow-ellipse) fixtures="$script_dir/fixtures-arrow-ellipse" ;;
  esac
  output="$evidence/reports/$corpus"
  mkdir -p -- "$output"
  status=0
  python3 "$script_dir/compare.py" \
    --oracle "$work_dir/oracle/bin/dot" \
    --candidate-command "$script_dir/candidate-command.example.json" \
    --target-platform macos \
    --fixtures "$fixtures" \
    --rasterizer-command "$evidence/inkscape-command.json" \
    --rasterizer-manifest "$evidence/manifests/inkscape-svg.json" \
    --native-gd-png \
    --native-gd-manifest "$evidence/manifests/graphviz-gd.json" \
    --font-manifest "$evidence/fonts/fonts-manifest.json" \
    --output "$output" || status=$?
  printf '%s\n' "$status" > "$output/compare-exit-code.txt"
  if [[ ! -s "$output/report.json" ]]; then
    echo "macOS DOT parity $corpus failed before writing report.json (exit $status)" >&2
    exit 2
  fi
  if [[ $status -ne 0 ]]; then
    mismatches=1
  fi
done

python3 - "$evidence/reports" "$evidence/summary.json" <<'PY_SUMMARY'
from collections import Counter
import json
from pathlib import Path
import sys
reports = Path(sys.argv[1])
summary = {}
for report in sorted(reports.glob('*/report.json')):
    cases = json.loads(report.read_text())['cases']
    summary[report.parent.name] = {
        'status': dict(Counter(case['status'] for case in cases)),
        'json_geometry_exact': sum(case.get('geometry', {}).get('exact') is True for case in cases),
        'svg_raster_exact': sum(case.get('svg', {}).get('raster', {}).get('exact') is True for case in cases),
        'native_gd_png_exact': sum(case.get('png', {}).get('native_graphviz', {}).get('exact') is True for case in cases),
        'cases': len(cases),
    }
Path(sys.argv[2]).write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2))
PY_SUMMARY

if [[ $mismatches -ne 0 ]]; then
  echo "macOS DOT parity has mismatches; review uploaded reports" >&2
  exit 1
fi
