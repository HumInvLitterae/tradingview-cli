# Screener and hotlist upstream feasibility 2026-04-25

This note investigates the original upstream Stock Screener and Hotlist pull
request surfaces before adding any Rust CLI commands.

## Sources checked

- `gh pr view 66 -R tradesdontlie/tradingview-mcp --json number,title,body,files,updatedAt,url`
- `gh pr diff 66 -R tradesdontlie/tradingview-mcp`
- `gh pr view 89 -R tradesdontlie/tradingview-mcp --json number,title,body,files,updatedAt,url`
- `gh pr diff 89 -R tradesdontlie/tradingview-mcp`
- `docs/notes/ui-screener-read-evidence-2026-04-26.md`
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
- remaining column reset flows need a reliable default source and should remain
  deferred unless separately planned; column add is implemented only as a
  low-level storage id + params insertion command

Disposition: do not implement PR #66 as a single Rust slice. If Rust adds UI
Screener support, start with a small read-oriented dialog slice only:
`status`, `open`, `get`, and `close`. Keep filter/screen/column mutation out of
the first implementation.

Later Rust slices added guarded visible-filter cleanup, menu-visible screen
list/switch, catalog-backed screen list/switch, exact screen action/save
support, guarded screen create/rename/save-as/delete, and storage-backed column
config/remove/reorder after separate evidence and safety plans. Column
add/reset remain deferred.

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
| Generic Stock Scanner REST reads | implemented as `tv scanner scan` | Read-only, no CDP, no UI mutation, and useful for basic market discovery before considering high-risk UI Screener mutation. |
| UI Screener status/get/open/close | implemented as `tv screener` | Useful but DOM-fragile and changes visible UI state. Implemented as a narrow read-only UI dialog slice after live evidence in `docs/notes/ui-screener-read-evidence-2026-04-26.md`. |
| UI filter list / column list / active screen read | implemented as `tv screener` metadata reads | Read-only after opening the dialog, implemented as lightweight metadata commands that restore the initial open/closed dialog state. |
| UI filter remove/clear | implemented as guarded `tv screener filters remove/clear` | Mutates the active Screener screen, so remove supports dry-run target reporting and clear requires explicit confirmation. |
| UI filter actions/add/modify | implemented as guarded `tv screener filters actions/add/modify` | `actions` reports visible add/edit capability and numeric preset options. `add` searches the visible add-filter catalog, supports dry-run candidate reporting, validates finite values before CDP, clicks candidate/range options through CDP mouse events, and verifies a new visible filter pill after mutation. `modify` supports existing visible numeric range presets and single visible option selection. The option path supports dry-run option reporting, clears other selected options when TradingView exposes selection state, and verifies visible filter text after mutation. Broader multi-option add/remove/replace semantics and free-text filter editing remain deferred. |
| UI menu-visible screen list/switch | implemented as guarded `tv screener screens list/switch` | Lists exact visible menu names and supports dry-run target reporting. Non-dry-run switch verifies the active title and fails safely if TradingView does not activate the clicked row. |
| UI saved-screen catalog list/switch | implemented as guarded `tv screener screens list/switch --catalog` | Uses the saved-screen catalog for exact-name targeting, supports dry-run target reporting, and verifies the active title after mutation. |
| UI screen actions/save | implemented as guarded `tv screener screens actions/save` | Lists visible screen menu actions and clicks only exact `Save screen` / `スクリーンを保存`; dry-run reports the target action before mutation. |
| UI screen create/rename/save-as/delete | implemented as guarded test-screen lifecycle commands | Create, rename, and save-as are guarded test-screen lifecycle commands with dry-run and active-title post-checks. Delete uses exact saved-screen storage API targeting, refuses non-test and active screens, and verifies post-delete absence. |
| UI column config/add/remove/reorder | implemented as guarded storage-backed commands | `columns config` reads saved storage column ids and params. `columns add` inserts a known storage id and JSON-object params. `columns remove` and `columns reorder` dry-run the expected storage order, limit normal mutation to test/disposable screens, save the custom column set through storage API, and verify the re-fetched order before success. |
| UI column reset | defer | Requires a reliable default column source that current evidence does not expose. |
| Scanner/product workflow packs | keep downstream | Rules packs and dashboards are workflow products, not core bridge replacement. |

Hotlist REST, generic scanner REST, the read-oriented UI Screener dialog slice,
guarded filter cleanup, preset-backed filter modify, single-option filter
editing, guarded filter add, guarded screen lifecycle commands, and
storage-backed column config/add/remove/reorder are now implemented. Any next
implementation plan should not bundle column reset, broader multi-option or
free-text filter editing, or downstream scanner workflow rules without fresh
evidence.

## Implemented REST scanner contract

The Rust implementation:

- adds the read-only command `tv scanner hotlist <SLUG> [--limit <N>]`
- adds the read-only command `tv scanner scan` for the `america` scanner market
- validate `SLUG` against the whitelist above before network access
- rejects `--limit 0`, defaults omitted limit to 20, and caps larger limits at 20
- rejects invalid scan market, columns, sort field, order flags, and non-finite
  numeric filters before network access
