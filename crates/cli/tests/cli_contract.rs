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

fn assert_date_field(label: &str, value: &str) {
    if value == "UNKNOWN" {
        return;
    }
    let parts: Vec<&str> = value.split('-').collect();
    assert_eq!(parts.len(), 3, "unexpected {label}: {value}");
    assert!(
        parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2,
        "unexpected {label}: {value}"
    );
    assert!(
        parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit())),
        "unexpected {label}: {value}"
    );
}

/// `built-at` is an RFC 3339 local timestamp such as `2026-08-21T07:15:01+09:00`.
fn assert_timestamp_field(value: &str) {
    assert_eq!(value.len(), 25, "unexpected built-at: {value}");
    assert_date_field("built-at date", &value[..10]);
    assert_eq!(&value[10..11], "T", "unexpected built-at: {value}");

    let (clock, offset) = value[11..].split_at(8);
    let clock_parts: Vec<&str> = clock.split(':').collect();
    assert_eq!(clock_parts.len(), 3, "unexpected built-at: {value}");
    assert!(
        clock_parts
            .iter()
            .all(|part| part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit())),
        "unexpected built-at: {value}"
    );

    assert!(
        matches!(&offset[..1], "+" | "-"),
        "unexpected built-at: {value}"
    );
    assert_eq!(&offset[3..4], ":", "unexpected built-at: {value}");
    assert!(
        offset[1..3]
            .bytes()
            .chain(offset[4..].bytes())
            .all(|b| b.is_ascii_digit()),
        "unexpected built-at: {value}"
    );
}

fn version_stdout(args: &[&str]) -> String {
    let output = tv()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("version output is UTF-8")
}

#[test]
fn version_flag_prints_build_provenance() {
    let expected = format!(
        "tv {} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("TV_VERSION_COMMIT"),
        env!("TV_VERSION_DATE")
    );

    for flag in ["--version", "-V"] {
        assert_eq!(version_stdout(&[flag]).trim_end(), expected);
    }
}

#[test]
fn verbose_version_flag_prints_detailed_build_provenance() {
    let short = version_stdout(&["--version"]);

    for args in [
        ["--version", "--verbose"].as_slice(),
        ["-V", "--verbose"].as_slice(),
    ] {
        let stdout = version_stdout(args);
        let mut lines = stdout.lines();

        assert_eq!(lines.next(), Some(short.trim_end()));
        assert_eq!(
            lines.collect::<Vec<_>>(),
            vec![
                "binary: tv".to_string(),
                format!("release: {}", env!("CARGO_PKG_VERSION")),
                format!("commit-hash: {}", env!("TV_BUILD_COMMIT_HASH")),
                format!("commit-date: {}", env!("TV_BUILD_COMMIT_DATE")),
                format!("built-at: {}", env!("TV_BUILD_BUILT_AT")),
                format!("dirty: {}", env!("TV_BUILD_DIRTY")),
                format!("target: {}", env!("TV_BUILD_TARGET")),
            ]
        );
    }
}

#[test]
fn verbose_flag_requires_version_flag() {
    let error = stderr_json(&tv().arg("--verbose").assert().failure());

    assert_eq!(error["error"]["kind"], "validation");
    assert!(
        error["error"]["message"]
            .as_str()
            .expect("message is a string")
            .contains("--version"),
        "unexpected message: {error}"
    );
}

#[test]
fn missing_subcommand_reports_help_as_validation_error() {
    let error = stderr_json(&tv().assert().failure());

    assert_eq!(error["error"]["kind"], "validation");
    let message = error["error"]["message"]
        .as_str()
        .expect("message is a string")
        .to_string();
    assert!(
        message.contains("Usage: tv"),
        "unexpected message: {message}"
    );
    assert!(
        message.contains("Commands:"),
        "unexpected message: {message}"
    );
}

#[test]
fn build_stamp_uses_expected_shape() {
    let version_commit = env!("TV_VERSION_COMMIT");
    let hash = version_commit
        .strip_suffix("-dirty")
        .unwrap_or(version_commit);
    assert!(
        hash == "UNKNOWN" || (!hash.is_empty() && hash.bytes().all(|b| b.is_ascii_hexdigit())),
        "unexpected commit field: {version_commit}"
    );

    let commit_hash = env!("TV_BUILD_COMMIT_HASH");
    assert!(
        commit_hash == "UNKNOWN"
            || (commit_hash.len() >= hash.len()
                && commit_hash.bytes().all(|b| b.is_ascii_hexdigit())),
        "unexpected commit-hash field: {commit_hash}"
    );

    assert_date_field("version date", env!("TV_VERSION_DATE"));
    assert_date_field("commit-date", env!("TV_BUILD_COMMIT_DATE"));
    assert_timestamp_field(env!("TV_BUILD_BUILT_AT"));

    assert!(
        matches!(env!("TV_BUILD_DIRTY"), "true" | "false" | "UNKNOWN"),
        "unexpected dirty field: {}",
        env!("TV_BUILD_DIRTY")
    );
    assert!(!env!("TV_BUILD_TARGET").is_empty());

    // The short line reduces the two to the one that describes the binary.
    match env!("TV_BUILD_DIRTY") {
        "true" => assert_eq!(
            env!("TV_VERSION_DATE"),
            &env!("TV_BUILD_BUILT_AT")[..10],
            "a dirty build dates the version line by its build time"
        ),
        "false" => assert_eq!(env!("TV_VERSION_DATE"), env!("TV_BUILD_COMMIT_DATE")),
        _ => assert_eq!(env!("TV_VERSION_DATE"), "UNKNOWN"),
    }
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
    assert_eq!(value["error"]["details"]["failure_stage"], "target_list");
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
    assert_eq!(value["error"]["details"]["failure_stage"], "target_list");
    assert_eq!(value["error"]["details"]["cdp_port"], port);
    assert!(
        value["error"]["details"]["next_action_hint"]
            .as_str()
            .unwrap()
            .contains("tv launch")
    );
}
