# Snapshot live smoke

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes a test/tooling slice for the `v0.8.0` snapshot lane.

## Purpose / Big Picture

`tv snapshot <SYMBOL>` is implemented as a Desktop-free single-symbol evidence packet, but its live behavior depends on TradingView scanner, symbol info, and fundamentals reads. This slice adds an opt-in ignored Rust integration test so maintainers can check the live JSON contract without making CI depend on TradingView availability.

After this change, a maintainer can run one explicit smoke command to verify that live snapshot output still contains the expected source metadata, section structure, section-level success or error shape, and follow-up hints.

## Progress

- [x] (2026-05-06T00:00Z) Created this ExecPlan and archived the completed snapshot workflow docs plan.
- [x] (2026-05-06T00:00Z) Add ignored `live_snapshot` integration test.
- [x] (2026-05-06T00:00Z) Add development docs for the opt-in smoke.
- [x] (2026-05-06T00:00Z) Update v0.8 roadmap, plan index, and changelog.
- [x] (2026-05-06T00:00Z) Validate normal tests, docs, packaging script syntax, and hygiene.
- [x] (2026-05-06T00:00Z) Commit the slice.

## Surprises & Discoveries

- Existing live smoke tests already provided the right shape: ignored Rust integration tests, explicit environment gates, `CARGO_BIN_EXE_tv`, and public-safe panic summaries.

## Decision Log

- Decision: Add an ignored Rust integration test instead of a script.
  Rationale: Existing live smoke checks for chart observation and lab bars use ignored Rust tests with explicit environment gates, so snapshot should use the same pattern.
  Date/Author: 2026-05-06 / Codex.

- Decision: Do not change `tv snapshot` behavior or payload in this slice.
  Rationale: The goal is live contract evidence, not a new command surface or data source.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Added `crates/cli/tests/live_snapshot.rs`, an ignored integration test gated by `TV_LIVE_SNAPSHOT_SMOKE=1`. It runs the test-built `tv snapshot` binary for configurable public symbols and checks source metadata, requested symbol, quote/info/fundamentals section shapes, top-level errors, and next-action hints without printing raw JSON.

Updated development docs, v0.8 roadmap, plan index, and changelog. Archived the completed snapshot workflow docs plan.

Validation passed with focused snapshot tests, full workspace tests, formatting, clippy, metadata, packaging script syntax, diff check, and hygiene grep. The hygiene grep reported existing policy language, archived validation-command examples, and this plan's safety wording; no new local path, credential, raw target id, account-local metadata, or raw live payload was added.

## Context and Orientation

The current snapshot payload is a normal JSON envelope with `command: "snapshot"`. Its `data` contains Desktop-free source metadata, `requested_symbol`, best resolved symbols, `sections.quote`, `sections.info`, `sections.fundamentals`, top-level `errors`, and `next_action_hints`.

The live smoke must validate public contract fields only. It must not print raw JSON output, raw live responses, target ids, account-local metadata, local absolute paths, cookies, tokens, or authorization values.

## Plan of Work

Add `crates/cli/tests/live_snapshot.rs` with one `#[ignore]` test gated by `TV_LIVE_SNAPSHOT_SMOKE=1`. Use `CARGO_BIN_EXE_tv` to run the test-built binary. Support optional CSV environment variables for symbols, fundamentals groups, fields, and positive repeat count.

The test should run `tv snapshot <SYMBOL>` for each symbol and validate:

- command exits successfully;
- envelope is `success: true`, `command: "snapshot"`;
- source metadata marks a Desktop-free, non-mutating read;
- requested symbol matches the input symbol;
- quote, info, and fundamentals sections exist;
- each section is either successful with `data` or failed with a public-safe `error`;
- at least one section succeeds;
- top-level `errors` and `next_action_hints` are arrays.

Update `docs/development.md` with the opt-in command and environment variables. Update `docs/v0.8-roadmap.md`, `docs/plans/README.md`, and `CHANGELOG.md` to record this as live contract evidence, not snapshot surface expansion.

## Concrete Steps

From the repository root:

    cargo test -p tradingview-cli --test live_snapshot
    cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional live smoke:

    TV_LIVE_SNAPSHOT_SMOKE=1 cargo test -p tradingview-cli --test live_snapshot -- --ignored --nocapture

With explicit fields or groups:

    TV_LIVE_SNAPSHOT_SMOKE=1 TV_LIVE_SNAPSHOT_GROUPS=earnings,dividends cargo test -p tradingview-cli --test live_snapshot -- --ignored --nocapture
    TV_LIVE_SNAPSHOT_SMOKE=1 TV_LIVE_SNAPSHOT_FIELDS=price_earnings_ttm,next_dividend_date cargo test -p tradingview-cli --test live_snapshot -- --ignored --nocapture

Run hygiene checks over public docs and packaged assets before committing. Existing policy language and archived validation commands are acceptable; newly added local paths, credentials, raw target ids, account-local metadata, or raw live payloads are not.

## Validation and Acceptance

Acceptance is met when:

- `cargo test -p tradingview-cli --test live_snapshot` compiles and reports the ignored smoke without running it;
- focused snapshot contract tests still pass;
- full workspace validation passes;
- development docs explain how to run the opt-in smoke without recording private local setup;
- public docs do not treat snapshot as batch, JSONL, chart-backed, screenshot-backed, or experimental bars evidence.

## Idempotence and Recovery

The ignored test is safe to rerun because it only performs Desktop-free reads. If TradingView live data is unavailable during the opt-in smoke, keep the failure local and do not paste raw output into tracked docs.

## Interfaces and Dependencies

No CLI option, JSON payload, Rust public API, dependency, or release package allowlist changes are part of this plan.

## Open Questions

None. Snapshot surface expansion remains deferred.
