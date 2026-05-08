# Desktop visible after-hours price source discovery

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and prepares the next `v0.13.0` evidence slice.

## Purpose / Big Picture

Agents need to distinguish scanner extended-hours values, selected chart
main-series values, Desktop quote-session fields, and visible TradingView
Desktop side-panel values. RKLB showed a useful gap: scanner REST returned a
delayed postmarket value, chart main-series and the existing quote-session
fields returned the regular close-like value, while the visible Desktop detail
panel displayed a different after-market price. This work adds an opt-in live
smoke that collects those four sources in one public-safe summary without
changing any public quote payload.

After this change, a maintainer can run one ignored Rust integration test
during postmarket and see whether the visible right-panel after-market value
matches scanner REST, chart main-series, quote-session selected fields, or
none of them. The test does not add a new user-facing command, does not modify
`tv quote --source chart`, and does not make live evidence part of normal CI.

## Progress

- [x] (2026-05-08T20:42Z) Added an ignored Rust integration test for Desktop
  visible after-hours panel source evidence.
- [x] (2026-05-08T20:42Z) Ran the opt-in smoke during postmarket with RKLB and
  confirmed that scanner REST, chart main-series, quote-session selected
  fields, and the visible side panel can be summarized in one run.
- [x] (2026-05-08T20:42Z) Updated current roadmap, plan index, source taxonomy
  docs, workflow docs, internal API notes, runtime skills, and changelog.

## Surprises & Discoveries

- Observation: the visible right-side detail panel can expose an after-market
  price that is not returned by chart main-series quote or the current
  quote-session selected field set.
  Evidence: the RKLB postmarket smoke reported scanner REST postmarket close
  near the delayed extended-hours value, chart main-series last and
  quote-session selected fields at `105.47`, and visible panel after-market
  price `110.17`.

- Observation: the existing quote-session field set still confirms
  `post-market` phase but does not expose the visible after-market price for
  RKLB.
  Evidence: the same smoke reported `phase=post-market` for RKLB quote-session
  updates, while `last_price`, `premarket_close`, and `postmarket_close`
  stayed at `105.47`.

- Observation: scanner REST is useful for delayed extended-hours values but
  cannot be used as an equality oracle for the Desktop visible panel.
  Evidence: in the RKLB run, scanner REST reported
  `update_mode=delayed_streaming_900`, `delay_seconds=900`, and a postmarket
  close below the visible after-market price.

## Decision Log

- Decision: Add a separate after-hours panel source smoke instead of extending
  `tv quote --source chart`.
  Rationale: chart-source quote is already defined as selected chart
  main-series bars. The visible right-panel value is a separate Desktop-backed
  visible UI source and should not be silently mixed into chart-source payloads.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat DOM-derived visible panel text as research evidence, not a
  stable public payload contract.
  Rationale: the DOM structure and labels can change, and the test is intended
  to locate the source of the visible value before designing any public
  support.
  Date/Author: 2026-05-08 / Codex.

- Decision: Keep scanner equality out of the success criteria.
  Rationale: scanner REST can be delayed while the visible panel can be based
  on a Desktop-backed streaming or UI-specific source.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

The new smoke proves that the visible Desktop after-market price can be
captured as compact DOM-derived evidence during postmarket. It also proves
that the current quote-session selected field set is insufficient for the RKLB
visible after-market price. The next design decision is whether to expose a
separate visible-panel after-hours read, continue searching for the underlying
TradingView store or quote-session field, or keep this as diagnostic tooling
only. Premarket behavior remains open.

## Context and Orientation

The existing chart-source quote implementation lives under
`crates/cli/src/ops/market/quote.rs` and reads selected chart main-series bars.
It already reports `session_boundary` metadata saying that scanner-style
extended-hours fields are not provided by that source.

The scanner REST quote implementation lives in `crates/market/src/quote.rs`.
It returns Desktop-free quote data and reshapes scanner fields into
`extended_hours.premarket` and `extended_hours.postmarket`.

The Desktop quote-session live smoke lives in
`crates/cli/tests/live_quote_session_extended_hours.rs`. It uses the unsafe
gated `tv ui eval` command only inside an ignored test to inspect
`window.getQuoteSessionInstance()` and prints compact summaries instead of raw
payloads.

The new file `crates/cli/tests/live_after_hours_panel_source.rs` follows the
same safety pattern, but it also reads visible right-panel text from the
Desktop DOM and extracts only compact fields such as whether the symbol was
seen, whether an after-market label was seen, and the visible after-market
price candidate.

