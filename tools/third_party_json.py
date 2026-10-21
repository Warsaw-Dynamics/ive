#!/usr/bin/env python3
"""Writes assets/licenses/third-party.json, the list the Settings page Third-Party Software shows.

Usage: cargo about generate --format json | tools/third_party_json.py

Every Rust crate of `cargo about` (but IVE's own) and every entry of tools/third-party-extra.toml becomes one row:
name, version, link, licence expression, and the licence texts that apply. Each distinct text is stored once.
"""
import json
import sys
import tomllib
from pathlib import Path

IVE = Path(__file__).resolve().parent.parent
OUT = IVE / "assets/licenses/third-party.json"
OWN_LICENSE = "LicenseRef-WarsawDynamics-Proprietary"


class Texts:
    """Licence texts, each stored once; `index` gives a text's position."""

    def __init__(self):
        self.texts = []
        self.positions = {}

    def index(self, text):
        text = text.replace("\r\n", "\n").strip("\n")
        if text not in self.positions:
            self.positions[text] = len(self.texts)
            self.texts.append(text)
        return self.positions[text]


def crate_rows(about, texts):
    """One row per crate, with every licence text `cargo about` found for it."""
    by_crate = {}
    for license in about["licenses"]:
        if license["id"] == OWN_LICENSE:
            continue
        position = texts.index(license["text"])
        for used in license["used_by"]:
            crate = used["crate"]
            row = by_crate.setdefault(crate["id"], {
                "name": crate["name"],
                "version": crate["version"],
                "url": crate.get("repository") or crate.get("homepage") or f"https://crates.io/crates/{crate['name']}",
                "license": crate.get("license") or license["id"],
                "texts": [],
            })
            if position not in row["texts"]:
                row["texts"].append(position)
    return list(by_crate.values())


def extra_rows(texts):
    """The rows of tools/third-party-extra.toml, reading each licence file."""
    extra = tomllib.loads((IVE / "tools/third-party-extra.toml").read_text())
    return [{
        "name": entry["name"],
        "version": entry.get("version", ""),
        "url": entry["url"],
        "license": entry["license"],
        "texts": [texts.index((IVE / entry["text"]).read_text())],
    } for entry in extra["software"]]


def main(about):
    texts = Texts()
    rows = crate_rows(about, texts) + extra_rows(texts)
    rows.sort(key=lambda row: (row["name"].lower(), row["version"]))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"licenses": texts.texts, "software": rows}, ensure_ascii=False, indent=1) + "\n")
    print(f"{OUT.relative_to(IVE)}: {len(rows)} entries, {len(texts.texts)} licence texts")


if __name__ == "__main__":
    main(json.load(sys.stdin))
