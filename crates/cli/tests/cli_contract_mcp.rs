use assert_cmd::Command;
use serde_json::Value;

fn tv() -> Command {
    Command::cargo_bin("tv").unwrap()
}

#[test]
fn independent_mcp_help_does_not_change_bars_entry() {
    let output = tv()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(output).unwrap();
    for command in [
        "login", "status", "bars", "logout", "search", "columns", "symbol",
    ] {
        assert!(help.contains(command));
    }
    let old = tv()
        .args(["bars", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(!String::from_utf8(old).unwrap().contains("--backend"));
}

#[test]
fn invalid_mcp_requests_fail_before_cdp_state_store_or_provider_access() {
    for args in [
        vec!["mcp", "search", " "],
        vec!["mcp", "search", "Example", "--type", "unsupported"],
        vec!["mcp", "columns", "--market", "america"],
        vec!["mcp", "symbol", "EXAMPLE"],
        vec![
            "mcp",
            "symbol",
            "NASDAQ:EXAMPLE",
            "--columns",
            "close,close",
        ],
        vec!["mcp", "bars", "AAPL"],
        vec!["mcp", "bars", "NASDAQ:EXAMPLE", "--timeframe", "5m"],
        vec!["mcp", "bars", "NASDAQ:EXAMPLE", "--count", "0"],
        vec!["mcp", "bars", "NASDAQ:EXAMPLE", "--count", "5001"],
        vec!["mcp", "bars", "NASDAQ:EXAMPLE", "--from", "2024-01-01"],
        vec!["--target-id", "synthetic-target", "mcp", "status"],
    ] {
        let output = tv()
            .args(args)
            .env("TV_CDP_PORT", "invalid")
            .env("HOME", "relative-home")
            .env("LOCALAPPDATA", "relative-state")
            .env("RUST_LOG", "trace")
            .assert()
            .code(1)
            .get_output()
            .clone();
        assert!(output.stdout.is_empty());
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["command"], "mcp");
        assert_eq!(error["error"]["kind"], "validation");
        assert_eq!(
            error["error"]["details"]["contract_version"],
            "mcp_error.v1"
        );
        assert_eq!(error["error"]["details"]["tool_attempts"], 0);
    }
}

#[test]
fn mcp_status_ignores_cdp_configuration_and_rejects_invalid_local_root() {
    let output = tv()
        .args(["mcp", "status"])
        .env("TV_CDP_PORT", "invalid")
        .env("HOME", "relative-home")
        .env("LOCALAPPDATA", "relative-state")
        .env("XDG_STATE_HOME", "relative-state")
        .env("RUST_LOG", "trace")
        .assert()
        .code(1)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["command"], "mcp");
    assert_eq!(error["error"]["details"]["code"], "local_state_unavailable");
}

#[test]
fn credential_worker_rejects_invalid_ipc_without_normal_logs_or_envelope() {
    let output = tv()
        .arg("--credential-worker")
        .env("RUST_LOG", "trace")
        .write_stdin("invalid synthetic input")
        .assert()
        .code(1)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
