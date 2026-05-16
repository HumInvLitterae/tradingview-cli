# `v0.18.0` pre-release audit

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the completion / refactor audit before `v0.18.0` release readiness.

## Purpose / Big Picture

`v0.18.0` matures existing Desktop-backed JSONL observation contracts. The
release adds command-local contract metadata to `tv observe chart` and
lower-level `tv stream ...` events so agents and downstream tools can identify
readiness, sample, and heartbeat lines without changing event meaning.

This audit confirms the event contract, source boundaries, docs, runtime
skills, help, and tests are aligned before release readiness. It does not add
new commands, options, data sources, realtime multi-symbol feeds, watch loops,
source mixing, ranking, or recommendations.

## Progress

- [x] (2026-05-17T00:00Z) Create this pre-release audit ExecPlan.
- [x] (2026-05-17T00:00Z) Archive the completed JSONL observation contract
  plan.
- [x] (2026-05-17T00:00Z) Update `docs/plans/README.md` and
  `docs/v0.18-roadmap.md` so the current plan is this audit.
- [x] (2026-05-17T01:05Z) Confirm `observe_chart.v1` and `stream.v1` docs, tests, help, and
  runtime skills remain aligned.
- [x] (2026-05-17T01:05Z) Run focused tests, full Rust baseline, docs checks, and hygiene scans.
- [x] (2026-05-17T01:05Z) Record the release-readiness recommendation.

## Surprises & Discoveries

- Observation: No release blocker was found in the `v0.18.0` pre-release
  audit.
  Evidence: focused observe / stream tests passed; `cargo clippy
  --workspace --all-targets --all-features -- -D warnings` and `cargo test
  --workspace` passed.

- Observation: The hygiene scans still report existing policy language,
  archived validation commands, fake example paths, and known assertion-style
  `panic!` calls in ignored live smokes.
  Evidence: the scans did not show a newly introduced raw JSONL live output,
  raw DOM, raw WebSocket frame, raw target id, credential, account-local
  metadata, or local validation path from this audit.

## Decision Log

- Decision: Treat the JSONL observation contract metadata slice as complete
  and move into release-readiness audit.
  Rationale: The implementation and previous validation already confirmed
  additive `observe_chart.v1` and `stream.v1` metadata without changing
  existing JSONL event meaning.
  Date/Author: 2026-05-17 / Codex.

- Decision: Do not add another feature or refactor in this audit unless a
  release blocker is found.
  Rationale: `v0.18.0` already has a focused contract-maturity theme.
  Additional behavior belongs in a later version.
  Date/Author: 2026-05-17 / Codex.

- Decision: Recommend `v0.18.0 release readiness` as the next step.
  Rationale: The JSONL event contracts, source boundaries, docs, runtime
  skills, help, and tests all validate without changing public behavior.
  Date/Author: 2026-05-17 / Codex.

## Outcomes & Retrospective

This audit confirms `v0.18.0` is ready to proceed to release readiness.

`tv observe chart` readiness, sample, and heartbeat events are documented and
tested with `contract_version: "observe_chart.v1"` and `_observe: "chart"`.
Observe sample / heartbeat events preserve the underlying selected-chart
stream metadata such as `_stream: "bars"` and
`source: "desktop_chart_stream"`.

Lower-level `tv stream ...` sample and heartbeat events are documented and
tested with `contract_version: "stream.v1"` while preserving existing
`_event`, `_stream`, sample payload, heartbeat, and bounded-control semantics.

The docs and runtime skills continue to describe observe / stream as
Desktop-backed selected-chart JSONL observation, not Desktop-free `tv bars`,
scanner quote, chart quote, quote-data, watch loop, multi-symbol realtime
feed, ranking, recommendation, or trading action.

## Context and Orientation

`tv observe chart` is a selected-chart, Desktop-backed, non-mutating JSONL
workflow. It emits a readiness event first, then selected-chart bar sample and
heartbeat events within bounded observation controls. Its command-local event
contract is marked by `contract_version: "observe_chart.v1"` and
`_observe: "chart"`.

Lower-level `tv stream ...` commands are selected-chart Desktop-backed JSONL
sample surfaces for specific stream kinds. Their sample and heartbeat events
are marked by `contract_version: "stream.v1"`, while preserving `_event`,
`_stream`, source metadata, sample payloads, and heartbeat counters.

Neither surface is Desktop-free browserless historical bars, scanner quote,
chart-source quote, quote-data, a watch loop, or a multi-symbol realtime feed.

## Plan of Work

Archive the completed implementation plan and update durable planning docs so
this audit is the current plan. Then inspect docs, runtime skills, help, and
tests for consistency around `observe_chart.v1`, `stream.v1`, readiness,
sample, heartbeat, and `desktop_chart_stream` source metadata.

Run focused observe / stream checks first, then the full Rust baseline and
repository hygiene scans. If all checks pass, record in this ExecPlan and the
roadmap that the next step is `v0.18.0 release readiness`.

## Concrete Steps

From the repository root, run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "observe_chart\\.v1|stream\\.v1|contract_version|JSONL|readiness|heartbeat|desktop_chart_stream|observe chart|stream|realtime|watch|JSONL compare|chart-backed compare|auto fallback|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

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

Validation completed for this audit:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg hygiene scans for private data, TODO / panic markers, and deferred-surface wording
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_observe_chart

Optional live smoke can be run if explicitly useful:

    TV_LIVE_OBSERVE_CHART_SMOKE=1 cargo test -p tradingview-cli --test live_observe_chart -- --ignored --nocapture

Do not paste raw JSONL live output into tracked docs. If smoke is recorded,
keep only a public-safe summary.

## Validation and Acceptance

Acceptance is met when the focused tests and full baseline pass, the docs and
skills consistently describe `observe_chart.v1` and `stream.v1`, and no
release blocker is found.

The audit must confirm:

- `tv observe chart` readiness, sample, and heartbeat events expose
  `contract_version: "observe_chart.v1"` and `_observe: "chart"`.
- observe sample and heartbeat events keep underlying selected-chart stream
  metadata such as `_stream: "bars"` and `source: "desktop_chart_stream"`.
- `tv stream ...` sample and heartbeat events expose
  `contract_version: "stream.v1"` while preserving existing `_event`,
  `_stream`, sample payload, heartbeat, and bounded-control semantics.
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
