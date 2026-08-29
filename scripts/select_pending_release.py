#!/usr/bin/env python3
"""Select the oldest merged release intent without a completed release unit."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

try:
    from .resolve_release_intent import ResolutionError, gh_json, resolve
except ImportError:
    from resolve_release_intent import ResolutionError, gh_json, resolve


def release_exists(repo: str, tag: str) -> bool:
    try:
        gh_json(f"repos/{repo}/releases/tags/{tag}")
        return True
    except ResolutionError as error:
        message = str(error)
        if "Not Found" in message or "HTTP 404" in message:
            return False
        raise


def select_pending(repo: str) -> dict[str, Any]:
    pulls = gh_json(
        f"repos/{repo}/pulls?state=closed&base=main&sort=created&direction=asc&per_page=100"
    )
    merged = [pull for pull in pulls if pull.get("merged_at") and pull.get("merge_commit_sha")]
    for pull in merged:
        try:
            intent = resolve(repo, pull["merge_commit_sha"])
        except ResolutionError:
            continue
        if not intent.get("publish"):
            continue
        tag = f"release/{intent['version']}"
        if release_exists(repo, tag):
            continue
        return intent
    raise ResolutionError("no pending release intent was found")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        intent = select_pending(args.repo)
        args.output.write_text(json.dumps(intent, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except (ResolutionError, OSError, json.JSONDecodeError) as error:
        print(f"release-queue: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
