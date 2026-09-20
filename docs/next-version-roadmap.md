# Next-version roadmap

Status: direction and order confirmed by the owner on 2026-09-20. v0.31.4
maintenance preparation comes first; the concrete MCP dependency/proof scope is
approved for the following stage. Do not mix MCP inputs into the patch.
Local planning/release commits are authorized after the passed Cargo consistency
check. Reuse valid test evidence; version-only commits do not require repeated
functional validation. Publication and additional agent sessions remain unauthorized.
Order and candidate evidence live in the [inventory](next-version-work-items.md);
the [MCP client plan](plans/tradingview-cli-official-mcp-client.md) owns contracts
and acceptance. This roadmap supersedes v0.31 direction for new work.

## Release recommendation

Ship the already accumulated maintenance work as **v0.31.4**. Develop the official MCP client
as an explicit, additive **v0.32.0 candidate**, with promotion conditional on
connection proof and downstream acceptance. Prepare 0.31.4 now; do not bump to 0.32.0 during patch preparation.

| Choice | Benefit | Cost / decision |
| --- | --- | --- |
| v0.31.4 first; v0.32.0 MCP later (recommended) | Deliver dependency and runtime-guidance maintenance independently of beta-service uncertainty. | Two release cycles; an additional release-readiness record is needed only when patch preparation starts. |
| One v0.32.0 including MCP and maintenance | One release cycle and a visible new capability. | Delays maintenance for OAuth, data semantics, and platform validation; requires explicit combined scope approval. |
| v0.31.4 only; defer MCP implementation | Smallest immediate maintenance commitment. | Leaves the newly demonstrated official-source opportunity untested. |

The ten commits after v0.31.3 contain nine dependency refreshes and one agent
guidance/package reorganization, with no `crates/` diff. That supports a patch
classification, but dependency changes can still affect executable behavior.
There is no newly evidenced small Rust fix to add from the inspected bars
paths, downstream consumers, and current CI. This is a scoped assessment, not
a repository-wide defect audit.

## Official-source direction

Prefer investigating a supported upstream source where it serves the workload.
Keep chart observations, custom Pine studies, and existing historical retrieval
available. A successful official read alone does not justify deleting the
WebSocket implementation or changing any default.

The first useful MCP increment is one exchange-qualified symbol, recent N bars,
and daily/weekly/monthly intervals, with durable OAuth reuse and explicit
uncertainty. The upstream owns connection/authentication, bounded requests,
response validation, and its CLI/JSON contract. The downstream owns selection,
acceptance for a particular use, storage transformation, research and backtests.
The integration boundary stays process invocation plus JSON.

The existing exclusion is **MCP server development**. An official-service MCP
client is an approved later-stage client scope. No daemon,
cookie import, account-login automation, source mixing, or trading authority is
introduced by this direction.

## Maintained defers

The [v0.31 trigger table](v0.31-roadmap.md#evidence-triggered-engineering-candidates)
and [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md)
remain in force. No new ordinary-operation failure evidence was supplied or
collected here. OAuth refresh for the proposed official client is not a trigger
for CDP reconnect, replaying Desktop mutations, or shared process ownership.

- Pre-dispatch resilience needs observed/reproducible target-list or WebSocket
  connection failure; shared session/broker needs lifecycle/stale-event evidence
  and an explicit background-process policy decision.
- Common public timing or recovery metadata needs a concrete consumer and
  evidence that the advertised timing or dispatch/effect state is derivable.
- Renderer foreground/indicator search needs a concrete render-linked mechanism
  or relevant Desktop build change; current no-go evidence remains intact.
- Raising the 5,000-bar cap needs a workload that cannot use bounded windows;
  extra intraday date ranges need a named timeframe and actual demand.
- Finite-f64 right-offset restoration needs a reviewed fractional contract and
  reversible runtime proof; width-derived drawing geometry needs an explicit
  sign, time-anchor and readback contract before a convenience command;
  Windows MSIX/AUMID activation needs a concrete installation/activation target
  and platform evidence. These are separate proposals, not MCP prerequisites.

## Next action

Prepare v0.31.4 first and keep its release artifacts in one final release
preparation commit. Preserve the approved MCP dependency and bounded-proof
scope for the following stage. The actual consent screen or changed scope can
still require a new decision; publication remains owner-controlled.
