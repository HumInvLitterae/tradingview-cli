# `tv --version` flag

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes how to add the root version flag without changing command behavior.

## Purpose / Big Picture

The `tv` binary supports `--help` but did not support `--version`. Release archive users, packaged agents, and downstream workflows need a simple way to confirm which binary is available before troubleshooting. This change enables clap's root version flag so `tv --version` and `tv -V` print the Cargo package version.

This work is CLI surface polish. It does not change command JSON envelopes, payloads, source taxonomy semantics, fallback behavior, or exit codes for existing commands.

## Progress

- [x] (2026-05-05T01:34Z) Confirmed `target/debug/tv --version` currently fails with a validation envelope.
- [x] (2026-05-05T01:40Z) Added root clap version support and CLI contract coverage for `--version` / `-V`.
- [x] (2026-05-05T01:43Z) Updated README, packaged agent guidance, changelog, roadmap, and plan index.
- [x] (2026-05-05T01:58Z) Ran focused contract test, smoke checks, full workspace validation, packaging syntax check, and hygiene grep.
- [x] (2026-05-05T02:00Z) Committed the related changes as `7de423f feat(cli): Add version flag`.

## Surprises & Discoveries

- None so far.

## Decision Log

- Decision: Use clap root version support rather than adding a custom command or JSON-returning path.
  Rationale: `--version` is CLI metadata like `--help`, and should return ordinary stdout before application dispatch.
  Date/Author: 2026-05-05 / Codex.

- Decision: Keep version support at the root only.
  Rationale: The practical need is binary sanity checking, not subcommand-specific version output.
  Date/Author: 2026-05-05 / Codex.

## Outcomes & Retrospective

Implemented. `tv --version` and `tv -V` now print the clap root version using the Cargo package version, and the CLI contract test covers both flags. Public docs now mention `tv --version` only as a binary sanity check. Focused tests, smoke checks, full workspace validation, packaging script syntax check, and diff checks passed.

## Context and Orientation

The Cargo version is centralized in the workspace package metadata and inherited by `crates/cli/Cargo.toml`. Clap can therefore print the correct `tradingview-cli` package version without duplicating it in source code.

`--version` and `-V` are expected to behave like `--help`: they are handled by clap before command dispatch and do not use the JSON error/success envelope.

## Plan of Work

Enable clap version output on the root parser in `crates/cli/src/cli.rs`.

Add an integration contract test proving `tv --version` and `tv -V` exit successfully and print both the binary name and `CARGO_PKG_VERSION`.

Update public-facing release and packaged-agent docs with a short binary sanity-check instruction. Avoid local validation environment details or machine-specific paths.

## Concrete Steps

From the repository root, run:

    cargo test -p tradingview-cli --test cli_contract version -- --nocapture
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    target/debug/tv --version
    target/debug/tv -V
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Also run a public-doc hygiene grep to ensure no local paths, credentials, raw target ids, account-local metadata, or local validation environment notes were added.

## Validation and Acceptance

Acceptance is reached when `tv --version` and `tv -V` both exit 0 and print the current package version, all tests pass, and docs describe the flag only as a public binary sanity check.

## Idempotence and Recovery

The change is additive. If clap output changes slightly, keep the contract test focused on the binary name and package version rather than exact formatting.

## Artifacts and Notes

Validation evidence:

    cargo test -p tradingview-cli --test cli_contract version -- --nocapture
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    target/debug/tv --version
    target/debug/tv -V
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

All completed successfully. The broad public-doc hygiene grep reported existing policy text, archived validation-command examples, and secret-safety wording only.

## Interfaces and Dependencies

No new dependencies are introduced.

New user-facing flags:

    tv --version
    tv -V

Both flags print ordinary clap stdout and do not connect to TradingView Desktop.

## Open Questions

No open questions.
