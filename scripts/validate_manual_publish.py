#!/usr/bin/env python3
"""Validate manual publication inputs against an immutable release intent."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from .release_intent import IntentError, parse_version
except ImportError:
    from release_intent import IntentError, parse_version


class ManualPublishError(ValueError):
    """Manual publication inputs do not match the release intent."""


def validate_manual_publish(
    intent: dict[str, object], version_mode: str, bump: str, version: str
) -> None:
    if intent.get("publish") is not True:
        raise ManualPublishError("manual publication requires a publishable intent")

    intent_type = intent.get("type")
    intent_version = intent.get("version")
    if not isinstance(intent_type, str) or not isinstance(intent_version, str):
        raise ManualPublishError("release intent is missing type or version")

    if version_mode == "bump":
        if bump not in {"major", "minor", "patch"}:
            raise ManualPublishError("bump must be major, minor, or patch")
        if intent_type != bump:
            raise ManualPublishError(
                f"bump {bump!r} does not match intent type:{intent_type}"
            )
        return

    if version_mode == "exact":
        if not version:
            raise ManualPublishError("exact mode requires a version")
        try:
            parse_version(version)
        except IntentError as error:
            raise ManualPublishError(str(error)) from error
        if version != intent_version:
            raise ManualPublishError(
                f"version {version!r} does not match intent version {intent_version!r}"
            )
        return

    raise ManualPublishError("version_mode must be bump or exact")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--intent", type=Path, required=True)
    parser.add_argument("--version-mode", required=True)
    parser.add_argument("--bump", default="")
    parser.add_argument("--version", default="")
    args = parser.parse_args()
    try:
        intent = json.loads(args.intent.read_text(encoding="utf-8"))
        validate_manual_publish(intent, args.version_mode, args.bump, args.version)
    except (ManualPublishError, OSError, json.JSONDecodeError) as error:
        print(f"manual-publish: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
