#!/usr/bin/env python3
"""Audit the separate developer generator's exact registry archives and unpacked files."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
GENERATOR = ROOT / "tools/plantuml-grammar-gen"
INVENTORY = ROOT / "provenance/plantuml-generator-dependencies.json"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def inspect_package(package, locked, features):
    source = Path(package["manifest_path"]).parent
    registry = source.parent.parent.parent
    archive = registry / "cache" / source.parent.name / f"{package['name']}-{package['version']}.crate"
    expected = locked[(package["name"], package["version"])]["checksum"]
    if digest(archive.read_bytes()) != expected:
        raise ValueError(f"generator registry archive checksum mismatch: {archive.name}")
    total = 0
    registered = set()
    with tarfile.open(archive) as contents:
        for member in contents:
            if member.isdir():
                continue
            relative = Path(member.name).relative_to(source.name)
            if member.isfile() is False or ".." in relative.parts or member.size > 16 * 1024 * 1024:
                raise ValueError(f"invalid generator archive member: {member.name}")
            total += member.size
            registered.add(relative)
            if total > 64 * 1024 * 1024:
                raise ValueError(f"generator archive exceeds 64 MiB: {archive.name}")
            if contents.extractfile(member).read() != (source / relative).read_bytes():
                raise ValueError(f"generator unpacked source differs from archive: {member.name}")
    actual = {path.relative_to(source) for path in source.rglob("*") if path.is_file()}
    actual.difference_update({Path(".cargo-ok"), Path(".cargo-checksum.json")})
    if actual != registered:
        raise ValueError(f"generator unpacked source inventory mismatch: {source.name}")
    return {"name": package["name"], "version": package["version"],
            "source": package["source"], "sha256": expected,
            "license": package["license"], "features": sorted(features)}


def collect():
    environment = dict(os.environ, CARGO_TARGET_DIR="/mnt/data/Projects/ive/target")
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--offline", "--format-version", "1",
        "--manifest-path", str(GENERATOR / "Cargo.toml"),
    ], env=environment, timeout=60))
    lock_bytes = (GENERATOR / "Cargo.lock").read_bytes()
    lock = tomllib.loads(lock_bytes.decode())
    locked = {(package["name"], package["version"]): package for package in lock["package"]}
    features = {node["id"]: node["features"] for node in metadata["resolve"]["nodes"]}
    packages = []
    for package in metadata["packages"]:
        if package["source"] is None:
            if package["name"] != "plantuml-grammar-gen":
                raise ValueError(f"unexpected generator local dependency: {package['name']}")
            continue
        if package["source"] != "registry+https://github.com/rust-lang/crates.io-index":
            raise ValueError(f"unexpected generator source: {package['source']}")
        packages.append(inspect_package(package, locked, features[package["id"]]))
    return {"schema": 1, "scope": "developer-only; not an engine/viewer dependency",
            "manifestSha256": digest((GENERATOR / "Cargo.toml").read_bytes()),
            "lockSha256": digest(lock_bytes),
            "packages": sorted(packages, key=lambda package: (package["name"], package["version"]))}


def main():
    if sys.argv[1:] not in [["record"], ["check"]]:
        raise ValueError("usage: audit-plantuml-generator.py record|check")
    mode = sys.argv[1]
    if mode == "record" and INVENTORY.exists():
        raise ValueError(f"generator closure inventory already exists: {INVENTORY}")
    current = collect()
    if mode == "record":
        with INVENTORY.open("x") as output:
            output.write(json.dumps(current, indent=2) + "\n")
    elif current != json.loads(INVENTORY.read_text()):
        raise ValueError("generator dependency checksum/license/feature inventory drift")
    print(f"Verified {len(current['packages'])} generator registry archives, unpacked sources, licenses and selected features ({mode}).")


if __name__ == "__main__":
    main()
