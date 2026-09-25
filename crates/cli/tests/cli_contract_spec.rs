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
