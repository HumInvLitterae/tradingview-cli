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

For user-facing command source names, use
`docs/command-source-taxonomy.md`. In this file, scanner REST and symbol-search
HTTP reads are Desktop-free reads, chart page objects and screenshots are
Desktop-backed reads, page-session account/storage changes are Desktop-backed
operations, `quote --source auto` is hybrid, and browserless WebSocket bars
are a stable bounded read that still uses an undocumented TradingView protocol.
The project keeps a single `tv` binary for now.

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
- `indicator add` resolves a case-sensitive exact public description from the
  selected chart's metainfo repository, uses the chart model's study inserter,
  and accepts success only when the first post-await inventory contains one new
  row with that name and the same chart-local ID resolves requested scalar
  inputs. It does not fall back to `createStudy` or indicator-dialog clicking.
- `readiness` is the first-line Desktop-backed readiness read. It aggregates
  CDP endpoint information, target handoff (`target_cli_args`), chart API
  readiness, and a one-bar OHLCV readiness check without mutating chart,
  account, or page state.
- `status`, `tab list`, `state`, and `ohlcv --count 1` remain lower-level
  follow-up reads when the aggregated readiness payload points to a specific
  endpoint, target, chart, or bars problem. These core Desktop-backed reads
  report source taxonomy metadata so downstream agents can keep Desktop-backed
  evidence separate from scanner REST reads.
- `quote <SYMBOL>` defaults to non-mutating scanner REST. `--source chart`
  explicitly chooses the selected TradingView Desktop chart feed, and
  `--source auto` is chart-first with scanner fallback only if chart access
  fails before any chart mutation. Chart switching must fail if the observed
  quote symbol or current chart symbol does not match the requested symbol. It
  must also wait for the chart bars backing the quote payload to reflect the
  requested symbol, require consecutive ready samples, retry that readiness
  wait once on timeout, and fail instead of reporting success if bars still
  look stale. When chart-source quote switches the visible chart, its payload
  reports `non_mutating: false` alongside `switch_performed`, `restored`, and
  `freshness_check`.
- chart-source quote reads the selected chart main-series last bar. It reports
  a `session_boundary` object with `price_session: "unknown"` and
  `extended_hours_status: "not_provided"` because this path does not expose
  scanner-style premarket or postmarket fields. Do not merge scanner
  `extended_hours` values into chart-source quote payloads.
- TradingView Desktop chart pages expose a page quote session through
  `window.getQuoteSessionInstance()` in current observed builds. Read-only
  probes showed that it supports temporary symbol subscriptions and can return
  selected fields such as `market-status`, `session-premarket`,
  `session-postmarket`, `premarket_close`, and `postmarket_close`. This is not
  the same source as chart main-series bars. During regular session, these
  pre/post fields can track current streaming values rather than scanner-backed
  extended-hours values. A postmarket probe observed `market-status.phase` as
  `post-market`, but the selected pre/post close fields matched each other and
  remained tied to quote-session streaming values, so treat them as
  experimental live evidence until both postmarket and premarket behavior are
  better understood.
