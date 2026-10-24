"""Regression checks for the isolated, bundled-font GD oracle setup."""

import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import unittest

from prepare_linux_ofl_gd_fonts import ROOT, prepare


@unittest.skipUnless(platform.system() == "Linux" and shutil.which("fc-match"), "Linux fontconfig required")
class OflGdFontsTest(unittest.TestCase):
    def test_should_resolve_times_and_courier_to_the_bundled_ofl_files(self):
        # given
        with tempfile.TemporaryDirectory() as temporary:
            config = prepare(Path(temporary))
            environment = {**os.environ, "FONTCONFIG_FILE": str(config)}

            # when
            selections = {
                family: subprocess.check_output(
                    ["fc-match", "-f", "%{file}", family], env=environment, text=True, timeout=10
                ).strip()
                for family in ("Times", "Nimbus Roman No9 L", "Courier", "Nimbus Mono L")
            }

            # then
            self.assertEqual(selections["Times"], str(ROOT / "web/fonts/LiberationSerif-Regular.ttf"))
            self.assertEqual(selections["Nimbus Roman No9 L"], selections["Times"])
            self.assertEqual(selections["Courier"], str(ROOT / "web/fonts/LiberationMono-Regular.ttf"))
            self.assertEqual(selections["Nimbus Mono L"], selections["Courier"])
