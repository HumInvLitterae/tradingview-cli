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
- `quote <SYMBOL>` defaults to non-mutating scanner REST. `--source chart`
  explicitly chooses the selected TradingView Desktop chart feed, and
  `--source auto` is chart-first with scanner fallback only if chart access
  fails before any chart mutation. Chart switching must fail if the observed
  quote symbol does not match the requested symbol.
- `ohlcv` depends on the selected chart target's main-series bars collection.
  When the chart API or bars collection is unavailable, it should fail with
  structured readiness details and a target-selection recovery hint rather than
  reporting stale or empty chart data as success.
- missing objects or changed method names should become
  `internal_api_unavailable`

## Scanner REST quote read

Category: unauthenticated TradingView scanner REST read.

Current command family:

- `quote <SYMBOL>` and `quote <SYMBOL> --source scanner` for symbol-targeted
  scanner reads without CDP
- `quote <SYMBOL> --source auto` as chart-first compatibility mode with scanner
  fallback only for pre-mutation chart unavailability
- `quotes <SYMBOL>...` for ordered Desktop-free batch quote reads
- `scanner scan --columns ...` for broader scanner-table reads with explicit
  fields

Safety boundary:

- this path is read-only and does not require a TradingView Desktop target
- price-bearing scanner REST reads are not a realtime entitlement guarantee;
  freshness can depend on exchange rules, TradingView feed selection, and
  market-data subscription state
- it returns scanner quote fields such as symbol, description, close, open,
  high, low, volume, change, exchange, type, and subtype
- scanner quote payloads also include `time`, `update_mode`, and
  `delay_seconds`. `time` is TradingView's returned quote timestamp when
  present. `delay_seconds` is parsed only from clearly shaped modes such as
  `delayed_streaming_900`; unknown or missing modes remain `null`.
- it also requests TradingView scanner extended-hours columns when available:
  `premarket_open`, `premarket_high`, `premarket_low`, `premarket_close`,
  `premarket_change`, `premarket_change_abs`, `premarket_gap`,
  `premarket_volume`, `postmarket_open`, `postmarket_high`,
  `postmarket_low`, `postmarket_close`, `postmarket_change`,
  `postmarket_change_abs`, and `postmarket_volume`
- extended-hours values are returned as a nested `extended_hours` object.
  Missing or inactive-session values remain `null`; the top-level `last` and
  `close` fields are not replaced by premarket or postmarket values
- `quotes <SYMBOL>...` returns ordered `items[]`; each successful item embeds
  the same quote shape as `quote <SYMBOL>`, and each failed item embeds a
  public-safe structured error for the requested symbol
- when the same extended-hours columns are requested through `scanner scan`,
  they remain table fields under each symbol row's `field_values` object rather
  than being reshaped into a nested object
- the current scanner REST watchlist-style read lane is sufficient for known
  practical needs: single quote, ordered batch quote, scanner table scan,
  explicit extended-hours columns, and metainfo field discovery. Additions
  should be driven by a concrete operator workflow and endpoint evidence rather
  than broad field harvesting.
- scanner validation failures, missing rows, ambiguous rows, and returned
  symbol mismatches are symbol-resolution failures. They do not trigger chart
  fallback, including in `--source auto`.
- `--source chart` and the chart side of `--source auto` must still fail when
  the observed symbol does not match the requested symbol.

## Scanner metainfo REST read

Category: unauthenticated TradingView scanner REST metadata read.

Current command family:

- `scanner metainfo [--market <MARKET>] [--field <FIELD>]...`

Safety boundary:

- this path is read-only and does not require a TradingView Desktop target
- it reads scanner field metadata, not prices, so quote freshness and
  real-time market-data entitlement are separate concerns
- the current CLI supports the same initial market boundary as `scanner scan`:
  `america`
- output is normalized to public-safe field summaries. The CLI does not expose
  raw metainfo payloads or a raw passthrough mode
- malformed or unexpectedly shaped responses should become
  `internal_api_unavailable`

## TradingView WebSocket bars research

Category: undocumented TradingView browserless WebSocket protocol.

Current command family:

- none. Rust does not currently expose Desktop-free historical bars or
  browserless streaming commands.

Comparable evidence:

- fiale-plus PR #47 implements experimental historical bars and bounded quote
  or bar streaming through TradingView's WebSocket data protocol.
