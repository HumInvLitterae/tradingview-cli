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
