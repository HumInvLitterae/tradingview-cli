# Desktop after-hours widget store evidence

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and prepares the next `v0.13.0` evidence slice.

## Purpose / Big Picture

TradingView Desktop can show an after-hours price in the right-side detail
panel that differs from scanner REST, selected chart main-series quote,
Desktop quote-session selected fields, and the bounded Network/WebSocket
capture. The prior source-discovery slices narrowed the visible RKLB value to
the right-side detail widget status/price node, then showed that a short CDP
Network/WebSocket capture did not expose the visible price token. This slice
looks inside the matched widget's React fiber chain and nearby in-page state
to see whether the value can be tied to a less brittle in-page store or
component prop source.

After this change, a maintainer can run an ignored Rust integration test
during postmarket and get a public-safe summary of which React components,
props, or state paths near the right-side detail widget contain the visible
after-hours price, symbol, or after-hours labels. The test does not add a
public command, does not modify `tv quote --source chart`, and does not write
raw props, raw state, target ids, account-local metadata, or raw live payloads
into tracked docs.

## Progress

- [x] (2026-05-08T22:18Z) Archived the completed Network/WebSocket source
  evidence plan.
- [x] (2026-05-08T22:18Z) Created this widget-store source discovery ExecPlan
  and made it the current plan.
- [x] (2026-05-08T21:20Z) Added an ignored Rust integration test that summarizes React fiber,
  component, prop, and state candidates around the matched right-panel price
  node.
- [x] (2026-05-08T21:20Z) Ran compile-only validation for the ignored test.
- [x] (2026-05-08T21:20Z) Ran the opt-in RKLB smoke if the visible after-hours panel value is still
  present.
- [x] (2026-05-08T21:20Z) Updated docs, runtime skills, and changelog with
  the public-safe result.
- [x] (2026-05-08T21:20Z) Ran focused tests, full Rust baseline, docs
  validation, packaging script syntax check, and skill validation.
- [ ] Commit the slice.

## Surprises & Discoveries

- Observation: Screenshot-backed rerun showed the right-side detail panel with
  a regular price and a separate after-market price. The panel-source smoke
  saw the values separately, with the after-market visible value changing
  during the run.
  Evidence: local screenshot and opt-in panel-source smoke showed RKLB regular
  price around the chart/quote-session value and a separate visible
  after-market value near the same time window.

- Observation: The widget-store smoke can find the right-side detail widget
  React chain and compact props. Those props include regular quote-like values
  such as `data.quotes.last_price`, and session readback such as
  `data.quotes.current_session=post_market`, but the visible after-market
  price token did not appear in the scoped props/state hits.
  Evidence: Opt-in widget-store smoke for RKLB accepted the `post-market`
  phase, saw the expected visible after-market price in the right-side text,
  reported `fiber=true`, and returned compact prop hits for quote metadata and
  regular quote values but not for the after-market visible price.

## Decision Log

- Decision: Inspect only the React fiber chain rooted at the matched visible
  price node and its ancestors.
  Rationale: broad global object scanning risks collecting unrelated
  account-local state. The matched node is already the best confirmed source,
  so a scoped fiber-chain probe gives useful evidence with less privacy and
  brittleness risk.
  Date/Author: 2026-05-08 / Codex.

- Decision: Return primitive path summaries only, never raw props or raw state.
  Rationale: React props and state may contain large internal structures or
  account-scoped values. The source-discovery question only needs to know
  whether compact primitive paths contain the visible price, symbol, currency,
  or after-hours labels.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat "matched visible node but no React props/fiber candidate" as
  a successful evidence result, not a test failure.
  Rationale: the research question includes negative evidence. If the visible
  right-panel value is accessible only as rendered DOM text in the scoped
  probe, that is useful source-boundary evidence and should be reported
  without forcing a false backing-store conclusion.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

The opt-in smoke is in place and can inspect the right-side detail widget
without returning raw DOM, raw props, raw state, target ids, or account-local
metadata. The screenshot-backed RKLB rerun confirmed a separate visible
after-market value in the right-side panel, but the scoped React prop/state
probe found regular quote-like values rather than the visible after-market
price token. For v0.13, the lowest public-safe identified source for the
after-market value remains visible right-panel DOM evidence; a stable in-page
store or communication source is still unconfirmed.

Validation passed with focused ignored-smoke compile tests, full workspace
formatting, clippy, tests, metadata, docs diff check, release packaging script
syntax check, and runtime skill validators.

## Context and Orientation

The previous active plan, now archived as
`docs/plans/archives/tradingview-cli-desktop-after-hours-network-source-evidence.md`,
added a CDP Network/WebSocket smoke. That smoke observed RKLB-related
WebSocket traffic while the right panel still showed the expected after-hours
price, but it did not observe a communication candidate containing the
visible price token.

