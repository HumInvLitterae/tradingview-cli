# Architecture

This document records the stable architecture boundary for the Rust-native
`tv` CLI.

## Product boundary

`tv` is a CLI-first Rust binary for operating the user's own local TradingView
Desktop session. It is not an MCP server, and this repository does not plan to
implement one.

Downstream tools should invoke `tv` as a normal process and consume structured
JSON output. The old JavaScript CLI command surface has been migrated for the
known commands, but the Rust CLI is not a JSON wire-format clone.

## Runtime model

Most chart and UI commands connect to TradingView Desktop through Chrome
DevTools Protocol on `127.0.0.1:9222`.

The CLI uses three kinds of TradingView surface:

- public or browser-accessible HTTP reads, such as scanner reads
- logged-in page-session endpoints and storage payloads, such as alert,
  watchlist, and Screener saved-screen operations
- visible DOM or page object interaction when the command is intentionally
  about visible UI state or when no safer API/storage path is known

CDP method calls are issued directly when needed. The client does not send
initial `Runtime.enable`, `Page.enable`, or `DOM.enable` calls during
connection because recent TradingView Desktop / Electron builds can hang on
those enable calls while still accepting `Runtime.evaluate`,
`Page.captureScreenshot`, and `Input.*` methods.

Before adding more DOM retries, check whether a page-session API, storage
payload, or endpoint can replace the visible UI path. The public-safe reference
for these dependencies is `docs/internal-tradingview-apis.md`.
For user and agent workflow guidance, `docs/command-source-taxonomy.md`
classifies commands as Desktop-free reads, Desktop-backed reads,
Desktop-backed operations, hybrid commands, or specialized browserless reads.
The project keeps one `tv` binary for now; it does not split Desktop-free and
Desktop-backed functionality into separate executables.
`docs/operation-adapter-boundaries.md` records which remaining operation
families intentionally stay in `ops`, which are API/storage replacement
candidates, and which kinds of logic belong in `tradingview-model` or the
Desktop-free service crates.
`docs/rust-api.md` records the current reusable Rust API boundary for internal
workspace crates.

## Crate boundary

The repository is now a Cargo workspace. The root `Cargo.toml` is a virtual
workspace manifest, meaning it has no package of its own. The CLI package lives
under `crates/cli/`, its package name remains `tradingview-cli`, and the
installed binary remains `tv`.

- `crates/cli/src/lib.rs` is the `tradingview-cli` package's library crate
  root. It exposes current internal modules so the binary can share a single
  module tree and future refactors can extract reusable pieces incrementally.
- `crates/cli/src/main.rs` is the `tv` binary entrypoint. It owns only process
  startup and the Windows larger-stack wrapper before calling the library-owned
  application runner.
- `crates/cli/src/app.rs` is the CLI package's application-layer facade. It
  owns CLI parsing, command dispatch, runtime connection orchestration,
  stdin/file input conversion, stream looping, safety gates, and success/error
  envelope output.
- `crates/core/src/lib.rs` owns the shared contract types: `AppError`,
  `ErrorKind`, `SuccessEnvelope`, `ErrorEnvelope`, and `ErrorBody`. These are
  intentionally small and dependency-light so future internal crates can share
  JSON envelope and exit-code semantics without depending on the full CLI.
- `crates/model/src/lib.rs` owns the shared I/O-free model and policy layer:
  request interpretation, validation, selector and target resolution,
  normalization, public-safe payload shaping, and fallback policy decisions. It
  depends on `tradingview-core` but does not depend on clap, CDP, HTTP clients,
  page-session execution, or UI automation.
- `crates/market/src/lib.rs` owns Desktop-free market reads for symbol search,
  symbol metadata, symbol quote lookup, and browserless historical bars. It
  exposes typed read results where those are intentionally reusable and keeps
  JSON-returning wrappers for CLI payload contracts such as `bars.v1`. It uses
  credential-free TradingView endpoints and does not depend on CDP, chart
  state, or UI automation.
- `crates/scanner/src/lib.rs` owns Desktop-free scanner reads for hotlists and
  basic scanner scans. It exposes typed read results as the reusable Rust API
  and keeps JSON-returning wrappers for the CLI payload contract. It uses
  credential-free scanner HTTP endpoints and does not depend on CDP, chart
  state, or UI automation.
- `crates/pine/src/lib.rs` owns Desktop-free Pine helpers for local static
  analysis, `alertcondition()` discovery, and Pine facade checks. Pine Editor
  operations remain in the root CLI crate because they depend on CDP.
- `crates/cdp/src/lib.rs` owns the TradingView Desktop connection substrate:
  CDP target discovery, explicit target selection, WebSocket method calls,
  runtime evaluation, screenshot capture, and input events.

These library crates are internal and unstable for now. Treat them as
maintainability boundaries until a future ExecPlan explicitly marks types or
functions as stable for downstream Rust callers. For the currently documented
reuse surface, see `docs/rust-api.md`.

## Rust module responsibilities

Keep responsibilities separated:

