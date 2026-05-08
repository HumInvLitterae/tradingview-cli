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

#[cfg(not(test))]
const QUOTE_READINESS_TIMEOUT: Duration = Duration::from_millis(3_000);
#[cfg(test)]
const QUOTE_READINESS_TIMEOUT: Duration = Duration::from_millis(100);
#[cfg(not(test))]
const QUOTE_READINESS_INTERVAL: Duration = Duration::from_millis(150);
#[cfg(test)]
const QUOTE_READINESS_INTERVAL: Duration = Duration::from_millis(0);
#[cfg(not(test))]
const QUOTE_READINESS_MAX_POLLS: usize = 100;
#[cfg(test)]
const QUOTE_READINESS_MAX_POLLS: usize = 4;
const QUOTE_STABLE_SAMPLES_REQUIRED: usize = 2;

pub async fn quote(
    runtime: &mut impl RuntimeEvaluator,
    symbol: Option<&str>,
) -> Result<Value, AppError> {
    let requested_symbol = symbol
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string);

    let Some(requested_symbol) = requested_symbol else {
        let sample = read_current_quote_sample(runtime).await?;
        let mut quote = sample.quote;
        let observed_symbol = sample.quote_symbol;
        add_quote_metadata(
            &mut quote,
            QuoteMetadata {
                requested_symbol: None,
                original_symbol: None,
                observed_symbol,
                switch_performed: false,
                restored: true,
                freshness_check: None,
            },
        );
        return Ok(quote);
    };

    let _lock = QuoteSymbolLock::acquire().await?;
    let original_sample = read_current_quote_sample(runtime).await?;
    let original_symbol = original_sample.quote_symbol.clone().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Could not determine current chart symbol before quote switch",
        )
    })?;
    let original_signature = original_sample.signature.clone();
    let switch_performed = bare_symbol(&original_symbol) != bare_symbol(&requested_symbol);

    let quote_readiness = if switch_performed {
        read_requested_quote_with_readiness(
            runtime,
            &requested_symbol,
            &original_symbol,
            original_signature.as_ref(),
        )
        .await
    } else {
        Ok(QuoteReadiness {
            quote: original_sample.quote,
            observed_symbol: Some(original_symbol.clone()),
            polls: 0,
            freshness_check: json!({
                "kind": "current_chart_quote_read",
                "passed": true,
                "attempts": 0,
                "polls": 0,
                "elapsed_ms": 0,
                "stable_samples_required": 1,
                "stable_samples_seen": 1,
                "chart_symbol_matched": true,
                "quote_symbol_matched": true,
                "bar_signature_changed": false,
                "bar_values_available": original_signature.is_some(),
            }),
        })
    };
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

    let quote_readiness = match quote_readiness {
        Ok(readiness) => readiness,
        Err(err) => {
            return Err(add_restore_details_to_readiness_error(
                err,
                &requested_symbol,
                &original_symbol,
                switch_performed,
                restored,
                &restore_observed,
            ));
        }
    };

    if !restored {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Quote command could not restore the original chart symbol",
        )
        .with_details(json!({
            "source": "chart_api",
            "source_category": "desktop_backed_read",
            "requires_desktop": true,
            "non_mutating": !switch_performed,
            "session_boundary": chart_quote_session_boundary(),
            "requested_symbol": requested_symbol,
            "original_symbol": original_symbol,
            "observed_symbol": restore_observed,
            "switch_performed": switch_performed,
            "restored": false,
            "freshness_check": quote_readiness.freshness_check,
        })));
    }

    let mut quote = quote_readiness.quote;
    let observed_symbol = quote_readiness.observed_symbol;
    ensure_quote_matches_request(
        &requested_symbol,
        observed_symbol.as_deref(),
        switch_performed,
        restored,
    )?;
    add_quote_metadata(
        &mut quote,
        QuoteMetadata {
            requested_symbol: Some(requested_symbol),
            original_symbol: Some(original_symbol),
            observed_symbol,
            switch_performed,
            restored,
            freshness_check: Some(quote_readiness.freshness_check),
        },
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
                    var chartSymbol = "";
                    try {{ chartSymbol = chart.symbol(); }} catch(e) {{}}
                    try {{ ext = chart.symbolExt() || {{}}; }} catch(e) {{}}
                    var quote = {{
                        symbol: chartSymbol,
                        chart_symbol: chartSymbol,
                        bar_index: null,
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
                        var lastIndex = bars.lastIndex();
                        quote.bar_index = lastIndex;
                        var last = bars.valueAt(lastIndex);
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

async fn read_current_quote_sample(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<QuoteSample, AppError> {
    let quote = read_current_quote(runtime).await?;
    Ok(QuoteSample::from_quote(quote))
}

fn quote_symbol(quote: &Value) -> Option<String> {
    quote
        .get("symbol")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
}

fn chart_symbol(quote: &Value) -> Option<String> {
    quote
        .get("chart_symbol")
        .and_then(Value::as_str)
        .or_else(|| quote.get("symbol").and_then(Value::as_str))
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
}

async fn read_requested_quote_with_readiness(
    runtime: &mut impl RuntimeEvaluator,
    requested_symbol: &str,
    original_symbol: &str,
    original_signature: Option<&QuoteBarSignature>,
) -> Result<QuoteReadiness, AppError> {
    let started = Instant::now();
    let mut total_polls = 0usize;
    let mut last_timeout: Option<ReadinessTimeout> = None;

    for attempt in 1..=2 {
        if let Err(err) = switch_quote_symbol(runtime, requested_symbol, "requested").await {
            let _ = switch_quote_symbol(runtime, original_symbol, "original").await;
            return Err(err);
        }

        match wait_for_quote_readiness(
            runtime,
            requested_symbol,
            original_signature,
            attempt,
            Instant::now(),
            started,
        )
        .await
        {
            Ok(mut readiness) => {
                total_polls += readiness.polls;
                readiness.freshness_check["polls"] = json!(total_polls);
                return Ok(readiness);
            }
            Err(timeout) => {
                total_polls += timeout.polls;
                last_timeout = Some(timeout.with_total_polls(total_polls));
            }
        }
    }

    let timeout = last_timeout.unwrap_or_else(|| ReadinessTimeout {
        quote_symbol: None,
        chart_symbol: None,
        polls: total_polls,
        elapsed_ms: elapsed_ms(started),
        last_signature: None,
        bar_values_available: false,
        bar_signature_changed: false,
        quote_symbol_matched: false,
        chart_symbol_matched: false,
        stable_samples_seen: 0,
    });
    Err(readiness_timeout_error(
        requested_symbol,
        original_symbol,
        timeout,
    ))
}

async fn wait_for_quote_readiness(
    runtime: &mut impl RuntimeEvaluator,
    requested_symbol: &str,
    original_signature: Option<&QuoteBarSignature>,
    attempt: usize,
    attempt_started: Instant,
    overall_started: Instant,
) -> Result<QuoteReadiness, ReadinessTimeout> {
    let mut polls = 0usize;
    let mut last_quote_symbol = None;
    let mut last_chart_symbol = None;
    let mut last_signature = None;
    let mut last_bar_values_available = false;
    let mut last_bar_signature_changed = false;
    let mut last_quote_symbol_matched = false;
    let mut last_chart_symbol_matched = false;
    let mut stable_samples_seen = 0usize;

    loop {
        polls += 1;
        if let Ok(sample) = read_current_quote_sample(runtime).await {
            let observed_symbol = sample.quote_symbol.clone();
            let chart_symbol = sample.chart_symbol.clone();
            let signature = sample.signature.clone();
            let bar_values_available = signature.is_some();
            let bar_signature_changed = original_signature
                .zip(signature.as_ref())
                .is_none_or(|(before, after)| before != after);
            let quote_symbol_matches = observed_symbol
                .as_deref()
                .is_some_and(|observed| bare_symbol(observed) == bare_symbol(requested_symbol));
            let chart_symbol_matches = chart_symbol
                .as_deref()
                .is_some_and(|observed| bare_symbol(observed) == bare_symbol(requested_symbol));

            last_quote_symbol = observed_symbol.clone();
            last_chart_symbol = chart_symbol.clone();
            last_signature = signature;
            last_bar_values_available = bar_values_available;
            last_bar_signature_changed = bar_signature_changed;
            last_quote_symbol_matched = quote_symbol_matches;
            last_chart_symbol_matched = chart_symbol_matches;

            if quote_symbol_matches
                && chart_symbol_matches
                && bar_values_available
                && bar_signature_changed
            {
                stable_samples_seen += 1;
            } else {
                stable_samples_seen = 0;
            }

            if stable_samples_seen >= QUOTE_STABLE_SAMPLES_REQUIRED {
                return Ok(QuoteReadiness {
                    quote: sample.quote,
                    observed_symbol,
                    polls,
                    freshness_check: json!({
                        "kind": "stable_requested_chart_quote_and_new_bars",
                        "passed": true,
                        "attempts": attempt,
                        "polls": polls,
                        "elapsed_ms": elapsed_ms(overall_started),
                        "stable_samples_required": QUOTE_STABLE_SAMPLES_REQUIRED,
                        "stable_samples_seen": stable_samples_seen,
                        "chart_symbol_matched": true,
                        "quote_symbol_matched": true,
                        "bar_signature_changed": true,
                        "bar_values_available": true,
                        "chart_symbol": chart_symbol,
                    }),
                });
            }
        }

        if polls >= QUOTE_READINESS_MAX_POLLS
            || attempt_started.elapsed() >= QUOTE_READINESS_TIMEOUT
        {
            return Err(ReadinessTimeout {
                quote_symbol: last_quote_symbol,
                chart_symbol: last_chart_symbol,
                polls,
                elapsed_ms: elapsed_ms(overall_started),
                last_signature,
                bar_values_available: last_bar_values_available,
                bar_signature_changed: last_bar_signature_changed,
                quote_symbol_matched: last_quote_symbol_matched,
                chart_symbol_matched: last_chart_symbol_matched,
                stable_samples_seen,
            });
        }

        sleep(QUOTE_READINESS_INTERVAL).await;
    }
}

fn readiness_timeout_error(
    requested_symbol: &str,
    original_symbol: &str,
    timeout: ReadinessTimeout,
) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Quote freshness check timed out before chart bars reflected the requested symbol",
    )
    .with_details(json!({
        "source": "chart_api",
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": false,
        "session_boundary": chart_quote_session_boundary(),
        "requested_symbol": requested_symbol,
        "original_symbol": original_symbol,
        "observed_symbol": timeout.quote_symbol,
        "chart_symbol": timeout.chart_symbol,
        "attempts": 2,
        "polls": timeout.polls,
        "elapsed_ms": timeout.elapsed_ms,
        "freshness_check": {
            "kind": "stable_requested_chart_quote_and_new_bars",
            "passed": false,
            "bar_values_available": timeout.bar_values_available,
            "bar_signature_changed": timeout.bar_signature_changed,
            "chart_symbol_matched": timeout.chart_symbol_matched,
            "quote_symbol_matched": timeout.quote_symbol_matched,
            "stable_samples_required": QUOTE_STABLE_SAMPLES_REQUIRED,
            "stable_samples_seen": timeout.stable_samples_seen,
            "last_bar_signature": timeout.last_signature.map(|signature| signature.to_diagnostic()),
        },
        "next_action_hint": "The chart source did not become fresh in time. Retry the command or use `--source scanner` if scanner feed freshness is acceptable.",
    }))
}

fn add_restore_details_to_readiness_error(
    mut err: AppError,
    requested_symbol: &str,
    original_symbol: &str,
    switch_performed: bool,
    restored: bool,
    restore_observed: &str,
) -> AppError {
    let mut details = err.details.take().unwrap_or_else(|| json!({}));
    merge_object(
        &mut details,
        json!({
            "source": "chart_api",
            "source_category": "desktop_backed_read",
            "requires_desktop": true,
            "non_mutating": !switch_performed,
            "session_boundary": chart_quote_session_boundary(),
            "requested_symbol": requested_symbol,
            "original_symbol": original_symbol,
            "restore_observed_symbol": restore_observed,
            "switch_performed": switch_performed,
            "restored": restored,
        }),
    );
    err.with_details(details)
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
            "source": "chart_api",
            "source_category": "desktop_backed_read",
            "requires_desktop": true,
            "non_mutating": false,
            "session_boundary": chart_quote_session_boundary(),
            "requested_symbol": symbol,
            "observed_symbol": observed,
            "phase": phase,
            "raw": value,
        })))
    }
}

