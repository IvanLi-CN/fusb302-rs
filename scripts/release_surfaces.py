#!/usr/bin/env python3
"""Idempotently prepare and finalize the GitHub surfaces of a release unit."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any


class SurfaceError(RuntimeError):
    """A tag or GitHub Release conflicts with the requested source."""


RELEASE_VISIBILITY_ATTEMPTS = 5
RELEASE_VISIBILITY_DELAY_SECONDS = 1


def gh_json(endpoint: str) -> Any:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise SurfaceError("GH_TOKEN is required")
    result = subprocess.run(
        ["gh", "api", endpoint, "-H", "Accept: application/vnd.github+json"],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "GH_TOKEN": token},
    )
    if result.returncode != 0:
        raise SurfaceError(result.stderr.strip() or f"GitHub API failed: {endpoint}")
    return json.loads(result.stdout)


def gh_command(arguments: list[str]) -> str:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise SurfaceError("GH_TOKEN is required")
    result = subprocess.run(
        ["gh", *arguments],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "GH_TOKEN": token},
    )
    if result.returncode != 0:
        raise SurfaceError(result.stderr.strip() or f"gh command failed: {' '.join(arguments)}")
    return result.stdout


def ensure_tag(repo: str, tag: str, source_sha: str) -> None:
    """Create the exact tag before calling the Releases API.

    Creating a release with ``--target`` requires workflow-write permission when
    the target commit changes workflow files. The contents-write token can still
    create the tag, after which the release can safely resolve that immutable
    tag without a target override.
    """
    try:
        gh_command(
            [
                "api",
                "--method",
                "POST",
                f"repos/{repo}/git/refs",
                "-f",
                f"ref=refs/tags/{tag}",
                "-f",
                f"sha={source_sha}",
            ]
        )
    except SurfaceError:
        # A concurrent or previous attempt may have created the tag already.
        if tag_commit_sha(repo, tag) != source_sha:
            raise


def tag_commit_sha(repo: str, tag: str) -> str | None:
    try:
        ref = gh_json(f"repos/{repo}/git/ref/tags/{tag}")
    except SurfaceError as error:
        if "HTTP 404" in str(error) or "Not Found" in str(error):
            return None
        raise
    obj = ref.get("object", {})
    if obj.get("type") == "commit":
        return obj.get("sha")
    if obj.get("type") == "tag":
        return gh_json(f"repos/{repo}/git/tags/{obj['sha']}").get("object", {}).get("sha")
    raise SurfaceError(f"tag {tag} has unsupported object type {obj.get('type')!r}")


def release(repo: str, tag: str) -> dict[str, Any] | None:
    """Read a release by its complete tag without relying on list visibility."""
    try:
        payload = json.loads(
            gh_command(
                [
                    "release",
                    "view",
                    tag,
                    "--repo",
                    repo,
                    "--json",
                    "tagName,isDraft,isPrerelease,targetCommitish",
                ]
            )
        )
    except SurfaceError as error:
        if "not found" in str(error).lower() or "HTTP 404" in str(error):
            return None
        raise
    return {
        "tag_name": payload.get("tagName"),
        "draft": payload.get("isDraft"),
        "prerelease": payload.get("isPrerelease"),
        "target_commitish": payload.get("targetCommitish"),
    }


def wait_for_release(
    repo: str,
    tag: str,
    *,
    attempts: int = RELEASE_VISIBILITY_ATTEMPTS,
    delay_seconds: float = RELEASE_VISIBILITY_DELAY_SECONDS,
) -> dict[str, Any] | None:
    """Wait briefly for GitHub to expose a release created or edited by this run."""
    if attempts < 1:
        raise ValueError("attempts must be positive")

    for attempt in range(attempts):
        current = release(repo, tag)
        if current is not None:
            return current
        if attempt + 1 < attempts:
            time.sleep(delay_seconds)
    return None


def ensure_draft(
    repo: str,
    tag: str,
    source_sha: str,
    title: str,
    notes: str,
    prerelease: bool,
    *,
    visibility_attempts: int = RELEASE_VISIBILITY_ATTEMPTS,
    visibility_delay_seconds: float = RELEASE_VISIBILITY_DELAY_SECONDS,
) -> dict[str, Any]:
    existing_tag = tag_commit_sha(repo, tag)
    if existing_tag and existing_tag != source_sha:
        raise SurfaceError(f"{tag} points to {existing_tag}, expected {source_sha}")
    if existing_tag is None:
        ensure_tag(repo, tag, source_sha)

    current = release(repo, tag)
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as notes_file:
        notes_file.write(notes)
        notes_path = notes_file.name
    try:
        if current is None:
            arguments = [
                "release",
                "create",
                tag,
                "--repo",
                repo,
                "--draft",
                "--title",
                title,
                "--notes-file",
                notes_path,
            ]
            if prerelease:
                arguments.append("--prerelease")
            gh_command(arguments)
        else:
            if current.get("tag_name") != tag:
                raise SurfaceError("GitHub Release tag does not match release unit")
            if current.get("draft"):
                arguments = [
                    "release",
                    "edit",
                    tag,
                    "--repo",
                    repo,
                    "--title",
                    title,
                    "--notes-file",
                    notes_path,
                ]
                if prerelease:
                    arguments.append("--prerelease")
                gh_command(arguments)
        resolved = tag_commit_sha(repo, tag)
        if resolved != source_sha:
            raise SurfaceError(f"{tag} resolved to {resolved}, expected {source_sha}")
        final = wait_for_release(
            repo,
            tag,
            attempts=visibility_attempts,
            delay_seconds=visibility_delay_seconds,
        )
        if final is None:
            raise SurfaceError(
                f"GitHub Release {tag} was not visible after "
                f"{visibility_attempts} exact lookup attempts"
            )
        return final
    finally:
        Path(notes_path).unlink(missing_ok=True)


def finalize(
    repo: str,
    tag: str,
    prerelease: bool,
    *,
    visibility_attempts: int = RELEASE_VISIBILITY_ATTEMPTS,
    visibility_delay_seconds: float = RELEASE_VISIBILITY_DELAY_SECONDS,
) -> dict[str, Any]:
    current = release(repo, tag)
    if current is None:
        raise SurfaceError(f"GitHub Release {tag} does not exist")
    if current.get("draft"):
        arguments = ["release", "edit", tag, "--repo", repo, "--draft=false"]
        arguments.append("--prerelease=true" if prerelease else "--prerelease=false")
        gh_command(arguments)
    final = wait_for_release(
        repo,
        tag,
        attempts=visibility_attempts,
        delay_seconds=visibility_delay_seconds,
    )
    if final is None:
        raise SurfaceError(
            f"GitHub Release {tag} was not visible after "
            f"{visibility_attempts} exact lookup attempts"
        )
    if final.get("draft"):
        raise SurfaceError(f"GitHub Release {tag} did not become public")
    return final


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    prepare = subparsers.add_parser("prepare")
    prepare.add_argument("--repo", required=True)
    prepare.add_argument("--tag", required=True)
    prepare.add_argument("--source-sha", required=True)
    prepare.add_argument("--title", required=True)
    prepare.add_argument("--notes-file", type=Path, required=True)
    prepare.add_argument("--prerelease", action="store_true")

    publish = subparsers.add_parser("finalize")
    publish.add_argument("--repo", required=True)
    publish.add_argument("--tag", required=True)
    publish.add_argument("--prerelease", action="store_true")

    args = parser.parse_args()
    try:
        if args.command == "prepare":
            result = ensure_draft(
                args.repo,
                args.tag,
                args.source_sha,
                args.title,
                args.notes_file.read_text(encoding="utf-8"),
                args.prerelease,
            )
        else:
            result = finalize(args.repo, args.tag, args.prerelease)
        print(json.dumps(result, indent=2, sort_keys=True))
    except (SurfaceError, OSError, json.JSONDecodeError) as error:
        print(f"release-surfaces: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
