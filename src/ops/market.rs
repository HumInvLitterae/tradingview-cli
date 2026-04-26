use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

use serde_json::{Value, json};
use tokio::time::sleep;

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

use super::common::{
    BARS_PATH, CHART_API, DEFAULT_OHLCV_COUNT, MAX_OHLCV_COUNT, SYMBOL_SEARCH_URL, js_string,
    merge_object, round2,
};

pub async fn symbol_search(query: &str) -> Result<Value, AppError> {
    let url = reqwest::Url::parse_with_params(
        SYMBOL_SEARCH_URL,
        &[
            ("text", query),
            ("hl", "1"),
            ("exchange", ""),
            ("lang", "en"),
            ("search_type", ""),
            ("domain", "production"),
        ],
    )
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?;
    let response = reqwest::Client::new()
        .get(url)
        .header("Origin", "https://www.tradingview.com")
        .header("Referer", "https://www.tradingview.com/")
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("Symbol search API returned {status}"),
        ));
    }

    let value = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))?;
    Ok(normalize_symbol_search_response(query, &value))
}

pub async fn quote(
    runtime: &mut impl RuntimeEvaluator,
    symbol: Option<&str>,
) -> Result<Value, AppError> {
    let requested_symbol = symbol
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string);

    let Some(requested_symbol) = requested_symbol else {
        let mut quote = read_current_quote(runtime).await?;
        let observed_symbol = quote
            .get("symbol")
            .and_then(Value::as_str)
            .map(str::to_string);
        add_quote_metadata(&mut quote, None, None, observed_symbol, false, true);
        return Ok(quote);
    };

    let _lock = QuoteSymbolLock::acquire().await?;
    let original_symbol = read_current_quote_symbol(runtime).await?;
    let switch_performed = bare_symbol(&original_symbol) != bare_symbol(&requested_symbol);

    if switch_performed
        && let Err(err) = switch_quote_symbol(runtime, &requested_symbol, "requested").await
    {
        let _ = switch_quote_symbol(runtime, &original_symbol, "original").await;
        return Err(err);
    }

    let quote_result = read_current_quote(runtime).await;
    let mut restored = !switch_performed;
    let mut restore_observed = original_symbol.clone();

    if switch_performed {
        match switch_quote_symbol(runtime, &original_symbol, "original").await {
            Ok(observed) => {
                restore_observed = observed;
                restored = bare_symbol(&restore_observed) == bare_symbol(&original_symbol);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    if !restored {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Quote command could not restore the original chart symbol",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "original_symbol": original_symbol,
            "observed_symbol": restore_observed,
            "switch_performed": switch_performed,
            "restored": false,
        })));
    }

    let mut quote = quote_result?;
    let observed_symbol = quote
        .get("symbol")
        .and_then(Value::as_str)
        .map(str::to_string);
    add_quote_metadata(
        &mut quote,
        Some(requested_symbol),
        Some(original_symbol),
        observed_symbol,
        switch_performed,
        restored,
    );
    Ok(quote)
}

struct QuoteSymbolLock {
    path: PathBuf,
}

impl QuoteSymbolLock {
    async fn acquire() -> Result<Self, AppError> {
        let path = std::env::temp_dir().join("tradingview-cli-quote-symbol.lock");
        let started = Instant::now();
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    let _ = writeln!(file, "{}", std::process::id());
                    return Ok(Self { path });
                }
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    remove_stale_quote_lock(&path);
                    if started.elapsed() >= Duration::from_secs(15) {
                        return Err(AppError::new(
                            ErrorKind::InternalApiUnavailable,
                            "Timed out waiting for another symbol-targeted quote command to finish",
                        )
                        .with_details(json!({
                            "lock": "quote_symbol",
                            "timeout_ms": 15_000,
                        })));
                    }
                    sleep(Duration::from_millis(100)).await;
                }
                Err(err) => {
                    return Err(AppError::new(
                        ErrorKind::InternalApiUnavailable,
                        format!("Could not create quote symbol lock: {err}"),
                    ));
                }
            }
        }
    }
}

impl Drop for QuoteSymbolLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn remove_stale_quote_lock(path: &PathBuf) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    let Ok(modified) = metadata.modified() else {
        return;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return;
    };
    if age > Duration::from_secs(30) {
        let _ = fs::remove_file(path);
    }
}

