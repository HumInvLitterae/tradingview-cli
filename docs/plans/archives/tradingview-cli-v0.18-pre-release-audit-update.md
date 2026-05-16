# `v0.18.0` pre-release audit update

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the updated completion / refactor audit before `v0.18.0` release readiness,
after the JSONL summary-event slice.

## Purpose / Big Picture

`v0.18.0` matures existing Desktop-backed JSONL observation contracts. The
release now includes command-local contract metadata and final bounded-window
summary events for `tv observe chart` and lower-level `tv stream ...`.

This audit confirms that readiness, sample, heartbeat, and summary events are
documented and tested consistently before release readiness. It does not add
new commands, options, data sources, realtime multi-symbol feeds, watch loops,
source mixing, ranking, or recommendations.

## Progress

- [x] (2026-05-17T03:10Z) Create this updated pre-release audit ExecPlan.
- [x] (2026-05-17T03:10Z) Archive the completed JSONL summary-event plan.
- [x] (2026-05-17T03:15Z) Update `docs/plans/README.md` and
  `docs/v0.18-roadmap.md` so the
  current plan is this audit.
- [x] (2026-05-17T03:35Z) Confirm `observe_chart.v1` and `stream.v1` docs, tests, help, and
  runtime skills cover readiness, sample, heartbeat, and summary events.
- [x] (2026-05-17T03:55Z) Run focused tests, full Rust baseline, docs checks, skill validation,
  and hygiene scans.
- [x] (2026-05-17T03:55Z) Record whether release readiness is the next step.

## Surprises & Discoveries

- Observation: The first `v0.18.0` pre-release audit happened before the final
  summary-event slice.
  Evidence: `docs/plans/tradingview-cli-v0.18-pre-release-audit.md` is marked
  as historical and points to
  `docs/plans/tradingview-cli-jsonl-observation-summary-event.md` as the
  additional polish completed before release readiness.

- Observation: No release blocker was found after including final JSONL
  summary events.
  Evidence: focused observe / stream tests passed; `cargo clippy
  --workspace --all-targets --all-features -- -D warnings` and
  `cargo test --workspace` passed.

- Observation: The hygiene scans still report existing policy language,
  archived validation commands, fake example paths, and expected
  assertion-style `panic!` calls in ignored live smokes.
  Evidence: the scans did not show a newly introduced raw JSONL live output,
  raw DOM, raw WebSocket frame, raw target id, credential, account-local
  metadata, or local validation path from this audit update.

## Decision Log

- Decision: Refresh the pre-release audit instead of going directly to
  release readiness.
  Rationale: The JSONL summary event changed the completed v0.18 surface after
  the previous audit, so release readiness should be based on the final event
  contract rather than the earlier metadata-only state.
  Date/Author: 2026-05-17 / Codex.

- Decision: Treat larger remaining candidates as post-v0.18 work unless this
  audit finds a small release blocker.
  Rationale: watch / JSONL compare, multi-symbol realtime feeds, standalone
  events, and daemon-style surfaces are separate themes. Mixing them into this
  JSONL contract polish release would blur the source boundary.
  Date/Author: 2026-05-17 / Codex.

- Decision: Recommend `v0.18.0 release readiness` as the next step.
  Rationale: The final event contract now covers readiness, sample, heartbeat,
  and summary events for both observe and stream surfaces, with docs, runtime
  skills, help, and tests aligned. Remaining deferred surfaces are larger
  product lanes rather than release blockers.
  Date/Author: 2026-05-17 / Codex.

## Outcomes & Retrospective

This audit confirms `v0.18.0` is ready to proceed to release readiness.

`tv observe chart` readiness, sample, heartbeat, and summary events are
documented and tested with `contract_version: "observe_chart.v1"` and
`_observe: "chart"`. Observe sample / heartbeat / summary events preserve the
underlying selected-chart stream metadata such as `_stream: "bars"` and
`source: "desktop_chart_stream"`.

Lower-level `tv stream ...` sample, heartbeat, and summary events are
documented and tested with `contract_version: "stream.v1"` while preserving
existing `_event`, `_stream`, sample payload, heartbeat, and bounded-control
semantics.

