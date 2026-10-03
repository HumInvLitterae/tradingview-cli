use assert_cmd::Command;
use serde_json::Value;

#[test]
fn schema_exports_without_desktop_configuration_or_provider_access() {
    for path in [
        vec![],
        vec!["values"],
        vec!["mcp", "bars"],
        vec!["ohlcv"],
        vec!["data", "lines"],
        vec!["data", "boxes"],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .env("TV_CDP_PORT", "invalid")
            .arg("schema")
            .args(&path)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["command"], "schema");
        assert_eq!(value["data"]["contract_version"], "cli_schema.v1");
        if path.is_empty() {
            assert_eq!(value["data"]["commands"].as_array().unwrap().len(), 5);
        } else {
            assert_eq!(value["data"]["coverage"], "documented_fields");
            assert_eq!(
                value["data"]["schema"]["$schema"],
                "https://json-schema.org/draft/2020-12/schema"
            );
        }
    }
}

#[test]
fn schema_errors_distinguish_unknown_from_unsupported_without_echoing_paths() {
    for (args, code) in [
        (vec!["schema", "data", "equity"], "unsupported_command"),
        (vec!["schema", "private-secret"], "unknown_command"),
        (
            vec!["--target-id", "private-secret", "schema", "values"],
            "unsupported_target",
        ),
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .args(args)
            .env("TV_CDP_PORT", "invalid")
            .assert()
            .code(1)
            .get_output()
            .clone();
        assert!(output.stdout.is_empty());
        let value: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(value["error"]["details"]["code"], code);
        assert!(!value.to_string().contains("private-secret"));
    }
}

#[test]
fn validate_is_offline_and_retains_candidate_globals() {
    for args in [
        vec!["validate", "--", "values"],
        vec![
            "validate",
            "--",
            "data",
            "lines",
            "--filter",
            "Example",
            "--verbose",
        ],
        vec!["validate", "--", "data", "boxes", "--filter", ""],
        vec!["validate", "--", "ohlcv", "--summary", "--count", "0"],
        vec![
            "validate",
            "--",
            "--target-id",
            "synthetic-target",
            "ohlcv",
            "--count",
            "501",
        ],
        vec![
            "validate",
            "--",
            "--target-id",
            "synthetic-target",
            "values",
        ],
        vec![
            "validate",
            "--",
            "mcp",
            "--timeout",
            "180",
            "bars",
            "NASDAQ:EXAMPLE",
            "--count",
            "20",
        ],
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .args(args)
            .env("TV_CDP_PORT", "invalid")
            .env("RUST_LOG", "trace")
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["command"], "validate");
        assert_eq!(value["data"]["contract_version"], "cli_validate.v1");
        assert_eq!(value["data"]["status"], "valid");
        assert_eq!(value["data"]["checks"]["runtime"], "not_checked");
        assert!(!value.to_string().contains("synthetic-target"));
    }
}

#[test]
fn validate_rejects_input_without_leaking_values_or_executing_it() {
    for (args, code) in [
        (
            vec![
                "validate",
                "--",
                "mcp",
                "bars",
                "NASDAQ:EXAMPLE",
                "--count",
                "5001",
            ],
            "invalid_request",
        ),
        (
            vec![
                "validate",
                "--",
                "mcp",
                "bars",
                "NASDAQ:EXAMPLE",
                "--count",
                "private-secret",
            ],
            "invalid_syntax",
        ),
        (vec!["validate", "private-secret"], "invalid_syntax"),
        (
            vec![
                "validate",
                "--",
                "--target-id",
                "private-secret",
                "mcp",
                "bars",
                "NASDAQ:EXAMPLE",
            ],
            "unsupported_capability",
        ),
        (
            vec!["--target-id", "private-secret", "validate", "--", "values"],
            "unsupported_target",
        ),
        (
            vec!["validate", "--", "mcp", "login"],
            "unsupported_command",
        ),
        (
            vec!["validate", "--", "mcp", "logout"],
            "unsupported_command",
        ),
        (
            vec!["validate", "--", "--credential-worker"],
            "invalid_syntax",
        ),
        (
            vec!["validate", "--", "data", "equity"],
            "unsupported_command",
        ),
        (vec!["validate", "--", "--help"], "non_executable_request"),
        (
            vec!["validate", "--", "--version"],
            "non_executable_request",
        ),
        (
            vec!["validate", "--", "validate", "--", "values"],
            "non_executable_request",
        ),
        (vec!["validate"], "invalid_syntax"),
    ] {
        let output = Command::cargo_bin("tv")
            .unwrap()
            .args(args)
            .env("TV_CDP_PORT", "invalid")
            .env("RUST_LOG", "trace")
            .assert()
            .code(1)
            .get_output()
            .clone();
        assert!(output.stdout.is_empty());
        let value: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(value["command"], "validate");
        assert_eq!(value["error"]["details"]["code"], code);
        assert!(!value.to_string().contains("private-secret"));
    }
}

#[test]
fn validate_does_not_read_candidate_files_or_connect_to_desktop() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("private-secret-missing.pine");
    let output = Command::cargo_bin("tv")
        .unwrap()
        .args(["validate", "--", "pine", "check", "--file"])
        .arg(&missing)
        .assert()
        .code(1)
        .get_output()
        .clone();
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["details"]["code"], "unsupported_command");
    assert!(!error.to_string().contains("private-secret"));
    assert_eq!(root.path().read_dir().unwrap().count(), 0);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    Command::cargo_bin("tv")
        .unwrap()
        .env(
            "TV_CDP_PORT",
            listener.local_addr().unwrap().port().to_string(),
        )
        .args(["validate", "--", "values"])
        .assert()
        .success();
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
