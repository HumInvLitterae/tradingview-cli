//! Credential-free market discovery and historical-bar semantics.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let mut result = json!({
        "requires": {"desktop": false, "authentication": false},
        "effects": {"chart_mutation": false, "account_mutation": false, "local_file_write": false},
        "output_contract": null,
        "constraints": {},
        "limits": [
            "This is a credential-free network request, not an offline operation or official MCP.",
            "No implicit Desktop or MCP fallback is used. Source success does not establish realtime access or entitlements."
        ]
    });
    match path {
        ["search"] => {
            result["source"] = json!("symbol_search_rest");
            result["constraints"]["query"] = json!({
                "nonblank": true,
                "normalization": "join positional words with spaces"
            });
            result["limits"].as_array_mut().unwrap().extend([
                json!("Returns at most the first 15 normalized results; this is not exhaustive search or proof of unique identity."),
                json!("Use data.results[].full_name to choose an exchange-qualified symbol. Missing/malformed result arrays normalize to an empty list; empty results alone do not prove absence.")
            ]);
            result["examples"] = json!([["tv", "search", "EXAMPLE"]]);
        }
        ["bars"] => {
            result["source"] = json!("tradingview_bars_ws");
            result["output_contract"] = json!("bars.v1");
            result["constraints"] = json!({
                "symbol": {"nonblank": true, "trimmed": true},
                "required_together": ["from", "to"],
                "timeframe": {
                    "default": "1D",
                    "recent_choices": ["1", "3", "5", "15", "30", "45", "60", "120", "180", "240", "1D", "1W", "1M"],
                    "range_choices": ["1", "5", "15", "30", "60", "1D", "1W", "1M"],
                    "trimmed": true,
                    "aliases": {
                        "1m": "1",
                        "3m": "3",
                        "5m": "5",
                        "15m": "15",
                        "30m": "30",
                        "45m": "45",
                        "1h": "60",
                        "2h": "120",
                        "3h": "180",
                        "4h": "240",
                        "1d": "1D",
                        "D": "1D",
                        "1w": "1W",
                        "W": "1W",
                        "M": "1M"
                    }
                },
                "dates": {"format": "YYYY-MM-DD", "ordering": "from <= to", "to_inclusive": true}
            });
            result["variants"] = json!([
                {
                    "when": {"absent": ["from", "to"]},
                    "operation": "recent_count",
                    "constraints": {"count": {"minimum": 1, "maximum": 500, "default": 100}}
                },
                {
                    "when": {"present": ["from", "to"]},
                    "operation": "date_range",
                    "constraints": {"count": {"minimum": 1, "maximum": 5000, "default": 500, "meaning": "returned-bar safety cap"}}
                },
                {
                    "when": {"exactly_one_present": ["from", "to"]},
                    "operation": "validation_error"
                }
            ]);
            result["discovery"] = json!([{
                "argument": "symbol",
                "argv": ["tv", "search", "<query>"],
                "result_path": "data.results[].full_name"
            }]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Bare symbols trigger REST search and select the first case-insensitive exact symbol match with a qualified name, not a uniqueness check. Pass EXCHANGE:SYMBOL to select an exchange and inspect requested_symbol/resolved_symbol."),
                json!("Symbol resolution can make a network request before later timeframe/count validation. Use spec for offline inspection, not a deliberately invalid bars invocation."),
                json!("Uses an unauthenticated undocumented WebSocket session with split adjustment requested. This does not establish equality with official MCP or Desktop data, or verified session/adjustment conditions."),
                json!("Date bounds filter period-start timestamps from UTC start-of-day through the exclusive start of the day after to. Weekly/monthly bars are not expanded to cover every day in a requested period."),
                json!("Range mode can page older data within bounded limits. Inspect range_alignment, range_fetch_summary, returned_range and range_coverage_status; reaching count is not proof of full period coverage."),
                json!("summary.coverage_status and data_quality.partial_result concern requested count, even when count is a range cap. A range may be complete while fewer than the cap were returned. Range coverage is based on observed timestamp bounds, not an exchange-calendar gap audit."),
                json!("An envelope can succeed with partial bars or incomplete source completion. Inspect data_quality.completed and source_availability as well as coverage. source_failure_stage diagnoses a failure boundary and does not authorize retries.")
            ]);
            result["examples"] = json!([
                ["tv", "bars", "NASDAQ:EXAMPLE", "--count", "20"],
                [
                    "tv",
                    "bars",
                    "NASDAQ:EXAMPLE",
                    "--timeframe",
                    "1D",
                    "--from",
                    "2024-01-01",
                    "--to",
                    "2024-01-31"
                ]
            ]);
        }
        _ => return None,
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn market_examples_parse() {
        for action in ["search", "bars"] {
            let detail = describe(&[action]).unwrap();
            for example in detail["examples"].as_array().unwrap() {
                assert!(
                    Cli::try_parse_from(
                        example
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_str().unwrap())
                    )
                    .is_ok()
                );
            }
        }
    }
}
