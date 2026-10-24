#!/usr/bin/env python3
"""Create a private fontconfig inventory for Graphviz 16.1.0's GD renderer."""

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

SOURCE = Path("/usr/share/fonts/opentype/urw-base35")
FONTS = {
    "NimbusMonoPS-Regular.otf": "4f225ca8e13acb16f733ce741693105e527d5f7a5443901b9ecc190fca4e149b",
    "NimbusSans-Regular.otf": "7c25be4d78155523080ab85b10277150657ff7dabbcad7037bdd536c9b6d0d08",
    "NimbusRoman-Regular.otf": "fd41669443616a7ae890c42e53a58f47fa24a940fd1e367acaff665cf5b6e9d8",
}
FAMILIES = {
    "Nimbus Mono PS": "Nimbus Mono L",
    "Nimbus Sans": "Nimbus Sans L",
    "Nimbus Roman": "Nimbus Roman No9 L",
}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_config(folder):
    root = ET.Element("fontconfig")
    ET.SubElement(root, "dir").text = str(folder / "fonts")
    ET.SubElement(root, "cachedir").text = str(folder / "cache")
    for current, legacy in FAMILIES.items():
        match = ET.SubElement(root, "match", target="scan")
        ET.SubElement(ET.SubElement(match, "test", name="family", compare="eq"), "string").text = current
        ET.SubElement(ET.SubElement(match, "edit", name="family", mode="assign"), "string").text = legacy
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
    environment = {**os.environ, "FONTCONFIG_FILE": str(config)}
    subprocess.run(["fc-cache", "-f", str(folder / "fonts")], env=environment, check=True,
                   capture_output=True, timeout=30)
    return config


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    config = prepare(args.output)
    print(f"export FONTCONFIG_FILE={config}")


if __name__ == "__main__":
    main()
