# Watch / JSONL compare contract implementation

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`v0.21.0` made `tv bars --from/--to` a mature Desktop-free historical range
surface. The next useful area is not another automatic data source. It is a
bounded observation workflow that lets users and agents compare a known
candidate set over a short window while preserving source metadata and event
contracts.

This plan implements the first `v0.22.0` candidate: bounded
`tv watch compare <SYMBOL>...`. It is a Desktop-free scanner-backed JSONL
workflow. It does not add a daemon, realtime feed, automatic source mixing,
ranking, recommendation, or stable export command.

## Progress

- [x] Create this ExecPlan.
- [x] Archive the completed `v0.21.0` release readiness plan.
- [x] Add the `v0.22.0` roadmap.
- [x] Update the plan index, `v0.21.0` roadmap, and changelog for the new
  roadmap state.
- [x] Audit existing `snapshot`, `compare`, `observe`, `stream`, and
  `follow_up_hints` contracts.
- [x] Decide the first bounded watch / JSONL compare event shape.
- [x] Decide that the first implementation reads scanner-backed quote evidence
  only.
- [x] Add `tv watch compare <SYMBOL>...` with bounded JSONL readiness, sample,
  heartbeat, and summary events.
- [x] Update docs and runtime skills for the new watch compare surface.
- [x] Run focused tests, baseline validation, docs validation, and runtime
  skill validation.

## Work Items

1. Inventory current evidence surfaces.
   - `tv snapshot` and `tv compare` are Desktop-free market evidence reads.
   - `tv bars` is Desktop-free historical OHLCV evidence.
   - `tv observe chart` and `tv stream ...` are Desktop-backed selected-chart
     JSONL observations.
   - `tv quote --source chart` is a selected-chart quote read.
   - `tv screenshot` is visual evidence, not structured market data.
2. Define and implement the watch / JSONL compare contract.
   - Keep it bounded by duration and / or max events.
   - Include sample, heartbeat, and final summary events.
   - Include command-local contract marker, source metadata, per-symbol
     availability, and public-safe counters.
   - Do not expose raw live payloads, raw JSONL from dependencies, raw target
     ids, account-local metadata, credentials, or local paths.
3. Preserve source boundaries.
   - Do not automatically combine scanner, bars, chart quote, observe, stream,
     and quote-data into a single implied source.
   - If a later implementation supports multiple evidence kinds in one log,
     each event must identify its source and contract marker.
4. Keep selected-chart export and Replay extraction as later `v0.22.0`
   feasibility lanes.
   - They may be planned after this contract slice.
   - They are not hidden fallbacks for `tv bars`.

## Decision Log

- Decision: Make watch / JSONL compare the first `v0.22.0` implementation
  candidate.
  Rationale: `observe_chart.v1` and `stream.v1` already established bounded
  JSONL lessons: readiness, sample, heartbeat, summary, source metadata, and
  public-safe counters.
- Decision: Implement `tv watch compare` as scanner-backed quote polling only.
  Rationale: this gives users a useful short-window candidate observation
  workflow without mixing selected-chart state, browserless bars, quote-data,
  or screenshots into one implied source.
- Decision: Do not make this a daemon or realtime feed.
  Rationale: v0.22 should improve explicit observation workflows without
  introducing long-running service semantics.
- Decision: Keep selected-chart historical export and Replay-based extraction
  in the roadmap, but behind feasibility / contract slices.
  Rationale: both depend heavily on Desktop and UI state and should not be
  presented as browserless historical bars sources.

## Outcomes

This plan adds `tv watch compare <SYMBOL>...` as the first bounded watch
workflow. It emits `watch_compare.v1` JSONL envelopes with readiness, sample,
heartbeat, and summary events. The command uses Desktop-free scanner-backed
quote evidence only and keeps selected-chart export and Replay extraction as
later `v0.22.0` lanes.

Focused validation, workspace baseline, runtime skill validation, and a
public-safe live smoke passed. The live smoke used two NASDAQ symbols and
confirmed one sample event, three heartbeat events, five polls, zero poll
errors, and `duration_elapsed` as the final summary reason without recording
raw JSONL output in tracked docs.

## Validation

Docs validation:

- [x] `git diff --check`
- [x] `bash -n scripts/stage-release-package-files.sh`
- [x] `rg -n "v0\\.22|watch|JSONL compare|follow_up_hints|snapshot|compare|observe chart|stream|selected-chart export|historical export|Replay|bars\\.v1|source mixing|quote-data auto|tv events|MCP|daemon|ranking|recommendation|watch_compare\\.v1" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`

Hygiene:

- [x] `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`

Implementation validation:

- [x] `cargo test -p tradingview-market quote -- --nocapture`
- [x] `cargo test -p tradingview-cli watch -- --nocapture`
- [x] `cargo test -p tradingview-cli --test cli_contract -- --nocapture`
- [x] `cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture`
- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo metadata --no-deps --format-version 1`
- [x] `git diff --check`
- [x] `bash -n scripts/stage-release-package-files.sh`

Runtime skill validation:

- [x] `uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation`
- [x] `uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan`
- [x] `uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/chart-analysis`
