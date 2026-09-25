---
name: account-management
description: Read or manage TradingView watchlists and simple price alerts with tv using official MCP, including firing history, mutation readback and uncertain outcomes.
---

# TradingView account management

Prefer official MCP for these operations: it avoids a Desktop connection and
visible-panel state. Honor an explicit source selection. Establish binary/auth
availability with [connection guidance](references/mcp-connection.md) when needed;
do not ask for source permission on each authenticated operation.

When supported by the installed binary, `tv spec <command path>` provides
argument metadata without connecting. `tv spec mcp alert history` also describes
source, limits and symbol discovery. List/get paths describe ID formats, filters
and partial results; see [read guidance](references/reads.md).
Watchlist/alert mutation paths describe
ID discovery, side effects and readback. For example, inspect
`tv spec mcp alert update` before deciding whether reactivation is intended.
Respect partial validation and unavailable semantic annotations; use `--help` on older binaries. Do not fetch the whole
index when the command is already known.

| User needs | First command | Continue when |
| --- | --- | --- |
| Find a watchlist | `tv mcp watchlist list` | Select the requested ID from the result; do not guess. |
| Read its symbols | `tv mcp watchlist get <ID>` | Preserve ordering, section labels, nulls and unknown completeness. |
| Find price alerts | `tv mcp alert list [--symbol <EXCHANGE:SYMBOL>] [--active true]` | Use returned IDs for `tv mcp alert get <ID>...`. Missing IDs are unreported, not deleted. |
| Check alert firing history | `tv mcp alert history --symbol <EXCHANGE:SYMBOL> --days 7 --limit 100` | Preserve unknown coverage; historical firing does not establish current alert state or notification delivery. |
| Create/edit/remove a watchlist or its symbols | `tv mcp watchlist create/update/add/remove/delete` | The target and account effect are requested; read [mutation guidance](references/mutations.md). |
| Create/change/stop/restart/delete simple price alerts | `tv mcp alert create/update/stop/restart/delete` | The condition, target and notifications are understood; read [mutation guidance](references/mutations.md). |

For an explicitly requested legacy Desktop watchlist change, read
[legacy watchlist guidance](references/legacy-watchlists.md). Its active-list
targeting, API-to-DOM fallback and existing-member ordering differ from MCP.

For an explicit legacy Desktop price-alert request, consult
[legacy alert guidance](references/legacy-alerts.md): notification defaults,
list errors and DOM creation evidence differ substantially from MCP.

Read intent does not authorize changes. Reuse authorization for the same target
and effect. Normal use acts on the user's requested object; disposable targets
are for implementation tests, not a mandatory extra object for every operation.
Official read commands do not activate a watchlist or create a default list.

History requires an explicit symbol. Days default to 7; the result cap defaults
to 100 and cannot exceed 2000. `returned_count` is measured; `limit_reached`
only means the cap was reached, not that more events exist. Empty or short
history does not prove coverage. `alert_id` identifies the alert, not the firing
event, and `fired_at_unix_seconds` is the parsed UTC firing time, not bar time.
Notification messages and webhook contents are omitted. Delivery status remains
unknown; a fire record is not proof that a notification reached a person.

MCP price alerts do not replace Pine `alertcondition()` workflows. A limited
condition projection is insufficient to recreate an existing complex alert.
For Pine-specific or visible Desktop state, explain the missing capability and
use a separately authorized Desktop workflow; never silently substitute it after
an MCP failure. The optional `pine-develop` or `chart-analysis` skills may help,
but no other skill is required for the account operations here.

Report the actual source, requested effect, response and readback separately.
Account IDs and raw live results stay private. Outer success does not prove the
requested change occurred; unknown outcomes never authorize blind replay.
