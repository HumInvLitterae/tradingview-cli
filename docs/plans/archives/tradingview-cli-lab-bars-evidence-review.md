# Lab bars evidence review and v0.5 data-lane boundary

This ExecPlan is a living document. Keep `Progress`, `Discoveries`, `Decisions`,
and `Validation` updated as work proceeds.

## Purpose

Review the first lab-gated `tv bars <SYMBOL>` prototype after implementation,
record bounded evidence, and decide whether the browserless historical-bars
lane remains a lab experiment, moves toward a reusable crate boundary, or should
pause while other `v0.5.0` work proceeds.

This slice does not add a new command and does not change the existing `tv
ohlcv` contract. It documents evidence around the experimental WebSocket bars
path and keeps the safety boundary durable before any follow-up work.

## Progress

- [x] Archived the completed lab-bars prototype plan.
- [x] Captured bounded live evidence for daily and hourly browserless bars.
- [x] Confirmed validation failures happen before network access for missing
      gate, bare symbols, and unsupported timeframe input.
- [x] Updated durable docs with the current lab boundary.
- [x] Ran validation and committed the combined prototype plus evidence review
      slice.

## Discoveries

- `TV_EXPERIMENTAL_BARS=1 tv bars NASDAQ:AAPL --timeframe 1D --count 5`
  returned five completed daily bars through the experimental WebSocket path.
- `TV_EXPERIMENTAL_BARS=1 tv bars NYSE:IONQ --timeframe 1D --count 5`
  returned five completed daily bars through the same path.
- `TV_EXPERIMENTAL_BARS=1 tv bars NASDAQ:AAPL --timeframe 60 --count 10`
  returned ten completed hourly bars.
- Without `TV_EXPERIMENTAL_BARS=1`, `tv bars` fails with a validation error
  before network access.
- Bare symbols remain rejected before network access because the prototype
  intentionally requires exchange-qualified input.
- Unsupported timeframe input is rejected before network access.
- `tv ohlcv --count 1` still reads the selected Desktop chart and remains
  separate from browserless `tv bars`.

These observations are intentionally summarized. Raw WebSocket frames, session
ids, target ids, and account-local values must not be written to tracked docs.

## Decisions

- Keep `tv bars` lab-gated for now. The prototype has enough evidence to be
  useful for bounded experiments, but it still depends on an undocumented
  TradingView WebSocket chart-session protocol.
- Keep the implementation in the CLI package rather than moving it into
  `tradingview-market`. The market crate remains the stable-ish Desktop-free
  scanner REST read boundary.
- Keep `tv ohlcv` as selected-chart / CDP bars read. Do not make `ohlcv`
  symbol-targeted or browserless unless equivalence is proven in a later plan.
- Future follow-up should be driven by downstream need: protocol stability,
  malformed-frame behavior, retry/timeout tuning, and whether a typed lab API
  boundary is worth introducing.

## Work Items

- Archive `docs/plans/tradingview-cli-lab-bars-prototype.md`.
- Create this evidence review plan and make it the current plan in
  `docs/plans/README.md`.
- Update `docs/v0.5-roadmap.md` so the lab-bars prototype is no longer the
  next implementation slice; the next step is evidence review and boundary
  stabilization.
- Update `docs/internal-tradingview-apis.md` and
  `docs/operation-adapter-boundaries.md` with the bounded evidence and the
  current lab-only status.
- Update `CHANGELOG.md` with the evidence-review/boundary note.
- Re-run prototype and docs validation.
- Commit all related changes together after validation.

## Validation

Run before committing:

- `TV_EXPERIMENTAL_BARS=1 target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5`
- `TV_EXPERIMENTAL_BARS=1 target/debug/tv bars NYSE:IONQ --timeframe 1D --count 5`
- `TV_EXPERIMENTAL_BARS=1 target/debug/tv bars NASDAQ:AAPL --timeframe 60 --count 10`
- `target/debug/tv bars NASDAQ:AAPL --count 5`
- `TV_EXPERIMENTAL_BARS=1 target/debug/tv bars AAPL --count 5`
- `TV_EXPERIMENTAL_BARS=1 target/debug/tv bars NASDAQ:AAPL --timeframe banana --count 5`
- `target/debug/tv ohlcv --count 1`
- `cargo test -p tradingview-cli market::bars -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract bars -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true`

## Rollback

If the evidence review uncovers instability severe enough to pause the
prototype, remove the `bars` command surface and its docs in the same branch,
then record the deferred reason here and in `docs/v0.5-roadmap.md`. If only the
evidence is inconclusive, keep the prototype lab-gated and document the
remaining uncertainty.
