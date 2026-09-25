# Watchlist and alert discovery

Use `tv spec mcp watchlist list|get` or `tv spec mcp alert list|get` for the
chosen command when supported; use help on older binaries. Here `list|get` means
choose one subcommand, not a literal argument. Reads require MCP authentication
but no Desktop connection. They do not activate lists or change alerts, although
credential refresh can update local authorization state.

Watchlist list returns `items` in provider order with decimal-string IDs. Pass
the selected ID unchanged to get, which returns `watchlist` and checks its ID
against the request. IDs allow zero and reject leading zeros/whitespace. Preserve
symbol order and section labels; a section label is not a tradable symbol.
`symbols: null` is unknown, not an empty list.

Alert list accepts an optional qualified symbol and explicit `--active true` or
`--active false`; omission leaves active state unfiltered. Known response fields
that contradict filters fail, but null fields remain unknown. List rows retain
provider order and positive numeric alert IDs. Get accepts 1–100 distinct IDs
up to 9223372036854775807. It returns requested order with `requested_id`,
`status` and nested `alert`. An unreported ID has a null alert and does not prove
deletion. `returned_count` counts actual alerts, not placeholder rows; check
`ids_status` and `unreported_count`. Unexpected or duplicate IDs fail.

Successful transport does not establish complete account coverage. These reads
have no pagination controls or automatic traversal. Receipt time is client time;
account time fields remain provider strings. Conditions are a limited projection
with unknown completeness, insufficient to recreate complex Pine alerts. Messages
and webhook URLs are excluded; notification flags do not establish delivery.
Keep account IDs and returned account data private.
