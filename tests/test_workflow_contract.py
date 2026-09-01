import unittest
from pathlib import Path

from scripts.check_quality_gates import OIDRUNE_NOTIFY_REF, main as check_quality_gates


NOTIFY_WORKFLOW = Path(__file__).parents[1] / ".github" / "workflows" / "notify-release-failure.yml"


class WorkflowContractTests(unittest.TestCase):
    def test_checked_in_quality_and_release_contracts_are_aligned(self):
        self.assertEqual(check_quality_gates(), 0)

    def test_release_failure_notification_has_immutable_oidrune_contract(self):
        workflow = NOTIFY_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(OIDRUNE_NOTIFY_REF, workflow)
        self.assertIn("name: Send Oidrune notification", workflow)
        self.assertIn("permissions:\n      id-token: write", workflow)
        self.assertIn("outcome: ${{ github.event.workflow_run.conclusion }}", workflow)
        self.assertIn("summary:", workflow)
        self.assertNotIn("gateway_url:", workflow)
        self.assertNotIn("oidc_audience:", workflow)
        self.assertNotIn("SHOUTRRR_URL", workflow)
        self.assertNotIn("secrets:", workflow)


if __name__ == "__main__":
    unittest.main()
