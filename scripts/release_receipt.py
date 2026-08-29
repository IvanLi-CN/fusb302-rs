#!/usr/bin/env python3
"""Create a durable release receipt for a GitHub Release asset."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--pull-request", default="recovery")
    parser.add_argument("--head-sha", required=True)
    parser.add_argument("--base-sha", default="")
    parser.add_argument("--type", required=True, choices=("major", "minor", "patch", "none", "recovery"))
    parser.add_argument("--channel", required=True, choices=("stable", "beta", "dev", "recovery"))
    parser.add_argument("--status", required=True, choices=("prepared", "published", "recovery"))
    parser.add_argument("--run-url", required=True)
    args = parser.parse_args()
    payload = {
        "schema_version": 1,
        "repository": args.repository,
        "version": args.version,
        "source_sha": args.source_sha,
        "pull_request": args.pull_request,
        "head_sha": args.head_sha,
        "base_sha": args.base_sha,
        "type": args.type,
        "channel": args.channel,
        "status": args.status,
        "run_url": args.run_url,
        "surfaces": {
            "crate": f"fusb302@{args.version}",
            "tag": f"release/{args.version}",
            "github_release": f"release/{args.version}",
        },
    }
    args.output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
