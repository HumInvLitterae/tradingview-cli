# Current work

Use the relevant work record and current code when resuming. A released patch's
validation does not establish the behavior of a proposed new dependency or
provider integration.

| Purpose | Record |
| --- | --- |
| Active feature: official MCP client | [MCP ExecPlan](tradingview-cli-official-mcp-client.md) — initial OHLCV plus search/columns/symbol implemented; multi-symbol data next, Windows qualification open |
| v0.32.0 candidate direction | [Roadmap](../next-version-roadmap.md) |
| Ordered work, ownership and next return point | [Inventory](../next-version-work-items.md) |
| Released baseline | [v0.31.4 closeout](archives/tradingview-cli-v0.31.4-release-readiness.md) |
| CDP stability triggers | [Strategy note](../notes/cdp-stability-and-autonomous-operation-strategy.md) |
| Completed plans and older context | [Historical catalog](archives/README.md) |

The v0.31.4 release prerequisite is satisfied. The existing MCP dependency and
same-target live scope stays approved; normal browser consent and genuinely new
effects are handled at their concrete boundary. The local HTTP/OAuth/credential
harness is implemented and fixture-verified.
OAuth login, native storage, cross-process reuse, refresh and OHLCV proof have
succeeded on macOS. The independent `tv mcp` command group is implemented.
The public commands also passed a macOS live smoke. Downstream reports initial
observation storage/readback and error compatibility complete; analysis adoption is separate. Required Windows runtime acceptance
remains open. The implementation and credential correction are committed.
Symbol search, column discovery and single-symbol data passed deterministic
checks and macOS public CLI smoke; multi-symbol data is next. The roadmap records the agreed
expansion order, with watchlists/alerts after intraday
bars and before financial data. Windows execution
is deferred by the owner. The former count/time approval gates were withdrawn; request counters remain evidence.
No Codex MCP setup is required. The work record owns the concrete execution
live observations and remaining acceptance gates.

State ownership follows [PLANS.md](../../.agents/PLANS.md): direction in the
roadmap, order in the inventory, and detailed acceptance in the existing plan.
Keep publication closeout, next-version planning and implementation changes in
coherent separate commits. Do not add a tracked evidence commit solely to record
its own hash, or rerun functional tests merely because plan documents changed.
