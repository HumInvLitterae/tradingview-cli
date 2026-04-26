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

## Replacement feasibility policy

When an existing command uses DOM selectors or visible button clicks, do not
automatically add retries. First classify whether the operation has a safer
non-public API, page-session object, or storage payload candidate.

Use these categories:

- `api_backed`: the command already uses a page object, endpoint, CDP target
  endpoint, or saved storage payload as its primary source.
- `replace_candidate`: a nearby implemented endpoint or storage shape suggests
  a better path may exist, but live read-only evidence is still required before
  changing behavior.
- `research_only`: a replacement might exist, but current value or safety does
  not justify implementation without a concrete workflow.
- `intentional_dom`: the command is supposed to inspect visible UI state,
  compute visible geometry, or preserve generic UI automation compatibility.

For any replacement, keep the Rust CLI rule: do not report success unless a
post-check proves the requested after-state. Account-state mutations need
dry-run where practical and guards against accidental production data changes.

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

Current implementation split:

- `alert list` and `alert delete` are `api_backed` through alert endpoints.
- `alert create` is currently a `replace_candidate`: it still uses the visible
  alert dialog and localized button labels, while the list/delete endpoint
  family proves nearby API surface exists. Do not replace it without first
  capturing public-safe live request-shape evidence and preserving the current
  practical fields.

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

Related DOM-backed Pine commands:

- `pine get/set/new/errors/console` intentionally use the local Monaco editor
  model and are not endpoint replacement priorities.
- `pine compile`, `pine raw-compile`, and `pine save` use visible editor
  actions, keyboard shortcuts, dirty-state checks, and save/compile buttons.
  Treat endpoint replacement as `research_only` unless a future plan proves a
  safe compile or save endpoint with the same editor/account semantics.

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

## Watchlist page-session API and DOM surface

Category: logged-in page-session watchlist API for mutations, plus visible
right-panel watchlist UI for readback.

Current command family:

- `watchlist get/add/add-bulk/remove`

Replacement classification:

- `watchlist get` is visible UI readback and may remain DOM-backed when the
  user wants the current visible watchlist.
- `watchlist add` and `watchlist remove` are API-backed account mutations when
  TradingView's logged-in symbols-list API is available for the active custom
  watchlist. They still verify presence or absence by re-fetching the active
  list before reporting success.
- `watchlist add-bulk` inherits the API-backed path because it calls the
  single-symbol add operation sequentially.
- DOM fallback remains for add/remove only when the API list or active list
  cannot be used before mutation. Post-check failures do not fall back.

Endpoint category:

- TradingView symbols-list API under the logged-in `www.tradingview.com`
  page session.
- Read shape: saved lists include custom and colored list records, active-list
  state, and symbol arrays.
- Mutation shape: append/remove accepts a symbol array against the active custom
  list, followed by a readback post-check.

Safety boundary:

- do not expose raw watchlist payloads, list ids, or live list names in tracked
  docs
- normal add/remove must still verify the symbol's presence or absence after
  mutation
- bulk add must preserve per-symbol result reporting and partial-success policy
- broader watchlist list/switch/create/rename/delete commands remain future
  feature research with separate safety requirements

## Screener page-session storage API

Category: logged-in page-session saved Screener storage endpoint discovered
from `window.initData`.

Current command families:

- `screener screens delete`
- `screener columns config/add/remove/reorder`
- `screener filters remove/clear`

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
- full-page Screener targets may be refreshed after storage-backed filter
  writes so the visible UI catches up with saved storage
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
- `screener filters remove/clear`

High-value storage/API audit candidates:

- `screener filters add`
- `screener filters modify --min/--max`
- `screener filters modify --option`
- `screener screens create/rename/save-as/save/switch`
- `screener columns reset`

Likely DOM-maintained boundaries:

- visible row reads
- visible filter and column display-text reads
- UI-only action discovery

The next Screener stabilization work should prefer storage/API evidence before
adding more DOM retries.

## App-tab DOM surface

Category: TradingView Desktop app-window tab strip visible in the
`/app/window/index.html` CDP target.

Current command family:

- `tab list/switch/new/close`

Replacement classification:

- `tab switch` is `api_backed` through the CDP target activation endpoint for
  chart targets.
- `tab new` and `tab close` are `research_only` replacement candidates. They
  currently click the app-window tab strip and verify tab-count changes. A
  non-DOM application command may exist, but the current code does not expose
  one.

Safety boundary:

- `tab close` must continue refusing to close the final app tab
- do not replace app-tab DOM operations without an exact target and post-count
  verification path

## Intentional DOM boundaries

These command families currently should stay DOM-backed unless new evidence
changes the boundary:

- `data depth`: reads the visible Depth of Market / DOM panel. No structured
  source is known.
- `screenshot --region chart`: uses DOM only to compute the visible chart
  rectangle before CDP screenshot capture.
- strategy DOM fallbacks: read currently rendered Strategy Tester rows only
  when chart-model report data is unavailable.
- generic `ui` commands: compatibility automation by definition; prefer
  higher-level commands rather than turning this into a broader API layer.
- diagnostic UI-state reads: intentionally summarize rendered panels and
  buttons for troubleshooting.

## Cross-command replacement priorities

Recommended order for future API/storage replacement work:

1. Alert create endpoint evidence.
2. Screener filters add/modify storage schema evidence.
3. Screener screen create/rename/save-as/save/switch storage or command
   evidence.
4. App-tab new/close non-DOM command evidence.

Do not start with `data depth`, chart screenshots, or generic UI automation;
their current DOM dependency is part of their observable contract.
