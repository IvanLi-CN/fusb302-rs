#!/usr/bin/env python3
"""Guard an Actions-only release recovery before GitHub writes."""

from __future__ import annotations

import argparse
import base64
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from typing import Any

from release_intent import cargo_version_from_toml
from source_guard import assert_source


class RecoveryError(RuntimeError):
    """Recovery inputs or external state are unsafe."""


def gh_json(endpoint: str) -> Any:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise RecoveryError("GH_TOKEN is required")
    result = subprocess.run(
        ["gh", "api", endpoint, "-H", "Accept: application/vnd.github+json"],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "GH_TOKEN": token},
    )
    if result.returncode != 0:
        raise RecoveryError(result.stderr.strip() or f"GitHub API failed: {endpoint}")
    return json.loads(result.stdout)


def assert_recoverable(repo: str, version: str, source_sha: str, confirmation: str) -> dict[str, str]:
    if confirmation != "recover-fusb302":
        raise RecoveryError("typed recovery confirmation is required")
    if len(source_sha) != 40 or any(char not in "0123456789abcdef" for char in source_sha):
        raise RecoveryError("source SHA must be a full lowercase commit SHA")
    if not version or "/" in version or " " in version:
        raise RecoveryError("invalid release version")
    try:
        assert_source(repo, source_sha)
    except Exception as error:
        raise RecoveryError(str(error)) from error
    manifest = gh_json(f"repos/{repo}/contents/Cargo.toml?ref={source_sha}")
    if manifest.get("encoding") != "base64":
        raise RecoveryError("recovery Cargo.toml response is not base64 encoded")
    source_version = cargo_version_from_toml(
        base64.b64decode(manifest["content"]).decode("utf-8")
    )
    if source_version != version:
        raise RecoveryError(
            f"recovery source declares {source_version}, expected {version}"
        )
    comparison = gh_json(f"repos/{repo}/compare/{source_sha}...main")
    if comparison.get("status") not in {"ahead", "identical"}:
        raise RecoveryError("recovery source is not reachable from main")
    runs = gh_json(
        f"repos/{repo}/actions/workflows/ci.yml/runs"
        f"?branch=main&head_sha={source_sha}&status=completed&per_page=100"
    ).get("workflow_runs", [])
    if not any(run.get("conclusion") == "success" for run in runs):
        raise RecoveryError("no successful Rust CI run exists for recovery source")
    request = urllib.request.Request(f"https://crates.io/api/v1/crates/fusb302/{version}")
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            version_data = json.load(response).get("version", {})
    except (urllib.error.HTTPError, urllib.error.URLError) as error:
        raise RecoveryError(f"crates.io lookup failed: {error}") from error
    if version_data.get("num") != version or version_data.get("yanked"):
        raise RecoveryError(f"crates.io does not expose an active fusb302@{version}")
    try:
        tag = gh_json(f"repos/{repo}/git/ref/tags/release/{version}")
    except RecoveryError as error:
        message = str(error)
        if "Not Found" not in message and "HTTP 404" not in message:
            raise
        tag = None
    if tag:
        tag_object = tag.get("object", {})
        if tag_object.get("type") == "commit":
            tag_sha = tag_object.get("sha")
        elif tag_object.get("type") == "tag":
            tag_sha = gh_json(f"repos/{repo}/git/tags/{tag_object['sha']}").get("object", {}).get("sha")
        else:
            tag_sha = None
        if tag_sha and tag_sha != source_sha:
            raise RecoveryError(f"release/{version} points to {tag_sha}, expected {source_sha}")
    return {"repository": repo, "version": version, "source_sha": source_sha}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--confirmation", required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(assert_recoverable(args.repo, args.version, args.source_sha, args.confirmation)))
    except (RecoveryError, OSError, json.JSONDecodeError) as error:
        print(f"release-recovery: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
