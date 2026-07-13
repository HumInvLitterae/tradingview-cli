#!/usr/bin/env python3

import shutil
import subprocess
import sys


EXPECTED_NODE_VERSION = "v24.18.0"
TEST_NAME = (
    "ops::data::study_values::tests::"
    "javascript_collector_distinguishes_same_name_instances_and_is_total"
)


def main() -> int:
    node = shutil.which("node")
    if node is None:
        print(
            "study-value JavaScript contract requires Node.js "
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
            "study-value JavaScript contract requires Node.js "
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
