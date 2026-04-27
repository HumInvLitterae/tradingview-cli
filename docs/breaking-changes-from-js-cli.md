# Breaking changes from the JavaScript CLI

This document summarizes the intentional breaking differences between the old JavaScript `tv` CLI from `tradingview-mcp` and this Rust-native `tv` CLI.

The known old JavaScript CLI command surface has been migrated, but this Rust CLI is not a JSON wire-format clone. Downstream adapters should treat this file as the short migration guide and use `docs/notes/rust-cli-contract-migration-2026-04-24.md` for command-by-command contract details.

## JSON envelope

The main breaking change is the output envelope.

The old JavaScript CLI usually returned command payload fields at the top level:

```json
{
  "success": true,
  "symbol": "NASDAQ:AAPL"
}
```

The Rust CLI returns successful command payloads under `data` and includes the command name at the top level:

```json
{
  "success": true,
  "command": "quote",
  "data": {
    "symbol": "NASDAQ:AAPL"
  }
}
```

Errors are also structured:

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

Downstream migration rule:

- keep reading top-level `success`
- read successful command payloads from `data`
- read failures from `error.kind`, `error.message`, and `error.details`
- do not expect old top-level command fields such as `symbol`, `price`, `alerts`, or `panes`

## Information compatibility

The Rust CLI may change field placement and documented field names, but migrated commands must preserve practical information from the old CLI.

If the old CLI exposed useful information, the Rust command should keep that information available under `data` unless a durable project decision explicitly accepts the loss.

## Behavior differences

Some Rust commands are intentionally safer or more explicit than the old JavaScript CLI:

- `launch` defaults to no-kill behavior and only terminates existing TradingView processes with explicit `--kill-existing`.
- `watchlist remove <SYMBOL>` is Rust-specific cleanup surface; the old CLI had `watchlist get/add` but no remove command.
- `tab close <INDEX>` requires an explicit app-tab index and refuses to close the final TradingView app tab.
- `layout switch <TARGET>` resolves by layout id or exact case-insensitive name, supports `--dry-run`, and does not automatically dismiss unsaved-change dialogs.
- `alert delete --all` supports `--dry-run`, reports target alert ids, and verifies post-delete state instead of only opening manual UI.
- `draw clear` supports `--dry-run`, reports target drawings, and verifies post-clear state.
- `pine compile` avoids save-related action buttons; `pine raw-compile` exists for old broad compile behavior and may click save-related Pine actions.
- `pine save` persists only the current already-saved script; it fails rather than typing into an unverified naming dialog for unsaved scripts.
- `stream ...` commands print newline-delimited JSON envelopes rather than one request-response JSON object.
- generic `ui ...` commands are present for compatibility but can mutate the active TradingView UI session; prefer higher-level commands when available.
- `ui eval` is present as a dangerous compatibility command but is disabled by default. Set `TV_ALLOW_UNSAFE_UI_EVAL=1` to run arbitrary JavaScript in the authenticated TradingView page context explicitly.

## Target selection

When multiple TradingView chart targets are open, most chart-specific commands fail with `target_ambiguous` unless a target is selected.

Use:

```bash
tv tab list
tv --target-id <TARGET_ID> status
```

`TV_CDP_TARGET_ID=<TARGET_ID>` remains a v0.2.x fallback, but new automation
should prefer `--target-id`.

## Exit codes

Rust uses explicit exit codes:

- `0`: success
- `1`: usage, validation, target ambiguity, or unexpected internal failure
- `2`: TradingView or CDP connection failure
- `3`: TradingView internal API unavailable
- `4`: timeout

## Related docs

- `docs/notes/rust-cli-contract-migration-2026-04-24.md`: full command contract migration note
- `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`: migrated command inventory and Rust-specific lifecycle decisions
- `docs/notes/remaining-deferred-surface-audit-2026-04-25.md`: remaining surface audit and closure notes
