//! Economic indicator discovery, release observations and dividend modes.

use serde_json::{Value, json};

use super::mcp_reads::{read_metadata, read_timeout, symbol_constraint};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [
        "mcp",
        action @ ("economic-symbols" | "economic-data" | "economic-calendar" | "dividends"),
    ] = path
    else {
        return None;
    };
    let mut result = read_metadata();
    result["constraints"] = json!({"timeout": read_timeout()});
    result["limits"] = json!([
        "Authenticated official-MCP read with no Desktop/scanner fallback or automatic paging. Credential refresh can update local authorization state.",
        "Completeness remains unconfirmed. Receipt time is separate from release, reference-period, ex-dividend and payment dates.",
        "Keep provider units, scale, nulls and notice/source metadata. Do not infer realtime access, timezone or missing values from successful transport."
    ]);
    let (contract, example) = match *action {
        "economic-symbols" => {
            result["constraints"]["country"] = codes(2, 1);
            result["constraints"]["category"] = json!({"choices": CATALOG_CATEGORIES});
            result["constraints"]["search"] =
                json!({"nonblank": true, "max_bytes": 256, "control_characters": false});
            result["variants"] = json!([
                {"when": {"absent": ["country", "category", "search"]}, "output_mode": "overview"},
                {"when": {"absent": ["country"], "any_present": ["category", "search"]}, "output_mode": "indicators"},
                {"when": {"present": ["country"]}, "output_mode": "symbols"}
            ]);
            result["limits"].as_array_mut().unwrap().push(json!(
                "Overview returns categories/countries. Indicators return items[].code, not fetchable ECONOMICS symbols. With country, use items[].symbol unchanged for economic-data; never synthesize a symbol from a code."
            ));
            (
                "mcp_economic_symbols.v1",
                json!([
                    "tv",
                    "mcp",
                    "economic-symbols",
                    "--country",
                    "US",
                    "--category",
                    "prce"
                ]),
            )
        }
        "economic-data" => {
            result["constraints"]["symbol"] = json!({"pattern": "^ECONOMICS:[A-Z0-9]{1,128}$"});
            result["constraints"]["dates"] = dates(false);
            result["discovery"] = json!([{
                "argument": "symbol",
                "argv": ["tv", "mcp", "economic-symbols", "--country", "<country>"],
                "result_path": "data.items[].symbol"
            }]);
            result["limits"].as_array_mut().unwrap().push(json!(
                "Series retains provider order and explicit null values. Echoed symbol must match, dates must be valid and unique, and actual_range is the minimum/maximum returned date, not requested-window coverage. This is economic data, not OHLCV."
            ));
            (
                "mcp_economic_data.v1",
                json!([
                    "tv",
                    "mcp",
                    "economic-data",
                    "ECONOMICS:USCPI",
                    "--from",
                    "2024-01-01"
                ]),
            )
        }
        "economic-calendar" => {
            result["constraints"]["countries"] = codes(2, 50);
            result["constraints"]["countries"]["default"] = json!("US");
            result["constraints"]["currencies"] = codes(3, 50);
            result["constraints"]["category"] = json!({"choices": CALENDAR_CATEGORIES});
            result["constraints"]["min_importance"] =
                json!({"minimum": -1, "maximum": 1, "default": -1});
            result["constraints"]["dates"] = dates(true);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Category vocabulary differs from economic-symbols and retains the accepted literal goverment. Do not silently correct it to government."),
                json!("Preserves actual/forecast/previous and their Raw counterparts separately, without calculating surprises or replacing missing actual values with forecasts."),
                json!("Provider status must be ok and event IDs unique. Event date is retained as provider text; it is not normalized to the user's timezone. requested_window_coverage stays unconfirmed even for nonempty results.")
            ]);
            (
                "mcp_economic_calendar.v1",
                json!([
                    "tv",
                    "mcp",
                    "economic-calendar",
                    "--countries",
                    "US,JP",
                    "--min-importance",
                    "1"
                ]),
            )
        }
        "dividends" => {
            result["constraints"]["symbols"] =
                json!({"max_items": 50, "unique": true, "item": symbol_constraint()});
            result["constraints"]["market"] = json!({"max_bytes": 64, "pattern": "^[a-z_]+$"});
            result["constraints"]["limit"] =
                json!({"minimum": 1, "maximum": 200, "default_in_market_mode": 50});
            result["constraints"]["dates"] = dates(false);
            result["variants"] = json!([
                {"when": {"present": ["symbols"]}, "operation": "explicit_symbols", "rejects": ["market", "from", "to", "limit"]},
                {"when": {"absent": ["symbols"]}, "operation": "market_screen", "requires": ["market"]}
            ]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Explicit-symbol outcomes preserve request order as returned/unreported. Unreported does not establish no dividend or no listing. Returned items retain provider order."),
                json!("Market mode is a bounded screen, not an exhaustive calendar. No offset is available. Duplicate/unexpected symbols and count/limit mismatches reject normalization."),
                json!("Keep recent/upcoming ex-date, payment date and amounts distinct. Missing optional fields normalize to null; do not infer currency, yield units or future payment certainty.")
            ]);
            (
                "mcp_dividends.v1",
                json!(["tv", "mcp", "dividends", "NASDAQ:EXAMPLE"]),
            )
        }
        _ => unreachable!(),
    };
    result["output_contract"] = json!(contract);
    result["examples"] = json!([example]);
    Some(result)
}

fn codes(width: usize, max: usize) -> Value {
    json!({"format": "comma-separated ASCII uppercase codes without spaces", "code_bytes": width, "max_items": max, "unique": true, "trimmed": false})
}

fn dates(calendar: bool) -> Value {
    json!({
        "arguments": ["from", "to"],
        "format": if calendar { "valid YYYY-MM-DD or YYYY-MM-DDTHH:MM:SSZ" } else { "valid YYYY-MM-DD" },
        "independently_optional": true,
        "ordering": "from <= to; mixed date/timestamp formats compare calendar dates only",
        "omitted": "provider default, not inferred client dates"
    })
}

const CATALOG_CATEGORIES: &[&str] = &[
    "gdp", "lbr", "prce", "hlth", "mny", "trd", "gov", "bsnss", "cnsm", "hse", "txs", "enrg",
    "clmt",
];
const CALENDAR_CATEGORIES: &[&str] = &[
    "all",
    "gdp",
    "bonds",
    "business",
    "consumer",
    "goverment",
    "health",
    "housing",
    "labor",
    "money",
    "prices",
    "trade",
    "taxes",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::mcp_economics::{CalendarOptions, DividendOptions, Request};

    #[test]
    fn economics_examples_categories_and_mode_boundaries_match_requests() {
        for action in [
            "economic-symbols",
            "economic-data",
            "economic-calendar",
            "dividends",
        ] {
            let spec = describe(&["mcp", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }

        for category in CATALOG_CATEGORIES {
            assert!(Request::symbols(Some("US"), Some(category), None).is_ok());
        }
        for category in CALENDAR_CATEGORIES {
            assert!(
                Request::calendar(CalendarOptions {
                    category: Some((*category).into()),
                    ..Default::default()
                })
                .is_ok()
            );
        }

        assert!(
            Request::calendar(CalendarOptions {
                countries: Some("US, JP".into()),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            Request::dividends(DividendOptions {
                symbols: vec!["NASDAQ:EXAMPLE".into()],
                limit: Some(20),
                ..Default::default()
            })
            .is_err()
        );
        assert!(Request::dividends(DividendOptions::default()).is_err());
    }
}
