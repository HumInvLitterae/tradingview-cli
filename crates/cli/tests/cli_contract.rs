mod support;

use predicates::prelude::*;

use support::{stderr_json, tv, tv_with_cdp_disconnect};

#[test]
fn version_flag_prints_package_version() {
    tv().arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tv"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));

    tv().arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("tv"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
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
        .stdout(predicate::str::contains("snapshot"))
        .stdout(predicate::str::contains("compare"))
        .stdout(predicate::str::contains("scanner"))
        .stdout(predicate::str::contains("values"))
        .stdout(predicate::str::contains("discover"))
        .stdout(predicate::str::contains("diagnose"))
        .stdout(predicate::str::contains("quotes"))
        .stdout(predicate::str::contains("Get source-labeled quote data"))
        .stdout(predicate::str::contains(
            "Get scanner-backed quotes for multiple symbols",
        ))
        .stdout(predicate::str::contains("Get real-time price quote").not())
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
        .stdout(predicate::str::contains("observe"))
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
fn unknown_command_exits_with_usage_error() {
    let assert = tv().arg("unknown-command").assert().failure().code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn connection_failure_uses_structured_json_and_exit_code_2() {
    let mut command = tv_with_cdp_disconnect();
    let port = command.port();
    let assert = command.arg("status").assert().failure().code(2);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "status");
    assert_eq!(value["error"]["kind"], "connection");
    assert_eq!(value["error"]["details"]["cdp_port"], port);
    assert!(
        value["error"]["details"]["next_action_hint"]
            .as_str()
            .unwrap()
            .contains("tv launch")
    );
}

#[test]
fn readiness_connection_failure_uses_structured_json_and_exit_code_2() {
    let mut command = tv_with_cdp_disconnect();
    let port = command.port();
    let assert = command.arg("readiness").assert().failure().code(2);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "readiness");
    assert_eq!(value["error"]["kind"], "connection");
    assert_eq!(value["error"]["details"]["cdp_port"], port);
    assert!(
        value["error"]["details"]["next_action_hint"]
            .as_str()
            .unwrap()
            .contains("tv launch")
    );
}
