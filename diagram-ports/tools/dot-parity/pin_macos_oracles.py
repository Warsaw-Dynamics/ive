#!/usr/bin/env python3
"""Write unreviewed macOS Graphviz GD and Inkscape identity manifests."""

import argparse
import json
from pathlib import Path
import platform

from gd_oracle import identify as identify_gd
from svg_raster_oracle import identify as identify_svg_raster


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--rasterizer-command", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("macOS oracle manifests require a macOS host")
    args.output.mkdir(parents=True, exist_ok=True)
    command = json.loads(args.rasterizer_command.read_text())
    gd = {**identify_gd(args.oracle), "reviewed": False}
    raster = {**identify_svg_raster(command, "macos"),
              "candidate_png_renderer": "Rust GD-compatible fast path with resvg fallback",
              "reviewed": False}
    (args.output / "graphviz-gd.json").write_text(json.dumps(gd, indent=2) + "\n")
    (args.output / "inkscape-svg.json").write_text(json.dumps(raster, indent=2) + "\n")
    print(f"wrote unreviewed macOS identities to {args.output}")


if __name__ == "__main__":
    main()