- `crates/cli/src/lib.rs` owns top-level module declarations for the CLI
  package.
- `crates/cli/src/main.rs` owns startup for the `tv` binary and delegates
  application behavior to `tradingview_cli::app`.
- `crates/cli/src/app.rs` and `crates/cli/src/app/` own application
  orchestration between the CLI surface and operation adapter modules.
- `crates/cli/src/cli.rs` owns the `clap` command surface, argument
  definitions, and command names.
- `crates/core/src/lib.rs` owns shared typed errors, JSON success/error
  envelopes, and error exit-code mapping.
- `crates/model/src/lib.rs` owns the shared model modules previously kept under
  the CLI package's in-package `domain` layer. `tradingview_model::watchlist`
  owns watchlist symbol normalization, bulk-add aggregation, and public payload
  normalization. `tradingview_model::alert` owns alert condition validation,
  public-safe alert payload normalization, sanitization, and API fallback
  policy. `tradingview_model::replay` owns replay date/speed/action validation,
  replay timestamp conversion, and replay action/status payload normalization.
  `tradingview_model::drawing` owns drawing request structs, direction parsing,
  override parsing, and position price validation. `tradingview_model::screener`
  owns Screener validation, target resolution, and storage payload shaping.
- `crates/market/src/lib.rs` owns direct HTTP implementations behind
  `tv search <QUERY>`, `tv info <SYMBOL>`, the default
  `tv quote <SYMBOL>` scanner source, and `tv quotes <SYMBOL>...`.
- `crates/scanner/src/lib.rs` owns direct HTTP implementations behind
  `tv scanner hotlist` and `tv scanner scan`.
- `crates/pine/src/lib.rs` owns Desktop-free Pine static analysis,
  `alertcondition()` discovery, and `tv pine check` support.
- `crates/cdp/src/lib.rs` owns Chrome DevTools Protocol evaluation,
  screenshot/input primitives, and TradingView CDP target discovery. `tv
  --target-id <CDP_TARGET_ID>` is the primary explicit target selection path.
- `crates/cli/src/ops.rs` is a thin facade that declares operation adapter
  modules and re-exports operation functions used by the application dispatch
  layer.
- `crates/cli/src/ops/desktop.rs` owns shared TradingView Desktop app-window
  helpers used by operation adapters, such as app-tab reads, create-new-tab
  clicks, close-tab clicks, and Desktop new-tab target waits. It stays in the
  CLI package because it depends on CDP runtime evaluation and Desktop UI
  structure; it is not domain/model logic.
- `crates/cli/src/app/dispatch.rs` may call `tradingview_model::*` directly when it is
  converting CLI command variants into validated primitive values or domain
  request types. It should call `ops::*` for executable TradingView
  operations. This keeps the dependency direction explicit: application
  dispatch interprets command input, the model crate validates and shapes pure command
  data, and ops executes live TradingView work.
- `crates/cli/src/ops/` contains operation adapter implementations grouped by
  capability. These modules still own command-facing TradingView operations;
  do not treat them as a pure domain crate boundary. Use
  `docs/operation-adapter-boundaries.md` before moving operation code into a
  crate.
- `crates/cli/src/ops/screener.rs` is the Screener operation adapter facade.
  It groups the public Screener adapter surface through `state`, `screens`,
  `filters`, `columns`, and `validation` submodules under
  `crates/cli/src/ops/screener/`. CDP-free validation, target resolution, and
  storage payload helpers live in `tradingview_model::screener`; `state`, `screens`,
  `filters`, and `columns` own their respective runtime operation bodies and
  focused tests. The remaining `engine` module is the shared Screener
  runtime/helper layer for open/restore sessions, state reads, active storage
  fetches, common click dispatch, and shared JavaScript helpers.
- `crates/cli/src/ops/alert.rs` is the Alert operation adapter facade. It
  groups alert list, normal alert create, Pine `alertcondition()` alert create,
  alert delete, and an adapter-facing payload re-export under
  `crates/cli/src/ops/alert/`. Alert remains a CLI-package adapter because it
  still combines page-session APIs, active chart metadata, Pine helper
  integration, DOM fallback, and post-check execution. CDP-free validation and
  public payload normalization live in `tradingview_model::alert`.
- `crates/cli/src/ops/layout.rs` is the historical Layout operation adapter
  facade. It groups the public watchlist and pane adapter surface through
  `watchlist` and `pane` submodules under `crates/cli/src/ops/layout/`.
  Watchlist uses `tradingview_model::watchlist` for CDP-free normalization and bulk
  aggregation, while the adapter owns API-backed mutation execution,
  visible-panel fallback, and post-checks. Pane owns chart pane list, layout,
  focus, and symbol operations.
- `crates/cli/src/ops/replay.rs` is the Replay operation adapter facade. It
  groups replay start, step, stop, status, autoplay, and trade operations under
  `crates/cli/src/ops/replay/`. Replay validation and payload normalization
  live in `tradingview_model::replay`; the adapter owns Replay API method calls, runtime
  evaluation, availability checks, and post-check behavior.
