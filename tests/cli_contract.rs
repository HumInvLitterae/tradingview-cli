use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::fs;

fn tv() -> Command {
    Command::cargo_bin("tv").expect("tv binary should build")
}

#[test]
fn help_lists_v1_commands() {
    tv().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("launch"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("scanner"))
        .stdout(predicate::str::contains("values"))
        .stdout(predicate::str::contains("discover"))
        .stdout(predicate::str::contains("ui-state"))
        .stdout(predicate::str::contains("watchlist"))
        .stdout(predicate::str::contains("alert"))
        .stdout(predicate::str::contains("indicator"))
        .stdout(predicate::str::contains("draw"))
        .stdout(predicate::str::contains("pine"))
        .stdout(predicate::str::contains("tab"))
        .stdout(predicate::str::contains("replay"))
        .stdout(predicate::str::contains("stream"))
        .stdout(predicate::str::contains("ui"))
        .stdout(predicate::str::contains("data"))
        .stdout(predicate::str::contains("pane"))
        .stdout(predicate::str::contains("layout"))
        .stdout(predicate::str::contains("range"))
        .stdout(predicate::str::contains("type"))
        .stdout(predicate::str::contains("scroll"))
        .stdout(predicate::str::contains("screenshot"));
}

#[test]
fn scanner_help_lists_hotlist_subcommand() {
    tv().args(["scanner", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hotlist"));

    tv().args(["scanner", "hotlist", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SLUG>"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn screener_help_lists_read_subcommands() {
    tv().args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("screener"));

    tv().args(["screener", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("open"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("screens"))
        .stdout(predicate::str::contains("filters"))
        .stdout(predicate::str::contains("columns"))
        .stdout(predicate::str::contains("close"));

    tv().args(["screener", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--limit"));

    tv().args(["screener", "screens", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("active"));

    tv().args(["screener", "filters", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));

    tv().args(["screener", "columns", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

#[test]
fn screener_get_rejects_zero_limit_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "get", "--limit", "0"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn scanner_hotlist_rejects_unknown_slug_before_network() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "hotlist", "unknown_slug"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
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
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "scanner");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn launch_help_lists_safety_options() {
    tv().args(["launch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--port"))
        .stdout(predicate::str::contains("--path"))
        .stdout(predicate::str::contains("--kill-existing"));
}

#[test]
fn launch_rejects_missing_explicit_path_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["launch", "--path", "target/does-not-exist-tradingview"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "launch");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn launch_rejects_port_zero_before_connecting() {
    let assert = tv()
        .args(["launch", "--port", "0"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "launch");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn unknown_command_exits_with_usage_error() {
    let assert = tv().arg("unknown-command").assert().failure().code(1);
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
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("add"));
    tv().args(["pane", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("layout"))
        .stdout(predicate::str::contains("focus"))
        .stdout(predicate::str::contains("symbol"));
}

#[test]
fn layout_help_lists_subcommands() {
    tv().args(["layout", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("switch"));
}

#[test]
fn alert_help_lists_read_subcommands() {
    tv().args(["alert", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"));

    tv().args(["alert", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--price"))
        .stdout(predicate::str::contains("--condition"))
        .stdout(predicate::str::contains("--message"));

    tv().args(["alert", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--id"))
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn indicator_help_lists_lifecycle_subcommands() {
    tv().args(["indicator", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("toggle"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("get"));

    tv().args(["indicator", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--inputs"));

    tv().args(["indicator", "set", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--inputs"));

    tv().args(["indicator", "toggle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--visible"))
        .stdout(predicate::str::contains("--hidden"));
}

#[test]
fn draw_help_lists_lifecycle_subcommands() {
    tv().args(["draw", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("shape"))
        .stdout(predicate::str::contains("position"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("clear"));

    tv().args(["draw", "shape", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--price"))
        .stdout(predicate::str::contains("--time"))
        .stdout(predicate::str::contains("--overrides"));

    tv().args(["draw", "position", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--entry-price"))
        .stdout(predicate::str::contains("--stop-loss"))
        .stdout(predicate::str::contains("--take-profit"))
        .stdout(predicate::str::contains("--entry-time"))
        .stdout(predicate::str::contains("--account-size"))
        .stdout(predicate::str::contains("--risk"))
        .stdout(predicate::str::contains("--lot-size"));

    tv().args(["draw", "clear", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn pine_help_lists_current_subcommands() {
    tv().args(["pine", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("compile"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("open"))
        .stdout(predicate::str::contains("save"))
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("errors"))
        .stdout(predicate::str::contains("console"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("raw-compile"));

    tv().args(["pine", "set", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--file"));

    tv().args(["pine", "analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--file"));

    tv().args(["pine", "check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--file"));

    tv().args(["pine", "save", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name").not());
}

#[test]
fn ui_help_lists_old_automation_subcommands() {
    tv().args(["ui", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("click"))
        .stdout(predicate::str::contains("keyboard"))
        .stdout(predicate::str::contains("hover"))
        .stdout(predicate::str::contains("scroll"))
        .stdout(predicate::str::contains("find"))
        .stdout(predicate::str::contains("eval"))
        .stdout(predicate::str::contains("type"))
        .stdout(predicate::str::contains("panel"))
        .stdout(predicate::str::contains("fullscreen"))
        .stdout(predicate::str::contains("mouse"));
}

#[test]
fn tab_help_lists_lifecycle_subcommands() {
    tv().args(["tab", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("switch"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("close"));

    tv().args(["tab", "new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--from"));
}

#[test]
fn replay_help_lists_basic_lifecycle_subcommands() {
    tv().args(["replay", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("step"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("autoplay"))
        .stdout(predicate::str::contains("trade"));

    tv().args(["replay", "autoplay", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--speed"));

    tv().args(["replay", "trade", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<ACTION>"));
}

#[test]
fn stream_help_lists_read_subcommands() {
    tv().args(["stream", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("quote"))
        .stdout(predicate::str::contains("bars"))
        .stdout(predicate::str::contains("values"))
        .stdout(predicate::str::contains("lines"))
        .stdout(predicate::str::contains("labels"))
        .stdout(predicate::str::contains("tables"))
        .stdout(predicate::str::contains("all"));

    tv().args(["stream", "lines", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--interval"));
}

#[test]
fn data_help_lists_advanced_read_subcommands() {
    tv().args(["data", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("indicator"))
        .stdout(predicate::str::contains("depth"))
        .stdout(predicate::str::contains("strategy"))
        .stdout(predicate::str::contains("trades"))
        .stdout(predicate::str::contains("equity"))
        .stdout(predicate::str::contains("lines"))
        .stdout(predicate::str::contains("labels"))
        .stdout(predicate::str::contains("tables"))
        .stdout(predicate::str::contains("boxes"))
        .stdout(predicate::str::contains("shapes"));

    tv().args(["data", "labels", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--verbose"));

    tv().args(["data", "shapes", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn data_indicator_requires_entity_id() {
    let assert = tv().args(["data", "indicator"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn data_shapes_rejects_zero_count_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["data", "shapes", "--count", "0"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "data");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn watchlist_add_requires_symbol() {
    let assert = tv().args(["watchlist", "add"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn watchlist_remove_requires_symbol() {
    let assert = tv()
        .args(["watchlist", "remove"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pine_set_requires_source_before_connecting() {
    let assert = tv()
        .args(["pine", "set"])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pine_set_reports_missing_file_before_connecting() {
    let assert = tv()
        .args(["pine", "set", "--file", "target/does-not-exist.pine"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pine_analyze_requires_source_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "analyze"])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pine_check_requires_source_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "check"])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pine_new_rejects_unknown_type_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "new", "study"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pine_open_requires_name_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "open"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pine_new_attempts_connection_when_cdp_is_unavailable() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "new", "indicator"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn pine_open_attempts_connection_when_cdp_is_unavailable() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "open", "My", "Script"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn pine_analyze_runs_without_cdp_connection() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "analyze"])
        .write_stdin("//@version=6\nindicator(\"X\")\na = array.from(1)\nx = array.get(a, 2)")
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["success"], true);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["data"]["input_source"], "stdin");
    assert_eq!(value["data"]["issue_count"], 1);
}

#[test]
fn pine_set_with_stdin_attempts_connection_when_cdp_is_unavailable() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "set"])
        .write_stdin("//@version=6\nindicator(\"X\")")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn pine_set_with_file_attempts_connection_when_cdp_is_unavailable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("source.pine");
    fs::write(&path, "//@version=6\nindicator(\"X\")").unwrap();

    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "set", "--file", path.to_str().unwrap()])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn stream_rejects_too_small_interval_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["stream", "quote", "--interval", "99"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "stream");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["minimum_interval_ms"], 100);
}

#[test]
fn stream_attempts_connection_when_cdp_is_unavailable() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["stream", "quote", "--interval", "100"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(stdout, "");
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "stream");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn pine_compile_attempts_connection_when_cdp_is_unavailable() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "compile"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn read_utilities_attempt_connection_when_cdp_is_unavailable() {
    for args in [
        vec!["info"],
        vec!["values"],
        vec!["discover"],
        vec!["ui-state"],
        vec!["watchlist", "get"],
        vec!["watchlist", "add", "NASDAQ:AAPL"],
        vec!["watchlist", "remove", "NASDAQ:AAPL"],
        vec!["alert", "list"],
        vec![
            "alert",
            "create",
            "--price",
            "100",
            "--condition",
            "crossing",
        ],
        vec!["alert", "delete", "--id", "4546454367"],
        vec!["alert", "delete", "--all", "--dry-run"],
        vec!["indicator", "add", "Volume"],
        vec!["indicator", "remove", "study-id"],
        vec!["indicator", "toggle", "study-id", "--hidden"],
        vec![
            "indicator",
            "set",
            "study-id",
            "--inputs",
            r#"{"length":20}"#,
        ],
        vec!["indicator", "get", "study-id"],
        vec!["draw", "shape", "--price", "100", "--time", "1700000000"],
        vec![
            "draw",
            "position",
            "long",
            "--entry-price",
            "100",
            "--stop-loss",
            "90",
            "--take-profit",
            "120",
        ],
        vec!["draw", "list"],
        vec!["draw", "get", "shape-id"],
        vec!["draw", "remove", "shape-id"],
        vec!["draw", "clear", "--dry-run"],
        vec!["draw", "clear"],
        vec!["pine", "get"],
        vec!["pine", "compile"],
        vec!["pine", "raw-compile"],
        vec!["pine", "save"],
        vec!["pine", "errors"],
        vec!["pine", "console"],
        vec!["pine", "list"],
        vec!["tab", "list"],
        vec!["tab", "switch", "0"],
        vec!["tab", "new"],
        vec!["tab", "close", "0"],
        vec!["replay", "start"],
        vec!["replay", "step"],
        vec!["replay", "stop"],
        vec!["replay", "status"],
        vec!["replay", "autoplay", "--speed", "1000"],
        vec!["replay", "trade", "close"],
        vec!["pane", "list"],
        vec!["pane", "layout", "s"],
        vec!["pane", "focus", "0"],
        vec!["pane", "symbol", "0", "NASDAQ:AAPL"],
        vec!["layout", "list"],
        vec!["layout", "switch", "Swing", "Layout", "--dry-run"],
        vec!["ui", "find", "Chart"],
        vec!["ui", "click", "--value", "Chart"],
        vec!["ui", "keyboard", "Escape"],
        vec!["ui", "hover", "--value", "Chart"],
        vec!["ui", "scroll", "down"],
        vec!["ui", "type", "hello"],
        vec!["ui", "panel", "watchlist", "open"],
        vec!["ui", "fullscreen"],
        vec!["ui", "mouse", "1", "2"],
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
fn ui_eval_is_disabled_before_connecting_without_env_gate() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["ui", "eval", "1+1"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "ui");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("TV_ALLOW_UNSAFE_UI_EVAL=1")
    );
}

#[test]
fn ui_eval_attempts_connection_when_env_gate_is_enabled() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .env("TV_ALLOW_UNSAFE_UI_EVAL", "1")
        .args(["ui", "eval", "1+1"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "ui");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn draw_get_and_remove_require_entity_id() {
    for args in [vec!["draw", "get"], vec!["draw", "remove"]] {
        let assert = tv().args(args).assert().failure().code(1);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        let value: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["command"], "tv");
        assert_eq!(value["error"]["kind"], "validation");
    }
}

#[test]
fn tab_switch_requires_index() {
    let assert = tv().args(["tab", "switch"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn tab_close_requires_index() {
    let assert = tv().args(["tab", "close"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn screenshot_requires_output_before_connecting() {
    let assert = tv()
        .args(["screenshot", "--region", "full"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn replay_trade_rejects_invalid_action_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["replay", "trade", "hold"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "replay");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn replay_autoplay_rejects_invalid_speed_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["replay", "autoplay", "--speed", "500"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "replay");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn replay_start_rejects_invalid_date_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["replay", "start", "--date", "2026-02-31"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "replay");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn draw_shape_rejects_invalid_inputs_before_connecting() {
    for args in [
        vec!["draw", "shape", "--price", "NaN", "--time", "1700000000"],
        vec!["draw", "shape", "--price", "100", "--time", "NaN"],
        vec![
            "draw",
            "shape",
            "--price",
            "100",
            "--time",
            "1700000000",
            "--price2",
            "101",
        ],
        vec![
            "draw",
            "shape",
            "--price",
            "100",
            "--time",
            "1700000000",
            "--overrides",
            "[]",
        ],
        vec![
            "draw",
            "shape",
            "--price",
            "100",
            "--time",
            "1700000000",
            "--overrides",
            "{",
        ],
    ] {
        let assert = tv()
            .env("TV_CDP_PORT", "9")
            .args(args)
            .assert()
            .failure()
            .code(1);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        let value: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["error"]["kind"], "validation");
    }
}

#[test]
fn draw_position_rejects_invalid_inputs_before_connecting() {
    for args in [
        vec![
            "draw",
            "position",
            "up",
            "--entry-price",
            "100",
            "--stop-loss",
            "90",
            "--take-profit",
            "120",
        ],
        vec![
            "draw",
            "position",
            "long",
            "--entry-price",
            "100",
            "--stop-loss",
            "100",
            "--take-profit",
            "120",
        ],
        vec![
            "draw",
            "position",
            "long",
            "--entry-price",
            "100",
            "--stop-loss",
            "90",
            "--take-profit",
            "99",
        ],
        vec![
            "draw",
            "position",
            "short",
            "--entry-price",
            "100",
            "--stop-loss",
            "99",
            "--take-profit",
            "80",
        ],
        vec![
            "draw",
            "position",
            "short",
            "--entry-price",
            "100",
            "--stop-loss",
            "110",
            "--take-profit",
            "100",
        ],
        vec![
            "draw",
            "position",
            "long",
            "--entry-price",
            "NaN",
            "--stop-loss",
            "90",
            "--take-profit",
            "120",
        ],
        vec![
            "draw",
            "position",
            "long",
            "--entry-price",
            "100",
            "--stop-loss",
            "90",
            "--take-profit",
            "120",
            "--risk",
            "0",
        ],
    ] {
        let assert = tv()
            .env("TV_CDP_PORT", "9")
            .args(args)
            .assert()
            .failure()
            .code(1);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        let value: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["command"], "draw");
        assert_eq!(value["error"]["kind"], "validation");
    }
}

#[test]
fn indicator_add_requires_name() {
    let assert = tv().args(["indicator", "add"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "indicator");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn indicator_set_rejects_invalid_inputs_before_connecting() {
    for args in [
        vec!["indicator", "set", "study-id", "--inputs", "[]"],
        vec!["indicator", "set", "study-id", "--inputs", "{}"],
        vec!["indicator", "set", "study-id", "--inputs", "{"],
        vec!["indicator", "add", "Volume", "--inputs", "[]"],
    ] {
        let assert = tv()
            .env("TV_CDP_PORT", "9")
            .args(args)
            .assert()
            .failure()
            .code(1);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        let value: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["command"], "indicator");
        assert_eq!(value["error"]["kind"], "validation");
    }
}

#[test]
fn indicator_toggle_rejects_conflicting_visibility_flags_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["indicator", "toggle", "study-id", "--visible", "--hidden"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "indicator");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn alert_delete_requires_id() {
    let assert = tv().args(["alert", "delete"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "alert");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn alert_delete_rejects_conflicting_targets_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["alert", "delete", "--id", "1", "--all"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "alert");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn pane_layout_rejects_unknown_layout_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pane", "layout", "banana"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "pane");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported"]
            .as_array()
            .expect("supported pane layouts should be listed")
            .contains(&json!({"layout": "4", "layout_name": "2x2 grid"}))
    );
}

#[test]
fn alert_create_requires_price() {
    let assert = tv().args(["alert", "create"]).assert().failure().code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn alert_create_rejects_invalid_condition_before_connecting() {
    let assert = tv()
        .args(["alert", "create", "--price", "100", "--condition", "above"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "alert");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported"]
            .as_array()
            .expect("supported alert conditions should be listed")
            .contains(&Value::String("crossing".to_string()))
    );
}

#[test]
fn alert_create_rejects_non_finite_price_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["alert", "create", "--price", "NaN"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "alert");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn data_read_commands_attempt_connection_when_cdp_is_unavailable() {
    for args in [
        vec!["data", "indicator", "study-id"],
        vec!["data", "depth"],
        vec!["data", "strategy"],
        vec!["data", "trades", "--max", "5"],
        vec!["data", "equity"],
        vec!["data", "lines", "--filter", "RS", "--verbose"],
        vec!["data", "labels", "--max", "5"],
        vec!["data", "tables"],
        vec!["data", "boxes", "--verbose"],
        vec!["data", "shapes", "--count", "5", "--verbose"],
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
        assert_eq!(value["command"], "data");
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
fn screenshot_chart_region_attempts_connection() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screenshot",
            "--region",
            "chart",
            "--output",
            "target/test.png",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "screenshot");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn screenshot_rejects_unsupported_region() {
    let assert = tv()
        .args([
            "screenshot",
            "--region",
            "strategy_tester",
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

#[test]
fn type_attempts_connection_when_cdp_is_unavailable() {
    for args in [vec!["type"], vec!["type", "Line"], vec!["type", "1"]] {
        let assert = tv()
            .env("TV_CDP_PORT", "9")
            .args(args)
            .assert()
            .failure()
            .code(2);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        let value: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["command"], "type");
        assert_eq!(value["error"]["kind"], "connection");
    }
}

#[test]
fn type_rejects_unknown_chart_type_before_connecting() {
    let assert = tv()
        .args(["type", "not-a-chart-type"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "type");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported"]
            .as_array()
            .expect("supported chart types should be listed")
            .contains(&Value::String("Candles".to_string()))
    );
}
