use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{
    bounded::{MAX_MULTI_SYMBOLS, MULTI_SYMBOL_CONCURRENCY, collect_ordered_bounded},
    http::{configured_client, map_http_error, remote_status_error},
    info::preferred_symbol_candidates,
    normalize::{bare_symbol, split_exchange_symbol},
    search::symbol_search_with_client,
    types::{
        BatchQuoteItem, BatchQuotes, ExtendedHoursQuote, FreshnessCheck, Quote, QuoteError,
        SessionQuote,
    },
};

const QUOTE_SCAN_URL: &str = "https://scanner.tradingview.com/america/scan";
const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";
const QUOTE_SCAN_COLUMNS: &[&str] = &[
    "name",
    "description",
    "close",
    "open",
    "high",
    "low",
    "volume",
    "change",
    "exchange",
    "type",
    "subtype",
    "premarket_open",
    "premarket_high",
    "premarket_low",
    "premarket_close",
    "premarket_change",
    "premarket_change_abs",
    "premarket_gap",
    "premarket_volume",
    "postmarket_open",
    "postmarket_high",
    "postmarket_low",
    "postmarket_close",
    "postmarket_change",
    "postmarket_change_abs",
    "postmarket_volume",
    "time",
    "update_mode",
];

pub async fn quote_symbol(symbol: &str) -> Result<Value, AppError> {
    serde_json::to_value(quote_symbol_typed(symbol).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads one scanner-backed quote without connecting to TradingView Desktop.
///
/// This is the typed Rust API. Use [`quote_symbol`] only when preserving the
/// CLI-compatible JSON payload shape is required.
pub async fn quote_symbol_typed(symbol: &str) -> Result<Quote, AppError> {
    let client = configured_client()?;
    quote_symbol_typed_with_client(&client, symbol).await
}

pub(crate) async fn quote_symbol_typed_with_client(
    client: &reqwest::Client,
    symbol: &str,
) -> Result<Quote, AppError> {
    quote_symbol_typed_with_client_and_url(client, symbol, QUOTE_SCAN_URL).await
}

async fn quote_symbol_typed_with_client_and_url(
    client: &reqwest::Client,
    symbol: &str,
    scan_url: &str,
) -> Result<Quote, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "quote symbol must not be empty",
        ));
    }
    let value = quote_symbol_via_scanner(client, requested_symbol, scan_url).await?;
    match normalize_scanner_quote_response_typed(requested_symbol, &value) {
        Ok(quote) => Ok(quote),
        Err(err) if err.kind == ErrorKind::Validation => {
            Err(add_symbol_search_candidates(client, err, requested_symbol).await)
        }
        Err(err) => Err(err),
    }
}

pub async fn quote_symbols(symbols: Vec<String>) -> Result<Value, AppError> {
    match quote_symbols_typed(symbols).await {
        Ok(quotes) => serde_json::to_value(quotes)
            .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string())),
        Err(error) => Err(error),
    }
}

/// Reads multiple scanner-backed quotes in input order.
///
/// Each item contains either a typed [`Quote`] or a public-safe item error.
/// This is the typed Rust API. Use [`quote_symbols`] only when preserving the
/// CLI-compatible JSON payload shape is required.
pub async fn quote_symbols_typed(symbols: Vec<String>) -> Result<BatchQuotes, AppError> {
    let client = configured_client()?;
    quote_symbols_typed_with_client(&client, symbols).await
}

pub(crate) async fn quote_symbols_typed_with_client(
    client: &reqwest::Client,
    symbols: Vec<String>,
) -> Result<BatchQuotes, AppError> {
    quote_symbols_typed_with_client_and_url(client, symbols, QUOTE_SCAN_URL).await
}

