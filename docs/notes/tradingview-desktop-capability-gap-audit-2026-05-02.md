# TradingView Desktop capability gap audit - 2026-05-02

This note records broad TradingView Desktop capability areas that are not fully
covered by `tv`. It is a prioritization aid, not a parity promise. The CLI
should keep focusing on agent/operator workflows where data provenance,
structured errors, and post-checks make the tool safer than ad hoc UI
automation.

## Implemented

- Chart state, symbol, timeframe, chart type, visible range, scroll, quote,
  OHLCV, screenshots, and selected chart data reads.
- Desktop-free symbol metadata, quote, batch quote, scanner scan, scanner
  hotlist, scanner metainfo, and bounded browserless historical bars.
- Watchlist read/add/remove, alert list/create/delete, indicator lifecycle,
  drawing lifecycle, Pine source/editor/check helpers, replay controls, pane
  operations, saved layout list/switch, tab management, and generic UI
  compatibility commands.

## Partial

- Fundamentals and earnings: `tv fundamentals <SYMBOL>` covers a curated
  scanner-backed single-symbol read. Broader financial statements, analyst
  estimates, transcripts, and rich event calendars remain outside the first
  implementation.
- Stock Screener: scanner REST reads, visible Screener reads, saved screen
  lifecycle, filters, and columns exist. Advanced UI-only filters and broad
  free-text or multi-option editors remain intentionally limited.
- News and events: the CLI can read price, bars, scanner fields, and
  screenshots, but it does not expose a dedicated TradingView news or events
  feed.
- Strategy / indicator evidence: current chart values and strategy report data
  are readable, but the CLI does not batch-run historical indicator series or
  optimize strategies.

## Candidate

- Earnings calendar and event read: useful for technical workflows because
  upcoming earnings can change trade risk. Prefer scanner-backed fields first;
  only investigate a richer Desktop panel or endpoint if scanner fields are
  insufficient.
- Financial statements table read: potentially useful if a public-safe
  Desktop-free or page-session read can return structured rows without raw
  account payloads.
- News/events read: useful only if the source and freshness are clear. Avoid
  scraping visible article text without a concrete workflow.
- Options/futures/crypto-specific reads: investigate only when a downstream
  workflow needs them and the data source boundary is clear.

## Deferred

- Advanced Screener UI parity, including arbitrary non-numeric filters and
  complex saved-screen editing, remains evidence-gated.
- Layout/preferences management beyond current saved-layout and pane surfaces
  is deferred until there is an operator workflow with safe readback.
- Browserless historical bars are available through stable `tv bars`; keep
  realtime streaming and automatic source mixing deferred until a separate
  workflow justifies them.

## Not planned / unsafe by default

- Broker/trading panel execution, order placement, account balances, or
  brokerage-connected state.
- Cookie/session import or export, login automation, or authenticated
  entitlement workarounds.
- A full TradingView Desktop parity project or generic browser automation
  framework.
