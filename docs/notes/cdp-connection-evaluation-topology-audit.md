# CDP connection and Runtime evaluation topology audit

Status: source audit completed on commit `b37d510`; focused independent audit
review pending. This note records source topology, not live performance.

## Scope and method

The audit covers production code under `crates/cli/src`. It inventories target
listing, selection, creation, activation, WebSocket connection, and
`Runtime.evaluate` ownership. Matches after a file's `mod tests` boundary are
excluded. The commands used were:

    rg -n "connect_runtime\(|CdpClient::connect|CdpHttpSession::|discover_target\(|fetch_targets\(|select_target\(|new_target_url\(|activate_target\(" crates/cli/src --glob '*.rs'
    rg -n "\.evaluate\(" crates/cli/src --glob '*.rs'

The evaluation inventory contains 134 production call sites in 44 files. The
transport inventory includes 75 dispatcher calls to `connect_runtime`, three
long-running runner calls, the two operations inside `connect_runtime`, and all
direct transport owners listed below.

## Normal single-owner dispatcher paths

Every `connect_runtime(config)` call in `crates/cli/src/app/dispatch.rs` creates
one selected-target connection for one chosen command arm and passes that same
mutable client through the operation. The complete production line inventory
at the audited commit is:

    51, 61, 147, 358, 525, 529, 544, 548, 563, 575, 590, 613, 617,
    624, 628, 643, 648, 658, 667, 677, 683, 698, 735, 746, 762, 780,
    790, 811, 822, 832, 899, 925, 929, 939, 949, 953, 959, 964, 968,
    972, 976, 982, 993, 1009, 1013, 1017, 1029, 1033, 1037, 1041,
    1045, 1049, 1057, 1061, 1065, 1079, 1085, 1090, 1094, 1104,
    1110, 1121, 1136, 1140, 1144, 1148, 1155, 1160, 1181, 1185,
    1261, 1389, 1394, 1398, 1401

Classification: `single_owner`. Cardinality: one discovery and one WebSocket
connection for the selected arm. Line 1401 is conditional chart-first quote
selection; failure may choose the existing scanner fallback, but it does not
open a second Desktop connection.

The stream, observe, and Replay-log runners connect once before their loops at
`app/stream.rs:26`, `app/observe.rs:33`, and `app/replay_log.rs:114`.
Classification: `single_owner`; cardinality: one connection per runner, not one
per sample or step.

`app/runtime.rs:5-6` owns the one discovery plus one WebSocket connection used
by all of these call sites. These two lower-level calls are implementation of
the dispatcher rows, not additional per-command connections.

## Direct transport owners

| Owner | Symbolic topology | Classification | Reason |
| --- | --- | --- | --- |
| `ops/readiness.rs::readiness` | one target list, zero or one chart connection | `single_owner` | One selected chart is reused for state and OHLCV readiness. |
| `ops/status.rs::status`, automatic target | one target list, zero or one chart connection | `single_owner` | Selection is performed from the fetched list. |
| `ops/status.rs::status`, explicit target | two target lists, one chart connection | `candidate_deferred` | `fetch_targets` is followed immediately by `discover_target`; the second read also revalidates target presence and preserves a separate selection failure boundary. |
| `ops/diagnostics.rs` | one target list, zero or one selected connection | `single_owner` | Connection diagnostics intentionally retain partial results. |
| `ops/desktop.rs::app_tabs_from_targets` | one app-window connection per supplied snapshot | `intentional_multi_target` | Chart targets cannot expose app-tab DOM. |
| `ops/desktop.rs::create_new_app_tab` | one fresh target list and one app-window connection | `intentional_multi_target` | The app window is the mutation owner. |
| `ops/desktop.rs::current_new_tab_target` / `wait_for_new_tab_target` | one target list per bounded attempt | `conditional_fallback` | These helpers are called by the Screener new-tab fallback and are grouped with that workflow below. |
| `ops/tab.rs::tab_list` | one target list and zero or one app-window connection | `intentional_multi_target` | Target metadata and app-tab DOM are separate sources. |
| `ops/tab.rs::tab_switch` / `activate_tab` | one target list and one activation request | `single_owner` | Activation is the requested operation, not redundant discovery. |
| `ops/tab.rs::tab_new` | before/after target lists, one app-window connection, one activation | `intentional_multi_target` | Before/after snapshots and app-window mutation verify tab creation. |
| `ops/tab.rs::tab_close` | before/after target lists and one app-window connection | `intentional_multi_target` | Both snapshots verify close completion. |
| `ops/screener/state.rs::screener_open_full_page` | initial list; conditional create; bounded target polling; conditional new-tab connection; final activation | `conditional_fallback` | Each repeated operation belongs to target creation, fallback, or bounded appearance verification. |
| `ops/launch.rs` | one pre-launch session and one post-launch session | `conditional_fallback` | The two sessions are separated by process launch/fallback and cannot share a target snapshot. |

The explicit-target `status` row is the only repeated transport candidate. The
first target snapshot is consumed by `desktop_readiness_summary`; it is not a
dead fetch. Removing the second target-list request would save one local HTTP
round trip, but it would also reuse that older snapshot and change the failure
point if the requested target disappears between reads. The completed transport
probe observed `target_list` p95 of 9 ms in one quiet run; that is not enough
evidence to trade freshness and error attribution for the small static call
reduction. The candidate is therefore deferred, not promoted.

## Runtime evaluation matrix

The matrix groups implementation call sites by file because several command
arms intentionally share one helper family. Counts are production
`.evaluate(...)` call sites, not runtime execution counts.

