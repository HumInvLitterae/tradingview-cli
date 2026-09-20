# v0.32.0 candidate roadmap

Status: independent CLI implemented and macOS command smoke passed; Windows/downstream qualification open, 2026-09-21.
[Work order](next-version-work-items.md) and the
[existing MCP ExecPlan](plans/tradingview-cli-official-mcp-client.md) own execution
and detailed contracts. No second feature plan is needed.

## Outcome

Add one explicitly selected official TradingView MCP read path to `tv`: authorize
once, reuse credentials across processes, and obtain recent daily/weekly/monthly
OHLCV for one exchange-qualified symbol with honest source and uncertainty
information. The downstream receives structured JSON and owns acceptance for its
analysis and storage. The first implementation target is connection proof, not
a wholesale replacement of existing data sources.

v0.31.4 is [released and archived](plans/archives/tradingview-cli-v0.31.4-release-readiness.md).
Its dependency/runtime-guidance changes are complete maintenance history. The
next feature is a **v0.32.0 candidate**, conditional on connection and downstream
acceptance. Keep the workspace at 0.31.4 during implementation; bump the version
when the accepted minor candidate enters release preparation.

## Settled scope

- Independent `tv mcp login/status/bars/logout`; the existing default, WebSocket
  `bars.v1`, Desktop/Pine observation paths and error envelope remain intact.
- One qualified symbol and recent count for `1D`, `1W`, `1M`; no date-range or
  pagination claim, intraday expansion, automatic source fallback, or batch API.
- Separate `mcp_bars.v1` and MCP-specific error details. Communication success,
  count sufficiency and data-condition knowledge are independent. Missing
  volume, unknown identity/delay/adjustment/session/finality remain explicit.
- Upstream owns connection, OAuth, protected credential reuse, bounded transport
  and response interpretation. Downstream owns use-specific acceptance, artifact
  conversion, provider-aware caching and changes to its fixed source/latency
  assumptions. No private downstream collection policy moves upstream.

The exact dependency additions and same-target live effects in the MCP plan are
already approved. Reuse that approval; browser consent still belongs to the
user, and broader actual consent or materially changed effects need a decision.
No additional agents, downstream writes or remote publication are authorized.
The local macOS HTTP/OAuth/credential harness and failure fixtures are implemented.
The owner explicitly confirmed the live scope; public discovery and client
registration, OAuth exchange, native storage and cross-process MCP discovery
succeeded after normal homepage sign-in. Fixed-worker access and refresh also
pass; the corrected wire-name mapping produced 20 daily, weekly and monthly bars
each, with matching symbol/timeframe echoes. The independent CLI now shares
that transport and emits `mcp_bars.v1`.
The owner authorized a local implementation commit after readability corrections,
deferred Windows runtime verification, and contacted the downstream PM. Windows
qualification and downstream acceptance remain open. The owner withdrew
artificial count/time stop gates. No direct async-trait dependency is needed. See the work record for evidence and the unchanged live scope.

## Delivery order

1. Implement the internal service/proof harness with synthetic OAuth/MCP and
   credential-store tests. Validate dependency features and platform boundaries.
2. Use the approved verification flow to prove actual authentication, restart,
   refresh and daily/weekly/monthly responses. Refine wire decoding from evidence.
3. Complete the initial CLI path, stable output mapping and usage documentation.
4. Have the downstream owner accept or explicitly quarantine saved observations,
   including negative cases and unknown semantics; assess backtest readiness
   separately from successful acquisition.
5. Qualify the minor candidate across the supported platforms, including
   mandatory Windows acceptance, and prepare its release only after the initial user journey is complete.

A beta-service or storage incompatibility may produce a precise no-go or narrower
useful observation capability. Do not manufacture schema evidence or convert a
recent sample into historical range coverage to satisfy a milestone.

## Maintained defers

The [v0.31 engineering triggers](v0.31-roadmap.md#evidence-triggered-engineering-candidates)
and [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md)
remain unchanged: CDP retry/reconnect, shared session/broker/daemon, common
public timing/recovery fields, foreground/indicator search, a higher bar cap,
additional intraday ranges, fractional-offset/width-derived geometry and Windows
MSIX activation are not prerequisites or promoted tasks. MCP OAuth refresh does
not authorize retries of Desktop operations. Screener/news/financials/watchlists/
alerts are outside the first MCP implementation.
