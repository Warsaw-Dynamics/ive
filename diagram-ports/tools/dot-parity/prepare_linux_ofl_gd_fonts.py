#!/usr/bin/env python3
"""Prepare isolated bundled OFL fonts for the Graphviz 16.1.0 GD oracle."""

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[2]
FONTS = {
    "LiberationSerif-Regular.ttf": (
        "705903ae1382f0150f115d6f10a30f5b2f3d6ade649c474afe6366c39174fb65",
        "Liberation Serif",
        "Nimbus Roman No9 L",
    ),
    "LiberationMono-Regular.ttf": (
        "395fa5ab8d40c8eba390ced528744ea75a7f69aabf3e68b6f925ca0e39a27370",
        "Liberation Mono",
        "Nimbus Mono L",
    ),
    "LiberationSans-Regular.ttf": (
        "4659bc0c58c5028dd488ec928d41d9265db43d9b669fc14ca8b0832daca7b144",
        "Liberation Sans",
        "Nimbus Sans L",
    ),
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_fonts() -> None:
    for name, (expected, _, _) in FONTS.items():
        path = ROOT / "web/fonts" / name
        actual = sha256(path)
        if actual != expected:
            raise ValueError(f"bundled OFL font {path} SHA-256 differs: expected {expected}, found {actual}")


def write_config(folder: Path) -> Path:
    root = ET.Element("fontconfig")
    ET.SubElement(root, "dir").text = str((ROOT / "web/fonts").resolve())
    ET.SubElement(root, "cachedir").text = str(folder / "cache")
    for _, source, alias in FONTS.values():
        match = ET.SubElement(root, "match", target="scan")
        ET.SubElement(ET.SubElement(match, "test", name="family", compare="eq"), "string").text = source
        ET.SubElement(ET.SubElement(match, "edit", name="family", mode="assign"), "string").text = alias
    for requested, selected in (
        ("Times", "Nimbus Roman No9 L"),
        ("Courier", "Nimbus Mono L"),
        ("Helvetica", "Nimbus Sans L"),
    ):
        alias = ET.SubElement(root, "alias", binding="strong")
        ET.SubElement(alias, "family").text = requested
        ET.SubElement(ET.SubElement(alias, "prefer"), "family").text = selected
    config = folder / "fonts.conf"
    ET.ElementTree(root).write(config, encoding="utf-8", xml_declaration=True)
    return config


def prepare(folder: Path) -> Path:
    verify_fonts()
    folder = folder.resolve()
    folder.mkdir(parents=True, exist_ok=True)
    (folder / "cache").mkdir(exist_ok=True)
    config = write_config(folder)
    environment = {**os.environ, "FONTCONFIG_FILE": str(config)}
    subprocess.run(
        ["fc-cache", "-f", str(ROOT / "web/fonts")],
        env=environment,
        check=True,
        capture_output=True,
        timeout=30,
    )
    return config


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    config = prepare(args.output)
    print(f"export FONTCONFIG_FILE={config}")


if __name__ == "__main__":
    main()
