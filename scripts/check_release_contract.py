#!/usr/bin/env python3
"""Check the public release surfaces for one immutable release unit."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


class ContractError(RuntimeError):
    """A release surface is absent or bound to a different source."""


def gh_json(endpoint: str) -> Any:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise ContractError("GH_TOKEN is required")
    result = subprocess.run(
        ["gh", "api", endpoint, "-H", "Accept: application/vnd.github+json"],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "GH_TOKEN": token},
    )
    if result.returncode != 0:
        raise ContractError(result.stderr.strip() or f"GitHub API failed: {endpoint}")
    return json.loads(result.stdout)


def published_crate(crate: str, version: str) -> dict[str, Any]:
    request = urllib.request.Request(f"https://crates.io/api/v1/crates/{crate}/{version}")
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            payload = json.load(response)
    except (urllib.error.HTTPError, urllib.error.URLError) as error:
        raise ContractError(f"crates.io lookup failed for {crate}@{version}: {error}") from error
    version_data = payload.get("version", {})
    if version_data.get("num") != version or version_data.get("yanked"):
        raise ContractError(f"crates.io does not expose an active {crate}@{version}")
    return version_data


def git_ref_sha(repo: str, tag: str) -> str:
    payload = gh_json(f"repos/{repo}/git/ref/tags/{tag}")
    obj = payload.get("object", {})
    if obj.get("type") == "tag":
        obj = gh_json(f"repos/{repo}/git/tags/{obj['sha']}").get("object", {})
    sha = obj.get("sha")
    if not sha:
        raise ContractError(f"tag {tag} has no commit target")
    return sha


def github_release(repo: str, tag: str) -> dict[str, Any]:
    payload = gh_json(f"repos/{repo}/releases/tags/{tag}")
    if payload.get("draft") or not payload.get("published_at"):
        raise ContractError(f"GitHub Release {tag} is not published")
    return payload


def check_contract(repo: str, version: str, source_sha: str, crate: str = "fusb302") -> dict[str, Any]:
    tag = f"release/{version}"
    published_crate(crate, version)
    tag_sha = git_ref_sha(repo, tag)
    if tag_sha != source_sha:
        raise ContractError(f"{tag} points to {tag_sha}, expected {source_sha}")
    release = github_release(repo, tag)
    if release.get("tag_name") != tag:
        raise ContractError("GitHub Release tag does not match release unit")
    return {
        "crate": f"{crate}@{version}",
        "tag": tag,
        "source_sha": source_sha,
        "github_release_id": release.get("id"),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--crate", default="fusb302")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        result = check_contract(args.repo, args.version, args.source_sha, args.crate)
        encoded = json.dumps(result, indent=2, sort_keys=True)
        print(encoded)
        if args.output:
            args.output.write_text(encoded + "\n", encoding="utf-8")
    except (ContractError, OSError, json.JSONDecodeError) as error:
        print(f"release-contract: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