async fn read_current_quote(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var bars = {BARS_PATH};
                    var ext = {{}};
                    try {{ ext = chart.symbolExt() || {{}}; }} catch(e) {{}}
                    var quote = {{
                        symbol: chart.symbol(),
                        time: null,
                        last: null,
                        close: null,
                        open: null,
                        high: null,
                        low: null,
                        volume: null,
                        description: ext.description || null,
                        exchange: ext.exchange || null,
                        type: ext.type || null
                    }};
                    if (bars && typeof bars.lastIndex === 'function') {{
                        var last = bars.valueAt(bars.lastIndex());
                        if (last) {{
                            quote.time = last[0];
                            quote.open = last[1];
                            quote.high = last[2];
                            quote.low = last[3];
                            quote.close = last[4];
                            quote.last = last[4];
                            quote.volume = last[5] || 0;
                        }}
                    }}
                    return quote;
                }})()
                "#
            ),
            false,
        )
        .await
}

async fn read_current_quote_symbol(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<String, AppError> {
    let value = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    try {{ return chart.symbol(); }} catch(e) {{}}
                    try {{
                        var ext = chart.symbolExt() || {{}};
                        return ext.symbol || "";
                    }} catch(e) {{}}
                    return "";
                }})()
                "#
            ),
            false,
        )
        .await?;
    value
        .as_str()
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Could not determine current chart symbol before quote switch",
            )
        })
}

async fn switch_quote_symbol(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
    phase: &str,
) -> Result<String, AppError> {
    let symbol_literal = js_string(symbol)?;
    let value = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var requested = {symbol_literal};
                    var chart = {CHART_API};
                    return new Promise(function(resolve) {{
                        chart.setSymbol(requested, {{}});
                        setTimeout(function() {{
                            var observed = "";
                            try {{ observed = chart.symbol(); }} catch(e) {{}}
                            resolve({{
                                requested_symbol: requested,
                                observed_symbol: observed,
                                chart_ready: String(observed).toUpperCase().indexOf(String(requested).split(":").pop().toUpperCase()) >= 0
                            }});
                        }}, 800);
                    }});
                }})()
                "#
            ),
            true,
        )
        .await?;

    let observed = value
        .get("observed_symbol")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if bare_symbol(&observed) == bare_symbol(symbol) {
        Ok(observed)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Quote command could not switch chart symbol during {phase} phase"),
        )
        .with_details(json!({
            "requested_symbol": symbol,
            "observed_symbol": observed,
            "phase": phase,
            "raw": value,
        })))
    }
}

fn add_quote_metadata(
    quote: &mut Value,
    requested_symbol: Option<String>,
    original_symbol: Option<String>,
    observed_symbol: Option<String>,
    switch_performed: bool,
    restored: bool,
) {
    merge_object(
        quote,
        json!({
            "requested_symbol": requested_symbol,
            "original_symbol": original_symbol,
            "observed_symbol": observed_symbol,
            "switch_performed": switch_performed,
            "restored": restored,
        }),
    );
}

fn bare_symbol(symbol: &str) -> String {
    symbol
        .split(':')
        .next_back()
        .unwrap_or(symbol)
        .trim()
        .to_ascii_uppercase()
}

