import unittest
from unittest.mock import patch

from scripts.release_surfaces import ensure_draft


class ReleaseSurfaceTests(unittest.TestCase):
    def test_prepare_creates_exact_tag_before_release_without_target_override(self):
        repo = "IvanLi-CN/fusb302-rs"
        tag = "release/0.1.0"
        source_sha = "b" * 40
        calls = []

        def record_command(arguments):
            calls.append(arguments)
            return "{}"

        with (
            patch("scripts.release_surfaces.tag_commit_sha", side_effect=[None, source_sha]),
            patch("scripts.release_surfaces.release", side_effect=[None, {"tag_name": tag}]),
            patch("scripts.release_surfaces.gh_command", side_effect=record_command),
        ):
            result = ensure_draft(repo, tag, source_sha, "fusb302 0.1.0", "notes", False)

        self.assertEqual(result["tag_name"], tag)
        self.assertEqual(calls[0][:4], ["api", "--method", "POST", f"repos/{repo}/git/refs"])
        self.assertIn(f"ref=refs/tags/{tag}", calls[0])
        self.assertIn(f"sha={source_sha}", calls[0])
        self.assertEqual(calls[1][:3], ["release", "create", tag])
        self.assertNotIn("--target", calls[1])


if __name__ == "__main__":
    unittest.main()