async fn quote_symbols_typed_with_client_and_url(
    client: &reqwest::Client,
    symbols: Vec<String>,
    scan_url: &str,
) -> Result<BatchQuotes, AppError> {
    let requested_symbols = normalize_quote_symbols(symbols)?;
    let requested_count = requested_symbols.len();
    let completed = collect_ordered_bounded(
        requested_symbols,
        MULTI_SYMBOL_CONCURRENCY,
        |_, requested_symbol| async move {
            let result =
                quote_symbol_typed_with_client_and_url(client, &requested_symbol, scan_url).await;
            (requested_symbol, result)
        },
    )
    .await;
    let mut items = Vec::with_capacity(requested_count);
    let mut resolved_count = 0usize;
    let mut first_error: Option<AppError> = None;

    for (requested_index, (requested_symbol, result)) in completed {
        match result {
            Ok(quote) => {
                resolved_count += 1;
                items.push(BatchQuoteItem {
                    requested_index,
                    requested_symbol,
                    ok: true,
                    quote: Some(quote),
                    error: None,
                });
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(AppError {
                        kind: error.kind,
                        message: error.message.clone(),
                        details: error.details.clone(),
                    });
                }
                items.push(BatchQuoteItem {
                    requested_index,
                    requested_symbol,
                    ok: false,
                    quote: None,
                    error: Some(error_payload(error)),
                });
            }
        }
    }

    finalize_quote_items(requested_count, resolved_count, items, first_error)
}

fn finalize_quote_items(
    requested_count: usize,
    resolved_count: usize,
    items: Vec<BatchQuoteItem>,
    first_error: Option<AppError>,
) -> Result<BatchQuotes, AppError> {
    let error_count = requested_count.saturating_sub(resolved_count);
    let payload = BatchQuotes {
        source: "scanner_scan_rest".to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_count,
        resolved_count,
        error_count,
        items,
    };

    if resolved_count > 0 {
        Ok(payload)
    } else {
        let first_error = first_error.unwrap_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                "No quote symbols were resolved by TradingView scanner REST",
            )
        });
        Err(AppError::new(
            first_error.kind,
            "TradingView scanner quote did not resolve any requested symbols",
        )
        .with_details(
            serde_json::to_value(payload)
                .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?,
        ))
    }
}

fn normalize_quote_symbols(symbols: Vec<String>) -> Result<Vec<String>, AppError> {
    if symbols.is_empty() {
        return Err(quote_batch_validation_error(
            "quotes requires at least one symbol",
        ));
    }
    if symbols.len() > MAX_MULTI_SYMBOLS {
        return Err(quote_batch_validation_error(format!(
            "quotes accepts at most {MAX_MULTI_SYMBOLS} symbols"
        )));
    }

    let mut normalized = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(quote_batch_validation_error(
                "quote symbol must not be empty",
            ));
        }
        normalized.push(symbol.to_string());
    }
    Ok(normalized)
}

fn quote_batch_validation_error(message: impl Into<String>) -> AppError {
    AppError::new(ErrorKind::Validation, message).with_details(json!({
        "minimum": 1,
        "maximum": MAX_MULTI_SYMBOLS,
        "source": "scanner_scan_rest",
        "source_category": DESKTOP_FREE_READ_CATEGORY,
        "requires_desktop": false,
        "non_mutating": true,
    }))
}

fn error_payload(error: AppError) -> QuoteError {
    QuoteError {
        kind: error.kind,
        message: error.message,
        details: error.details,
    }
}

