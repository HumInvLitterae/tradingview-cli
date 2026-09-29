# Legacy Desktop watchlist changes

Prefer official `tv mcp watchlist list/get` followed by an explicit list-ID
mutation when it meets the request. Legacy `watchlist add/add-bulk/remove` needs
Desktop and targets the active list; it has no list-ID or dry-run option.
Use `tv spec watchlist <action>` when available to inspect its behavior offline.
Do not silently replace a requested source or target.

The legacy path first tries the Desktop-session internal API. DOM fallback is
limited to recognized pre-dispatch list failures; once a POST is attempted,
transport, HTTP and readback failures stop without a second DOM mutation.
Readback requires the original list ID and readable symbols, even if another
list becomes active. Failure does not imply rollback. Older binaries can replay
uncertain writes through DOM or check another list; inspect the installed spec. DOM paths can open the panel and alter focus/input without restoring it.
Rendered-row counts and presence/absence are not complete account-list evidence.

Symbols are trimmed and must be nonempty, with no case normalization or strict
exchange qualification check. Legacy add skips an exact existing member; official
MCP add instead moves an existing member to the end. Legacy remove reports an
absent member as an error. DOM remove uses a row control and does not confirm a
deletion dialog. Inspect the actual source and outcome before continuing.

Bulk add allows at most 50 unique trimmed symbols, deduplicated case-sensitively.
The delay is 0–10000 ms, default 750, between unique attempts. All unique symbols
are attempted even after failures; `--allow-partial` changes only the final
result, not continuation. Without it, failures produce an error with the batch
payload in details; with it, the payload succeeds despite failed rows. Neither
mode rolls back earlier changes.

Read each result and its added/already_present/failed/skipped_duplicate status.
Requested count includes duplicates; processed count excludes them. Each item
resolves the active list again, so selection changes can split a batch across
lists. Do not retry a whole batch or repeat uncertain writes without checking
current state. Keep account-local details and raw errors out of shared artifacts.