- The relevant design opens a WebSocket, sends an auth-token message, creates a
  chart session, resolves a symbol, creates a series, parses bar updates, and
  waits for completion or a bounded timeout.
- That design is explicitly lab-gated and treats the protocol as experimental.
  It has an anonymous-token path, but also optional session-cookie-related
  configuration. Rust should therefore not treat it as equivalent to the
  credential-free scanner REST reads.

Safety boundary:

- classify Desktop-free historical bars as `research_candidate`, not
  `api_backed`
- the feasibility pass is complete for now, but no stable Rust CLI command has
  been implemented. This is a deferred research boundary, not a completed
  feature and not a canceled idea.
- do not add cookie/session import, login automation, or authenticated direct
  HTTP/WebSocket setup without a separate safety plan
- do not replace `tv ohlcv`; it reads current chart bars through the selected
  Desktop target
- if this becomes a Rust feature, prefer a separate lab-gated symbol-targeted
  command and keep requests bounded by count or duration
- failures, malformed protocol frames, missing series completion, and symbol
  errors must become structured failures rather than empty successful bar lists

## Symbol search REST read

Category: unauthenticated TradingView symbol search REST read.

Current command families:

- `search <QUERY>`
- `info <SYMBOL>` for Desktop-free symbol metadata reads

Safety boundary:

- this path is read-only and does not require a TradingView Desktop target
- `info <SYMBOL>` resolves exchange-qualified input strictly; bare input uses
  TradingView's search ordering and returns the first exact symbol match
- the command returns practical metadata such as symbol, full name, exchange,
  description, and type
- missing or exchange-mismatched inputs are validation errors and should include
  candidate symbols when available
- `info` without a symbol is still the current-chart metadata command and uses
  the page-session chart API

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

- `alert list`, `alert create`, and `alert delete` are `api_backed` through
  alert endpoints.
- `alert create` reads active chart metadata from the page session, submits the
  create request through the logged-in alert endpoint, and confirms the new
  alert through a list readback before reporting success.
- `alert create` sends its JSON as a plain string request body with no custom
  `Content-Type` header. Adding custom headers can trigger a rejected
  cross-origin preflight in TradingView's page context.
- `alert delete` uses the bare delete endpoint shape, sends numeric alert ids
  as numbers, and verifies absence after mutation.

Safety boundary:

- reads preserve endpoint error details with an empty list when appropriate
- creates and deletes require post-mutation readback before success
- create only falls back to visible dialog automation if the API path fails
  before the create request is sent
- post-create ambiguity must not trigger DOM fallback, because retries can
  create duplicate alerts
- deletes support dry-run where applicable and require post-delete absence
- alert list/create/delete payloads sanitize condition details and must not
  expose raw Pine series, saved-script identifiers, input maps, or endpoint
  payloads
- bulk account mutation must remain explicit and guarded
- do not record live alert ids in tracked docs

Indicator alertcondition alerts:

- Upstream PR #112 shows that Pine `alertcondition()` alerts can likely be
  created through the same alert endpoint family by referencing saved Pine
  script metadata and a plot-like alert condition id.
- Rust now has the first safe discovery building block:
  `tv pine alertconditions [--file <PATH>]` scans local Pine source and reports
  best-effort `alertcondition()` candidates such as `plot_1`. It does not use
  TradingView account metadata, does not connect to CDP, and does not create
  alerts.
- Rust also has a guarded create/preview command:
  `tv alert create-indicator --script <NAME> --file <PATH>
  --condition-title <TITLE>|--alert-cond-id <ID> [--dry-run]`. It combines a
  local static candidate with an exact saved-script display-name match from the
  logged-in Pine facade list. Dry-run returns a sanitized preview. Normal mode
  creates through the alert endpoint only when required saved-script and input
  metadata can be resolved safely, then verifies the new alert through a list
  readback before reporting success.
- Raw indicator-alert endpoint primitives remain intentionally unexposed. The
  CLI does not ask users for saved script ids, raw Pine input payloads, raw plot
  offsets, or webhook fields in this initial surface.
- If Pine `input.*` declarations are present and a matching active chart study
  does not expose input values, normal creation must fail before the create
  request is sent.
