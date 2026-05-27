mod support;

use predicates::prelude::*;
use serde_json::json;

use support::{stderr_json, tv};

#[test]
fn quote_help_explains_symbol_and_target_selection() {
    tv().args(["quote", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[SYMBOL]"))
        .stdout(predicate::str::contains("current chart target"))
        .stdout(predicate::str::contains("--source"))
        .stdout(predicate::str::contains("scanner"))
        .stdout(predicate::str::contains("chart"))
        .stdout(predicate::str::contains("quote-data"))
        .stdout(predicate::str::contains("auto"))
        .stdout(predicate::str::contains("realtime guarantee"))
        .stdout(predicate::str::contains("time"))
        .stdout(predicate::str::contains("update_mode"))
        .stdout(predicate::str::contains("delay_seconds"))
        .stdout(predicate::str::contains("extended_hours"))
        .stdout(predicate::str::contains("session_boundary"))
        .stdout(predicate::str::contains("regular quote-data `lp`"))
        .stdout(predicate::str::contains("auto does not use quote-data"))
        .stdout(predicate::str::contains("Get a real-time price quote").not())
        .stdout(predicate::str::contains("--target-id"));
}

#[test]
fn quotes_help_explains_batch_symbol_reads() {
    tv().args(["quotes", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[SYMBOLS]"))
        .stdout(predicate::str::contains("scanner-backed"))
        .stdout(predicate::str::contains("realtime guarantee"))
        .stdout(predicate::str::contains("time"))
        .stdout(predicate::str::contains("update_mode"))
        .stdout(predicate::str::contains("delay_seconds"))
        .stdout(predicate::str::contains("data.items"))
        .stdout(predicate::str::contains("real-time").not());
}

#[test]
fn quotes_rejects_invalid_inputs_before_connecting() {
    let no_symbols = tv()
        .env("TV_CDP_PORT", "9")
        .arg("quotes")
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&no_symbols);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quotes");
    assert_eq!(value["error"]["kind"], "validation");

    let blank_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quotes", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&blank_symbol);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quotes");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn fundamentals_help_explains_desktop_free_read() {
    tv().args(["fundamentals", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SYMBOL>"))
        .stdout(predicate::str::contains("--group"))
        .stdout(predicate::str::contains("--field"))
        .stdout(predicate::str::contains("Desktop"))
        .stdout(predicate::str::contains("earnings"));
}

#[test]
fn fundamentals_rejects_invalid_inputs_before_connecting() {
    let blank_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["fundamentals", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&blank_symbol);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "fundamentals");
    assert_eq!(value["error"]["kind"], "validation");

    let unknown_field = tv()
        .env("TV_CDP_PORT", "9")
        .args(["fundamentals", "NYSE:IONQ", "--field", "banana"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&unknown_field);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "fundamentals");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported_fields"]
            .as_array()
            .unwrap()
            .contains(&json!("earnings_release_next_date"))
    );
    assert!(
        value["error"]["details"]["supported_fields"]
            .as_array()
            .unwrap()
            .contains(&json!("dividend_amount_recent"))
    );

    let unknown_group = tv()
        .env("TV_CDP_PORT", "9")
        .args(["fundamentals", "NYSE:IONQ", "--group", "banana"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&unknown_group);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "fundamentals");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported_groups"]
            .as_array()
            .unwrap()
            .contains(&json!("earnings"))
    );
}

#[test]
fn snapshot_help_explains_desktop_free_evidence_packet() {
    tv().args(["snapshot", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SYMBOL>"))
        .stdout(predicate::str::contains("--group"))
        .stdout(predicate::str::contains("--field"))
        .stdout(predicate::str::contains("Desktop-free"))
        .stdout(predicate::str::contains("quote"))
        .stdout(predicate::str::contains("fundamentals"))
        .stdout(predicate::str::contains("Follow-up hints"))
        .stdout(predicate::str::contains("not auto-run"))
        .stdout(predicate::str::contains("observe chart"));
}

#[test]
fn snapshot_rejects_invalid_inputs_before_connecting() {
    let blank_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["snapshot", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&blank_symbol);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "snapshot");
    assert_eq!(value["error"]["kind"], "validation");

    let unknown_field = tv()
        .env("TV_CDP_PORT", "9")
        .args(["snapshot", "NYSE:IONQ", "--field", "banana"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&unknown_field);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "snapshot");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported_fields"]
            .as_array()
            .unwrap()
            .contains(&json!("price_earnings_ttm"))
    );

    let unknown_group = tv()
        .env("TV_CDP_PORT", "9")
        .args(["snapshot", "NYSE:IONQ", "--group", "banana"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&unknown_group);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "snapshot");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported_groups"]
            .as_array()
            .unwrap()
            .contains(&json!("earnings"))
    );
}

#[test]
fn compare_help_explains_desktop_free_comparison() {
    tv().args(["compare", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SYMBOLS"))
        .stdout(predicate::str::contains("Desktop-free"))
        .stdout(predicate::str::contains("scanner quote"))
        .stdout(predicate::str::contains("fundamentals"))
        .stdout(predicate::str::contains("Follow-up hints"))
        .stdout(predicate::str::contains("not auto-run"))
        .stdout(predicate::str::contains("snapshot"))
        .stdout(predicate::str::contains("observe chart"));
}

#[test]
fn compare_rejects_invalid_inputs_before_connecting() {
    let no_symbols = tv()
        .env("TV_CDP_PORT", "9")
        .arg("compare")
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&no_symbols);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "compare");
    assert_eq!(value["error"]["kind"], "validation");

    let one_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["compare", "AAPL"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&one_symbol);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "compare");
    assert_eq!(value["error"]["kind"], "validation");

    let blank_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["compare", "AAPL", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&blank_symbol);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "compare");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn watch_compare_help_explains_bounded_scanner_jsonl() {
    tv().args(["watch", "compare", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SYMBOLS"))
        .stdout(predicate::str::contains("scanner"))
        .stdout(predicate::str::contains("JSON"))
        .stdout(predicate::str::contains("readiness"))
        .stdout(predicate::str::contains("heartbeat"))
        .stdout(predicate::str::contains("summary"))
        .stdout(predicate::str::contains("--interval"))
        .stdout(predicate::str::contains("--duration-ms"))
        .stdout(predicate::str::contains("--max-events"))
        .stdout(predicate::str::contains("--heartbeat-ms"))
        .stdout(predicate::str::contains("buy/sell").not());
}

#[test]
fn watch_compare_rejects_invalid_inputs_before_connecting() {
    let one_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["watch", "compare", "AAPL"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&one_symbol);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "watch");
    assert_eq!(value["error"]["kind"], "validation");

    let blank_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["watch", "compare", "AAPL", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&blank_symbol);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "watch");
    assert_eq!(value["error"]["kind"], "validation");

    let too_many_symbols: Vec<String> = std::iter::once("watch".to_string())
        .chain(std::iter::once("compare".to_string()))
        .chain((0..26).map(|idx| format!("NASDAQ:T{idx}")))
        .collect();
    let too_many = tv()
        .env("TV_CDP_PORT", "9")
        .args(too_many_symbols)
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&too_many);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "watch");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["maximum"], 25);
}

