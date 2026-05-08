# Desktop quote-data RTC source feasibility and contract design

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and covers the next `v0.13.0` source-design slice.

## Purpose / Big Picture

TradingView Desktop can display a right-side after-hours price that is not the
selected chart main-series last bar and is not scanner REST. Recent bounded
evidence narrowed the strongest current backing-source candidate to
TradingView WebSocket quote-data messages, specifically the `qsd.rtc` field.
This plan does not expose that value yet. It prepares the next implementation
or research slice by fixing the source boundary, failure behavior, and public
contract that must exist before any `qsd.rtc`-based read is shipped.

After this plan is implemented, a future contributor should be able to decide
whether to build a labeled Desktop quote-data source such as
`tv quote <SYMBOL> --source quote-data`, or to keep the result as evidence
tooling only. The important user-visible outcome is that agents must not
mistake chart main-series quote, scanner `extended_hours`, visible UI, and
WebSocket quote-data as interchangeable sources.

## Progress

- [x] (2026-05-09T23:35Z) Created this ExecPlan and made it the current
  source-design plan.
- [x] (2026-05-09T23:35Z) Archived the completed WebSocket correlation
  evidence plan.
- [x] (2026-05-09T23:45Z) Updated roadmap, source taxonomy docs, workflow
  docs, skills, and changelog with the quote-data RTC source-design boundary.
- [x] (2026-05-09T23:50Z) Ran docs, packaging, skill, and compile-only smoke
  validation before commit.

## Surprises & Discoveries

- Observation: The strongest current source candidate is no longer only a
  generic WebSocket numeric token. It is a quote-data message field:
  `qsd.rtc`, with related readbacks such as `rtc_time`, `rch`, and `rchp`.
  Evidence: The archived WebSocket correlation plan recorded a bounded RKLB
  postmarket run where visible after-market samples matched RKLB `qsd.rtc`
  candidates while regular close-like values remained separate.

- Observation: `qsd.rtc` is promising but still internal. The project has not
  shipped a stable source contract for WebSocket quote-data messages.
  Evidence: Current public quote surfaces still distinguish scanner REST
  `extended_hours`, selected chart main-series quote, and visible-panel source
  discovery. No public command reads `qsd.rtc` today.

## Decision Log

- Decision: Do not add `qsd.rtc` to `tv quote --source chart`.
  Rationale: chart-source quote currently means selected chart main-series
  last bar. Adding quote-data WebSocket values to that payload would blur the
  source boundary that v0.13 is trying to make explicit.
  Date/Author: 2026-05-09 / Codex.

- Decision: Do not merge scanner REST `extended_hours` with quote-data
  WebSocket readback.
  Rationale: scanner REST can be delayed and has a different source contract.
  Quote-data WebSocket values may have different entitlement, timing, symbol,
  and session semantics. Combining them would make agents overconfident.
  Date/Author: 2026-05-09 / Codex.

- Decision: If this becomes public, design it as a separate source candidate:
  `tv quote <SYMBOL> --source quote-data`.
  Rationale: a separate source label lets payloads report
  `source: "desktop_quote_data_ws"` and `source_category:
  "desktop_backed_read"` without changing chart main-series behavior.
  Date/Author: 2026-05-09 / Codex.

- Decision: Treat `rtc`, `rtc_time`, `rch`, `rchp`, `current_session`,
  `market_phase`, and `update_mode` as source-labeled readbacks, not as a
  scanner-style `extended_hours` object.
  Rationale: available evidence shows `rtc` can match visible after-hours
  price, but it does not yet prove complete regular/premarket/postmarket
  semantics across symbols and phases.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

This docs-only source-design slice records the boundary for a later
`quote-data` source implementation. The next slice can either design and
implement bounded `qsd.rtc` support as a separate source, or keep `qsd.rtc` as
research tooling until more premarket/postmarket evidence exists.

## Context and Orientation

There are currently three distinct market-data reads that agents must keep
separate:

- Scanner REST quote reads are Desktop-free and can return a delayed
  `extended_hours` object. They are used by `tv quote <SYMBOL>`,
  `tv quotes <SYMBOL>...`, `tv snapshot <SYMBOL>`, and `tv compare
  <SYMBOL>...`.
- Chart-source quote reads are Desktop-backed and use the selected TradingView
  chart's main-series last bar. They are used by `tv quote --source chart` and
  report `session_boundary` to say that scanner-style extended-hours values
  are not included.
- Desktop quote-data WebSocket messages are internal TradingView traffic. The
  recent source-discovery smoke found `qsd.rtc` values that matched the
  visible right-side after-hours panel price, but this is not yet a public CLI
  source.

The current chart-source quote implementation lives in
`crates/cli/src/ops/market/quote.rs`. Scanner-backed quote normalization lives
in `crates/market/src/quote.rs`. The current evidence smoke lives in
`crates/cli/tests/live_after_hours_ws_correlation.rs`. This plan should not
change user-facing JSON payloads; it should define the next implementation
contract clearly enough that a later plan can add the source safely.

## Plan of Work

First, keep this slice docs-first. Move the completed WebSocket correlation
plan into `docs/plans/archives/`, then make this file the current plan in
`docs/plans/README.md` and `docs/v0.13-roadmap.md`.

