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

UI Screener reads are now evidence-backed enough for a narrow implementation
plan, but they should remain separate from `tv scanner hotlist`.

The next implementation, if chosen, should only add:

- `tv screener status`
- `tv screener open`
- `tv screener get [--limit <N>]`
- `tv screener close`

The implementation should not depend only on `[class*="screenerContainer"]`.
It should detect the current dialog with a small set of current indicators such
as visible Screener heading text, `[class*="screener"]`, visible Screener
`data-name` attributes, and the table presence. `close` should prefer an
explicit close affordance if discovered, but must support `Escape` because that
restored the live session safely.

`get` should document that it reads the currently visible Screener rows and
localized display text. It should not present the result as a stable REST
scanner schema.

## Still deferred

- filter remove / clear
- screen save / switch / save-as / rename / create / delete
- column add / remove / reorder / reset
- workflow scanner rules, dashboards, or downstream strategy packs

These surfaces need separate safety policy and workflow evidence before they
belong in the core Rust CLI.

## Validation

The live smoke changed only visible UI state and restored the Screener dialog to
its initial closed state. No TradingView layout, watchlist, alert, drawing,
Pine script, filter, screen, or column setting was intentionally modified.
