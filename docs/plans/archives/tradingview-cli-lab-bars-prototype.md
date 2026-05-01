# Lab-gated Desktop-free bars prototype

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` updated while the
work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

This slice adds a bounded experimental `tv bars <SYMBOL>` command for
Desktop-free historical OHLCV bars. It intentionally does not replace
`tv ohlcv`, which remains the selected-chart/CDP bars command. The new command
uses an undocumented TradingView WebSocket chart-session path and therefore is
lab-gated behind `TV_EXPERIMENTAL_BARS=1`.

After this slice, agents can try browserless bars only when they explicitly
accept the experimental boundary. Stable market reads still prefer scanner REST
commands, and stable chart bars still use `tv ohlcv`.

## Progress

- [x] (2026-05-02) Created this ExecPlan as the first v0.5 Desktop-free data
  lab slice after readiness diagnostics.
- [x] (2026-05-02) Added `tv bars <SYMBOL> --timeframe <TIMEFRAME> --count
  <N>` with a `TV_EXPERIMENTAL_BARS=1` gate.
- [x] (2026-05-02) Implemented a bounded anonymous TradingView WebSocket chart
  session read in the CLI package, not in `tradingview-market`.
- [x] (2026-05-02) Added parser and validation tests plus CLI contract tests
  for the lab gate and invalid inputs.
- [x] (2026-05-02) Live-smoked `NASDAQ:AAPL` and `NYSE:IONQ` with count 5
  through the experimental path without TradingView Desktop/CDP.
- [x] (2026-05-02) Updated docs, skills, and changelog.
- [x] (2026-05-02) Ran focused and full validation.
- [ ] Commit the related changes.

## Surprises & Discoveries

- The existing CDP WebSocket dependency only needed local `ws://` support in
  practice. The experimental browserless path needs `wss://`, so the workspace
  `tokio-tungstenite` dependency needs rustls WebPKI roots enabled.

## Decision Log

- Decision: Add a separate `tv bars` command instead of changing `tv ohlcv`.
  Rationale: `tv ohlcv` is selected-chart/CDP state. Browserless historical
  bars use a different undocumented protocol and must not silently change the
  meaning or freshness boundary of the stable command.
  Date/Author: 2026-05-02 / Codex

- Decision: Keep the first implementation in `tradingview-cli`, not
  `tradingview-market`.
  Rationale: `tradingview-market` is the stable-ish Desktop-free scanner REST
  read crate. The WebSocket chart-session protocol is lab-only and should not
  become part of the reusable typed market API before more evidence exists.
  Date/Author: 2026-05-02 / Codex

- Decision: Require exchange-qualified symbols for the first prototype.
  Rationale: Avoiding bare-symbol resolution keeps the lab path bounded and
  prevents accidental exchange ambiguity in an experimental data source.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

Implemented the lab-gated path. `tv bars` succeeds for exchange-qualified
symbols when `TV_EXPERIMENTAL_BARS=1` is set, includes `experimental: true`,
`source: "experimental_tradingview_ws"`, bounded normalized bars, and
data-quality warnings. Without the gate, or with bare symbols or out-of-range
counts, it fails before network.

Validation passed with focused bars unit tests, bars CLI contract tests,
skill validation for the changed runtime skills, release packaging script
syntax check, `cargo fmt --check`, `cargo clippy --workspace --all-targets
--all-features -- -D warnings`, `cargo test --workspace`, `cargo metadata
--no-deps --format-version 1`, and `git diff --check`. Live smoke confirmed
the experimental path with public example symbols and confirmed `tv ohlcv
--count 1` still reads the selected chart/CDP bars path.

## Context and Orientation

`tv info`, `tv quote`, `tv quotes`, `scanner scan`, `scanner hotlist`, and
`scanner metainfo` already provide Desktop-free scanner REST reads. They do not
provide historical OHLCV bars. The earlier feasibility pass found only an
undocumented WebSocket chart-session path, similar to the fiale-plus
experimental PR, so this slice treats bars as a lab feature.

The command contract is:

```bash
TV_EXPERIMENTAL_BARS=1 tv bars NASDAQ:AAPL --timeframe 1D --count 5
```

The output is a normal JSON envelope whose data contains
`source: "experimental_tradingview_ws"`, `experimental: true`, the requested
symbol/timeframe/count, normalized `bars[]`, and data-quality warnings. Raw
frames, session ids, cookies, tokens, and account-local values must not be
printed or recorded in tracked docs.

## Plan of Work

Add the CLI command, validate inputs before network, then perform a bounded
anonymous WebSocket read:

1. send an anonymous auth token,
2. create a chart session,
3. resolve the exchange-qualified symbol,
4. create one series with the requested timeframe and bounded count,
5. collect `timescale_update` / `du` bars,
6. complete on `series_completed` or return a structured failure if no bars are
   available.

Do not add streaming, extended sessions, authenticated sessions, cookie import,
or scanner/quote fallback in this slice.

## Validation and Acceptance

Required:

- `cargo test -p tradingview-cli market::bars -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract bars -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

Optional live smoke:

- `TV_EXPERIMENTAL_BARS=1 target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5`
- `TV_EXPERIMENTAL_BARS=1 target/debug/tv bars NYSE:IONQ --timeframe 1D --count 5`
- `target/debug/tv ohlcv --count 1`

Acceptance criteria:

- `tv bars` is hidden behind `TV_EXPERIMENTAL_BARS=1`.
- Invalid symbol/timeframe/count inputs fail before network.
- Successful bars payloads are bounded and public-safe.
- `tv ohlcv` behavior remains chart/CDP-dependent and unchanged.

## Idempotence and Recovery

The command is read-only. Re-running it may create a new temporary WebSocket
chart session but does not mutate account or Desktop state. If the protocol
fails or returns malformed data, return `internal_api_unavailable` or
`connection`/`timeout` as appropriate, never an empty successful bars payload.

## Artifacts and Notes

Do not paste live raw WebSocket frames, session ids, cookies, tokens,
machine-specific local paths, or account-local values into tracked docs.

## Interfaces and Dependencies

The CLI package uses `tokio-tungstenite` and `futures-util` for this lab
adapter. TLS roots are enabled at the workspace dependency level because the
browserless path uses `wss://`.

## Open Questions

- Whether this should later move into a reusable crate depends on more
  downstream evidence. For now, keep it CLI-owned and lab-gated.