Next, record the public contract boundary. The future source candidate is
`tv quote <SYMBOL> --source quote-data`, not an extension of `--source chart`
and not a scanner fallback. Its payload source candidate is
`desktop_quote_data_ws`, with `source_category: "desktop_backed_read"`,
`requires_desktop: true`, and `non_mutating: true` when it observes the
currently selected symbol without switching. Any future symbol-switching
behavior must be explicitly planned later and must not be inferred by this
slice.

Then, define the minimum bounded-read feasibility that a later implementation
must prove. A read is feasible only if the CLI can observe quote-data messages
for the currently selected chart or right-side detail symbol, attribute them
to the requested public symbol without raw frame dumps, and return a
structured unavailable/timing result when no matching `qsd.rtc` arrives during
the bounded window. The success condition is not equality with scanner REST;
the success condition is a public-safe summary showing that the visible panel
value and `qsd.rtc` agree during the same run.

Finally, update docs and runtime skills to keep the source boundary clear.
The wording should say that `qsd.rtc` is the strongest current backing-source
candidate for the visible after-hours panel value, but it remains unshipped
source discovery until a separate `quote-data` contract exists.

## Concrete Steps

Work from the repository root.

Archive the completed plan and add this one as current:

    mkdir -p docs/plans/archives
    mv docs/plans/tradingview-cli-after-hours-websocket-correlation.md docs/plans/archives/tradingview-cli-after-hours-websocket-correlation.md

Update the following tracked docs without adding local paths, raw frame
contents, raw DOM, target ids, account-local metadata, or raw live payloads:

- `docs/plans/README.md`
- `docs/v0.13-roadmap.md`
- `docs/internal-tradingview-apis.md`
- `docs/command-source-taxonomy.md`
- `docs/observation-workflows.md`
- `.agents/skills/market-data-interpretation/SKILL.md`
- `.agents/skills/chart-analysis/SKILL.md`
- `CHANGELOG.md`

Compile the existing ignored smoke after docs updates to ensure the source
evidence tooling still builds:

    cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --nocapture

Run the docs-only validation and commit:

    git diff --check
    bash -n scripts/stage-release-package-files.sh

If runtime skills changed, validate them with the existing skill validator.
Do not run an opt-in live smoke unless the user explicitly asks for another
postmarket/premarket evidence run.

## Validation and Acceptance

Acceptance is met when the repository has a current ExecPlan for the
quote-data RTC source design, the previous WebSocket correlation plan is
archived, and docs consistently say:

- `qsd.rtc` is the strongest current candidate for the visible right-side
  after-hours value;
- it is still internal source-discovery evidence;
- it is not part of `tv quote --source chart`;
- it is not scanner REST `extended_hours`;
- any public support should be a separately labeled source such as
  `--source quote-data`.

Run:

    cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --nocapture
    git diff --check
    bash -n scripts/stage-release-package-files.sh

If code changes accidentally enter the slice, also run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

## Idempotence and Recovery

This is a docs and planning slice. It should be safe to repeat validation
commands. If the archive move is accidentally repeated, restore the active
plan by checking `docs/plans/README.md` and ensuring only this file is listed
as current. Do not delete the archived evidence plan; future contributors need
it as rationale.

If a later live smoke shows `qsd.rtc` does not match the visible value for a
new symbol or session phase, update `Surprises & Discoveries` and keep
`quote-data` support deferred until the attribution is clear.

## Artifacts and Notes

Public-safe evidence summary to preserve in docs:

    RKLB postmarket visible right-side after-hours samples matched qsd.rtc
    candidates in bounded WebSocket quote-data summaries. lp and
    regular_close remained regular close-like readbacks. qsd.rtc is the
    strongest current backing-source candidate but not a public payload field.

Do not include HAR paths, raw WebSocket frame payloads, raw DOM text, raw
React props, raw screenshots, target ids, cookies, authorization values,
account-local metadata, or local absolute paths in tracked repository files.

## Interfaces and Dependencies

No new public interface is added by this slice. The design target for a future
slice is:

    tv quote <SYMBOL> --source quote-data

The future payload should be additive and source-labeled. Candidate fields:

    source: "desktop_quote_data_ws"
    source_category: "desktop_backed_read"
    requires_desktop: true
    non_mutating: true
    requested_symbol
    observed_symbol
    quote_data: {
      rtc,
      rtc_time,
      rch,
      rchp,
      current_session,
      market_phase,
      update_mode
    }

Those names are design candidates, not implemented fields. A later
implementation plan must decide the final schema and tests before modifying
CLI behavior.

## Open Questions

- Can the CLI reliably subscribe to or observe `qsd` quote-data messages for
  the requested/current symbol without mutating the chart or quote-session
  fields?
- Is `rtc` stable across postmarket and premarket, or only observed so far in
  postmarket evidence?
- Should a future `quote-data` source require the symbol to already be visible
  in the selected chart/right-side panel, or should it support explicit
  symbol-targeted reads?
- What structured unavailable result should be returned when no matching
  quote-data frame arrives within a bounded window?
