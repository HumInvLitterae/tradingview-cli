# `tv bars` bare symbol resolution

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained and describes the implementation, validation, and documentation work needed to let `tv bars AAPL ...` resolve to an exchange-qualified TradingView symbol while preserving the Desktop-free `bars.v1` source boundary.

## Purpose / Big Picture

Users and agents often type `tv bars AAPL ...` even though the historical bars command has historically expected `EXCHANGE:SYMBOL`, such as `NASDAQ:AAPL`. After this change, `tv bars` accepts a bare symbol when TradingView's Desktop-free symbol search can resolve it to an exact candidate. The command still reports what the user typed and what was actually used, so a caller can notice when the chosen exchange is not the intended one and retry with an explicit `EXCHANGE:SYMBOL`.

The observable outcome is that `tv bars AAPL --timeframe 1D --count 5` resolves `AAPL` through `symbol_search_rest`, reads bars from `NASDAQ:AAPL`, and returns a `bars.v1` payload with `requested_symbol: "AAPL"`, `resolved_symbol: "NASDAQ:AAPL"`, and `symbol: "NASDAQ:AAPL"`.

## Progress

- [x] (2026-06-03 14:10Z) Created this ExecPlan and archived the completed launch process handling plan.
- [x] (2026-06-03 14:25Z) Added internal symbol resolution for `tv bars` using Desktop-free TradingView symbol search for bare symbols.
- [x] (2026-06-03 14:35Z) Added additive `resolved_symbol` and `symbol_resolution` readback to `bars.v1` success payloads and structured failure details.
- [x] (2026-06-03 14:45Z) Updated market and CLI contract tests for the new bare-symbol behavior.
- [x] (2026-06-03 15:05Z) Updated README, source taxonomy, getting-started docs, packaged agent guidance, and runtime skills.
- [x] (2026-06-03 15:25Z) Ran focused tests, baseline validation, runtime skill validation, and public-safe live smoke.
- [ ] Commit the related implementation and documentation changes.

## Surprises & Discoveries

- Observation: `tv search AAPL` returns multiple exact `AAPL` candidates, but the first normalized result is `NASDAQ:AAPL`.
  Evidence: A local Desktop-free search smoke confirmed the first exact candidate, while also showing additional exchange candidates. The implementation therefore records `candidate_count` and the selected `resolved_symbol` instead of pretending the bare symbol was unambiguous.

- Observation: The live `tv bars AAPL` smoke resolved to `NASDAQ:AAPL` in both recent-count and date-range modes.
  Evidence: Public-safe summaries showed `requested_symbol: "AAPL"`, `resolved_symbol: "NASDAQ:AAPL"`, `symbol: "NASDAQ:AAPL"`, `candidate_count: 15`, and `range_coverage_status: "complete"`; raw bars were not recorded.

## Decision Log

- Decision: Bare symbols are resolved only by the Desktop-free symbol search API.
  Rationale: `tv bars` is a Desktop-free historical bars command. Using selected-chart state, scanner quotes, quote-data, Replay, or chart export as fallback would blur the source boundary and make the returned bars harder to reproduce.
  Date/Author: 2026-06-03 / Codex

- Decision: The first exact `symbol` candidate from TradingView search is used for a bare input.
  Rationale: This matches the observed search ordering for common inputs such as `AAPL` while keeping the rule simple and visible. The payload reports both the input and the resolution so callers can retry with `EXCHANGE:SYMBOL` if the chosen exchange is not intended.
  Date/Author: 2026-06-03 / Codex

- Decision: Exchange-qualified input is never replaced by search.
  Rationale: If the user says `NASDAQ:AAPL`, that is already the explicit symbol identity. Automatically substituting another exchange would be more surprising than returning source diagnostics if the requested symbol has no bars.
  Date/Author: 2026-06-03 / Codex

## Outcomes & Retrospective

Implemented. `tv bars` now resolves bare symbols through Desktop-free symbol search when an exact candidate is available, reports requested and resolved symbols in `bars.v1`, and keeps exchange-qualified input as the explicit override. The change adds no new command, no new dependency, no version bump, and no selected-chart or quote-source fallback.

## Context and Orientation

The `tv bars` command is implemented through the CLI adapter in `crates/cli/src/ops/market/bars.rs`, which calls the Desktop-free market crate functions in `crates/market/src/bars.rs`. The market crate validates requests in `crates/market/src/bars/validation.rs`, fetches historical bars through a TradingView WebSocket chart-session path in `crates/market/src/bars/transport.rs`, and shapes JSON payloads in `crates/market/src/bars/payload.rs`.

The term "bare symbol" means a symbol without an exchange prefix, for example `AAPL`. The term "exchange-qualified symbol" means the TradingView-style `EXCHANGE:SYMBOL` form, for example `NASDAQ:AAPL`. The term "Desktop-free" means the command does not connect to TradingView Desktop or Chrome DevTools Protocol. For symbol lookup, this plan uses the existing TradingView search code in `crates/market/src/search.rs`, whose public source marker is recorded here as `symbol_search_rest`.

