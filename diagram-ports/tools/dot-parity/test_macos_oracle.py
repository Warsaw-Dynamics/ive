"""Portable checks for the unexecuted macOS oracle identity path."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from compare import prepare_oracle
from gd_oracle import macos_library_paths, macos_linked_libraries, sha256
from prepare_macos_fonts import write_config
from svg_raster_oracle import identify as identify_svg_raster


class MacosOracleTests(unittest.TestCase):
    def test_should_pin_macos_inkscape_identity_without_claiming_wasm_oracle(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "inkscape"
            binary.write_bytes(b"renderer")
            calls = [subprocess.CompletedProcess([], 0, b"Inkscape 1.4.4", b""),
                     subprocess.CompletedProcess([], 0, b"24A123", b"")]
            command = [str(binary), "{input}", "--export-filename={output}"]
            # when
            with patch("svg_raster_oracle.platform.system", return_value="Darwin"), \
                 patch("svg_raster_oracle.shutil.which", return_value=str(binary)), \
                 patch("svg_raster_oracle.subprocess.run", side_effect=calls), \
                 patch("svg_raster_oracle.linked_libraries", return_value={}), \
                 patch("svg_raster_oracle.font_identity", return_value={}), \
                 patch.dict(os.environ, {"FONTCONFIG_FILE": str(binary)}):
                identity = identify_svg_raster(command, "macos")
            # then
            self.assertEqual(identity["system"], "Darwin")
            self.assertEqual(identity["macos_build"], "24A123")
            self.assertFalse(identity["native_wasm_graphviz_reference"])

    def test_should_keep_installed_dot_at_its_original_path(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "dot"
            binary.write_bytes(b"oracle")
            version = subprocess.CompletedProcess([], 0, b"", b"dot - graphviz version 16.1.0")
            # when
            with patch("compare.command_output", return_value=version):
                selected, identity = prepare_oracle(binary, Path(directory))
            # then
            self.assertEqual(selected, binary)
            self.assertEqual(identity["sha256"], sha256(binary))

    def test_should_resolve_macho_loader_and_executable_paths(self):
        # given
        executable = Path("/tmp/oracle/bin/dot")
        plugin = Path("/tmp/oracle/lib/graphviz/libgvplugin_gd.dylib")
        # when
        loader = macos_library_paths(plugin, "@loader_path/../libgvc.dylib", executable)
        binary = macos_library_paths(plugin, "@executable_path/../lib/libgvc.dylib", executable)
        # then
        self.assertEqual(loader, [plugin.parent / "../libgvc.dylib"])
        self.assertEqual(binary, [executable.parent / "../lib/libgvc.dylib"])

    def test_should_hash_resolved_macho_dependency_and_record_shared_cache(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            prefix = Path(directory)
            binary = prefix / "bin/dot"
            library = prefix / "lib/libfoo.dylib"
            binary.parent.mkdir()
            library.parent.mkdir()
            binary.write_bytes(b"dot")
            library.write_bytes(b"foo")

            def otool(command):
                if command[1] == "-l":
                    return ""
                if command[2] == str(binary):
                    return f"{binary}:\n\t@rpath/libfoo.dylib (compatibility version 1)\n\t/usr/lib/libSystem.B.dylib (compatibility version 1)\n"
                return f"{library}:\n\t@rpath/libfoo.dylib (compatibility version 1)\n"

            # when
            with patch("gd_oracle.output", side_effect=otool):
                inventory = macos_linked_libraries((binary,))
            # then
            self.assertEqual(inventory[str(library.resolve())], sha256(library))
            self.assertIn("macOS shared cache:/usr/lib/libSystem.B.dylib", inventory)

    def test_should_write_private_fontconfig_aliases(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            # when
            config = write_config(folder)
            content = config.read_text()
            # then
            self.assertIn("Liberation Mono", content)
            self.assertIn(str(folder / "fonts"), content)


if __name__ == "__main__":
    unittest.main()
