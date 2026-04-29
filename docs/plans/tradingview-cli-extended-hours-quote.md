# Add extended-hours fields to Desktop-free quote reads

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

Users need to see premarket and postmarket prices when they run `tv quote <SYMBOL>`, especially around market open and close. Today the Desktop-free quote path returns regular scanner quote fields such as `close`, `open`, `high`, `low`, `volume`, and `change`, but it does not expose TradingView's extended-hours columns. After this change, `tv quote NYSE:IONQ` can still be used without a TradingView Desktop target and will include an `extended_hours` object with premarket and postmarket fields when TradingView returns them.

The change must be additive. Existing top-level quote fields keep their current meaning, and current-chart `tv quote` without a symbol remains unchanged.

## Progress

- [x] (2026-04-29 14:05Z) Confirmed the working tree was clean after the `v0.3.0` release commits.
- [x] (2026-04-29 14:07Z) Confirmed `crates/market/src/quote.rs` owns the scanner quote request and normalization.
- [x] (2026-04-29 14:08Z) Confirmed scanner REST accepts `premarket_*` and `postmarket_*` columns and rejects several alternative spellings.
- [x] (2026-04-29 14:15Z) Added extended-hours scanner columns and JSON normalization.
- [x] (2026-04-29 14:18Z) Updated tests, README, changelog, internal API reference, CLI help, and plan index.
- [x] (2026-04-29 14:20Z) Focused quote tests passed.
- [x] (2026-04-29 14:21Z) Read-only smoke confirmed `extended_hours` appears for `NYSE:IONQ` and `NASDAQ:AAPL`.
- [x] (2026-04-29 14:31Z) Full workspace validation and tracked-doc hygiene checks passed.

## Surprises & Discoveries

- Observation: TradingView scanner REST accepts compact `premarket_*` and `postmarket_*` column names, not underscored `pre_market_*` or generic `extended_*` names.
  Evidence: Read-only probes returned values for `premarket_close`, `premarket_change`, `premarket_gap`, `premarket_volume`, `premarket_high`, `premarket_low`, `premarket_open`, `premarket_change_abs`, `postmarket_close`, `postmarket_change`, `postmarket_volume`, `postmarket_high`, `postmarket_low`, `postmarket_open`, and `postmarket_change_abs`. The same endpoint returned HTTP 400 for `pre_market_close`, `pre_market_price`, `post_market_close`, `postmarket_price`, `extended_hours_close`, `market_status`, `session`, and `subsession`.

## Decision Log

- Decision: Add `extended_hours` as a nested object instead of changing top-level `last` or `close`.
  Rationale: Existing callers expect top-level quote fields to retain their current scanner regular-session meaning. Nested fields are additive and safer for downstream users.
  Date/Author: 2026-04-29 / Codex
- Decision: Add extended-hours fields only for scanner-backed `tv quote <SYMBOL>` in this slice.
  Rationale: The current-chart quote path reads chart bars and has no proven reliable extended-hours metadata source. Faking these values from chart state would risk reporting misleading data.
  Date/Author: 2026-04-29 / Codex

## Outcomes & Retrospective

Implementation is complete and validation passed. `tv quote <SYMBOL>` scanner-backed reads now include a nested `extended_hours` object while keeping existing top-level fields unchanged. Read-only smoke showed `extended_hours.premarket` values for `NYSE:IONQ` and `NASDAQ:AAPL`; `postmarket` values were `null`, which is expected outside postmarket hours.

Validation run:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    git diff --check

Tracked-doc hygiene grep found only existing policy text and archived validation examples.

## Context and Orientation

The `tv quote <SYMBOL>` Desktop-free read lives in the `tradingview-market` internal crate. The file `crates/market/src/quote.rs` defines `QUOTE_SCAN_COLUMNS`, posts a scanner request to `https://scanner.tradingview.com/america/scan`, and normalizes the first row into the public JSON payload. The CLI package delegates symbol-targeted quote reads to this crate through `crates/cli/src/ops/market/direct.rs`.

The scanner response row has `s` for the full symbol and `d` for values in the same order as the requested columns. The existing code uses a `field(index)` helper, so adding columns means updating the column list and mapping later indexes into the new nested payload.

## Plan of Work

Edit `crates/market/src/quote.rs`. Extend `QUOTE_SCAN_COLUMNS` after the existing `subtype` column with the confirmed extended-hours columns. Keep the existing first eleven fields in the same order so old index mappings remain correct.

In `normalize_scanner_quote_response`, add an `extended_hours` object to the returned JSON. Map premarket fields to `premarket.open`, `high`, `low`, `last`, `close`, `change_percent`, `change_abs`, `gap_percent`, and `volume`. Map postmarket fields to `postmarket.open`, `high`, `low`, `last`, `close`, `change_percent`, `change_abs`, and `volume`. Use `*_close` for both `last` and `close`. Missing values should remain `null`.

Update unit tests in the same file so the normal success payload proves both old top-level fields and new nested fields are present. Add a case where the row has only the old shorter values and verify the nested extended-hours fields exist as `null`.

Update `docs/internal-tradingview-apis.md` to record the public-safe scanner extended-hours columns. Update README to mention that `tv quote <SYMBOL>` may include `extended_hours`. Update `CHANGELOG.md` under `Unreleased`.

Update `docs/plans/README.md` so this plan is listed as current.

## Concrete Steps

Run commands from the repository root.

First run focused tests:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture

Then run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    git diff --check

Finally run read-only smoke when network access is available:

    target/debug/tv quote NYSE:IONQ
    target/debug/tv quote NASDAQ:AAPL

The smoke output should include `source: "scanner_scan_rest"` and a nested `extended_hours` object. Premarket fields may be null outside premarket hours, and postmarket fields may be null outside postmarket hours.

## Validation and Acceptance

Acceptance is met when the focused quote tests pass, full workspace validation passes, and `tv quote <SYMBOL>` still returns all existing top-level fields while also returning `extended_hours.premarket` and `extended_hours.postmarket`. A valid result must not require TradingView Desktop or CDP.

## Idempotence and Recovery

This is an additive read-only payload change. It is safe to rerun tests and smoke commands. If TradingView removes or renames an extended-hours column, the scanner endpoint may return HTTP 400. In that case, remove the unsupported column and record the observation in this plan before accepting the change.

## Artifacts and Notes

Do not paste raw live scanner responses into tracked docs. Use only field names and high-level observations.

## Interfaces and Dependencies

No new CLI flag is added. The public JSON interface gains a new nested `extended_hours` object for scanner-backed `tv quote <SYMBOL>` results. Existing top-level fields and error behavior remain unchanged.

## Open Questions

None.
