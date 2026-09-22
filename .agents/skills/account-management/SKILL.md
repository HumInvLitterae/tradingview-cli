---
name: account-management
description: Read or manage TradingView watchlists and simple price alerts with tv using official MCP, including mutation readback and uncertain outcomes.
---

# TradingView account management

Prefer official MCP for these operations: it avoids a Desktop connection and
visible-panel state. Honor an explicit source selection. Establish binary/auth
availability with [connection guidance](references/mcp-connection.md) when needed;
do not ask for source permission on each authenticated operation.

| User needs | First command | Continue when |
| --- | --- | --- |
| Find a watchlist | `tv mcp watchlist list` | Select the requested ID from the result; do not guess. |
| Read its symbols | `tv mcp watchlist get <ID>` | Preserve ordering, section labels, nulls and unknown completeness. |
| Find price alerts | `tv mcp alert list [--symbol <EXCHANGE:SYMBOL>] [--active true]` | Use returned IDs for `tv mcp alert get <ID>...`. Missing IDs are unreported, not deleted. |
| Create/edit/remove a watchlist or its symbols | `tv mcp watchlist create/update/add/remove/delete` | The target and account effect are requested; read [mutation guidance](references/mutations.md). |
| Create/change/stop/restart/delete simple price alerts | `tv mcp alert create/update/stop/restart/delete` | The condition, target and notifications are understood; read [mutation guidance](references/mutations.md). |

Read intent does not authorize changes. Reuse authorization for the same target
and effect. Normal use acts on the user's requested object; disposable targets
are for implementation tests, not a mandatory extra object for every operation.
Official read commands do not activate a watchlist or create a default list.

MCP price alerts do not replace Pine `alertcondition()` workflows. A limited
condition projection is insufficient to recreate an existing complex alert.
For Pine-specific or visible Desktop state, explain the missing capability and
use a separately authorized Desktop workflow; never silently substitute it after
an MCP failure. The optional `pine-develop` or `chart-analysis` skills may help,
but no other skill is required for the account operations here.

Report the actual source, requested effect, response and readback separately.
Account IDs and raw live results stay private. Outer success does not prove the
requested change occurred; unknown outcomes never authorize blind replay.