async fn quote_symbol_via_scanner(
    client: &reqwest::Client,
    symbol: &str,
    scan_url: &str,
) -> Result<Value, AppError> {
    let (exchange, name) = split_exchange_symbol(symbol);
    let mut filters = vec![json!({
        "left": "name",
        "operation": "equal",
        "right": name,
    })];
    if let Some(exchange) = exchange {
        filters.push(json!({
            "left": "exchange",
            "operation": "in_range",
            "right": [exchange],
        }));
    }
    let body = json!({
        "columns": QUOTE_SCAN_COLUMNS,
        "filter": filters,
        "range": [0, 2],
    });
    let response = client
        .post(scan_url)
        .json(&body)
        .send()
        .await
        .map_err(|err| map_http_error(err, "Scanner quote request"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(remote_status_error("TradingView scanner quote API", status));
    }

    response
        .json::<Value>()
        .await
        .map_err(|err| map_http_error(err, "Scanner quote response"))
}

#[cfg(test)]
fn normalize_scanner_quote_response(
    requested_symbol: &str,
    value: &Value,
) -> Result<Value, AppError> {
    serde_json::to_value(normalize_scanner_quote_response_typed(
        requested_symbol,
        value,
    )?)
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

fn normalize_scanner_quote_response_typed(
    requested_symbol: &str,
    value: &Value,
) -> Result<Quote, AppError> {
    let rows = value.get("data").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView scanner quote payload did not include data rows",
        )
    })?;
    if rows.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "TradingView scanner quote did not return the requested symbol",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "resolution_error": "not_found",
            "source": "scanner_scan_rest",
        })));
    }
    if rows.len() > 1 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Quote symbol is ambiguous; use EXCHANGE:SYMBOL",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "candidate_count": rows.len(),
            "candidates": scanner_quote_candidates(rows),
            "resolution_error": "ambiguous",
        })));
    }

    let row = &rows[0];
    let full_symbol = row
        .get("s")
        .and_then(Value::as_str)
        .filter(|symbol| !symbol.trim().is_empty())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "TradingView scanner quote row did not include a symbol",
            )
            .with_details(row.clone())
        })?;
    if bare_symbol(full_symbol) != bare_symbol(requested_symbol) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Scanner quote freshness check failed because the returned symbol did not match the requested symbol",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "observed_symbol": full_symbol,
            "resolution_error": "symbol_mismatch",
            "freshness_check": {
                "kind": "requested_symbol_matches_observed_symbol",
                "passed": false,
            },
        })));
    }

    let values = row.get("d").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView scanner quote row did not include quote values",
        )
        .with_details(row.clone())
    })?;
    let field = |index: usize| values.get(index).cloned().unwrap_or(Value::Null);
    let close = field(2);
    let update_mode = field(27);
    Ok(Quote {
        symbol: full_symbol.to_string(),
        time: field(26),
        last: close,
        close: field(2),
        open: field(3),
        high: field(4),
        low: field(5),
        volume: field(6),
        change: field(7),
        description: field(1),
        exchange: field(8),
        symbol_type: field(9),
        subtype: field(10),
        extended_hours: ExtendedHoursQuote {
            premarket: SessionQuote {
                open: field(11),
                high: field(12),
                low: field(13),
                last: field(14),
                close: field(14),
                change_percent: field(15),
                change_abs: field(16),
                gap_percent: Some(field(17)),
                volume: field(18),
            },
            postmarket: SessionQuote {
                open: field(19),
                high: field(20),
                low: field(21),
                last: field(22),
                close: field(22),
                change_percent: field(23),
                change_abs: field(24),
                gap_percent: None,
                volume: field(25),
            },
        },
        update_mode,
        delay_seconds: parse_update_delay_seconds(&field(27)),
        source: "scanner_scan_rest".to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_symbol: requested_symbol.to_string(),
        original_symbol: Value::Null,
        observed_symbol: full_symbol.to_string(),
        switch_performed: false,
        restored: true,
        freshness_check: FreshnessCheck {
            kind: "requested_symbol_matches_observed_symbol".to_string(),
            passed: true,
        },
    })
}

fn parse_update_delay_seconds(update_mode: &Value) -> Value {
    let Some(update_mode) = update_mode.as_str() else {
        return Value::Null;
    };
    let Some(seconds) = update_mode.strip_prefix("delayed_streaming_") else {
        return Value::Null;
    };
    seconds
        .parse::<u64>()
        .map(|seconds| json!(seconds))
        .unwrap_or(Value::Null)
}

async fn add_symbol_search_candidates(
    client: &reqwest::Client,
    mut error: AppError,
    requested_symbol: &str,
) -> AppError {
    let Ok(search) = symbol_search_with_client(client, requested_symbol).await else {
        return error;
    };
    let candidates = preferred_symbol_candidates(requested_symbol, &search);
    if let Some(details) = error.details.as_mut().and_then(Value::as_object_mut) {
        details.insert("candidate_count".to_string(), json!(candidates.len()));
        details.insert("candidates".to_string(), Value::Array(candidates));
        details.insert("candidate_source".to_string(), json!("symbol_search_rest"));
    }
    error
}

