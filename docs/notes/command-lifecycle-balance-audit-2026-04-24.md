# Command lifecycle balance audit 2026-04-24

This note audits old JavaScript CLI surfaces and the current Rust CLI for asymmetric lifecycle commands: create/add/open/start commands that do not have an obvious cleanup, remove/close/stop, or restore path.

## Current Rust CLI lifecycle state

The highest-priority lifecycle gap has been closed by `watchlist remove`.

Rust now implements both `tv watchlist add <SYMBOL>` and `tv watchlist remove <SYMBOL>`. The remove command is not an old CLI migration backlog item in the narrow sense, because the source CLI only exposed `watchlist get` and `watchlist add`. It is a Rust CLI safety command: live smoke tests and downstream operator workflows can add symbols to an account watchlist, so the Rust CLI needs a matching cleanup command.

`watchlist remove` is intentionally row-scoped and exact-match only. It must prove the requested `data-symbol-full` exists before deletion and prove it is absent afterward. If the row is missing, or if TradingView does not expose a safe row-scoped remove control, the command fails instead of attempting a broad cleanup.

`alert create` no longer has the same gap. Rust now implements `tv alert delete --id <ALERT_ID>`, which is the cleanup pair for created alerts. Bulk alert deletion remains deferred because it has a much larger account-level blast radius than deleting one known alert ID.

`pane layout`, `pane focus`, `pane symbol`, `symbol`, `timeframe`, `type`, `range`, and `scroll` are chart state mutations rather than account resource creation. Their recovery model is to read the previous value and restore it after smoke or downstream workflows. They do not require a delete command, but tests and live smoke should keep using restore-safe patterns.

`indicator add/remove/toggle/set/get` is now implemented as a complete chart-local lifecycle surface. `indicator add` returns the new `entity_id`, `indicator remove` removes by `entity_id`, `indicator toggle` can hide or show that same study, `indicator set` changes known input ids, and `indicator get` exposes the same practical indicator information as `data indicator`.

`draw shape/list/get/remove` is now implemented as a chart-local drawing lifecycle surface. `draw shape` returns the new drawing `entity_id`, `draw get` inspects that one drawing, and `draw remove` removes that exact drawing by id. `draw clear` remains deferred because it removes all chart drawings and has a much larger blast radius.

`tab list/switch` is now implemented as a non-resource-destructive target operation surface. `tab list` reads existing TradingView chart targets from the CDP target list, and `tab switch` activates one existing chart target by index. These commands do not create or close tabs, so they do not introduce a new cleanup gap.

`replay start/step/stop/status/autoplay` is now implemented as a bounded replay lifecycle surface. `start` enters replay mode, `step` advances one replay bar, `autoplay` toggles autoplay and can set only known safe delays, `stop` returns to realtime or reports `already_stopped`, and `status` reads the current state. Live smoke and downstream workflows must still disable autoplay if it was turned on and use `stop` after a successful `start`.

## Old CLI lifecycle pairs not yet migrated

Some old JavaScript CLI areas expose lifecycle pairs, but those whole surfaces are still deferred in Rust. They should be planned as full high-risk surfaces, not treated as a single missing cleanup command for an already-implemented Rust mutation.

- `tab new/close`
- `replay trade`

These surfaces can mutate chart state, account state, or UI session state. Each needs its own ExecPlan before implementation, with downstream need, safety constraints, information compatibility, live smoke strategy, and recovery behavior recorded.

## Deferred high-risk cleanup and control surfaces

The following are intentionally not next by default:

- `alert delete --all`
- alert edit, pause, and resume commands
- drawing `clear`
- tab `new`
- tab `close`
- replay trade commands
- generic UI automation commands

These commands can remove many account or chart resources or depend heavily on localized UI state. They need stronger safety design than simple old CLI parity.

## Recommended next candidate

No immediate asymmetric lifecycle gap is known in the implemented Rust CLI after `watchlist remove`.

The next mutation surface should still be checked against this note before implementation. Account-level or destructive commands such as bulk alert deletion, drawing clear, tab close, and generic UI automation remain high-risk and need their own ExecPlan.
