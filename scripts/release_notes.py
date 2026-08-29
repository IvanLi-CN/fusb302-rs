#!/usr/bin/env python3
"""Render a release description from one immutable intent."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--intent", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    intent: dict[str, Any] = json.loads(args.intent.read_text(encoding="utf-8"))
    version = intent["version"]
    channel = intent.get("channel", "unknown")
    labels = ", ".join(intent.get("labels", [])) or "unknown"
    pr = intent.get("pull_request") or "unknown"
    body = "\n".join(
        (
            f"Release Unit: `fusb302@{version}`",
            f"Source SHA: `{intent['merge_commit_sha'] or intent['head_sha']}`",
            f"Pull request: `{pr}`",
            f"Labels: `{labels}`",
            f"Channel: `{channel}`",
            "",
            "This release was produced by the repository Release workflow.",
        )
    )
    args.output.write_text(body + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