- The visible right-side symbol detail panel is a separate Desktop UI surface.
  A postmarket RKLB probe showed that the visible after-market price can be
  extracted from the panel text while scanner REST, chart main-series quote,
  and the current quote-session selected field set report different values.
  A lower-level rerun narrowed the visible value to the detail widget's
  status/price nodes, with React metadata present on the matched node. This is
  source-discovery evidence only. A bounded CDP Network/WebSocket smoke
  observed symbol-related WebSocket traffic while the visible value was
  present, but did not find the visible after-hours price token in captured
  communication candidates. A scoped in-page widget inspection later found the
  right-panel detail widget React chain and regular quote-like props, including
  current-session and regular last-price fields, but did not expose the visible
  after-hours price token in compact prop/state hits. A bounded WebSocket
  correlation smoke later sampled visible after-market prices during the same
  capture window and found exact numeric matches in received WebSocket frame
  summaries. A follow-up Web TradingView HAR and live run narrowed the likely
  source further to `qsd` quote-data WebSocket messages: `lp` and
  `regular_close` remained the regular close-like value, while `rtc`,
  `rtc_time`, `rch`, and `rchp` changed with the visible postmarket readback.
  This makes `qsd.rtc` the strongest current candidate for the visible
  after-market display. Do not treat raw DOM, React props, or opportunistic
  Network frames as a stable API or merge the visible value into chart-source
  quote payloads. `tv quote <SYMBOL> --source quote-data` is the explicit
  Desktop-backed WebSocket quote-data readback surface for this source. It
  can carry `rtc`, `rtc_time`, `rch`, `rchp`, `current_session`,
  `market_phase`, and `update_mode` as source-specific readbacks rather than
  scanner-style extended-hours fields. Quote-data success payloads and
  unavailable details now carry `contract_version: "quote_data.v1"` and
  `source_availability`, so an agent can distinguish "no matching quote-data
  frame arrived" from "the symbol has no price" without reading raw frames.
  `source_availability.unavailable_reason` further classifies source
  diagnostics such as no WebSocket activity, no qsd messages, no matching
  symbol, or matching qsd without `rtc`. Success payloads include
  `quote_data.session_readback`, which normalizes TradingView-provided
  `market_phase` and `current_session` spelling only; it does not infer a
  session or convert quote-data into scanner `extended_hours`.
  Success payloads also include `quote_data.price_readback`, which labels
  whether the read came from `qsd.v.rtc` or regular quote-data `qsd.v.lp`.
  `regular_close` is carried as supporting source context when available, but
  it is not used alone as a price-readback success condition. This keeps
  regular-session quote-data behavior from being misclassified as a Desktop
  API prohibition while still avoiding scanner/chart/quote-data synthesis.
  `tv diagnose quote-data <SYMBOL>` wraps the same explicit source boundary
  in a troubleshooting packet: target selection, quote-data availability,
  public-safe WebSocket/qsd counts, and a separate scanner freshness
  reference. It must not include raw frames or synthesize a single quote from
  scanner and quote-data fields.
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
- stable scanner REST quote, batch quote, scanner table, hotlist, and
  metainfo payloads report `source_category: "desktop_free_read"`,
  `requires_desktop: false`, and `non_mutating: true`
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
  the observed quote symbol or current chart symbol does not match the
  requested symbol, or when the requested-symbol bars do not become fresh and
  stable within the bounded readiness wait.

## Scanner metainfo REST read

Category: unauthenticated TradingView scanner REST metadata read.

Current command family:

- `scanner metainfo [--market <MARKET>] [--field <FIELD>]...`

Safety boundary:

- this path is read-only and does not require a TradingView Desktop target
- output reports `source_category: "desktop_free_read"`,
  `requires_desktop: false`, and `non_mutating: true`
- it reads scanner field metadata, not prices, so quote freshness and
  real-time market-data entitlement are separate concerns
- the current CLI supports the same initial market boundary as `scanner scan`:
  `america`
- output is normalized to public-safe field summaries. The CLI does not expose
  raw metainfo payloads or a raw passthrough mode
- malformed or unexpectedly shaped responses should become
  `internal_api_unavailable`

## Scanner fundamentals REST read

Category: unauthenticated TradingView scanner REST read.

Current command family:

- `fundamentals <SYMBOL> [--group <GROUP>]... [--field <FIELD>]...`
- `events <SYMBOL> [--event-type all|earnings|dividends]`
- `scanner scan --columns ...` when callers request fundamental or earnings
  fields as table columns

Safety boundary:

- this path is read-only and does not require a TradingView Desktop target
- output reports `source_category: "desktop_free_read"`,
  `requires_desktop: false`, and `non_mutating: true`
- it reads scanner fundamental fields for a single resolved symbol and returns
  raw scanner values under `field_values`
- default fields are intentionally curated around symbol identity,
  sector/industry, market cap, valuation, EPS, dividend yield, and earnings
  date/time fields
- field groups are local scanner field bundles, not separate TradingView
  financial statement APIs; supported groups are `earnings`, `valuation`,
  `dividends`, and `financials`
- when groups and explicit fields are both used, group fields are expanded
  first, explicit fields are appended, and duplicate field names are removed
  while preserving order
- earnings date/time fields, such as `earnings_release_next_date`,
  `earnings_release_date`, and `earnings_release_next_time`, are returned as
  TradingView scanner values; the CLI does not infer timezone, before-market,
  or after-market semantics
