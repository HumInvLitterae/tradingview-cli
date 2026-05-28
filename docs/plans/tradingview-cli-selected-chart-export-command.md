# Selected-chart historical export command

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` updated while
working. This document follows `.agents/PLANS.md`.

## Purpose

After `v0.22.0`, `tv range` and `tv ohlcv` can report selected-chart context,
visible range, returned bars range, and a conservative range-match diagnostic.
Users still do not have an explicit command that says, "move the selected
TradingView Desktop chart to this range, read the selected-chart bars, and
report whether the returned bars match that chart range."

This plan turns that feasibility readback into a narrow selected-chart
historical export workflow. The result should be useful when the user
intentionally wants evidence from the currently selected TradingView Desktop
chart. It must not replace Desktop-free `tv bars --from/--to`.

## Progress

- [x] (2026-05-28) Create this ExecPlan.
- [x] (2026-05-28) Archive the completed `v0.22.0` release readiness plan.
- [x] (2026-05-28) Add the `v0.23.0` roadmap.
- [x] (2026-05-28) Update the plan index, `v0.22.0` roadmap, and changelog for
  the new roadmap state.
- [x] (2026-05-28) Decide the public command surface and help wording:
  `tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>`.
- [x] (2026-05-28) Implement the narrow selected-chart export workflow.
- [x] (2026-05-28) Update docs and runtime skills.
- [x] (2026-05-28) Run focused tests, baseline validation, docs validation,
  and runtime skill validation.

## Surprises & Discoveries

- Existing `tv ohlcv` readback already exposed the selected-chart context,
  returned bars range, and conservative range-match diagnostic needed by the
  export command, so the implementation could stay as a thin orchestration
  layer rather than duplicating chart-bar JavaScript probes.

## Decision Log

- Decision: Make selected-chart historical export the first `v0.23.0`
  implementation candidate.
  Rationale: `v0.22.0` added the required evidence readback to `tv range` and
  `tv ohlcv`; this is the most direct next step from feasibility to explicit
  workflow.
  Date/Author: 2026-05-28 / Codex.

- Decision: Keep selected-chart export separate from Desktop-free `tv bars`.
  Rationale: `tv bars --from/--to` is a reproducible Desktop-free historical
  bars read, while selected-chart export depends on TradingView Desktop state,
  the selected chart, visible range, and target readiness.
  Date/Author: 2026-05-28 / Codex.

- Decision: Do not add automatic source mixing, ranking, recommendation, or a
  hidden fallback.
  Rationale: The user-facing value is explicit evidence provenance. A hidden
  fallback would make downstream use less reproducible.
  Date/Author: 2026-05-28 / Codex.

- Decision: Use `tv export chart-bars` as the first stable command surface.
  Rationale: a top-level `export` family makes the state-changing selected
  chart workflow explicit without overloading `tv ohlcv`, and `chart-bars`
  says the source is the selected chart rather than Desktop-free `tv bars`.
  Date/Author: 2026-05-28 / Codex.

- Decision: The first slice prints JSON to stdout and does not add `--output`.
  Rationale: all current `tv` commands return JSON envelopes through stdout;
  file writing can be added later after the contract is proven.
  Date/Author: 2026-05-28 / Codex.

- Decision: Reuse `tv range` and selected-chart `tv ohlcv` helpers inside the
  CLI layer.
  Rationale: the new command is an explicit workflow over existing
  Desktop-backed operations; reusing the helpers keeps source diagnostics and
  chart context consistent with the underlying commands.
  Date/Author: 2026-05-28 / Codex.

## Outcomes & Retrospective

Implemented `tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>`
with optional `--count` and `--summary`. The payload uses
`contract_version: "export_chart_bars.v1"` and reports the requested visible
range, range operation, selected-chart context, returned bars range, and
range-match diagnostic. The command remains a Desktop-backed selected-chart
operation and is not a fallback for Desktop-free `tv bars --from/--to`.

## Context and Orientation

The repository builds a single Rust CLI binary named `tv`. Desktop-free market
reads live in crates such as `tradingview-market` and do not require a running
TradingView Desktop session. Desktop-backed commands use Chrome DevTools
Protocol to inspect or operate the user's local TradingView Desktop chart.

The existing command `tv bars --from/--to` is the formal Desktop-free
historical bars source. It reads from the browserless `bars.v1` source and is
the right tool for reproducible historical sample preparation.

The existing command `tv range` reads or changes the visible range of the
selected TradingView Desktop chart. The existing command `tv ohlcv` reads bars
from the selected Desktop chart. In `v0.22.0`, these commands gained public-safe
readback that can show selected-chart context, visible range, returned bars
range, and whether the returned bars overlap the observed visible range.

The implementation should live in the CLI layer because it orchestrates
Desktop-backed commands and target state. It may reuse existing chart and
OHLCV helpers rather than duplicating their JavaScript probes.

## Plan of Work

First, add a narrow command surface:

    tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS> [--count <N>] [--summary]

`--from` and `--to` are required together and use the same Unix-second visible
range values as `tv range`. `--count` defaults to 500 and is capped at 500,
matching selected-chart `tv ohlcv` behavior. `--summary` returns a compact
summary payload that still includes the selected-chart export context. Without
`--summary`, the command returns raw selected-chart bars in the normal success
envelope. The command prints JSON to stdout only; it does not write files in
the first implementation slice.

Second, implement the workflow as explicit orchestration:

1. validate requested range and count before connecting;
2. connect to the selected TradingView Desktop target;
3. observe initial chart context;
4. call the existing visible-range operation;
5. read selected-chart OHLCV bars;
6. return a success payload that separates requested range, observed visible
   range, returned bars range, and range-match diagnostic.

Third, shape success and failure output as source diagnostics. The payload
must include source metadata showing that this is a Desktop-backed selected
chart read. It must report target ambiguity, readiness failure, range mismatch,
and bars unavailable as source or chart-state diagnostics, not as trading
judgments.

Fourth, update public docs and runtime skills so users know when to use this
workflow and when to use `tv bars --from/--to` instead. Docs must explicitly
say that selected-chart export is state dependent and is not a hidden fallback
for Desktop-free bars.

## Concrete Steps

Run all commands from the repository root.

1. Inspect current CLI command organization:

       rg -n "Ohlcv|Range|Export|Replay|Watch" crates/cli/src/cli.rs crates/cli/src/app crates/cli/src/ops

2. Add the command surface and validation in the CLI package. Prefer reusing
   existing `tv range` and `tv ohlcv` helpers where possible.

3. Add focused tests for help, validation, success payload shaping, and failure
   details.

4. Update docs:

       README.md
       docs/command-source-taxonomy.md
       docs/observation-workflows.md
       docs/development.md
       packaging/agent/AGENTS.md
       .agents/skills/chart-analysis/SKILL.md
       .agents/skills/market-data-interpretation/SKILL.md

5. Validate:

       cargo test -p tradingview-cli ops::market::ohlcv -- --nocapture
       cargo test -p tradingview-cli ops::chart -- --nocapture
       cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo metadata --no-deps --format-version 1
       git diff --check
       bash -n scripts/stage-release-package-files.sh

## Validation and Acceptance

The implementation is accepted when a user can run a selected-chart export
workflow against a running TradingView Desktop chart:

    tv export chart-bars --from 1704067200 --to 1706745600 --count 500

and see:

- the requested visible range;
- the selected chart symbol and timeframe;
- the observed visible range after the range operation;
- the returned bars range and bar count;
- a conservative range-match status;
- source metadata that says this is Desktop-backed and selected-chart
  dependent;
- `contract_version: "export_chart_bars.v1"`;
- `source_category: "desktop_backed_operation"`, `requires_desktop: true`, and
  `non_mutating: false`, because the command moves the selected chart's
  visible range before reading bars.

The command may reuse the same internal helpers as `tv range` and `tv ohlcv`,
but it must not call `tv bars`, scanner quote, chart quote, quote-data,
Replay, screenshot, `observe`, or `stream` as hidden fallbacks.

Validation errors must happen before network or Desktop access when inputs are
invalid. Failure details must avoid raw target ids, raw DOM, raw payloads,
account-local metadata, credentials, and machine-local paths.

## Idempotence and Recovery

The workflow changes only the selected chart's visible range. Running it again
with the same inputs should produce another bounded selected-chart readback.
If the range operation succeeds but the bar read fails, the error must explain
that chart state may have changed and should include a safe next action such
as rerunning `tv state` or `tv ohlcv --count 1`.

If implementation uncovers that a stable export command cannot prove range
coverage reliably, stop before widening the command and record the blocker in
this plan. Do not add an automatic fallback to `tv bars`.

## Artifacts and Notes

Do not paste raw command output from a live TradingView session into tracked
docs. If optional smoke is run, record only public-safe summary fields such as
symbol, timeframe, requested range, observed visible range, returned bar count,
range-match status, and source marker.

## Interfaces and Dependencies

No new dependency is planned. The implementation should use existing CLI,
CDP, chart, and OHLCV modules. Add a top-level `Export` command family with a
`ChartBars` subcommand in `crates/cli/src/cli.rs` and dispatch it through the
existing CLI runner. The public JSON contract is command-local:

    contract_version: "export_chart_bars.v1"
    source: "selected_chart_cdp"
    source_category: "desktop_backed_operation"
    requires_desktop: true
    non_mutating: false

The success payload must include `requested_visible_range`, `range_operation`,
`chart_context`, `returned_bars_range`, `selected_chart_range_match`, and
either `bars[]` or summary fields depending on `--summary`. Existing
`tv range`, `tv ohlcv`, and `tv bars` payload fields must not be removed or
renamed.

## Open Questions

None for the first implementation slice. Later work may add file output, more
formats, or a broader export family, but those are intentionally outside this
plan.
