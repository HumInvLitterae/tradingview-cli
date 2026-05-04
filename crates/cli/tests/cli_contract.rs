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
        .stdout(predicate::str::contains("readiness"))
        .stdout(predicate::str::contains("launch"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fundamentals"))
        .stdout(predicate::str::contains("scanner"))
        .stdout(predicate::str::contains("values"))
        .stdout(predicate::str::contains("discover"))
        .stdout(predicate::str::contains("quotes"))
        .stdout(predicate::str::contains("bars"))
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
        .stdout(predicate::str::contains("screenshot"))
        .stdout(predicate::str::contains("--target-id"));
}

#[test]
fn readiness_help_explains_desktop_backed_non_mutating_read() {
    tv().args(["readiness", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Desktop"))
        .stdout(predicate::str::contains("chart API"))
        .stdout(predicate::str::contains("non-mutating"))
        .stdout(predicate::str::contains("--target-id"));
}

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
        .stdout(predicate::str::contains("auto"))
        .stdout(predicate::str::contains("extended_hours"))
        .stdout(predicate::str::contains("--target-id"));
}

#[test]
fn quotes_help_explains_batch_symbol_reads() {
    tv().args(["quotes", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[SYMBOLS]"))
        .stdout(predicate::str::contains("Desktop-free"))
        .stdout(predicate::str::contains("data.items"));
}

#[test]
fn quotes_rejects_invalid_inputs_before_connecting() {
    let no_symbols = tv()
        .env("TV_CDP_PORT", "9")
        .arg("quotes")
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(no_symbols.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quotes");
    assert_eq!(value["error"]["kind"], "validation");

    let blank_symbol = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quotes", " "])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(blank_symbol.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
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
    let stderr = String::from_utf8(blank_symbol.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "fundamentals");
    assert_eq!(value["error"]["kind"], "validation");

    let unknown_field = tv()
        .env("TV_CDP_PORT", "9")
        .args(["fundamentals", "NYSE:IONQ", "--field", "banana"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(unknown_field.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "fundamentals");
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["details"]["supported_fields"]
            .as_array()
            .unwrap()
            .contains(&json!("earnings_release_next_date"))
    );

    let unknown_group = tv()
        .env("TV_CDP_PORT", "9")
        .args(["fundamentals", "NYSE:IONQ", "--group", "banana"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(unknown_group.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
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
fn bars_help_explains_lab_gate_and_desktop_free_boundary() {
    tv().args(["bars", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SYMBOL>"))
        .stdout(predicate::str::contains("--timeframe"))
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("TV_EXPERIMENTAL_BARS=1"))
        .stdout(predicate::str::contains("Desktop"))
        .stdout(predicate::str::contains("tv ohlcv"));
}

#[test]
fn bars_rejects_disabled_gate_before_network() {
    let assert = tv()
        .env_remove("TV_EXPERIMENTAL_BARS")
        .env("TV_CDP_PORT", "9")
        .args(["bars", "NASDAQ:AAPL", "--count", "5"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(
        value["error"]["details"]["required_env"],
        "TV_EXPERIMENTAL_BARS"
    );
}

#[test]
fn bars_rejects_invalid_inputs_before_network() {
    let bare_symbol = tv()
        .env("TV_EXPERIMENTAL_BARS", "1")
        .env("TV_CDP_PORT", "9")
        .args(["bars", "AAPL", "--count", "5"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(bare_symbol.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(
        value["error"]["details"]["expected_format"],
        "EXCHANGE:SYMBOL"
    );

    let zero_count = tv()
        .env("TV_EXPERIMENTAL_BARS", "1")
        .env("TV_CDP_PORT", "9")
        .args(["bars", "NASDAQ:AAPL", "--count", "0"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(zero_count.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["maximum"], 500);

    let too_many = tv()
        .env("TV_EXPERIMENTAL_BARS", "1")
        .env("TV_CDP_PORT", "9")
        .args(["bars", "NASDAQ:AAPL", "--count", "501"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(too_many.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "bars");
    assert_eq!(value["error"]["kind"], "validation");
    assert_eq!(value["error"]["details"]["maximum"], 500);
}

#[test]
fn symbol_help_explains_read_set_and_set_flag_absence() {
    tv().args(["symbol", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[SYMBOL]"))
        .stdout(predicate::str::contains("Run without SYMBOL"))
        .stdout(predicate::str::contains("There is no --set flag"))
        .stdout(predicate::str::contains("--target-id"));
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
    tv().args(["screener", "open", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--full-page"));

    tv().args(["screener", "screens", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("active"))
        .stdout(predicate::str::contains("actions"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("switch"))
        .stdout(predicate::str::contains("save"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("rename"))
        .stdout(predicate::str::contains("save-as"))
        .stdout(predicate::str::contains("delete"));
    tv().args(["screener", "screens", "actions", "--help"])
        .assert()
        .success();
    tv().args(["screener", "screens", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--catalog"));
    tv().args(["screener", "screens", "switch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--catalog"));
    tv().args(["screener", "screens", "save", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "screens", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "screens", "rename", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--to"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "screens", "save-as", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "screens", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--confirm-delete"));

    tv().args(["screener", "filters", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("actions"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("modify"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("clear"));
    tv().args(["screener", "filters", "actions", "--help"])
        .assert()
        .success();
    tv().args(["screener", "filters", "modify", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--index"))
        .stdout(predicate::str::contains("--text"))
        .stdout(predicate::str::contains("--min"))
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--option"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "filters", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--min"))
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "filters", "remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--index"))
        .stdout(predicate::str::contains("--text"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "filters", "clear", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--confirm-clear"));

    tv().args(["screener", "columns", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("actions"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("reorder"));
    tv().args(["screener", "columns", "config", "--help"])
        .assert()
        .success();
    tv().args(["screener", "columns", "remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--index"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "columns", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--id"))
        .stdout(predicate::str::contains("--params-json"))
        .stdout(predicate::str::contains("--after-index"))
        .stdout(predicate::str::contains("--dry-run"));
    tv().args(["screener", "columns", "reorder", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--from-index"))
        .stdout(predicate::str::contains("--to-index"))
        .stdout(predicate::str::contains("--dry-run"));
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
fn screener_filter_mutations_reject_invalid_inputs_before_connecting() {
    let missing_target = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "filters", "remove"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(missing_target.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let conflicting_target = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener", "filters", "remove", "--index", "0", "--text", "PER",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(conflicting_target.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let modify_missing_target = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "filters", "modify", "--min", "0", "--max", "5"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(modify_missing_target.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let modify_conflicting_target = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener", "filters", "modify", "--index", "0", "--text", "EMA", "--min", "0",
            "--max", "5",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(modify_conflicting_target.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let modify_missing_range = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "filters", "modify", "--text", "EMA"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(modify_missing_range.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let modify_blank_option = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener", "filters", "modify", "--text", "EMA", "--option", " ",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(modify_blank_option.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let modify_option_with_range = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener", "filters", "modify", "--text", "EMA", "--option", "買い", "--min", "0",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(modify_option_with_range.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let modify_invalid_range = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener", "filters", "modify", "--text", "EMA", "--min", "0", "--max", "7",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(modify_invalid_range.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let add_blank_name = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "filters", "add", "--name", " ", "--min", "70"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(add_blank_name.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let add_missing_range = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "filters", "add", "--name", "RSI"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(add_missing_range.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let clear_without_confirmation = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "filters", "clear"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(clear_without_confirmation.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let same_index_reorder = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener",
            "columns",
            "reorder",
            "--from-index",
            "1",
            "--to-index",
            "1",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(same_index_reorder.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn screener_column_remove_rejects_invalid_inputs_before_connecting() {
    let missing_target = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "columns", "remove"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(missing_target.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let conflicting_target = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener", "columns", "remove", "--index", "0", "--name", "Price",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(conflicting_target.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let blank_add_id = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "columns", "add", "--id", " "])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(blank_add_id.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_params = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "screener",
            "columns",
            "add",
            "--id",
            "Change",
            "--params-json",
            "[]",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(invalid_params.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn screener_screen_switch_rejects_empty_name_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["screener", "screens", "switch", "--name", "   "])
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
fn screener_screen_lifecycle_rejects_invalid_inputs_before_connecting() {
    let output = tv()
        .args(["screener", "screens", "create", "--name", "   "])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stderr
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let output = tv()
        .args([
            "screener",
            "screens",
            "create",
            "--name",
            "Production Screen",
        ])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stderr
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let output = tv()
        .args([
            "screener",
            "screens",
            "rename",
            "--name",
            "CLI-Test1",
            "--to",
            "CLI-Test1",
        ])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stderr
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let output = tv()
        .args(["screener", "screens", "delete", "--name", "CLI-Test1"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stderr
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["command"], "screener");
    assert_eq!(value["error"]["kind"], "validation");

    let output = tv()
        .args([
            "screener",
            "screens",
            "delete",
            "--name",
            "Production Screen",
            "--confirm-delete",
        ])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stderr
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
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
fn scanner_scan_rejects_invalid_inputs_before_network() {
    let invalid_market = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--market", "global"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(invalid_market.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "scanner");
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_column = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--columns", "name,unknown"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(invalid_column.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_limit = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--limit", "0"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(invalid_limit.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_sector = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--sector", " "])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(invalid_sector.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_rsi = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--max-rsi", "101"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(invalid_rsi.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["error"]["kind"], "validation");

    let invalid_recommendation = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--min-recommendation", "-1.1"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(invalid_recommendation.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["error"]["kind"], "validation");

    let signed_change_reaches_scanner_validation = tv()
        .env("TV_CDP_PORT", "9")
        .args(["scanner", "scan", "--max-change", "-5", "--limit", "0"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(
        signed_change_reaches_scanner_validation
            .get_output()
            .stderr
            .clone(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["error"]["kind"], "validation");
    assert!(
        value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("--limit")
    );
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
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("selected chart target"))
        .stdout(predicate::str::contains("--target-id"))
        .stdout(predicate::str::contains("tv tab list"));
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
fn timeframe_help_explains_interval_is_not_command() {
    tv().args(["timeframe", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tv timeframe D"))
        .stdout(predicate::str::contains("`interval` is not a `tv` command"));
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
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("add-bulk"))
        .stdout(predicate::str::contains("remove"));
    tv().args(["watchlist", "add-bulk", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--delay-ms"))
        .stdout(predicate::str::contains("--allow-partial"));
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
        .stdout(predicate::str::contains("DIRECTION"))
        .stdout(predicate::str::contains("--direction"))
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
        .stdout(predicate::str::contains("--interval"))
        .stdout(predicate::str::contains("--duration-ms"))
        .stdout(predicate::str::contains("--max-events"))
        .stdout(predicate::str::contains("--heartbeat-ms"));

    tv().args(["stream", "quote", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--duration-ms"))
        .stdout(predicate::str::contains("--max-events"))
        .stdout(predicate::str::contains("--heartbeat-ms"));
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
fn watchlist_add_bulk_rejects_invalid_inputs_before_connecting() {
    let no_symbols = tv()
        .env("TV_CDP_PORT", "9")
        .args(["watchlist", "add-bulk"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(no_symbols.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "watchlist");
    assert_eq!(value["error"]["kind"], "validation");

    let bad_delay = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "watchlist",
            "add-bulk",
            "NASDAQ:AAPL",
            "--delay-ms",
            "10001",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(bad_delay.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "watchlist");
    assert_eq!(value["error"]["kind"], "validation");

    let mut args = vec!["watchlist".to_string(), "add-bulk".to_string()];
    args.extend((0..51).map(|index| format!("NASDAQ:TEST{index}")));
    let too_many = tv()
        .env("TV_CDP_PORT", "9")
        .args(args)
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(too_many.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["command"], "watchlist");
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
fn pine_alertconditions_requires_source_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "alertconditions"])
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
fn pine_alertconditions_runs_without_cdp_connection() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["pine", "alertconditions"])
        .write_stdin(
            r#"//@version=6
indicator("Signals")
plot(close)
alertcondition(close > open, "Long", "Long message")"#,
        )
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["success"], true);
    assert_eq!(value["command"], "pine");
    assert_eq!(value["data"]["input_source"], "stdin");
    assert_eq!(value["data"]["candidate_count"], 1);
    assert_eq!(value["data"]["candidates"][0]["alert_cond_id"], "plot_1");
    assert_eq!(value["data"]["candidates"][0]["title"], "Long");
}

#[test]
fn pine_alertconditions_help_is_available() {
    let assert = tv()
        .args(["pine", "alertconditions", "--help"])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Discover Pine alertcondition() candidates"));
    assert!(stdout.contains("--file"));
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
fn stream_rejects_invalid_observation_controls_before_connecting() {
    for (flag, field) in [
        ("--duration-ms", "duration_ms"),
        ("--max-events", "max_events"),
        ("--heartbeat-ms", "heartbeat_ms"),
    ] {
        let assert = tv()
            .env("TV_CDP_PORT", "9")
            .args(["stream", "quote", flag, "0"])
            .assert()
            .failure()
            .code(1);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        let value: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["command"], "stream");
        assert_eq!(value["error"]["kind"], "validation");
        assert_eq!(value["error"]["details"]["field"], field);
    }
}

#[test]
fn quote_rejects_empty_symbol_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args(["quote", " "])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
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
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
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
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "quote");
    assert_eq!(value["error"]["kind"], "connection");
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
            "--direction",
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
            "--direction",
            "long",
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
fn alert_create_indicator_normal_mode_attempts_connection_after_source_validation() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "alert",
            "create-indicator",
            "--script",
            "Signals",
            "--condition-title",
            "Long",
        ])
        .write_stdin("alertcondition(close > open, \"Long\")")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "alert");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn alert_create_indicator_rejects_conflicting_condition_selectors_before_connecting() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "alert",
            "create-indicator",
            "--script",
            "Signals",
            "--condition-title",
            "Long",
            "--alert-cond-id",
            "plot_1",
            "--dry-run",
        ])
        .write_stdin("alertcondition(close > open, \"Long\")")
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
fn alert_create_indicator_dry_run_attempts_connection_after_source_validation() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .args([
            "alert",
            "create-indicator",
            "--script",
            "Signals",
            "--condition-title",
            "Long",
            "--dry-run",
        ])
        .write_stdin("alertcondition(close > open, \"Long\")")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "alert");
    assert_eq!(value["error"]["kind"], "connection");
}

#[test]
fn alert_create_indicator_help_is_available() {
    let assert = tv()
        .args(["alert", "create-indicator", "--help"])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Create or preview a Pine alertcondition() alert"));
    assert!(stdout.contains("--condition-title"));
    assert!(stdout.contains("--alert-cond-id"));
    assert!(stdout.contains("--dry-run"));
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
    assert_eq!(value["error"]["details"]["cdp_port"], 9);
    assert!(
        value["error"]["details"]["next_action_hint"]
            .as_str()
            .unwrap()
            .contains("tv launch")
    );
}

#[test]
fn readiness_connection_failure_uses_structured_json_and_exit_code_2() {
    let assert = tv()
        .env("TV_CDP_PORT", "9")
        .arg("readiness")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let value: Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "readiness");
    assert_eq!(value["error"]["kind"], "connection");
    assert_eq!(value["error"]["details"]["cdp_port"], 9);
    assert!(
        value["error"]["details"]["next_action_hint"]
            .as_str()
            .unwrap()
            .contains("tv launch")
    );
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
