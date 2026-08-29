#!/usr/bin/env python3
"""Serialize an exact-SHA manual recovery intent."""

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
    parser.add_argument("--run-url", required=True)
    args = parser.parse_args()
    payload = {
        "schema_version": 1,
        "repository": args.repository,
        "intent_kind": "recovery",
        "pull_request": None,
        "merge_commit_sha": args.source_sha,
        "head_sha": args.source_sha,
        "base_sha": None,
        "version": args.version,
        "publish": False,
        "run_url": args.run_url,
        "recovery": True,
    }
    args.output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
