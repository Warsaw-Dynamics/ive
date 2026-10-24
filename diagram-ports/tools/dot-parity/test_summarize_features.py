"""Feature report classification checks."""

import unittest

from summarize_features import classify, is_numeric_difference, summarize


class FeatureSummaryTests(unittest.TestCase):
    def test_should_classify_normalized_route_strings_as_numeric(self):
        # given
        route = {"path": "edges.e0.points", "oracle": [["27", "36.187"]],
                 "candidate": [["27", "36.303"]]}
        # when
        numeric = is_numeric_difference(route)
        # then
        self.assertTrue(numeric)

    def test_should_classify_unsupported_candidate_without_counting_geometry(self):
        # given
        case = {"name": "shape.dot", "status": "error", "detail": "unsupported DOT shape cylinder",
                "oracle_svg_raster_size": [100, 100]}
        # when
        categories = classify(case)
        # then
        self.assertEqual(categories, ["candidate_unsupported"])

    def test_should_count_numeric_and_pixel_differences_independently(self):
        # given
        report = {"cases": [{"name": "style.dot", "status": "mismatch", "geometry":
                             {"exact": False, "difference_count": 2, "differences":
                              [{"path": "graph.bb[2]", "oracle": 10.0, "candidate": 11.0},
                               {"path": "objects.%1", "oracle": True, "candidate": False}]}, "svg": {"raster":
                             {"exact": False, "different_pixels": 5}}, "png":
                             {"exact": False, "different_pixels": 7}}]}
        # when
        summary = summarize(report, {"style.dot": ["style"]})
        # then
        self.assertEqual(summary["counts"]["numeric_geometry"], 1)
        self.assertEqual(summary["counts"]["identity_or_structure"], 1)
        self.assertEqual(summary["counts"]["json_exact"], 0)
        self.assertEqual(summary["counts"]["svg_exact"], 0)
        self.assertEqual(summary["counts"]["svg_pixels"], 1)
        self.assertEqual(summary["counts"]["png_pixels"], 1)


if __name__ == "__main__":
    unittest.main()
