# Quote help source-boundary wording

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The top-level help currently describes `tv quote` as a real-time quote command.
That is too broad now that quote reads can come from scanner REST, selected
chart main-series bars, or explicit Desktop quote-data WebSocket readback.
Scanner-backed quote and quotes are Desktop-free, but they can be delayed; the
payload's `time`, `update_mode`, and `delay_seconds` are the freshness
readback.

This slice changes help/docs/tests only. It does not change quote behavior,
JSON payloads, source selection, or extended-hours normalization.

## Progress

- [x] (2026-05-10T03:10Z) Created this ExecPlan and archived the updated
  v0.13 pre-release audit.
- [x] (2026-05-10T03:20Z) Updated quote and quotes help wording.
- [x] (2026-05-10T03:25Z) Updated contract tests and minimal public docs.
- [x] (2026-05-10T03:40Z) Ran validation.

## Surprises & Discoveries

- Observation: the long help already explains most source boundaries, but the
  opening sentence and top-level summary still say `real-time`.
  Evidence: `tv --help` prints `quote         Get real-time price quote`, and
  `tv quote --help` starts with `Get a real-time price quote.`

- Observation: `quotes` already says scanner-backed in long help, but the
  top-level summary only says Desktop-free.
  Evidence: `tv --help` prints `quotes        Get Desktop-free quotes for
  multiple symbols`.

## Decision Log

- Decision: Replace `real-time` with source-labeled wording for `quote`.
  Rationale: source and freshness guarantees differ by scanner, chart, and
  quote-data paths. The help should tell users to inspect freshness metadata
  instead of implying a universal real-time guarantee.
  Date/Author: 2026-05-10 / Codex.

- Decision: Keep `quotes` explicitly scanner-backed in help.
  Rationale: `quotes` does not use chart or quote-data sources, and its
  Desktop-free scanner REST result can be delayed.
  Date/Author: 2026-05-10 / Codex.

## Outcomes & Retrospective

Implementation is complete. `tv --help` now describes `quote` as
source-labeled quote data and `quotes` as scanner-backed multi-symbol quotes.
`tv quote --help` and `tv quotes --help` now explicitly say scanner-backed
reads are not a realtime guarantee and point users to `time`, `update_mode`,
and `delay_seconds` for freshness.

No quote behavior, source selection, JSON payload, or extended-hours shape was
changed. Focused contract tests and the workspace Rust baseline passed. The
next step is `v0.13.0 release readiness`.

## Plan of Work

Update the clap command help for `quote` and `quotes` so `tv --help`,
`tv quote --help`, and `tv quotes --help` all communicate:

- scanner-backed reads are Desktop-free REST reads and may be delayed;
- freshness lives in `time`, `update_mode`, and `delay_seconds`;
- chart-source quote is selected chart main-series data and not scanner-style
  extended-hours evidence;
- quote-data is an explicit Desktop-backed WebSocket readback and is not part
  of `--source auto`.

Update focused CLI contract tests so this wording stays stable. Sync README,
source taxonomy, observation workflow docs, and runtime skills only where they
still imply scanner-backed quote is real-time.

## Concrete Steps

Run from the repository root:

    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

## Validation and Acceptance

Acceptance is met when `quote` and `quotes` help no longer imply real-time
scanner-backed data, tests lock the source/freshness wording, and no behavior
or JSON contract changes are introduced.

## Idempotence and Recovery

This is a documentation and help-text slice. If tests reveal a behavior issue,
stop and create a separate implementation plan rather than mixing behavior
changes into this wording correction.

## Open Questions

None. `stream` help is intentionally out of scope.
