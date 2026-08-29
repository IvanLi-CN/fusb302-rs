#!/usr/bin/env python3
"""Resolve a merged commit to the immutable Label Gate intent artifact."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


class ResolutionError(RuntimeError):
    """The merged commit has no trustworthy release intent."""


def gh_json(endpoint: str) -> Any:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise ResolutionError("GH_TOKEN is required")
    result = subprocess.run(
        ["gh", "api", endpoint, "-H", "Accept: application/vnd.github+json"],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "GH_TOKEN": token},
    )
    if result.returncode != 0:
        raise ResolutionError(result.stderr.strip() or f"GitHub API failed: {endpoint}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise ResolutionError(f"invalid GitHub JSON for {endpoint}: {error}") from error


def merged_pull_request(repo: str, merge_sha: str) -> dict[str, Any]:
    pulls = gh_json(f"repos/{repo}/commits/{merge_sha}/pulls?per_page=100")
    matches = [
        pull
        for pull in pulls
        if pull.get("merge_commit_sha") == merge_sha
        and pull.get("state") == "closed"
    ]
    if len(matches) != 1:
        raise ResolutionError(
            f"expected exactly one closed PR for merge SHA {merge_sha}, found {len(matches)}"
        )
    return matches[0]


def label_gate_run(repo: str, head_sha: str) -> int:
    payload = gh_json(
        f"repos/{repo}/actions/workflows/label-gate.yml/runs"
        "?event=pull_request_target&status=completed&per_page=100"
    )
    runs = [
        run
        for run in payload.get("workflow_runs", [])
        if run.get("head_sha") == head_sha and run.get("conclusion") == "success"
    ]
    if not runs:
        raise ResolutionError(f"no successful Label Gate run for head SHA {head_sha}")
    return int(sorted(runs, key=lambda run: run.get("created_at", ""), reverse=True)[0]["id"])


def download_intent(repo: str, run_id: int) -> dict[str, Any]:
    temp_dir = Path(tempfile.mkdtemp(prefix="fusb302-release-intent-"))
    try:
        result = subprocess.run(
            [
                "gh",
                "run",
                "download",
                str(run_id),
                "--repo",
                repo,
                "--name",
                "release-intent",
                "--dir",
                str(temp_dir),
            ],
            check=False,
            capture_output=True,
            text=True,
            env=os.environ,
        )
        if result.returncode != 0:
            raise ResolutionError(result.stderr.strip() or "failed to download release intent")
        intent_path = temp_dir / "release-intent.json"
        if not intent_path.is_file():
            raise ResolutionError("Label Gate artifact has no release-intent.json")
        return json.loads(intent_path.read_text(encoding="utf-8"))
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)


def resolve(repo: str, merge_sha: str) -> dict[str, Any]:
    pull = merged_pull_request(repo, merge_sha)
    head_sha = pull.get("head", {}).get("sha")
    base_sha = pull.get("base", {}).get("sha")
    if not head_sha or not base_sha:
        raise ResolutionError("merged PR did not include head and base SHAs")
    run_id = label_gate_run(repo, head_sha)
    intent = download_intent(repo, run_id)
    expected = {
        "pull_request": pull["number"],
        "head_sha": head_sha,
        "base_sha": base_sha,
    }
    for key, value in expected.items():
        if intent.get(key) != value:
            raise ResolutionError(f"intent {key} does not match merged PR")
    intent["merge_commit_sha"] = merge_sha
    intent["label_gate_run_id"] = run_id
    intent["run_url"] = intent.get("run_url") or ""
    return intent


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--merge-sha", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        intent = resolve(args.repo, args.merge_sha)
        args.output.write_text(json.dumps(intent, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except (ResolutionError, OSError, json.JSONDecodeError) as error:
        print(f"release-resolve: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
