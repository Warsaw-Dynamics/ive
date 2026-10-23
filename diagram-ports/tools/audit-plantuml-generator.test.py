"""Behavioral notice-coverage guards for the separately audited generator closure."""
import hashlib
import importlib.util
from pathlib import Path
import tempfile
import sys
import unittest

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location("generator_audit", Path(__file__).with_name("audit-plantuml-generator.py"))
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)
GRANT = "Copyright (c) 2017 Redox OS Developers\n\nMIT License\n\nPermission is hereby granted, free of charge.\n"


def package(name="redox_syscall", version="0.5.18", license="MIT"):
    return {"name": name, "version": version, "license": license}


def coverage(selected, spdx="MIT"):
    return {"packages": [{"name": selected["name"], "version": selected["version"], "grants": [{
        "notice": "notice.md", "spdx": spdx,
        "textSha256": hashlib.sha256(GRANT.encode()).hexdigest(),
    }]}]}


class NoticeCoverageTests(unittest.TestCase):
    def test_should_reject_missing_exact_package_notice_coverage(self):
        # given
        selected = package()
        # when / then
        with self.assertRaisesRegex(ValueError, "redox_syscall 0.5.18.*coverage"):
            AUDIT.validate_notice_coverage([selected], {"packages": []}, Path("/tmp"))

    def test_should_reject_removed_retained_copyright(self):
        # given
        selected = package()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            text = f"## MIT License\n\nUsed by:\n- [redox_syscall 0.5.18](https://example.test)\n\n```\n{GRANT}```\n"
            (root / "notice.md").write_text(text.replace("Copyright (c) 2017 Redox OS Developers\n", ""))
            # when / then
            with self.assertRaisesRegex(ValueError, "redox_syscall 0.5.18.*retained grant"):
                AUDIT.validate_notice_coverage([selected], coverage(selected), root)

    def test_should_reject_grant_associated_with_another_package_version(self):
        # given
        selected = package()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "notice.md").write_text(f"## MIT License\n\nUsed by:\n- [redox_syscall 9.9.9](https://example.test)\n\n```\n{GRANT}```\n")
            # when / then
            with self.assertRaisesRegex(ValueError, "redox_syscall 0.5.18.*retained grant"):
                AUDIT.validate_notice_coverage([selected], coverage(selected), root)

    def test_should_reject_incomplete_compound_license_coverage(self):
        # given
        selected = package("unicode-ident", "1.0.27", "(MIT OR Apache-2.0) AND Unicode-3.0")
        # when / then
        with self.assertRaisesRegex(ValueError, "unicode-ident 1.0.27.*license"):
            AUDIT.validate_notice_coverage([selected], coverage(selected, "Apache-2.0"), Path("/tmp"))

    def test_should_accept_retained_selected_alternate_license(self):
        # given
        selected = package("version_check", "0.9.5", "MIT/Apache-2.0")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "notice.md").write_text(f"## MIT License\n\nUsed by:\n- [version_check 0.9.5](https://example.test)\n\n```\n{GRANT}```\n")
            # when
            result = AUDIT.validate_notice_coverage([selected], coverage(selected), root)
            # then
            self.assertIsNone(result)


if __name__ == "__main__":
    unittest.main()
