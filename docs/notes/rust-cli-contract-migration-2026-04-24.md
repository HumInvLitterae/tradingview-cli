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
- `discover` payload is under `data` in Rust and includes `apis_available`, `apis_total`, and `apis`.
- `ui-state` payload is under `data` in Rust and includes panel state, button groups, key buttons, chart summary, and replay state.
- `watchlist get`, `watchlist add`, and `pane list` are implemented in Rust. Their payloads still live under `data`; `watchlist get` may return `source: "panel_closed"` with `count: 0` when the watchlist panel is closed.
- `watchlist add` is an explicit operator mutation. It preserves the practical old CLI fields `symbol` and `action` under `data`, with Rust-specific context such as `requested_symbol`, `source`, `opened_panel`, and `add_button` also available.
- `alert list` is implemented in Rust. Its payload lives under `data` and includes `alert_count`, `source`, `alerts`, and optional `error`; alert rows preserve practical old CLI fields such as `alert_id`, `symbol`, `type`, `message`, `active`, `condition`, `resolution`, `created`, `last_fired`, and `expiration`.
- `alert list` depends on the current TradingView page session and the internal `pricealerts.tradingview.com/list_alerts` endpoint. When the page fetch fails, Rust preserves the failure as `data.error` with an empty alert list rather than treating the read command itself as a CDP failure.
- `data indicator`, `data strategy`, `data trades`, `data equity`, `data lines`, `data labels`, `data tables`, `data boxes`, and `data depth` are implemented in Rust. Their payloads live under `data`; the practical old CLI fields such as `metric_count`, `trade_count`, `data_points`, `study_count`, `studies`, `inputs`, `metrics`, `trades`, `equity_summary`, `bid_levels`, `ask_levels`, `spread`, `bids`, and `asks` remain available.
- `data depth` remains DOM-panel dependent. If TradingView's DOM or Depth of Market panel is closed or does not expose readable rows, the command may fail with `internal_api_unavailable` rather than returning an empty success payload.
- `type [CHART_TYPE]` is implemented in Rust. Its payload lives under `data`; the practical old CLI fields `chart_type` and `type_num` remain available. Set-mode payloads also include requested, previous, and observed chart type fields so callers can restore the original chart type after smoke or screenshot workflows.
- `screenshot --region full` and `screenshot --region chart` are implemented in Rust. Their payloads live under `data`; screenshot payloads include the old CLI practical fields `method`, `file_path`, `region`, and `size_bytes`, plus Rust's `output_path`. Chart-region screenshot payloads also include `clip` fields with `x`, `y`, `width`, `height`, and `scale`, and `capture_mode`.
- Chart-region screenshots prefer CDP `Page.captureScreenshot` with a `clip` parameter, matching the old JavaScript CLI's practical capture path. If clipped CDP capture fails, Rust falls back to a full-page CDP screenshot plus local PNG crop.
- A clipped CDP timeout was observed during development, but later checks showed both the old JavaScript CLI and the Rust implementation could capture the same chart through clipped CDP. Treat that timeout as an intermittent CDP/session-state observation, not as a confirmed TradingView or Rust library limitation.
- `screenshot --region chart` remains DOM-selector dependent. If TradingView changes the visible chart DOM or no chart element is available, the command may fail with `internal_api_unavailable`.
- The old JavaScript CLI included `success: true` inside `discover` and `ui-state` payloads. Rust does not duplicate that field inside `data`; success remains the top-level envelope field.

These differences are migration gaps, not proof that the information is out of scope.
