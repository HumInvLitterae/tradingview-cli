# JSONL observation contract maturity

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It plans
the first `v0.18.0` implementation slice after `v0.17.0`.

## Purpose / Big Picture

`tv observe chart` and `tv stream ...` already provide bounded
Desktop-backed JSONL observation. They are useful, but their event contract is
less mature than the recently hardened `compare.v1`, `quote_data.v1`, and
`bars.v1` payloads.

This slice will make JSONL events easier for agents and downstream tools to
read safely by adding additive contract metadata and clarifying readiness,
sample, heartbeat, and source-boundary semantics. It does not add a new
market-data source, a multi-symbol realtime feed, watch / JSONL compare,
source mixing, ranking, or recommendations.

## Progress

- [x] (2026-05-16T00:00Z) Create this ExecPlan and move the completed
  `v0.17.0` release-readiness plan to archives.
- [x] (2026-05-16T00:00Z) Add the `v0.18.0` roadmap direction.
- [ ] Add additive JSONL contract metadata to `tv observe chart` readiness,
  sample, and heartbeat events.
- [ ] Add compatible additive contract metadata to lower-level
  `tv stream ...` sample and heartbeat events.
- [ ] Update docs, runtime skills, and contract tests.
- [ ] Run focused tests, baseline, docs validation, and hygiene checks.

## Surprises & Discoveries

- No surprises have been recorded yet.

## Decision Log

- Decision: Start `v0.18.0` with JSONL observation contract maturity instead
  of new realtime or watch-loop behavior.
  Rationale: `tv bars` is now stable historical evidence, while existing
  Desktop-backed JSONL observation still needs clearer event contracts before
  downstream tools should build richer workflows on it.
  Date/Author: 2026-05-16 / Codex.

- Decision: Keep `tv observe chart` and `tv stream ...` selected-chart,
  Desktop-backed observation surfaces.
  Rationale: They should not be confused with Desktop-free browserless
  historical bars, scanner quote reads, chart-source quote, or quote-data
  readback.
  Date/Author: 2026-05-16 / Codex.

## Outcomes & Retrospective

This section will be completed after implementation and validation.

## Plan of Work

Add command-local JSONL contract metadata to the existing observation events
without changing event meaning. Keep readiness, sample, heartbeat, source
metadata, bounded controls, dedupe behavior, and non-mutating semantics intact.

Do not add new commands, options, data sources, realtime multi-symbol feeds,
watch loops, source mixing, ranking, scoring, recommendations, or release
version bumps in this slice.

## Concrete Steps

1. Inspect current `stream` and `observe` helpers, CLI contract tests, and
   ignored live smoke expectations.
2. Add additive event metadata for `tv observe chart`:
   - command-local `contract_version`;
   - event kind remains `readiness`, `sample`, or `heartbeat`;
   - source metadata remains `desktop_chart_stream` /
     `desktop_backed_read` for sample and heartbeat events;
   - readiness remains first event.
3. Add compatible additive metadata for lower-level `tv stream ...`:
   - preserve existing `_event`, `_stream`, source metadata, and sample
     payload fields;
   - preserve heartbeat sample-count readback.
4. Update CLI contract tests and live smoke tests so compile-only CI confirms
   the new additive fields without requiring live Desktop evidence.
5. Update public docs and runtime skills with the new event contract wording.
6. Run validation and hygiene checks.

## Acceptance Criteria

- `tv observe chart` JSONL events carry the new additive contract metadata on
  readiness, sample, and heartbeat events.
- `tv stream ...` sample and heartbeat events carry compatible additive
  contract metadata.
- Existing JSONL event shapes are not broken: `_event`, `_stream`, source
  metadata, sample payloads, heartbeat sample counts, and bounded controls are
  preserved.
- `tv observe chart` remains non-mutating and does not switch symbols,
  activate tabs, capture screenshots, or read browserless bars.
- `tv stream ...` remains a lower-level selected-chart observation surface,
  not a multi-symbol realtime feed or watch loop.
- Docs and runtime skills explain how to read readiness, sample, heartbeat,
  and source metadata without mixing observe / stream with scanner, bars,
  chart quote, or quote-data.
- No raw live payloads, raw DOM, raw WebSocket frames, target ids,
  account-local metadata, credentials, or local absolute paths are added to
  tracked docs.

## Validation

Focused tests:

    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_observe_chart

Baseline and docs checks:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "v0\\.18|observe chart|stream|JSONL|contract_version|heartbeat|readiness|desktop_chart_stream|bars\\.v1|quote-data|realtime|watch|JSONL compare|chart-backed compare|auto fallback|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true

Optional live smoke, only when useful:

    TV_LIVE_OBSERVE_CHART_SMOKE=1 cargo test -p tradingview-cli --test live_observe_chart -- --ignored --nocapture

Live smoke output must not be pasted into tracked docs except as a public-safe
summary.
