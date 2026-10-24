#!/usr/bin/env python3
"""Compare a Rust DOT renderer with pinned Graphviz 16.1.0 on bounded fixtures."""

import argparse
from collections import Counter
from dataclasses import dataclass
from decimal import Decimal
import hashlib
import json
import locale
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile

from PIL import Image, ImageChops, __version__ as pillow_version
from gd_oracle import verified_identity
from svg_raster_oracle import verified_identity as verified_svg_raster_identity

MAX_SOURCE = 256 * 1024
MAX_OUTPUT = 16 * 1024 * 1024
MAX_CASES = 100
MAX_PIXELS = 16_777_216
NUMBER = re.compile(r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command_output(command, timeout=10):
    return subprocess.run(command, capture_output=True, check=False, timeout=timeout)


def font_metadata():
    matches = {}
    for family in ("sans", "serif", "monospace"):
        try:
            result = command_output(["fc-match", "-f", "%{family}|%{file}\n", family], 3)
        except (OSError, subprocess.TimeoutExpired):
            return {"source": "fontconfig unavailable", "matches": matches}
        if result.returncode == 0:
            matches[family] = result.stdout.decode(errors="replace").strip()
    return {"source": "fontconfig", "matches": matches}


def platform_metadata():
    font_directory = Path(__file__).resolve().parents[2] / "web" / "fonts"
    bundled = {path.name: digest(path) for path in sorted(font_directory.glob("*.ttf"))}
    return {
        "system": platform.system(), "release": platform.release(),
        "machine": platform.machine(), "python": platform.python_version(),
        "pillow": pillow_version,
        "locale": locale.setlocale(locale.LC_CTYPE), "timezone": os.environ.get("TZ", "system default"),
        "fonts": font_metadata(), "bundled_wasm_font_sha256": bundled,
    }


def wasm_metadata(path, fonts):
    if path is None:
        return None
    size = path.stat().st_size
    with path.open("rb") as artifact:
        header = artifact.read(8)
    if size < 8 or size > 64 * 1024 * 1024 or header != b"\x00asm\x01\x00\x00\x00":
        raise ValueError(f"WASM artifact {path} lacks a version-1 header or exceeds 64 MiB")
    version = command_output(["node", "--version"], 5)
    if version.returncode:
        raise ValueError("Node version could not be identified for WASM parity run")
    node = shutil.which("node")
    if node is None:
        raise ValueError("Node executable was not found for WASM parity run")
    return {"artifact": str(path), "sha256": digest(path), "bytes": size,
            "target": "wasm32-unknown-unknown", "node_version": version.stdout.decode().strip(),
            "node_binary": node, "node_sha256": digest(Path(node)),
            "bundled_font_sha256": fonts, "native_graphviz_wasm_reference": False}


def verify_wasm_command(command, artifact):
    if command is None:
        raise ValueError("WASM parity requires a candidate command")
    locations = [Path(command[index + 1]).resolve() for index, part in enumerate(command[:-1])
                 if part == "--wasm"]
    if locations != [artifact.resolve()]:
        raise ValueError(f"candidate command must pass the pinned WASM artifact {artifact} exactly once")


def checked_run(command, output, timeout):
    result = command_output(command, timeout)
    if result.returncode:
        raise RuntimeError(f"command {command[0]!r} exited {result.returncode}: {result.stderr.decode(errors='replace')[:1000]}")
    if not output.is_file() or output.stat().st_size > MAX_OUTPUT:
        raise RuntimeError(f"output {output} is missing or exceeds {MAX_OUTPUT} bytes")


def load_command(path):
    command = json.loads(path.read_text())
    if not isinstance(command, list) or not command or not all(isinstance(item, str) for item in command):
        raise ValueError(f"command file {path} must contain a nonempty JSON string array")
    if not any("{input}" in item for item in command) or not any("{output}" in item for item in command):
        raise ValueError(f"command file {path} must use {{input}} and {{output}} placeholders")
    return command


def expand(command, source, output, format_name):
    return [part.replace("{input}", str(source)).replace("{output}", str(output)).replace("{format}", format_name) for part in command]


def decimal(value):
    parsed = Decimal(str(value))
    if not parsed.is_finite():
        raise ValueError(f"nonfinite geometry value {value!r}")
    return "0" if parsed == 0 else str(parsed.normalize())


def coordinates(text):
    return [Decimal(part) for part in NUMBER.findall(str(text))]


def point(value, bounds=None):
    values = coordinates(value) if isinstance(value, str) else [Decimal(str(item)) for item in value]
    if len(values) != 2:
        raise ValueError(f"invalid geometry point {value!r}")
    if bounds:
        x0, _, _, y1 = bounds
        return [decimal(values[0] - x0), decimal(y1 - values[1])]
    return [decimal(item) for item in values]


def rectangle(value, bounds):
    parts = coordinates(value)
    if len(parts) != 4:
        raise ValueError(f"invalid geometry rectangle {value!r}")
    x0, _, _, y1 = bounds
    return [decimal(parts[0] - x0), decimal(y1 - parts[3]),
            decimal(parts[2] - x0), decimal(y1 - parts[1])]


def route(value, bounds):
    points = []
    for segment in value.split(";"):
        endpoint = None
        for token in segment.split():
            if token.startswith("e,"):
                endpoint = point(token[2:], bounds)
            elif token.startswith("s,"):
                points.append(point(token[2:], bounds))
            else:
                points.append(point(token, bounds))
        if endpoint:
            points.append(endpoint)
    return points


def normalize_oracle(document):
    bounds = coordinates(document["bb"])
    if len(bounds) != 4:
        raise ValueError("Graphviz JSON has an invalid graph bb")
    objects = {}
    names = {}
    for item in document.get("objects", []):
        name = item.get("name")
        if name is None:
            continue
        names[item.get("_gvid")] = name
        entry = {"name": name}
        for key in ("pos", "lp", "width", "height", "bb"):
            if key in item:
                entry[key] = (point(item[key], bounds) if key in ("pos", "lp") else
                              rectangle(item[key], bounds) if key == "bb" else decimal(item[key]))
        objects[name] = entry
    edges = {}
    for index, edge in enumerate(document.get("edges", [])):
        key = str(edge.get("id", f"e{index}"))
        entry = {"id": key, "tail": names.get(edge.get("tail"), edge.get("tail")),
                 "head": names.get(edge.get("head"), edge.get("head"))}
        if "pos" in edge:
            entry["points"] = route(edge["pos"], bounds)
            if ";" in edge["pos"]:
                entry["segments"] = edge["pos"].count(";") + 1
        for label in ("lp", "head_lp", "tail_lp"):
            if label in edge:
                entry[label] = point(edge[label], bounds)
        edges[key] = entry
    return {"name": document.get("name", ""), "directed": document.get("directed"),
            "strict": document.get("strict"), "width": decimal(bounds[2] - bounds[0]),
            "height": decimal(bounds[3] - bounds[1]), "objects": objects, "edges": edges}


def normalize_candidate(document):
    objects = {}
    for item in document.get("objects", []):
        name = item["name"]
        entry = {"name": name}
        for key in ("pos", "lp", "width", "height", "bb"):
            if key in item:
                entry[key] = (point(item[key]) if key in ("pos", "lp") else
                              [decimal(value) for value in coordinates(item[key])] if key == "bb" else decimal(item[key]))
        objects[name] = entry
    edges = {}
    for index, edge in enumerate(document.get("edges", [])):
        key = str(edge.get("id", f"e{index}"))
        entry = {"id": key, "tail": edge.get("tail"), "head": edge.get("head")}
        if "points" in edge:
            entry["points"] = [point(value) for value in edge["points"]]
        for label in ("lp", "head_lp", "tail_lp"):
            if label in edge:
                entry[label] = point(edge[label])
        edges[key] = entry
    return {"name": document.get("name", ""), "directed": document.get("directed"),
            "strict": document.get("strict"), "width": decimal(document.get("width", 0)),
            "height": decimal(document.get("height", 0)), "objects": objects, "edges": edges}


def compare_fields(path, expected, actual, differences):
    for field in sorted(expected.keys() | actual.keys()):
        before = expected.get(field)
        after = actual.get(field)
        if before != after:
            differences.append({"path": f"{path}.{field}", "oracle": before, "candidate": after})


def compare_collection(name, expected, actual, differences):
    for key in sorted(expected.keys() | actual.keys()):
        if key not in expected or key not in actual:
            differences.append({"path": f"{name}.{key}", "oracle": key in expected, "candidate": key in actual})
            continue
        compare_fields(f"{name}.{key}", expected[key], actual[key], differences)


def compare_geometry(oracle, candidate):
    before = normalize_oracle(oracle) if "bb" in oracle else normalize_candidate(oracle)
    after = normalize_oracle(candidate) if "bb" in candidate else normalize_candidate(candidate)
    differences = []
    compare_fields("graph", {key: value for key, value in before.items() if key not in ("objects", "edges")},
                   {key: value for key, value in after.items() if key not in ("objects", "edges")}, differences)
    compare_collection("objects", before["objects"], after["objects"], differences)
    compare_collection("edges", before["edges"], after["edges"], differences)
    return {"exact": not differences, "difference_count": len(differences), "differences": differences[:500]}


def read_png(path):
    with Image.open(path) as image:
        if image.width * image.height > MAX_PIXELS or image.width > 4096 or image.height > 4096:
            raise ValueError(f"PNG {path} exceeds raster bounds: {image.width}x{image.height}")
        return image.convert("RGBA")


def compare_png(oracle, candidate):
    before = read_png(oracle)
    after = read_png(candidate)
    size = (max(before.width, after.width), max(before.height, after.height))
    left = Image.new("RGBA", size)
    right = Image.new("RGBA", size)
    left.paste(before)
    right.paste(after)
    difference = ImageChops.difference(left, right)
    changed = sum(pixel != (0, 0, 0, 0) for pixel in difference.getdata())
    return {"oracle_size": [before.width, before.height], "candidate_size": [after.width, after.height],
            "different_pixels": changed, "canvas_pixels": size[0] * size[1],
            "exact": changed == 0 and before.size == after.size}


def rasterize(command, source, output, timeout):
    checked_run(expand(command, source, output, "png"), output, timeout)


def render_oracle(binary, source, folder, timeout, rasterizer, native_png):
    outputs = {}
    for format_name, extension in (("json0", "json"), ("svg", "svg")):
        output = folder / f"oracle.{extension}"
        checked_run([str(binary), f"-T{format_name}", "-o", str(output), str(source)], output, timeout)
        outputs[extension] = output
    if rasterizer:
        outputs["png"] = folder / "oracle.png"
        rasterize(rasterizer, outputs["svg"], outputs["png"], timeout)
    if native_png:
        outputs["native_png"] = folder / "oracle-native.png"
        checked_run([str(binary), "-Tpng:gd:gd", "-o", str(outputs["native_png"]), str(source)],
                    outputs["native_png"], timeout)
    return outputs


def render_candidate(command, source, folder, timeout, rasterizer, formats):
    outputs = {}
    for extension in formats:
        output = folder / f"candidate.{extension}"
        checked_run(expand(command, source, output, extension), output, timeout)
        outputs[extension] = output
    if rasterizer:
        outputs["svg_png"] = folder / "candidate-svg.png"
        rasterize(rasterizer, outputs["svg"], outputs["svg_png"], timeout)
    return outputs


def compare_case(name, oracle, candidate, certified):
    geometry = compare_geometry(json.loads(oracle["json"].read_text()), json.loads(candidate["json"].read_text()))
    svg = {"exact_bytes": oracle["svg"].read_bytes() == candidate["svg"].read_bytes(),
           "oracle_sha256": digest(oracle["svg"]), "candidate_sha256": digest(candidate["svg"])}
    native = compare_png(oracle["native_png"], candidate["png"]) if "native_png" in oracle and "png" in candidate else None
    if "png" not in oracle:
        status = "incomplete" if geometry["exact"] and (native is None or native["exact"]) else "mismatch"
        return {"name": name, "status": status, "geometry": geometry, "svg": svg,
                "png": {"available": native is not None, "native_graphviz": native,
                        "reason": "no shared SVG rasterizer configured"}}
    svg["raster"] = compare_png(oracle["png"], candidate["svg_png"])
    png = compare_png(oracle["png"], candidate["png"]) if "png" in candidate else {"available": False}
    mismatch = not geometry["exact"] or not svg["raster"]["exact"] or ("png" in candidate and not png["exact"])
    status = "mismatch" if mismatch else "incomplete" if "png" not in candidate else "exact" if certified else "diagnostic_match"
    return {"name": name, "status": status,
            "geometry": geometry, "png": {**png, "native_graphviz": native}, "svg": svg}


def prepare_oracle(path, folder):
    binary = path if path.name == "dot" else folder / "dot"
    if binary != path:
        shutil.copy2(path, binary)
        binary.chmod(binary.stat().st_mode | 0o100)
    version = command_output([str(binary), "-V"], 5)
    text = (version.stdout + version.stderr).decode(errors="replace").strip()
    if version.returncode or "graphviz version 16.1.0" not in text:
        raise ValueError(f"oracle must be Graphviz 16.1.0; received {text!r}")
    return binary, {"version": text, "sha256": digest(path), "source_path": str(path)}


def fixture_paths(directory):
    paths = sorted(directory.glob("*.dot"))
    if not paths or len(paths) > MAX_CASES:
        raise ValueError(f"fixture count {len(paths)} must be 1..{MAX_CASES}")
    for path in paths:
        if not path.is_file() or path.stat().st_size > MAX_SOURCE:
            raise ValueError(f"fixture {path} exceeds {MAX_SOURCE} bytes or is not a regular file")
    return paths


def fixture_provenance(path, fixtures):
    if path is None:
        return {}
    manifest = json.loads(path.read_text())
    cases = {item["fixture"]: item for item in manifest["cases"]}
    expected = {fixture.name for fixture in fixtures}
    if set(cases) != expected:
        raise ValueError(f"provenance {path} fixture names differ from DOT inputs")
    for fixture in fixtures:
        case = cases[fixture.name]
        if case["dot_sha256"] != digest(fixture):
            raise ValueError(f"provenance {path} has stale hash for {fixture.name}")
        if "source" in case:
            relative = Path(case["source"])
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError(f"provenance {path} has invalid source path {relative}")
            source = Path(__file__).resolve().parents[2] / relative
            if case["source_sha256"] != digest(source):
                raise ValueError(f"provenance {path} has stale source hash for {fixture.name}")
    return cases


def oracle_assertions(directory, fixtures):
    path = directory / "oracle-assertions.json"
    if not path.is_file():
        return {}
    assertions = json.loads(path.read_text())
    if set(assertions) != {fixture.name for fixture in fixtures}:
        raise ValueError(f"oracle assertions {path} must name every DOT fixture exactly once")
    if any(not isinstance(markers, list) or not markers or
           not all(isinstance(marker, str) and marker for marker in markers)
           for markers in assertions.values()):
        raise ValueError(f"oracle assertions {path} must contain nonempty string marker lists")
    return assertions


def check_oracle_svg(path, markers, fixture):
    svg = path.read_text()
    for marker in markers:
        if marker not in svg:
            raise ValueError(f"oracle SVG for {fixture} lacks required capability marker {marker!r}")


def run_case(path, settings, oracle, candidate, rasterizer, certified, native_png, formats, assertions):
    folder = settings.output / path.stem
    folder.mkdir(parents=True, exist_ok=True)
    expected = {}
    try:
        expected = render_oracle(oracle, path, folder, settings.timeout, rasterizer, native_png)
        check_oracle_svg(expected["svg"], assertions.get(path.name, ()), path.name)
        if candidate is None:
            return {"name": path.name, "status": "oracle_ready", "source_sha256": digest(path),
                    "oracle": {kind: digest(file) for kind, file in expected.items()}}
        actual = render_candidate(candidate, path, folder, settings.timeout, rasterizer, formats)
        return {"source_sha256": digest(path), **compare_case(path.name, expected, actual, certified)}
    except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired, json.JSONDecodeError) as error:
        native_size = list(read_png(expected["native_png"]).size) if "native_png" in expected else None
        svg_raster_size = list(read_png(expected["png"]).size) if "png" in expected else None
        return {"name": path.name, "status": "error", "detail": str(error)[:1200],
                "oracle_native_png_size": native_size, "oracle_svg_raster_size": svg_raster_size}


