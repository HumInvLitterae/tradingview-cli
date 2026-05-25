mod support;

use predicates::prelude::*;

use support::{stderr_json, tv};

#[test]
fn bars_help_explains_stable_desktop_free_boundary() {
    tv().args(["bars", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SYMBOL>"))
        .stdout(predicate::str::contains("--timeframe"))
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("--from"))
        .stdout(predicate::str::contains("--to"))
        .stdout(predicate::str::contains("date-range"))
        .stdout(predicate::str::contains("1W"))
        .stdout(predicate::str::contains("1M"))
        .stdout(predicate::str::contains("range_alignment"))
        .stdout(predicate::str::contains("safety cap"))
        .stdout(predicate::str::contains("historical OHLCV bars"))
        .stdout(predicate::str::contains("bars.v1"))
        .stdout(predicate::str::contains("tradingview_bars_ws"))
        .stdout(predicate::str::contains("TV_EXPERIMENTAL_BARS").not())
        .stdout(predicate::str::contains("lab-gated").not())
        .stdout(predicate::str::contains("Desktop"))
        .stdout(predicate::str::contains("tv ohlcv"));
}

#[test]
fn bars_env_gate_is_not_required_before_validation() {
    let assert = tv()
        .env_remove("TV_EXPERIMENTAL_BARS")
        .env("TV_CDP_PORT", "9")
        .args(["bars", "AAPL", "--count", "5"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(
        value["error"]["details"]["expected_format"],
        "EXCHANGE:SYMBOL"
    );
}

#[test]
fn bars_rejects_invalid_inputs_before_network() {
    let bare_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["bars", "AAPL", "--count", "5"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&bare_symbol);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(
        value["error"]["details"]["expected_format"],
        "EXCHANGE:SYMBOL"
    );

    let zero_count = tv()
        .env("TV_CDP_PORT", "9")
        .args(["bars", "NASDAQ:AAPL", "--count", "0"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&zero_count);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["maximum"], 500);

    let too_many = tv()
        .env("TV_CDP_PORT", "9")
        .args(["bars", "NASDAQ:AAPL", "--count", "501"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&too_many);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["maximum"], 500);
}

#[test]
fn bars_rejects_invalid_date_range_inputs_before_network() {
    let missing_to = tv()
        .env("TV_CDP_PORT", "9")
        .args(["bars", "NASDAQ:AAPL", "--from", "2020-01-01"])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&missing_to);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["from_provided"], true);
    assert_eq!(value["error"]["details"]["to_provided"], false);

    let invalid_date = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "bars",
            "NASDAQ:AAPL",
            "--from",
            "2023-02-29",
            "--to",
            "2023-03-01",
        ])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&invalid_date);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["expected_format"], "YYYY-MM-DD");

    let reversed = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "bars",
            "NASDAQ:AAPL",
            "--from",
            "2020-03-31",
            "--to",
            "2020-01-01",
        ])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&reversed);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["from"], "2020-03-31");
    assert_eq!(value["error"]["details"]["to"], "2020-01-01");

    let intraday = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "bars",
            "NASDAQ:AAPL",
            "--timeframe",
            "1",
            "--from",
            "2020-01-01",
            "--to",
            "2020-03-31",
        ])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&intraday);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["requested_timeframe"], "1");
    assert_eq!(
        value["error"]["details"]["supported_timeframes"],
        serde_json::json!(["1D", "1W", "1M"])
    );
}
