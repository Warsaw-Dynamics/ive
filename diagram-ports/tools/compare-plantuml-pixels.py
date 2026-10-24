#!/usr/bin/env python3
"""Compare original PlantUML and native CLI PNG pixels for local source files."""

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile

from PIL import Image, ImageChops


def run(command, *, input_bytes=None, environment=None):
    return subprocess.run(
        command, input=input_bytes, capture_output=True, timeout=20,
        env=environment, check=False,
    )


def compare(reference, native):
    with Image.open(io.BytesIO(reference)) as source, Image.open(native) as result:
        original = source.convert("RGBA")
        actual = result.convert("RGBA")
    size = (max(original.width, actual.width), max(original.height, actual.height))
    before = Image.new("RGBA", size, "white")
    after = Image.new("RGBA", size, "white")
    before.paste(original, (0, 0))
    after.paste(actual, (0, 0))
    difference = ImageChops.difference(before, after).convert("RGB")
    changed = sum(1 for pixel in difference.getdata() if pixel != (0, 0, 0))
    return {
        "original_size": list(original.size), "native_size": list(actual.size),
        "canvas_size": list(size), "different_pixels": changed,
        "canvas_pixels": size[0] * size[1], "exact": changed == 0 and original.size == actual.size,
    }


def render_case(source, options, directory):
    data = source.read_bytes()
    record = {"source": str(source), "sha256": hashlib.sha256(data).hexdigest()}
    if len(data) > 262144:
        return {**record, "status": "oversize"}
    environment = {**os.environ, "GRAPHVIZ_DOT": str(options.dot)}
    try:
        validated = run(["java", "-jar", str(options.jar), "-checkonly", str(source)], environment=environment)
        if validated.returncode:
            return {**record, "status": "original-rejected", "detail": validated.stderr.decode(errors="replace")[:500]}
        reference = run(["java", "-jar", str(options.jar), "-pipe", "-tpng"], input_bytes=data, environment=environment)
        if reference.returncode or not reference.stdout.startswith(b"\x89PNG\r\n\x1a\n"):
            return {**record, "status": "original-error", "detail": reference.stderr.decode(errors="replace")[:500]}
        native = directory / "native.png"
        result = run([str(options.viewer), "--language", "plantuml", "--render", str(source), "--output", str(native)])
        if result.returncode:
            return {**record, "status": "native-rejected", "detail": result.stderr.decode(errors="replace")[:500]}
        return {**record, "status": "compared", **compare(reference.stdout, native)}
    except (subprocess.TimeoutExpired, OSError, ValueError) as error:
        return {**record, "status": "error", "detail": str(error)[:500]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--dot", type=Path, required=True)
    parser.add_argument("--viewer", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("sources", nargs="+", type=Path)
    options = parser.parse_args()
    if len(options.sources) > 100:
        parser.error("at most 100 source files are allowed")
    options.output.mkdir(parents=True, exist_ok=True)
    rows = []
    for index, source in enumerate(options.sources):
        with tempfile.TemporaryDirectory(prefix=f"case-{index:03d}-", dir=options.output) as temporary:
            rows.append(render_case(source, options, Path(temporary)))
    report = {"schema": 1, "original_jar": str(options.jar), "graphviz_dot": str(options.dot),
              "viewer": str(options.viewer), "cases": rows}
    (options.output / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    counts = {status: sum(row["status"] == status for row in rows) for status in sorted({row["status"] for row in rows})}
    print(json.dumps(counts, sort_keys=True))
    for row in rows:
        if row["status"] == "compared":
            print(f"{row['source']}: {row['original_size']} vs {row['native_size']}; {row['different_pixels']}/{row['canvas_pixels']} pixels differ")


if __name__ == "__main__":
    main()
