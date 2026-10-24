"""Identify a developer-only Graphviz 16.1.0 GD PNG oracle."""

import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess

SOURCE = Path(__file__).resolve().parents[2] / "vendor/graphviz/graphviz-16.1.0.tar.xz"
SOURCE_SHA256 = "0f661718f3e5268dc3bad0fb53ab646605d26db70f4080799aff1dc2f61783c3"
VERSION = "graphviz version 16.1.0"
FORMAT = "png:gd:gd"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def output(command):
    result = subprocess.run(command, capture_output=True, check=False, timeout=10)
    if result.returncode:
        raise ValueError(f"cannot identify GD oracle via {command[0]}: {result.stderr.decode(errors='replace')[:500]}")
    return (result.stdout + result.stderr).decode(errors="replace")


def plugin_path(binary):
    prefix = binary.resolve().parent.parent
    suffixes = ("dylib", "so") if platform.system() == "Darwin" else ("so",)
    matches = sorted({path.resolve() for suffix in suffixes
                      for path in prefix.glob(f"lib*/graphviz/libgvplugin_gd.{suffix}")})
    if len(matches) != 1:
        raise ValueError(f"Graphviz GD plugin under {prefix} must have one GD shared library; found {len(matches)}")
    return matches[0]


def linux_linked_libraries(paths):
    libraries = {}
    for target in paths:
        for line in output(["ldd", str(target)]).splitlines():
            match = re.search(r"(?:=>\s*)?(/\S+)\s+\(", line)
            if match:
                path = Path(match.group(1)).resolve()
                libraries[str(path)] = sha256(path)
    return dict(sorted(libraries.items()))


def macos_rpaths(target, executable):
    lines = output(["otool", "-l", str(target)]).splitlines()
    paths = []
    for index, line in enumerate(lines):
        if line.strip() != "cmd LC_RPATH":
            continue
        for nearby in lines[index + 1:index + 5]:
            match = re.search(r"\bpath\s+(\S+)\s+\(offset", nearby)
            if match:
                value = match.group(1)
                value = value.replace("@loader_path", str(target.parent))
                value = value.replace("@executable_path", str(executable.parent))
                paths.append(Path(value))
                break
    return paths


def macos_install_name(target):
    lines = output(["otool", "-l", str(target)]).splitlines()
    for index, line in enumerate(lines):
        if line.strip() != "cmd LC_ID_DYLIB":
            continue
        for nearby in lines[index + 1:index + 5]:
            match = re.search(r"\bname\s+(\S+)\s+\(offset", nearby)
            if match:
                return match.group(1)
    return None


def macos_library_paths(target, name, executable):
    if name.startswith("@loader_path/"):
        return [target.parent / name.removeprefix("@loader_path/")]
    if name.startswith("@executable_path/"):
        return [executable.parent / name.removeprefix("@executable_path/")]
    if not name.startswith("@rpath/"):
        return [Path(name)]
    prefix = executable.resolve().parent.parent
    brew = Path("/opt/homebrew" if platform.machine() == "arm64" else "/usr/local")
    search = [target.parent, executable.parent, prefix / "lib", prefix / "Frameworks",
              prefix / "Resources" / "lib", brew / "lib"]
    search.extend(macos_rpaths(target, executable))
    search.extend(Path(path) for path in os.environ.get("DYLD_LIBRARY_PATH", "").split(":") if path)
    return [path / name.removeprefix("@rpath/") for path in search]


def macos_linked_libraries(paths):
    libraries = {}
    executable = paths[0].resolve()
    pending = list(paths)
    inspected = set()
    while pending:
        target = pending.pop()
        if target in inspected:
            continue
        inspected.add(target)
        install_name = macos_install_name(target)
        for line in output(["otool", "-L", str(target)]).splitlines()[1:]:
            name = line.strip().split(" (", 1)[0]
            if not name or name == install_name:
                continue
            if name.startswith("/usr/lib/") or name.startswith("/System/Library/"):
                libraries[f"macOS shared cache:{name}"] = "system image; see macos_build"
                continue
            found = next((path.resolve() for path in macos_library_paths(target, name, executable) if path.is_file()), None)
            if found is not None:
                libraries[str(found)] = sha256(found)
                pending.append(found)
            else:
                raise ValueError(f"cannot resolve macOS linked library {name!r} of {target}; set DYLD_LIBRARY_PATH")
            if len(libraries) > 1024:
                raise ValueError("macOS linked library inventory exceeds 1024 entries")
    return dict(sorted(libraries.items()))


def linked_libraries(paths):
    if platform.system() == "Darwin":
        return macos_linked_libraries(paths)
    return linux_linked_libraries(paths)


def font_identity():
    inventory = output(["fc-list", "-f", "%{file}|%{family}|%{style}\n"])
    families = ("Courier", "Times-Roman", "Helvetica", "sans", "serif", "monospace")
    matches = {}
    for family in families:
        selected = output(["fc-match", "-f", "%{file}", family]).strip()
        path = Path(selected).resolve()
        matches[family] = {"path": str(path), "sha256": sha256(path)}
    config_name = os.environ.get("FONTCONFIG_FILE", "")
    config_hash = sha256(Path(config_name)) if config_name else None
    return {"inventory_sha256": hashlib.sha256("\n".join(sorted(inventory.splitlines())).encode()).hexdigest(),
            "matches": matches, "FONTCONFIG_FILE": config_name,
            "config_sha256": config_hash,
            "FONTCONFIG_PATH": os.environ.get("FONTCONFIG_PATH", ""),
            "FC_LANG": os.environ.get("FC_LANG", "")}


def identify(binary):
    system = platform.system()
    if system not in ("Linux", "Darwin"):
        raise ValueError(f"native GD oracle identity is unavailable on {system}")
    source_hash = sha256(SOURCE)
    if source_hash != SOURCE_SHA256:
        raise ValueError(f"vendored Graphviz source hash differs: expected {SOURCE_SHA256}, found {source_hash}")
    version = output([str(binary), "-V"]).strip()
    if VERSION not in version:
        raise ValueError(f"native GD oracle must be Graphviz 16.1.0; received {version!r}")
    plugin = plugin_path(binary)
    macos_build = output(["sw_vers", "-buildVersion"]).strip() if system == "Darwin" else None
    return {"schema": 1, "system": system, "version": version, "format": FORMAT,
            "source_sha256": source_hash, "binary_sha256": sha256(binary),
            "plugin_sha256": sha256(plugin), "plugin_path": str(plugin),
            "linked_libraries": linked_libraries((binary, plugin)), "fonts": font_identity(),
            **({"macos_build": macos_build,
                "DYLD_LIBRARY_PATH": os.environ.get("DYLD_LIBRARY_PATH", "")}
               if system == "Darwin" else {})}


def verified_identity(binary, manifest_path):
    identity = identify(binary)
    if manifest_path is None:
        return {"identity": identity, "certified": False}
    manifest = json.loads(manifest_path.read_text())
    expected = {key: value for key, value in manifest.items() if key != "reviewed"}
    if expected != identity:
        raise ValueError(f"native GD oracle manifest {manifest_path} does not match executable, plugin, libraries or fonts")
    return {"identity": identity, "certified": manifest.get("reviewed") is True}
