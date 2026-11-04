#!/usr/bin/env python3
"""Check exported schemas against production fixtures using python-jsonschema."""

import argparse
import importlib.metadata
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys
import tomllib

from required_tests import run_required_tests

from jsonschema import Draft202012Validator

EXPECTED_VERSION = "4.26.0"


def check_fixtures():
    for case in json.load(sys.stdin):
        schema = case["schema"]
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator(schema)
        for value in case["valid"]:
            validator.validate(value)
        for value in case["invalid"]:
            if validator.is_valid(value):
                raise AssertionError("Malformed output passed its exported schema")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    if importlib.metadata.version("jsonschema") != EXPECTED_VERSION:
        parser.error(f"requires jsonschema=={EXPECTED_VERSION}")
    if args.fixtures:
        check_fixtures()
        return 0
    root = Path(__file__).resolve().parent.parent
    with open(root / "mise.toml", "rb") as file:
        expected = "v" + tomllib.load(file)["tools"]["node"]
    executable = shutil.which("node")
    if executable is None:
        parser.error(f"production OHLCV fixtures require Node.js {expected}")
    node = subprocess.run(
        [executable, "--version"], capture_output=True, text=True, check=False
    )
    if node.returncode != 0 or node.stdout.strip() != expected:
        parser.error(f"production OHLCV fixtures require Node.js {expected}")
    env = os.environ.copy()
    env["TV_SCHEMA_TEST_PYTHON"] = sys.executable
    env.setdefault("CARGO_BUILD_JOBS", "1")
    env.setdefault("RUST_TEST_THREADS", "1")
    return run_required_tests(
        ["cargo", "test", "--locked", "-p", "tradingview-cli", "--lib",
         "app::schema::tests::output_schemas_match_production_fixtures", "--", "--ignored", "--exact"],
        cwd=root,
        env=env,
    )


if __name__ == "__main__":
    raise SystemExit(main())
