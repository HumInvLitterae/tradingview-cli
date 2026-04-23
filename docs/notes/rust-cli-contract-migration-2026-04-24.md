# Rust CLI contract migration 2026-04-24

This note records the JSON contract policy for migrating from the JavaScript `tradingview-mcp` CLI to this Rust-native `tv` CLI.

## Decision

The Rust CLI keeps its structured envelope:

```json
{
  "success": true,
  "command": "quote",
  "data": {}
}
```

Error output uses the matching structured error envelope:

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

This is intentionally different from the JavaScript CLI, which generally returned command payload fields at the top level:

```json
{
  "success": true,
  "symbol": "NASDAQ:AAPL"
}
```

## Rationale

The Rust envelope is easier to process consistently across commands. It gives downstream callers stable places for command name, success state, command payload, error kind, error message, and optional details.

The compatibility cost is real: adapters that read old top-level command fields must read `data` instead. This is an accepted breaking wire-format change.

## Information compatibility rule

The wire shape may change, but migrated commands must preserve practical information compatibility.

If the old JavaScript CLI exposed useful information for a command, the Rust implementation of that command must make the same information available unless a durable project decision explicitly accepts the loss. New fields are allowed. Field names may change when the replacement is documented, but silently dropping useful information is not allowed.

When an old field name is important to downstream workflows, prefer one of these outcomes:

- keep the old field name inside `data`
- add a clearly documented replacement field inside `data`
- record why the information is unavailable and get an explicit decision before merging

## Downstream adapter migration checklist

- Continue reading top-level `success`.
- Read successful command payloads from top-level `data`.
- Read failure text from `error.message`.
- Use `error.kind` for structured handling where useful.
- Use top-level `command` for logging, diagnostics, and command correlation.
- Do not assume the Rust CLI is a drop-in JSON wire-format clone of the JavaScript CLI.

## Current known differences

- `status` payload is under `data` in Rust and now includes CDP target fields plus chart API availability and current chart fields such as `chart_symbol`, `chart_resolution`, `chart_type`, and `api_available`.
- `state` payload is under `data` in Rust and includes both old and new naming where useful, including `resolution`, `timeframe`, `chartType`, `chart_type`, `studies`, and `visible_range`.
- `quote` payload is under `data` in Rust and includes the practical quote fields from the old CLI shape, including `symbol`, `time`, `last`, `close`, `open`, `high`, `low`, `volume`, and best-effort symbol metadata.
- `ohlcv --summary` payload is under `data` in Rust and includes summary fields such as `symbol`, `timeframe`, `bar_count`, `period`, `range`, `change`, `change_pct`, `avg_volume`, `volume`, and `last_5_bars`.
- Raw `ohlcv --count`, `range`, and `scroll` are implemented in Rust. Their payloads still live under `data`.
- `info` payload is under `data` in Rust and includes symbol metadata such as `symbol`, `full_name`, `exchange`, `description`, `type`, `pro_name`, `typespecs`, `resolution`, and `chart_type`.
- `search` payload is under `data` in Rust and includes `query`, `source`, `count`, and normalized `results` rows with `symbol`, `description`, `exchange`, `type`, and `full_name`.
- `values` payload is under `data` in Rust and includes `study_count` plus `studies` rows with `name` and `values`.
- `watchlist get` and `pane list` are implemented in Rust. Their payloads still live under `data`; `watchlist get` may return `source: "panel_closed"` with `count: 0` when the watchlist panel is closed.
- `discover`, `ui-state`, and `screenshot --region chart` are not yet implemented in Rust.

These differences are migration gaps, not proof that the information is out of scope.
