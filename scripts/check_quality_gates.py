#!/usr/bin/env python3
"""Verify that the checked-in required checks match the Rust CI job names."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GATES_PATH = ROOT / ".github" / "quality-gates.json"
WORKFLOW_PATH = ROOT / ".github" / "workflows" / "ci.yml"
EXPECTED_JOBS = ("fmt", "clippy", "test", "docs", "msrv", "package")


def main() -> int:
    gates = json.loads(GATES_PATH.read_text(encoding="utf-8"))
    expected_checks = [f"Rust CI / {job}" for job in EXPECTED_JOBS]
    if gates.get("branch") != "main":
        return fail("quality gates must target main")
    if gates.get("required_checks") != expected_checks:
        return fail("required checks must be the six stable Rust CI check names")
    if gates.get("required_review_count") != 0:
        return fail("required review count must be zero")
    if gates.get("require_signed_commits") is not True:
        return fail("verified signed commits must be required")

    workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
    if not re.search(r"^name: Rust CI$", workflow, re.MULTILINE):
        return fail("ci workflow must be named Rust CI")
    jobs_section = workflow.split("\njobs:\n", maxsplit=1)[1]
    jobs = re.findall(r"^  ([a-z][a-z0-9-]*):$", jobs_section, re.MULTILINE)
    if tuple(jobs) != EXPECTED_JOBS:
        return fail(f"ci jobs drifted: expected {EXPECTED_JOBS}, found {tuple(jobs)}")
    for job in EXPECTED_JOBS:
        if not re.search(rf"^  {re.escape(job)}:\n    name: {re.escape(job)}$", workflow, re.MULTILINE):
            return fail(f"ci job {job} must retain its stable display name")
    return 0


def fail(message: str) -> int:
    print(f"quality-gates: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
