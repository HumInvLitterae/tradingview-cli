# UI Screener read evidence 2026-04-26

This note records a live evidence pass for the remaining upstream Stock
Screener UI surface from `tradesdontlie/tradingview-mcp` PR #66.

## Sources checked

- `gh pr view 66 -R tradesdontlie/tradingview-mcp --json number,title,body,files,updatedAt,url`
- `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`
- `docs/notes/upstream-pr-triage-2026-04-25.md`
- Live TradingView Desktop smoke through the current Rust `tv` CLI

No raw Screener table payloads, account-linked identifiers, or local absolute
paths are recorded in this note.

## Upstream PR #66 boundary

Upstream PR #66 mixes two kinds of surface:

- read-oriented dialog actions: `screener_open`, `screener_status`,
  `screener_get`, and `screener_close`
- screen, filter, and column management actions that can mutate the active
  Screener state

The Rust CLI should not import the whole PR. The mutation actions remain
deferred because they can change the active TradingView Screener screen, remove
filters, save cloud-backed screen state, or depend on modal/catalog UI flows.

## Live evidence

The smoke target initially had multiple TradingView CDP pages, so the active
target was selected with `TV_CDP_TARGET_ID` after `tv tab list`. The visible
Screener dialog was initially closed.

Observed read-only selectors and structure:

- `[data-name="screener-dialog-button"]` found one visible right-toolbar button
  with Japanese aria-label text for Screener.
- `[class*="screenerContainer"]` did not match the current dialog after open.
  This means the exact upstream status selector is stale for this live session.
- `[class*="screener"]` did match the open dialog area.
- `[data-name*="screener"]` exposed the screen title and visible filter pill
  buttons.
- `[data-name^="screener-filter-pill-"]` found 19 visible filter pills.
- `table` found one visible Screener table.
- `table th` found 14 visible header cells.
- `table tbody tr` returned 20 visible row elements through the generic
  `ui find` cap.

The dialog could be opened with:

```bash
tv ui click --by data-name --value screener-dialog-button
```

Clicking the same toolbar button did not close the dialog in this session.
Pressing `Escape` closed it and restored the original closed state:

```bash
tv ui keyboard Escape
```

## Recommendation

UI Screener reads are now implemented as a narrow Rust CLI slice, but they
remain separate from `tv scanner hotlist`.

The implemented surface is:

- `tv screener status`
- `tv screener open`
- `tv screener get [--limit <N>]`
- `tv screener screens active`
- `tv screener screens actions`
- `tv screener screens save [--dry-run]`
- `tv screener filters list`
- `tv screener filters remove --index <N>|--text <TEXT> [--dry-run]`
- `tv screener filters clear [--dry-run] --confirm-clear`
- `tv screener columns list`
- `tv screener columns actions`
- `tv screener columns remove --index <N>|--name <TEXT> --dry-run`
- `tv screener close`

The implementation does not depend only on `[class*="screenerContainer"]`.
It detects the current dialog with a small set of current indicators such
as visible Screener heading text, `[class*="screener"]`, visible Screener
`data-name` attributes, and the table presence. `close` uses `Escape` because
that restored the live session safely.

`get` should document that it reads the currently visible Screener rows and
localized display text. The metadata commands should document that they read the
active screen title, visible filter pills, and visible columns from the same UI
state. None of these commands should present the result as a stable REST scanner
schema.

The column-management follow-up can read the visible column settings categories
and resolve a visible column target in dry-run mode. Live evidence on
2026-04-26 found the settings categories, search, add-column configuration, and
header sort/move menus, but did not expose a safe visible per-column remove
action. Normal column remove, reset, add, and reorder therefore remain
evidence-gated.

The guarded filter mutation follow-up uses the same visible filter pill surface.
Live evidence showed that opening a filter pill exposes a popover button whose
class includes `removeButton`; the Rust command clicks that button and then
verifies the target filter disappeared.

## Still deferred

- screen save / switch / save-as / rename / create / delete
- column add / remove / reorder / reset
- workflow scanner rules, dashboards, or downstream strategy packs

These surfaces need separate safety policy and workflow evidence before they
belong in the core Rust CLI.

## Validation

The original read-only live smoke changed only visible UI state and restored the
Screener dialog to its initial closed state. No TradingView layout, watchlist,
alert, drawing, Pine script, filter, screen, or column setting was intentionally
modified during that read evidence pass.
