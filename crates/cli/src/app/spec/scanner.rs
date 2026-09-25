//! Existing REST scan modes and bounded sequential-observation semantics.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if path != ["scanner", "scan"] {
        return None;
    }
    Some(json!({
        "source": "scanner_scan_rest",
        "requires": {"desktop": false, "authentication": false},
        "effects": {"chart_mutation": false, "account_mutation": false, "local_file_write": false},
        "output_contract": null,
        "constraints": {
            "market": {"choices": ["america"], "trimmed": true, "case_sensitive": true},
            "limit": {"default": 20, "minimum": 1, "clamped_maximum": 100},
            "offset": {"minimum": 0, "ordering": "offset + effective limit must fit usize"},
            "max_results": {"minimum": 1, "maximum": 10000, "conflicts": ["offset", "limit"]},
            "page_size": {"minimum": 1, "maximum": 100, "default": 100, "requires": "max_results"},
            "page_budget": "ceil(max_results / page_size) <= 100",
            "columns": {
                "format": "comma-separated exact field names; trim and discard empty entries",
                "minimum_fields": 1,
                "deduplicated": false,
                "default": ["name", "description", "close", "change", "volume", "market_cap_basic"],
                "choices": SUPPORTED_FIELDS
            },
            "sort": {"default": "market_cap_basic", "choices": SUPPORTED_FIELDS, "trimmed": true},
            "direction": {"default": "desc", "mutually_exclusive": ["asc", "desc"]},
            "exchange": {"trimmed": true, "case": "ASCII uppercase", "blank_entries": "ignored"},
            "string_filters": {"arguments": ["sector", "industry", "symbol_type", "subtype"], "trimmed": true, "nonblank": true},
            "numeric_filters": {
                "all_finite": true,
                "nonnegative": ["min_price", "max_price", "min_volume", "min_market_cap", "min_relative_volume", "max_pe", "min_average_volume"],
                "signed": ["min_change", "max_change", "min_performance_week", "max_performance_week", "min_performance_month", "max_performance_month", "min_performance_quarter", "max_performance_quarter"],
                "rsi": {"minimum": 0, "maximum": 100},
                "recommendation": {"minimum": -1, "maximum": 1},
                "provider_operators": {"min": "greater", "max": "less", "strings": "in_range"},
                "min_max_order_checked": false
            }
        },
        "variants": [
            {"when": {"absent": ["offset", "max_results"]}, "operation": "first_page"},
            {"when": {"present": ["offset"], "absent": ["max_results"]}, "operation": "explicit_page"},
            {"when": {"present": ["max_results"]}, "operation": "bounded_sequential_population"}
        ],
        "discovery": [{
            "argument": "columns",
            "argv": ["tv", "scanner", "metainfo"],
            "result_path": "data.fields[].name",
            "limit": "provider metadata can exceed this CLI's supported field list"
        }],
        "limits": [
            "This credential-free HTTP source is separate from Desktop Screener and official MCP. It does not change saved screens or filters.",
            "First/explicit page returns at most limit rows and can have a null total_count. A successful page is not proof of complete population coverage; offset=0 still selects the explicit-page output variant.",
            "max_results is an allowed population bound, not a request for the first N matches. Aggregate mode fails if totalCount is missing, exceeds that bound, a page is short/overfull or a page request fails; it does not return the accumulated subset as success.",
            "Aggregate pages are sequential observations. Symbols deduplicate by exact string with first occurrence retained; inspect raw_count, duplicate_count, pages_fetched, total_count_changed and observation timestamps.",
            "No duplicate/total drift flag does not prove a stable snapshot: membership or sort order can change while totals stay equal. The query fingerprint identifies request fields, not dataset identity.",
            "Filter bound validation is distinct from provider matching. Minimum/maximum filters use greater/less operators, not promised inclusive thresholds. Inverted bounds are not locally rejected.",
            "symbol_type is the clap argument ID for --type. Field values and units depend on the provider; missing fields are not zero and results carry no realtime guarantee.",
            "Inspect returned columns, filters and sort before analysis. No automatic source fallback or page retry is authorized by a failed scan."
        ],
        "examples": [
            ["tv", "scanner", "scan", "--columns", "name,close,volume", "--limit", "20"],
            ["tv", "scanner", "scan", "--offset", "100", "--limit", "50"],
            ["tv", "scanner", "scan", "--min-volume", "1000000", "--max-results", "1000", "--page-size", "100"]
        ]
    }))
}

// Mirrors the existing scanner's private allowlist; adding a field here alone
// does not add execution support. Provider metainfo is a separate catalog.
const SUPPORTED_FIELDS: &[&str] = &[
    "name",
    "description",
    "close",
    "change",
    "change_abs",
    "volume",
    "average_volume_10d_calc",
    "relative_volume_10d_calc",
    "market_cap_basic",
    "exchange",
    "type",
    "subtype",
    "sector",
    "industry",
    "open",
    "high",
    "low",
    "price_earnings_ttm",
    "earnings_per_share_basic_ttm",
    "dividend_yield_recent",
    "earnings_release_next_date",
    "earnings_release_date",
    "earnings_release_next_time",
    "earnings_release_next_calendar_date",
    "earnings_release_next_trading_date_fq",
    "earnings_release_trading_date_fq",
    "earnings_release_time",
    "earnings_publication_type_next_fq",
    "earnings_publication_type_fq",
    "dividend_amount_recent",
    "dividend_amount_upcoming",
    "dividend_frequency_recent",
    "dividend_frequency_upcoming",
    "next_dividend_date",
    "expected_annual_dividends",
    "Perf.W",
    "Perf.1M",
    "Perf.3M",
    "RSI",
    "Recommend.All",
    "premarket_open",
    "premarket_high",
    "premarket_low",
    "premarket_close",
    "premarket_change",
    "premarket_change_abs",
    "premarket_gap",
    "premarket_volume",
    "postmarket_open",
    "postmarket_high",
    "postmarket_low",
    "postmarket_close",
    "postmarket_change",
    "postmarket_change_abs",
    "postmarket_volume",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn scan_examples_and_mode_conflicts_match_parser() {
        let detail = describe(&["scanner", "scan"]).unwrap();
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
        assert!(
            Cli::try_parse_from([
                "tv",
                "scanner",
                "scan",
                "--max-results",
                "100",
                "--limit",
                "10"
            ])
            .is_err()
        );
        assert!(Cli::try_parse_from(["tv", "scanner", "scan", "--page-size", "10"]).is_err());
    }
}
