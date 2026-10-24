#!/usr/bin/env python3
"""Summarize full JSON identity and SVG pixels in a WASM reference report."""

import argparse
from collections import Counter
import json
from pathlib import Path


def summarize(directory):
    report = json.loads((directory / "report.json").read_text())
    if report.get("oracle_wasm_reference") is None:
        raise ValueError(f"report {directory / 'report.json'} has no pinned WASM reference")
    cases = []
    for item in report["cases"]:
        case = directory / Path(item["name"]).stem
        oracle = case / "oracle.json"
        candidate = case / "candidate.json"
        rendered = item["status"] != "error"
        cases.append({"name": item["name"], "status": item["status"],
                      "full_json_identity": rendered and oracle.is_file() and candidate.is_file()
                      and json.loads(oracle.read_text()) == json.loads(candidate.read_text()),
                      "geometry_exact": item.get("geometry", {}).get("exact") is True,
                      "svg_pixels_exact": item.get("svg", {}).get("raster", {}).get("exact") is True,
                      "svg_pixels_changed": item.get("svg", {}).get("raster", {}).get("different_pixels"),
                      "svg_oracle_size": item.get("svg", {}).get("raster", {}).get("oracle_size"),
                      "svg_candidate_size": item.get("svg", {}).get("raster", {}).get("candidate_size")})
    return {"schema": 1, "source_report": str(directory / "report.json"),
            "reference_reviewed": False, "cases": cases,
            "counts": {"total": len(cases), "status": dict(Counter(case["status"] for case in cases)),
                       "full_json_identity": sum(case["full_json_identity"] for case in cases),
                       "geometry_exact": sum(case["geometry_exact"] for case in cases),
                       "svg_pixels_exact": sum(case["svg_pixels_exact"] for case in cases)}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report_directory", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = summarize(args.report_directory)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result["counts"], sort_keys=True))


if __name__ == "__main__":
    main()
