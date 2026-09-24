# Account mutations and readback

## Explicit watchlist changes

Use these explicit changes only for the requested account effect.
Use account-local IDs obtained from `watchlist list`.

```sh
tv mcp watchlist create "Example research" --symbols NASDAQ:EXAMPLE,NYSE:OTHER
tv mcp watchlist update 12 --name "Renamed research"
tv mcp watchlist update 12 --description ""
tv mcp watchlist add 12 NASDAQ:EXAMPLE
tv mcp watchlist remove 12 NYSE:OTHER
tv mcp watchlist delete 12
```

The IDs and symbols above are synthetic. Creation requires an explicit nonblank
name (up to 500 characters). Update requires a name and/or description; omission
keeps a field unchanged, while an empty description requests clearing it.
Descriptions are bounded to 4096 characters by the client. Create accepts up to
100 distinct qualified symbols; add/remove require 1..100. No implicit chunking
occurs. Adding an existing symbol moves it to the end; it is not a no-op.
Deletion names one target and cannot be inferred from a read request.

Each invocation sends at most one mutation and, after a valid response, one
readback using the same credential/admission service and overall deadline.
No mutation is replayed after timeout, authentication failure or response error.
OAuth renewal before dispatch retains the existing behavior; a rejected mutation
does not trigger the read-command refresh-and-repeat path. Scopes are never
expanded automatically. The public OAuth metadata advertises `mcp:read` and
`mcp:tools`, but the watchlist tool definitions do not declare per-tool scopes;
the disposable-list lifecycle succeeded under the existing `mcp:read` grant.
That scope name is not a token-level guarantee against account writes. No
additional scope was needed for the observed workflow.

Results use `mcp_watchlist_mutation.v1`,
`source_category:desktop_free_mutation` and `requires_desktop:false`.
The outer success envelope means a valid mutation response was received.
It does **not** assert that the requested state was confirmed; consumers must
inspect `readback.status`. A successful response with failed readback is not
a reason to repeat the mutation.

Selected fields from a confirmed rename:

```json
{
  "contract_version": "mcp_watchlist_mutation.v1",
  "operation": "update",
  "target_id": "12",
  "mutation": {
    "status": "response_received",
    "tool_attempts": 1,
    "automatic_retry": false
  },
  "readback": {"status": "matched", "tool_attempts": 1}
}
```

| Readback status | Meaning |
| --- | --- |
| `matched` | Requested name/description/symbol postconditions match the ID-specific observation. |
| `mismatch` | A reported field contradicts the requested postcondition. |
| `unconfirmed` | Required observation fields are absent/null. |
| `not_reported` | After deletion, the target is absent from the returned list; account-wide list completeness remains unconfirmed. |
| `still_present` | The deleted target is still reported by the list read. |
| `not_performed` | Creation returned no usable target ID; the client does not guess by name. |
| `failed` | Readback failed; its structured error is retained separately from the received mutation response. |

Ordinary readback includes the normalized target snapshot. Deletion reports
only membership/timestamp/completeness evidence, not unrelated account lists.
A missing description stays unknown, including after a requested update.

A mutation failure uses `mcp_error.v1` with `details.mutation.status` equal to
`not_attempted` or `outcome_unknown`; validation/local preflight errors retain
zero tool attempts. HTTP/MCP rejection does not prove that no change occurred.
For example, a timeout after attempted dispatch has
`code:deadline_exceeded`, `mutation.status:outcome_unknown`, and
`automatic_retry:false`. Inspect the target before considering a new mutation.
For an uncertain create without an ID, inspect the list and resolve ownership;
do not create another list or pick a same-named list automatically.

## Explicit alert changes

Before updating an inactive alert, explain that even a rename reactivates it.
If reactivation is not already in the requested scope, resolve that effect before
updating. Do not reconstruct a Pine alert from its partial condition projection.

`tv mcp alert` provides simple price-alert creation, settings updates and
explicit lifecycle operations. The existing Desktop commands remain separate.
These synthetic examples change the account when run with real symbols/IDs:

