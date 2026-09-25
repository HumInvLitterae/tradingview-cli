//! Desktop-free evidence packets and their interpretation limits.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if let Some(result) = analysis_support(path) {
        return Some(result);
    }
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

fn analysis_support(path: &[&str]) -> Option<Value> {
    let mut result = json!({
        "source": "scanner_fundamentals_rest",
        "output_contract": null,
        "requires": {"desktop": false, "authentication": false},
        "effects": {"chart_mutation": false, "account_mutation": false},
        "constraints": {},
        "limits": [
            "Uses credential-free scanner REST, not official MCP. No Desktop fallback, ranking, recommendation or synchronized market snapshot is implied."
        ]
    });
    match path {
        ["fundamentals"] => {
            let snapshot = describe(&["snapshot"])?;
            result["constraints"] = snapshot["constraints"].clone();
            result["constraints"]["selection"]["scope"] = json!("fundamentals fields");
            result["examples"] = json!([
                ["tv", "fundamentals", "NASDAQ:EXAMPLE"],
                [
                    "tv",
                    "fundamentals",
                    "NASDAQ:EXAMPLE",
                    "--group",
                    "valuation"
                ]
            ]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Reads the america scanner market. Empty or ambiguous rows fail; candidate search can supplement validation errors but does not automatically select another symbol."),
                json!("The response exposes requested_symbol and observed_symbol. Local response identity validation compares bare symbols, not exchange prefixes; inspect the observed exchange before combining evidence."),
                json!("field_values preserves provider values including explicit null. missing_fields lists absent array positions only, so an empty list does not mean every requested value is known. No currency, period, freshness or financial interpretation is inferred from the raw values."),
                json!("No versioned success contract is emitted. Group expansion precedes explicit fields and deduplicates by first occurrence; groups do not change the source.")
            ]);
        }
        ["events", "compare"] => {
            result["output_contract"] = json!("events_compare.v1");
            result["constraints"] = json!({
                "symbols": {"min_items": 2, "max_items": 25, "trimmed": true, "nonblank": true, "deduplicated": false},
                "event_type": {"choices": ["all", "earnings", "dividends"]}
            });
            result["examples"] = json!([[
                "tv",
                "events",
                "compare",
                "NASDAQ:EXAMPLE",
                "NYSE:OTHER",
                "--event-type",
                "earnings"
            ]]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Preserves request order, duplicates and requested_index. Each successful item contains events.v1; failed items retain sanitized failure_details. Even all-item failure can return outer success: inspect item status and summary.error_count."),
                json!("Shapes scanner fundamentals into event entries, not a complete calendar or historical event series. no_events_returned does not establish that no event exists. Inspect source_availability and raw field readback."),
                json!("Does not infer timezone, before/after-market session, confirmed/estimated status or event significance. Summary counts reflect shaped entries and successful items, not calendar completeness.")
            ]);
        }
        ["watch", "compare"] => {
            result["source"] = json!("scanner_scan_rest");
            result["output_contract"] = json!("watch_compare.v1");
            result["constraints"] = json!({
                "symbols": {"min_items": 2, "max_items": 25, "trimmed": true, "nonblank": true, "deduplicated": false},
                "interval": {"minimum": 1000, "unit": "milliseconds"},
                "heartbeat_ms": {"minimum": 1000},
                "duration_ms": {"minimum": 1, "maximum": 300000},
                "max_events": {"minimum": 1, "optional": true}
            });
            result["examples"] = json!([[
                "tv",
                "watch",
                "compare",
                "NASDAQ:EXAMPLE",
                "NYSE:OTHER",
                "--duration-ms",
                "5000",
                "--max-events",
                "2"
            ]]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Emits JSONL success envelopes on stdout: readiness, sample, heartbeat and summary. Poll errors go to stderr and polling continues. Readiness validates inputs only; it does not probe provider availability or resolve symbols."),
                json!("Polls quotes, not the multi-section tv compare packet. Preserve ordered item errors and inspect resolved_count/error_count; successful loop completion does not prove successful or complete observations."),
                json!("Consecutive equal samples suppress output after excluding _ts, _event, elapsed_ms and poll_index. max_events counts emitted samples only. Heartbeats indicate loop activity, not new market data; _ts is client emission time."),
                json!("The interval starts after each poll completes. Duration is checked between polls, not enforced as an in-flight request deadline; slow requests can exceed the requested duration and delay heartbeats."),
                json!("Summary last_resolved_count/last_error_count describe the latest successful poll, not cumulative coverage. Inspect poll_error_count too. A closed stdout pipe ends cleanly and may omit the summary. This is bounded polling, not a realtime feed or daemon.")
            ]);
        }
        _ => return None,
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
        for path in [
            vec!["snapshot"],
            vec!["compare"],
            vec!["fundamentals"],
            vec!["events", "compare"],
            vec!["watch", "compare"],
        ] {
            let detail = describe(&path).unwrap();
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
