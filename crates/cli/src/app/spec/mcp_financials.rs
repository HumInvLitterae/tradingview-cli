//! Official financial reads, fiscal periods and unconfirmed coverage.

use serde_json::{Value, json};

use super::mcp_reads::{read_metadata, read_timeout, symbol_constraint, symbol_discovery};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [
        "mcp",
        action @ ("financials" | "financial-history" | "forecasts" | "earnings"),
    ] = path
    else {
        return None;
    };
    let mut result = read_metadata();
    result["constraints"] = json!({"timeout": read_timeout()});
    result["constraints"][if *action == "earnings" {
        "symbols"
    } else {
        "symbol"
    }] = if *action == "earnings" {
        json!({"min_items": 1, "max_items": 50, "unique": true, "item": symbol_constraint()})
    } else {
        symbol_constraint()
    };
    result["discovery"] = json!([symbol_discovery(if *action == "earnings" {
        "symbols"
    } else {
        "symbol"
    })]);
    result["limits"] = json!([
        "One authenticated official-MCP read; no Desktop/scanner fallback or automatic pagination. Credential refresh can update local authorization state.",
        "Inspect provider_metadata and client_observation separately from request. Completeness remains unconfirmed; received_at is client receipt time, not financial publication or market-data time.",
        "Currency, unit, scale and as_of remain unknown unless returned. Do not infer them from a symbol, metric name or fiscal label.",
        "For single-symbol reads, returned symbol/period conflicts reject the response. Missing symbol metadata leaves symbol_status unconfirmed; a ticker name alone is not confirmed exchange-qualified identity.",
        "Absent optional projected values can become null. Do not turn unknowns into zero or assume every null distinguishes omitted versus explicit provider null."
    ]);
    let (contract, example) = match *action {
        "financials" => {
            result["constraints"]["period"] =
                json!({"choices": ["fy", "fq", "ttm", "fh", "current"], "default": "ttm"});
            result["constraints"]["metrics"] = json!({
                "cli_option": "--metric",
                "max_items": 50,
                "unique": true,
                "item": {"min_bytes": 1, "max_bytes": 128, "pattern": "^[A-Za-z0-9_|.\\-]+$"},
                "empty": "omit metric selection and use provider defaults"
            });
            result["limits"].as_array_mut().unwrap().push(json!(
                "Fields preserve provider scalar types and names; metric_selection_status stays unconfirmed. Requested aliases or period-specific names are not guessed or guaranteed to match returned fields. Local name validation does not establish metric availability."
            ));
            (
                "mcp_financials.v1",
                json!([
                    "tv",
                    "mcp",
                    "financials",
                    "NASDAQ:EXAMPLE",
                    "--period",
                    "ttm"
                ]),
            )
        }
        "financial-history" => {
            result["constraints"]["period"] = json!({"choices": ["fy", "fq"], "default": "fq"});
            result["limits"].as_array_mut().unwrap().extend([
                json!("Fiscal labels and metric series preserve provider order. Every series must align with labels; null points stay unknown. Labels are not converted to dates or a price-bar timeframe."),
                json!("requested_window_coverage remains unconfirmed, including empty history. value/yoy_pct and capex_latest preserve returned values without computing missing growth rates.")
            ]);
            (
                "mcp_financial_history.v1",
                json!([
                    "tv",
                    "mcp",
                    "financial-history",
                    "NASDAQ:EXAMPLE",
                    "--period",
                    "fq",
                    "--from",
                    "2024-01-01"
                ]),
            )
        }
        "forecasts" => {
            result["limits"].as_array_mut().unwrap().push(json!(
                "Analyst ratings, targets and EPS/revenue estimates are provider opinions or forecasts, not realized results or trade instructions. Required forecast groups can be explicit null; missing groups or invalid value types reject the response."
            ));
            (
                "mcp_forecasts.v1",
                json!(["tv", "mcp", "forecasts", "NASDAQ:EXAMPLE"]),
            )
        }
        "earnings" => {
            result["limits"].as_array_mut().unwrap().extend([
                json!("Items preserve provider order and multiple events for the same requested symbol. symbol_results preserves request order with returned/unreported and item_indices."),
                json!("Unreported means no returned event for that symbol, not no earnings, no listing or full calendar coverage. Empty success is distinct from a provider failure."),
                json!("Unexpected symbols and count mismatches reject the response. Dates remain provider strings; do not infer timezone, before/after-market timing or completeness of the requested window.")
            ]);
            (
                "mcp_earnings.v1",
                json!([
                    "tv",
                    "mcp",
                    "earnings",
                    "NASDAQ:EXAMPLE",
                    "NYSE:OTHER",
                    "--from",
                    "2024-01-01",
                    "--to",
                    "2024-01-31"
                ]),
            )
        }
        _ => unreachable!(),
    };
    if matches!(*action, "financial-history" | "earnings") {
        result["constraints"]["dates"] = json!({
            "arguments": ["from", "to"],
            "format": "valid calendar date YYYY-MM-DD, year 0001..9999",
            "independently_optional": true,
            "ordering": "from <= to when both provided",
            "omitted": "provider default; no client-inferred range"
        });
    }
    result["output_contract"] = json!(contract);
    result["examples"] = json!([example]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::mcp_financials::Request;

    #[test]
    fn financial_examples_and_choices_match_request_validation() {
        for action in ["financials", "financial-history", "forecasts", "earnings"] {
            let spec = describe(&["mcp", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
            if let Some(periods) = spec["constraints"]["period"]["choices"].as_array() {
                for period in periods {
                    let period = period.as_str().unwrap();
                    let request = if action == "financials" {
                        Request::snapshot("NASDAQ:EXAMPLE", period, &[])
                    } else {
                        Request::history("NASDAQ:EXAMPLE", period, None, None)
                    };
                    assert!(request.is_ok());
                }
            }
        }

        assert!(Request::history("NASDAQ:EXAMPLE", "fq", Some("2024-02-29"), None).is_ok());
        assert!(Request::history("NASDAQ:EXAMPLE", "fq", Some("2023-02-29"), None).is_err());
        assert!(
            Request::history(
                "NASDAQ:EXAMPLE",
                "fq",
                Some("2024-02-01"),
                Some("2024-01-01")
            )
            .is_err()
        );
        assert!(
            Request::earnings(
                &["NASDAQ:EXAMPLE".into(), "NASDAQ:EXAMPLE".into()],
                None,
                None
            )
            .is_err()
        );
    }
}
