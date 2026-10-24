"""Developer-only comparator contracts; no Graphviz executable is needed."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import json
import xml.etree.ElementTree as ET

from PIL import Image

from compare import (check_oracle_svg, compare_case, compare_geometry, compare_png,
                     fixture_provenance, verify_wasm_command)
from generate_plantuml_corpus import project
from prepare_linux_gd_fonts import write_config
from svg_raster_oracle import verified_identity as verified_svg_raster_identity


class CompareTests(unittest.TestCase):
    def test_should_match_numeric_geometry_with_equivalent_spelling(self):
        # given
        oracle = {"bb": "0,0,80,154.400", "objects": [{"_gvid": 0, "name": "a", "pos": "40,134.4"}],
                  "edges": [{"id": "e0", "tail": 0, "head": 0, "pos": "e,40,20 40,134.4"}]}
        candidate = {"width": 80.0, "height": 154.4, "objects": [{"name": "a", "pos": "40.0,20.0"}],
                     "edges": [{"id": "e0", "tail": "a", "head": "a", "points": [[40, 20], [40, 134.4]]}]}
        # when
        result = compare_geometry(oracle, candidate)
        # then
        self.assertTrue(result["exact"])

    def test_should_match_graphviz_style_candidate_json0_geometry(self):
        # given
        oracle = {"bb": "0,0,80,154.4", "objects": [{"_gvid": 0, "name": "a", "pos": "40,134.4", "width": "1"}],
                  "edges": [{"id": "e0", "tail": 0, "head": 0, "pos": "e,40,20 40,134.4"}]}
        candidate = {"bb": "0,0,80.0,154.400", "objects": [{"_gvid": 0, "name": "a", "pos": "40.0,134.40", "width": "1.0"}],
                     "edges": [{"id": "e0", "tail": 0, "head": 0, "pos": "e,40.0,20.0 40,134.40"}]}
        # when
        result = compare_geometry(oracle, candidate)
        # then
        self.assertTrue(result["exact"])

    def test_should_report_route_and_label_differences(self):
        # given
        oracle = {"bb": "0,0,80,154.4", "objects": [],
                  "edges": [{"id": "e0", "pos": "e,40,40 40,114", "lp": "40,77"}]}
        candidate = {"width": 80, "height": 154.4, "objects": [],
                     "edges": [{"id": "e0", "tail": None, "head": None,
                                "points": [[40, 40.4], [40, 113.4]], "lp": [40, 76.4]}]}
        # when
        result = compare_geometry(oracle, candidate)
        # then
        self.assertEqual(result["difference_count"], 2)
        self.assertEqual({item["path"] for item in result["differences"]}, {"edges.e0.points", "edges.e0.lp"})

    def test_should_count_a_single_changed_png_pixel(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            oracle = Path(directory) / "oracle.png"
            candidate = Path(directory) / "candidate.png"
            Image.new("RGBA", (2, 2), "white").save(oracle)
            changed = Image.new("RGBA", (2, 2), "white")
            changed.putpixel((1, 0), (0, 0, 0, 255))
            changed.save(candidate)
            # when
            result = compare_png(oracle, candidate)
            # then
            self.assertFalse(result["exact"])
            self.assertEqual(result["different_pixels"], 1)

    def test_should_keep_unpinned_pixel_match_diagnostic(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            oracle = {kind: folder / f"oracle.{kind}" for kind in ("json", "svg", "png")}
            candidate = {kind: folder / f"candidate.{kind}" for kind in ("json", "svg", "png")}
            candidate["svg_png"] = folder / "candidate-svg.png"
            oracle["json"].write_text('{"bb":"0,0,10,10","objects":[],"edges":[]}')
            candidate["json"].write_text('{"width":10,"height":10,"objects":[],"edges":[]}')
            for path in (oracle["svg"], candidate["svg"]):
                path.write_text('<svg xmlns="http://www.w3.org/2000/svg"/>')
            for path in (oracle["png"], candidate["png"], candidate["svg_png"]):
                Image.new("RGBA", (1, 1), "white").save(path)
            # when
            diagnostic = compare_case("empty.dot", oracle, candidate, False)
            certified = compare_case("empty.dot", oracle, candidate, True)
            # then
            self.assertEqual(diagnostic["status"], "diagnostic_match")
            self.assertEqual(certified["status"], "exact")

    def test_should_project_parallel_plantuml_relations_with_stable_ids(self):
        # given
        source = "@startuml\nclass A\nclass B\nA --> B : one\nA ..> B : two\nA --> A\n@enduml\n"
        # when
        dot, nodes, edges = project(source, "uml_parallel")
        # then
        self.assertEqual((nodes, edges), (2, 3))
        self.assertIn('n0 -> n1 [label="one"]', dot)
        self.assertIn('n0 -> n1 [label="two", style=dashed]', dot)
        self.assertIn("n0 -> n0;", dot)

    def test_should_reject_stale_plantuml_projection_provenance(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            fixture = folder / "case.dot"
            manifest = folder / "provenance.json"
            fixture.write_text("digraph G {}")
            manifest.write_text('{"cases":[{"fixture":"case.dot","dot_sha256":"stale"}]}')
            # when
            with self.assertRaisesRegex(ValueError, "stale hash"):
                fixture_provenance(manifest, [fixture])

    def test_should_report_native_graphviz_png_dimensions_without_svg_rasterizer(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            oracle = {kind: folder / f"oracle.{kind}" for kind in ("json", "svg")}
            oracle["native_png"] = folder / "oracle-native.png"
            candidate = {kind: folder / f"candidate.{kind}" for kind in ("json", "svg", "png")}
            oracle["json"].write_text('{"bb":"0,0,10,10","objects":[],"edges":[]}')
            candidate["json"].write_text('{"bb":"0,0,10,10","objects":[],"edges":[]}')
            oracle["svg"].write_text('<svg/>')
            candidate["svg"].write_text('<svg/>')
            Image.new("RGBA", (2, 2), "white").save(oracle["native_png"])
            Image.new("RGBA", (3, 2), "white").save(candidate["png"])
            # when
            result = compare_case("native.dot", oracle, candidate, False)
            # then
            self.assertEqual(result["status"], "mismatch")
            self.assertEqual(result["png"]["native_graphviz"]["oracle_size"], [2, 2])
            self.assertEqual(result["png"]["native_graphviz"]["candidate_size"], [3, 2])

    def test_should_configure_private_legacy_urw_family_aliases(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            # when
            config = write_config(folder)
            root = ET.parse(config).getroot()
            names = [entry.text for entry in root.findall("./match/edit/string")]
            # then
            self.assertEqual(set(names), {"Nimbus Mono L", "Nimbus Sans L", "Nimbus Roman No9 L"})
            self.assertEqual(root.findtext("dir"), str(folder / "fonts"))

    def test_should_compare_svg_pixels_without_candidate_native_png(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            oracle = {kind: folder / f"oracle.{kind}" for kind in ("json", "svg", "png")}
            candidate = {kind: folder / f"candidate.{kind}" for kind in ("json", "svg")}
            candidate["svg_png"] = folder / "candidate-svg.png"
            oracle["json"].write_text('{"bb":"0,0,10,10","objects":[],"edges":[]}')
            candidate["json"].write_text('{"bb":"0,0,10,10","objects":[],"edges":[]}')
            oracle["svg"].write_text("<svg/>")
            candidate["svg"].write_text("<svg/>")
            Image.new("RGBA", (2, 2), "white").save(oracle["png"])
            Image.new("RGBA", (2, 2), "white").save(candidate["svg_png"])
            # when
            result = compare_case("svg.dot", oracle, candidate, False)
            # then
            self.assertEqual(result["status"], "incomplete")
            self.assertTrue(result["svg"]["raster"]["exact"])
            self.assertFalse(result["png"]["available"])

    def test_should_require_matching_svg_rasterizer_pin_and_review(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "rasterizer.json"
            identity = {"binary_sha256": "pinned", "fonts": {"config_sha256": "pinned-fonts"},
                        "renderer": "Inkscape"}
            manifest.write_text(json.dumps({**identity, "candidate_png_renderer": "resvg", "reviewed": False}))
            # when
            with patch("svg_raster_oracle.identify", return_value=identity):
                result = verified_svg_raster_identity(["inkscape", "{input}", "{output}"], manifest, "linux")
            # then
            self.assertFalse(result["certified"])
            self.assertFalse(result["svg_pixel_oracle_certified"])
            manifest.write_text(json.dumps({**identity, "binary_sha256": "stale", "reviewed": True}))
            with patch("svg_raster_oracle.identify", return_value=identity):
                with self.assertRaisesRegex(ValueError, "differs"):
                    verified_svg_raster_identity(["inkscape", "{input}", "{output}"], manifest, "linux")

    def test_should_require_candidate_command_to_use_pinned_wasm_artifact(self):
        # given
        artifact = Path("target/wasm32-unknown-unknown/release/diagram_web.wasm")
        command = ["node", "run_wasm.mjs", "--wasm", str(artifact), "--input", "{input}"]
        # when
        verify_wasm_command(command, artifact)
        # then
        with self.assertRaisesRegex(ValueError, "pinned WASM artifact"):
            verify_wasm_command(["node", "run_wasm.mjs", "--wasm", "web/pkg/diagram_web.wasm"], artifact)

    def test_should_reject_oracle_without_required_html_table_cells(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            svg = Path(directory) / "oracle.svg"
            svg.write_text('<svg><text>order</text></svg>')
            # when
            with self.assertRaisesRegex(ValueError, "required capability marker 'Order'"):
                check_oracle_svg(svg, ["Order", "42"], "html_table_cells.dot")


if __name__ == "__main__":
    unittest.main()
