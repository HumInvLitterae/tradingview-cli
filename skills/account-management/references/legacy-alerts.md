# Legacy Desktop price alerts

Prefer official MCP for supported explicit-symbol price alerts and explicit-ID
management. Legacy `alert list/create/delete` uses the selected Desktop session
and private account endpoints. It is not interchangeable with the MCP contract;
use `tv spec alert <action>` when available before choosing it. Pine indicator
alerts are a separate workflow, not a simple price-alert substitute.

List returns an error for failed reads or malformed row collections; a valid
empty collection still succeeds. Creation/deletion checks use the same strict
reader, so unavailable readback cannot confirm success. Older binaries can
return outer success with `data.error`; never treat that as an empty account.
Missing active state defaults to true. Provider timestamps are not normalized.
When condition.value is absent, a simple cross/cross_up/cross_down condition
with exactly two series entries, `{type:barset}` then `{type:value,value:N}`,
projects its finite numeric threshold into condition.value. Extra entry fields,
study conditions, reversed or additional entries and nonnumeric thresholds are
not projected. Real zero and existing value fields (including null) are preserved.
Raw series remain omitted; projected conditions cannot reconstruct complex Pine
alerts. Symbols are not decoded or normalized, so a projected price does not
establish symbol-marker identity, currency, adjustment or session equivalence.
Messages remain in legacy output, so keep account details private.

Create uses the active chart symbol/timeframe. Conditions crossing, greater_than
and less_than map to cross, cross_up and cross_down. The internal API uses
on-first-fire, auto-deactivation and about 30-day expiry; popup and mobile push
are enabled, email/SMS are disabled and webhook is null. Currency can fall back
to USD and resolution to 1. These are different defaults from official MCP.
There is no dry-run.

Creation requires a positive safe-integer `r.alert_id` from the create response,
absent before dispatch and identifying exactly one saved row after dispatch.
Content similarity never substitutes for that identity. The saved symbol settings,
exact price, condition, frequency, message, notification settings, auto-deactivation
and expiry must match the submitted settings. Symbol-marker object key order is
ignored; expiry allows the server's truncation to whole seconds.

For these simple price conditions, saved condition resolution must be `"1"`
and `cross_interval:true`, including the outer cross-interval flag. The outer
resolution still matches the chart. This is the price-alert normalization, not
an exception for indicators or drawings; creating from a daily chart does not
request a daily-close trigger. Both `conditions` and an available `condition`
alias are checked.

Success includes `confirmation:{identity:"matched",settings:"matched"}` and
`activation:{state:"active"|"pending"|"unknown"}`. The CLI reads back once and
never waits or reactivates. Inactive with explicit null stop/error fields is
`pending`; missing or contradictory status is `unknown`. Neither proves eventual
activation. Inactive with a reported stop/error is `stopped` and fails.

An attributed row with mismatched or unavailable settings, or stopped activation,
returns an error with `created:true`, `alert_id`, `confirmation` and `activation`.
Settings are `mismatch` or `unknown` as appropriate. Inspect that ID; do not create
a replacement blindly. Missing/ambiguous response identity, failed readback,
request failures and lost/malformed Runtime results remain `created:null` with
`creation_outcome:"unknown"`. Only proven preflight failures retain `created:false`.
There is no automatic DOM/MCP fallback, retry or cleanup. Notification defaults
and `price_set` retain their existing meaning. Returned conditions remain a
sanitized projection rather than a complete reusable alert payload.

Delete requires exactly one of `--id` and `--all`. Dry-run is supported only
with all and performs a fresh account read. Execution of all targets every alert
listed at that time, not the chart symbol or a prior preview's fixed set. No
separate confirmation flag is implemented. A missing single ID fails; an empty
all-target set is a no-op. Numeric-looking IDs are converted to JavaScript Number
without a safe-integer bound, unlike MCP's validated integer contract.

Deletion readback checks targeted IDs are absent, not that the account has no
newly created alerts. Partial deletion or failed verification does not roll back
the write. Inspect current state before repeating a mutation.
