# Desktop after-hours WebSocket correlation evidence

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and covers the next `v0.13.0` source-discovery slice.

## Purpose / Big Picture

TradingView Desktop's right-side detail panel can show a postmarket price that
differs from scanner REST, selected chart main-series quote, and the current
Desktop quote-session selected fields. Prior source-discovery slices narrowed
the visible value to the right-side detail panel and then to a scoped React
component area, but did not identify a stable backing source.

The user observed that the panel updates like a realtime or push source and
also noted that repeated DOM investigation may leave the right-side detail
panel in an abnormal state after switching symbols. This slice therefore keeps
DOM interaction minimal and focuses on correlating compact visible price
samples with CDP WebSocket frame summaries. It does not add a public command,
option, payload field, or data source.

## Progress

- [x] (2026-05-08T21:35Z) Created this WebSocket correlation ExecPlan and
  made it the current plan.
- [x] (2026-05-08T21:35Z) Added an ignored live smoke that samples the visible
  right-side detail panel and CDP WebSocket frames without subscribing to the
  Desktop quote session.
- [x] (2026-05-08T21:50Z) Ran compile-only validation for the ignored smoke.
- [x] (2026-05-08T21:50Z) Ran the opt-in RKLB smoke while the visible panel was in a useful postmarket
  state.
- [x] (2026-05-08T22:10Z) Updated docs, skills, and changelog with
  public-safe evidence.
- [x] (2026-05-08T22:25Z) Ran validation for the smoke, affected live-smoke
  compile checks, skills, workspace baseline, and packaging script syntax.
- [x] (2026-05-09T22:45Z) Extended the smoke to parse public-safe `qsd`
  quote-data readback fields such as `rtc`, `rtc_time`, `rch`, and `rchp`
  from WebSocket frames.
- [x] (2026-05-09T23:00Z) Ran screenshot-backed RKLB postmarket evidence and
  confirmed visible after-market price samples matched RKLB `qsd.rtc`
  candidates during the same bounded capture.
- [x] (2026-05-09T23:20Z) Updated docs and skills, reran validation, and
  prepared the RTC evidence hardening for commit.

## Surprises & Discoveries

- Observation: Screenshot-backed postmarket RKLB run showed the right-side
  panel regular price and a separate visible after-market price. During the
  same bounded correlation smoke, the sampled visible after-market value moved
  through multiple nearby prices, and each sampled after-market price appeared
  as an exact numeric candidate in received WebSocket frames.
  Evidence: Opt-in WebSocket correlation smoke sampled visible after-market
  prices in one run and reported exact WebSocket candidate matches for each
  observed visible price, without raw frame output.

- Observation: The chart surface itself can still look confusing because the
  crosshair may show historical bar OHLC on the chart while the right-side
  detail panel shows the live symbol detail. Screenshot context is useful for
  avoiding this misread.
  Evidence: Before and after screenshots showed RKLB selected in the right-side
  detail panel while chart overlay text reflected the hovered bar.

- Observation: A Web TradingView HAR and a follow-up live run both point to
  `qsd` quote-data messages, not the previously probed pre/post close fields,
  as the likely backing stream for the visible right-side after-market value.
  In the live run, visible RKLB after-market samples changed during the
  capture window, and RKLB `qsd.rtc` candidates matched later visible samples.
  Evidence: The smoke recorded public-safe `rtc`, `rtc_time`, `rch`, and
  `rchp` summaries while the right-side panel remained visible and stable.

## Decision Log

- Decision: Do not use `window.getQuoteSessionInstance().subscribe()` in this
  slice.
  Rationale: quote-session probing changes a page subscription and field set
  even when restored afterward. The user observed right-side detail panel
  abnormalities after DOM/source investigation, so this slice should use a
  lower-risk path: CDP Network event listening plus minimal visible text
  sampling.
  Date/Author: 2026-05-08 / Codex.

- Decision: Sample only compact right-side detail text and numeric tokens.
  Rationale: broad DOM walks, raw DOM dumps, and raw React state are noisier
  and may trigger unnecessary layout work. The correlation question only needs
  visible regular price, visible after-market price, phase label, and
  public-safe numeric candidates.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat WebSocket matching as evidence, not as a public payload
  source.
  Rationale: even if a WebSocket frame contains the visible price or a nearby
  numeric token, that does not yet prove stable semantics or entitlement
  behavior. A separate design is required before exposing any value in
  `tv quote --source chart`.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat `qsd.rtc` as the strongest current candidate for the visible
  after-market value, but not as a shipped public field yet.
  Rationale: `rtc` matched visible after-market samples in a bounded live run
  and `rch`/`rchp` look like regular-close-relative change readbacks. However,
  the source is still an internal WebSocket quote-data stream and needs a
  separate contract before any public payload support.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

