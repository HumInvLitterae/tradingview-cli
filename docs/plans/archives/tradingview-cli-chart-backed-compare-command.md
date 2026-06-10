# Chart-backed compare command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can continue from this file without needing prior conversation history.

## Purpose / Big Picture

The user should be able to compare a small set of symbols using explicit TradingView Desktop chart evidence without confusing that workflow with the existing Desktop-free scanner compare. Today, `tv compare <SYMBOL>...` and `tv watch compare <SYMBOL>...` use scanner-backed data. A chart-backed compare needs different behavior because it depends on the selected TradingView Desktop chart, may need to switch the chart symbol, and must report whether chart state changed.

The implemented command shape is `tv chart compare <SYMBOL>...`. It adds the command without source mixing, hidden fallbacks, or ambiguous output.

## Progress

- [x] (2026-06-11) Create this ExecPlan as the first `v0.25.0` implementation candidate.
- [x] (2026-06-11) Record `v0.25.0` roadmap direction with chart-backed compare as Lane 1.
- [x] (2026-06-11) Inspect the current CLI command tree and choose `Command::Chart { command }` with `ChartCommand::Compare` as the command placement.
- [x] (2026-06-11) Design the first narrow payload contract as `chart_compare.v1`.
- [x] (2026-06-11) Implement the command, tests, docs, and runtime skill updates.

## Surprises & Discoveries

- Observation: The repository already had a completed chart-backed compare feasibility plan, but no stable chart-backed compare command before this slice.
  Evidence: `docs/plans/archives/tradingview-cli-chart-backed-compare-contract.md` records that `tv compare` must remain Desktop-free and that a separated command is preferable if implementation proceeds.

## Decision Log

- Decision: Use `tv chart compare <SYMBOL>...` as the stable command shape for the first chart-backed compare implementation.
  Rationale: It keeps Desktop-backed selected-chart workflows visibly separate from `tv compare <SYMBOL>...`, which is already Desktop-free and scanner-backed. It also avoids making `--source chart` look like a simple source toggle on a command with different operating risks.
  Date/Author: 2026-06-11 / Codex

- Decision: The first chart-backed compare implementation must report source and mutation state, not just quote values.
  Rationale: A chart-backed compare may depend on chart switching, selected target state, symbol/timeframe context, and restore attempts. Downstream tools need to know what was requested, what was actually read, and whether the chart state was changed.
  Date/Author: 2026-06-11 / Codex

- Decision: Do not use `tv bars`, scanner compare, chart quote, Replay, chart export, or quote-data as hidden fallbacks.
  Rationale: The project has repeatedly kept source boundaries explicit. Chart-backed compare should be its own workflow evidence, not an automatic mixture of unrelated sources.
  Date/Author: 2026-06-11 / Codex

## Outcomes & Retrospective

Implementation completed the first narrow command. Users can run `tv chart compare <SYMBOL>...` for 2 to 10 symbols. The command returns a normal JSON success envelope with `command: "chart"` and `data.contract_version: "chart_compare.v1"`, ordered item status, before/after chart context, and restore readback.

## Context and Orientation

The `tv` binary is a Rust CLI. Desktop-free market reads live mostly under `crates/market` and related scanner modules. Desktop-backed chart reads and operations use TradingView Desktop through command adapters under `crates/cli/src/ops/`.

Existing compare commands are scanner-backed:

- `tv compare <SYMBOL>...` is a single-shot Desktop-free compare packet.
- `tv watch compare <SYMBOL>...` is a bounded Desktop-free JSONL polling workflow.

Existing selected-chart commands are Desktop-backed:

- `tv quote <SYMBOL> --source chart` reads chart-backed quote evidence for a single symbol.
- `tv ohlcv` reads bars from the selected chart.
- `tv export chart-bars` is an explicit selected-chart export workflow.
- Replay commands operate on selected TradingView Desktop Replay state.

In this plan, "Desktop-free" means the command does not require TradingView Desktop to be running. "Desktop-backed" means the command depends on TradingView Desktop and a selected chart. "Hidden fallback" means silently using a different source when the requested source is unavailable; this plan forbids that.

## Plan of Work

First, inspect the current CLI command tree under `crates/cli` and confirm where a `chart` command group should live. The implementation places a `ChartCommand` group under the existing top-level command enum, with `tv chart compare <SYMBOL>...` as the first subcommand.

Second, define the first narrow chart-backed compare contract. The payload should include a command-local contract marker, requested symbols, resolved symbols where available, per-symbol read status, selected chart context, source metadata, whether TradingView Desktop is required, whether the operation is mutating, and restore status if chart switching is attempted. Failure details must be public-safe and must not include raw target ids, raw DOM, raw payloads, credentials, account-local metadata, or local paths.

Third, implement a small bounded workflow. The first implementation should not try to become a broad comparison engine. It should compare only the requested symbols through the selected TradingView Desktop workflow, report item-level success or failure, and stop with clear diagnostics when the selected target is ambiguous, the chart cannot be switched, or chart evidence cannot be read.

Fourth, update public docs, runtime skills, and packaged agent guidance so users and agents understand when to use scanner-backed compare versus chart-backed compare. The docs must state that chart-backed compare is not ranking, recommendation, or buy/sell judgment.

## Concrete Steps

Run commands from the repository root.

1. Inspect command layout and existing tests:

       rg -n "Compare|Watch|Chart|Export|quote.*source chart|chart-bars" crates/cli/src crates/market/src tests

2. Add the smallest command surface for `tv chart compare <SYMBOL>...` after the module placement is confirmed.

3. Add or update focused tests for help text, validation, success payload shape, source metadata, target ambiguity, per-symbol failures, and public-safe error details.

4. Update docs and runtime skills after the command behavior is stable.

5. Run validation:

       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo metadata --no-deps --format-version 1
       git diff --check
       bash -n scripts/stage-release-package-files.sh

## Validation and Acceptance

The implementation is acceptable when a user can run `tv chart compare <SYMBOL>...` and receive a payload that clearly says it is Desktop-backed selected-chart evidence, shows which symbols were requested and read, reports selected chart context, reports item-level failures without raw private data, and never silently falls back to scanner compare, `tv bars`, Replay, chart export, or quote-data.

The existing `tv compare <SYMBOL>...` and `tv watch compare <SYMBOL>...` contracts must remain Desktop-free and scanner-backed. Tests should prove those command contracts did not change.

## Idempotence and Recovery

All implementation work should be additive. If chart switching is implemented, the command should record restore status and should not hide restore failure. Retrying the command should be safe as long as the user understands that Desktop-backed chart operations may change selected chart state.

If a validation command fails, fix the smallest relevant issue and rerun the focused test before rerunning the full baseline.

## Artifacts and Notes

Do not paste raw live output, raw chart payloads, raw target ids, account-local metadata, credentials, or local absolute paths into tracked docs. Optional live smoke evidence may be summarized with command name, source marker, symbol count, per-symbol status count, and restore status only.

## Interfaces and Dependencies

This plan adds no dependency and no version bump. The command reuses existing TradingView Desktop target resolution and chart quote operation helpers. It does not introduce a new data source; it makes selected-chart evidence explicit.

## Open Questions

No blocker remains for this slice. Follow-up work can revisit whether broader chart-backed workflows need a separate helper layer, but this implementation is intentionally narrow.
