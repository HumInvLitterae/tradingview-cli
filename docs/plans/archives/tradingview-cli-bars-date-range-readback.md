# Add date-range historical readback to `tv bars`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. Keep it self-contained enough that a new contributor can implement the feature without reading archived plans.

## Purpose / Big Picture

After this change, a user should be able to prepare reproducible historical OHLCV input for a specific old date range with a command like `tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31`.

Today, `tv range --from ... --to ...` can move the visible TradingView Desktop chart viewport, but `tv ohlcv --count ...` is not guaranteed to return bars from that displayed historical window. `tv bars` is already the Desktop-free historical bars source, but it only supports count-based recent bars. This plan extends `tv bars` with explicit date-range readback so old source-guided examples, such as VCP or cup-with-handle samples, can become stable downstream input without relying on chart viewport state.

## Progress

- [x] (2026-05-21T00:00Z) Create the `v0.19.0` roadmap and this first implementation ExecPlan.
- [x] (2026-05-21T01:10Z) Research the current `tradingview-market` browserless bars request path and identify `request_more_data` as the smallest chart-session extension for older daily ranges.
- [x] (2026-05-21T01:35Z) Add CLI validation for `tv bars --from <DATE> --to <DATE>` while preserving existing count-based behavior.
- [x] (2026-05-21T02:05Z) Add market crate request, transport, payload, and tests for daily date-range historical bars.
- [x] (2026-05-21T02:25Z) Update docs and runtime skills so users do not confuse `tv range` / `tv ohlcv` with reproducible historical export.
- [x] (2026-05-21T03:35Z) Run focused tests, baseline checks, and optional public-safe live smoke.

## Surprises & Discoveries

- Observation: Downstream reports show that `tv range` can move the visible Desktop chart to an old period, but that does not make `tv ohlcv --count ...` a reliable historical export for that displayed period.
  Evidence: The reported CRUS 2010 workflow moved the visible period but still received recent bars from `tv ohlcv`.

- Observation: Current `tv bars` has the right source boundary for this problem but not the right request shape.
  Evidence: `tv bars` is Desktop-free and symbol-targeted, but its current CLI shape is `tv bars <SYMBOL> --timeframe <TF> --count <N>` with a bounded count and no `--from` / `--to`.

- Observation: The existing browserless bars path can be extended without a new source by using TradingView chart-session `request_more_data` after `create_series`.
  Evidence: The implementation still uses `chart_create_session`, `resolve_symbol`, `create_series`, `switch_timezone`, and `request_more_data` on `wss://data.tradingview.com/socket.io/websocket?type=chart`; no CDP, `tv range`, Replay, or selected-chart state was added.

- Observation: Daily bar timestamps are not necessarily midnight UTC, so the `--to` date must be treated as an inclusive calendar date with an exclusive upper bound at the following UTC day start.
  Evidence: Public-safe AAPL 2020-Q1 smoke showed that filtering with `bar.time <= to_day_start` can omit the requested `--to` day's market bar.

## Decision Log

- Decision: Implement historical date-range input on `tv bars`, not by coupling `tv range` to `tv ohlcv`.
  Rationale: `tv range` is a Desktop chart viewport operation and `tv ohlcv` is a selected-chart CDP read. Using those as an implicit historical export path would blur source boundaries and reproduce the downstream confusion. `tv bars` already owns Desktop-free historical bars.
  Date/Author: 2026-05-21 / Codex.

- Decision: Start with daily bars for date-range mode.
  Rationale: Old intraday history is more likely to be limited by entitlement and retention rules. Daily bars cover the source-guided textbook examples that motivated this work and are lower risk for the first stable contract.
  Date/Author: 2026-05-21 / Codex.

- Decision: Preserve `bars.v1` and make all new fields additive.
  Rationale: Existing `tv bars` consumers should continue reading count-based payloads. Date-range readback should extend the contract with requested and returned range metadata, not rename existing `summary`, `range`, `source_availability`, or `bars[]` fields.
  Date/Author: 2026-05-21 / Codex.

- Decision: In date-range mode, treat `--count` as a safety cap and default it to `500`.
  Rationale: Count-only mode already defaults to `100` and must remain stable. Date-range mode is asking for a period, not an exact count, so the count should cap returned rows rather than imply expected row count. The maximum remains the existing `500` bar cap.
  Date/Author: 2026-05-21 / Codex.

- Decision: Use `request_more_data` paging rather than adding a direct date-range protocol surface.
  Rationale: The current TradingView chart-session path already has count-based `create_series` behavior and can page older daily bars with `request_more_data`. This preserves the same source and avoids inventing a second bars source.
  Date/Author: 2026-05-21 / Codex.

## Outcomes & Retrospective

