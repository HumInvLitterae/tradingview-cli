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

- `status` in Rust v1 focuses on CDP target connectivity. The JavaScript CLI also exposed chart API availability and current chart fields such as `chart_symbol`, `chart_resolution`, and `api_available`.
- `state` in Rust v1 uses `timeframe` and `chart_type`; the JavaScript CLI used `resolution` and `chartType` and included `studies`.
- `quote` payload is under `data` in Rust. The JavaScript CLI returned quote fields at the top level.
- `ohlcv --summary` payload is under `data` in Rust and does not yet expose every JavaScript summary field.
- Raw `ohlcv --count`, `range`, `scroll`, `values`, `watchlist get`, `pane list`, `search`, and `screenshot --region chart` are not yet implemented in Rust v1.

These differences are migration gaps, not proof that the information is out of scope.
