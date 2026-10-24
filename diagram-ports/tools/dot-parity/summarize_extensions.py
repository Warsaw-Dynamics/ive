#!/usr/bin/env python3
"""Combine static SVG and native GD reports for the extension corpus."""

import argparse
import json
from pathlib import Path

from summarize_features import is_numeric_difference


def case_map(report, coverage):
    cases = {case["name"]: case for case in report["cases"]}
    if set(cases) != set(coverage):
        raise ValueError("extension report fixture names differ from coverage.json")
    return cases


def difference_counts(case):
    differences = case.get("geometry", {}).get("differences", [])
    numeric = sum(is_numeric_difference(item) for item in differences)
    return numeric, len(differences) - numeric


def error_kind(case):
    if case["status"] != "error":
        return None
    detail = case.get("detail", "").lower()
    if "unsupported" in detail or "not supported" in detail:
        return "unsupported"
    return "error"


def row(name, tags, static, gd):
    if static.get("source_sha256") != gd.get("source_sha256"):
        raise ValueError(f"source hash differs between static and GD reports for {name}")
    s_num, s_structure = difference_counts(static)
    g_num, g_structure = difference_counts(gd)
    svg = static.get("svg", {})
    raster = svg.get("raster", {})
    native = gd.get("png", {}).get("native_graphviz") or {}
    return {"fixture": name, "features": tags, "source_sha256": static.get("source_sha256"),
            "static_error": error_kind(static), "static_detail": static.get("detail"),
            "static_json_exact": static.get("geometry", {}).get("exact"),
            "static_json_numeric_fields": s_num, "static_json_structure_fields": s_structure,
            "static_svg_raw_bytes_exact": svg.get("exact_bytes"),
            "static_svg_pixels_exact": raster.get("exact"),
            "static_svg_pixels_changed": raster.get("different_pixels"),
            "static_png_vs_svg_pixels_changed": static.get("png", {}).get("different_pixels"),
            "gd_error": error_kind(gd), "gd_detail": gd.get("detail"),
            "gd_json_exact": gd.get("geometry", {}).get("exact"),
            "gd_json_numeric_fields": g_num, "gd_json_structure_fields": g_structure,
            "gd_svg_raw_bytes_exact": gd.get("svg", {}).get("exact_bytes"),
            "native_gd_png_exact": native.get("exact"),
            "native_gd_png_pixels_changed": native.get("different_pixels"),
            "native_gd_oracle_size": native.get("oracle_size"),
            "native_gd_candidate_size": native.get("candidate_size")}


def target_summary(static_report, gd_report, coverage):
    static_cases = case_map(static_report, coverage)
    gd_cases = case_map(gd_report, coverage)
    rows = [row(name, tags, static_cases[name], gd_cases[name])
            for name, tags in sorted(coverage.items())]
    fields = ("static_json_exact", "static_svg_raw_bytes_exact", "static_svg_pixels_exact",
              "gd_json_exact", "gd_svg_raw_bytes_exact", "native_gd_png_exact")
    counts = {field: sum(case[field] is True for case in rows) for field in fields}
    counts.update({"static_unsupported": sum(case["static_error"] == "unsupported" for case in rows),
                   "gd_unsupported": sum(case["gd_error"] == "unsupported" for case in rows),
                   "static_other_error": sum(case["static_error"] == "error" for case in rows),
                   "gd_other_error": sum(case["gd_error"] == "error" for case in rows)})
    return {"counts": counts, "cases": rows,
            "static_oracle_sha256": static_report["oracle"]["sha256"],
            "gd_oracle_sha256": gd_report["oracle"]["sha256"],
            "candidate_wasm": static_report.get("candidate_wasm"),
            "rasterizer_reviewed": (static_report.get("rasterizer") or {}).get("reviewed"),
            "gd_oracle_certified": (gd_report.get("native_gd_png_oracle") or {}).get("certified")}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("desktop_static", "desktop_gd", "wasm_static", "wasm_gd"):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    parser.add_argument("--coverage", type=Path,
                        default=Path(__file__).with_name("fixtures-extensions") / "coverage.json")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    coverage = json.loads(args.coverage.read_text())
    reports = {name: json.loads(getattr(args, name).read_text())
               for name in ("desktop_static", "desktop_gd", "wasm_static", "wasm_gd")}
    summary = {"schema": 1, "scope": "separate diagnostic extension corpus",
               "fixture_count": len(coverage),
               "desktop": target_summary(reports["desktop_static"], reports["desktop_gd"], coverage),
               "wasm": target_summary(reports["wasm_static"], reports["wasm_gd"], coverage)}
    args.output.write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps({target: summary[target]["counts"] for target in ("desktop", "wasm")}, sort_keys=True))


if __name__ == "__main__":
    main()
