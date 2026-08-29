import json
import unittest
from pathlib import Path

from scripts.release_intent import IntentError, cargo_version_from_toml, validate_intent


class ReleaseIntentTests(unittest.TestCase):
    def test_stable_minor_bump(self):
        intent = validate_intent(
            ["type:minor", "channel:stable"], "0.1.0", "0.2.0"
        )
        self.assertEqual(intent["version"], "0.2.0")
        self.assertTrue(intent["publish"])

    def test_beta_and_dev_require_matching_suffix(self):
        self.assertEqual(
            validate_intent(["type:patch", "channel:beta"], "0.1.0", "0.1.1-beta.1")[
                "channel"
            ],
            "beta",
        )
        with self.assertRaisesRegex(IntentError, "channel:dev"):
            validate_intent(["type:patch", "channel:dev"], "0.1.0", "0.1.1-beta.1")

    def test_prerelease_train_is_monotonic(self):
        validate_intent(["type:minor", "channel:dev"], "0.2.0-dev.1", "0.2.0-dev.2")
        validate_intent(["type:minor", "channel:beta"], "0.2.0-dev.2", "0.2.0-beta.1")
        validate_intent(["type:minor", "channel:stable"], "0.2.0-beta.1", "0.2.0")
        with self.assertRaisesRegex(IntentError, "advance"):
            validate_intent(["type:minor", "channel:dev"], "0.2.0-beta.1", "0.2.0-dev.2")

    def test_none_is_exact_no_release(self):
        intent = validate_intent(["type:none", "channel:stable"], "0.1.0", "0.1.0")
        self.assertFalse(intent["publish"])
        with self.assertRaisesRegex(IntentError, "unchanged"):
            validate_intent(["type:none", "channel:stable"], "0.1.0", "0.1.1")

    def test_unknown_duplicate_and_invalid_labels_fail(self):
        for labels, message in (
            (["type:minor", "type:patch", "channel:stable"], "exactly one type"),
            (["type:minor"], "exactly one channel"),
            (["type:minor", "channel:stable", "channel:beta"], "exactly one channel"),
            (["type:minor", "channel:stable", "type:security"], "unknown"),
        ):
            with self.subTest(labels=labels), self.assertRaisesRegex(IntentError, message):
                validate_intent(labels, "0.1.0", "0.2.0")

    def test_build_metadata_and_leading_zero_are_rejected(self):
        for value in ("0.2.0+build", "0.2.0-dev.01"):
            with self.subTest(value=value), self.assertRaises(IntentError):
                validate_intent(["type:minor", "channel:dev"], "0.1.0", value)

    def test_cargo_version_extraction_is_package_scoped_enough(self):
        self.assertEqual(
            cargo_version_from_toml('[package]\nname = "fusb302"\nversion = "0.1.0"\n'),
            "0.1.0",
        )


if __name__ == "__main__":
    unittest.main()
