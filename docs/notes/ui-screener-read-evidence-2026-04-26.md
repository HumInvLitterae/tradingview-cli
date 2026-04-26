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
- `tv screener screens create --name <NAME> [--dry-run]`
- `tv screener screens rename --name <CURRENT> --to <NEW> [--dry-run]`
- `tv screener screens save-as --name <NAME> [--dry-run]`
- `tv screener screens delete --name <NAME> [--dry-run] --confirm-delete`
- `tv screener filters list`
- `tv screener filters actions`
- `tv screener filters add --name <TEXT> --min <N>|--max <N> [--dry-run]`
- `tv screener filters modify --index <N>|--text <TEXT> --min <N>|--max <N> [--dry-run]`
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

Later hardening narrowed that detection to a visible, in-viewport Screener panel
root. The right-toolbar Screener button, unrelated right-panel content, and
off-viewport tables no longer count as `open: true`.

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

The filter modify follow-up found the active test screen with 17 visible
filters. A reliability pass found that early action discovery could accidentally
read the `変動` filter's `0% 〜 5%` option while targeting `EMA (21)`. The
implementation now scopes range-option discovery to the target filter popover.
For the current test screen, `EMA (21)` exposes `0% 〜 10%`, `10%以上`, and
`20%以上`; `0% 〜 5%` is not an `EMA (21)` option. `tv screener filters modify`
is still implemented defensively with preset validation and a visible-text
post-check. One live manual mutation from `0% 〜 10%` to `10%以上` succeeded and
the filter was restored to `0% 〜 10%`, but repeated CLI normal mutation was not
reliable enough to treat as fully stable. Dry-run and `filters actions` are the
reliable parts of this surface; normal modify must fail with
`internal_api_unavailable` rather than claiming success when the visible pill
does not change.

The filter add follow-up confirmed that the add-filter button opens a searchable
catalog whose input responds to real inserted text. Selecting `RSI` opened a
numeric preset list, selecting `> 70` added a visible `RSI (14)` filter pill,
and the test filter was removed afterward with `tv screener filters remove`.
Rust now exposes this as guarded `tv screener filters add --name <TEXT>
--min <N>|--max <N> [--dry-run]`: dry-run resolves the catalog candidate without
mutation, normal mode clicks the candidate and range preset through CDP mouse
events, and success requires a new visible filter pill.

The screen lifecycle follow-up found active screen menu actions for save, copy,
rename, CSV download, create, recent, and open. The current UI exposes name
dialogs for create, rename, and copy/save-as; Rust now exposes guarded
`screens create`, `screens rename`, and `screens save-as` commands with dry-run
dialog reporting, test-name validation for normal mutations, and active-title
post-checks before success. Delete is intentionally narrower: Rust resolves an
exact saved-screen catalog target in dry-run mode, but normal delete returns
`internal_api_unavailable` until a safe exact-screen delete action and
confirmation path are verified. One disposable live screen named
`CLI-Test-Codex-426A` was created during smoke and remained visible because
normal delete was not verified.

## Still deferred

- normal screen delete
- column add / normal remove / reorder / reset
- generic non-numeric filter editing
- workflow scanner rules, dashboards, or downstream strategy packs

These surfaces need separate safety policy and workflow evidence before they
belong in the core Rust CLI.

## Validation

The original read-only live smoke changed only visible UI state and restored the
Screener dialog to its initial closed state. No TradingView layout, watchlist,
alert, drawing, Pine script, filter, screen, or column setting was intentionally
modified during that read evidence pass.