The earlier visible-panel plan, archived as
`docs/plans/archives/tradingview-cli-desktop-after-hours-panel-source-evidence.md`,
added `crates/cli/tests/live_after_hours_panel_source.rs`. That test found
the visible price in the right-side detail widget's status/price nodes, with
React metadata present on the matched node. This slice builds on that by
walking the React fiber chain. A React fiber is an internal object React uses
to connect a DOM node to the component instance, props, and state that
rendered it. This is an internal implementation detail, so evidence from it
must remain research-only until a separate stable public contract is designed.

## Plan of Work

Create `crates/cli/tests/live_after_hours_widget_store_source.rs`. The test
must be `#[ignore]` and gated by
`TV_LIVE_AFTER_HOURS_WIDGET_STORE_SMOKE=1`. It should accept
`TV_LIVE_AFTER_HOURS_TARGET_ID`, `TV_LIVE_AFTER_HOURS_SYMBOL`,
`TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE`, and
`TV_LIVE_AFTER_HOURS_EXPECT_PHASE`. It should use the test-built `tv` binary
and existing unsafe gated `tv ui eval` path to run a read-only JavaScript
probe inside the selected TradingView Desktop chart page.

The JavaScript probe should find the same right-side visible price node used
by the panel-source smoke. From that node and nearby ancestors, it should find
React fiber keys, walk the fiber return chain, collect component names, and
search `memoizedProps`, `memoizedState`, and small selected state-like fields
for primitive values that contain the expected visible price, requested
symbol, currency, or after-hours labels. The output must be compact:
component names, fiber tags, hit counts, and short path/value pairs. It must
not return raw props, raw state objects, raw DOM, target ids, cookies,
authorization values, or account-local metadata.

Update `docs/plans/README.md` and `docs/v0.13-roadmap.md` so this plan is the
current slice. Update stable docs and runtime skills only after evidence is
available, and keep the message conservative: widget-store evidence is source
discovery, not public payload support.

## Concrete Steps

From the repository root, create and compile the ignored test:

    cargo test -p tradingview-cli --test live_after_hours_widget_store_source

Run the opt-in smoke during postmarket when the right panel shows a distinct
after-hours value:

    TV_LIVE_AFTER_HOURS_WIDGET_STORE_SMOKE=1 TV_LIVE_AFTER_HOURS_EXPECT_PHASE=postmarket TV_LIVE_AFTER_HOURS_SYMBOL=RKLB TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE=110.17 cargo test -p tradingview-cli --test live_after_hours_widget_store_source -- --ignored --nocapture

If multiple chart targets are open, pass the intended target through the
environment. Do not paste the actual target id into tracked docs:

    TV_LIVE_AFTER_HOURS_TARGET_ID=<ID> TV_LIVE_AFTER_HOURS_WIDGET_STORE_SMOKE=1 TV_LIVE_AFTER_HOURS_EXPECT_PHASE=postmarket TV_LIVE_AFTER_HOURS_SYMBOL=RKLB TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE=110.17 cargo test -p tradingview-cli --test live_after_hours_widget_store_source -- --ignored --nocapture

## Validation and Acceptance

Run the following validation before committing:

    cargo test -p tradingview-cli --test live_after_hours_widget_store_source
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

Acceptance is met when the ignored widget-store smoke compiles in normal test
runs, the opt-in command can produce a public-safe React/fiber candidate
summary, and docs clearly state whether the visible after-hours value was
found in component props/state or remains DOM-only visible evidence.

## Idempotence and Recovery

The smoke is read-only. It runs a bounded `tv ui eval` expression against one
chart target and exits. It does not mutate the chart, subscribe to account
mutations, or write files. It is safe to rerun. If multiple chart targets are
open, pass the intended target explicitly through the environment.

## Artifacts and Notes

Expected public-safe output should look like this shape when a scoped React
candidate is found:

    ok visible=matched=true after_price=<value> react=fiber_found=true components=<names> hits=props:<path>=<value>|state:<path>=<value>

When the visible node is found but no scoped React candidate is exposed, the
smoke should still return a useful summary such as:

    ok symbol=<symbol> phase=post-market matched=true fiber=false hit_count=0 hits=<none>

Raw React props, raw state objects, raw DOM, target ids, cookies,
authorization values, account-local metadata, and raw live payloads must not
appear in terminal summaries committed to docs.

## Interfaces and Dependencies

No new user-facing CLI command, option, dependency, or source fallback is
introduced. The new Rust test uses the existing `tv ui eval` unsafe gate only
inside an ignored live smoke.

## Open Questions

- Does the React fiber chain expose a stable component or prop/state path that
  contains the visible after-hours price?
- If a path exists, does it look like a rendered UI prop only, or does it point
  toward a reusable in-page store behind the detail widget?
- If no path exists beyond the DOM text node, should the next design prefer a
  visible-panel evidence read or stop source discovery for v0.13?