- uses `reqwest` outside CDP, similar in spirit to `tv search`
- returns the Rust JSON envelope with payload under `data`
- includes practical fields such as `slug`, `limit`, `count`, `total_count`,
  `fields`, and normalized `symbols`
- scan payloads include `source: "scanner_scan_rest"`, `market`, `columns`,
  `sort`, `filters`, `total_count`, and normalized `symbols`
- avoids recording raw live scanner payloads in tracked docs

## Implemented UI Screener contract

The Rust implementation is separate from Hotlist REST and currently includes:

- `tv screener status`
- `tv screener open`
- `tv screener get [--limit <N>]`
- `tv screener screens active`
- `tv screener screens actions`
- `tv screener screens list`
- `tv screener screens switch --name <NAME> [--dry-run]`
- `tv screener screens save [--dry-run]`
- `tv screener filters list`
- `tv screener filters actions`
- `tv screener filters add --name <TEXT> --min <N>|--max <N> [--dry-run]`
- `tv screener filters modify --index <N>|--text <TEXT> --min <N>|--max <N> [--dry-run]`
- `tv screener filters modify --index <N>|--text <TEXT> --option <TEXT> [--dry-run]`
- `tv screener filters remove --index <N>|--text <TEXT> [--dry-run]`
- `tv screener filters clear [--dry-run] --confirm-clear`
- `tv screener columns list`
- `tv screener columns actions`
- `tv screener columns config`
- `tv screener columns add --id <COLUMN_ID> [--params-json <JSON>] [--after-index <N>] [--dry-run]`
- `tv screener columns remove --index <N>|--name <TEXT> [--dry-run]`
- `tv screener columns reorder --from-index <N> --to-index <N> [--dry-run]`
- `tv screener close`

`screens list` and `screens switch` are intentionally narrower than upstream PR
#66's stretch screen-management surface. By default they use entries visible in
the active screen title menu, such as recent screens, and return
`scope: "screen_title_menu"` to make that boundary explicit. With `--catalog`,
they use the saved-screen catalog and return `scope: "screen_catalog"`.
`screens actions` reads visible menu actions, and `screens save` clicks only the
exact visible `Save screen` / `スクリーンを保存` action after optional dry-run
target reporting. `columns actions` reads the visible column settings categories
and reports whether safe visible remove/reset actions are present. `columns
config`, `columns add`, `columns remove`, and `columns reorder` use the
saved-screen storage API instead of the visible column settings dialog; normal
add/remove/reorder are limited to test/disposable screen names and require
post-save storage order checks. `columns add` is intentionally id-based and
does not search a display-name catalog. The implementation does not include
column reset because the current evidence does not expose a reliable default
source.

The column reset feasibility pass on 2026-04-27 checked the full-page test
Screener target with read-only storage and DOM evidence. The active saved
screen exposed only the current custom column set under
`default_custom_column_set`; no separate default or preset column set was found
in `window.initData`, `screen_data`, or the fetched saved-screen response.
Visible column-management action discovery also reported `reset_supported:
false`. Keep reset deferred unless a future UI/storage build exposes a
post-checkable default source.
Live smoke on 2026-04-25 showed that menu-visible
entries were readable and dry-run targeting worked, but the current TradingView
Desktop session did not activate a clicked visible screen row; non-dry-run
switch therefore failed with `internal_api_unavailable` rather than reporting a
false success.

It depends on TradingView Desktop DOM selectors. Metadata reads return the
active screen title, visible filter pills, and visible table column names
without reading table rows.
Filter remove/clear commands operate on visible UI filter pills, not a stable
REST schema. `remove` resolves exactly one visible filter before clicking the
pill's popover remove button. `clear` is intentionally confirmation-gated
because it can remove every visible filter from the active screen.
`filters actions` now scopes numeric range options to the target filter popover;
for the current test screen, `EMA (21)` exposes `0% 〜 10%`, `10%以上`, and
`20%以上`, while `0% 〜 5%` belongs to a different `変動` filter. Normal
`filters modify` remains guarded by visible-text post-check and should be
treated as evidence-gated in live UI. The add-filter button opens a searchable
catalog. `filters add` is now implemented for visible numeric range presets
with dry-run candidate reporting and visible-pill post-checks. The first
non-numeric editing surface is now implemented as `filters modify --option` for
single visible option selection on an existing filter pill; multi-option
workflow semantics remain deferred.

The 2026-04-26 live evidence pass found that the upstream
`[class*="screenerContainer"]` selector did not match the current dialog. A Rust
implementation should use broader current-state indicators such as visible
Screener heading text, `[class*="screener"]`, visible Screener `data-name`
attributes, and table presence. The live session opened from
`[data-name="screener-dialog-button"]`; `Escape` closed and restored the
initial closed state.

## Historical CI note

While the original feasibility pass was running, the pushed `v0.1.1` Release
and CI runs were observed as failed. A later `fix(cli): Fix Windows CLI stack
overflow` push was observed with successful `CI` and `Release` runs for
`v0.1.1`.

## Assumptions

- This note does not implement any Rust code.
- MCP server implementation remains not planned.
- Stock Screener UI automation is not old CLI migration backlog; it is a future
  feature decision.
- Hotlist REST reads are still an undocumented TradingView endpoint dependency,
  even if the endpoint is public and no-auth.
