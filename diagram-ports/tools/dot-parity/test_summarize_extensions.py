"""Extension report aggregation checks."""

import hashlib
import json
from pathlib import Path
import unittest

from summarize_extensions import target_summary


class ExtensionSummaryTests(unittest.TestCase):
    def test_should_match_ranked_self_projection_provenance(self):
        # given
        directory = Path(__file__).resolve().parent
        project = directory.parents[1]
        provenance = json.loads((directory / "fixtures-extensions/projection-provenance.json").read_text())
        paths = {"source_sha256": project / provenance["source"],
                 "generator_sha256": project / provenance["generator"],
                 "dot_sha256": directory / "fixtures-extensions" / provenance["fixture"]}
        # when
        actual = {field: hashlib.sha256(path.read_bytes()).hexdigest()
                  for field, path in paths.items()}
        # then
        self.assertEqual(actual, {field: provenance[field] for field in paths})

    def test_should_count_json_structure_svg_pixels_and_native_gd_separately(self):
        # given
        static = {"oracle": {"sha256": "static"}, "cases": [{"name": "case.dot", "status": "mismatch",
                  "source_sha256": "source", "geometry": {"exact": False, "differences":
                  [{"path": "objects.%1", "oracle": True, "candidate": False}]},
                  "svg": {"exact_bytes": False, "raster": {"exact": True, "different_pixels": 0}},
                  "png": {"different_pixels": 5}}]}
        gd = {"oracle": {"sha256": "gd"}, "cases": [{"name": "case.dot", "status": "mismatch",
              "source_sha256": "source", "geometry": {"exact": True, "differences": []},
              "svg": {"exact_bytes": False}, "png": {"native_graphviz":
              {"exact": True, "different_pixels": 0, "oracle_size": [10, 10],
               "candidate_size": [10, 10]}}}]}
        # when
        summary = target_summary(static, gd, {"case.dot": ["shape"]})
        # then
        self.assertEqual(summary["counts"]["static_json_exact"], 0)
        self.assertEqual(summary["cases"][0]["static_json_structure_fields"], 1)
        self.assertEqual(summary["counts"]["static_svg_pixels_exact"], 1)
        self.assertEqual(summary["counts"]["native_gd_png_exact"], 1)


if __name__ == "__main__":
    unittest.main()
