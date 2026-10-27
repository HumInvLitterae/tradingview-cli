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

## Saved inspection and section changes

Use `tv watchlist inspect <ID>` to read one saved custom list through Desktop
without selecting it. For the displayed custom list, use `watchlist section
rename <SECTION> <NAME>`, `collapse <SECTION>` or `expand <SECTION>`. Names are
exact and preserved literally. The section must be rendered and unique, with no
selected rows or active display sort. Do not clear selections or switch lists
without authorization to make a request pass.

Section changes use the existing UI handlers once and confirm both independent
saved content and the target display. Inspect `saved_confirmation` and
`display_confirmation` separately on errors. Handler invocation is not proof of
persistence; null `changed` means uncertain. Changes can synchronize to other
tabs. Never repeat an uncertain write or infer rollback from a restored display.

## Reordering

Use `watchlist move <SYMBOL> --before <SYMBOL>`, `--after <SYMBOL>` or
`--to-section <NAME>`, with exactly one destination. `watchlist section move
<NAME> --before <NAME>` or `--after <NAME>` moves the entire section and its
members. Source symbols must be in the full display array and destination
sections expanded; rows need not be inside the visible viewport. Whole-section
moves preserve collapse state. Any duplicate symbol or heading
anywhere in the list, selection, or display sort blocks the operation.

These commands use full-array replacement. The final preflight GET can detect
existing conflicts but cannot prevent a concurrent edit from being overwritten.
Use only when the list is not being edited elsewhere. Saved and display
confirmation are separate, and a failed save may leave the UI reordered. Do not
retry or send a compensating replacement automatically.
