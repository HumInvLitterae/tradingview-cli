use assert_cmd::Command;
use serde_json::Value;

#[test]
fn schema_exports_without_desktop_configuration_or_provider_access() {
    for path in [vec![], vec!["values"], vec!["mcp", "bars"]] {
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
            assert_eq!(value["data"]["commands"].as_array().unwrap().len(), 2);
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
