# Screener and hotlist upstream feasibility 2026-04-25

This note investigates the original upstream Stock Screener and Hotlist pull
request surfaces before adding any Rust CLI commands.

## Sources checked

- `gh pr view 66 -R tradesdontlie/tradingview-mcp --json number,title,body,files,updatedAt,url`
- `gh pr diff 66 -R tradesdontlie/tradingview-mcp`
- `gh pr view 89 -R tradesdontlie/tradingview-mcp --json number,title,body,files,updatedAt,url`
- `gh pr diff 89 -R tradesdontlie/tradingview-mcp`
- Read-only probe:
  `https://scanner.tradingview.com/presets/US_volume_gainers?label-product=right-hotlists`

The probe returned a public scanner preset payload shape with top-level
`fields`, `symbols`, `time`, and `totalCount`. It returned 20 symbol rows, had a
single field entry, and each row used compact keys `s` and `f`. No raw payload is
recorded here.

## Upstream PR #66: Stock Screener UI automation

Upstream PR #66 adds a broad TradingView Stock Screener dialog surface:

- `screener_open`, `screener_status`, `screener_get`, and `screener_close`
- screen management actions: `active`, `menu_actions`, and `save`
- filter actions: `list`, `remove`, and `clear`
- column action: `list`
- many modal or catalog actions as explicit `not_implemented_yet` stubs

The upstream implementation is pure UI automation over CDP. It opens a floating
Screener dialog using the right-toolbar
`[data-name="screener-dialog-button"]`, detects dialog state through
`[class*="screenerContainer"]`, reads the visible table, and reads filter pills
from `[data-name^="screener-filter-pill-"]`. The upstream diff notes that first
open can take several seconds while TradingView loads rows.

The read-only subset is useful, but still DOM fragile:

- status/open/get/close depend on current TradingView Desktop UI structure
- `get` reads only the currently displayed screen rows
- opening and closing the dialog changes UI state, even if it does not mutate
  account data
- table columns and row cells are display text, not a stable REST schema

The mutation subset is riskier:

- `filters remove` and `filters clear` mutate the active screen's filters
- `screens save` can persist screen changes to TradingView cloud state
- filter add/modify, column add/remove/reorder, and screen switch/save-as/delete
  are modal/catalog flows and should remain deferred unless separately planned

Disposition: do not implement PR #66 as a single Rust slice. If Rust adds UI
Screener support, start with a small read-oriented dialog slice only:
`status`, `open`, `get`, and `close`. Keep filter/screen/column mutation out of
the first implementation.

## Upstream PR #89: Hotlist scanner presets

Upstream PR #89 includes a separate `hotlist_get` surface backed by TradingView
scanner preset endpoints:

`GET https://scanner.tradingview.com/presets/US_<slug>?label-product=right-hotlists`

The upstream fork treats this as pure REST, no DOM, no auth, and validates slugs
against a whitelist:

- `volume_gainers`
- `percent_change_gainers`
- `percent_change_losers`
- `percent_range_gainers`
- `percent_range_losers`
- `gap_gainers`
- `gap_losers`
- `percent_gap_gainers`
- `percent_gap_losers`

The upstream tool caps `limit` at 20 because the preset endpoint returns one
page. The endpoint is closer to market-discovery input than active-chart control,
but it is read-only and materially less fragile than the UI Screener dialog.

Disposition: implemented as the narrow read-only Rust command
`tv scanner hotlist <SLUG> [--limit <N>]`. This keeps Hotlist REST reads separate
from future UI Screener dialog automation.

## Rust CLI boundary recommendation

Classify the remaining Screener/Hotlist ideas as follows:

| Surface | Recommendation | Rationale |
| --- | --- | --- |
| Hotlist preset REST reads | implemented as `tv scanner hotlist` | Read-only, no CDP, no UI mutation, small whitelistable surface. |
| UI Screener status/get/open/close | needs live UI evidence | Useful but DOM-fragile and changes visible UI state. |
| UI filter list / column list / active screen read | possible later | Read-only after opening the dialog, but depends on UI text and table structure. |
| UI filter remove/clear | defer | Mutates the active Screener screen and can persist through TradingView behavior. |
| UI screen save/switch/save-as/delete/rename/create | defer | Cloud-state and modal-flow risk; upstream mostly uses stubs for these. |
| UI column add/remove/reorder/reset | defer | Catalog and drag/drop UI automation; likely too brittle for core CLI now. |
| Scanner/product workflow packs | keep downstream | Rules packs and dashboards are workflow products, not core bridge replacement. |

Hotlist REST is now the first implemented Screener-like slice. Any next
implementation plan should not bundle UI Screener automation, watchlist bulk
mutation, or downstream scanner workflow rules.

## Implemented Hotlist contract

The Rust implementation:

- adds the read-only command `tv scanner hotlist <SLUG> [--limit <N>]`
- validate `SLUG` against the whitelist above before network access
- rejects `--limit 0`, defaults omitted limit to 20, and caps larger limits at 20
- uses `reqwest` outside CDP, similar in spirit to `tv search`
- returns the Rust JSON envelope with payload under `data`
- includes practical fields such as `slug`, `limit`, `count`, `total_count`,
  `fields`, and normalized `symbols`
- avoids recording raw live scanner payloads in tracked docs

## Suggested UI Screener implementation outline

A future UI Screener implementation should be separate from Hotlist REST. It
should start with only:

- `tv screener status`
- `tv screener open`
- `tv screener get [--limit <N>]`
- `tv screener close`

It should require a live smoke plan because it depends on TradingView Desktop
DOM selectors. It should not save screens, clear filters, remove filters, or
change columns in the first slice.

## Current CI note

While this feasibility pass was running, the pushed `v0.1.1` Release and CI
runs were observed as failed. That failure is outside this note's scope and
should be investigated as a separate CI/release task before more release work.

## Assumptions

- This note does not implement any Rust code.
- MCP server implementation remains not planned.
- Stock Screener UI automation is not old CLI migration backlog; it is a future
  feature decision.
- Hotlist REST reads are still an undocumented TradingView endpoint dependency,
  even if the endpoint is public and no-auth.
