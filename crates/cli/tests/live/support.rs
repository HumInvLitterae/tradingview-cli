//! Helpers shared by several live smokes.

/// Compares market phases ignoring case and punctuation, so TradingView's
/// `post-market`, `post_market` and `postmarket` name the same phase.
pub fn phase_matches_expected(expected: &str, observed: &str) -> bool {
    normalize_phase(expected) == normalize_phase(observed)
}

fn normalize_phase(phase: &str) -> String {
    phase
        .trim()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

#[test]
fn phase_matching_accepts_aliases_and_keeps_regular_distinct() {
    for (expected, observed) in [
        ("postmarket", "post-market"),
        ("post-market", "postmarket"),
        ("premarket", "pre-market"),
        ("pre-market", "premarket"),
        ("premarket", "pre_market"),
        ("PostMarket", " post market "),
    ] {
        assert!(
            phase_matches_expected(expected, observed),
            "{expected} {observed}"
        );
    }
    for (expected, observed) in [
        ("postmarket", "regular"),
        ("premarket", "regular"),
        ("premarket", "postmarket"),
    ] {
        assert!(
            !phase_matches_expected(expected, observed),
            "{expected} {observed}"
        );
    }
}

/// Compares a reported symbol with what the smoke requested. A qualified
/// request (`EXCHANGE:NAME`) must come back with the same exchange; a bare one
/// is compared by name only.
pub fn symbol_matches_request(requested: &str, reported: &str) -> bool {
    let requested = requested.trim();
    let reported = reported.trim();
    if requested.contains(':') {
        return requested.eq_ignore_ascii_case(reported);
    }
    let name = |symbol: &str| {
        symbol
            .rsplit(':')
            .next()
            .unwrap_or(symbol)
            .to_ascii_uppercase()
    };
    name(requested) == name(reported)
}

/// Reads the selected chart's symbol with `tv symbol`, independently of the
/// command under test.
pub fn read_chart_symbol(tv: &str, target_id: Option<&str>) -> String {
    let mut command = std::process::Command::new(tv);
    if let Some(target_id) = target_id {
        command.args(["--target-id", target_id]);
    }
    let output = command
        .arg("symbol")
        .output()
        .expect("test-built tv binary should execute");
    assert!(output.status.success(), "chart symbol readback failed");
    let envelope: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("chart symbol readback should be JSON");
    envelope
        .pointer("/data/symbol")
        .and_then(serde_json::Value::as_str)
        .expect("chart symbol readback should include data.symbol")
        .to_string()
}

#[test]
fn symbol_matching_keeps_the_exchange_of_qualified_requests() {
    assert!(symbol_matches_request("NASDAQ:AAPL", "nasdaq:aapl"));
    assert!(!symbol_matches_request("NASDAQ:AAPL", "BATS:AAPL"));
    assert!(!symbol_matches_request("NASDAQ:AAPL", "AAPL"));
    assert!(symbol_matches_request("AAPL", "NASDAQ:AAPL"));
    assert!(!symbol_matches_request("AAPL", "NASDAQ:AAPLX"));
}
