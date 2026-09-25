use assert_cmd::Command;
use serde_json::Value;

#[test]
fn spec_is_offline_even_with_invalid_desktop_configuration() {
    for path in [
        vec!["mcp", "search"],
        vec!["mcp", "columns"],
        vec!["mcp", "symbol"],
        vec!["mcp", "symbols"],
        vec!["mcp", "bars"],
        vec!["mcp", "alert", "history"],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .arg("spec")
            .args(&path)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
        let data = &envelope["data"];
        assert_eq!(data["contract_version"], "cli_spec.v1");
        assert_eq!(data["semantics"]["requires"]["desktop"], false);
        assert_eq!(data["semantics"]["effects"]["account_mutation"], false);
        assert_eq!(data["coverage"]["validation"], "partial");
        assert!(
            data["semantics"]["output_contract"]
                .as_str()
                .unwrap()
                .starts_with("mcp_")
        );
    }
}

#[test]
fn unknown_spec_path_uses_validation_error_without_connecting() {
    let output = Command::cargo_bin("tv")
        .unwrap()
        .env("TV_CDP_PORT", "invalid")
        .args(["spec", "mcp", "not-a-command"])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["command"], "spec");
    assert!(error.to_string().contains("Unknown spec command path"));
}

#[test]
fn mutation_specs_are_offline_descriptions_not_account_operations() {
    for (family, actions) in [
        ("watchlist", ["create", "update", "add", "remove", "delete"]),
        ("alert", ["create", "update", "stop", "restart", "delete"]),
    ] {
        for action in actions {
            let output = Command::cargo_bin("tv")
                .unwrap()
                .env("TV_CDP_PORT", "invalid")
                .args(["spec", "mcp", family, action])
                .assert()
                .success()
                .get_output()
                .clone();
            assert!(output.stderr.is_empty());
            let value: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                value["data"]["semantics"]["effects"]["account_mutation"],
                true
            );
            assert_eq!(
                value["data"]["semantics"]["execution"]["automatic_retry"],
                false
            );
        }
    }
}

#[test]
fn desktop_specs_do_not_connect_or_resolve_effects_without_arguments() {
    for path in [
        vec!["symbol"],
        vec!["timeframe"],
        vec!["type"],
        vec!["range"],
        vec!["info"],
        vec!["state"],
        vec!["readiness"],
        vec!["tab", "list"],
        vec!["launch"],
        vec!["tab", "switch"],
        vec!["tab", "new"],
        vec!["tab", "close"],
        vec!["chart", "compare"],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .arg("spec")
            .args(&path)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["data"]["coverage"]["semantics"], "documented");
        if ["symbol", "timeframe", "type", "range"].contains(&path[0]) {
            assert!(result["data"]["semantics"]["effects"]["chart_mutation"].is_null());
        }
    }
}

#[test]
fn replay_specs_do_not_advance_practice_or_create_attachments() {
    for action in [
        "start", "step", "stop", "status", "autoplay", "trade", "log",
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .args(["spec", "replay", action])
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let spec = &value["data"]["semantics"];
        assert_eq!(spec["effects"]["replay_state_mutation"], action != "status");
        assert_eq!(
            spec["output_format"],
            if action == "log" { "jsonl" } else { "json" }
        );
    }
}

#[test]
fn ui_specs_remain_offline_with_arbitrary_eval_disabled() {
    for action in [
        "find",
        "click",
        "hover",
        "keyboard",
        "type",
        "scroll",
        "panel",
        "fullscreen",
        "mouse",
        "eval",
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .env_remove("TV_ALLOW_UNSAFE_UI_EVAL")
            .args(["spec", "ui", action])
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["data"]["coverage"]["semantics"], "documented");
        if action == "eval" {
            assert!(value["data"]["semantics"]["effects"]["ui_mutation"].is_null());
        }
    }
}

#[test]
fn indicator_specs_are_offline_and_do_not_create_or_modify_studies() {
    for path in [
        vec!["indicator", "add"],
        vec!["indicator", "get"],
        vec!["indicator", "set"],
        vec!["indicator", "toggle"],
        vec!["indicator", "remove"],
        vec!["data", "indicator"],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .arg("spec")
            .args(&path)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["data"]["coverage"]["semantics"], "documented");
        assert_eq!(
            value["data"]["semantics"]["effects"]["chart_mutation"],
            path[0] == "indicator" && path[1] != "get"
        );
    }
}

