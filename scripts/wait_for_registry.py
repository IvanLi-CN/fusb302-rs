#!/usr/bin/env python3
"""Poll crates.io until one active version is visible."""

from __future__ import annotations

import argparse
import json
import time

try:
    from .registry_state import RegistryError, lookup
except ImportError:
    from registry_state import RegistryError, lookup


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--crate", default="fusb302")
    parser.add_argument("--version", required=True)
    parser.add_argument("--timeout-seconds", type=int, default=600)
    parser.add_argument("--interval-seconds", type=int, default=15)
    args = parser.parse_args()
    deadline = time.monotonic() + args.timeout_seconds
    while time.monotonic() < deadline:
        try:
            state = lookup(args.crate, args.version)
            if state.get("exists") and not state.get("yanked"):
                print(json.dumps(state, sort_keys=True))
                return 0
        except RegistryError:
            pass
        time.sleep(args.interval_seconds)
    print(f"registry-state: timed out waiting for {args.crate}@{args.version}", flush=True)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
