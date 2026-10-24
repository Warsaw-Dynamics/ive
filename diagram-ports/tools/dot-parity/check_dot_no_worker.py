#!/usr/bin/env python3
"""Verify Linux desktop DOT exports execute no viewer child process."""

import argparse
import json
from pathlib import Path
import subprocess


def trace_export(executable, source, output, trace):
    command = ["strace", "-f", "-e", "trace=execve", "-o", str(trace), str(executable),
               "--render", str(source), "--output", str(output)]
    result = subprocess.run(command, capture_output=True, text=True, timeout=15, check=False)
    launches = [line for line in trace.read_text().splitlines() if "execve(" in line]
    return {"exit_code": result.returncode, "stderr": result.stderr[:1000],
            "execve_count": len(launches), "worker_launches": sum('"--worker"' in line for line in launches),
            "output_bytes": output.stat().st_size if output.is_file() else 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--viewer", type=Path, required=True)
    parser.add_argument("--dot", type=Path, required=True)
    parser.add_argument("--plantuml", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    viewer = args.viewer.resolve()
    if not viewer.is_file():
        parser.error(f"viewer executable {viewer} is missing")
    args.output.mkdir(parents=True, exist_ok=True)
    cases = {}
    for extension in ("svg", "png", "json"):
        output = args.output / f"dot.{extension}"
        trace = args.output / f"dot-{extension}.trace"
        cases[extension] = trace_export(viewer, args.dot.resolve(), output, trace)
    cases["plantuml_control"] = trace_export(
        viewer, args.plantuml.resolve(), args.output / "plantuml.svg",
        args.output / "plantuml.trace")
    passed = all(case["exit_code"] == 0 and case["execve_count"] == 1
                 and case["worker_launches"] == 0 and case["output_bytes"] > 0
                 for name, case in cases.items() if name != "plantuml_control")
    control = cases["plantuml_control"]
    passed &= control["exit_code"] == 0 and control["worker_launches"] == 1
    report = {"schema": 1, "viewer": str(viewer), "cases": cases, "passed": passed}
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"passed": passed, "cases": {key: value["execve_count"] for key, value in cases.items()}}))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
