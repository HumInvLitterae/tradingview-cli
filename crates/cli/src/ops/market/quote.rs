use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

use serde_json::{Value, json};
use tokio::time::sleep;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{BARS_PATH, CHART_API, js_string, merge_object};

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
    ensure_quote_matches_request(
        &requested_symbol,
        observed_symbol.as_deref(),
        switch_performed,
        restored,
    )?;
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
            "source": "chart_api",
            "requested_symbol": requested_symbol,
            "original_symbol": original_symbol,
            "observed_symbol": observed_symbol,
            "switch_performed": switch_performed,
            "restored": restored,
        }),
    );
}

fn ensure_quote_matches_request(
    requested_symbol: &str,
    observed_symbol: Option<&str>,
    switch_performed: bool,
    restored: bool,
) -> Result<(), AppError> {
    let observed_symbol = observed_symbol.unwrap_or_default();
    if bare_symbol(observed_symbol) == bare_symbol(requested_symbol) {
        return Ok(());
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Quote freshness check failed because the observed quote symbol did not match the requested symbol",
    )
    .with_details(json!({
        "requested_symbol": requested_symbol,
        "observed_symbol": observed_symbol,
        "switch_performed": switch_performed,
        "restored": restored,
        "freshness_check": {
            "kind": "requested_symbol_matches_observed_symbol",
            "passed": false,
        },
    })))
}

fn bare_symbol(symbol: &str) -> String {
    symbol
        .split(':')
        .next_back()
        .unwrap_or(symbol)
        .trim()
        .to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_core::ErrorKind;

    #[test]
    fn bare_symbol_compares_exchange_prefixed_inputs() {
        assert_eq!(bare_symbol("NASDAQ:AAPL"), "AAPL");
        assert_eq!(bare_symbol("aapl"), "AAPL");
    }

    #[tokio::test]
    async fn quote_without_symbol_reads_current_chart_only() {
        let payload = json!({"symbol": "NASDAQ:AAPL", "last": 10.0});
        let mut runtime = FakeRuntime::new([payload]);

        let result = quote(&mut runtime, None).await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["requested_symbol"], Value::Null);
        assert_eq!(result["original_symbol"], Value::Null);
        assert_eq!(result["observed_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["switch_performed"], false);
        assert_eq!(result["restored"], true);
    }

    #[tokio::test]
    async fn quote_other_symbol_switches_reads_and_restores() {
        let mut runtime = FakeRuntime::new([
            json!("NASDAQ:AAPL"),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            json!({"symbol": "NASDAQ:MSFT", "last": 42.0}),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let result = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:MSFT");
        assert_eq!(result["requested_symbol"], "NASDAQ:MSFT");
        assert_eq!(result["original_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["observed_symbol"], "NASDAQ:MSFT");
        assert_eq!(result["switch_performed"], true);
        assert_eq!(result["restored"], true);
    }

    #[tokio::test]
    async fn quote_same_symbol_skips_switch() {
        let mut runtime = FakeRuntime::new([
            json!("NASDAQ:AAPL"),
            json!({"symbol": "AAPL", "last": 42.0}),
        ]);

        let result = quote(&mut runtime, Some("AAPL")).await.unwrap();

        assert_eq!(result["switch_performed"], false);
        assert_eq!(result["restored"], true);
        assert_eq!(runtime.evaluated.len(), 2);
    }

    #[tokio::test]
    async fn quote_other_symbol_fails_when_restore_is_not_verified() {
        let mut runtime = FakeRuntime::new([
            json!("NASDAQ:AAPL"),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            json!({"symbol": "NASDAQ:MSFT", "last": 42.0}),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:MSFT"}),
        ]);

        let error = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.message.contains("original"));
    }

    #[tokio::test]
    async fn quote_requested_symbol_fails_when_observed_quote_is_stale() {
        let mut runtime = FakeRuntime::new([
            json!("NASDAQ:AAPL"),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            json!({"symbol": "NASDAQ:AAPL", "last": 42.0}),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let error = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.message.contains("freshness"));
    }
}
