# Compare follow-up contract polish

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to make `tv compare <SYMBOL>...` easier for
downstream agents to consume without adding rankings, recommendations, new
data sources, or chart automation.

## Purpose / Big Picture

`tv compare <SYMBOL>...` already returns a Desktop-free multi-symbol evidence
packet. After `v0.11.0`, it also returns a command-local contract marker,
requested-order indexes, follow-up hints, and field coverage counts.

The next improvement is to make those metadata fields stable enough that
downstream agents can join results back to requested symbols, summarize
coverage, and choose explicit follow-up commands without reinterpreting raw
items. After this change, a caller should be able to read stable follow-up
hint values, field coverage count semantics, requested-order guarantees, a
thin coverage status, and compare failure details with the same contract guard.

## Progress

- [x] (2026-05-08T06:23Z) Created this ExecPlan after the `v0.11.0` release
  and recorded the initial v0.12 compare follow-up contract direction.
- [x] (2026-05-08T07:42Z) Audited current compare payload construction,
  contract tests, docs, and runtime skills.
- [x] (2026-05-08T07:42Z) Stabilized follow-up hint vocabulary and field
  coverage semantics in docs and tests.
- [x] (2026-05-08T07:42Z) Added additive `summary.coverage_status` readback.
- [x] (2026-05-08T07:42Z) Ensured structured total-failure details expose the
  compare contract marker and blocked coverage readback.
- [x] (2026-05-08T07:42Z) Ran focused compare tests, full Rust baseline, docs
  validation, runtime skill validation, and package script validation.
- [x] (2026-05-08T07:42Z) Updated stable docs, runtime skills, roadmap, and
  changelog for the completed slice.
- [x] (2026-05-08T07:42Z) Prepared the completed slice for a single local
  commit.

## Surprises & Discoveries

- Observation: downstream feedback asks for `quote_chart`, while the current
  `v0.11.0` compare payload uses `chart_quote`.
  Evidence: the current code and tests should be audited before implementation
  to confirm the shipped value. The initial decision is to keep the shipped
  value stable instead of renaming it.

- Observation: structured total-failure already returned the compare payload
  in error details.
  Evidence: the implementation added `coverage_status` through the existing
  `CompareSummary`, and the new unit test verifies error details contain
  `contract_version: "compare.v1"` and `summary.coverage_status: "blocked"`.

- Observation: the broad hygiene grep reports existing safety policy wording,
  archived validation-command examples, and deferred-surface wording.
  Evidence: no new local path, credential, raw target id, account-local
  metadata, raw live payload, downstream repo path, or downstream private
  workflow name was added by this slice.

## Decision Log

- Decision: Keep `chart_quote` as the stable follow-up hint kind instead of
  renaming it to `quote_chart`.
  Rationale: `chart_quote` shipped in `v0.11.0`; renaming it would break
  downstream consumers that already adopted the new metadata. If an alias is
  ever needed, add it in a separate additive plan.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat `coverage_status` as evidence coverage, not a trading
  signal.
  Rationale: downstream needs a compact readback such as `complete`,
  `partial`, or `blocked`, but `compare` must remain an evidence command and
  not become a ranking or recommendation engine.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Implemented. `tv compare <SYMBOL>...` now serializes additive
`summary.coverage_status` with `complete`, `partial`, or `blocked` values.
The field is derived from already finalized compare items and does not add
network reads, source changes, ranking, scoring, or recommendations. Existing
`items[]`, section errors, `summary.resolved_symbols[]`, `field_coverage`,
`follow_up_hints[]`, top-level counts, `errors[]`, `next_action_hints`, and
source metadata remain intact.

Stable docs and runtime skills now explain that `coverage_status` is evidence
coverage only. The total-failure path remains structured and now has test
coverage proving that the compare payload in error details retains
`contract_version: "compare.v1"` and blocked coverage status.

## Context and Orientation

The repository is a Cargo workspace for the Rust-native `tv` CLI. The
Desktop-free market evidence code lives under `crates/market/`. The CLI
contract tests live under `crates/cli/tests/`. Stable documentation lives under
`docs/`, and runtime agent skills live under `.agents/skills/`.

`compare` is a Desktop-free command. It must not use TradingView Desktop, CDP,
chart switching, screenshots, or the lab-gated `tv bars` prototype. Its raw
`items[]` array remains the evidence source. Summary and follow-up metadata
are readback helpers that make the raw evidence easier to scan and route.

The current `v0.11.0` compare contract includes:

