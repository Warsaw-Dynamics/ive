#!/usr/bin/env python3
"""Project selected PlantUML UML graph fixtures into a bounded DOT corpus."""

import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
SOURCES = Path(__file__).with_name("plantuml-sources.json")
DECLARATION = re.compile(
    r'^(class|object|actor|usecase|component|database|node|package|rectangle)\s+'
    r'(?:("[^"]+")|([A-Za-z_][\w.]*))(?:\s+as\s+([A-Za-z_][\w.]*))?\s*(\{)?$'
)
RELATION = re.compile(
    r'^([A-Za-z_][\w.]*)\s+(?:"[^"]+"\s+)?'
    r'(<\|--|-->|o--|\*--|\.\.>|--|\.\.\|>)\s+'
    r'(?:"[^"]+"\s+)?([A-Za-z_][\w.]*)'
    r'(?:\s*:\s*(.*))?$'
)
CONTAINERS = {"node", "package", "rectangle"}
MAX_SOURCE = 256 * 1024


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def extract_test_body(source, name):
    marker = re.search(rf'^fn {re.escape(name)}\(\) \{{', source, re.MULTILINE)
    if marker is None:
        raise ValueError(f"PlantUML test {name!r} is missing")
    next_test = source.find("\n#[test]", marker.end())
    section = source[marker.end():next_test if next_test >= 0 else len(source)]
    literal = re.search(r'let source = diagram\(\s*("(?:[^"\\]|\\.)*")', section)
    if literal is None:
        raise ValueError(f"PlantUML test {name!r} has no direct diagram literal")
    return f"@startuml\n{json.loads(literal.group(1))}\n@enduml\n"


def source_text(spec):
    relative = Path(spec["source"])
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError(f"invalid PlantUML source path {relative}")
    path = ROOT / relative
    data = path.read_bytes()
    if len(data) > MAX_SOURCE:
        raise ValueError(f"PlantUML source {relative} exceeds {MAX_SOURCE} bytes")
    content = data.decode("utf-8")
    body = extract_test_body(content, spec["test"]) if "test" in spec else content
    return body, sha256(data)


def project(source, name):
    lines = [f"digraph {name} {{", "  graph [rankdir=TB];", "  node [shape=box];"]
    nodes = {}
    edges = []
    blocks = []
    cluster_number = 0
    for number, raw in enumerate(source.splitlines(), 1):
        line = raw.strip()
        if line in ("@startuml", "@enduml", ""):
            continue
        if line == "}":
            if not blocks:
                raise ValueError(f"{name}:{number}: unexpected closing brace")
            kind = blocks.pop()
            if kind == "container":
                lines.append("  " * (len(blocks) + 1) + "}")
            continue
        if blocks and blocks[-1] == "body":
            continue
        declaration = DECLARATION.fullmatch(line)
        if declaration:
            kind, quoted, plain, alias, opened = declaration.groups()
            label = json.loads(quoted) if quoted else plain
            identifier = alias or plain or label
            if kind in CONTAINERS and opened:
                lines.append("  " * (len(blocks) + 1) + f"subgraph cluster_{cluster_number} {{")
                lines.append("  " * (len(blocks) + 2) + f"label={json.dumps(label)};")
                cluster_number += 1
                blocks.append("container")
                continue
            if identifier in nodes:
                raise ValueError(f"{name}:{number}: duplicate node {identifier!r}")
            nodes[identifier] = f"n{len(nodes)}"
            lines.append("  " * (len(blocks) + 1) + f'{nodes[identifier]} [label={json.dumps(label)}];')
            if opened:
                blocks.append("body")
            continue
        relation = RELATION.fullmatch(line)
        if relation:
            tail, arrow, head, label = relation.groups()
            edges.append((number, tail, head, label, arrow))
            continue
        raise ValueError(f"{name}:{number}: unsupported projection statement {line!r}")
    if blocks:
        raise ValueError(f"{name}: {len(blocks)} unclosed blocks")
    for number, tail, head, label, arrow in edges:
        if tail not in nodes or head not in nodes:
            raise ValueError(f"{name}:{number}: unknown relation endpoint {tail!r} or {head!r}")
        attributes = [f"label={json.dumps(label)}"] if label else []
        if arrow == "..>":
            attributes.append("style=dashed")
        suffix = f" [{', '.join(attributes)}]" if attributes else ""
        lines.append(f"  {nodes[tail]} -> {nodes[head]}{suffix};")
    lines.append("}")
    return "\n".join(lines) + "\n", len(nodes), len(edges)


def generate(output):
    specs = json.loads(SOURCES.read_text())
    if len(specs) > 20:
        raise ValueError("PlantUML projection corpus exceeds 20 fixtures")
    output.mkdir(parents=True, exist_ok=True)
    entries = []
    for spec in specs:
        body, source_hash = source_text(spec)
        dot, node_count, edge_count = project(body, spec["name"])
        path = output / f'{spec["name"]}.dot'
        path.write_text(dot)
        entries.append({"fixture": path.name, "source": spec["source"],
                        "test": spec.get("test"), "source_sha256": source_hash,
                        "plantuml_sha256": sha256(body.encode()), "dot_sha256": sha256(dot.encode()),
                        "nodes": node_count, "edges": edge_count})
    manifest = {"schema": 1, "projection": "bounded UML declaration/relation topology v1",
                "generator_sha256": sha256(Path(__file__).read_bytes()), "cases": entries}
    (output / "provenance.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest = generate(args.output)
    print(json.dumps({"generated": len(manifest["cases"]),
                      "nodes": sum(case["nodes"] for case in manifest["cases"]),
                      "edges": sum(case["edges"] for case in manifest["cases"])}))


if __name__ == "__main__":
    main()
