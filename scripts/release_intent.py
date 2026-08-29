#!/usr/bin/env python3
"""Validate and serialize the immutable PR release intent."""

from __future__ import annotations

import argparse
import base64
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SEMVER_RE = re.compile(
    r"^(?P<major>0|[1-9][0-9]*)\."
    r"(?P<minor>0|[1-9][0-9]*)\."
    r"(?P<patch>0|[1-9][0-9]*)"
    r"(?:-(?P<stage>dev|beta)\.(?P<stage_number>[1-9][0-9]*))?$"
)
TYPE_NAMES = frozenset(("major", "minor", "patch", "none"))
CHANNEL_NAMES = frozenset(("stable", "beta", "dev"))
RELEASE_LABEL_PREFIXES = ("type:", "channel:")
STAGE_ORDER = {None: 2, "dev": 0, "beta": 1}


class IntentError(ValueError):
    """A release label or version contract is invalid."""


@dataclass(frozen=True)
class Version:
    major: int
    minor: int
    patch: int
    stage: str | None = None
    stage_number: int | None = None

    @property
    def core(self) -> tuple[int, int, int]:
        return (self.major, self.minor, self.patch)


def parse_version(value: str) -> Version:
    match = SEMVER_RE.fullmatch(value)
    if match is None:
        raise IntentError(f"unsupported Cargo version: {value!r}")
    stage = match.group("stage")
    number = match.group("stage_number")
    return Version(
        major=int(match.group("major")),
        minor=int(match.group("minor")),
        patch=int(match.group("patch")),
        stage=stage,
        stage_number=int(number) if number else None,
    )


def _labels_by_group(labels: list[str]) -> tuple[str, str]:
    unknown = sorted(
        label
        for label in labels
        if label.startswith(RELEASE_LABEL_PREFIXES)
        and not (
            label in {f"type:{name}" for name in TYPE_NAMES}
            or label in {f"channel:{name}" for name in CHANNEL_NAMES}
        )
    )
    if unknown:
        raise IntentError(f"unknown release labels: {', '.join(unknown)}")

    type_labels = [label[5:] for label in labels if label.startswith("type:")]
    channel_labels = [label[8:] for label in labels if label.startswith("channel:")]
    if len(type_labels) != 1:
        raise IntentError("exactly one type:* label is required")
    if len(channel_labels) != 1:
        raise IntentError("exactly one channel:* label is required")
    if type_labels[0] not in TYPE_NAMES:
        raise IntentError(f"invalid release type: {type_labels[0]!r}")
    if channel_labels[0] not in CHANNEL_NAMES:
        raise IntentError(f"invalid release channel: {channel_labels[0]!r}")
    return type_labels[0], channel_labels[0]


def _validate_channel(version: Version, channel: str) -> None:
    if channel == "stable" and version.stage is not None:
        raise IntentError("channel:stable requires a stable Cargo version")
    if channel in {"beta", "dev"} and version.stage != channel:
        raise IntentError(f"channel:{channel} requires -{channel}.N")


def _validate_transition(base: Version, head: Version, release_type: str) -> None:
    if release_type == "none":
        if base != head:
            raise IntentError("type:none requires Cargo version to remain unchanged")
        return

    if base.core == head.core and (base.stage is not None or head.stage is not None):
        if base.stage is None:
            raise IntentError("a prerelease train cannot start without a core version bump")
        if STAGE_ORDER[head.stage] < STAGE_ORDER[base.stage]:
            raise IntentError("prerelease channel must advance dev -> beta -> stable")
        if STAGE_ORDER[head.stage] == STAGE_ORDER[base.stage]:
            if head.stage_number is None or base.stage_number is None:
                raise IntentError("prerelease stage number is required")
            if head.stage_number <= base.stage_number:
                raise IntentError("prerelease stage number must increase")
        return

    expected = {
        "major": (base.major + 1, 0, 0),
        "minor": (base.major, base.minor + 1, 0),
        "patch": (base.major, base.minor, base.patch + 1),
    }[release_type]
    if head.core != expected:
        raise IntentError(
            f"type:{release_type} requires core version "
            f"{expected[0]}.{expected[1]}.{expected[2]}"
        )


def validate_intent(
    labels: list[str], base_version: str, head_version: str
) -> dict[str, Any]:
    release_type, channel = _labels_by_group(labels)
    base = parse_version(base_version)
    head = parse_version(head_version)
    _validate_channel(head, channel)
    _validate_transition(base, head, release_type)
    return {
        "labels": sorted(labels),
        "type": release_type,
        "channel": channel,
        "base_version": base_version,
        "version": head_version,
        "publish": release_type != "none",
    }


def cargo_version_from_toml(content: str) -> str:
    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith("version") and "=" in stripped:
            _, value = stripped.split("=", 1)
            value = value.split("#", 1)[0].strip().strip('"\'')
            parse_version(value)
            return value
    raise IntentError("Cargo.toml package version was not found")


def _gh_json(endpoint: str) -> Any:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise IntentError("GH_TOKEN is required for GitHub metadata lookup")
    result = subprocess.run(
        ["gh", "api", endpoint],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "GH_TOKEN": token},
    )
    if result.returncode != 0:
        raise IntentError(result.stderr.strip() or f"GitHub API failed: {endpoint}")
    return json.loads(result.stdout)


def _cargo_content(repo: str, ref: str) -> str:
    payload = _gh_json(f"repos/{repo}/contents/Cargo.toml?ref={ref}")
    if payload.get("encoding") != "base64":
        raise IntentError("GitHub Cargo.toml response is not base64 encoded")
    return base64.b64decode(payload["content"]).decode("utf-8")


def intent_from_github(
    repo: str,
    pr_number: int,
    head_sha: str,
    base_sha: str,
    output: Path,
) -> dict[str, Any]:
    pull = _gh_json(f"repos/{repo}/pulls/{pr_number}")
    if pull.get("head", {}).get("sha") != head_sha:
        raise IntentError("PR head SHA changed while Label Gate was running")
    labels = [item["name"] for item in pull.get("labels", [])]
    base_version = cargo_version_from_toml(_cargo_content(repo, base_sha))
    head_version = cargo_version_from_toml(_cargo_content(repo, head_sha))
    intent = validate_intent(labels, base_version, head_version)
    intent.update(
        {
            "pull_request": pr_number,
            "head_sha": head_sha,
            "base_sha": base_sha,
            "merge_commit_sha": None,
            "run_url": os.environ.get("GITHUB_SERVER_URL", "https://github.com")
            + f"/{repo}/actions/runs/{os.environ.get('GITHUB_RUN_ID', 'unknown')}",
        }
    )
    output.write_text(json.dumps(intent, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return intent


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--labels-file", type=Path)
    parser.add_argument("--base-version")
    parser.add_argument("--head-version")
    parser.add_argument("--repo")
    parser.add_argument("--pr-number", type=int)
    parser.add_argument("--head-sha")
    parser.add_argument("--base-sha")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.repo:
            if not all((args.pr_number, args.head_sha, args.base_sha)):
                raise IntentError("GitHub mode requires PR number and both SHAs")
            intent_from_github(
                args.repo, args.pr_number, args.head_sha, args.base_sha, args.output
            )
        else:
            if not args.labels_file or not args.base_version or not args.head_version:
                raise IntentError("fixture mode requires labels file and both versions")
            labels = json.loads(args.labels_file.read_text(encoding="utf-8"))
            intent = validate_intent(labels, args.base_version, args.head_version)
            args.output.write_text(
                json.dumps(intent, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
    except (IntentError, OSError, json.JSONDecodeError) as error:
        print(f"release-intent: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