| File | Sites | Purposes and symbolic cardinality | Classification |
| --- | ---: | --- | --- |
| `ops/screener/screens.rs` | 15 | catalog/menu reads, UI mutations, storage mutation, post-check polling | preflight / mutation / verification / polling |
| `ops/screener/filters.rs` | 10 | storage writes, refresh/reload, option search/click, post-check polling | mutation / verification / polling |
| `ops/layout/watchlist.rs` | 10 | state read, panel open, keyboard-assisted mutation, API mutation, bounded wait | read / preflight / mutation / polling |
| `ops/chart.rs` | 9 | independent state reads and chart setters | single_read / mutation |
| `ops/pine/editor/compile.rs` | 6 | compile trigger, study-count pre/post checks, errors and console reads | preflight / mutation / verification / read |
| `ops/layout/pane.rs` | 5 | pane reads, layout/focus/symbol mutations and symbol readback | read / mutation / verification |
| `ops/chart/visible_range.rs` | 5 | initial inspection, bounded history request/inspection, range apply/readback | preflight / mutation / polling / verification |
| `ops/ui/dom.rs` | 4 | independent DOM query helpers | single_read |
| `ops/pine/editor/scripts.rs` | 4 | saved-script binding, save preflight and post-shortcut verification, list | preflight / mutation / verification / read |
| `ops/pine/editor/runtime.rs` | 4 | readiness inspection and bounded panel-open attempts | preflight / mutation / polling |
| `ops/indicator.rs` | 4 | metainfo resolution, insertion, immediate readback, cleanup | preflight / mutation / verification / cleanup |
| `ops/data/drawings.rs` | 4 | drawing collection and related reads | single_read |
| `ops/screener/state.rs` | 3 | dialog state, close verification, new-tab tile mutation | read / mutation / verification |
| `ops/screener/engine.rs` | 3 | state inspection with open/restore behavior | preflight / mutation / restoration |
| `ops/replay/control.rs` | 3 | Replay mutation and status verification | mutation / verification |
| `ops/drawing/create.rs` | 3 | inventory baseline, creation, identity/point verification | preflight / mutation / verification |
| `ops/desktop.rs` | 3 | app-tab reads and create/close mutations | read / mutation |
| `ops/data/strategy.rs` | 3 | strategy result reads from distinct surfaces | single_read |
| `ops/screener/columns.rs` | 2 | column mutation and verification | mutation / verification |
| `ops/saved_layout.rs` | 2 | layout read/switch and readback | read / mutation / verification |
| `ops/market/quote.rs` | 2 | selected-chart quote read and switch/restore evidence | read / mutation / restoration |
| `ops/drawing/read.rs` | 2 | drawing inventory and detail read | single_read |
| `ops/drawing/lifecycle.rs` | 2 | drawing mutation and verification | mutation / verification |
| `ops/diagnostics.rs` | 2 | chart and quote-data diagnostic reads | single_read |
| `ops/data/indicator.rs` | 2 | indicator values and metadata from the same study wrapper | single_read |
| `ops/alert/indicator.rs` | 2 | indicator resolution and alert mutation | preflight / mutation |
| `ops/alert/delete.rs` | 2 | deletion mutation and verification | mutation / verification |
| `ops/alert/create.rs` | 2 | creation preflight and mutation | preflight / mutation |
| `ops/ui/selectors.rs` | 1 | selector query | single_read |
| `ops/ui/input.rs` | 1 | focus/readiness around CDP input dispatch | preflight |
| `ops/ui/eval.rs` | 1 | explicit unsafe user evaluation | single_read |
| `ops/stream.rs` | 1 | study-value sample per configured loop iteration | polling |
| `ops/status.rs` | 1 | chart status read | single_read |
| `ops/screenshot/render_wait.rs` | 1 | bounded render-readiness polling | polling |
| `ops/screenshot.rs` | 1 | screenshot-region bounds read before capture | preflight |
| `ops/replay/trade.rs` | 1 | Replay trade mutation/readback expression | mutation / verification |
| `ops/replay/status.rs` | 1 | Replay status read | single_read |
| `ops/replay/autoplay.rs` | 1 | autoplay mutation/readback | mutation / verification |
| `ops/pine/editor/source.rs` | 1 | source read or set/new with internal verification | read / mutation / verification |
| `ops/market/ohlcv.rs` | 1 | selected-chart OHLCV read | single_read |
| `ops/data_depth.rs` | 1 | market-depth read | single_read |
| `ops/data/strategy_selection.rs` | 1 | Strategy Tester selection/readback | preflight / mutation / verification |
| `ops/data/shapes.rs` | 1 | shape read | single_read |
| `ops/alert/list.rs` | 1 | alert list read | single_read |

The 134 sites are fully represented by these 44 rows. Adjacent evaluations in
the high-count families were checked with their surrounding helpers. Input
dispatch through keyboard, text, or pointer CDP methods counts as an intervening
mutation even though it is not found by the `.evaluate(` inventory. Screenshot
capture similarly separates its bounds preflight from the capture operation.

No evaluation pair qualifies for promotion. The apparent repetitions belong
to separate command entry points, bounded polling, mutation and post-check,
restoration, cleanup, or different DOM/storage ownership. Combining them would
weaken failure attribution or change ordering; source call count provides no
contrary performance evidence.

## Outcome

Outcome: `candidate_deferred`.

No production topology change is justified by this audit. The explicit-target
`status` duplicate target listing remains a documented candidate, but its
freshness and error-attribution implications need a dedicated plan and measured
benefit before implementation. No Runtime-evaluation candidate survives.

This result does not promote retry, shared connection, broker, session,
recovery metadata, or a wait command. It also does not claim that current
topology is globally optimal; it establishes that no reviewed change belongs in
this source-audit slice.