## Plan of Work

Create `crates/cli/tests/live_after_hours_panel_source.rs`. The test must be
`#[ignore]` and gated by `TV_LIVE_AFTER_HOURS_PANEL_SMOKE=1`. It should use
the test-built `tv` binary from `CARGO_BIN_EXE_tv`, accept or resolve one chart
target, collect scanner REST quote, collect selected chart main-series quote,
start a temporary Desktop quote-session probe, then read a compact visible
right-panel summary with `tv ui eval`.

The visible panel probe must not return raw DOM. It should collect visible
text from the right side of the TradingView page, filter it to compact
symbol/price/session snippets, and return only selected fields:
`symbol_seen`, `after_market_label_seen`, `visible_after_market_price`,
`expected_visible_price_seen`, a small list of USD candidates, a small list of
numbers near the after-market label, and a short snippet list.

Update durable docs so agents understand that Desktop visible panel
after-hours values can differ from scanner REST, chart main-series quote, and
the current Desktop quote-session selected fields.

## Concrete Steps

From the repository root, run the normal compile-only check for the ignored
test:

    cargo test -p tradingview-cli --test live_after_hours_panel_source

To collect RKLB postmarket evidence, run the opt-in smoke during a real
postmarket window:

    TV_LIVE_AFTER_HOURS_PANEL_SMOKE=1 TV_LIVE_AFTER_HOURS_EXPECT_PHASE=postmarket TV_LIVE_AFTER_HOURS_SYMBOL=RKLB TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL=NASDAQ:RKLB cargo test -p tradingview-cli --test live_after_hours_panel_source -- --ignored --nocapture

If multiple chart targets are open, choose the intended chart target and pass
it through the environment:

    TV_LIVE_AFTER_HOURS_TARGET_ID=<ID> TV_LIVE_AFTER_HOURS_PANEL_SMOKE=1 TV_LIVE_AFTER_HOURS_EXPECT_PHASE=postmarket TV_LIVE_AFTER_HOURS_SYMBOL=RKLB TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL=NASDAQ:RKLB cargo test -p tradingview-cli --test live_after_hours_panel_source -- --ignored --nocapture

If the user has a visible price to verify, provide it through
`TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE`. The test should then fail if that
exact compact visible price is not found.

Do not paste live target ids, raw DOM, raw live payloads, account-local
metadata, cookies, tokens, authorization values, or machine-specific paths into
tracked docs.

## Validation and Acceptance

Run the following validation before committing:

    cargo test -p tradingview-cli --test live_after_hours_panel_source
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
opt-in command can produce a public-safe source-by-source summary, and docs /
skills no longer imply that chart main-series quote or current quote-session
selected fields are the visible after-market price.

## Idempotence and Recovery

The test is safe to rerun. It creates a temporary quote-session subscription,
waits a bounded time, unsubscribes, and restores prior quote-session fields
when possible. The visible panel probe is read-only. If multiple chart targets
are open, choose one explicitly with `TV_LIVE_AFTER_HOURS_TARGET_ID`.

## Artifacts and Notes

Expected public-safe live output should look like this shape:

    ok scanner=last=<value> close=<value> update_mode=<mode> delay_seconds=<n> premarket_close=<value> postmarket_close=<value> chart=symbol=<symbol> last=<value> close=<value> time=<epoch> session_status=<status> quote_session=done=true update_count=<n> updates=<symbol>:phase=<phase> last=<value> pre=<value> post=<value> mode=<mode> visible_panel=symbol_seen=true after_label=true after_price=<value> expected_price=<value-or-null> expected_seen=<bool-or-null> ...

In the RKLB postmarket run, the compact summary showed scanner delayed
postmarket data, chart main-series and quote-session selected fields at
`105.47`, and visible panel after-market price `110.17`.

## Interfaces and Dependencies

No new user-facing CLI command, option, dependency, or source fallback is
introduced. The only new Rust interface is an ignored integration test file.
The test depends on existing `tv ui eval` and explicitly sets
`TV_ALLOW_UNSAFE_UI_EVAL=1` only for child commands that inspect the
TradingView Desktop page.

## Open Questions

- Which lower-level TradingView store or quote-session field feeds the visible
  after-market side-panel value?
- Should a later public payload expose DOM-derived visible after-hours values,
  or should the implementation continue searching for a less brittle source?
- During premarket, does the visible panel show a similarly distinct
  premarket price that can be read by the same probe?
