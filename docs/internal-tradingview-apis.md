# Internal TradingView API reference

This document records the non-public TradingView surfaces that the Rust-native
`tv` CLI depends on.

These are not official TradingView APIs. They may change without notice. The
CLI uses them only inside the user's own running TradingView Desktop session or
through public TradingView web endpoints already used by the app. The CLI does
not bypass access controls, does not embed credentials, and should report
`internal_api_unavailable` rather than guessing when these surfaces disappear or
return an unexpected shape.

This is not an integration guide for third-party callers. Do not add session
credentials, auth headers, account-linked identifiers, full raw payloads, or
copy-paste mutation recipes to this file.

## Documentation boundary

It is acceptable to document:

- the category of API or page object
- which `tv` commands depend on it
- whether the dependency is read-only or mutating
- the validation and post-check boundary
- when failures must become `internal_api_unavailable`

It is not acceptable to document:

- session credentials, auth headers, or token values
- account-linked saved screen, alert, script, layout, or watchlist ids
- raw request or response payloads copied from a live account
- personal script, screen, alert, watchlist, or layout names
- instructions that imply access-control bypass

## Page-session chart API

Category: private page object exposed in the TradingView Desktop page.

Known entrypoints:

- active chart widget
- chart widget collection
- main series bars collection

Current command families:

- chart reads and mutations: `status`, `state`, `info`, `quote`, `ohlcv`,
  `range`, `scroll`, `symbol`, `timeframe`, `type`
- pane and layout operations: `pane list/layout/focus/symbol`
- indicator operations: `indicator add/remove/toggle/set`
- drawing operations: `draw shape/list/get/remove/clear/position`
- data reads: `data indicator/strategy/trades/equity/lines/labels/tables/boxes/shapes`

Safety boundary:

- user input must be serialized into JavaScript, not hand-escaped
- mutation commands must verify the observable after-state before returning
  success
- missing objects or changed method names should become
  `internal_api_unavailable`

## Replay page API

Category: private page object for TradingView Replay state and controls.

Current command family:

- `replay start/step/stop/status/autoplay/trade`

Safety boundary:

- commands require visible replay API state before acting
- unsupported or missing methods become `internal_api_unavailable`
- replay mutation is chart-local UI state, not a durable account object

## Saved chart layout API

Category: private page object for saved chart layouts.

Current command family:

- `layout list/switch`

Safety boundary:

- reads preserve error payloads rather than pretending an empty account state
- switching is exact-target and post-checked
- the CLI does not dismiss unsaved-layout dialogs automatically

## Alert REST endpoints

Category: page-session REST calls to TradingView alert endpoints.

Current command family:

- `alert list/create/delete`

Safety boundary:

- reads preserve endpoint error details with an empty list when appropriate
- deletes support dry-run where applicable and require post-delete absence
- bulk account mutation must remain explicit and guarded
- do not record live alert ids in tracked docs

## Pine facade endpoints

Category: TradingView Pine service endpoints called either from the page session
or directly when the operation does not require the editor.

Current command family:

- `pine list/open/check`

Safety boundary:

- saved script identifiers are account-linked metadata and must not appear in
  public docs
- `pine check` validates source without mutating the Pine Editor
- `pine open` loads a script into the local editor buffer but does not save or
  compile
- malformed or unavailable responses should become validation or
  `internal_api_unavailable` errors, depending on whether the user input or the
  endpoint shape is at fault

## Scanner and symbol HTTP endpoints

Category: TradingView HTTP reads that do not require CDP for current use.

Current command families:

- `search`
- `scanner hotlist`
- `scanner scan`

Safety boundary:

- these commands are read-only
- supported markets and field names are intentionally explicit
- unexpected response shapes are rejected rather than normalized by guesswork

## Screener page-session storage API

Category: logged-in page-session saved Screener storage endpoint discovered
from `window.initData`.

Current command families:

- `screener screens delete`
- `screener columns config/add/remove/reorder`

Observed high-level shape:

- the full-page Screener target exposes storage URL, storage release version,
  standalone Screener type, and `screen_data`
- active `screen_data` includes screen metadata, active view mode, active
  column set, custom column set, filters, sort metadata, market settings, and
  watchlists
- active saved screen fetch returns a matching high-level shape with column,
  filter, watchlist, sort, view, market, id, title, and version fields

Safety boundary:

- mutation is limited to prepared test or disposable screen names when the
  command edits saved account state
- storage writes must be followed by a re-fetch and exact post-check
- commands must not write raw storage payloads or account-linked ids to tracked
  docs
- missing storage init data, failed fetches, failed saves, or failed post-checks
  become `internal_api_unavailable`

## Screener DOM and UI surfaces

Category: visible TradingView Screener UI queried or clicked through CDP.

Current command families:

- `screener status/open/get/close`
- `screener screens active/actions/list/switch/save/create/rename/save-as`
- `screener filters list/actions/add/modify/remove/clear`
- `screener columns list/actions`

Safety boundary:

- reads may open the Screener UI and then restore the previous open/closed state
- UI mutation commands support dry-run where practical
- visible-text and visible-count post-checks are required before success
- stale popovers and localized labels are expected fragility points

## Current Screener stabilization classification

Storage/API-backed today:

- `screener screens delete`
- `screener columns config/add/remove/reorder`

High-value storage/API audit candidates:

- `screener filters add`
- `screener filters modify --min/--max`
- `screener filters modify --option`
- `screener filters remove`
- `screener filters clear`
- `screener screens create/rename/save-as/save/switch`
- `screener columns reset`

Likely DOM-maintained boundaries:

- visible row reads
- visible filter and column display-text reads
- UI-only action discovery

The next Screener stabilization work should prefer storage/API evidence before
adding more DOM retries.
