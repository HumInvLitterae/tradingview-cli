#!/usr/bin/env python3

import shutil
import subprocess
import sys


EXPECTED_NODE_VERSION = "v24.21.0"
TEST_NAME = (
    "ops::data::strategy::tests::"
    "javascript_equity_preserves_branch_identity_and_zero"
)


def main() -> int:
    node = shutil.which("node")
    if node is None:
        print(
            "equity JavaScript contract requires Node.js "
            f"{EXPECTED_NODE_VERSION.removeprefix('v')}",
            file=sys.stderr,
        )
        return 1

    version = subprocess.run(
        [node, "--version"],
        check=False,
        capture_output=True,
        text=True,
    )
    observed = version.stdout.strip()
    if version.returncode != 0 or observed != EXPECTED_NODE_VERSION:
        print(
            "equity JavaScript contract requires Node.js "
            f"{EXPECTED_NODE_VERSION}; observed {observed or 'unavailable'}",
            file=sys.stderr,
        )
        return 1

    result = subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "tradingview-cli",
            "--lib",
            TEST_NAME,
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ],
        check=False,
    )
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
