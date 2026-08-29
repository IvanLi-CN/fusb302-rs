import json
import tempfile
import unittest
from pathlib import Path
from subprocess import run


class ReleaseReceiptTests(unittest.TestCase):
    def test_receipt_contains_all_surfaces_and_source(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            result = run(
                [
                    "python3",
                    "scripts/release_receipt.py",
                    "--output",
                    str(output),
                    "--repository",
                    "IvanLi-CN/fusb302-rs",
                    "--version",
                    "0.2.0",
                    "--source-sha",
                    "a" * 40,
                    "--pull-request",
                    "7",
                    "--head-sha",
                    "b" * 40,
                    "--base-sha",
                    "c" * 40,
                    "--type",
                    "minor",
                    "--channel",
                    "stable",
                    "--status",
                    "prepared",
                    "--run-url",
                    "https://github.com/IvanLi-CN/fusb302-rs/actions/runs/7",
                ],
                check=False,
            )
            self.assertEqual(result.returncode, 0)
            receipt = json.loads(output.read_text())
            self.assertEqual(receipt["source_sha"], "a" * 40)
            self.assertEqual(receipt["surfaces"]["tag"], "release/0.2.0")

    def test_recovery_receipt_allows_missing_base_and_release_labels(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            result = run(
                [
                    "python3",
                    "scripts/release_receipt.py",
                    "--output",
                    str(output),
                    "--repository",
                    "IvanLi-CN/fusb302-rs",
                    "--version",
                    "0.1.0",
                    "--source-sha",
                    "a" * 40,
                    "--head-sha",
                    "a" * 40,
                    "--type",
                    "recovery",
                    "--channel",
                    "recovery",
                    "--status",
                    "recovery",
                    "--run-url",
                    "https://github.com/IvanLi-CN/fusb302-rs/actions/runs/8",
                ],
                check=False,
            )
            self.assertEqual(result.returncode, 0)
            receipt = json.loads(output.read_text())
            self.assertEqual(receipt["status"], "recovery")
            self.assertEqual(receipt["base_sha"], "")


if __name__ == "__main__":
    unittest.main()
