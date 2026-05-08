# Desktop after-hours network source evidence

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and prepares the next `v0.13.0` evidence slice.

## Purpose / Big Picture

TradingView Desktop can show an after-hours price in the right-side detail
panel that differs from scanner REST, selected chart main-series quote, and
the current Desktop quote-session selected fields. The prior slice narrowed
that visible value to a right-panel detail widget status/price node. This
slice goes one level lower by observing the page's CDP Network and WebSocket
events for a short opt-in window, looking for communication frames or response
summaries that correlate with the visible after-hours value.

After this change, a maintainer can run an ignored Rust integration test during
postmarket and see whether any observed Network or WebSocket candidate mentions
the symbol, the expected visible price, after-hours labels, or quote/session
tokens. The test does not add a public command, does not modify `tv quote
--source chart`, and does not write raw frames, target ids, cookies, account
metadata, or raw live payloads into tracked docs.

## Progress

- [x] (2026-05-08T21:35Z) Archived the completed visible after-hours panel
  source evidence plan.
- [x] (2026-05-08T21:35Z) Created this Network/WebSocket source discovery
  ExecPlan and made it the current plan.
- [x] (2026-05-08T21:46Z) Added an ignored Rust integration test that captures public-safe CDP
  Network/WebSocket summaries for the selected chart target.
- [x] (2026-05-08T21:47Z) Ran compile-only validation for the ignored test.
- [x] (2026-05-08T21:49Z) Ran the opt-in RKLB postmarket smoke while the
  visible panel still showed the expected after-hours price.
- [x] (2026-05-08T21:50Z) Updated docs, runtime skills, and changelog with the
  public-safe result.
- [x] (2026-05-08T22:02Z) Ran focused tests, full workspace tests, clippy,
  metadata, diff check, packaging script syntax check, skill validation, and
  hygiene grep.

## Surprises & Discoveries

- Observation: CDP Network/WebSocket observation sees quote-related traffic
  for RKLB, but the bounded run did not see the visible after-hours price in
  observed frames.
  Evidence: the RKLB opt-in smoke observed WebSocket frames with the requested
  symbol and qualified symbol tokens while the visible panel still showed the
  expected after-hours price. None of the candidate frame summaries contained
  the expected visible price token.

- Observation: the existing CDP Network event stream may not provide URL
  metadata for WebSockets that were already open before `Network.enable`.
  Evidence: the public-safe summary reported received WebSocket frames under
  an unknown WebSocket source because no matching `Network.webSocketCreated`
  event was observed during the bounded capture window.

## Decision Log

- Decision: Use CDP Network/WebSocket event observation as a research smoke,
  not a public CLI command.
  Rationale: The goal is source discovery for a visible TradingView Desktop UI
  value. Exposing a public payload before identifying a stable backing source
  would repeat the source-mixing risk this lane is trying to avoid.
  Date/Author: 2026-05-08 / Codex.

- Decision: Capture only compact candidate summaries, never raw frames or raw
  response bodies.
  Rationale: TradingView page traffic may contain session-scoped or
  account-local data. The useful evidence is whether a frame or response
  category appears to contain the symbol, expected visible price, and
  after-hours context, not the full payload.
  Date/Author: 2026-05-08 / Codex.

- Decision: Do not promote Network/WebSocket traffic to public payload support
  from this evidence alone.
  Rationale: the bounded capture did not identify a frame containing the
  visible after-hours price. It only showed that symbol-related quote traffic
  continues while the visible panel value is present.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

This slice adds an opt-in CDP Network/WebSocket smoke and records a
public-safe RKLB postmarket run. The run confirms that symbol-related
WebSocket traffic is observable, but it does not identify the visible
after-hours price in the communication events captured during the bounded
window. The lowest source identified for the visible value therefore remains
the right-panel detail widget status/price node from the previous slice.
Further source discovery would need either a longer capture, a forced
right-panel refresh, or in-page store inspection behind the detail widget.
Validation passed with focused tests, full workspace tests, formatting,
clippy, metadata, diff check, packaging script syntax check, and runtime skill
validation. Hygiene grep reported existing policy and archive hits plus this
plan's safety wording; no new raw live payload, target id, credential, or
account-local metadata was added.

## Context and Orientation

The previous active plan, now archived as
`docs/plans/archives/tradingview-cli-desktop-after-hours-panel-source-evidence.md`,
added `crates/cli/tests/live_after_hours_panel_source.rs`. That test compares
four sources in one public-safe run: scanner REST, chart main-series quote,
Desktop quote-session selected fields, and compact right-panel visible text.
For RKLB during postmarket, it found that the visible right-panel value did not
match scanner REST, chart main-series quote, or the selected quote-session
fields. A lower-level rerun located the visible value inside the right-side
detail widget's status/price nodes, with React metadata present.

This slice adds a second ignored live smoke. CDP means Chrome DevTools
Protocol, the local debugging protocol used by `tv` to talk to TradingView
Desktop. The CDP Network domain can emit events for HTTP responses and
WebSocket frames. A WebSocket frame is one message on a persistent WebSocket
connection. This test observes those events for a bounded period and emits
only compact summaries, such as counts, URL host/path hints, and whether
candidate text contains the symbol, expected visible price, or after-hours
tokens.

## Plan of Work

