# Command lifecycle balance audit 2026-04-24

This note audits old JavaScript CLI surfaces and the current Rust CLI for asymmetric lifecycle commands: create/add/open/start commands that do not have an obvious cleanup, remove/close/stop, or restore path.

## Current Rust CLI lifecycle state

The highest-priority lifecycle gap has been closed by `watchlist remove`.

Rust now implements both `tv watchlist add <SYMBOL>` and `tv watchlist remove <SYMBOL>`. The remove command is not an old CLI migration backlog item in the narrow sense, because the source CLI only exposed `watchlist get` and `watchlist add`. It is a Rust CLI safety command: live smoke tests and downstream operator workflows can add symbols to an account watchlist, so the Rust CLI needs a matching cleanup command.

`watchlist remove` is intentionally row-scoped and exact-match only. It must prove the requested `data-symbol-full` exists before deletion and prove it is absent afterward. If the row is missing, or if TradingView does not expose a safe row-scoped remove control, the command fails instead of attempting a broad cleanup.

`alert create` no longer has the same gap. Rust now implements `tv alert delete --id <ALERT_ID>`, which is the cleanup pair for created alerts. Bulk alert deletion remains deferred because it has a much larger account-level blast radius than deleting one known alert ID.

`pane layout`, `pane focus`, `pane symbol`, `symbol`, `timeframe`, `type`, `range`, and `scroll` are chart state mutations rather than account resource creation. Their recovery model is to read the previous value and restore it after smoke or downstream workflows. They do not require a delete command, but tests and live smoke should keep using restore-safe patterns.

`indicator add/remove/toggle/set/get` is now implemented as a complete chart-local lifecycle surface. `indicator add` returns the new `entity_id`, `indicator remove` removes by `entity_id`, `indicator toggle` can hide or show that same study, `indicator set` changes known input ids, and `indicator get` exposes the same practical indicator information as `data indicator`.

`draw shape/list/get/remove/clear` is now implemented as a chart-local drawing lifecycle surface. `draw shape` returns the new drawing `entity_id`, `draw get` inspects that one drawing, `draw remove` removes that exact drawing by id, and `draw clear` removes all chart-local drawings only after a `--dry-run`-capable preflight and post-clear verification. Live smoke must not use `draw clear` when pre-existing user drawings are present.

`tab list/switch/new/close` is now implemented as a bounded tab lifecycle surface. `tab list` reads existing TradingView chart targets from the CDP target list and also reports TradingView Desktop app tabs from the tab-strip DOM, `tab switch` activates one existing chart target by index, `tab new` opens a new app tab from an explicit or unambiguous source chart tab, and `tab close` closes an explicit app-tab index. `tab close` refuses to close the final TradingView app tab. This is intentionally safer than the old current-tab close behavior.

`replay start/step/stop/status/autoplay/trade` is now implemented as a bounded replay lifecycle surface. `start` enters replay mode, `step` advances one replay bar, `autoplay` toggles autoplay and can set only known safe delays, `trade` can buy, sell, or close a replay position, `stop` returns to realtime or reports `already_stopped`, and `status` reads the current state. Live smoke and downstream workflows must still close any replay position, disable autoplay if it was turned on, and use `stop` after a successful `start`.

## Old CLI lifecycle pairs not yet migrated

No old JavaScript CLI lifecycle pair is currently half-migrated in Rust.

The remaining larger old CLI surfaces can still mutate chart state, account state, or UI session state. Each needs its own ExecPlan before implementation, with downstream need, safety constraints, information compatibility, live smoke strategy, and recovery behavior recorded.

## High-risk cleanup and control surfaces

The previously deferred high-risk old CLI surfaces now have explicit Rust contracts:

- `alert delete --all` includes `--dry-run`, target alert reporting, and post-delete verification.
- generic `ui` automation commands are implemented as compatibility commands, but higher-level CLI commands remain preferred for durable workflows.

Alert edit, pause, and resume were not found as old JavaScript CLI commands during the migration closure pass, so they are future feature research rather than remaining old CLI migration backlog.

## Recommended next candidate

No immediate asymmetric lifecycle gap is known in the implemented Rust CLI after `watchlist remove`, `draw clear`, and `alert delete --all`.

The next mutation surface should still be checked against this note before implementation. Account-level or broad UI automation remains high-risk even when a compatibility command exists.
