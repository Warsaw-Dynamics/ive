"""Contracts for the developer-only prebuilt Graphviz WASM reference."""

import json
from pathlib import Path
import tempfile
import unittest

from compare import wasm_reference_identity
from prepare_hpcc_wasm import extract_package
from summarize_wasm_reference import summarize


class HpccWasmTests(unittest.TestCase):
    def test_should_reject_unpinned_package_archive(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "package.tgz"
            archive.write_bytes(b"untrusted")
            # when / then
            with self.assertRaisesRegex(ValueError, "differs from pinned SHA-256"):
                extract_package(archive, root / "extracted")

    def test_should_reject_reviewed_flag_on_prebuilt_reference(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            manifest.write_text(json.dumps({"kind": "third-party-prebuilt-graphviz-wasm",
                                            "reviewed": True,
                                            "source_provenance_reviewed": False,
                                            "vendor_source_equivalence_verified": False}))
            # when / then
            with self.assertRaisesRegex(ValueError, "unreviewed prebuilt package"):
                wasm_reference_identity(manifest)

    def test_should_reject_claim_of_verified_source_equivalence(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            manifest.write_text(json.dumps({"kind": "third-party-prebuilt-graphviz-wasm",
                                            "reviewed": False,
                                            "source_provenance_reviewed": False,
                                            "vendor_source_equivalence_verified": True}))
            # when / then
            with self.assertRaisesRegex(ValueError, "unreviewed prebuilt package"):
                wasm_reference_identity(manifest)

    def test_should_count_full_json_identity_separately_from_geometry(self):
        # given
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            case = root / "chain"
            case.mkdir()
            (case / "oracle.json").write_text('{"bb":"0,0,1,1","label":"oracle"}')
            (case / "candidate.json").write_text('{"bb":"0,0,1,1","label":"candidate"}')
            (root / "report.json").write_text(json.dumps({
                "oracle_wasm_reference": {"reviewed": False},
                "cases": [{"name": "chain.dot", "status": "incomplete",
                           "geometry": {"exact": True}, "svg": {"raster": {"exact": True}}}]}))
            # when
            result = summarize(root)
            # then
            self.assertEqual(result["counts"]["full_json_identity"], 0)
            self.assertEqual(result["counts"]["geometry_exact"], 1)
            self.assertEqual(result["counts"]["svg_pixels_exact"], 1)


if __name__ == "__main__":
    unittest.main()