- `crates/cli/src/ops/data.rs` is a thin facade for larger sub-surfaces under
  a same-named directory. `crates/cli/src/ops/pine.rs` is a facade that
  combines Desktop-free helpers from `tradingview_pine` with CDP-dependent Pine
  Editor operations under `crates/cli/src/ops/pine/`. The Pine Editor adapter
  keeps `editor.rs` as a facade over `runtime`, `source`, `scripts`, and
  `compile` modules so Monaco/CDP helpers, source editing, saved-script
  operations, and compile/report reads stay separated.
- `crates/cli/src/ops/drawing.rs` is the Drawing operation adapter facade. It
  groups drawing creation, drawing reads, and drawing lifecycle cleanup under
  `crates/cli/src/ops/drawing/` while preserving the public `draw` command
  surface. Drawing request types, direction parsing, override parsing, and
  position validation live in `tradingview_model::drawing`; the adapter owns chart API
  JavaScript, entity post-checks, and live chart reads.
- `crates/cli/src/ops/market.rs` is the chart-dependent Market operation
  adapter facade. Its `direct` module delegates Desktop-free symbol search,
  symbol info, and scanner quote reads to `tradingview_market`; its `quote`
  and `ohlcv` modules keep current-chart quote reads, `--source chart` /
  `--source auto` chart execution, quote freshness checks, and chart-bars reads
  in the CLI package.
- `crates/cli/src/ops/ui.rs` is the generic UI automation compatibility
  adapter facade. It groups DOM actions, input events, selector lookup, and
  eval execution under `crates/cli/src/ops/ui/`. The unsafe `ui eval` gate
  stays in the application safety/dispatch layer; the eval adapter only
  receives already-authorized expressions.
- `crates/cli/src/ops/scanner.rs` delegates scanner reads to
  `tradingview_scanner`.

Do not grow command implementation logic in `crates/cli/src/main.rs` or
`crates/cli/src/cli.rs`. If a capability module becomes difficult to scan,
split it before adding more surface area.

## Module layout

This project uses Rust 2024. Do not introduce `mod.rs`.

Prefer a facade file plus a same-named directory for submodules, as with:

- `crates/cli/src/ops.rs` plus `crates/cli/src/ops/`
- `crates/cli/src/ops/screener.rs` plus `crates/cli/src/ops/screener/`
- `crates/cli/src/ops/alert.rs` plus `crates/cli/src/ops/alert/`
- `crates/cli/src/ops/layout.rs` plus `crates/cli/src/ops/layout/`
- `crates/cli/src/ops/data.rs` plus `crates/cli/src/ops/data/`
- `crates/cli/src/ops/pine.rs` plus `crates/cli/src/ops/pine/`

Add new operations to the capability module that matches the user-visible
surface, not to a generic catch-all. Keep shared helpers private or
`pub(super)` unless another top-level module truly needs them.

## JSON contract

Successful output uses the Rust envelope:

```json
{
  "success": true,
  "command": "quote",
  "data": {}
}
```

Failures use:

```json
{
  "success": false,
  "command": "quote",
  "error": {
    "kind": "connection",
    "message": "CDP connection failed",
    "details": null
  }
}
```

Do not move command payload fields back to the top level to mimic the old
JavaScript CLI.

For migrated commands, preserve practical information compatibility: useful
information exposed by the old CLI should remain available under `data` unless
a durable project decision accepts the loss.

Use `docs/breaking-changes-from-js-cli.md` as the short downstream migration
guide. Use `docs/notes/rust-cli-contract-migration-2026-04-24.md` for the
command-by-command contract inventory.

## Mutation safety

Mutation commands must validate inputs early and prove the requested after-state
before reporting success.

Examples:

- watchlist add/remove prefer the logged-in symbols-list API and verify
  presence or absence by readback
- alert create uses the logged-in alert endpoint and verifies the new alert by
  list readback before success
- Screener saved-screen, filter cleanup, and column operations use storage
  payloads where available and require storage post-checks
- chart, drawing, tab, layout, replay, Pine, and generic UI commands must fail
  with a structured error rather than guessing when the required page object or
  visible state is unavailable

Prefer dry-run modes for broad or account-state mutations where practical.
Guard destructive test-only operations to disposable names when the command
edits saved account state.

## Current deferred boundaries

The current known implementation set is intentionally not trying to become a
general TradingView automation framework.

Keep these boundaries unless a future ExecPlan records new evidence:

- `columns reset` remains deferred until a reliable default Screener column
  source is found
- broad multi-option and free-text Screener filter editing remains deferred
- Pine save for a new unsaved script remains deferred when the naming dialog is
  outside the verified CDP target
- generic `ui` commands exist for compatibility, but higher-level commands are
  preferred
- direct HTTP operation without TradingView Desktop page-session context is
  still future research for account/session-bound commands, documented in
  `docs/plans/archives/tradingview-cli-direct-http-feasibility.md`
