#!/usr/bin/env python3
"""Write a compact exact-pixel PNG difference report and color-coded mask."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

from PIL import Image

MAX_PIXELS = 4_194_304


def identity(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def color(pixel):
    red, green, blue, _ = pixel
    if (red, green, blue) == (255, 255, 255):
        return "white"
    if (red, green, blue) == (0, 0, 0):
        return "black"
    if blue > red and blue > green:
        return "blue"
    return "gray_or_other"


def analyze(oracle_path, candidate_path, output):
    with Image.open(oracle_path) as image:
        oracle_mode, oracle_size = image.mode, image.size
        oracle = image.convert("RGBA")
    with Image.open(candidate_path) as image:
        candidate_mode, candidate_size = image.mode, image.size
        candidate = image.convert("RGBA")
    width = max(oracle.width, candidate.width)
    height = max(oracle.height, candidate.height)
    if width * height > MAX_PIXELS:
        raise ValueError(f"PNG comparison canvas {width}x{height} exceeds {MAX_PIXELS} pixels")
    mask = Image.new("RGB", (width, height), "white")
    categories = Counter()
    changed = 0
    bbox = [width, height, 0, 0]
    for y in range(height):
        for x in range(width):
            before = oracle.getpixel((x, y)) if x < oracle.width and y < oracle.height else (0, 0, 0, 0)
            after = candidate.getpixel((x, y)) if x < candidate.width and y < candidate.height else (0, 0, 0, 0)
            if before == after:
                continue
            changed += 1
            bbox = [min(bbox[0], x), min(bbox[1], y), max(bbox[2], x + 1), max(bbox[3], y + 1)]
            if x >= candidate.width or y >= candidate.height:
                categories["outside_candidate_canvas"] += 1
                mask.putpixel((x, y), (255, 128, 0))
                continue
            if x >= oracle.width or y >= oracle.height:
                categories["outside_oracle_canvas"] += 1
                mask.putpixel((x, y), (0, 128, 255))
                continue
            first, second = color(before), color(after)
            categories[f"{first}_to_{second}"] += 1
            mask.putpixel((x, y), (255, 0, 0) if second == "white" else
                          (0, 80, 255) if first == "white" else (160, 0, 160))
    summary = {"oracle_sha256": identity(oracle_path), "candidate_sha256": identity(candidate_path),
               "oracle_mode": oracle_mode, "candidate_mode": candidate_mode,
               "oracle_size": list(oracle_size), "candidate_size": list(candidate_size),
               "different_pixels": changed, "difference_bbox": bbox if changed else None,
               "categories": dict(sorted(categories.items())),
               "oracle_partial_alpha_pixels": sum(pixel[3] < 255 for pixel in oracle.getdata()),
               "candidate_partial_alpha_pixels": sum(pixel[3] < 255 for pixel in candidate.getdata())}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.with_suffix(".json").write_text(json.dumps(summary, indent=2) + "\n")
    mask.save(output.with_name(output.name + "-mask.png"))
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="report prefix without extension")
    args = parser.parse_args()
    summary = analyze(args.oracle, args.candidate, args.output)
    print(json.dumps({"different_pixels": summary["different_pixels"],
                      "categories": summary["categories"]}))


if __name__ == "__main__":
    main()
