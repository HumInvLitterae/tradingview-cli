use assert_cmd::Command;
use serde_json::Value;

#[test]
fn spec_is_offline_even_with_invalid_desktop_configuration() {
    let output = Command::cargo_bin("tv")
        .unwrap()
        .env("TV_CDP_PORT", "invalid")
        .args(["spec", "mcp", "alert", "history"])
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["data"]["contract_version"], "cli_spec.v1");
    assert_eq!(envelope["data"]["semantics"]["requires"]["desktop"], false);
    assert_eq!(
        envelope["data"]["semantics"]["output_contract"],
        "mcp_alert_history.v1"
    );
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
