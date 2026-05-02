# Chart quote opt-in live endurance smoke

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can finish the opt-in live smoke without prior chat context.

## Purpose / Big Picture

`tv quote <SYMBOL> --source chart` now has stronger stale-data guards, but the most realistic failure mode is still live TradingView Desktop timing while switching multiple symbols in sequence. This slice adds an opt-in Rust integration smoke that can be run manually before a patch release or after chart-source quote changes. It must not run in normal CI and must not introduce a Python, Node, jq, or shell-script runtime dependency.

After this change, maintainers can run one ignored cargo test to cycle through a small list of public symbols and verify that every chart-source quote reports matching requested, observed, and chart symbols with stable readiness metadata.

## Progress

- [x] (2026-05-02) Archived the completed chart-source stable readiness ExecPlan.
- [x] (2026-05-02) Added this ExecPlan.
- [x] (2026-05-02) Added an ignored Rust integration test for opt-in chart quote sequence smoke.
- [x] (2026-05-02) Documented how to run the smoke and how to interpret failures.
- [x] (2026-05-02) Ran normal validation and confirmed the ignored smoke does not run by default.
- [x] (2026-05-02) Ran the opt-in live smoke against the default symbol sequence.
- [x] (2026-05-02) Commit the related changes.

## Surprises & Discoveries

- Observation: the repository has no existing live-smoke script directory.
  Evidence: `find scripts -maxdepth 3 -type f` lists hook and release scripts only.

- Observation: when the requested symbol is already the chart symbol, the chart quote command may take the same-symbol fast path and report one stable sample.
  Evidence: the opt-in smoke passed with `MU` reporting `stable_samples=1` and `elapsed_ms=21`; switched-symbol reads in the same run reported two stable samples.

## Decision Log

- Decision: Use an ignored Rust integration test instead of a Python or shell script.
  Rationale: the user does not want to add another runtime dependency for this narrow smoke, and Rust tests can reuse the existing `tv` test binary and `serde_json`.
  Date/Author: 2026-05-02 / Codex

- Decision: Require `TV_LIVE_CHART_QUOTE_SMOKE=1` even though the test is ignored.
  Rationale: `--ignored` alone is too easy to run accidentally. The environment gate makes the chart mutation explicit.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

Added `crates/cli/tests/live_chart_quote.rs` as an ignored integration test that drives the test-built `tv` binary and validates chart-source quote readiness through the public JSON envelope. The test has an explicit environment gate, supports configurable symbol sequences, repeat count, and optional target id, and reports only scrubbed failure summaries.

Normal validation confirms the smoke is not executed by default. The opt-in live smoke passed the default sequence `PLUG,AAPL,MSFT,IONQ,MU,PLUG`; switched-symbol reads required two stable samples, while same-symbol fast-path reads are accepted with one stable sample.

## Context and Orientation

The live smoke is intentionally separate from unit tests and CLI contract tests. It requires a running TradingView Desktop session with CDP enabled and temporarily switches the selected chart symbol. It validates the behavior of the already-built `tv` binary through its public JSON envelope.

Relevant files:

- `crates/cli/tests/live_chart_quote.rs` will contain the ignored test.
- `docs/development.md` documents live-smoke boundaries and how to run the test.
- `docs/plans/README.md` points to this active plan.

## Plan of Work

Add `crates/cli/tests/live_chart_quote.rs` with one `#[ignore]` test. The test reads:

- `TV_LIVE_CHART_QUOTE_SMOKE=1` as the required opt-in gate.
- `TV_LIVE_CHART_QUOTE_SYMBOLS` as an optional comma-separated symbol sequence, defaulting to `PLUG,AAPL,MSFT,IONQ,MU,PLUG`.
- `TV_LIVE_CHART_QUOTE_RUNS` as an optional positive repeat count, defaulting to `1`.
- `TV_LIVE_CHART_QUOTE_TARGET_ID` as an optional target selector passed as `--target-id <ID>`.

For each symbol, invoke the test-built `tv` binary as `tv [--target-id <ID>] quote <SYMBOL> --source chart`, parse stdout as JSON, and check public-safe success conditions: envelope success, matching requested/observed/chart bare symbols, `freshness_check.passed == true`, `restored == true`, and stable sample count. Switched-symbol reads must report at least two stable samples; same-symbol fast-path reads may report one stable sample because no chart switch occurred.

Failure messages should include only a scrubbed summary: requested symbol, exit code, error kind/message, observed symbol, chart symbol, stable sample count, and freshness pass state. Do not print the raw envelope, raw target id, or account-local metadata.

## Validation and Acceptance

Run:

    cargo test -p tradingview-cli --test live_chart_quote
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Optional live smoke:

    TV_LIVE_CHART_QUOTE_SMOKE=1 cargo test -p tradingview-cli --test live_chart_quote -- --ignored --nocapture

If multiple chart targets are open, first run `target/debug/tv tab list`, then:

    TV_LIVE_CHART_QUOTE_SMOKE=1 TV_LIVE_CHART_QUOTE_TARGET_ID=<ID> cargo test -p tradingview-cli --test live_chart_quote -- --ignored --nocapture

Acceptance is met when the ignored test compiles, normal test runs do not execute the live smoke, the validation baseline passes, and any optional live smoke either passes or fails with public-safe diagnostics.

## Idempotence and Recovery

Re-running the ignored test is safe but mutates the visible chart symbol during the run. The underlying quote command restores the original symbol after each item. If the smoke fails, inspect the scrubbed summary and rerun after confirming the target with `tv tab list`. Do not add the smoke to CI or release packaging as a required runtime workflow.

## Interfaces and Dependencies

No public CLI command, JSON payload, or Rust crate API changes. No new dependencies. The new test uses the existing `tv` test binary and `serde_json`.

## Open Questions

None.
