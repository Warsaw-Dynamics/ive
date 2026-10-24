#!/usr/bin/env python3
"""Write an unreviewed Linux Inkscape SVG raster oracle manifest."""

import argparse
import json
from pathlib import Path

from svg_raster_oracle import identify


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--command", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--candidate-target", choices=("linux", "wasm"), default="linux")
    args = parser.parse_args()
    command = json.loads(args.command.read_text())
    identity = identify(command, args.candidate_target)
    manifest = {**identity, "candidate_png_renderer": "Rust GD-compatible fast path with resvg fallback",
                "reviewed": False}
    args.output.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"wrote unreviewed SVG rasterizer identity to {args.output}")


if __name__ == "__main__":
    main()
