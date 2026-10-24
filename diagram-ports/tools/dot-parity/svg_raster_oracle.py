"""Identify a pinned Linux or macOS Inkscape SVG-to-PNG pixel reference."""

import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

from gd_oracle import font_identity, linked_libraries, sha256


def identify(command, target_platform):
    system = platform.system()
    supported = ((system == "Linux" and target_platform in ("linux", "wasm")) or
                 (system == "Darwin" and target_platform == "macos"))
    if not supported:
        raise ValueError(f"SVG raster oracle has no {system} host and {target_platform} candidate pairing")
    executable = shutil.which(command[0])
    if executable is None:
        raise ValueError(f"SVG rasterizer executable {command[0]!r} was not found")
    binary = Path(executable).resolve()
    result = subprocess.run([str(binary), "--version"], capture_output=True, check=True, timeout=10)
    version = (result.stdout + result.stderr).decode(errors="replace").strip()
    if not version.startswith("Inkscape "):
        raise ValueError(f"SVG rasterizer must be Inkscape; received {version!r}")
    if not any("{input}" in part for part in command) or not any("{output}" in part for part in command):
        raise ValueError("SVG rasterizer command must use {input} and {output}")
    config = os.environ.get("FONTCONFIG_FILE", "")
    if not config:
        raise ValueError("SVG raster oracle requires an explicit FONTCONFIG_FILE")
    macos_build = subprocess.run(["sw_vers", "-buildVersion"], capture_output=True,
                                 check=True, timeout=10).stdout.decode().strip() if system == "Darwin" else None
    return {"schema": 1, "system": system, "target_platform": target_platform,
            "renderer": "Inkscape", "version": version, "command": command,
            "binary_path": str(binary), "binary_sha256": sha256(binary),
            "linked_libraries": linked_libraries((binary,)), "fonts": font_identity(),
            "native_wasm_graphviz_reference": False,
            **({"macos_build": macos_build,
                "DYLD_LIBRARY_PATH": os.environ.get("DYLD_LIBRARY_PATH", "")}
               if system == "Darwin" else {})}


def verified_identity(command, manifest_path, target_platform):
    identity = identify(command, target_platform)
    if manifest_path is None:
        return {"identity": identity, "reviewed": False, "certified": False}
    manifest = json.loads(manifest_path.read_text())
    expected = {key: value for key, value in manifest.items()
                if key not in ("reviewed", "candidate_png_renderer")}
    if expected != identity:
        raise ValueError(f"SVG rasterizer manifest {manifest_path} differs from executable, libraries, command or fonts")
    reviewed = manifest.get("reviewed") is True
    native_compatible = manifest.get("candidate_png_renderer") == identity["renderer"]
    host_target = target_platform in ("linux", "macos")
    return {"identity": identity, "reviewed": reviewed,
            "svg_pixel_oracle_certified": reviewed and host_target,
            "candidate_native_png_renderer_matches": native_compatible,
            "certified": reviewed and native_compatible and host_target}