#[test]
fn drawing_specs_never_create_or_clear_chart_objects() {
    for action in ["shape", "position", "list", "get", "remove", "clear"] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .args(["spec", "draw", action])
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let effects = &value["data"]["semantics"]["effects"];
        assert_eq!(effects["broker_order"], false);
        if action == "clear" {
            assert!(effects["chart_mutation"].is_null());
        } else {
            assert_eq!(
                effects["chart_mutation"],
                !["list", "get"].contains(&action)
            );
        }
    }
}

#[test]
fn layout_specs_do_not_connect_focus_or_navigate() {
    for path in [
        ["pane", "list"],
        ["pane", "layout"],
        ["pane", "focus"],
        ["pane", "symbol"],
        ["layout", "list"],
        ["layout", "switch"],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .args(["spec", path[0], path[1]])
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let semantics = &value["data"]["semantics"];
        assert_eq!(semantics["requires"]["desktop"], true);
        if path == ["layout", "switch"] {
            assert!(semantics["effects"]["chart_mutation"].is_null());
            assert_eq!(semantics["variants"][0]["effects"]["chart_mutation"], false);
            assert_eq!(semantics["variants"][1]["effects"]["chart_mutation"], true);
        } else {
            assert_eq!(semantics["effects"]["chart_mutation"], path[1] != "list");
        }
    }
}

#[test]
fn pine_specs_never_read_source_connect_or_compile() {
    for action in [
        "get",
        "set",
        "compile",
        "raw-compile",
        "save",
        "new",
        "open",
        "analyze",
        "alertconditions",
        "check",
        "errors",
        "console",
        "list",
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .args(["spec", "pine", action])
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let semantics = &value["data"]["semantics"];
        let desktop = !["analyze", "alertconditions", "check"].contains(&action);
        assert_eq!(semantics["requires"]["desktop"], desktop);
        if desktop {
            assert!(semantics["effects"]["transmits_source"].is_null());
        } else {
            assert_eq!(semantics["effects"]["transmits_source"], action == "check");
        }
        assert_eq!(
            semantics["effects"]["may_open_editor"],
            desktop && action != "list"
        );
        assert_eq!(
            semantics["effects"]["may_save_script"],
            ["save", "raw-compile"].contains(&action)
        );
    }
}

#[test]
fn capture_specs_do_not_connect_move_viewport_or_write_files() {
    for path in [
        vec!["ohlcv"],
        vec!["export", "chart-bars"],
        vec!["scroll"],
        vec!["screenshot"],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .arg("spec")
            .args(&path)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let effects = &value["data"]["semantics"]["effects"];
        assert_eq!(effects["local_file_write"], path[0] == "screenshot");
        assert_eq!(
            effects["chart_mutation"],
            ["export", "scroll"].contains(&path[0])
        );
        if path[0] == "screenshot" {
            assert_eq!(effects["overwrites_existing_file"], true);
        }
    }
}

#[test]
fn market_specs_need_no_query_symbol_or_network() {
    for action in ["search", "bars"] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .args(["spec", action])
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let semantics = &value["data"]["semantics"];
        assert_eq!(semantics["requires"]["desktop"], false);
        assert_eq!(semantics["requires"]["authentication"], false);
        assert_eq!(semantics["effects"]["local_file_write"], false);
        if action == "bars" {
            assert_eq!(semantics["output_contract"], "bars.v1");
            assert_eq!(semantics["variants"][2]["operation"], "validation_error");
        }
    }
}

#[test]
fn quote_specs_do_not_select_or_contact_a_source() {
    for path in [vec!["quote"], vec!["quotes"], vec!["scanner", "metainfo"]] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .arg("spec")
            .args(&path)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let semantics = &value["data"]["semantics"];
        if path == ["quote"] {
            assert!(semantics["source"].is_null());
            assert!(semantics["effects"]["chart_mutation"].is_null());
            assert!(semantics["requires"]["desktop"].is_null());
        } else {
            assert_eq!(semantics["effects"]["chart_mutation"], false);
            assert_eq!(semantics["requires"]["desktop"], false);
        }
    }
}

#[test]
fn analysis_specs_do_not_inspect_or_change_a_chart() {
    for path in [
        vec!["values"],
        vec!["data", "strategy"],
        vec!["data", "trades"],
        vec!["data", "equity"],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .arg("spec")
            .args(&path)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let semantics = &value["data"]["semantics"];
        assert_eq!(semantics["requires"]["desktop"], true);
        assert_eq!(semantics["effects"]["chart_mutation"], false);
        assert_eq!(semantics["effects"]["opens_strategy_tester"], false);
        assert_eq!(semantics["effects"]["changes_study_visibility"], false);
    }
}
