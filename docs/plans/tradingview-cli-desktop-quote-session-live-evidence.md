# Desktop quote session extended-hours live evidence

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and prepares the next `v0.13.0` evidence slice.

## Purpose / Big Picture

Agents need to know whether extended-hours evidence came from scanner REST, the
selected chart's main-series bar, or TradingView Desktop's page quote session.
`tv quote <SYMBOL> --source chart` currently reads the selected chart
main-series last bar and must not be treated as premarket or postmarket
evidence. Recent live probing found that the Desktop page also exposes a quote
session with `premarket_*` and `postmarket_*` field names, but regular-session
values did not behave like scanner-backed premarket values. This work adds an
opt-in live smoke so maintainers can collect public-safe postmarket and
premarket evidence before deciding whether to expose those fields in a future
payload.

After this change, a maintainer can wait for postmarket or premarket, run one
ignored Rust integration test, and see a compact source-by-source summary:
scanner extended-hours fields versus Desktop quote-session selected fields.
The test does not add a new user-facing command, does not modify
`tv quote --source chart`, and does not make live evidence part of normal CI.

## Progress

- [x] (2026-05-09T00:00Z) Reframed v0.13 from "chart-source quote cannot
  provide extended-hours" to "Desktop quote session has promising fields that
  require session-phase evidence".
- [x] (2026-05-09T00:00Z) Archived the completed chart-source session-boundary
  plan and created this quote-session live-evidence plan.
- [x] (2026-05-09T00:00Z) Added an ignored Rust integration test for opt-in
  Desktop quote-session extended-hours evidence.
- [x] (2026-05-09T00:00Z) Updated current roadmap, development docs, source
  taxonomy docs, workflow docs, internal API notes, runtime skills, and
  changelog.

## Surprises & Discoveries

- Observation: `window.getQuoteSessionInstance()` is available in the
  TradingView Desktop chart page.
  Evidence: read-only `tv ui eval` probes returned an object with quote
  session methods such as `setFields`, `subscribe`, and `unsubscribe`.

- Observation: a temporary quote-session subscription can return field names
  such as `premarket_close`, `postmarket_close`, `session-premarket`,
  `session-postmarket`, and `market-status`.
  Evidence: a public-safe probe against a current chart returned selected
  pre/post field names without requiring raw live payloads in tracked docs.

- Observation: regular-session quote-session pre/post fields should not be
  interpreted as scanner-style extended-hours evidence.
  Evidence: during regular session, selected quote-session `premarket_close`
  and `postmarket_close` tracked the streaming current price, while
  scanner-backed `extended_hours.premarket.close` reflected a distinct delayed
  premarket value.

## Decision Log

- Decision: Add an opt-in live smoke before adding any premarket or postmarket
  value to chart-source quote payloads.
  Rationale: the fields exist, but their session-phase semantics are not stable
  enough to expose as a public payload contract.
  Date/Author: 2026-05-09 / Codex.

- Decision: Do not use scanner equality as the success criterion for the live
  smoke.
  Rationale: scanner REST can be delayed, while Desktop quote session fields
  can be streaming and entitlement-dependent. The first goal is provenance and
  session-phase evidence, not value equality.
  Date/Author: 2026-05-09 / Codex.

- Decision: Keep Desktop quote session separate from chart main-series quote.
  Rationale: `tv quote --source chart` reads selected chart bars today. Mixing
  scanner extended-hours or Desktop quote-session values into that payload
  without a separate source label would make agent reasoning worse.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

This slice adds tooling and documentation for collecting postmarket and
premarket evidence. It does not yet decide whether Desktop quote-session
extended-hours fields are suitable for stable payload support. That decision
requires running the ignored smoke during the relevant market phases and
recording only public-safe summaries.

## Context and Orientation

Chart-source quote code lives in `crates/cli/src/ops/market/quote.rs`. It uses
TradingView Desktop page objects to read the current chart symbol and selected
chart main-series bars. It already reports `session_boundary` metadata saying
that scanner-style extended-hours values are not provided by that source.

Scanner-backed quote code lives in `crates/market/src/quote.rs`. It reads
TradingView scanner REST fields without Desktop or CDP and reshapes
`premarket_*` and `postmarket_*` columns into the nested
`extended_hours.premarket` and `extended_hours.postmarket` objects.

The Desktop page quote session is different from both of those sources. It is
available inside the TradingView Desktop chart page as
`window.getQuoteSessionInstance()`. It supports temporary subscriptions to
symbol fields. This plan treats it as an evidence candidate only until
postmarket and premarket behavior are observed.

## Plan of Work