pub async fn ohlcv_bars(
    runtime: &mut impl RuntimeEvaluator,
    count: Option<usize>,
) -> Result<Value, AppError> {
    let limit = normalized_count(count);
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var bars = {BARS_PATH};
                    if (!bars || typeof bars.lastIndex !== 'function') {{
                        throw new Error('Could not extract OHLCV data. The chart may still be loading.');
                    }}
                    var result = [];
                    var end = bars.lastIndex();
                    var start = Math.max(bars.firstIndex(), end - {limit} + 1);
                    for (var i = start; i <= end; i++) {{
                        var v = bars.valueAt(i);
                        if (v) result.push({{time: v[0], open: v[1], high: v[2], low: v[3], close: v[4], volume: v[5] || 0}});
                    }}
                    if (result.length === 0) {{
                        throw new Error('Could not extract OHLCV data. The chart may still be loading.');
                    }}
                    return {{
                        symbol: chart.symbol(),
                        resolution: chart.resolution(),
                        timeframe: chart.resolution(),
                        bar_count: result.length,
                        total_available: (typeof bars.size === 'function') ? bars.size() : null,
                        source: "direct_bars",
                        bars: result
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn ohlcv_summary(
    runtime: &mut impl RuntimeEvaluator,
    count: Option<usize>,
) -> Result<Value, AppError> {
    let data = ohlcv_bars(runtime, count).await?;
    summarize_ohlcv(data)
}
fn normalized_count(count: Option<usize>) -> usize {
    count
        .unwrap_or(DEFAULT_OHLCV_COUNT)
        .clamp(1, MAX_OHLCV_COUNT)
}

fn summarize_ohlcv(data: Value) -> Result<Value, AppError> {
    let bars = data.get("bars").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "OHLCV data did not include bars",
        )
    })?;
    let first = bars.first().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Could not extract OHLCV data. The chart may still be loading.",
        )
    })?;
    let last = bars.last().expect("non-empty bars should have last bar");
    let highs = bars
        .iter()
        .filter_map(|bar| bar.get("high").and_then(Value::as_f64))
        .collect::<Vec<_>>();
    let lows = bars
        .iter()
        .filter_map(|bar| bar.get("low").and_then(Value::as_f64))
        .collect::<Vec<_>>();
    let volumes = bars
        .iter()
        .filter_map(|bar| bar.get("volume").and_then(Value::as_f64))
        .collect::<Vec<_>>();
    let open = first.get("open").and_then(Value::as_f64).unwrap_or(0.0);
    let close = last.get("close").and_then(Value::as_f64).unwrap_or(0.0);
    let high = highs.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let low = lows.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let volume: f64 = volumes.iter().sum();
    let avg_volume = if volumes.is_empty() {
        0.0
    } else {
        (volume / volumes.len() as f64).round()
    };
    let change = round2(close - open);
    let change_pct = if open == 0.0 {
        "0%".to_string()
    } else {
        format!("{}%", round2(((close - open) / open) * 100.0))
    };
    let last_5_bars = bars
        .iter()
        .skip(bars.len().saturating_sub(5))
        .cloned()
        .collect::<Vec<_>>();
    let mut summary = json!({
        "bar_count": bars.len(),
        "period": {
            "from": first.get("time").cloned().unwrap_or(Value::Null),
            "to": last.get("time").cloned().unwrap_or(Value::Null),
        },
        "first_time": first.get("time").cloned().unwrap_or(Value::Null),
        "last_time": last.get("time").cloned().unwrap_or(Value::Null),
        "open": open,
        "close": close,
        "high": high,
        "low": low,
        "range": round2(high - low),
        "change": change,
        "change_pct": change_pct,
        "avg_volume": avg_volume,
        "volume": volume,
        "last_5_bars": last_5_bars,
    });
    for field in ["symbol", "resolution", "timeframe"] {
        if let Some(value) = data.get(field) {
            summary[field] = value.clone();
        }
    }
    Ok(summary)
}
pub(super) fn normalize_symbol_search_response(query: &str, value: &Value) -> Value {
    let rows = value
        .get("symbols")
        .and_then(Value::as_array)
        .or_else(|| value.as_array())
        .cloned()
        .unwrap_or_default();
    let results = rows
        .into_iter()
        .take(15)
        .map(|row| {
            let symbol = strip_em(
                row.get("symbol")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let description = strip_em(
                row.get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let exchange = row
                .get("exchange")
                .or_else(|| row.get("prefix"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let symbol_type = row
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let full_name = if exchange.is_empty() {
                symbol.clone()
            } else {
                format!("{exchange}:{symbol}")
            };
            json!({
                "symbol": symbol,
                "description": description,
                "exchange": exchange,
                "type": symbol_type,
                "full_name": full_name,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "query": query,
        "source": "rest_api",
        "count": results.len(),
        "results": results,
    })
}

fn strip_em(value: &str) -> String {
    value.replace("<em>", "").replace("</em>", "")
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[test]
    fn normalize_symbol_search_response_handles_object_and_em_tags() {
        let response = json!({
            "symbols": [{
                "symbol": "<em>AAPL</em>",
                "description": "Apple <em>Inc</em>",
                "exchange": "NASDAQ",
                "type": "stock"
            }]
        });

        let result = normalize_symbol_search_response("AAPL", &response);

        assert_eq!(result["query"], "AAPL");
        assert_eq!(result["source"], "rest_api");
        assert_eq!(result["count"], 1);
        assert_eq!(result["results"][0]["symbol"], "AAPL");
        assert_eq!(result["results"][0]["description"], "Apple Inc");
        assert_eq!(result["results"][0]["full_name"], "NASDAQ:AAPL");
    }

    #[tokio::test]
    async fn ohlcv_count_is_clamped_to_500() {
        let mut runtime = FakeRuntime::new([
            json!({"symbol": "NASDAQ:AAPL", "timeframe": "D", "bars": [{"time": 1, "open": 1, "high": 1, "low": 1, "close": 1, "volume": 1}]}),
        ]);

        let _ = ohlcv_bars(&mut runtime, Some(900)).await;

        assert!(runtime.evaluated[0].0.contains("end - 500 + 1"));
    }

    #[tokio::test]
    async fn quote_without_symbol_reads_current_chart_only() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "last": 200.0,
            "close": 200.0
        })]);

        let result = quote(&mut runtime, None).await.unwrap();

        assert_eq!(runtime.evaluated.len(), 1);
        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["requested_symbol"], Value::Null);
        assert_eq!(result["original_symbol"], Value::Null);
        assert_eq!(result["observed_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["switch_performed"], false);
        assert_eq!(result["restored"], true);
    }

    #[tokio::test]
    async fn quote_same_symbol_skips_switch() {
        let mut runtime = FakeRuntime::new([
            json!("NASDAQ:AAPL"),
            json!({
                "symbol": "NASDAQ:AAPL",
                "last": 200.0,
                "close": 200.0
            }),
        ]);

        let result = quote(&mut runtime, Some("AAPL")).await.unwrap();

        assert_eq!(runtime.evaluated.len(), 2);
        assert!(
            !runtime
                .evaluated
                .iter()
                .any(|(expr, _)| expr.contains("setSymbol"))
        );
        assert_eq!(result["requested_symbol"], "AAPL");
        assert_eq!(result["original_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["observed_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["switch_performed"], false);
        assert_eq!(result["restored"], true);
    }

    #[tokio::test]
    async fn quote_other_symbol_switches_reads_and_restores() {
        let mut runtime = FakeRuntime::new([
            json!("NASDAQ:AAPL"),
            json!({"observed_symbol": "NASDAQ:MSFT"}),
            json!({
                "symbol": "NASDAQ:MSFT",
                "last": 300.0,
                "close": 300.0
            }),
            json!({"observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let result = quote(&mut runtime, Some("MSFT")).await.unwrap();

        assert_eq!(runtime.evaluated.len(), 4);
        assert!(runtime.evaluated[1].0.contains("setSymbol"));
        assert!(runtime.evaluated[1].0.contains("\"MSFT\""));
        assert!(runtime.evaluated[3].0.contains("setSymbol"));
        assert!(runtime.evaluated[3].0.contains("\"NASDAQ:AAPL\""));
        assert_eq!(result["symbol"], "NASDAQ:MSFT");
        assert_eq!(result["requested_symbol"], "MSFT");
        assert_eq!(result["original_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["observed_symbol"], "NASDAQ:MSFT");
        assert_eq!(result["switch_performed"], true);
        assert_eq!(result["restored"], true);
    }

    #[tokio::test]
    async fn quote_other_symbol_fails_when_restore_is_not_verified() {
        let mut runtime = FakeRuntime::new([
            json!("NASDAQ:AAPL"),
            json!({"observed_symbol": "NASDAQ:MSFT"}),
            json!({
                "symbol": "NASDAQ:MSFT",
                "last": 300.0,
                "close": 300.0
            }),
            json!({"observed_symbol": "NASDAQ:MSFT"}),
        ]);

        let error = quote(&mut runtime, Some("MSFT")).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.details.as_ref().unwrap()["requested_symbol"],
            "NASDAQ:AAPL"
        );
        assert_eq!(
            error.details.as_ref().unwrap()["observed_symbol"],
            "NASDAQ:MSFT"
        );
        assert_eq!(error.details.as_ref().unwrap()["phase"], "original");
    }

    #[test]
    fn bare_symbol_compares_exchange_prefixed_inputs() {
        assert_eq!(bare_symbol("NASDAQ:AAPL"), bare_symbol("AAPL"));
        assert_eq!(bare_symbol("nyse:brk.b"), "BRK.B");
    }

    #[tokio::test]
    async fn ohlcv_summary_returns_legacy_practical_fields() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "timeframe": "D",
            "bars": [
                {"time": 10, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 100.0},
                {"time": 20, "open": 11.0, "high": 13.0, "low": 10.0, "close": 12.0, "volume": 200.0}
            ]
        })]);

        let summary = ohlcv_summary(&mut runtime, Some(2)).await.unwrap();

        assert_eq!(summary["bar_count"], 2);
        assert_eq!(summary["period"]["from"], 10);
        assert_eq!(summary["period"]["to"], 20);
        assert_eq!(summary["range"], 4.0);
        assert_eq!(summary["change"], 2.0);
        assert_eq!(summary["avg_volume"], 150.0);
        assert_eq!(summary["symbol"], "NASDAQ:AAPL");
        assert_eq!(summary["timeframe"], "D");
        assert_eq!(summary["last_5_bars"].as_array().unwrap().len(), 2);
    }
}
