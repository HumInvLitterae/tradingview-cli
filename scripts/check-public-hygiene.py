#!/usr/bin/env python3
"""Reject machine-specific user-home paths in tracked text files."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import Iterable


MAC_HOME = "/" + "Users" + "/"
LINUX_HOME = "/" + "home" + "/"
SYNTHETIC_URL = (
    "file://"
    + MAC_HOME
    + "example/TradingView.app/Contents/Resources/app.asar/app/window/index.html"
)
SYNTHETIC_LINE = f'"{SYNTHETIC_URL}",'

ALLOWED_LINES = {
    "crates/cdp/src/transport.rs": frozenset({SYNTHETIC_LINE}),
    "crates/cli/src/ops/status.rs": frozenset({SYNTHETIC_LINE}),
}


@dataclass(frozen=True)
class Detector:
    name: str
    pattern: re.Pattern[str]


@dataclass(frozen=True)
class Violation:
    path: str
    line_number: int
    detector: str


DETECTORS = (
    Detector(
        "macos_user_home",
        re.compile(re.escape(MAC_HOME) + r"[A-Za-z0-9._-]+/"),
    ),
    Detector(
        "macos_username_alternation",
        re.compile(
            re.escape(MAC_HOME)
            + r"[A-Za-z0-9._-]+\|"
            + re.escape(MAC_HOME)
        ),
    ),
    Detector(
        "linux_user_home",
        re.compile(re.escape(LINUX_HOME) + r"[A-Za-z0-9._-]+/"),
    ),
    Detector(
        "windows_user_home_backslash",
        re.compile(r"[A-Za-z]:\\+Users\\+[^\\/\r\n]+\\+", re.IGNORECASE),
    ),
    Detector(
        "windows_user_home_slash",
        re.compile(r"[A-Za-z]:/Users/[^/\r\n]+/", re.IGNORECASE),
    ),
)


def scan_entries(
    entries: Iterable[tuple[str, bytes]],
    *,
    require_allowed_fixtures: bool,
) -> list[Violation]:
    violations: list[Violation] = []
    allowed_hits = {path: 0 for path in ALLOWED_LINES}

    for path, raw in entries:
        if b"\0" in raw:
            continue

        text = raw.decode("utf-8", errors="replace")
        allowed = ALLOWED_LINES.get(path, frozenset())
        for line_number, line in enumerate(text.splitlines(), start=1):
            stripped = line.strip()
            matching = [
                detector.name
                for detector in DETECTORS
                if detector.pattern.search(line)
            ]
            if not matching:
                continue
            if stripped in allowed:
                allowed_hits[path] += 1
                continue
            violations.extend(
                Violation(path, line_number, detector) for detector in matching
            )

    if require_allowed_fixtures:
        for path, count in allowed_hits.items():
            if count != 1:
                violations.append(
                    Violation(path, 0, f"allowed_fixture_count_{count}_expected_1")
                )

    return violations


def repository_entries() -> tuple[list[tuple[str, bytes]], Path]:
    root_result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    root = Path(root_result.stdout.decode("utf-8").strip())
    files_result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    paths = [
        value.decode("utf-8", errors="surrogateescape")
        for value in files_result.stdout.split(b"\0")
        if value
    ]
    entries: list[tuple[str, bytes]] = []
    for path in paths:
        full_path = root / path
        if not os.path.lexists(full_path):
            continue
        if full_path.is_symlink():
            raw = os.readlink(full_path).encode("utf-8", errors="surrogateescape")
        else:
            raw = full_path.read_bytes()
        entries.append((path, raw))
    return entries, root


def run_self_test() -> None:
    allowed_entries = [
        (path, (next(iter(lines)) + "\n").encode("utf-8"))
        for path, lines in ALLOWED_LINES.items()
    ]
    assert not scan_entries(allowed_entries, require_allowed_fixtures=True)

    same_value_elsewhere = scan_entries(
        [("docs/example.md", (SYNTHETIC_LINE + "\n").encode("utf-8"))],
        require_allowed_fixtures=False,
    )
    assert same_value_elsewhere

    changed_value_at_allowed_path = scan_entries(
        [
            (
                "crates/cdp/src/transport.rs",
                ("\"file://" + MAC_HOME + "another/private.txt\"\n").encode(
                    "utf-8"
                ),
            )
        ],
        require_allowed_fixtures=False,
    )
    assert changed_value_at_allowed_path

    forbidden_examples = (
        "prefix " + MAC_HOME + "local-user/project",
        "regex " + MAC_HOME + "local-user|" + MAC_HOME,
        "prefix " + LINUX_HOME + "local-user/project",
        "prefix " + "C:" + "\\" + "Users" + "\\" + "local-user\\project",
        "prefix " + "C:/" + "Users/local-user/project",
    )
    violations = scan_entries(
        [
            (f"docs/forbidden-{index}.md", (value + "\n").encode("utf-8"))
            for index, value in enumerate(forbidden_examples)
        ],
        require_allowed_fixtures=False,
    )
    assert len({violation.path for violation in violations}) == len(
        forbidden_examples
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run deterministic detector and exact-allowlist checks",
    )
    args = parser.parse_args()

    if args.self_test:
        run_self_test()
        print("public hygiene self-test passed")
        return 0

    try:
        entries, _root = repository_entries()
    except (OSError, subprocess.CalledProcessError):
        print(
            "public hygiene check could not read all tracked repository files",
            file=sys.stderr,
        )
        return 2

    violations = scan_entries(entries, require_allowed_fixtures=True)
    if violations:
        print(
            f"public hygiene check failed with {len(violations)} violation(s):",
            file=sys.stderr,
        )
        for violation in violations:
            location = (
                f"{violation.path}:{violation.line_number}"
                if violation.line_number
                else violation.path
            )
            print(f"- {location}: {violation.detector}", file=sys.stderr)
        return 1

    print(f"public hygiene check passed: {len(entries)} tracked files inspected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
