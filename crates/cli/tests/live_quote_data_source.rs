use std::{
    process::Command,
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOL: &str = "NASDAQ:RKLB";

#[test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_QUOTE_DATA_SMOKE=1"]
fn quote_data_live_smoke() {
    if std::env::var("TV_LIVE_QUOTE_DATA_SMOKE").ok().as_deref() != Some("1") {
        panic!(
            "live quote-data smoke is gated; set TV_LIVE_QUOTE_DATA_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbol = env_string("TV_LIVE_QUOTE_DATA_SYMBOL").unwrap_or_else(|| DEFAULT_SYMBOL.into());
    let runs = positive_env("TV_LIVE_QUOTE_DATA_RUNS").unwrap_or(1);
    let expected_phase = env_string("TV_LIVE_QUOTE_DATA_EXPECT_PHASE");
    let allow_unavailable = bool_env("TV_LIVE_QUOTE_DATA_ALLOW_UNAVAILABLE").unwrap_or(true);
    let target_id = env_string("TV_LIVE_QUOTE_DATA_TARGET_ID");

    println!(
        "quote-data live smoke: symbol={} runs={} expected_phase={} allow_unavailable={} target_id={}",
        symbol,
        runs,
        expected_phase.as_deref().unwrap_or("<none>"),
        allow_unavailable,
        if target_id.is_some() {
            "<provided>"
        } else {
            "<auto>"
        },
    );

    let mut success_count = 0usize;
    let mut unavailable_count = 0usize;
    let mut slowest: Option<Duration> = None;
    for run in 1..=runs {
        let started = Instant::now();
        let output = run_quote_data(tv, target_id.as_deref(), &symbol);
        let elapsed = started.elapsed();
        let envelope = parse_output(&symbol, output, elapsed);
        match envelope.get("success").and_then(Value::as_bool) {
            Some(true) => {
                assert_quote_data_success(&symbol, &envelope, elapsed);
                success_count += 1;
                let phase = envelope
                    .pointer("/data/quote_data/market_phase")
                    .and_then(Value::as_str)
                    .or_else(|| {
                        envelope
                            .pointer("/data/quote_data/current_session")
                            .and_then(Value::as_str)
                    });
                if let Some(expected) = expected_phase.as_deref() {
                    print_phase_result(expected, phase);
                }
                println!(
                    "ok run={} result=success source={} phase={} elapsed_ms={}",
                    run,
                    envelope
                        .pointer("/data/source")
                        .and_then(Value::as_str)
                        .unwrap_or("<missing>"),
                    phase.unwrap_or("<missing>"),
                    elapsed.as_millis(),
                );
            }
            Some(false) => {
                assert_quote_data_unavailable(&symbol, &envelope, elapsed, allow_unavailable);
                unavailable_count += 1;
                println!(
                    "ok run={} result=unavailable summary={} elapsed_ms={}",
                    run,
                    summarize_envelope(&envelope),
                    elapsed.as_millis(),
                );
            }
            None => panic!(
                "quote-data live smoke missing success flag: symbol={} elapsed_ms={} summary={}",
                symbol,
                elapsed.as_millis(),
                summarize_envelope(&envelope)
            ),
        }
        if slowest.is_none_or(|previous| elapsed > previous) {
            slowest = Some(elapsed);
        }
    }

    println!(
        "quote-data live smoke passed: symbol={} runs={} successes={} unavailable={} slowest_elapsed_ms={}",
        symbol,
        runs,
        success_count,
        unavailable_count,
        slowest.map(|duration| duration.as_millis()).unwrap_or(0),
    );
}

fn run_quote_data(tv: &str, target_id: Option<&str>, symbol: &str) -> std::process::Output {
    let mut command = Command::new(tv);
    if let Some(target_id) = target_id {
        command.args(["--target-id", target_id]);
    }
    command
        .args(["quote", symbol, "--source", "quote-data"])
        .output()
        .expect("test-built tv binary should execute")
}

fn parse_output(symbol: &str, output: std::process::Output, elapsed: Duration) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed = serde_json::from_str::<Value>(&stdout)
        .or_else(|_| serde_json::from_str::<Value>(&stderr))
        .unwrap_or_else(|_| {
            panic!(
                "quote-data live smoke returned non-JSON output: symbol={} status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                symbol,
                output.status,
                elapsed.as_millis(),
                output.stdout.len(),
                output.stderr.len()
            )
        });

    let success = parsed.get("success").and_then(Value::as_bool);
    if !output.status.success() && success != Some(false) {
        panic!(
            "quote-data live smoke command failed without structured envelope: symbol={} status={} elapsed_ms={} summary={}",
            symbol,
            output.status,
            elapsed.as_millis(),
            summarize_envelope(&parsed)
        );
    }

    parsed
}

fn assert_quote_data_success(symbol: &str, envelope: &Value, elapsed: Duration) {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let quote_data = data.get("quote_data").unwrap_or(&Value::Null);
    if envelope.get("command").and_then(Value::as_str) != Some("quote")
        || data.get("contract_version").and_then(Value::as_str) != Some("quote_data.v1")
        || data.get("source").and_then(Value::as_str) != Some("desktop_quote_data_ws")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_backed_read")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(true)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(true)
        || data.get("requested_symbol").and_then(Value::as_str) != Some(symbol)
        || data
            .get("scanner_extended_hours_included")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .get("chart_main_series_included")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .pointer("/source_availability/available")
            .and_then(Value::as_bool)
            != Some(true)
        || data
            .pointer("/source_availability/status")
            .and_then(Value::as_str)
            != Some("available")
        || data
            .pointer("/source_availability/rtc_observed")
            .and_then(Value::as_bool)
            .is_none()
        || data
            .pointer("/source_availability/price_readback_observed")
            .and_then(Value::as_bool)
            != Some(true)
        || !data
            .pointer("/source_availability/unavailable_reason")
            .is_some_and(Value::is_null)
        || data
            .pointer("/source_availability/timed_out")
            .and_then(Value::as_bool)
            != Some(false)
        || !data
            .pointer("/source_availability/next_action")
            .is_some_and(Value::is_null)
        || data
            .pointer("/source_availability/raw_frame_included")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .pointer("/source_availability/wait_summary/raw_frame_included")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .pointer("/source_availability/wait_summary/qsd_with_rtc_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/matching_symbol_qsd_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/matching_symbol_without_rtc_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/matching_symbol_with_lp_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/matching_symbol_with_regular_close_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/matching_symbol_with_price_readback_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/quote_session_symbol_mappings_seen")
            .and_then(Value::as_u64)
            .is_none()
        || quote_data.get("session_readback").is_none()
        || quote_data
            .pointer("/price_readback/available")
            .and_then(Value::as_bool)
            != Some(true)
        || quote_data
            .pointer("/price_readback/kind")
            .and_then(Value::as_str)
            .is_none()
        || quote_data.pointer("/price_readback/value").is_none()
        || quote_data
            .pointer("/price_readback/source_field")
            .and_then(Value::as_str)
            .is_none()
        || quote_data
            .pointer("/price_readback/session_source")
            .and_then(Value::as_str)
            != Some("tradingview_quote_data_fields")
        || quote_data
            .pointer("/price_readback/session_inferred")
            .and_then(Value::as_bool)
            != Some(false)
        || quote_data
            .pointer("/session_readback/session_source")
            .and_then(Value::as_str)
            != Some("tradingview_quote_data_fields")
        || quote_data
            .pointer("/session_readback/session_inferred")
            .and_then(Value::as_bool)
            != Some(false)
        || quote_data.get("rtc").is_none()
        || data.get("extended_hours").is_some()
    {
        panic!(
            "quote-data live smoke success validation failed: symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
}

fn assert_quote_data_unavailable(
    symbol: &str,
    envelope: &Value,
    elapsed: Duration,
    allow_unavailable: bool,
) {
    let error = envelope.get("error").unwrap_or(&Value::Null);
    let details = error.get("details").unwrap_or(&Value::Null);
    if !allow_unavailable {
        panic!(
            "quote-data live smoke unavailable result was not allowed: symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
    if envelope.get("command").and_then(Value::as_str) != Some("quote")
        || error.get("kind").and_then(Value::as_str) != Some("internal_api_unavailable")
        || details.get("contract_version").and_then(Value::as_str) != Some("quote_data.v1")
        || details.get("source").and_then(Value::as_str) != Some("desktop_quote_data_ws")
        || details.get("source_category").and_then(Value::as_str) != Some("desktop_backed_read")
        || details.get("requires_desktop").and_then(Value::as_bool) != Some(true)
        || details.get("non_mutating").and_then(Value::as_bool) != Some(true)
        || details.get("requested_symbol").and_then(Value::as_str) != Some(symbol)
        || details
            .pointer("/wait_summary/raw_frame_included")
            .and_then(Value::as_bool)
            != Some(false)
        || details
            .pointer("/source_availability/available")
            .and_then(Value::as_bool)
            != Some(false)
        || details
            .pointer("/source_availability/status")
            .and_then(Value::as_str)
            != Some("unavailable")
        || details
            .pointer("/source_availability/rtc_observed")
            .and_then(Value::as_bool)
            != Some(false)
        || details
            .pointer("/source_availability/price_readback_observed")
            .and_then(Value::as_bool)
            != Some(false)
        || details
            .pointer("/source_availability/unavailable_reason")
            .and_then(Value::as_str)
            .is_none()
        || details
            .pointer("/source_availability/timed_out")
            .and_then(Value::as_bool)
            != Some(true)
        || details
            .pointer("/source_availability/next_action")
            .and_then(Value::as_str)
            .is_none()
        || details
            .pointer("/source_availability/raw_frame_included")
            .and_then(Value::as_bool)
            != Some(false)
        || details
            .pointer("/source_availability/wait_summary/raw_frame_included")
            .and_then(Value::as_bool)
            != Some(false)
        || details
            .pointer("/source_availability/wait_summary/qsd_with_rtc_seen")
            .and_then(Value::as_u64)
            .is_none()
        || details
            .pointer("/source_availability/wait_summary/matching_symbol_qsd_seen")
            .and_then(Value::as_u64)
            .is_none()
        || details
            .pointer("/source_availability/wait_summary/matching_symbol_without_rtc_seen")
            .and_then(Value::as_u64)
            .is_none()
        || details
            .pointer("/source_availability/wait_summary/matching_symbol_with_lp_seen")
            .and_then(Value::as_u64)
            .is_none()
        || details
            .pointer("/source_availability/wait_summary/matching_symbol_with_regular_close_seen")
            .and_then(Value::as_u64)
            .is_none()
        || details
            .pointer("/source_availability/wait_summary/matching_symbol_with_price_readback_seen")
            .and_then(Value::as_u64)
            .is_none()
        || details
            .pointer("/source_availability/wait_summary/quote_session_symbol_mappings_seen")
            .and_then(Value::as_u64)
            .is_none()
    {
        panic!(
            "quote-data live smoke unavailable validation failed: symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
}

fn print_phase_result(expected: &str, observed: Option<&str>) {
    match observed {
        Some(observed) if phase_matches_expected(expected, observed) => {
            println!(
                "phase_result=matched expected={} observed={}",
                expected, observed
            );
        }
        Some(observed) => {
            println!(
                "phase_result=observed_different expected={} observed={}",
                expected, observed
            );
        }
        None => {
            println!("phase_result=missing expected={}", expected);
        }
    }
}

fn phase_matches_expected(expected: &str, observed: &str) -> bool {
    normalize_phase(expected) == normalize_phase(observed)
}

fn normalize_phase(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "")
}

fn summarize_envelope(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let error = envelope.get("error").unwrap_or(&Value::Null);
    let details = error.get("details").unwrap_or(&Value::Null);
    format!(
        "success={} command={} kind={} message={} contract={} source={} requested={} observed={} availability={} reason={} next_action={} rtc_present={} market_phase={} current_session={} wait_summary={}",
        bool_field(envelope, "success"),
        string_field(envelope, "command").unwrap_or("<missing>"),
        string_field(error, "kind").unwrap_or("<none>"),
        string_field(error, "message").unwrap_or("<none>"),
        string_field(data, "contract_version")
            .or_else(|| string_field(details, "contract_version"))
            .unwrap_or("<missing>"),
        string_field(data, "source")
            .or_else(|| string_field(details, "source"))
            .unwrap_or("<missing>"),
        string_field(data, "requested_symbol")
            .or_else(|| string_field(details, "requested_symbol"))
            .unwrap_or("<missing>"),
        string_field(data, "observed_symbol")
            .or_else(|| string_field(details, "observed_symbol"))
            .unwrap_or("<missing>"),
        data.pointer("/source_availability/status")
            .or_else(|| details.pointer("/source_availability/status"))
            .and_then(Value::as_str)
            .unwrap_or("<missing>"),
        data.pointer("/source_availability/unavailable_reason")
            .or_else(|| details.pointer("/source_availability/unavailable_reason"))
            .and_then(Value::as_str)
            .unwrap_or("<none>"),
        data.pointer("/source_availability/next_action")
            .or_else(|| details.pointer("/source_availability/next_action"))
            .and_then(Value::as_str)
            .unwrap_or("<none>"),
        data.pointer("/quote_data/rtc").is_some(),
        data.pointer("/quote_data/market_phase")
            .and_then(Value::as_str)
            .unwrap_or("<missing>"),
        data.pointer("/quote_data/current_session")
            .and_then(Value::as_str)
            .unwrap_or("<missing>"),
        details.get("wait_summary").is_some(),
    )
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn positive_env(name: &str) -> Option<usize> {
    env_string(name).map(|value| {
        let parsed = value
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("{name} must be a positive integer"));
        assert!(parsed > 0, "{name} must be positive");
        parsed
    })
}

fn bool_env(name: &str) -> Option<bool> {
    env_string(name).map(|value| match value.as_str() {
        "1" | "true" | "TRUE" | "yes" | "YES" => true,
        "0" | "false" | "FALSE" | "no" | "NO" => false,
        _ => panic!("{name} must be one of 1,0,true,false,yes,no"),
    })
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn bool_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_bool)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<missing>".to_string())
}

#[cfg(test)]
fn quote_data_args(target_id: Option<&str>, symbol: &str) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(target_id) = target_id {
        args.extend(["--target-id".to_string(), target_id.to_string()]);
    }
    args.extend([
        "quote".to_string(),
        symbol.to_string(),
        "--source".to_string(),
        "quote-data".to_string(),
    ]);
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_matching_accepts_extended_hours_aliases() {
        assert!(phase_matches_expected("postmarket", "post-market"));
        assert!(phase_matches_expected("premarket", "pre_market"));
        assert!(!phase_matches_expected("postmarket", "regular"));
    }

    #[test]
    fn quote_data_args_include_optional_target_before_command() {
        let without_target = quote_data_args(None, "NASDAQ:RKLB");
        assert_eq!(
            without_target,
            ["quote", "NASDAQ:RKLB", "--source", "quote-data"]
        );

        let with_target = quote_data_args(Some("target-1"), "NASDAQ:RKLB");
        assert_eq!(
            with_target,
            [
                "--target-id",
                "target-1",
                "quote",
                "NASDAQ:RKLB",
                "--source",
                "quote-data"
            ]
        );
    }
}