Create `crates/cli/tests/live_quote_session_extended_hours.rs`. The test must
be `#[ignore]` and gated by `TV_LIVE_QUOTE_SESSION_SMOKE=1`. It should use the
test-built `tv` binary from `CARGO_BIN_EXE_tv`, discover or accept a chart
target, run scanner-backed `tv quote <SYMBOL> --source scanner`, then use
`tv ui eval` with `TV_ALLOW_UNSAFE_UI_EVAL=1` to start and later read a
temporary Desktop quote-session probe.

The probe should request only selected fields needed for evidence:
`market-status`, `last_price`, `regular_close`, `prev_close_price`,
`premarket_open`, `premarket_high`, `premarket_low`, `premarket_close`,
`premarket_volume`, `postmarket_open`, `postmarket_high`, `postmarket_low`,
`postmarket_close`, `postmarket_volume`, `session-premarket`,
`session-postmarket`, `session-regular`, `update_mode`, `delay_seconds`,
`lp_time`, and `rt-update-time`. It must unsubscribe after a short bounded
window and restore the original quote fields when possible.

Update durable docs so agents read scanner extended-hours, chart main-series
quote, and Desktop quote-session fields as separate sources. Do not tell agents
that Desktop quote-session pre/post fields are stable extended-hours evidence
until live postmarket or premarket evidence proves it.

## Concrete Steps

From the repository root, run the normal compile-only check for the ignored
test:

    cargo test -p tradingview-cli --test live_quote_session_extended_hours

To collect evidence after the market close, run:

    TV_LIVE_QUOTE_SESSION_SMOKE=1 TV_LIVE_QUOTE_SESSION_EXPECT_PHASE=postmarket cargo test -p tradingview-cli --test live_quote_session_extended_hours -- --ignored --nocapture

To collect evidence during premarket, run:

    TV_LIVE_QUOTE_SESSION_SMOKE=1 TV_LIVE_QUOTE_SESSION_EXPECT_PHASE=premarket cargo test -p tradingview-cli --test live_quote_session_extended_hours -- --ignored --nocapture

If multiple chart targets are open, first choose the intended chart target and
pass it through the environment:

    TV_LIVE_QUOTE_SESSION_TARGET_ID=<ID> TV_LIVE_QUOTE_SESSION_SMOKE=1 TV_LIVE_QUOTE_SESSION_EXPECT_PHASE=postmarket cargo test -p tradingview-cli --test live_quote_session_extended_hours -- --ignored --nocapture

Do not paste live target ids or raw live payloads into tracked docs. Record
only compact summaries such as scanner last/update mode, scanner pre/post
close, quote-session phase, quote-session last, quote-session pre/post close,
and update mode.

## Validation and Acceptance

Run the following validation before committing:

    cargo test -p tradingview-cli --test live_quote_session_extended_hours
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Acceptance is met when the ignored test compiles in normal test runs, the
opt-in command can produce public-safe scanner and quote-session summaries in a
live environment, and docs / skills no longer imply that scanner
extended-hours, chart main-series quote, and Desktop quote-session fields are
the same source.

## Idempotence and Recovery

The test is safe to rerun. It creates a temporary quote-session subscription,
waits a bounded time, unsubscribes, and restores prior quote-session fields
when possible. If the test fails after starting the probe, rerunning it should
create a new subscription name. If multiple chart targets are open, choose one
explicitly with `TV_LIVE_QUOTE_SESSION_TARGET_ID`.

## Artifacts and Notes

Expected public-safe live output should look like this shape:

    ok scanner=last=<value> update_mode=<mode> delay_seconds=<n> premarket_close=<value-or-null> postmarket_close=<value-or-null> quote_session=done=true update_count=<n> updates=<symbol>:phase=<phase> last=<value> pre=<value-or-null> post=<value-or-null> mode=<mode> elapsed_ms=<n>

This output intentionally avoids raw JSON payloads, target ids, account-local
metadata, cookies, tokens, authorization values, and machine-specific paths.

## Interfaces and Dependencies

No new user-facing CLI command, option, dependency, or source fallback is
introduced. The only new Rust interface is an ignored integration test file.
The test depends on existing `tv ui eval` and explicitly sets
`TV_ALLOW_UNSAFE_UI_EVAL=1` only for the child commands that inspect the
TradingView Desktop page.

## Open Questions

- During postmarket, does `market-status.phase` become `postmarket` and do
  `postmarket_*` fields behave like visible after-hours values?
- During premarket, does `market-status.phase` become `premarket` and do
  `premarket_*` fields behave like visible premarket values?
- If values are useful only in the matching session phase, should a later
  payload expose them as a separate Desktop quote-session source rather than
  as chart main-series quote metadata?
