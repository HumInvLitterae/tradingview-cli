# Screener discovery before edits

Use `tv spec screener screens list`, `screens actions`, `filters actions`,
`columns actions` or `columns config` for the chosen path when available; expand
the latter paths with the same `tv spec screener` prefix. Discovery does not
authorize changing the screen, filters or columns.

Screen lists read the title menu by default or open the catalog with `--catalog`.
They do not enumerate all account storage. Names and indexes are UI observations;
IDs and owner/shared flags can be absent. Scope and exact names matter when
selecting a target. Screen actions report recognized menu entries. `save_enabled`
means an enabled save candidate was seen, not that saving succeeded.

Filter actions probes one numeric-filter candidate's manual-range popover. Its
`range_options` does not describe every filter. `add_supported` is currently
false because this probe does not verify the add catalog; a separate filter-add
command exists. Column actions reads settings categories, while its header-menu
actions are currently empty. Thus `remove_supported` and `reset_supported` are
false even though a separate storage-based remove command exists. Reset remains
unimplemented. Treat these flags as limits of the probe, not a CLI capability map.

These menu probes change visible UI even when the Screener is already open.
Successful paths attempt to close their popups and any panel they opened, but
do not reconstruct prior popup/focus state. Early failures can bypass cleanup.
`restored_open_state` is the initial panel state, not cleanup success.

Columns config instead requires an already open panel and reads screen storage
using the Desktop session. It does not open menus or use MCP OAuth. Missing init
data, failed fetches and exact title mismatches fail. Matching titles alone cannot
distinguish same-title screens. Returned storage fields can fall back to loaded
init data, so they need not match unsaved UI edits. Column `id` and `params` come
from storage, while `name` is paired by visible position and identified with
`name_source: visible_column_index`; this is not an ID-based name match.

Keep screen IDs/names and configurations private. Official MCP screening can
answer suitable data queries but does not supply these saved-screen menus or
storage definitions. Reread target state before a separately requested edit.

## Opening and closing the Screener

Inspect `tv spec screener open` or `tv spec screener close` offline when
available. Default open operates the selected target's dialog and requires its
Screener button, even if a panel is already detected. It either reports the
existing open state or clicks and waits for panel detection. This does not prove
rows are loaded or authorize saved-screen edits.

`open --full-page` instead activates the first matching Screener target; it does
not use `--target-id` to select that target or reject multiple matches. If none
exists, it tries CDP tab creation, then a TradingView new-tab tile fallback on
creation error. Post-check can accept another matching target. Inspect `tab list`
first and use the returned `target_cli_args` for subsequent operations. The
created/reused flags describe the path taken, not unique tab ownership or data
readiness. Previous focus is not restored; failures can leave tabs or UI changed.
Inspect actual state before retrying rather than assuming nothing happened.

`close` sends Escape to the selected target only when a panel is detected. It
waits for panel disappearance and fails if still open; Escape may dismiss a
popup instead. An already closed panel returns `action=already_closed` and
`closed=false`, which is a successful no-op. This command does not close a tab
and has no full-page option. A full-page panel may remain detected after Escape;
do not automatically substitute a tab-close operation.
