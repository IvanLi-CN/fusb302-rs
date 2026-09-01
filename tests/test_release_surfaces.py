import unittest
from unittest.mock import patch

from scripts.release_surfaces import ensure_draft, release


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

    def test_prepare_retries_when_an_existing_draft_is_not_immediately_visible(self):
        repo = "IvanLi-CN/fusb302-rs"
        tag = "release/0.2.2"
        source_sha = "a" * 40
        draft = {"tag_name": tag, "draft": True}
        calls = []

        def record_command(arguments):
            calls.append(arguments)
            return "{}"

        with (
            patch("scripts.release_surfaces.tag_commit_sha", return_value=source_sha),
            patch(
                "scripts.release_surfaces.release",
                side_effect=[draft, None, draft],
            ),
            patch("scripts.release_surfaces.gh_command", side_effect=record_command),
            patch("scripts.release_surfaces.time.sleep") as sleep,
        ):
            result = ensure_draft(
                repo,
                tag,
                source_sha,
                "fusb302 0.2.2",
                "notes",
                False,
                visibility_attempts=2,
                visibility_delay_seconds=0,
            )

        self.assertEqual(result, draft)
        self.assertEqual(calls[0][:3], ["release", "edit", tag])
        sleep.assert_called_once_with(0)

    def test_release_uses_an_exact_tag_lookup(self):
        with patch(
            "scripts.release_surfaces.gh_command",
            return_value=(
                '{"tagName":"release/0.2.2","isDraft":true,'
                '"isPrerelease":false,"targetCommitish":"main"}'
            ),
        ) as command:
            result = release("IvanLi-CN/fusb302-rs", "release/0.2.2")

        self.assertEqual(result["tag_name"], "release/0.2.2")
        self.assertEqual(
            command.call_args.args[0][:3],
            ["release", "view", "release/0.2.2"],
        )

    def test_prepare_reuses_a_published_release_when_its_tag_matches_source(self):
        repo = "IvanLi-CN/fusb302-rs"
        tag = "release/0.2.1"
        source_sha = "b" * 40
        published = {
            "tag_name": tag,
            "draft": False,
            "target_commitish": "main",
        }

        with (
            patch("scripts.release_surfaces.tag_commit_sha", return_value=source_sha),
            patch("scripts.release_surfaces.release", side_effect=[published, published]),
            patch("scripts.release_surfaces.gh_command") as command,
        ):
            result = ensure_draft(
                repo,
                tag,
                source_sha,
                "fusb302 0.2.1",
                "notes",
                False,
                visibility_attempts=1,
            )

        self.assertEqual(result, published)
        command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
