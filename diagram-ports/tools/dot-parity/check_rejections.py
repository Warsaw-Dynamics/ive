#!/usr/bin/env python3
"""Check deterministic malformed DOT and parser admission-limit rejections."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from compare import expand, load_command

CASES = {
    "missing_closing_brace": ("digraph G { a -> b", "unclosed brace", True),
    "unterminated_quote": ('digraph G { "a -> b }', "unterminated", True),
    "source_limit": ("digraph G { /*" + "x" * (256 * 1024) + "*/ }",
                     ("262144-byte limit", "outside 1..262144"), False),
    "statement_limit": ("digraph G {\n" + "a;\n" * 10_001 + "}", "statement count exceeds 10000", False),
    "nesting_limit": ("digraph G {" + "subgraph cluster_x {" * 65 + "a;" + "}" * 65 + "}",
                      "nesting exceeds 64", False),
}


def run(command, timeout):
    try:
        result = subprocess.run(command, capture_output=True, check=False, timeout=timeout)
    except subprocess.TimeoutExpired:
        return {"exit_code": None, "timeout": True, "stderr": "", "stdout": ""}
    return {"exit_code": result.returncode, "timeout": False,
            "stderr": result.stderr.decode(errors="replace")[:1000],
            "stdout": result.stdout.decode(errors="replace")[:1000]}


def check_case(folder, name, source, phrase, oracle_check, candidate, oracle, timeout):
    input_path = folder / f"{name}.dot"
    input_path.write_text(source)
    candidate_result = run(expand(candidate, input_path, folder / f"{name}.json", "json"), timeout)
    phrases = (phrase,) if isinstance(phrase, str) else phrase
    candidate_result["expected_rejection"] = (
        candidate_result["exit_code"] not in (None, 0)
        and any(fragment in candidate_result["stderr"] for fragment in phrases)
    )
    oracle_result = None
    if oracle_check and oracle is not None:
        oracle_result = run([str(oracle), "-Tjson0", "-o", str(folder / f"{name}-oracle.json"),
                             str(input_path)], timeout)
        oracle_result["expected_rejection"] = oracle_result["exit_code"] not in (None, 0)
    return {"name": name, "source_bytes": len(source.encode()),
            "source_sha256": hashlib.sha256(source.encode()).hexdigest(),
            "candidate": candidate_result, "oracle": oracle_result}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-command", type=Path, required=True)
    parser.add_argument("--oracle", type=Path, help="Graphviz dot executable for malformed-input rejection")
    parser.add_argument("--check-layout-work", action="store_true",
                        help="also run the in-process public engine layout_work rejection probe")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=10)
    args = parser.parse_args()
    if args.timeout < 1 or args.timeout > 30:
        parser.error("timeout must be in 1..30 seconds")
    args.output.mkdir(parents=True, exist_ok=True)
    candidate = load_command(args.candidate_command)
    cases = [check_case(args.output, name, *spec, candidate, args.oracle, args.timeout)
             for name, spec in CASES.items()]
    layout_work = None
    if args.check_layout_work:
        script = Path(__file__).with_name("check_layout_work.sh")
        layout_work = run(["bash", str(script), str(args.output / "layout-work")], args.timeout)
        layout_work["expected_rejection"] = (layout_work["exit_code"] == 0 and
                                             "DOT work count 2 exceeds 1" in layout_work["stdout"])
    report = {"schema": 1, "candidate_command": candidate, "oracle": str(args.oracle) if args.oracle else None,
              "scope": "DOT lexical/syntax and parser admission limits; optional in-process engine layout_work probe",
              "cases": cases, "layout_work": layout_work}
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    passed = sum(case["candidate"]["expected_rejection"] and
                 (case["oracle"] is None or case["oracle"]["expected_rejection"]) for case in cases)
    if layout_work is not None:
        passed += bool(layout_work["expected_rejection"])
    total = len(cases) + (layout_work is not None)
    print(json.dumps({"passed": passed, "total": total}))
    return 0 if passed == total else 1


if __name__ == "__main__":
    raise SystemExit(main())
