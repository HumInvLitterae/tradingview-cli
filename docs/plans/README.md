# Current work

Use the relevant work record and current code when resuming. A released patch's
validation does not establish the behavior of a proposed new dependency or
provider integration.

| Purpose | Record |
| --- | --- |
| Active feature: official MCP client, first local proof slice | [MCP ExecPlan](tradingview-cli-official-mcp-client.md) — approved scope, ready for implementation |
| v0.32.0 candidate direction | [Roadmap](../next-version-roadmap.md) |
| Ordered work, ownership and next return point | [Inventory](../next-version-work-items.md) |
| Released baseline | [v0.31.4 closeout](archives/tradingview-cli-v0.31.4-release-readiness.md) |
| CDP stability triggers | [Strategy note](../notes/cdp-stability-and-autonomous-operation-strategy.md) |
| Completed plans and older context | [Historical catalog](archives/README.md) |

The v0.31.4 release prerequisite is satisfied. The existing MCP dependency and
bounded live scope stays approved; normal browser consent and genuinely new
effects are handled at their concrete boundary. The current request prepares
the plan; implementation starts with the local service/proof harness.

State ownership follows [PLANS.md](../../.agents/PLANS.md): direction in the
roadmap, order in the inventory, and detailed acceptance in the existing plan.
Keep publication closeout, next-version planning and implementation changes in
coherent separate commits. Do not add a tracked evidence commit solely to record
its own hash, or rerun functional tests merely because plan documents changed.
