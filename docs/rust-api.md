# Rust API guide

This repository is still CLI-first. The workspace crates are internal and not
published as stable crates.io APIs, but several crates are intentionally shaped
for reuse by other Rust applications in the same workspace or local tooling.

## Current boundary

- `tradingview-core` contains shared error and JSON envelope contracts.
- `tradingview-model` contains I/O-free request interpretation, validation,
  selector resolution, payload shaping, and fallback policy.
- `tradingview-market` contains credential-free, Desktop-free market reads for
  symbol search, symbol metadata, single-symbol quotes, batch quotes, and
  browserless historical bars.
- `tradingview-scanner` contains credential-free, Desktop-free scanner hotlist,
  scan, and field metadata reads.
- `tradingview-pine` contains Desktop-free Pine source analysis and check
  helpers.
- `tradingview-cdp` contains TradingView Desktop / Chrome DevTools Protocol
  connection primitives.

Desktop-free means the crate does not connect to the locally running
TradingView Desktop app and does not use chart state, UI automation, cookies,
session export, or account mutation.

## Market reads

Prefer the typed functions from `tradingview-market` for Rust callers:

- `search_symbols_typed(query)`
- `symbol_info_typed(symbol)`
- `fundamentals_symbol_typed(symbol, fields)`
- `quote_symbol_typed(symbol)`
- `quote_symbols_typed(symbols)`

The JSON-returning functions `symbol_search`, `symbol_info`,
`fundamentals_symbol`, `quote_symbol`, `quote_symbols`, and `bars_symbol`
remain public for CLI payload compatibility. New Rust code should not parse
those JSON payloads unless it is specifically preserving the `tv` command
contract.

Example shape:

    let quote = tradingview_market::quote_symbol_typed("NYSE:IONQ").await?;
    println!("{} {:?}", quote.symbol, quote.last);

Market quote reads use TradingView scanner data. They are useful for screening
and Desktop-free checks, but they are not execution-grade realtime guarantees.
Use `Quote::time`, `Quote::update_mode`, and `Quote::delay_seconds` when the
caller needs to show feed timing or delayed-streaming metadata.
Desktop-free market typed results expose `source_category:
"desktop_free_read"`, `requires_desktop: false`, and `non_mutating: true`.
`bars_symbol` currently exposes the CLI-compatible `bars.v1` JSON contract for
bounded historical OHLCV reads; typed bars structs are intentionally not a
stable Rust API yet.

## Scanner reads

Prefer the typed functions from `tradingview-scanner` for Rust callers:

- `scanner_hotlist_typed(slug, limit)`
- `scanner_scan_typed(request)`
- `scanner_scan_page_typed(request)`
- `scanner_scan_aggregate_typed(request)`
- `scanner_metainfo_typed(request)`

The JSON-returning functions `scanner_hotlist`, `scanner_scan`,
`scanner_scan_page`, `scanner_scan_aggregate`, and `scanner_metainfo` remain
public for CLI payload compatibility. Explicit-offset reads return a separate
`ScannerPageScanResult` wrapper so `ScannerScanResult` and the default
first-page JSON contract remain source- and payload-compatible.

`ScannerScanResult::symbols[].field_values` intentionally remains a JSON value
because scanner columns can contain numbers, strings, booleans, or nulls
depending on the requested column and market state. A later internal API review
may introduce stronger field-value enums if downstream consumers need that.
Scanner typed results expose `source_category: "desktop_free_read"`,
`requires_desktop: false`, and `non_mutating: true`.

`ScannerAggregateScanRequest` wraps a `ScannerScanRequest` plus `page_size`
and `max_results`. The aggregate API reuses one configured HTTP client, keeps
each page at 100 rows or fewer, requires an integer provider total on every
page, and returns `ScannerAggregateScanResult` only after bounded completion.
Its deduplicated symbols and drift metadata describe a sequential observation,
not an atomic snapshot.

Example shape:

    let request = tradingview_scanner::ScannerMetainfoRequest {
        market: "america".to_string(),
        fields: vec!["close".to_string(), "premarket_close".to_string()],
    };
    let metainfo = tradingview_scanner::scanner_metainfo_typed(request).await?;
    println!("{} fields", metainfo.field_count);

## What stays in the CLI package

The `tradingview-cli` package owns the `tv` command surface, JSON envelopes,
target selection, TradingView Desktop runtime orchestration, UI automation, and
account mutation workflows. It may keep using JSON wrappers when that preserves
the public CLI contract.

Do not move chart fallback quote reads, `tv ohlcv`, Screener UI operations,
watchlist/alert mutations, Pine Editor operations, or generic UI automation
into `tradingview-market` or `tradingview-scanner`. Those paths are not
Desktop-free read APIs.
