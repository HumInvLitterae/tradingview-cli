#!/usr/bin/env python3

import shutil
import subprocess
import sys

from required_tests import run_required_tests

EXPECTED_NODE_VERSION = "v24.21.0"
TEST_NAME = (
    "ops::indicator::tests::"
    "javascript_indicator_add_contract_verifies_immediate_name_and_inputs"
)


def main() -> int:
    node = shutil.which("node")
    if node is None:
        print(
            "Indicator insertion JavaScript contract requires Node.js "
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
            "Indicator insertion JavaScript contract requires Node.js "
            f"{EXPECTED_NODE_VERSION}; observed {observed or 'unavailable'}",
            file=sys.stderr,
        )
        return 1

    return run_required_tests(
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
    )


if __name__ == "__main__":
    raise SystemExit(main())