- scanner metainfo exposed additional earnings and dividend-adjacent fields,
  and the confirmed subset is now included in the `earnings` and `dividends`
  groups. They are still scanner fields, not a complete TradingView event
  calendar. Public notes record field names and types only.
- `tv events` is a narrow `events.v1` view over scanner fundamentals earnings
  and dividends fields. It reports event type, date/time wording as returned
  by scanner fields, source availability, and missing/unavailable reasons, but
  it is not a complete TradingView event calendar and must not become a
  fallback for fundamentals, quotes, compare, bars, chart reads, or Replay.
- symbol no-row, exchange mismatch, ambiguity, or returned-symbol mismatch are
  validation errors with candidate symbols when possible; they do not fall
  back to chart state
- unknown requested fields fail before network access with supported field
  details
- do not add raw scanner payloads or account-local values to public payloads or
  tracked docs

## TradingView WebSocket bars research

Category: undocumented TradingView browserless WebSocket protocol.

Current command family:

- `bars <SYMBOL> --timeframe <TIMEFRAME> --count <N>`. This is a stable
  bounded Desktop-free historical bars read, not a replacement for
  selected-chart `ohlcv`.
- Rust does not currently expose browserless streaming commands.

Comparable evidence:

- fiale-plus PR #47 implements experimental historical bars and bounded quote
  or bar streaming through TradingView's WebSocket data protocol.
- The relevant design opens a WebSocket, sends an auth-token message, creates a
  chart session, resolves a symbol, creates a series, parses bar updates, and
  waits for completion or a bounded timeout.
- That design started as lab-gated evidence and treats the protocol as
  undocumented. It has an anonymous-token path, but also optional
  session-cookie-related configuration. Rust should therefore not treat it as
  equivalent to the credential-free scanner REST reads.
- The Rust lab prototype has been smoke-tested with bounded daily bars for
  public exchange-qualified symbols and an hourly request for a public
  exchange-qualified symbol. This is evidence that the path can work, not a
  guarantee that the undocumented protocol is stable.
- An opt-in ignored Rust live smoke exists to re-check the public `tv bars`
  JSON contract when needed. It should be treated as evidence tooling, not as a
  CI guarantee.
- Existing `tv stream ...` commands are not browserless WebSocket streams;
  they are Desktop-backed current-chart JSONL polling reads. Future
  observation work may improve their event contract or add browserless stream
  candidates, but the source boundary must remain explicit. Stream sample and
  heartbeat events, plus the final bounded-window summary event, currently
  report `source: "desktop_chart_stream"` and
  `source_category: "desktop_backed_read"`. Summary events describe counts,
  elapsed time, controls, and end reason; they are not market-data samples.
- `tv observe chart` is a workflow-level Desktop-backed observation command. It
  combines readiness, selected-chart last-bar observation, and a final summary
  event; it does not use the browserless WebSocket path and does not replace
  `tv stream ...` for specific stream sample types.

Safety boundary:

- classify Desktop-free historical bars as `desktop_free_read` with
  `source: "tradingview_bars_ws"` and `contract_version: "bars.v1"`, not as
  scanner REST or selected-chart bars
- the feasibility pass is complete and the Rust CLI now has a bounded stable
  command. It is still not equivalent to credential-free scanner REST because
  it uses an undocumented WebSocket chart-session protocol.
- do not add cookie/session import, login automation, or authenticated direct
  HTTP/WebSocket setup without a separate safety plan
- do not replace `tv ohlcv`; it reads current chart bars through the selected
  Desktop target
- `tv bars` is a separate symbol-targeted command and keeps requests bounded
  by count or by supported intraday, daily, weekly, or monthly date range with
  a count safety cap
- bare `tv bars` symbols are resolved by Desktop-free `symbol_search_rest`
  before the bars WebSocket request. Exchange-qualified input is used as-is.
  Payloads report `requested_symbol`, `resolved_symbol`, `symbol`, and
  `symbol_resolution` so callers can detect which TradingView symbol was used.
- `tv bars --from YYYY-MM-DD --to YYYY-MM-DD --timeframe 5|15|30|60|1D|1W|1M` is
  the reproducible historical-source preparation path for supported intraday,
  daily, and higher-timeframe samples. The `--to` value is an inclusive
  calendar date. In date-range mode, `--count` defaults to 500 and may be
  raised up to 5000 as a returned-bar safety cap. Recent count mode remains
  capped at 500. Other intraday timeframes remain guarded in date-range mode.
  `tv range` only changes the selected Desktop chart viewport and must not be
  treated as a hidden input to `tv ohlcv`.
