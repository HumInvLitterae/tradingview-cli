//! Public-safe clues from provider-declared failures.

use serde_json::{Value, json};

/// These are textual clues, not HTTP statuses or verified root causes.
/// Never retain the provider's arbitrary error string or nested private values.
pub fn provider_error_hints(error: Option<&Value>) -> Value {
    let text = error
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let groups: &[(&str, &[&str])] = &[
        ("rate_limit", &["429", "rate limit", "too many requests"]),
        // This is an error-text reference, not proof of a request made by this client.
        ("screener_endpoint", &["scanner.tradingview.com"]),
        (
            "authorization",
            &["401", "403", "unauthorized", "forbidden", "permission"],
        ),
        (
            "entitlement",
            &["subscription", "premium", "paid plan", "entitlement"],
        ),
        ("timeout", &["timeout", "timed out"]),
        (
            "symbol",
            &["invalid symbol", "unknown symbol", "symbol not found"],
        ),
        (
            "interval",
            &["unsupported interval", "invalid interval", "timeframe"],
        ),
        (
            "implementation",
            &[
                "typeerror",
                "nameerror",
                "keyerror",
                "attributeerror",
                "not defined",
                "unexpected keyword",
                "internal server",
            ],
        ),
        ("not_found", &["404", "not found"]),
    ];
    let matched: Vec<_> = groups
        .iter()
        .filter(|(_, needles)| needles.iter().any(|needle| text.contains(needle)))
        .map(|(name, _)| *name)
        .collect();
    json!({"textual_clues": matched, "root_cause": "unconfirmed"})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_failures_preserve_contracts_without_exposing_provider_text() {
        let bars = crate::mcp_bars::Request::new("NASDAQ:EXAMPLE", "1D", 20).unwrap();
        let data = crate::mcp_data::Request::symbol("NASDAQ:EXAMPLE", &["close".into()]).unwrap();
        for (error, expected) in [
            (
                json!("429 at https://scanner.tradingview.com/private?token=secret"),
                json!(["rate_limit", "screener_endpoint"]),
            ),
            (json!("secret unknown failure"), json!([])),
            (json!({"private": "429 secret"}), json!([])),
            (Value::Null, json!([])),
        ] {
            let payload = json!({"success": false, "error": error});
            let failures = [
                crate::mcp_data::normalize(&data, payload.clone(), 0).unwrap_err(),
                crate::mcp_bars::normalize(&bars, payload, 0).unwrap_err(),
            ];
            for failure in failures {
                let details = failure.details.unwrap();
                assert_eq!(details["code"], "provider_error");
                assert_eq!(details["contract_version"], "mcp_error.v1");
                assert_eq!(details["provider_error_hints"]["textual_clues"], expected);
                assert_eq!(details["provider_error_hints"]["root_cause"], "unconfirmed");
                assert!(details.get("http_status").is_none());
                for forbidden in ["private", "secret", "https", "429"] {
                    assert!(!details.to_string().contains(forbidden));
                }
            }
        }
    }
}
