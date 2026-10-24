#!/usr/bin/env python3
"""Create a private pinned fontconfig inventory for macOS DOT parity runs."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import xml.etree.ElementTree as ET

SOURCE = Path(__file__).resolve().parents[2] / "web" / "fonts"
FONTS = {
    "LiberationMono-Bold.ttf": "626655e94dd82f3f42549daf995c921b0915fa8ab1f4b839559e8892ea41d240",
    "LiberationMono-BoldItalic.ttf": "15eb161953e3ecc7fc05a3fec8a59e0f4e0a54a4e375736adf8e98d65981f813",
    "LiberationMono-Italic.ttf": "a71b2c25c89da05cf0e7c4dbba8d473fdead0b181ae56165217747d2c1f39215",
    "LiberationMono-Regular.ttf": "395fa5ab8d40c8eba390ced528744ea75a7f69aabf3e68b6f925ca0e39a27370",
    "LiberationSans-Bold.ttf": "3973aa5054fb467dd5627245d3dc82e37bf16fe075756156a570455871351582",
    "LiberationSans-BoldItalic.ttf": "c80fa7f2ffa0e01d4d8dcd6a6d1e43eda665222d1b4db597dde1174c456006cf",
    "LiberationSans-Italic.ttf": "830c5fa600505fb4c1a271b4271c53c44bae43f492b2a240d0e98a3a7a380121",
    "LiberationSans-Regular.ttf": "4659bc0c58c5028dd488ec928d41d9265db43d9b669fc14ca8b0832daca7b144",
    "LiberationSerif-Bold.ttf": "9e66c25e20868756bf12eb24a520a4552727c9cd108f577f7ae33e3c0110e39c",
    "LiberationSerif-BoldItalic.ttf": "379010e87421a883f7bbfa7936d23dfe0257a54ce850819b74556e9f2c615a8a",
    "LiberationSerif-Italic.ttf": "eaeaf7f2b12544ecde64290c1fa7ec403ff70fd58b313637b265400b20249242",
    "LiberationSerif-Regular.ttf": "705903ae1382f0150f115d6f10a30f5b2f3d6ade649c474afe6366c39174fb65",
}
ALIASES = {
    "Courier": "Liberation Mono", "Nimbus Mono L": "Liberation Mono",
    "Helvetica": "Liberation Sans", "Nimbus Sans L": "Liberation Sans",
    "Times-Roman": "Liberation Serif", "Nimbus Roman No9 L": "Liberation Serif",
}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_config(folder):
    root = ET.Element("fontconfig")
    ET.SubElement(root, "dir").text = str(folder / "fonts")
    ET.SubElement(root, "cachedir").text = str(folder / "cache")
    for name, replacement in ALIASES.items():
        alias = ET.SubElement(root, "alias")
        ET.SubElement(alias, "family").text = name
        ET.SubElement(ET.SubElement(alias, "prefer"), "family").text = replacement
    path = folder / "fonts.conf"
    ET.ElementTree(root).write(path, encoding="utf-8", xml_declaration=True)
    return path


def prepare(folder):
    folder = folder.resolve()
    (folder / "fonts").mkdir(parents=True, exist_ok=True)
    (folder / "cache").mkdir(parents=True, exist_ok=True)
    for name, expected in FONTS.items():
        source = SOURCE / name
        actual = sha256(source)
        if actual != expected:
            raise ValueError(f"font {source} hash differs: expected {expected}, found {actual}")
        shutil.copy2(source, folder / "fonts" / name)
    config = write_config(folder)
    subprocess.run(["fc-cache", "-f", str(folder / "fonts")],
                   env={**os.environ, "FONTCONFIG_FILE": str(config)},
                   check=True, capture_output=True, timeout=30)
    return config


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("private macOS font preparation requires a macOS host")
    config = prepare(args.output)
    manifest = {"schema": 1, "system": "Darwin", "fonts": FONTS,
                "fontconfig_file": str(config), "fontconfig_sha256": sha256(config),
                "reviewed": False}
    (args.output / "fonts-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"export FONTCONFIG_FILE={config}")


if __name__ == "__main__":
    main()