```sh
tv mcp alert create NASDAQ:EXAMPLE --price 100 --condition greater --resolution 1D --name "Example threshold"
tv mcp alert update 12 --name "Renamed threshold" --email false
tv mcp alert stop 12 13
tv mcp alert restart 12
tv mcp alert delete 12
```

Create requires a qualified symbol, finite price and a nonblank name of at most
300 characters. Supported conditions are `cross` (default), `cross_up`,
`cross_down`, `greater` and `less`. `--resolution` uses official chart strings:
`1` (default), `5`, `15`, `30`, `60`, `240`, `1D`, `1W`, `1M`.
Creation explicitly sends `email:false`, `mobile_push:false`, `popup:false`,
`auto_deactivate:false` and `monitor:false`. Override the first four using
`--email true|false`, `--mobile-push true|false`, `--popup true|false`, and
`--auto-deactivate true|false`. Notifications default off even where the official
API defaults on. Creation still creates an active alert and consumes account
capacity. Expiration uses the provider default; its observed value is retained
by `alert get`.

Update accepts `--name` and those four boolean settings; at least one is required.
Omitted settings stay unchanged, and explicit `false` is transmitted.
**Updating reactivates the alert**, including a name-only change to a stopped
alert. It cannot change symbol, condition, price or resolution. Stop preserves
settings and history; restart activates the existing conditions and notification
settings. Delete also deletes fire history. Lifecycle commands accept 1..100
distinct positive integer IDs, and never imply all alerts. Obtain IDs from
explicit reads; do not guess them.

This slice omits message/webhook editing, expiration editing, server-side
monitoring, fire-log reads and Pine condition creation/reconstruction. Missing
notification values in reads remain null, not false. No new dependency,
credential store or implicit Desktop fallback is introduced.

The data contract is `mcp_alert_mutation.v1`, with
`source:tradingview_mcp`, `source_category:desktop_free_mutation` and
`requires_desktop:false`. Like watchlist changes, one mutation is followed by
one readback. `mutation.status:response_received` is separate from verified
postconditions. For example, before `stop 12 13` both IDs may be active. A valid
reply followed by ID 12 stopped and ID 13 omitted produces this abbreviated data:

```json
{
  "contract_version": "mcp_alert_mutation.v1",
  "operation": "stop",
  "request": {"alert_ids": [12, 13]},
  "target_ids": [12, 13],
  "effects": {"reactivates": false, "deletes_fire_history": false},
  "mutation": {"status": "response_received", "tool_attempts": 1, "automatic_retry": false},
  "readback": {
    "status": "unconfirmed",
    "tool_attempts": 1,
    "items": [
      {"requested_id": 12, "status": "matched", "alert": {"alert_id": 12, "active": false}},
      {"requested_id": 13, "status": "unreported", "alert": null}
    ]
  }
}
```

For creation, `matched` requires observed symbol, resolution, condition, price,
name, active state and requested notification settings to match; absent evidence
is `unconfirmed`. Updates compare requested settings and active state. Stop and
restart compare active state for every requested ID. A known contradiction is
`mismatch`; a missing row is `unreported`. Batch aggregate is `matched` only when
all targets match; any known contradiction yields `mismatch`, otherwise missing
or unknown evidence yields `unconfirmed`. Matching covers the checked fields,
not lossless alert/Pine reconstruction or notification delivery.

Delete reads the alert list and reports each target as `still_present` or
`not_reported`; absence alone does not establish complete deletion proof.
Unrelated account rows are excluded. Creation without a returned ID uses
`readback.status:not_performed`, never a guessed identity. Readback failure uses
`readback.status:failed` with its structured error; the received mutation reply
is retained. For example, a readback 429 is not a reason to create another alert.
Mutation errors retain `not_attempted` or `outcome_unknown`; an authentication
rejection, timeout or malformed response after dispatch never triggers automatic
refresh-and-replay. Inspect the target before another explicit change.

Do not infer write permission from the OAuth scope name or catalog annotations.