fn scanner_quote_candidates(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .take(10)
        .filter_map(|row| {
            let symbol = row.get("s").and_then(Value::as_str)?;
            let values = row.get("d").and_then(Value::as_array);
            Some(json!({
                "full_name": symbol,
                "symbol": symbol.split(':').next_back().unwrap_or(symbol),
                "exchange": symbol.split(':').next().unwrap_or_default(),
                "description": values.and_then(|values| values.get(1)).cloned().unwrap_or(Value::Null),
                "type": values.and_then(|values| values.get(9)).cloned().unwrap_or(Value::Null),
            }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use serde_json::json;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        time::{Duration, sleep},
    };

    use super::*;

    #[tokio::test]
    async fn multi_symbol_quote_is_bounded_and_preserves_order() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let server_active = Arc::clone(&active);
        let server_max_active = Arc::clone(&max_active);
        let server = tokio::spawn(async move {
            let mut tasks = Vec::new();
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let active = Arc::clone(&server_active);
                let max_active = Arc::clone(&server_max_active);
                tasks.push(tokio::spawn(async move {
                    let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_active.fetch_max(now_active, Ordering::SeqCst);
                    let request = read_http_request(&mut stream).await;
                    let (symbol, description, last, delay_ms) = if request.contains("AAPL") {
                        ("AAPL", "Apple Inc.", 100.0, 80)
                    } else {
                        ("MSFT", "Microsoft", 200.0, 10)
                    };
                    sleep(Duration::from_millis(delay_ms)).await;
                    let payload = json!({
                        "data": [{
                            "s": format!("NASDAQ:{symbol}"),
                            "d": [
                                symbol, description, last, last, last, last,
                                1000, 0.0, "NASDAQ", "stock", "common"
                            ]
                        }]
                    })
                    .to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        payload.len(),
                        payload
                    );
                    stream.write_all(response.as_bytes()).await.unwrap();
                    active.fetch_sub(1, Ordering::SeqCst);
                }));
            }
            for task in tasks {
                task.await.unwrap();
            }
        });

        let client = crate::http::configured_client().unwrap();
        let result = quote_symbols_typed_with_client_and_url(
            &client,
            vec!["NASDAQ:AAPL".to_string(), "NASDAQ:MSFT".to_string()],
            &format!("http://{address}/scan"),
        )
        .await
        .unwrap();

        assert_eq!(result.resolved_count, 2);
        assert_eq!(result.items[0].requested_index, 0);
        assert_eq!(result.items[0].requested_symbol, "NASDAQ:AAPL");
        assert_eq!(
            result.items[0].quote.as_ref().unwrap().symbol,
            "NASDAQ:AAPL"
        );
        assert_eq!(result.items[1].requested_index, 1);
        assert_eq!(result.items[1].requested_symbol, "NASDAQ:MSFT");
        assert_eq!(
            result.items[1].quote.as_ref().unwrap().symbol,
            "NASDAQ:MSFT"
        );
        server.await.unwrap();
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert_eq!(max_active.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_all_failed_quotes_keep_first_input_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let mut tasks = Vec::new();
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().await.unwrap();
                tasks.push(tokio::spawn(async move {
                    let request = read_http_request(&mut stream).await;
                    let (delay_ms, response) = if request.contains("FIRST") {
                        (
                            80,
                            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                                .to_string(),
                        )
                    } else {
                        let body = json!({"data": []}).to_string();
                        (
                            10,
                            format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                body.len(),
                                body
                            ),
                        )
                    };
                    sleep(Duration::from_millis(delay_ms)).await;
                    stream.write_all(response.as_bytes()).await.unwrap();
                }));
            }
            for task in tasks {
                task.await.unwrap();
            }
        });

        let client = crate::http::configured_client().unwrap();
        let error = quote_symbols_typed_with_client_and_url(
            &client,
            vec!["NASDAQ:FIRST".to_string(), "NASDAQ:SECOND".to_string()],
            &format!("http://{address}/scan"),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert_eq!(details["items"][0]["requested_index"], 0);
        assert_eq!(details["items"][0]["requested_symbol"], "NASDAQ:FIRST");
        assert_eq!(details["items"][1]["requested_index"], 1);
        assert_eq!(details["items"][1]["requested_symbol"], "NASDAQ:SECOND");
        server.await.unwrap();
    }

    async fn read_http_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut buffer = [0u8; 512];
        let (header_end, content_length) = loop {
            let count = stream.read(&mut buffer).await.unwrap();
            assert!(count > 0, "client closed before a complete request");
            request.extend_from_slice(&buffer[..count]);
            if let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                break (header_end + 4, content_length);
            }
        };
        while request.len() < header_end + content_length {
            let count = stream.read(&mut buffer).await.unwrap();
            assert!(count > 0, "client closed before the request body completed");
            request.extend_from_slice(&buffer[..count]);
        }
        String::from_utf8(request).unwrap()
    }

    #[test]
    fn normalize_scanner_quote_response_returns_non_mutating_quote_payload() {
        let payload = json!({
            "totalCount": 1,
            "data": [{
                "s": "NASDAQ:AAPL",
                "d": [
                    "AAPL",
                    "Apple Inc.",
                    266.39,
                    266.09,
                    268.36,
                    265.07,
                    16427115,
                    -1.72,
                    "NASDAQ",
                    "stock",
                    "common",
                    269.81,
                    269.98,
                    267.85,
                    268.2,
                    -0.9271914594953977,
                    -2.509999999999991,
                    -0.3324590890620876,
                    174665,
                    null,
                    null,
                    null,
                    null,
                    null,
                    null,
                    null,
                    1777469400,
                    "delayed_streaming_900"
                ]
            }]
        });

        let result = normalize_scanner_quote_response("AAPL", &payload).unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["last"], 266.39);
        assert_eq!(result["close"], 266.39);
        assert_eq!(result["description"], "Apple Inc.");
        assert_eq!(result["source"], "scanner_scan_rest");
        assert_eq!(result["source_category"], "desktop_free_read");
        assert_eq!(result["requires_desktop"], false);
        assert_eq!(result["time"], 1777469400);
        assert_eq!(result["update_mode"], "delayed_streaming_900");
        assert_eq!(result["delay_seconds"], 900);
        assert_eq!(result["non_mutating"], true);
        assert_eq!(result["switch_performed"], false);
        assert_eq!(result["restored"], true);
        assert_eq!(result["freshness_check"]["passed"], true);
        assert_eq!(result["extended_hours"]["premarket"]["open"], 269.81);
        assert_eq!(result["extended_hours"]["premarket"]["last"], 268.2);
        assert_eq!(result["extended_hours"]["premarket"]["close"], 268.2);
        assert_eq!(
            result["extended_hours"]["premarket"]["change_percent"],
            -0.9271914594953977
        );
        assert_eq!(
            result["extended_hours"]["premarket"]["change_abs"],
            -2.509999999999991
        );
        assert_eq!(
            result["extended_hours"]["premarket"]["gap_percent"],
            -0.3324590890620876
        );
        assert_eq!(result["extended_hours"]["premarket"]["volume"], 174665);
        assert_eq!(result["extended_hours"]["postmarket"]["last"], Value::Null);
        assert_eq!(
            result["extended_hours"]["postmarket"]["change_percent"],
            Value::Null
        );
    }

    #[test]
    fn normalize_scanner_quote_response_typed_preserves_feed_metadata() {
        let payload = json!({
            "data": [{
                "s": "NASDAQ:AAPL",
                "d": [
                    "AAPL", "Apple Inc.", 266.39, 266.09, 268.36, 265.07,
                    16427115, -1.72, "NASDAQ", "stock", "common",
                    null, null, null, 268.2, null, null, null, 174665,
                    null, null, null, null, null, null, null,
                    1777469400, "delayed_streaming_900"
                ]
            }]
        });

        let quote = normalize_scanner_quote_response_typed("AAPL", &payload).unwrap();

        assert_eq!(quote.symbol, "NASDAQ:AAPL");
        assert_eq!(quote.time, json!(1777469400));
        assert_eq!(quote.update_mode, json!("delayed_streaming_900"));
        assert_eq!(quote.delay_seconds, json!(900));
        assert_eq!(quote.extended_hours.premarket.last, json!(268.2));
        assert!(quote.freshness_check.passed);
    }

    #[test]
    fn normalize_scanner_quote_response_defaults_missing_extended_hours_to_null() {
        let payload = json!({
            "totalCount": 1,
            "data": [{
                "s": "NYSE:IONQ",
                "d": [
                    "IONQ",
                    "IonQ, Inc.",
                    43.1,
                    43.84,
                    44.26,
                    42.89,
                    12000000,
                    -1.68,
                    "NYSE",
                    "stock",
                    "common"
                ]
            }]
        });

        let result = normalize_scanner_quote_response("NYSE:IONQ", &payload).unwrap();

        assert_eq!(result["symbol"], "NYSE:IONQ");
        assert_eq!(result["last"], 43.1);
        assert_eq!(result["time"], Value::Null);
        assert_eq!(result["update_mode"], Value::Null);
        assert_eq!(result["delay_seconds"], Value::Null);
        assert_eq!(result["extended_hours"]["premarket"]["last"], Value::Null);
        assert_eq!(
            result["extended_hours"]["premarket"]["gap_percent"],
            Value::Null
        );
        assert_eq!(
            result["extended_hours"]["postmarket"]["volume"],
            Value::Null
        );
    }

    #[test]
    fn parse_update_delay_seconds_handles_known_delayed_mode_only() {
        assert_eq!(
            parse_update_delay_seconds(&json!("delayed_streaming_900")),
            json!(900)
        );
        assert_eq!(parse_update_delay_seconds(&json!("streaming")), Value::Null);
        assert_eq!(
            parse_update_delay_seconds(&json!("delayed_streaming_unknown")),
            Value::Null
        );
        assert_eq!(parse_update_delay_seconds(&Value::Null), Value::Null);
    }

    #[test]
    fn normalize_scanner_quote_response_rejects_ambiguous_symbol() {
        let payload = json!({
            "data": [
                {"s": "NASDAQ:ABC", "d": []},
                {"s": "NYSE:ABC", "d": []}
            ]
        });

        let error = normalize_scanner_quote_response("ABC", &payload).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("ambiguous"));
        assert_eq!(
            error.details.as_ref().unwrap()["resolution_error"],
            "ambiguous"
        );
        assert_eq!(
            error.details.as_ref().unwrap()["candidates"][0]["full_name"],
            "NASDAQ:ABC"
        );
    }

    #[test]
    fn normalize_scanner_quote_response_rejects_missing_symbol_as_validation() {
        let payload = json!({
            "data": []
        });

        let error = normalize_scanner_quote_response("NASDAQ:IONQ", &payload).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.details.as_ref().unwrap()["resolution_error"],
            "not_found"
        );
    }

    #[test]
    fn normalize_quote_symbols_rejects_empty_inputs_before_network() {
        assert_eq!(
            normalize_quote_symbols(Vec::new()).unwrap_err().kind,
            ErrorKind::Validation
        );
        assert_eq!(
            normalize_quote_symbols(vec![" ".to_string()])
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            normalize_quote_symbols(vec![" AAPL ".to_string(), "NYSE:IONQ".to_string()]).unwrap(),
            vec!["AAPL".to_string(), "NYSE:IONQ".to_string()]
        );
        let too_many = normalize_quote_symbols(
            (0..=MAX_MULTI_SYMBOLS)
                .map(|index| format!("SYM{index}"))
                .collect(),
        )
        .unwrap_err();
        assert_eq!(too_many.kind, ErrorKind::Validation);
        assert_eq!(too_many.details.unwrap()["maximum"], MAX_MULTI_SYMBOLS);
    }

    #[test]
    fn finalize_quote_items_preserves_order_and_counts_for_mixed_results() {
        let quote = normalize_scanner_quote_response_typed(
            "AAPL",
            &json!({
                "data": [{
                    "s": "NASDAQ:AAPL",
                    "d": [
                        "AAPL", "Apple Inc.", 266.39, null, null, null, null, null,
                        "NASDAQ", "stock", "common"
                    ]
                }]
            }),
        )
        .unwrap();
        let items = vec![
            BatchQuoteItem {
                requested_index: 0,
                requested_symbol: "AAPL".to_string(),
                ok: true,
                quote: Some(quote),
                error: None,
            },
            BatchQuoteItem {
                requested_index: 1,
                requested_symbol: "BANANA".to_string(),
                ok: false,
                quote: None,
                error: Some(QuoteError {
                    kind: ErrorKind::Validation,
                    message: "missing".to_string(),
                    details: Some(json!({ "resolution_error": "not_found" })),
                }),
            },
        ];

        let payload = finalize_quote_items(2, 1, items, None).unwrap();
        let payload = serde_json::to_value(payload).unwrap();

        assert_eq!(payload["source"], "scanner_scan_rest");
        assert_eq!(payload["source_category"], "desktop_free_read");
        assert_eq!(payload["requires_desktop"], false);
        assert_eq!(payload["non_mutating"], true);
        assert_eq!(payload["requested_count"], 2);
        assert_eq!(payload["resolved_count"], 1);
        assert_eq!(payload["error_count"], 1);
        assert_eq!(payload["items"][0]["requested_symbol"], "AAPL");
        assert_eq!(payload["items"][0]["requested_index"], 0);
        assert_eq!(payload["items"][0]["quote"]["source"], "scanner_scan_rest");
        assert_eq!(
            payload["items"][0]["quote"]["source_category"],
            "desktop_free_read"
        );
        assert_eq!(payload["items"][0]["quote"]["requires_desktop"], false);
        assert_eq!(payload["items"][0]["quote"]["non_mutating"], true);
        assert_eq!(payload["items"][1]["requested_symbol"], "BANANA");
        assert_eq!(payload["items"][1]["requested_index"], 1);
        assert_eq!(payload["items"][1]["error"]["kind"], "validation");
    }

    #[test]
    fn finalize_quote_items_returns_typed_batch_result() {
        let items = vec![BatchQuoteItem {
            requested_index: 0,
            requested_symbol: "BANANA".to_string(),
            ok: false,
            quote: None,
            error: Some(QuoteError {
                kind: ErrorKind::Validation,
                message: "missing".to_string(),
                details: Some(json!({ "resolution_error": "not_found" })),
            }),
        }];

        let error = finalize_quote_items(
            1,
            0,
            items,
            Some(AppError::new(ErrorKind::Validation, "missing")),
        )
        .unwrap_err();

        assert_eq!(error.details.as_ref().unwrap()["items"][0]["ok"], false);
        assert_eq!(
            error.details.as_ref().unwrap()["items"][0]["error"]["details"]["resolution_error"],
            "not_found"
        );
    }

    #[test]
    fn finalize_quote_items_returns_error_with_ordered_details_when_all_fail() {
        let items = vec![BatchQuoteItem {
            requested_index: 0,
            requested_symbol: "BANANA".to_string(),
            ok: false,
            quote: None,
            error: Some(QuoteError {
                kind: ErrorKind::Validation,
                message: "missing".to_string(),
                details: Some(json!({ "resolution_error": "not_found" })),
            }),
        }];
        let first_error = AppError::new(ErrorKind::Validation, "missing");

        let error = finalize_quote_items(1, 0, items, Some(first_error)).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.as_ref().unwrap()["requested_count"], 1);
        assert_eq!(error.details.as_ref().unwrap()["resolved_count"], 0);
        assert_eq!(error.details.as_ref().unwrap()["error_count"], 1);
        assert_eq!(
            error.details.as_ref().unwrap()["items"][0]["requested_symbol"],
            "BANANA"
        );
    }
}