Implementation is complete. The CLI and market crate now accept daily date-range bars requests, use the existing Desktop-free chart-session WebSocket path, page older data with `request_more_data`, filter returned bars to the requested range, and add `request_mode`, `requested_range`, `returned_range`, `observed_range`, and `range_coverage_status` to `bars.v1` payloads.

Validation passed for focused market and CLI bars tests, live-bars compile-only smoke, formatting, clippy, workspace tests, metadata generation, diff whitespace checks, and release packaging shell syntax. Optional public-safe live smoke succeeded for `NASDAQ:AAPL` over 2020-01-01 through 2020-03-31 and for `NASDAQ:CRUS` over 2010-01-01 through 2010-12-31. After the inclusive `--to` fix, the AAPL smoke returned 62 daily bars and the CRUS smoke returned 252 daily bars, both with complete requested range coverage. No hidden `tv range`, `tv ohlcv`, Replay, or selected-chart fallback was added.

## Context and Orientation

The repository builds a Rust CLI named `tv`. The command definitions live in `crates/cli/src/cli.rs`. Command dispatch and output envelopes live under `crates/cli/src/app/` and `crates/cli/src/ops/`. Desktop-free market reads live in `crates/market/`; this is where `tv bars` belongs because it does not require TradingView Desktop, Chrome DevTools Protocol, or a selected chart target.

The existing `tv bars` public API is `tv bars <EXCHANGE:SYMBOL> --timeframe <TIMEFRAME> --count <N>`. The implementation facade is `crates/market/src/bars.rs`, with submodules under `crates/market/src/bars/`:

- `validation.rs` validates the exchange-qualified symbol, timeframe, and count.
- `transport.rs` connects to TradingView's browserless chart-session WebSocket and performs the bounded wait.
- `protocol.rs` constructs TradingView chart-session messages, parses packets, merges bars, and converts a bar to JSON.
- `payload.rs` builds the `bars.v1` success payload and structured error details.
- `types.rs` holds internal request, result, bar, wait-summary, and availability types.

`tv range` is a different command. It changes the visible range of the selected Desktop chart. `tv ohlcv` is also different. It reads bars from the selected Desktop chart through CDP. This plan must not make either of those commands a hidden fallback for `tv bars`.

The term "Desktop-free" means the command can run without TradingView Desktop. The term "bounded" means the command waits only for a limited time or count, then returns either data or structured unavailable details. The term "source availability" means whether the source produced usable evidence during that bounded read; it is not a statement about whether a symbol has history or whether a setup is valid.

## Plan of Work

First, research the current browserless TradingView chart-session request flow in `crates/market/src/bars/transport.rs` and `crates/market/src/bars/protocol.rs`. Identify whether the existing request can be adapted with an older anchor point, a visible range, a date range, or a paged request. Keep notes public-safe: record message kinds and high-level field names, not raw WebSocket payloads or session identifiers.

Second, extend CLI validation in `crates/cli/src/cli.rs` and the market crate validation in `crates/market/src/bars/validation.rs`. Add `--from <DATE>` and `--to <DATE>` to `tv bars`. Accept dates in `YYYY-MM-DD` format for the first slice. Reject blank dates, invalid dates, `from` after `to`, and incomplete ranges before network access. Preserve existing count-based behavior when neither date option is present. Date-range mode allows `--count` as a safety cap and defaults it to `500`; count-only mode keeps the existing default of `100`.

Third, add typed request state in `crates/market/src/bars/types.rs`. The request should represent either recent-count mode or date-range mode. Keep the existing public Rust function `tradingview_market::bars_symbol(symbol, timeframe, count)` for compatibility. Add a new crate-facing function only if needed, such as `bars_symbol_range(symbol, timeframe, from, to)`, without committing to a broad stable Rust API beyond what the CLI needs.

Fourth, update transport and protocol code. The implementation must keep using the Desktop-free browserless chart-session path and must not connect to CDP. The returned bars must be sorted ascending, deduplicated by timestamp, and bounded. Daily bars should be the first supported date-range timeframe. Use `request_more_data` to page older bars until the requested start date is reached, the bounded wait ends, or the source stops making progress. If TradingView returns partial coverage, return partial coverage honestly instead of retrying hidden source paths.

Fifth, extend `payload.rs` additively. The success payload should keep existing fields such as `contract_version: "bars.v1"`, `source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`, `summary`, `range`, `source_availability`, and `bars[]`. Add date-range readback such as `request_mode: "date_range"`, `requested_range`, `returned_range`, and `range_coverage_status`. For count mode, either omit date-range-only fields or set `request_mode: "recent_count"` with no breaking changes to existing fields. Structured failures should include the requested range and source availability details without exposing raw frames or session ids.

