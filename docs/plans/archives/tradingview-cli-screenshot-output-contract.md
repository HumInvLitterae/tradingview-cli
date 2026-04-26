# Harden screenshot output contract

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust CLI already lets users choose an exact screenshot file path with `tv screenshot --region <full|chart> --output <PATH>`. Upstream PR #43 added an `output_dir` option to the old Node project so agent workflows could save screenshots somewhere readable. Rust can satisfy that need through the existing explicit `--output <PATH>` contract, so this slice locks the behavior in tests and docs instead of adding a second output API.

## Progress

- [x] (2026-04-25T12:14:41Z) Confirmed the working tree was clean and inspected current screenshot code, CLI contract tests, README, and upstream PR #43 context.
- [x] (2026-04-25T12:16:10Z) Added tests for parent directory creation and missing `--output` validation.
- [x] (2026-04-25T12:16:10Z) Updated README and upstream PR triage notes.
- [x] (2026-04-25T12:18:36Z) Ran validation and bounded live smoke.
- [x] (2026-04-25T12:19:10Z) Prepared the completed work for commit.

## Surprises & Discoveries

- No surprises yet.

## Decision Log

- Decision: Do not add `--output-dir`.
  Rationale: Rust already accepts an explicit file path with `--output <PATH>` and creates parent directories. Adding directory-based auto-naming would introduce new filename, collision, and region suffix policy without improving the core CLI contract.
  Date/Author: 2026-04-25 / Codex

- Decision: Treat `--output <PATH>` as required.
  Rationale: Requiring the caller to choose a file path avoids hidden default directories that may be unreadable to an agent or downstream adapter.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The screenshot output contract is now locked without adding `--output-dir`. `tv screenshot` continues to require an explicit `--output <PATH>` file path, creates missing parent directories, and reports the exact path in both `file_path` and `output_path`. Upstream PR #43 is addressed for Rust as a documentation and test-hardening follow-up rather than a new flag.

Automated validation passed:

    cargo test screenshot_full_creates_parent_output_directory -- --nocapture
    cargo test screenshot_requires_output_before_connecting -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

Live smoke used `tv screenshot --region full --output target/smoke/screenshot-output-contract/full.png` with an explicit CDP target id because multiple TradingView chart targets were open. The command returned `region: "full"`, `file_path` and `output_path` equal to the requested path, and positive `size_bytes`; `file` confirmed the output was a PNG image.

## Context and Orientation

Screenshot commands are parsed in `src/cli.rs` and dispatched from `src/main.rs` to `src/ops/screenshot.rs`. The `screenshot_full` and `screenshot_chart` operations both call `write_screenshot`, which creates the parent directory and writes the PNG file. Successful payloads include both `file_path` and `output_path` with the requested path.

Upstream PR #43 was about letting old MCP/Claude Desktop workflows place screenshots in a readable directory. Rust is CLI-first and does not have the old batch screenshot API, so an exact output file path is the narrower and clearer contract.

## Plan of Work

Add one operation-level unit test proving `screenshot_full` creates a missing parent directory and returns the exact requested path in both `file_path` and `output_path`. Add one CLI contract test proving `tv screenshot --region full` without `--output` fails validation before CDP connection.

Update README to tell users and agent workflows to pass an explicit readable `--output <PATH>`. Update the upstream PR triage note to mark PR #43 addressed for Rust by the existing output path contract plus these tests.

## Concrete Steps

From the repository root, run:

    cargo test screenshot_full_creates_parent_output_directory -- --nocapture
    cargo test screenshot_requires_output_before_connecting -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

If TradingView Desktop is available, run bounded smoke:

    tv screenshot --region full --output target/smoke/screenshot-output-contract/full.png

The command should write a PNG, return `region: "full"`, report positive `size_bytes`, and echo the requested path through `file_path` and `output_path`.

## Validation and Acceptance

Automated acceptance is that the targeted tests pass and the full validation baseline passes. Behavioral acceptance is that live smoke can write a full screenshot to a nested `target/` path without pre-creating the parent directory.

## Idempotence and Recovery

The automated tests use temporary directories. Live smoke writes only under `target/`, which is a build-artifact area and can be safely removed. No TradingView chart, account, layout, alert, watchlist, Pine, drawing, replay, or tab state is changed.

## Artifacts and Notes

Relevant upstream evidence:

    PR #43 adds output_dir to the old Node screenshot tools so callers can choose a readable directory. Rust already asks for an explicit output file path and creates parent directories.

## Interfaces and Dependencies

No new crate dependency is required. No new command or flag is introduced. The public command remains:

    tv screenshot --region <full|chart> --output <PATH>

The successful payload continues to include:

    file_path: string
    output_path: string
    region: "full" | "chart"
    size_bytes: number

## Open Questions

No critical questions are open.
