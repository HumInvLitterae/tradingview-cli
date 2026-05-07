# Chart Quote Concurrency / Realtime Source Strategy Audit

This ExecPlan tracks the reliability audit for symbol-targeted chart-source
quotes before `v0.9.0` comparison work starts.

## Purpose

`tv quote <SYMBOL> --source chart` has been hardened several times, but rare
reports still indicate that the command can return data that appears to belong
to a previous or different symbol. This slice tests the hypothesis that the
failure is more likely when multiple commands run concurrently, or close enough
together that visible-chart mutation and restore boundaries overlap.

The desired outcome is not another blind readiness condition. The desired
outcome is evidence for one of these decisions:

- chart-source quote still needs a `v0.8.1`-style patch around locking,
  diagnostics, or failure behavior;
- chart-source quote should remain a single-symbol, operator-intent source, and
  multi-symbol realtime-ish reads should use a different design;
- Desktop-free scanner quote and future comparison commands are enough for
  broad comparison, without promising selected-chart realtime feed semantics.

## Progress

- [x] Archived the completed `v0.8.0` release-readiness ExecPlan.
- [x] Inspected the current chart-source quote critical section.
- [x] Added an ignored Rust live smoke for near-concurrent chart-source quote
      processes.
- [x] Recorded the source strategy boundary in docs and runtime skills.
- [x] Ran validation.
- [x] Committed the related changes.
- [x] Ran opt-in live concurrency smoke and recorded the source decision.

## Current Findings

`QuoteSymbolLock` serializes symbol-targeted chart-source quote calls that use
the same machine, user, and temporary directory. Its critical section covers
the original chart read, requested symbol switch, readiness polling, restore,
and success payload construction.

The lock is intentionally scoped to `quote <SYMBOL> --source chart` and the
chart path of `quote <SYMBOL> --source auto`. Other chart mutation commands,
such as `tv symbol`, do not currently share this lock. That means chart-source
quote can still race with external Desktop UI actions or other `tv` commands
that mutate the selected chart.

This is acceptable only if chart-source quote is documented and tested as a
correctness-first single-symbol source, not as a multi-symbol realtime batch
source.

## Decisions

- Do not add a user-facing command or option in this slice.
- Add an opt-in ignored integration test instead of a CI-required live test.
- Keep `tv quote <SYMBOL>` default scanner behavior and `tv quotes` batch
  behavior unchanged.
- Treat `tv compare` as blocked on this reliability boundary: broad comparison
  should not inherit chart-switching realtime feed promises by accident.
- If the new smoke reproduces mismatch or restore failures, plan a focused
  patch before broadening comparison features.

## Live Smoke Design

The new test is `crates/cli/tests/live_chart_quote_concurrency.rs`.

It is ignored by default and requires:

```bash
TV_LIVE_CHART_QUOTE_CONCURRENCY_SMOKE=1 cargo test -p tradingview-cli --test live_chart_quote_concurrency -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_CHART_QUOTE_CONCURRENCY_SYMBOLS`
- `TV_LIVE_CHART_QUOTE_CONCURRENCY_RUNS`
- `TV_LIVE_CHART_QUOTE_CONCURRENCY_TARGET_ID`
- `TV_LIVE_CHART_QUOTE_CONCURRENCY_WIDTH`

The test starts batches of near-concurrent `tv quote <SYMBOL> --source chart`
child processes against the same target. It validates only public-safe contract
fields: requested symbol, observed quote symbol, current chart symbol,
freshness pass state, stable sample count, restore state, and elapsed time.
Failure summaries intentionally avoid raw JSON payloads, target ids, account
metadata, and local machine paths.

## Validation

Required before commit:

```bash
cargo test -p tradingview-cli --test live_chart_quote_concurrency
cargo test -p tradingview-cli --test live_chart_quote
cargo test -p tradingview-cli market::quote -- --nocapture
cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
git diff --check
bash -n scripts/stage-release-package-files.sh
```

Optional live smoke:

```bash
TV_LIVE_CHART_QUOTE_CONCURRENCY_SMOKE=1 cargo test -p tradingview-cli --test live_chart_quote_concurrency -- --ignored --nocapture
```

## Outcomes

Validation passed for the normal non-live path:

- `cargo test -p tradingview-cli --test live_chart_quote_concurrency`
- `cargo test -p tradingview-cli --test live_chart_quote`
- `cargo test -p tradingview-cli market::quote -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract quote -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`

The two modified runtime skills validated successfully. The broad hygiene grep
reported only existing safety language and archived validation examples.

The opt-in live concurrency smoke can be rerun for targeted investigation when
a TradingView Desktop/CDP session is prepared:

```bash
TV_LIVE_CHART_QUOTE_CONCURRENCY_SMOKE=1 cargo test -p tradingview-cli --test live_chart_quote_concurrency -- --ignored --nocapture
```

Follow-up live evidence on 2026-05-07:

- TradingView Desktop/CDP readiness was available with one chart target.
- The first live smoke attempt exposed a test harness issue: child process
  stdout/stderr was inherited instead of piped, so the test could not parse the
  public JSON envelope despite successful child commands. The harness now pipes
  child stdout/stderr before parsing.
- The opt-in live smoke passed with default width 2 over six public symbols.
- The opt-in live smoke also passed with width 3 over the same public symbol
  set.
- No requested/observed/chart symbol mismatch, restore failure, or freshness
  failure reproduced in those runs.

Decision: move `v0.9.0` planning forward as Desktop-free comparison first.
Chart-source quote remains a correctness-first single-symbol selected-chart
feed check, not a multi-symbol realtime batch source. If a mismatch is later
captured, treat it as a focused patch lane rather than broadening compare to
chart-switching reads.
