#!/usr/bin/env python3

import shutil
import subprocess
import sys


EXPECTED_NODE_VERSION = "v24.18.0"
TEST_NAMES = [
    "javascript_three_point_probe_contract_is_bounded_and_verified",
    "javascript_three_point_production_contract_is_bounded_and_verified",
]


def main() -> int:
    node = shutil.which("node")
    if node is None:
        print(
            "Three-point drawing JavaScript contract requires Node.js "
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
            "Three-point drawing JavaScript contract requires Node.js "
            f"{EXPECTED_NODE_VERSION}; observed {observed or 'unavailable'}",
            file=sys.stderr,
        )
        return 1

    for test_name in TEST_NAMES:
        result = subprocess.run(
            [
                "cargo",
                "test",
                "-p",
                "tradingview-cli",
                "--test",
                "live_three_point_drawing_capability",
                test_name,
                "--",
                "--ignored",
                "--exact",
                "--nocapture",
            ],
            check=False,
        )
        if result.returncode != 0:
            return result.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
