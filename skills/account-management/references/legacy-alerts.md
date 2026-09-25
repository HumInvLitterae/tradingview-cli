# Legacy Desktop price alerts

Prefer official MCP for supported explicit-symbol price alerts and explicit-ID
management. Legacy `alert list/create/delete` uses the selected Desktop session
and private account endpoints. It is not interchangeable with the MCP contract;
use `tv spec alert <action>` when available before choosing it. Pine indicator
alerts are a separate workflow, not a simple price-alert substitute.

List can return outer success with `data.error` and empty alerts. Malformed or
missing collections can also become empty results; do not infer no account
alerts. Missing active state defaults to true. Provider timestamps are not
normalized and projected conditions cannot reconstruct complex Pine alerts.
Messages remain in legacy output, so keep account details private.

Create uses the active chart symbol/timeframe. Conditions crossing, greater_than
and less_than map to cross, cross_up and cross_down. The internal API uses
on-first-fire, auto-deactivation and about 30-day expiry; popup and mobile push
are enabled, email/SMS are disabled and webhook is null. Currency can fall back
to USD and resolution to 1. These are different defaults from official MCP.
There is no dry-run.

The API path checks for a new matching ID, symbol marker, message, condition type
and approximate price. Only pre-create failures can fall back to DOM; after an
API creation attempt, request/readback failures do not. DOM fallback sets a
price field and optionally message, but does not set the requested condition UI.
Its returned condition is an echo and `created: true` means Create was clicked,
not that the alert persisted. Check source before reporting success or retrying.

Delete requires exactly one of `--id` and `--all`. Dry-run is supported only
with all and performs a fresh account read. Execution of all targets every alert
listed at that time, not the chart symbol or a prior preview's fixed set. No
separate confirmation flag is implemented. A missing single ID fails; an empty
all-target set is a no-op. Numeric-looking IDs are converted to JavaScript Number
without a safe-integer bound, unlike MCP's validated integer contract.

Deletion readback checks targeted IDs are absent, not that the account has no
newly created alerts. Partial deletion or failed verification does not roll back
the write. Inspect current state before repeating a mutation.
