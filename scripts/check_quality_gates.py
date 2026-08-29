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
LABEL_WORKFLOW_PATH = ROOT / ".github" / "workflows" / "label-gate.yml"
POLICY_PATH = ROOT / ".github" / "pr-label-release.json"
RELEASE_WORKFLOW_PATH = ROOT / ".github" / "workflows" / "release.yml"
NOTIFY_WORKFLOW_PATH = ROOT / ".github" / "workflows" / "notify-release-failure.yml"
EXPECTED_RUST_JOBS = ("fmt", "clippy", "test", "docs", "msrv", "package")
EXPECTED_CHECKS = ("Label Gate", *EXPECTED_RUST_JOBS)
EXPECTED_TYPES = {"type:major", "type:minor", "type:patch", "type:none"}
EXPECTED_CHANNELS = {"channel:stable", "channel:beta", "channel:dev"}


def main() -> int:
    gates = json.loads(GATES_PATH.read_text(encoding="utf-8"))
    expected_checks = list(EXPECTED_CHECKS)
    if gates.get("branch") != "main":
        return fail("quality gates must target main")
    if gates.get("required_checks") != expected_checks:
        return fail("required checks must include Label Gate followed by the six Rust CI checks")
    if gates.get("required_review_count") != 0:
        return fail("required review count must be zero")
    if gates.get("require_signed_commits") is not True:
        return fail("verified signed commits must be required")

    workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
    if not re.search(r"^name: Rust CI$", workflow, re.MULTILINE):
        return fail("ci workflow must be named Rust CI")
    jobs_section = workflow.split("\njobs:\n", maxsplit=1)[1]
    jobs = re.findall(r"^  ([a-z][a-z0-9-]*):$", jobs_section, re.MULTILINE)
    if tuple(jobs) != EXPECTED_RUST_JOBS:
        return fail(f"ci jobs drifted: expected {EXPECTED_RUST_JOBS}, found {tuple(jobs)}")
    for job in EXPECTED_RUST_JOBS:
        if not re.search(rf"^  {re.escape(job)}:\n    name: {re.escape(job)}$", workflow, re.MULTILINE):
            return fail(f"ci job {job} must retain its stable display name")

    label_workflow = LABEL_WORKFLOW_PATH.read_text(encoding="utf-8")
    if not re.search(r"^name: Label Gate$", label_workflow, re.MULTILINE):
        return fail("label workflow must be named Label Gate")
    if not re.search(r"pull_request_target:", label_workflow):
        return fail("Label Gate must run from a trusted pull_request_target workflow")
    if "actions/checkout" not in label_workflow:
        return fail("Label Gate must check out only trusted workflow code")
    if re.search(r"github\.event\.pull_request\.head\.ref", label_workflow):
        return fail("Label Gate must not checkout a pull request head ref")

    policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    groups = {group["name"]: group for group in policy.get("label_groups", [])}
    if set(groups.get("type", {}).get("allowed", [])) != EXPECTED_TYPES:
        return fail("type label policy drifted")
    if set(groups.get("channel", {}).get("allowed", [])) != EXPECTED_CHANNELS:
        return fail("channel label policy drifted")
    if policy.get("required_quality_gate", {}).get("check_name") != "Label Gate":
        return fail("label policy must declare Label Gate as required")

    branch_protection = gates.get("branch_protection", {})
    if branch_protection.get("pull_request_only") is not True:
        return fail("main must remain pull-request-only")
    if branch_protection.get("restrict_administrators") is not True:
        return fail("administrators must be subject to the main branch gate")
    if branch_protection.get("allow_force_pushes") is not False:
        return fail("force pushes must remain disabled")

    release_workflow = RELEASE_WORKFLOW_PATH.read_text(encoding="utf-8")
    if not re.search(r"^name: Release$", release_workflow, re.MULTILINE):
        return fail("release workflow must be named Release")
    if "workflow_run:" not in release_workflow or "workflows: [Rust CI]" not in release_workflow:
        return fail("Release must consume successful Rust CI workflow runs")
    if "type: choice" not in release_workflow or "- publish" not in release_workflow:
        return fail("Release must expose an explicit manual publish dispatch mode")
    if "rust-lang/crates-io-auth-action@v1" not in release_workflow:
        return fail("Release must use crates.io OIDC authentication")
    if "CARGO_REGISTRY_TOKEN" in release_workflow:
        return fail("Release must not use the legacy CARGO_REGISTRY_TOKEN")
    if "publish-fusb302" not in release_workflow:
        return fail("manual publish must require typed confirmation")
    if "name: Upload trusted registry tooling" not in release_workflow:
        return fail("Release must publish trusted registry tooling for registry preflight")
    if "name: Download trusted registry tooling" not in release_workflow:
        return fail("Release must use trusted registry tooling during immutable source preflight")
    if '"${RUNNER_TEMP}/release-tooling/registry_state.py"' not in release_workflow:
        return fail("Release must run registry lookup from trusted tooling outside the source worktree")

    notify_workflow = NOTIFY_WORKFLOW_PATH.read_text(encoding="utf-8")
    if not re.search(r"^name: Notify release failure$", notify_workflow, re.MULTILINE):
        return fail("failure notifier must retain its stable workflow name")
    if "actions/checkout" in notify_workflow:
        return fail("failure notifier must not checkout repository code")
    if "IvanLi-CN/github-workflows/.github/workflows/release-failure-telegram.yml@main" not in notify_workflow:
        return fail("failure notifier must call the shared Telegram notifier")
    if "SHOUTRRR_URL" not in notify_workflow:
        return fail("failure notifier must require SHOUTRRR_URL")
    return 0


def fail(message: str) -> int:
    print(f"quality-gates: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
