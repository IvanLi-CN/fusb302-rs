import unittest

from scripts.check_quality_gates import main as check_quality_gates


class WorkflowContractTests(unittest.TestCase):
    def test_checked_in_quality_and_release_contracts_are_aligned(self):
        self.assertEqual(check_quality_gates(), 0)


if __name__ == "__main__":
    unittest.main()
