import unittest

from scripts.validate_manual_publish import ManualPublishError, validate_manual_publish


def intent(release_type="patch", version="0.1.1"):
    return {"publish": True, "type": release_type, "version": version}


class ManualPublishTests(unittest.TestCase):
    def test_semantic_bump_matches_label_intent(self):
        validate_manual_publish(intent(), "bump", "patch", "")
        with self.assertRaisesRegex(ManualPublishError, "does not match"):
            validate_manual_publish(intent(), "bump", "minor", "")

    def test_exact_version_matches_cargo_intent(self):
        validate_manual_publish(intent("minor", "0.2.0"), "exact", "", "0.2.0")
        with self.assertRaisesRegex(ManualPublishError, "does not match"):
            validate_manual_publish(intent("minor", "0.2.0"), "exact", "", "0.3.0")

    def test_exact_version_requires_valid_semver(self):
        with self.assertRaisesRegex(ManualPublishError, "unsupported Cargo version"):
            validate_manual_publish(intent(), "exact", "", "0.1.1+build")

    def test_none_intent_cannot_be_manually_published(self):
        with self.assertRaisesRegex(ManualPublishError, "publishable"):
            validate_manual_publish(
                {"publish": False, "type": "none", "version": "0.1.0"},
                "bump",
                "patch",
                "",
            )


if __name__ == "__main__":
    unittest.main()