The smoke is implemented and produced positive correlation evidence in a
postmarket RKLB run: compact visible after-market samples changed during the
capture window, and received WebSocket frame summaries contained exact numeric
candidates matching those visible prices. This supports the hypothesis that
the right-side detail panel after-market value is push-fed or WebSocket-backed.
The next hardening pass identified `qsd.rtc` as the best current field-level
candidate: when the visible after-market value moved, RKLB quote-data
summaries later carried matching `rtc` values plus `rtc_time`, `rch`, and
`rchp` readbacks. This is still source-discovery evidence, not a public
payload contract, so payload support remains deferred.

## Context and Orientation

The current right-side panel source evidence is public-safe but incomplete:

- scanner REST can report delayed `extended_hours` values;
- chart main-series quote reports the selected chart last bar;
- Desktop quote-session selected fields show session and regular quote-like
  values but not necessarily the visible after-market price;
- the right-side detail panel can visibly show a separate after-market value;
- prior bounded Network/WebSocket exact-token capture did not find the visible
  price token;
- scoped widget-store inspection found regular quote-like props but not the
  visible after-market price token.

This slice tests a more realistic hypothesis: the visible value may arrive as
push data, but not as a directly searchable display string in a short capture.
The smoke therefore samples visible prices over time and records whether
captured WebSocket numeric candidates exactly match any visible after-market
price observed during the same run.

## Plan of Work

Create `crates/cli/tests/live_after_hours_ws_correlation.rs`. The test must be
`#[ignore]` and gated by
`TV_LIVE_AFTER_HOURS_WS_CORRELATION_SMOKE=1`. It should accept:

- `TV_LIVE_AFTER_HOURS_TARGET_ID`;
- `TV_LIVE_AFTER_HOURS_SYMBOL`, default `RKLB`;
- `TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL`, default `NASDAQ:RKLB`;
- `TV_LIVE_AFTER_HOURS_EXPECT_PHASE`, for example `postmarket`;
- `TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE`, optional;
- `TV_LIVE_AFTER_HOURS_WS_CORRELATION_DURATION_MS`, bounded;
- `TV_LIVE_AFTER_HOURS_WS_CORRELATION_SAMPLE_MS`, bounded.

The smoke should connect to the selected chart target through CDP, enable the
Network domain, and collect WebSocket frame events. During the same bounded
window, it should periodically run a minimal `Runtime.evaluate` that reads
only the right-side detail widget text and returns compact fields:

- visible regular price;
- visible after-market price;
- phase label;
- expected-price seen flag;
- numeric token list.

The WebSocket summary must not record raw frames. It may record URL category,
frame direction, byte size, symbol flags, after-token flags, and compact
numeric tokens. The final summary should say whether any WebSocket numeric
candidate exactly matched a visible after-market price observed during the
same run.

## Concrete Steps

Compile the ignored smoke:

    cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --nocapture

Optionally run it during postmarket with screenshot-backed visible context:

    target/debug/tv screenshot --target-id <ID> --region full --output target/live-evidence/rklb-after-hours-correlation.png
    TV_LIVE_AFTER_HOURS_TARGET_ID=<ID> TV_LIVE_AFTER_HOURS_WS_CORRELATION_SMOKE=1 TV_LIVE_AFTER_HOURS_EXPECT_PHASE=postmarket TV_LIVE_AFTER_HOURS_SYMBOL=RKLB TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL=NASDAQ:RKLB cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --ignored --nocapture

Do not paste the actual target id, raw screenshot, raw DOM, raw frames, or raw
payloads into tracked docs.

## Validation and Acceptance

Run the following validation before committing:

    cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --nocapture
    cargo test -p tradingview-cli --test live_after_hours_network_source -- --nocapture
    cargo test -p tradingview-cli --test live_after_hours_panel_source -- --nocapture
    cargo test -p tradingview-cli --test live_after_hours_widget_store_source -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Acceptance is met when the ignored smoke compiles in normal test runs, an
opt-in run can produce public-safe visible-sample and WebSocket-candidate
summaries, and docs clearly state whether correlation was found, absent, or
still inconclusive.

## Idempotence and Recovery

The smoke is read-only from the user's perspective. It enables CDP Network
events and samples visible text. It does not click, type, switch symbols,
subscribe to the page quote session, or mutate account state. If the panel
display becomes abnormal, stop source discovery, capture a screenshot for
local inspection only, and do not treat the run as source evidence.

## Artifacts and Notes

Expected public-safe output shape:

    ok visible_samples=<time:after-price-list> network=events=<n> ws_received=<n> visible_prices=<prices> exact_matches=<direction:price>

Raw WebSocket frames, raw DOM, target ids, cookies, authorization values,
account-local metadata, and raw live payloads must not appear in tracked docs.

## Interfaces and Dependencies

No new user-facing CLI command, option, dependency, or source fallback is
introduced. The test uses existing CDP target discovery and WebSocket support
already present in the test environment.

## Open Questions

- Do WebSocket numeric candidates exactly match visible after-market prices
  during the same run?
- If exact matches do not appear, do visible price changes correlate with
  nearby numeric candidates that may be scaled or rounded?
- Is the right-side detail panel abnormality reproducible from DOM sampling,
  from quote-session probing, or from unrelated TradingView UI state?
