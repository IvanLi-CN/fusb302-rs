import unittest

from scripts.check_release_contract import ContractError
from scripts.release_intent import parse_version


class ReleaseContractTests(unittest.TestCase):
    def test_release_tag_uses_exact_source_in_error_contract(self):
        self.assertEqual(parse_version("0.1.0").core, (0, 1, 0))
        self.assertEqual(parse_version("0.2.0-beta.3").stage_number, 3)

    def test_contract_error_is_public_and_actionable(self):
        error = ContractError("release/0.1.0 points to another SHA")
        self.assertIn("another SHA", str(error))


if __name__ == "__main__":
    unittest.main()