#[test]
fn watch_compare_rejects_invalid_controls_before_connecting() {
    for args in [
        vec!["watch", "compare", "AAPL", "MSFT", "--interval", "999"],
        vec!["watch", "compare", "AAPL", "MSFT", "--duration-ms", "0"],
        vec![
            "watch",
            "compare",
            "AAPL",
            "MSFT",
            "--duration-ms",
            "300001",
        ],
        vec!["watch", "compare", "AAPL", "MSFT", "--max-events", "0"],
        vec!["watch", "compare", "AAPL", "MSFT", "--heartbeat-ms", "999"],
    ] {
        let assert = tv()
            .env("TV_CDP_PORT", "9")
            .args(args)
            .assert()
            .failure()
            .code(1);
        let value = stderr_json(&assert);
        assert_eq!(value["success"], false);
        assert_eq!(value["command"], "watch");
        assert_eq!(value["error"]["kind"], "validation");
    }
}

#[test]
fn scanner_help_lists_hotlist_subcommand() {
    tv().args(["scanner", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hotlist"))
        .stdout(predicate::str::contains("metainfo"))
        .stdout(predicate::str::contains("scan"));

    tv().args(["scanner", "hotlist", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SLUG>"))
        .stdout(predicate::str::contains("--limit"));

    tv().args(["scanner", "metainfo", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--market"))
        .stdout(predicate::str::contains("--field"));

    tv().args(["scanner", "scan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--exchange"))
        .stdout(predicate::str::contains("--sector"))
        .stdout(predicate::str::contains("--industry"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--subtype"))
        .stdout(predicate::str::contains("--columns"))
        .stdout(predicate::str::contains("--sort"))
        .stdout(predicate::str::contains("--min-price"))
        .stdout(predicate::str::contains("--min-change"))
        .stdout(predicate::str::contains("--max-change"))
        .stdout(predicate::str::contains("--min-relative-volume"))
        .stdout(predicate::str::contains("--max-pe"))
        .stdout(predicate::str::contains("--min-average-volume"))
        .stdout(predicate::str::contains("--min-performance-week"))
        .stdout(predicate::str::contains("--max-performance-week"))
        .stdout(predicate::str::contains("--min-performance-month"))
        .stdout(predicate::str::contains("--max-performance-month"))
        .stdout(predicate::str::contains("--min-performance-quarter"))
        .stdout(predicate::str::contains("--max-performance-quarter"))
        .stdout(predicate::str::contains("--min-rsi"))
        .stdout(predicate::str::contains("--max-rsi"))
        .stdout(predicate::str::contains("--min-recommendation"))
        .stdout(predicate::str::contains("--max-recommendation"));
}

#[test]
fn scanner_hotlist_rejects_unknown_slug_before_network() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "hotlist", "unknown_slug"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "scanner");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn scanner_hotlist_rejects_zero_limit_before_network() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "hotlist", "volume_gainers", "--limit", "0"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "scanner");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn scanner_scan_rejects_invalid_inputs_before_network() {
    let invalid_market = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--market", "global"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&invalid_market);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "scanner");
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_column = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--columns", "name,unknown"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&invalid_column);
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported_fields"]
            .as_array()
            .unwrap()
            .contains(&json!("earnings_release_next_trading_date_fq"))
    );
    assert!(
        value["error"]["details"]["supported_fields"]
            .as_array()
            .unwrap()
            .contains(&json!("dividend_amount_recent"))
    );

    let invalid_limit = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--limit", "0"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&invalid_limit);
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_sector = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--sector", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&invalid_sector);
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_rsi = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--max-rsi", "101"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&invalid_rsi);
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_recommendation = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--min-recommendation", "-1.1"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&invalid_recommendation);
    assert_eq!(value["error"]["kind"], "validation");

    let signed_change_reaches_scanner_validation = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--max-change", "-5", "--limit", "0"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&signed_change_reaches_scanner_validation);
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("--limit")
    );
}

#[test]
fn info_help_explains_current_chart_and_symbol_modes() {
    tv().args(["info", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[SYMBOL]"))
        .stdout(predicate::str::contains("Without SYMBOL"))
        .stdout(predicate::str::contains("With SYMBOL"))
        .stdout(predicate::str::contains(
            "without connecting to TradingView Desktop",
        ))
        .stdout(predicate::str::contains("tv quote <SYMBOL>"));
}

#[test]
fn search_requires_query() {
    let assert = tv().arg("search").assert().failure().code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "search");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn quote_rejects_empty_symbol_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quote", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quote");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn quote_scanner_source_requires_symbol_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quote", "--source", "scanner"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quote");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn quote_data_source_requires_symbol_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quote", "--source", "quote-data"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quote");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn quote_chart_source_attempts_connection_when_cdp_is_unavailable() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quote", "AAPL", "--source", "chart"])
        .assert()
        .failure()
        .code(2);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quote");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn quote_data_source_attempts_connection_when_cdp_is_unavailable() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quote", "AAPL", "--source", "quote-data"])
        .assert()
        .failure()
        .code(2);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quote");
    assert_eq!(value["error"]["kind"], "connection");
}