struct QuoteMetadata {
    requested_symbol: Option<String>,
    original_symbol: Option<String>,
    observed_symbol: Option<String>,
    switch_performed: bool,
    restored: bool,
    freshness_check: Option<Value>,
}

fn add_quote_metadata(quote: &mut Value, metadata: QuoteMetadata) {
    merge_object(
        quote,
        json!({
            "source": "chart_api",
            "source_category": "desktop_backed_read",
            "requires_desktop": true,
            "non_mutating": !metadata.switch_performed,
            "session_boundary": chart_quote_session_boundary(),
            "requested_symbol": metadata.requested_symbol,
            "original_symbol": metadata.original_symbol,
            "observed_symbol": metadata.observed_symbol,
            "switch_performed": metadata.switch_performed,
            "restored": metadata.restored,
            "freshness_check": metadata.freshness_check,
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
        "source": "chart_api",
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": !switch_performed,
        "session_boundary": chart_quote_session_boundary(),
        "requested_symbol": requested_symbol,
        "observed_symbol": observed_symbol,
        "switch_performed": switch_performed,
        "restored": restored,
        "freshness_check": {
            "kind": "requested_symbol_matches_observed_symbol",
            "passed": false,
        },
        "next_action_hint": "Run `tv tab list` to confirm the selected chart target, then retry with `tv --target-id <ID> quote <SYMBOL> --source chart`. Use `--source scanner` only if scanner feed freshness is acceptable.",
    })))
}

