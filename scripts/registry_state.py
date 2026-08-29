#!/usr/bin/env python3
"""Read a crates.io version without publishing or changing registry state."""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from typing import Any


class RegistryError(RuntimeError):
    """The registry response was unavailable or invalid."""


def lookup(crate: str, version: str) -> dict[str, Any]:
    request = urllib.request.Request(f"https://crates.io/api/v1/crates/{crate}/{version}")
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            payload = json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return {"exists": False, "crate": crate, "version": version}
        raise RegistryError(f"crates.io returned HTTP {error.code}") from error
    except urllib.error.URLError as error:
        raise RegistryError(f"crates.io lookup failed: {error}") from error
    version_data = payload.get("version", {})
    if version_data.get("num") != version:
        raise RegistryError(f"crates.io returned an unexpected version for {crate}")
    return {
        "exists": True,
        "crate": crate,
        "version": version,
        "yanked": bool(version_data.get("yanked")),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--crate", default="fusb302")
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(lookup(args.crate, args.version), sort_keys=True))
    except (RegistryError, OSError, json.JSONDecodeError) as error:
        print(f"registry-state: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
