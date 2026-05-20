# Add date-range historical readback to `tv bars`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. Keep it self-contained enough that a new contributor can implement the feature without reading archived plans.

## Purpose / Big Picture

After this change, a user should be able to prepare reproducible historical OHLCV input for a specific old date range with a command like `tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31`.

Today, `tv range --from ... --to ...` can move the visible TradingView Desktop chart viewport, but `tv ohlcv --count ...` is not guaranteed to return bars from that displayed historical window. `tv bars` is already the Desktop-free historical bars source, but it only supports count-based recent bars. This plan extends `tv bars` with explicit date-range readback so old source-guided examples, such as VCP or cup-with-handle samples, can become stable downstream input without relying on chart viewport state.

## Progress

- [x] (2026-05-21T00:00Z) Create the `v0.19.0` roadmap and this first implementation ExecPlan.
- [ ] Research the current `tradingview-market` browserless bars request path and identify the smallest TradingView chart-session request change needed for date-range reads.
- [ ] Add CLI validation for `tv bars --from <DATE> --to <DATE>` while preserving existing count-based behavior.
- [ ] Add market crate request, transport, payload, and tests for daily date-range historical bars.
- [ ] Update docs and runtime skills so users do not confuse `tv range` / `tv ohlcv` with reproducible historical export.
- [ ] Run focused tests, baseline checks, and optional public-safe live smoke.

## Surprises & Discoveries

- Observation: Downstream reports show that `tv range` can move the visible Desktop chart to an old period, but that does not make `tv ohlcv --count ...` a reliable historical export for that displayed period.
  Evidence: The reported CRUS 2010 workflow moved the visible period but still received recent bars from `tv ohlcv`.

- Observation: Current `tv bars` has the right source boundary for this problem but not the right request shape.
  Evidence: `tv bars` is Desktop-free and symbol-targeted, but its current CLI shape is `tv bars <SYMBOL> --timeframe <TF> --count <N>` with a bounded count and no `--from` / `--to`.

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

## Outcomes & Retrospective

This section will be updated after implementation. The intended outcome is a stable Desktop-free command path for historical date-range bars and clear documentation that `tv range` is a viewport operation, not a reproducible historical data export mechanism.

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

Second, extend CLI validation in `crates/cli/src/cli.rs` and the market crate validation in `crates/market/src/bars/validation.rs`. Add `--from <DATE>` and `--to <DATE>` to `tv bars`. Accept dates in `YYYY-MM-DD` format for the first slice. Reject blank dates, invalid dates, `from` after `to`, and incomplete ranges before network access. Preserve existing count-based behavior when neither date option is present. Decide whether date-range mode allows `--count`; if it does, define it as a safety cap. If it does not, reject the combination with a clear validation error.

Third, add typed request state in `crates/market/src/bars/types.rs`. The request should represent either recent-count mode or date-range mode. Keep the existing public Rust function `tradingview_market::bars_symbol(symbol, timeframe, count)` for compatibility. Add a new crate-facing function only if needed, such as `bars_symbol_range(symbol, timeframe, from, to)`, without committing to a broad stable Rust API beyond what the CLI needs.

Fourth, update transport and protocol code. The implementation must keep using the Desktop-free browserless chart-session path and must not connect to CDP. The returned bars must be sorted ascending, deduplicated by timestamp, and bounded. Daily bars should be the first supported date-range timeframe. If TradingView returns partial coverage, return partial coverage honestly instead of retrying hidden source paths.

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
- What is the best TradingView chart-session request shape for historical daily ranges: direct date range, older anchor plus count, or bounded pagination?
- Should the first implementation support only `1D`, or also weekly and monthly bars if they use the same protocol path?

Revision note 2026-05-21: Initial plan created from the post-`v0.18.0` roadmap discussion and downstream report that `tv range` plus `tv ohlcv` does not provide reproducible old-period bars input.