- Do not document raw request bodies, saved script ids, webhook URLs, or copied
  alert payloads for this surface.

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
- `scanner metainfo`
- `scanner scan`
- `info <SYMBOL>`
- `quote <SYMBOL>` before chart fallback
- `quotes <SYMBOL>...`

Safety boundary:

- these commands are read-only
- scanner price reads are useful for screening but are not guaranteed to be
  realtime for every exchange or subscription state
- supported markets and field names are intentionally explicit
- unexpected response shapes are rejected rather than normalized by guesswork

Direct HTTP feasibility status:

- `search`, `scanner hotlist`, `scanner scan`, symbol-targeted `info`,
  symbol-targeted `quote`, and `pine check` are the current credential-safe
  direct HTTP reads.
- No additional direct HTTP command candidate is selected from the first
  `v0.3.0` feasibility pass.
- Future candidates need a concrete read-only operator need, endpoint evidence,
  and no requirement to copy browser credentials, session state, or
  account-linked identifiers.

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

- `screener open --full-page` reuses an existing full-page Screener tab and
  returns `target_cli_args`. It attempts the local CDP target creation endpoint
  first, but current live evidence shows TradingView Desktop may reject that
  path with `Could not create new page`. When that happens, the CLI uses a
  bounded Desktop new-tab fallback: create or reuse the `new-tab` page target,
  click the Stock Screener tile, and report success only after a full-page
  Screener target appears. This is not a TradingView account API; it only
  manages local Desktop page targets.
- `screener screens delete`
- `screener columns config/add/remove/reorder`
- `screener filters modify --min/--max` for simple saved-storage `Condition`
  filters selected by index
- `screener filters remove/clear`

High-value storage/API audit candidates:

- `screener filters add`
- `screener filters modify --option`
- `screener screens create/rename/save-as/save/switch`
- `screener columns reset`

Likely DOM-maintained boundaries:

- visible row reads
- visible filter and column display-text reads
- UI-only action discovery

The next Screener stabilization work should prefer storage/API evidence before
adding more DOM retries.

2026-04-29 bounded audit result: a full-page Screener target exposes enough
saved-screen filter schema to storage-back `filters modify --min/--max` for
simple `Condition` filters selected by index. The implementation rewrites only
the saved filter's `operation` and `right` range fields, saves the active screen,
and succeeds only after a storage re-fetch matches the expected payload.
Unsupported filter schemas, text selectors, missing storage init data, and
pre-save storage unavailability fall back to the existing UI-backed path.
Post-save post-check failures do not fall back to UI. `filters add` and
`filters modify --option` remain UI-backed because no safe catalog or option
value source has been proven for constructing those raw storage payloads.

## CDP transport boundary

Category: local Chrome DevTools Protocol endpoint exposed by TradingView
Desktop.

Current command family:

- chart, tab, screenshot, Pine, drawing, replay, data, and UI commands that
  need the running desktop session

Compatibility notes:

- The default endpoint host is `127.0.0.1`; `TV_CDP_HOST` and `TV_CDP_PORT`
  remain available for explicit local overrides.
- CDP methods are called directly when needed. The client does not send
  initial `Runtime.enable`, `Page.enable`, or `DOM.enable` during connection
  because recent TradingView Desktop / Electron builds can hang on those
  bootstrap calls while still accepting the direct methods used by this CLI.
- TradingView Desktop app-window targets are useful for app-tab operations and
  diagnostics, but they are not treated as automatic chart API targets.

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

The first high-value replacement candidates have been addressed:

- `watchlist add/remove` now prefer the logged-in symbols-list API.
- `alert create` now prefers the alert endpoint and requires alert-list
  readback.
- Screener storage is already used for screen delete, filter remove/clear, and
  column config/add/remove/reorder.

Remaining replacement work is evidence-gated rather than urgent:

1. Screener filters add/modify storage schema evidence.
2. Screener screen create/rename/save-as/save/switch storage or command
   evidence.
3. App-tab new/close non-DOM command evidence.

`docs/plans/archives/tradingview-cli-direct-http-feasibility.md` records the
first direct HTTP feasibility pass. That work prefers credential-safe read-only
endpoints and does not move account mutations away from the user's logged-in
page session without a separate safety plan.

Do not start with `data depth`, chart screenshots, or generic UI automation;
their current DOM dependency is part of their observable contract.
