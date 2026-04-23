use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tv() -> Command {
    Command::cargo_bin("tv").expect("tv binary should build")
}

#[test]
fn help_lists_v1_commands() {
    tv().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("values"))
        .stdout(predicate::str::contains("watchlist"))
        .stdout(predicate::str::contains("pane"))
        .stdout(predicate::str::contains("range"))
        .stdout(predicate::str::contains("scroll"))
        .stdout(predicate::str::contains("screenshot"));
}

#[test]
fn unknown_command_exits_with_usage_error() {
    let assert = tv().arg("pine").assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn ohlcv_without_summary_attempts_connection() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .arg("ohlcv")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "ohlcv");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn ohlcv_accepts_count_argument() {
    tv().args(["ohlcv", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--count"));
}

#[test]
fn search_requires_query() {
    let assert = tv().arg("search").assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "search");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn watchlist_and_pane_help_list_read_subcommands() {
    tv().args(["watchlist", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"));
    tv().args(["pane", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

#[test]
fn read_utilities_attempt_connection_when_cdp_is_unavailable() {
    for args in [
        vec!["info"],
        vec!["values"],
        vec!["watchlist", "get"],
        vec!["pane", "list"],
    ] {
        let assert = tv()
            .env("TV_CDP_PORT", "9")
            .args(args)
            .assert()
            .failure()
            .code(2);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        let value: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["error"]["kind"], "connection");
    }
}

#[test]
fn connection_failure_uses_structured_json_and_exit_code_2() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .arg("status")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "status");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn screenshot_rejects_chart_region_for_v1() {
    let assert = tv()
        .args([
            "screenshot",
            "--region",
            "chart",
            "--output",
            "target/test.png",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "screenshot");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn range_requires_from_and_to_together() {
    let assert = tv()
        .args(["range", "--from", "1"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "range");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn symbol_and_timeframe_allow_read_mode() {
    tv().env("TV_CDP_PORT", "9")
        .arg("symbol")
        .assert()
        .failure()
        .code(2);
    tv().env("TV_CDP_PORT", "9")
        .arg("timeframe")
        .assert()
        .failure()
        .code(2);
}
