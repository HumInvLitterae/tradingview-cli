# Chart quote session boundary and extended-hours feasibility

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and prepares the first `v0.13.0` implementation slice.

## Purpose / Big Picture

Agents can mistake `tv quote <SYMBOL> --source chart` for a source that also
provides scanner-style premarket and postmarket fields. Today chart-source
quote reads the selected TradingView Desktop chart's main-series bars and chart
symbol metadata, while scanner-backed quote reads request explicit
extended-hours scanner columns.

This plan sets up the next slice: investigate whether chart-source quote can
safely expose extended-hours or session-boundary metadata, and if not, make
the limitation impossible for agents to miss.

## Progress

- [x] (2026-05-08T09:40Z) Created this plan after `v0.12.0` release
  readiness and v0.13 roadmap planning.
- [x] (2026-05-08T09:40Z) Added `docs/v0.13-roadmap.md`, updated the plan
  index and v0.12 roadmap, and archived the completed v0.12 release-readiness
  plan.
- [x] (2026-05-08T09:40Z) Ran docs validation, package script syntax check,
  and public-safety hygiene grep.
- [x] (2026-05-08T10:35Z) Inspected the chart-source quote adapter and
  confirmed it reads `chart.symbol()`, `chart.symbolExt()`, and the selected
  chart main-series `bars.valueAt(lastIndex)` rather than scanner
  extended-hours fields.
- [x] (2026-05-08T10:35Z) Chose explicit unavailable / not-guaranteed
  readback for chart-source quote by adding additive `session_boundary`
  metadata.
- [x] (2026-05-08T10:35Z) Updated docs, runtime skills, and tests to prevent
  source/session misreads.
- [x] (2026-05-08T10:45Z) Ran focused quote tests, full workspace tests,
  clippy, metadata, diff check, package script syntax check, and hygiene grep.
- [x] (2026-05-08T10:50Z) Commit the completed implementation slice.

## Surprises & Discoveries

- Observation: the current repository already clearly separates scanner
  extended-hours fields from chart-source quote implementation.
  Evidence: scanner quote requests explicit premarket/postmarket scanner
  columns, while chart-source quote reads selected chart bars and `symbolExt`.

- Observation: static inspection of the chart-source quote adapter found no
  existing stable chart-source premarket or postmarket field.
  Evidence: the adapter builds the quote payload from selected chart symbol
  metadata and the last main-series bar values: `time`, `open`, `high`, `low`,
  `close`, `last`, and `volume`.

- Observation: the hygiene grep reported existing policy language, archived
  validation examples, and this plan's safety wording.
  Evidence: no new local path, raw live payload, target id, account-local
  metadata, cookie, token, or authorization value was added.

## Decision Log

- Decision: Do not merge scanner `extended_hours` into chart-source quote
  payloads during the first slice.
  Rationale: scanner and chart source boundaries are intentionally different.
  Mixing them would make provenance harder for agents to reason about.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat chart-source extended-hours support as unconfirmed until
  page-object evidence proves it.
  Rationale: existing Rust code reads main-series bars and `symbolExt`, while
  scanner extended-hours values come from explicit scanner REST columns.
  Date/Author: 2026-05-08 / Codex.

- Decision: Ship `session_boundary` as an explicit not-provided readback
  instead of adding premarket or postmarket price fields in this slice.
  Rationale: this prevents agent misreads while preserving source provenance.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

The implementation added additive `session_boundary` metadata to chart-source
quote payloads and chart-source quote error details. The metadata states that
the price comes from the selected chart main-series last bar, that the price
session is unknown, and that scanner-style extended-hours values are not
provided or guaranteed by this source.

The slice also updated docs and runtime skills so agents use scanner-backed
`tv quote`, `tv quotes`, `tv snapshot`, or `tv compare` when premarket or
postmarket fields matter.

## Context and Orientation

The chart-source quote implementation lives in the CLI operation adapter under
`crates/cli/src/ops/market/quote.rs`. It evaluates TradingView Desktop page
objects, reads the selected chart's current symbol and main-series bars, and
uses readiness checks to avoid stale symbol data.

Scanner quote reads live in `tradingview-market` and request extended-hours
columns such as `premarket_close` and `postmarket_close`. Those values are
returned as `extended_hours.premarket` and `extended_hours.postmarket`.

The next implementation must preserve that source distinction.

## Plan of Work

First, inspect chart page objects from a Desktop-backed read-only probe. The
probe should summarize available method names and high-level value shapes only;
it must not write raw live payloads, target ids, account-local metadata, or
local paths into tracked docs.

Look specifically for safe evidence around:

- whether main-series bars include extended-session bars when the chart is
  configured for extended hours;
- whether chart API, main series, `symbolExt`, or visible session settings
  expose a stable session marker;
- whether `last`, `close`, `time`, and `volume` can be labeled as regular,
  extended, or unknown without guessing.

If stable chart-source extended-hours evidence exists, add only additive
metadata to chart-source quote and tests. If evidence is absent or unstable,
add explicit metadata and docs saying extended-hours values are unavailable or
not guaranteed from chart-source quote, and direct agents to scanner-backed
reads for premarket/postmarket fields.

Do not change scanner quote, `quotes`, `snapshot`, `compare`, `observe chart`,
or `ohlcv` contracts except for docs / skills needed to clarify source
selection.

## Validation and Acceptance

The implementation slice should include:

- focused tests for chart-source quote payload metadata or explicit
  unavailable/not-guaranteed readback;
- contract tests proving scanner quote still exposes `extended_hours` and
  chart-source quote does not silently pretend to;
- docs and runtime skill updates that tell agents to use scanner-backed quote,
  `snapshot`, `quotes`, or `compare` when premarket/postmarket values matter;
- no automatic scanner fallback or scanner-value injection after chart
  mutation starts.

Run, at minimum:

    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id|downstream)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts || true
    rg -n "session_boundary|extended_hours|premarket|postmarket|quote --source chart|chart_quote|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Optional live smoke may use `tv readiness`, `tv quote <SYMBOL> --source
chart`, and `tv quote <SYMBOL> --source scanner`, but tracked docs must contain
only public-safe summaries.

## Idempotence and Recovery

This work is safe to rerun if it remains additive. If chart page-object probing
finds no stable extended-hours source, do not keep digging by adding fragile
DOM scraping. Prefer explicit unavailable/not-guaranteed metadata and clear
agent guidance.

## Interfaces and Dependencies

No new command, option, dependency, or source fallback should be added in the
first implementation slice. Any payload change must be additive.

## Open Questions

None for planning. The implementation slice must answer the technical
feasibility question with read-only evidence.

## Validation Evidence

Passed:

    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id|downstream)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts || true
    rg -n "session_boundary|extended_hours|premarket|postmarket|quote --source chart|chart_quote|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
