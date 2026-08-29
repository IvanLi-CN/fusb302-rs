import unittest

from scripts.recovery_guard import RecoveryError, validate_recovery_inputs


class RecoveryGuardTests(unittest.TestCase):
    def test_requires_typed_confirmation_and_full_lowercase_sha(self):
        with self.assertRaisesRegex(RecoveryError, "confirmation"):
            validate_recovery_inputs("0.1.0", "a" * 40, "recover")
        with self.assertRaisesRegex(RecoveryError, "full lowercase"):
            validate_recovery_inputs("0.1.0", "A" * 40, "recover-fusb302")
        with self.assertRaisesRegex(RecoveryError, "full lowercase"):
            validate_recovery_inputs("0.1.0", "a" * 39, "recover-fusb302")

    def test_rejects_path_like_versions(self):
        with self.assertRaisesRegex(RecoveryError, "invalid release version"):
            validate_recovery_inputs("release/0.1.0", "a" * 40, "recover-fusb302")
        validate_recovery_inputs("0.1.0", "a" * 40, "recover-fusb302")


if __name__ == "__main__":
    unittest.main()
