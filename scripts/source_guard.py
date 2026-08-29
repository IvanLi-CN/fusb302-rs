#!/usr/bin/env python3
"""Verify that a release source is a signed commit reachable from main with green CI."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from typing import Any


class SourceError(RuntimeError):
    """The proposed release source is not safe to publish."""


REQUIRED_CHECKS = ("fmt", "clippy", "test", "docs", "msrv", "package")


def gh_json(endpoint: str) -> Any:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise SourceError("GH_TOKEN is required")
    result = subprocess.run(
        ["gh", "api", endpoint, "-H", "Accept: application/vnd.github+json"],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "GH_TOKEN": token},
    )
    if result.returncode != 0:
        raise SourceError(result.stderr.strip() or f"GitHub API failed: {endpoint}")
    return json.loads(result.stdout)


def assert_source(repo: str, source_sha: str) -> dict[str, Any]:
    if len(source_sha) != 40 or any(char not in "0123456789abcdef" for char in source_sha):
        raise SourceError("source SHA must be a full lowercase commit SHA")
    commit = gh_json(f"repos/{repo}/commits/{source_sha}")
    verification = commit.get("commit", {}).get("verification", {})
    if verification.get("verified") is not True:
        raise SourceError("source commit is not cryptographically verified")
    comparison = gh_json(f"repos/{repo}/compare/{source_sha}...main")
    if comparison.get("status") not in {"ahead", "identical"}:
        raise SourceError("source commit is not reachable from main")
    checks = gh_json(f"repos/{repo}/commits/{source_sha}/check-runs?per_page=100").get(
        "check_runs", []
    )
    conclusions = {check.get("name"): check.get("conclusion") for check in checks}
    missing = [name for name in REQUIRED_CHECKS if conclusions.get(name) != "success"]
    if missing:
        raise SourceError(f"required CI checks are not green for source: {', '.join(missing)}")
    return {"source_sha": source_sha, "verified": True, "checks": list(REQUIRED_CHECKS)}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--source-sha", required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(assert_source(args.repo, args.source_sha), indent=2, sort_keys=True))
    except (SourceError, OSError, json.JSONDecodeError) as error:
        print(f"release-source: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