Sixth, update docs and runtime skills. In `README.md`, `docs/command-source-taxonomy.md`, `docs/observation-workflows.md`, `docs/internal-tradingview-apis.md`, and relevant skills under `.agents/skills/`, explain that `tv bars --from --to` is the reproducible historical bars path. Explicitly state that `tv range` moves the visible Desktop chart range and that `tv ohlcv` reads selected-chart bars; neither is a reliable historical export contract by itself.

## Concrete Steps

Run these commands from the repository root while implementing:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

For optional live smoke, run only public-safe commands and do not paste raw live payloads into tracked docs:

    target/debug/tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31
    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --from 2020-01-01 --to 2020-03-31

Expected successful output should be a normal JSON envelope whose `data.contract_version` is `bars.v1`, whose `data.source` is `tradingview_bars_ws`, whose `data.request_mode` identifies date-range mode, and whose range coverage fields make clear what period was actually returned.

## Validation and Acceptance

Acceptance for the implementation is behavior-focused:

- `tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31` validates before network access and uses the Desktop-free bars source when the dates are valid.
- `tv bars NASDAQ:CRUS --timeframe 1D --from 2010-12-31 --to 2010-01-01` fails validation before network access with a clear error.
- Existing `tv bars NASDAQ:AAPL --timeframe 1D --count 5` behavior and payload fields continue to work.
- Date-range success payloads include requested range, returned range, coverage status, source availability, summary, range, and raw `bars[]`.
- Partial coverage is reported as partial, not as complete and not as a trading conclusion.
- No-bars or blocked source results remain structured failures with public-safe details.
- `tv range` and `tv ohlcv` behavior does not change.
- No raw WebSocket frame, raw payload, session id, credential, account-local metadata, target id, or local absolute path is added to payloads, docs, panic messages, or test fixtures.

## Idempotence and Recovery

All edits should be additive and safe to retry. Validation failures should happen before network access where possible. If live smoke fails because TradingView does not provide the requested historical range, keep the failure as evidence of source availability and do not broaden the implementation to hidden fallback sources.

If a date-range protocol attempt proves infeasible, update this plan with the failed request strategy, public-safe observations, and the next feasible option. Do not silently replace the feature with `tv range` plus `tv ohlcv`, because that would preserve the current downstream confusion.

## Artifacts and Notes

Do not store raw WebSocket frames, raw live payloads, session identifiers, target ids, credentials, account-local metadata, or local absolute paths in this plan or any tracked documentation.

Public-safe evidence may include:

- command name and symbol;
- requested timeframe and date range;
- number of bars returned;
- first and last returned bar timestamps;
- coverage status;
- high-level unavailable reason.

## Interfaces and Dependencies

The existing public CLI command is `tv bars`. This plan adds options to that command rather than adding a new command.

The existing public Rust function `tradingview_market::bars_symbol(symbol, timeframe, count)` should remain available. If the implementation needs a new internal or crate-facing function, prefer a narrow signature such as:

    pub async fn bars_symbol_range(
        symbol: &str,
        timeframe: &str,
        from: &str,
        to: &str,
    ) -> Result<serde_json::Value, tradingview_core::AppError>;

This function should return the same JSON contract family as count-based `bars_symbol`: `contract_version: "bars.v1"` and `source: "tradingview_bars_ws"`.

No new source category is expected. The source remains `desktop_free_read`. No new dependency should be added unless protocol research proves the current date parsing or time conversion code cannot safely handle the required input.

## Open Questions

- Should date-range mode allow `--count` as a maximum safety cap, or should `--count` be rejected when `--from` / `--to` are present?
  Answer: allow `--count` as a safety cap; default to `500` in date-range mode.
- What is the best TradingView chart-session request shape for historical daily ranges: direct date range, older anchor plus count, or bounded pagination?
  Answer: use bounded pagination with `request_more_data` on the existing chart-session source.
- Should the first implementation support only `1D`, or also weekly and monthly bars if they use the same protocol path?
  Answer: support only `1D` in this slice; leave weekly, monthly, and intraday ranges for later.

Revision note 2026-05-21: Initial plan created from the post-`v0.18.0` roadmap discussion and downstream report that `tv range` plus `tv ohlcv` does not provide reproducible old-period bars input.

Revision note 2026-05-21: Updated during implementation to record the `request_more_data` approach, date-range validation decisions, additive `bars.v1` fields, and documentation synchronization.

Revision note 2026-05-21: Completed validation and recorded public-safe live smoke summaries for AAPL 2020-Q1 and CRUS 2010 daily range reads.

Revision note 2026-05-21: Adjusted date filtering so `--to` is an inclusive calendar date instead of a UTC-day-start instant.
