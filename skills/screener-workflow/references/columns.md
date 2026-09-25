# Saved column changes

Inspect `tv spec screener columns <action>` when supported. Add/remove/reorder
require an already open Screener and readable active saved configuration. They
use the Desktop session to fetch storage; they do not open the panel or force a
UI refresh. Execution is restricted to active screen names containing the
case-sensitive substring `CLI-Test` or `テスト`. Dry-run still reads storage,
but only calculates the proposed `after_columns` without saving.

- Add uses a trimmed nonempty storage `--id` and optional JSON-object
  `--params-json` (default `{}`). Omitted `--after-index` appends; a supplied
  zero-based index must refer to an existing saved column. Duplicate IDs/params
  are not rejected locally, and preview does not verify provider support.
- Remove requires exactly one of `--index` or a nonblank `--name`. The name is a
  case-insensitive substring and must match one visible column. The visible
  index then selects the saved column at that position; verify ID/params because
  this is not an ID-based match. No local guard prevents removing the last column.
- Reorder requires distinct in-range saved `--from-index` and `--to-index`. The
  destination is the final position after removal, not an insert-after position.

Execution writes the fetched screen document with the new custom column sequence
and `active_column_set: custom`. It reads storage back and compares ordered
ID/params pairs and count. This does not verify refreshed UI, displayed data or
every screen field. Returned names derive from earlier positional mapping, not
a fresh identity check; init-data fallback can also differ from unsaved UI state.
Intervening edits are not reconciled by the adapter.

A write may complete before readback fails. Inspect current storage before
retrying, especially add, which can insert duplicates. These are account-setting
changes, not temporary display toggles. Official MCP does not edit these saved
column sets. Keep account-local configuration and IDs private.
