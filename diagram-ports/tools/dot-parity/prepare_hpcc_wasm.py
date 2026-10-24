#!/usr/bin/env python3
"""Extract and pin a developer-only prebuilt Graphviz 16.1.0 WASM reference."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile

TARBALL_SHA256 = "0105defc8548e241a08a22e40ee0499a851402c345e38745a5b4d521d776d57f"
BUNDLE_SHA256 = "77d45a274928484dd1daccd55ab3e41ddda5b4262e29240e205f7188839d9d5d"
PACKAGE_VERSION = "1.30.0"
GRAPHVIZ_VERSION = "16.1.0"
FILES = ("package/package.json", "package/dist/index.js", "package/LICENSE")


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def extract_package(tarball, output):
    if tarball.stat().st_size > 4 * 1024 * 1024 or sha256(tarball) != TARBALL_SHA256:
        raise ValueError(f"package {tarball} exceeds 4 MiB or differs from pinned SHA-256 {TARBALL_SHA256}")
    package = output / "package"
    package.mkdir(parents=True, exist_ok=True)
    with tarfile.open(tarball, "r:gz") as archive:
        for name in FILES:
            member = archive.getmember(name)
            if not member.isfile() or member.size > 4 * 1024 * 1024:
                raise ValueError(f"package member {name} is missing, not regular, or oversized")
            target = output / name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source:
                target.write_bytes(source.read())
    bundle = package / "dist/index.js"
    if sha256(bundle) != BUNDLE_SHA256:
        raise ValueError(f"WASM bundle {bundle} differs from pinned SHA-256 {BUNDLE_SHA256}")
    return package


def node_identity():
    result = subprocess.run(["node", "-p", "process.execPath"], capture_output=True,
                            text=True, check=True, timeout=10)
    binary = Path(result.stdout.strip()).resolve()
    version = subprocess.run(["node", "--version"], capture_output=True,
                             text=True, check=True, timeout=10).stdout.strip()
    return {"path": str(binary), "sha256": sha256(binary), "version": version}


def verify_runtime(package):
    runner = Path(__file__).with_name("run_hpcc_wasm.cjs")
    result = subprocess.run(["node", str(runner), "-V"],
                            env={**os.environ, "DOT_WASM_PACKAGE_ROOT": str(package)},
                            capture_output=True, text=True, check=True, timeout=15)
    if f"graphviz version {GRAPHVIZ_VERSION}" not in result.stdout:
        raise ValueError(f"embedded WASM version differs: {result.stdout!r}")
    return result.stdout.strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tarball", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    package = extract_package(args.tarball, output)
    metadata = json.loads((package / "package.json").read_text())
    if metadata.get("name") != "@hpcc-js/wasm-graphviz" or metadata.get("version") != PACKAGE_VERSION:
        raise ValueError(f"WASM package metadata differs: {metadata.get('name')}@{metadata.get('version')}")
    version = verify_runtime(package)
    manifest = {"schema": 1, "kind": "third-party-prebuilt-graphviz-wasm",
                "reviewed": False, "source_provenance_reviewed": False,
                "vendor_source_equivalence_verified": False,
                "package_name": metadata["name"], "package_version": metadata["version"],
                "package_git_head": metadata.get("gitHead"),
                "package_tarball_path": str(args.tarball.resolve()),
                "package_tarball_sha256": TARBALL_SHA256,
                "bundle_path": str(package / "dist/index.js"), "bundle_sha256": BUNDLE_SHA256,
                "runner_path": str(Path(__file__).with_name("run_hpcc_wasm.cjs")),
                "runner_sha256": sha256(Path(__file__).with_name("run_hpcc_wasm.cjs")),
                "embedded_version": version, "node": node_identity(),
                "formats": ["json0", "svg"], "native_png_available": False,
                "external_font_files_shipped": []}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"wrote unreviewed Graphviz WASM identity to {output / 'manifest.json'}")


if __name__ == "__main__":
    main()
