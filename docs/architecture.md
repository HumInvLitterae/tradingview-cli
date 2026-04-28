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

## Crate boundary

The repository is now a Cargo workspace. The root package remains
`tradingview-cli`, and the installed binary remains `tv`. The workspace also
contains internal support crates under `crates/`.

- `crates/core/src/lib.rs` owns the shared contract types: `AppError`,
  `ErrorKind`, `SuccessEnvelope`, `ErrorEnvelope`, and `ErrorBody`. These are
  intentionally small and dependency-light so future internal crates can share
  JSON envelope and exit-code semantics without depending on the full CLI.
- `crates/market/src/lib.rs` owns Desktop-free market reads for symbol search,
  symbol metadata, and symbol quote lookup. It uses credential-free TradingView
  HTTP endpoints and does not depend on CDP, chart state, or UI automation.
- `crates/scanner/src/lib.rs` owns Desktop-free scanner reads for hotlists and
  basic scanner scans. It uses credential-free scanner HTTP endpoints and does
  not depend on CDP, chart state, or UI automation.
- `crates/pine/src/lib.rs` owns Desktop-free Pine helpers for local static
  analysis, `alertcondition()` discovery, and Pine facade checks. Pine Editor
  operations remain in the root CLI crate because they depend on CDP.
- `src/lib.rs` is the root package's library crate root. It exposes the
  current internal modules so the binary can share a single module tree and
  future refactors can extract reusable pieces incrementally.
- `src/main.rs` is the `tv` binary entrypoint. It owns only process startup
  and the Windows larger-stack wrapper before calling the library-owned
  application runner.
- `src/app.rs` is the root package's application-layer facade. It owns CLI
  parsing, command dispatch, runtime connection orchestration, stdin/file input
  conversion, stream looping, safety gates, and success/error envelope output.

These library crates are internal and unstable for now. Treat them as
maintainability boundaries until a future ExecPlan explicitly marks types or
functions as stable for downstream Rust callers.

## Rust module responsibilities

Keep responsibilities separated:

- `src/lib.rs` owns top-level module declarations.
- `src/main.rs` owns startup for the `tv` binary and delegates application
  behavior to `tradingview_cli::app`.
- `src/app.rs` and `src/app/` own application orchestration between the CLI
  surface and domain operation modules.
- `src/cli.rs` owns the `clap` command surface, argument definitions, and
  command names.
- `crates/core/src/lib.rs` owns shared typed errors, JSON success/error
  envelopes, and error exit-code mapping.
- `crates/market/src/lib.rs` owns direct HTTP implementations behind
  `tv search <QUERY>`, `tv info <SYMBOL>`, and `tv quote <SYMBOL>`.
- `crates/scanner/src/lib.rs` owns direct HTTP implementations behind
  `tv scanner hotlist` and `tv scanner scan`.
- `crates/pine/src/lib.rs` owns Desktop-free Pine static analysis,
  `alertcondition()` discovery, and `tv pine check` support.
- `src/ops.rs` is a thin facade that declares operation modules and re-exports
  operation functions used by `src/main.rs`.
- `src/ops/` contains operation implementations grouped by capability.
- `src/ops/data.rs` is a thin facade for larger sub-surfaces under a
  same-named directory. `src/ops/pine.rs` is a facade that combines
  Desktop-free helpers from `tradingview_pine` with CDP-dependent Pine Editor
  operations under `src/ops/pine/`.
- `src/cdp.rs` owns Chrome DevTools Protocol evaluation and screenshot
  primitives.
- `src/ops/market.rs` owns chart-dependent market reads, including current
  chart quote fallback and OHLCV. Its Desktop-free functions delegate to
  `tradingview_market`.
- `src/ops/scanner.rs` delegates scanner reads to `tradingview_scanner`.
- `src/transport.rs` owns TradingView CDP target discovery and connection
  setup. `tv --target-id <CDP_TARGET_ID>` is the primary explicit target
  selection path.

Do not grow command implementation logic in `src/main.rs` or `src/cli.rs`.
If a capability module becomes difficult to scan, split it before adding more
surface area.

## Module layout

This project uses Rust 2024. Do not introduce `mod.rs`.

Prefer a facade file plus a same-named directory for submodules, as with:

- `src/ops.rs` plus `src/ops/`
- `src/ops/data.rs` plus `src/ops/data/`
- `src/ops/pine.rs` plus `src/ops/pine/`

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