fn chart_quote_session_boundary() -> Value {
    json!({
        "price_source": "selected_chart_main_series_last_bar",
        "price_session": "unknown",
        "extended_hours_status": "not_provided",
        "extended_hours_guaranteed": false,
        "scanner_extended_hours_included": false,
        "reason": "chart_source_does_not_expose_scanner_extended_hours",
    })
}

fn bare_symbol(symbol: &str) -> String {
    symbol
        .split(':')
        .next_back()
        .unwrap_or(symbol)
        .trim()
        .to_ascii_uppercase()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QuoteSample {
    quote: Value,
    quote_symbol: Option<String>,
    chart_symbol: Option<String>,
    signature: Option<QuoteBarSignature>,
}

impl QuoteSample {
    fn from_quote(quote: Value) -> Self {
        let quote_symbol = quote_symbol(&quote);
        let chart_symbol = chart_symbol(&quote);
        let signature = QuoteBarSignature::from_quote(&quote);
        Self {
            quote,
            quote_symbol,
            chart_symbol,
            signature,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QuoteBarSignature {
    bar_index: Value,
    time: Value,
    open: Value,
    high: Value,
    low: Value,
    close: Value,
    last: Value,
    volume: Value,
}

impl QuoteBarSignature {
    fn from_quote(quote: &Value) -> Option<Self> {
        let signature = Self {
            bar_index: quote.get("bar_index").cloned().unwrap_or(Value::Null),
            time: quote.get("time").cloned().unwrap_or(Value::Null),
            open: quote.get("open").cloned().unwrap_or(Value::Null),
            high: quote.get("high").cloned().unwrap_or(Value::Null),
            low: quote.get("low").cloned().unwrap_or(Value::Null),
            close: quote.get("close").cloned().unwrap_or(Value::Null),
            last: quote.get("last").cloned().unwrap_or(Value::Null),
            volume: quote.get("volume").cloned().unwrap_or(Value::Null),
        };
        if signature.has_bar_values() {
            Some(signature)
        } else {
            None
        }
    }

    fn has_bar_values(&self) -> bool {
        !self.time.is_null()
            && (!self.open.is_null()
                || !self.high.is_null()
                || !self.low.is_null()
                || !self.close.is_null()
                || !self.last.is_null())
    }

    fn to_diagnostic(&self) -> Value {
        json!({
            "bar_index": self.bar_index,
            "time": self.time,
            "open": self.open,
            "high": self.high,
            "low": self.low,
            "close": self.close,
            "last": self.last,
            "volume": self.volume,
        })
    }
}

struct QuoteReadiness {
    quote: Value,
    observed_symbol: Option<String>,
    polls: usize,
    freshness_check: Value,
}

struct ReadinessTimeout {
    quote_symbol: Option<String>,
    chart_symbol: Option<String>,
    polls: usize,
    elapsed_ms: u128,
    last_signature: Option<QuoteBarSignature>,
    bar_values_available: bool,
    bar_signature_changed: bool,
    quote_symbol_matched: bool,
    chart_symbol_matched: bool,
    stable_samples_seen: usize,
}

impl ReadinessTimeout {
    fn with_total_polls(mut self, polls: usize) -> Self {
        self.polls = polls;
        self
    }
}

fn elapsed_ms(started: Instant) -> u128 {
    started.elapsed().as_millis()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_core::ErrorKind;

    fn assert_chart_session_boundary(value: &Value) {
        assert_eq!(
            value["session_boundary"]["price_source"],
            "selected_chart_main_series_last_bar"
        );
        assert_eq!(value["session_boundary"]["price_session"], "unknown");
        assert_eq!(
            value["session_boundary"]["extended_hours_status"],
            "not_provided"
        );
        assert_eq!(
            value["session_boundary"]["extended_hours_guaranteed"],
            false
        );
        assert_eq!(
            value["session_boundary"]["scanner_extended_hours_included"],
            false
        );
        assert!(value.get("extended_hours").is_none());
    }

    #[test]
    fn bare_symbol_compares_exchange_prefixed_inputs() {
        assert_eq!(bare_symbol("NASDAQ:AAPL"), "AAPL");
        assert_eq!(bare_symbol("aapl"), "AAPL");
    }

    fn quote_payload(symbol: &str, time: i64, last: f64) -> Value {
        json!({
            "symbol": symbol,
            "time": time,
            "open": last - 1.0,
            "high": last + 1.0,
            "low": last - 2.0,
            "close": last,
            "last": last,
            "volume": (last * 100.0) as i64,
        })
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
        assert_eq!(result["source_category"], "desktop_backed_read");
        assert_eq!(result["requires_desktop"], true);
        assert_eq!(result["non_mutating"], true);
        assert_chart_session_boundary(&result);
    }

    #[tokio::test]
    async fn quote_other_symbol_switches_reads_and_restores() {
        let mut runtime = FakeRuntime::new([
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let result = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:MSFT");
        assert_eq!(result["requested_symbol"], "NASDAQ:MSFT");
        assert_eq!(result["original_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["observed_symbol"], "NASDAQ:MSFT");
        assert_eq!(result["switch_performed"], true);
        assert_eq!(result["restored"], true);
        assert_eq!(result["source_category"], "desktop_backed_read");
        assert_eq!(result["requires_desktop"], true);
        assert_eq!(result["non_mutating"], false);
        assert_eq!(result["freshness_check"]["passed"], true);
        assert_chart_session_boundary(&result);
    }

    #[tokio::test]
    async fn quote_same_symbol_skips_switch() {
        let mut runtime = FakeRuntime::new([quote_payload("AAPL", 1, 42.0)]);

        let result = quote(&mut runtime, Some("AAPL")).await.unwrap();

        assert_eq!(result["switch_performed"], false);
        assert_eq!(result["restored"], true);
        assert_eq!(runtime.evaluated.len(), 1);
    }

    #[tokio::test]
    async fn quote_other_symbol_fails_when_restore_is_not_verified() {
        let mut runtime = FakeRuntime::new([
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:MSFT"}),
        ]);

        let error = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.message.contains("original"));
    }

    #[tokio::test]
    async fn quote_requested_symbol_fails_when_observed_quote_is_stale() {
        let mut runtime = FakeRuntime::new([
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            quote_payload("NASDAQ:AAPL", 1, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let error = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.message.contains("freshness"));
        assert!(
            error.details.unwrap()["next_action_hint"]
                .as_str()
                .unwrap()
                .contains("--source scanner")
        );
    }

    #[tokio::test]
    async fn quote_waits_until_bar_signature_changes_after_symbol_switch() {
        let mut runtime = FakeRuntime::new([
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:MSFT", 1, 10.0),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let result = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:MSFT");
        assert_eq!(result["last"], 42.0);
        assert_eq!(result["freshness_check"]["attempts"], 1);
        assert_eq!(result["freshness_check"]["polls"], 3);
        assert_eq!(result["freshness_check"]["bar_signature_changed"], true);
        assert_eq!(result["freshness_check"]["stable_samples_seen"], 2);
    }

    #[tokio::test]
    async fn quote_retries_once_when_readiness_times_out() {
        let mut runtime = FakeRuntime::new([
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:MSFT", 1, 10.0),
            quote_payload("NASDAQ:MSFT", 1, 10.0),
            quote_payload("NASDAQ:MSFT", 1, 10.0),
            quote_payload("NASDAQ:MSFT", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let result = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:MSFT");
        assert_eq!(result["freshness_check"]["attempts"], 2);
        assert_eq!(result["freshness_check"]["polls"], 6);
    }

    fn quote_payload_with_chart_symbol(
        symbol: &str,
        chart_symbol: &str,
        time: i64,
        last: f64,
    ) -> Value {
        let mut payload = quote_payload(symbol, time, last);
        payload["chart_symbol"] = json!(chart_symbol);
        payload
    }

    #[tokio::test]
    async fn quote_does_not_succeed_when_quote_symbol_matches_but_chart_symbol_is_stale() {
        let mut runtime = FakeRuntime::new([
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:AAPL", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let error = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert_eq!(details["chart_symbol"], "NASDAQ:AAPL");
        assert_eq!(details["freshness_check"]["quote_symbol_matched"], true);
        assert_eq!(details["freshness_check"]["chart_symbol_matched"], false);
        assert_chart_session_boundary(&details);
    }

    #[tokio::test]
    async fn quote_does_not_succeed_when_chart_symbol_matches_but_quote_symbol_is_stale() {
        let mut runtime = FakeRuntime::new([
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let error = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert_eq!(details["observed_symbol"], "NASDAQ:AAPL");
        assert_eq!(details["freshness_check"]["quote_symbol_matched"], false);
        assert_eq!(details["freshness_check"]["chart_symbol_matched"], true);
    }

    #[tokio::test]
    async fn quote_requires_consecutive_ready_samples_before_success() {
        let mut runtime = FakeRuntime::new([
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:AAPL", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:MSFT", 2, 42.0),
            quote_payload_with_chart_symbol("NASDAQ:MSFT", "NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
        ]);

        let result = quote(&mut runtime, Some("NASDAQ:MSFT")).await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:MSFT");
        assert_eq!(result["freshness_check"]["polls"], 4);
        assert_eq!(result["freshness_check"]["stable_samples_seen"], 2);
    }
}
