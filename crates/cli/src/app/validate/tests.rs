use std::ffi::OsString;

use serde_json::{Value, json};
use tradingview_mcp::Operation;

use super::check;

fn argv(parts: &[&str]) -> Vec<OsString> {
    parts.iter().map(OsString::from).collect()
}

#[test]
fn supported_requests_apply_only_local_checks() {
    for parts in [
        vec!["values"],
        vec!["--target-id", "synthetic-target", "values"],
        vec!["mcp", "bars", "NASDAQ:EXAMPLE"],
        vec![
            "mcp",
            "--timeout",
            "180",
            "bars",
            "NASDAQ:EXAMPLE",
            "--timeframe",
            "1M",
            "--count",
            "5000",
        ],
    ] {
        let value = check(&argv(&parts)).unwrap();
        assert_eq!(value["status"], "valid");
        assert_eq!(value["checks"]["runtime"], "not_checked");
        assert!(!value.to_string().contains("synthetic-target"));
        assert!(!value.to_string().contains("NASDAQ:EXAMPLE"));
    }
}

#[test]
fn errors_report_coverage_without_echoing_candidate_values() {
    for (parts, code, status) in [
        (vec![], "invalid_syntax", "invalid"),
        (vec!["private-secret"], "invalid_syntax", "invalid"),
        (
            vec!["mcp", "bars", "NASDAQ:EXAMPLE", "--count", "private-secret"],
            "invalid_syntax",
            "invalid",
        ),
        (
            vec!["mcp", "bars", "private-secret"],
            "invalid_request",
            "invalid",
        ),
        (
            vec!["mcp", "bars", "NASDAQ:EXAMPLE", "--count", "5001"],
            "invalid_request",
            "invalid",
        ),
        (
            vec![
                "mcp",
                "bars",
                "NASDAQ:EXAMPLE",
                "--timeframe",
                "private-secret",
            ],
            "unsupported_capability",
            "invalid",
        ),
        (
            vec!["mcp", "bars", "NASDAQ:EXAMPLE", "--from", "private-secret"],
            "unsupported_capability",
            "invalid",
        ),
        (
            vec![
                "--target-id",
                "private-secret",
                "mcp",
                "bars",
                "NASDAQ:EXAMPLE",
            ],
            "unsupported_capability",
            "invalid",
        ),
        (
            vec!["mcp", "--timeout", "0", "bars", "NASDAQ:EXAMPLE"],
            "invalid_request",
            "invalid",
        ),
        (vec!["data", "equity"], "unsupported_command", "unsupported"),
        (vec!["mcp", "login"], "unsupported_command", "unsupported"),
        (vec!["--help"], "non_executable_request", "unsupported"),
        (
            vec!["values", "--help"],
            "non_executable_request",
            "unsupported",
        ),
        (vec!["--version"], "non_executable_request", "unsupported"),
        (
            vec!["spec", "values"],
            "non_executable_request",
            "unsupported",
        ),
        (
            vec!["schema", "values"],
            "non_executable_request",
            "unsupported",
        ),
        (
            vec!["validate", "--", "values"],
            "non_executable_request",
            "unsupported",
        ),
        (vec!["--credential-worker"], "invalid_syntax", "invalid"),
    ] {
        let error = check(&argv(&parts)).unwrap_err();
        assert_eq!(error.exit_code(), 1);
        assert!(!format!("{error:?}").contains("private-secret"));
        let details = error.details.unwrap();
        assert_eq!(details["code"], code, "{parts:?}");
        assert_eq!(details["status"], status, "{parts:?}");
        assert_eq!(details["checks"]["runtime"], "not_checked");
    }
    let error = check(&argv(&["data", "equity"]))
        .unwrap_err()
        .details
        .unwrap();
    assert_eq!(error["command"], json!(["data", "equity"]));
    assert_eq!(error["checks"]["local_constraints"], "not_checked");
}

#[test]
fn bars_boundaries_match_shared_execution_preparation() {
    for count in [0, 1, 5000, 5001] {
        for timeframe in ["1m", "1D", "1W", "1M", "M", "private-secret"] {
            let actual =
                crate::ops::prepare_mcp_bars("NASDAQ:EXAMPLE", timeframe, count, None, None);
            let result = check(&argv(&[
                "mcp",
                "bars",
                "NASDAQ:EXAMPLE",
                "--timeframe",
                timeframe,
                "--count",
                &count.to_string(),
            ]));
            assert_eq!(result.is_ok(), actual.is_ok());
        }
    }
    for timeout in [0, 1, 30, 180, 181] {
        let request = crate::ops::prepare_mcp_bars("NASDAQ:EXAMPLE", "1D", 20, None, None).unwrap();
        let actual = Operation::Bars(request).timeout_duration(Some(timeout));
        let result = check(&argv(&[
            "mcp",
            "--timeout",
            &timeout.to_string(),
            "bars",
            "NASDAQ:EXAMPLE",
            "--count",
            "20",
        ]));
        assert_eq!(result.is_ok(), actual.is_ok());
    }
    let error: Value = check(&argv(&["mcp", "bars", "NASDAQ:EXAMPLE", "--count", "5001"]))
        .unwrap_err()
        .details
        .unwrap();
    assert_eq!(error["field"], "count");
}