The docs and runtime skills continue to describe observe / stream as
Desktop-backed selected-chart JSONL observation, not Desktop-free `tv bars`,
scanner quote, chart quote, quote-data, watch loop, multi-symbol realtime
feed, ranking, recommendation, or trading action.

## Context and Orientation

`tv observe chart` is a selected-chart, Desktop-backed, non-mutating JSONL
workflow. It emits a readiness event first, then selected-chart bar sample or
heartbeat events, and now ends normal bounded runs with a summary event. Its
command-local event contract is marked by
`contract_version: "observe_chart.v1"` and `_observe: "chart"`.

Lower-level `tv stream ...` commands are selected-chart Desktop-backed JSONL
sample surfaces for specific stream kinds. They now emit sample, heartbeat,
and summary events marked by `contract_version: "stream.v1"` while preserving
`_event`, `_stream`, source metadata, sample payloads, heartbeat counters, and
bounded controls.

Neither surface is Desktop-free browserless historical bars, scanner quote,
chart-source quote, quote-data, a watch loop, or a multi-symbol realtime feed.

## Plan of Work

Archive the completed summary-event implementation plan and update durable
planning docs so this audit is the current plan. Then inspect docs, runtime
skills, help, and tests for consistency around `observe_chart.v1`,
`stream.v1`, readiness, sample, heartbeat, summary, `end_reason`, and
`desktop_chart_stream` source metadata.

Run focused observe / stream checks first, then the full Rust baseline,
repository hygiene scans, and runtime skill validators for touched skills. If
all checks pass, record in this ExecPlan and the roadmap that the next step is
`v0.18.0 release readiness`.

## Concrete Steps

From the repository root, run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "observe_chart\\.v1|stream\\.v1|summary event|end_reason|JSONL|readiness|heartbeat|desktop_chart_stream|observe chart|stream|realtime|watch|JSONL compare|chart-backed compare|auto fallback|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Then run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Run focused checks:

    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_observe_chart

If runtime skills changed, run:

    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/chart-analysis
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/market-data-interpretation

Validation completed for this audit update:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg hygiene scans for private data, TODO / panic markers, and deferred-surface wording
    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_observe_chart
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/chart-analysis
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/market-data-interpretation

Optional live smoke can be run if explicitly useful:

    TV_LIVE_OBSERVE_CHART_SMOKE=1 cargo test -p tradingview-cli --test live_observe_chart -- --ignored --nocapture

Do not paste raw JSONL live output into tracked docs. If smoke is recorded,
keep only a public-safe summary.

## Validation and Acceptance

Acceptance is met when the focused tests and full baseline pass, the docs and
skills consistently describe `observe_chart.v1` and `stream.v1`, and no
release blocker is found.

The audit must confirm:

- `tv observe chart` readiness, sample, heartbeat, and summary events expose
  `contract_version: "observe_chart.v1"` and `_observe: "chart"`.
- observe sample, heartbeat, and summary events keep underlying selected-chart
  stream metadata such as `_stream: "bars"` and
  `source: "desktop_chart_stream"`.
- `tv stream ...` sample, heartbeat, and summary events expose
  `contract_version: "stream.v1"` while preserving existing `_event`,
  `_stream`, sample payload, heartbeat, and bounded-control semantics.
- summary events are documented as bounded observation-window readbacks, not
  market-data samples.
- observe / stream are not described as Desktop-free `tv bars`, scanner
  quote, chart quote, quote-data, watch loop, multi-symbol realtime feed,
  ranking, recommendation, or trading action.
- no raw JSONL live output, raw DOM, raw WebSocket frame, raw target id,
  credential, account-local metadata, or local validation path is added to
  public docs or packaged assets.

## Idempotence and Recovery

This audit is docs-first and safe to repeat. If a validation command fails,
record the failure in `Surprises & Discoveries`, fix only release blockers,
and rerun the failed command plus the relevant focused tests. If an unrelated
working-tree change appears, do not revert it; inspect whether it affects this
audit before deciding how to proceed.

## Interfaces and Dependencies

This audit does not change public interfaces. No new command, option,
dependency, source, version bump, realtime feed, automatic fallback, source
mixing, ranking, scoring, recommendation, or trading action is planned.

## Open Questions

There are no unresolved critical questions. If validation finds a blocker, the
blocker should be fixed in this slice only when the fix is small and directly
related to release readiness.
