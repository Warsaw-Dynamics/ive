"""Integrity check for source-pinned styled-shortcut diagnostic cases."""

import hashlib
import json
from pathlib import Path
import unittest

from compare import fixture_paths, fixture_provenance


FIXTURES = Path(__file__).with_name("fixtures-edge-style-triangle")


class EdgeStyleTriangleGoldensTest(unittest.TestCase):
    def test_should_keep_all_three_oracle_outputs_tied_to_the_dot_sources(self):
        # given
        paths = fixture_paths(FIXTURES)
        manifest = json.loads((FIXTURES / "provenance.json").read_text())

        # when
        verified = fixture_provenance(FIXTURES / "provenance.json", paths)

        # then
        self.assertEqual(len(paths), 3)
        self.assertFalse(manifest["reviewed"])
        for fixture in paths:
            case = verified[fixture.name]
            for suffix in ("json", "svg"):
                oracle = fixture.with_suffix(f".{suffix}")
                digest = hashlib.sha256(oracle.read_bytes()).hexdigest()
                self.assertEqual(digest, case[f"{suffix}_sha256"])
