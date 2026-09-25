//! Desktop-free evidence packets and their interpretation limits.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [action @ ("snapshot" | "compare")] = path else {
        return None;
    };
    let snapshot = *action == "snapshot";
    let mut result = json!({
        "source": if snapshot { "snapshot_desktop_free" } else { "compare_desktop_free" },
        "output_contract": if snapshot { "snapshot.v1" } else { "compare.v1" },
        "requires": {"desktop": false, "authentication": false},
        "effects": {"chart_mutation": false, "account_mutation": false, "executes_follow_up_hints": false},
        "constraints": {},
        "limits": [
            "Collects scanner quote, REST symbol info and scanner fundamentals as separate sections. This is not official MCP or a synchronized market snapshot.",
            "One successful section is sufficient for a successful symbol packet. Inspect sections.*.ok, section errors, missing_evidence and summary separately from envelope success.",
            "Complete coverage means successful sections with no tracked fundamentals missing fields. Quote/info missing-field counts are not exhaustive field audits; complete does not guarantee freshness, entitlements or all values populated.",
            "Top-level symbol identity uses quote, then fundamentals, then info. It does not verify agreement across sections; inspect underlying section identities before combining them.",
            "Follow-up hints are advisory, not ranking or authority to run commands. auto_execute=false; inspect the hinted command's own spec and supply the intended target.",
            "Existing hints can label chart_quote/screenshot non_mutating despite chart symbol switching or file writes. Do not use that flag as proof of no side effects; quote --source chart can switch/restore a symbol and screenshot writes a file.",
            "No implied recommendation, missing-value imputation or automatic Desktop/MCP fallback is performed."
        ]
    });
    if snapshot {
        result["constraints"] = json!({
            "symbol": {"nonblank": true, "trimmed": true},
            "groups": {"choices": ["earnings", "valuation", "dividends", "financials"], "trimmed": true, "case_sensitive": true},
            "fields": {"choices": FIELDS, "trimmed": true, "nonblank": true, "case_sensitive": true},
            "selection": {
                "scope": "fundamentals section only",
                "default_fields": DEFAULT_FIELDS,
                "default_when": "both groups and fields omitted",
                "ordering": "expand groups before explicit fields; deduplicate preserving first occurrence"
            }
        });
        result["limits"].as_array_mut().unwrap().push(json!(
            "If all three sections fail, the outer error details retain the snapshot packet. Groups/fields do not select quote/info fields or change their source."
        ));
        result["examples"] = json!([
            ["tv", "snapshot", "NASDAQ:EXAMPLE"],
            [
                "tv",
                "snapshot",
                "NASDAQ:EXAMPLE",
                "--group",
                "valuation",
                "--field",
                "market_cap_basic"
            ]
        ]);
    } else {
        result["constraints"]["symbols"] = json!({
            "min_items": 2, "max_items": 25,
            "item": {"nonblank": true, "trimmed": true},
            "deduplicated": false
        });
        result["limits"].as_array_mut().unwrap().extend([
            json!("Input order and requested_index are preserved, including duplicates. Each item is ok if any section succeeds; resolved_count does not mean complete evidence for that many symbols."),
            json!("All unresolved items produce an outer error with the comparison packet in details. Mixed success keeps failed items and section errors in a successful packet."),
            json!("Uses default fundamentals fields; compare has no group/field options. Use snapshot for selected-symbol fundamentals selection."),
            json!("movement.regular_change_percent comes from sections.quote.data.change. Unknown regular_change_abs remains unknown; do not reconstruct it from unrelated session prices.")
        ]);
        result["examples"] = json!([["tv", "compare", "NASDAQ:EXAMPLE", "NYSE:OTHER"]]);
    }
    Some(result)
}

// Metadata copies of the market crate's private lists; validate supported names
// through its public I/O-free validator, without adding a public catalog API.
const DEFAULT_FIELDS: &[&str] = &[
    "name",
    "description",
    "exchange",
    "sector",
    "industry",
    "market_cap_basic",
    "price_earnings_ttm",
    "earnings_per_share_basic_ttm",
    "dividend_yield_recent",
    "earnings_release_next_date",
    "earnings_release_next_time",
    "earnings_release_date",
];
const FIELDS: &[&str] = &[
    "name",
    "description",
    "exchange",
    "type",
    "subtype",
    "sector",
    "industry",
    "market_cap_basic",
    "price_earnings_ttm",
    "price_earnings_forward_fy",
    "earnings_per_share_basic_ttm",
    "earnings_per_share_basic_fq",
    "earnings_per_share_fq",
    "earnings_per_share_forecast_next_fq",
    "earnings_per_share_forecast_next_fy",
    "revenue_forecast_next_fq",
    "revenue_forecast_next_fy",
    "total_revenue_ttm",
    "total_revenue_fq",
    "net_income_ttm",
    "net_income_fq",
    "dividend_yield_recent",
    "dividends_yield_current",
    "dividend_ex_date_recent",
    "dividend_ex_date_upcoming",
    "dividend_payment_date_recent",
    "dividend_payment_date_upcoming",
    "dividend_amount_recent",
    "dividend_amount_upcoming",
    "dividend_frequency_recent",
    "dividend_frequency_upcoming",
    "next_dividend_date",
    "expected_annual_dividends",
    "earnings_release_next_date",
    "earnings_release_date",
    "earnings_release_next_time",
    "earnings_release_next_calendar_date",
    "earnings_release_calendar_date",
    "earnings_release_next_trading_date_fy",
    "earnings_release_trading_date_fy",
    "earnings_release_next_trading_date_fq",
    "earnings_release_trading_date_fq",
    "earnings_publication_type_next_fq",
    "earnings_release_time",
    "earnings_publication_type_fq",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_market::validate_fundamentals_selection;

    #[test]
    fn packet_examples_and_fundamentals_choices_match_execution() {
        for action in ["snapshot", "compare"] {
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

        let detail = describe(&["snapshot"]).unwrap();
        for group in detail["constraints"]["groups"]["choices"]
            .as_array()
            .unwrap()
        {
            assert!(
                validate_fundamentals_selection(vec![group.as_str().unwrap().into()], vec![])
                    .is_ok()
            );
        }
        for field in FIELDS.iter().chain(DEFAULT_FIELDS) {
            assert!(validate_fundamentals_selection(vec![], vec![(*field).into()]).is_ok());
        }
        assert!(validate_fundamentals_selection(vec![], vec!["unsupported".into()]).is_err());
    }
}