Create `crates/cli/tests/live_after_hours_network_source.rs`. The test must be
`#[ignore]` and gated by `TV_LIVE_AFTER_HOURS_NETWORK_SMOKE=1`. It should use
the test-built `tv` binary for existing public reads where useful, but it
should connect directly to the selected chart target's CDP WebSocket to enable
the Network domain and collect events. It should accept
`TV_LIVE_AFTER_HOURS_TARGET_ID`, `TV_LIVE_AFTER_HOURS_SYMBOL`,
`TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL`,
`TV_LIVE_AFTER_HOURS_EXPECT_PHASE`,
`TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE`, and
`TV_LIVE_AFTER_HOURS_NETWORK_DURATION_MS`.

The test should send `Network.enable` through CDP, then observe events such as
`Network.webSocketFrameReceived`, `Network.webSocketFrameSent`,
`Network.responseReceived`, and `Network.loadingFinished` for a bounded
duration. For each observed event, it must build a compact candidate summary.
For WebSocket frames, it may inspect the payload string in memory, but it must
not print or store the raw payload. It should count whether the payload
contains the requested symbol, the qualified symbol, the expected visible
price, and after-hours words such as `postmarket`, `post-market`,
`aftermarket`, `after-market`, or Japanese after-market labels. For HTTP
responses, it should summarize URL host/path, resource type, and status
without printing query strings or bodies.

The test should also read the visible panel summary through the existing
`tv ui eval` pattern or a compact local JavaScript probe, so the Network
capture can be interpreted next to the visible price. This read must remain
public-safe and should avoid raw DOM output.

Update `docs/plans/README.md` and `docs/v0.13-roadmap.md` so this plan is the
current slice. Update docs and skills only after evidence exists or after the
compile-only smoke is in place, making clear that communication capture is
source discovery, not public payload support.

## Concrete Steps

From the repository root, create the ignored test:

    cargo test -p tradingview-cli --test live_after_hours_network_source

Run the opt-in smoke during postmarket when the right panel shows a distinct
after-hours value:

    TV_LIVE_AFTER_HOURS_NETWORK_SMOKE=1 TV_LIVE_AFTER_HOURS_EXPECT_PHASE=postmarket TV_LIVE_AFTER_HOURS_SYMBOL=RKLB TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL=NASDAQ:RKLB TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE=110.17 cargo test -p tradingview-cli --test live_after_hours_network_source -- --ignored --nocapture

If multiple chart targets are open, pass the intended target through the
environment. Do not paste the actual target id into tracked docs:

    TV_LIVE_AFTER_HOURS_TARGET_ID=<ID> TV_LIVE_AFTER_HOURS_NETWORK_SMOKE=1 TV_LIVE_AFTER_HOURS_EXPECT_PHASE=postmarket TV_LIVE_AFTER_HOURS_SYMBOL=RKLB TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL=NASDAQ:RKLB TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE=110.17 cargo test -p tradingview-cli --test live_after_hours_network_source -- --ignored --nocapture

## Validation and Acceptance

Run the following validation before committing:

    cargo test -p tradingview-cli --test live_after_hours_network_source
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

Acceptance is met when the ignored Network/WebSocket smoke compiles in normal
test runs, the opt-in command can produce a public-safe candidate summary, and
docs clearly state whether a lower-level communication candidate was found.
If the opt-in smoke cannot run because the postmarket window is over, record
that timing result and keep the test ready for the next phase-specific run.

The RKLB opt-in smoke did run while the expected visible value was still
present. It produced a public-safe summary with WebSocket event counts and
candidate token flags. It did not find a candidate containing the expected
visible price.

## Idempotence and Recovery

The smoke is read-only. It enables CDP Network observation for one chart target
and then exits. It does not mutate the chart, subscribe to account mutations,
or write files. It is safe to rerun. If the CDP connection drops, rerun the
test after confirming `tv readiness`. If multiple chart targets are open, pass
the intended target explicitly through the environment.

## Artifacts and Notes

Expected public-safe output should look like this shape:

    ok visible_panel=after_price=<value> expected_seen=<bool> network=duration_ms=<n> websocket_frames=<n> response_count=<n> candidates=ws:<host/path>:symbol=true expected_price=true after_token=true ...

In the RKLB postmarket run, scanner REST remained delayed, the visible panel
continued to show the expected after-hours price, and the Network/WebSocket
summary observed WebSocket frames containing symbol tokens but not the expected
visible price token. No raw frame or target identifier was recorded.

Raw WebSocket frames, raw HTTP response bodies, target ids, cookies,
authorization values, account-local metadata, and raw DOM must not appear in
terminal summaries committed to docs.

## Interfaces and Dependencies

No new user-facing CLI command, option, dependency, or source fallback is
introduced. The new Rust test may use existing workspace dependencies:
`reqwest` to read the local CDP target list, `tokio-tungstenite` to connect to
the selected target's CDP WebSocket, `serde_json` for CDP messages, and
`tokio` for the async test runtime.

## Open Questions

- Do Network/WebSocket events expose the right-panel after-hours value in a
  compact, source-identifiable form?
- If a candidate frame exists, is it tied to the visible detail widget source
  or only to an unrelated chart bid/ask or quote-session update?
- If no candidate frame appears during a short run, is that because the value
  was already cached, because the window ended, or because the useful source is
  an in-page store not visible through Network events?
