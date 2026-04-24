# Command lifecycle balance audit 2026-04-24

This note audits old JavaScript CLI surfaces and the current Rust CLI for asymmetric lifecycle commands: create/add/open/start commands that do not have an obvious cleanup, remove/close/stop, or restore path.

## Current Rust CLI lifecycle state

The highest-priority lifecycle gap is `watchlist add`.

Rust now implements `tv watchlist add <SYMBOL>`, but neither this Rust CLI nor the old JavaScript CLI currently has `watchlist remove`. This is not an old CLI migration backlog item in the narrow sense, because the source CLI only exposed `watchlist get` and `watchlist add`. It is still a Rust CLI safety consideration because live smoke tests and downstream operator workflows can add symbols to an account watchlist without a matching cleanup command.

`alert create` no longer has the same gap. Rust now implements `tv alert delete --id <ALERT_ID>`, which is the cleanup pair for created alerts. Bulk alert deletion remains deferred because it has a much larger account-level blast radius than deleting one known alert ID.

`pane layout`, `pane focus`, `pane symbol`, `symbol`, `timeframe`, `type`, `range`, and `scroll` are chart state mutations rather than account resource creation. Their recovery model is to read the previous value and restore it after smoke or downstream workflows. They do not require a delete command, but tests and live smoke should keep using restore-safe patterns.

## Old CLI lifecycle pairs not yet migrated

Some old JavaScript CLI areas expose lifecycle pairs, but those whole surfaces are still deferred in Rust. They should be planned as full high-risk surfaces, not treated as a single missing cleanup command for an already-implemented Rust mutation.

- `indicator add/remove/toggle/set/get`
- `draw shape/list/get/remove/clear`
- `tab new/close/switch/list`
- `replay start/stop/status/step/autoplay/trade`

These surfaces can mutate chart state, account state, or UI session state. Each needs its own ExecPlan before implementation, with downstream need, safety constraints, information compatibility, live smoke strategy, and recovery behavior recorded.

## Deferred high-risk cleanup and control surfaces

The following are intentionally not next by default:

- `alert delete --all`
- alert edit, pause, and resume commands
- drawing `clear`
- tab `close`
- generic UI automation commands

These commands can remove many account or chart resources or depend heavily on localized UI state. They need stronger safety design than simple old CLI parity.

## Recommended next candidate

The next cleanup-oriented command candidate is `tv watchlist remove <SYMBOL>`.

Before implementing it, write an ExecPlan that treats it as a Rust-native operator cleanup command rather than an old CLI migration. The plan should determine whether a stable internal API or a sufficiently safe DOM path exists, how to match symbols exactly, how to verify the symbol was removed, and how to live-smoke without damaging a user's real watchlist.

The acceptance bar should be higher than `watchlist add`: deletion must prove that the requested symbol existed before removal and is absent afterward, or fail clearly without touching unrelated rows.

