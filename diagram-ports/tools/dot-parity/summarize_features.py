#!/usr/bin/env python3
"""Summarize a bounded feature-corpus parity report by error and mismatch class."""

import argparse
from decimal import Decimal, InvalidOperation
import json
from pathlib import Path


def has_numeric_value(value):
    if type(value) in (int, float):
        return True
    if isinstance(value, (list, tuple)):
        return any(has_numeric_value(item) for item in value)
    if isinstance(value, str):
        try:
            return Decimal(value).is_finite()
        except InvalidOperation:
            return False
    return False


def is_numeric_difference(difference):
    return any(has_numeric_value(difference.get(side)) for side in ("oracle", "candidate"))


def classify(case):
    if case["status"] == "error":
        detail = case.get("detail", "")
        if case.get("oracle_svg_raster_size") is None:
            return ["oracle_error"]
        if "unsupported" in detail.lower() or "not supported" in detail.lower():
            return ["candidate_unsupported"]
        return ["candidate_error"]
    categories = []
    differences = case["geometry"]["differences"]
    if any(is_numeric_difference(item) for item in differences):
        categories.append("numeric_geometry")
    if any(not is_numeric_difference(item) for item in differences):
        categories.append("identity_or_structure")
    if case["svg"]["raster"]["exact"] is False:
        categories.append("svg_pixels")
    if case["png"]["exact"] is False:
        categories.append("png_pixels")
    return categories or ["all_exact"]


def summarize(report, coverage):
    cases = report["cases"]
    if {case["name"] for case in cases} != set(coverage):
        raise ValueError("feature report fixture names differ from feature-coverage.json")
    rows = [{"fixture": case["name"], "source_sha256": case.get("source_sha256"),
             "features": coverage[case["name"]],
             "categories": classify(case),
             "json_exact": case.get("geometry", {}).get("exact"),
             "svg_exact": case.get("svg", {}).get("raster", {}).get("exact"),
             "png_exact": case.get("png", {}).get("exact"),
             "geometry_fields": case.get("geometry", {}).get("difference_count"),
             "numeric_fields": sum(is_numeric_difference(item) for item in
                                   case.get("geometry", {}).get("differences", [])),
             "svg_pixels": case.get("svg", {}).get("raster", {}).get("different_pixels"),
             "png_pixels": case.get("png", {}).get("different_pixels"),
             "error": case.get("detail")}
            for case in cases]
    names = ("oracle_error", "candidate_unsupported", "candidate_error", "numeric_geometry",
             "identity_or_structure", "svg_pixels", "png_pixels", "all_exact")
    counts = {name: sum(name in row["categories"] for row in rows) for name in names}
    counts.update({"json_exact": sum(row["json_exact"] is True for row in rows),
                   "svg_exact": sum(row["svg_exact"] is True for row in rows),
                   "png_exact": sum(row["png_exact"] is True for row in rows)})
    return {"schema": 1, "scope": "separate diagnostic DOT feature corpus",
            "oracle_sha256": report.get("oracle", {}).get("sha256"),
            "candidate_target": report.get("candidate_target"),
            "reference_kind": report.get("reference_kind"),
            "rasterizer_reviewed": (report.get("rasterizer") or {}).get("reviewed"),
            "counts_are_overlapping": True, "fixture_count": len(rows), "counts": counts, "cases": rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    coverage_path = Path(__file__).with_name("fixtures-features") / "feature-coverage.json"
    report = json.loads(args.report.read_text())
    coverage = json.loads(coverage_path.read_text())
    summary = summarize(report, coverage)
    args.output.write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary["counts"], sort_keys=True))


if __name__ == "__main__":
    main()