@dataclass(frozen=True)
class Settings:
    output: Path
    timeout: int


def rasterizer_identity(command, manifest_path, target_platform):
    if command is None:
        return {"available": False, "certified": False}
    if manifest_path is None:
        executable = shutil.which(command[0])
        return {"available": executable is not None, "command": command,
                "executable": executable, "certified": False}
    return {"available": True, **verified_svg_raster_identity(command, manifest_path, target_platform)}


def wasm_reference_identity(path):
    if path is None:
        return None
    identity = json.loads(path.read_text())
    if (identity.get("kind") != "third-party-prebuilt-graphviz-wasm" or
        identity.get("reviewed") is not False or
        identity.get("source_provenance_reviewed") is not False or
        identity.get("vendor_source_equivalence_verified") is not False):
        raise ValueError(f"WASM reference manifest {path} must identify an unreviewed prebuilt package")
    for file_key, hash_key in (("package_tarball_path", "package_tarball_sha256"),
                               ("bundle_path", "bundle_sha256"), ("runner_path", "runner_sha256")):
        file = Path(identity[file_key])
        if digest(file) != identity[hash_key]:
            raise ValueError(f"WASM reference {file_key} {file} differs from manifest {path}")
    node = identity["node"]
    actual = command_output(["node", "-p", "process.execPath"], 5)
    binary = Path(actual.stdout.decode().strip()).resolve()
    if actual.returncode or str(binary) != node["path"] or digest(binary) != node["sha256"]:
        raise ValueError(f"WASM reference Node binary differs from manifest {path}")
    version = command_output(["node", "--version"], 5)
    if version.returncode or version.stdout.decode().strip() != node["version"]:
        raise ValueError(f"WASM reference Node version differs from manifest {path}")
    return identity


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--oracle", type=Path, required=True, help="pinned Graphviz 16.1.0 executable")
    parser.add_argument("--oracle-metadata", type=Path,
                        help="unreviewed pinned third-party WASM Graphviz reference identity")
    parser.add_argument("--candidate-command", type=Path, help="JSON string array with {input} and {output}")
    parser.add_argument("--candidate-wasm", type=Path, help="built browser WASM artifact; records hash, target and bundled fonts")
    parser.add_argument("--candidate-svg-only", action="store_true",
                        help="compare JSON/SVG when candidate native PNG export is unavailable")
    parser.add_argument("--rasterizer-command", type=Path, help="JSON command to rasterize both SVG files")
    parser.add_argument("--rasterizer-manifest", type=Path, help="pin-matched renderer manifest required to certify PNG parity")
    parser.add_argument("--native-gd-png", action="store_true", help="render native Graphviz 16.1.0 PNG via GD plugin")
    parser.add_argument("--native-gd-manifest", type=Path,
                        help="Graphviz GD oracle binary, library and font identity manifest")
    parser.add_argument("--fixtures", type=Path, default=Path(__file__).with_name("fixtures"))
    parser.add_argument("--provenance-manifest", type=Path,
                        help="generator provenance JSON; all fixture names and hashes must match")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=10)
    parser.add_argument("--target-platform", choices=("linux", "macos", "wasm"),
                        default="macos" if platform.system() == "Darwin" else "linux")
    parser.add_argument("--font-manifest", type=Path, help="platform-specific pinned font inventory JSON")
    args = parser.parse_args()
    if args.timeout < 1 or args.timeout > 30:
        parser.error("timeout must be in 1..30 seconds")
    args.output.mkdir(parents=True, exist_ok=True)
    fixtures = fixture_paths(args.fixtures)
    assertions = oracle_assertions(args.fixtures, fixtures)
    provenance_path = args.provenance_manifest
    if provenance_path is None and (args.fixtures / "provenance.json").is_file():
        provenance_path = args.fixtures / "provenance.json"
    provenance = fixture_provenance(provenance_path, fixtures)
    candidate = load_command(args.candidate_command) if args.candidate_command else None
    if args.target_platform == "wasm" and args.candidate_wasm is None:
        parser.error("--target-platform wasm requires --candidate-wasm for artifact and font provenance")
    if args.candidate_wasm is not None:
        verify_wasm_command(candidate, args.candidate_wasm)
    candidate_formats = ("json", "svg") if args.candidate_svg_only else ("json", "svg", "png")
    rasterizer = load_command(args.rasterizer_command) if args.rasterizer_command else None
    rasterizer_info = rasterizer_identity(rasterizer, args.rasterizer_manifest, args.target_platform)
    if args.native_gd_manifest and not args.native_gd_png:
        parser.error("--native-gd-manifest requires --native-gd-png")
    if args.oracle_metadata and args.native_gd_png:
        parser.error("prebuilt WASM Graphviz reference has no native GD PNG plugin")
    oracle_wasm_reference = wasm_reference_identity(args.oracle_metadata)
    if oracle_wasm_reference and args.oracle.resolve() != Path(oracle_wasm_reference["runner_path"]).resolve():
        parser.error("--oracle must be the runner pinned by --oracle-metadata")
    native_info = verified_identity(args.oracle, args.native_gd_manifest) if args.native_gd_png else None
    certified = oracle_wasm_reference is None and rasterizer_info["certified"] and (native_info is None or native_info["certified"])
    font_manifest = json.loads(args.font_manifest.read_text()) if args.font_manifest else None
    host = platform_metadata()
    candidate_wasm = wasm_metadata(args.candidate_wasm, host["bundled_wasm_font_sha256"])
    with tempfile.TemporaryDirectory(prefix="dot-parity-") as temporary:
        oracle, identity = prepare_oracle(args.oracle, Path(temporary))
        settings = Settings(args.output, args.timeout)
        cases = [{**run_case(path, settings, oracle, candidate, rasterizer,
                             certified, args.native_gd_png, candidate_formats, assertions),
                  **({"provenance": provenance[path.name]} if provenance else {})}
                 for path in fixtures]
    report = {"schema": 1, "oracle": identity, "candidate_command": candidate,
              "candidate_target": args.target_platform, "rasterizer": rasterizer_info,
              "font_manifest": font_manifest, "platform": host,
              "candidate_wasm": candidate_wasm,
              "reference_kind": "unreviewed third-party prebuilt Graphviz 16.1.0 WASM in Node; SVG raster is host-rendered"
                  if oracle_wasm_reference else
                  (f"{host['system']}-hosted Graphviz 16.1.0; not a native WASM Graphviz reference"
                   if args.target_platform == "wasm" else "host Graphviz 16.1.0"),
              "oracle_wasm_reference": oracle_wasm_reference,
              "native_gd_png_oracle": native_info,
              "provenance_manifest": str(provenance_path) if provenance_path else None,
              "oracle_assertions": assertions,
              "cases": cases}
    (args.output / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    counts = Counter(item["status"] for item in cases)
    print(json.dumps(dict(sorted(counts.items()))))
    return 0 if all(item["status"] in ("exact", "oracle_ready") for item in cases) else 1


if __name__ == "__main__":
    sys.exit(main())