- `data.contract_version: "compare.v1"`;
- `items[].requested_index`;
- `summary.resolved_symbols[].requested_index`;
- `items[].follow_up_hints[]`;
- `summary.field_coverage`;
- existing top-level counts, `items[]`, `errors[]`, and `next_action_hints`.

The implementation must preserve all of those existing fields and may only add
metadata.

## Plan of Work

First audit `crates/market/src/types.rs`, `crates/market/src/compare.rs`, and
the compare tests to confirm the current shipped field names and values. Treat
that audit as source of truth for compatible changes.

Then update the typed compare payload to add `summary.coverage_status`. The
status values are:

- `complete`: every requested item has evidence and the total missing count is
  zero;
- `partial`: at least one item has evidence, but some section error or missing
  evidence remains;
- `blocked`: the compare payload is structured, but no requested item has
  usable evidence.

The status is about evidence coverage only. It does not rank securities or
suggest trading action.

Stabilize follow-up hint vocabulary in code comments or tests where useful,
and in public docs. The stable `follow_up_hints[].kind` values are:

- `snapshot`;
- `observe_chart`;
- `chart_quote`;
- `screenshot`.

The stable `follow_up_hints[].reason` values remain readback strings naming why
that follow-up surface is available, such as one-symbol detail, selected-chart
observation, single-symbol chart quote, or visual evidence.

Document `summary.field_coverage` semantics. Counts are calculated across the
validated requested items in input order. Section ok counts count item-level
section success. Missing counts are derived from existing missing fields and
missing summaries; they do not infer whether missing data is good or bad.
Earnings and dividends counts only cover fields that already belong to the
existing fundamentals groups.

Ensure structured total-failure details still include a compare payload with
`contract_version: "compare.v1"`, `summary`, `items`, `errors`, and source
metadata. If the existing failure path already includes those fields, add tests
and docs rather than changing behavior.

Finally update `docs/observation-workflows.md`,
`docs/command-source-taxonomy.md`, the relevant runtime skills, and
`CHANGELOG.md` so they describe metadata as readback helpers, not evidence
replacement or recommendations.

## Concrete Steps

From the repository root:

    rg -n "contract_version|requested_index|follow_up_hints|field_coverage|missing_summary" crates/market crates/cli/tests docs .agents/skills

Edit the market compare types and summary construction only as needed for the
additive `summary.coverage_status` and failure-side contract guard. Do not add
new CLI options, new commands, dependencies, or network reads.

Update compare contract tests so they prove:

- successful compare payloads include `summary.coverage_status`;
- the status is `complete`, `partial`, or `blocked`;
- `follow_up_hints[].kind` uses the stable values;
- `items[]` and `summary.resolved_symbols[]` keep requested-order indexes;
- structured total-failure details include `contract_version: "compare.v1"`;
- existing `items[]`, `errors[]`, `next_action_hints`, source metadata, and
  top-level counts are still present.

Run the validation commands listed below and record the outcomes in this plan.

Validation evidence recorded for this completed slice:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

The three changed runtime skills were also validated with the existing local
skill validator. The exact validator path is local tooling and is intentionally
not recorded in this public plan.

## Validation and Acceptance

Run:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional read-only smoke:

    target/debug/tv compare NASDAQ:AAPL NYSE:IONQ

Acceptance is met when `tv compare` still works without TradingView Desktop,
existing fields are unchanged, `summary.coverage_status` is present, follow-up
hint values and field coverage semantics are documented and tested, and failure
details retain the compare contract marker.

## Idempotence and Recovery

This slice is additive. Re-running tests and docs validation is safe. If
coverage status logic proves ambiguous, keep the implementation conservative:
prefer `partial` when any evidence exists but completeness is uncertain, and
reserve `blocked` for no usable per-item evidence.

If a test reveals that total-failure details already include the required
contract marker, do not refactor the error path just for aesthetics. Add a
focused assertion and document the behavior.

## Interfaces and Dependencies

No new dependency is required. The relevant public JSON additions are limited
to `tv compare <SYMBOL>...`.

The expected additive field is:

    data.summary.coverage_status: "complete" | "partial" | "blocked"

The stable compare follow-up hint kinds are:

    "snapshot"
    "observe_chart"
    "chart_quote"
    "screenshot"

`contract_version` remains command-local to `compare`; it is not a global
envelope schema version.

## Open Questions

None for this completed slice. Larger item-level missing evidence and
cross-command vocabulary alignment remain later v0.12 candidates.