## Plan of Work

First, add a small `BarsSymbolResolution` readback object to `crates/market/src/bars/types.rs`. `BarsRequest` should keep `symbol` as the actual exchange-qualified symbol passed to the bars source, add `requested_symbol` for the user input, and add `symbol_resolution` for machine-readable resolution metadata.

Second, change `crates/market/src/bars.rs` so `bars_symbol(...)` and `bars_symbol_range(...)` resolve bare symbols before validation. Inputs that already contain `:` should be treated as exchange-qualified and should not trigger a search request. Bare inputs should call `search_symbols_typed(...)`, find the first result whose `symbol` exactly matches the input, and use that result's `full_name`. If no exact candidate exists, return a public-safe validation error after search with `expected_format: "EXCHANGE:SYMBOL"`, `candidate_count`, candidate summaries, and a `next_action_hint`.

Third, update `crates/market/src/bars/payload.rs` so success payloads and structured failure details include `requested_symbol`, `resolved_symbol`, `symbol`, and `symbol_resolution`. No existing `bars.v1` fields should be deleted or renamed.

Fourth, update CLI help, public docs, packaged agent guidance, and runtime skills so humans and agents understand that bare symbols can be resolved, but `EXCHANGE:SYMBOL` remains the right way to force a specific exchange. Agents should report both `requested_symbol` and `resolved_symbol`.

## Concrete Steps

Work from the repository root.

1. Edit `crates/market/src/bars/types.rs`, `crates/market/src/bars.rs`, `crates/market/src/bars/validation.rs`, and `crates/market/src/bars/payload.rs` as described above.
2. Edit `crates/cli/src/cli.rs` so `tv bars --help` describes bare symbol resolution and explicit exchange-qualified override.
3. Update docs and runtime skills:
   - `README.md`
   - `docs/command-source-taxonomy.md`
   - `docs/getting-started.md`
   - `docs/ja/getting-started.md`
   - `docs/observation-workflows.md`
   - `packaging/agent/AGENTS.md`
   - `.agents/skills/market-data-interpretation/SKILL.md`
   - `.agents/skills/chart-analysis/SKILL.md`
   - `.agents/skills/multi-symbol-scan/SKILL.md`
4. Run the focused tests and validation commands listed below.

## Validation and Acceptance

Run these focused tests and expect them to pass:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture

Result: passed on 2026-06-03. The market bars focused run passed 21 tests, and the CLI bars contract run passed 4 tests.

Run the baseline checks and expect no failures:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Result: passed on 2026-06-03. `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo test --workspace` also passed.

Validate changed runtime skills:

    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan

Result: all three changed runtime skills validated successfully.

Optionally run live Desktop-free smoke commands. Do not paste raw bars into tracked docs; record only a public-safe summary if needed.

    target/debug/tv bars AAPL --timeframe 1D --count 5
    target/debug/tv bars AAPL --timeframe 1D --from 2020-01-01 --to 2020-01-31

Acceptance means the bare-symbol smoke returns `contract_version: "bars.v1"` with `requested_symbol: "AAPL"`, `resolved_symbol: "NASDAQ:AAPL"`, `symbol: "NASDAQ:AAPL"`, and `symbol_resolution.resolution_source: "symbol_search_rest"`. Exchange-qualified input such as `tv bars NASDAQ:AAPL ...` should continue to work with `symbol_resolution.resolution_source: "input_exchange_qualified"`.

Result: passed on 2026-06-03. The recent-count smoke returned 5 bars with complete coverage. The date-range smoke for 2020-01-01 through 2020-01-31 returned 21 bars with complete coverage and `range_truncation_reason: "none"`.

## Idempotence and Recovery

The implementation is additive. Re-running tests is safe. If live symbol search or bars smoke fails because of network availability or TradingView source behavior, rely on the fixture and contract tests and record the live failure as source availability evidence rather than changing source boundaries. Do not add selected-chart, scanner quote, quote-data, Replay, or chart export fallbacks to make a failing bars read succeed.

## Artifacts and Notes

The public-safe search observation for `AAPL` is that the first exact candidate resolves to `NASDAQ:AAPL` and multiple other exchange candidates exist. That is why the payload records both requested and resolved symbols.

## Interfaces and Dependencies

No new crate dependency is added. The implementation uses `crate::search::search_symbols_typed` from `crates/market/src/search.rs`, which returns a typed `SymbolSearchResponse`. The existing Rust APIs remain:

    pub async fn bars_symbol(symbol: &str, timeframe: &str, count: usize) -> Result<Value, AppError>
    pub async fn bars_symbol_range(symbol: &str, timeframe: &str, from: &str, to: &str, count_cap: usize) -> Result<Value, AppError>

The new internal readback type is `BarsSymbolResolution` in `crates/market/src/bars/types.rs`. It is not a public API type.

## Open Questions

No blocker remains for this slice. Future work may choose a stricter ambiguity policy, but this first slice intentionally follows TradingView search ordering and exposes the selected exchange in payload readback.