- `tv bars` reports `summary` / `range` for requested-vs-returned count and
  time coverage, plus `requested_range` / `returned_range`,
  `range_coverage_status`, `range_alignment`, `range_fetch_summary`,
  `source_availability`, and a public-safe `wait_summary` for bounded source
  diagnostics. `range_alignment` states period-start timestamp semantics and
  the `timestamp_within_requested_range` filter policy for date ranges.
  `range_fetch_summary` reports bounded fetch-window count, `request_more_data`
  count, observed / filtered / returned counts, count-cap truncation, and
  timeout or source-exhaustion truncation reasons.
  `data_quality` still reports
  `realtime_guarantee: false`, `entitlement_checked: false`, completion state,
  elapsed time, and partial result readback. Callers should read those fields
  before treating raw `bars[]` as operational evidence.
- no-bars, timeout, WebSocket close/read failure, and protocol error details
  use public unavailable reasons such as `timeout_no_bars` or
  `websocket_read_failed`. They do not expose raw WebSocket frames, raw
  payloads, credentials, session ids, or account-local metadata.
- the stable command accepts resolved bare symbols but still does not add
  extended sessions, streaming, selected-chart fallback, or authenticated reads
- failures, malformed protocol frames, missing series completion, and symbol
  errors must become structured failures rather than empty successful bar lists
- keep evidence summaries high level. Do not write raw WebSocket frames,
  session ids, or live protocol payloads into tracked docs

## Symbol search REST read

Category: unauthenticated TradingView symbol search REST read.

Current command families:

- `search <QUERY>`
- `info <SYMBOL>` for Desktop-free symbol metadata reads

Safety boundary:

- this path is read-only and does not require a TradingView Desktop target
- `search` and symbol-targeted `info` payloads report
  `source_category: "desktop_free_read"`, `requires_desktop: false`, and
  `non_mutating: true`
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
  model and are not endpoint replacement priorities. `pine set` and `pine new`
  post-check source after normalizing CRLF, LF, and lone CR line endings because
  Monaco may normalize the buffer convention; non-line-ending source
  differences still fail closed.
- `pine compile`, `pine raw-compile`, and `pine save` use visible editor
  actions, keyboard shortcuts, dirty-state checks, and save/compile buttons.
  `pine save` dispatches Command+S on macOS and Control+S on Windows/Linux.
  Pine save preflight/post-shortcut Runtime evaluation failures are sanitized
  at the operation boundary, and malformed outcome/page-error diagnostics use
  a fixed whitelist rather than forwarding runtime payloads.
  Treat endpoint replacement as `research_only` unless a future plan proves a
  safe compile or save endpoint with the same editor/account semantics.

Safety boundary:

- saved script identifiers are account-linked metadata; `pine open` compares
  them in-page without returning them, and public docs must not contain their
  values
- `pine check` validates source without mutating the Pine Editor
- `pine open` resolves saved metadata through Pine facade, opens the selected
  script through the popup semantically linked to the visible Pine-owned
  saved-script trigger, and succeeds only when the same Save-bound store
  confirms internal identity, version, and public display name. Internal IDs
  are compared in-page; public output contains only non-identifying
  availability/verification booleans. The command does not save or compile and
  does not fall back to source-only Monaco replacement
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
- `screenshot --region strategy`: uses DOM only to compute the visible
  Strategy Tester / backtesting panel rectangle before CDP screenshot capture.
- Screenshot payloads report `source: "desktop_screenshot"`,
  `source_category: "desktop_backed_read"`, `requires_desktop: true`,
  `non_mutating: true`, `writes_file: true`, and `visual_evidence: true`.
  Bounds, crop, and file-write failures should return public-safe phase details
  and a recovery hint rather than raw DOM payloads.
- `screenshot --wait-for-render`: performs an opt-in bounded observation of
  selected-chart symbol, resolution, main-series last-bar time, pane and
  requested-region geometry, and known scoped loading state. It uses the
  current chart API and intentional screenshot DOM boundaries only; timeout
  captures and writes nothing.
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
