use serde_json::{Value, json};
use tradingview_mcp::Operation;
use tradingview_model::{mcp_account::Request, mcp_bars, mcp_data};

fn read_metadata() -> Value {
    json!({
        "source": "tradingview_mcp",
        "requires": {"authentication": true, "desktop": false},
        "effects": {"account_mutation": false, "provider_request": true}
    })
}

pub(super) fn history() -> Value {
    let mut result = read_metadata();
    result["output_contract"] = json!(Request::ALERT_HISTORY_CONTRACT);
    result["constraints"] = json!({
        "symbol": symbol_constraint(),
        "days": {"minimum": 1, "maximum": u32::MAX},
        "limit": {"minimum": 1, "maximum": Request::ALERT_HISTORY_MAX_LIMIT},
        "timeout": read_timeout()
    });
    result["discovery"] = json!([symbol_discovery("symbol")]);
    result["examples"] = json!([[
        "tv",
        "mcp",
        "--timeout",
        "90",
        "alert",
        "history",
        "--symbol",
        "NASDAQ:AAPL",
        "--days",
        "7",
        "--limit",
        "100"
    ]]);
    result["limits"] = json!([
        "Coverage is unconfirmed even when the request succeeds.",
        "Notification messages and webhook contents are excluded.",
        "Credential refresh can update local authorization state."
    ]);
    result
}

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if path == ["mcp", "alert", "history"] {
        return Some(history());
    }
    let ["mcp", name] = path else { return None };
    let (contract, constraints, discovery, examples, interpretation) = match *name {
        "search" => (
            mcp_data::Kind::Search.contract(),
            json!({"query": text_constraint(mcp_data::MAX_TEXT_BYTES),
                "type": {"choices": mcp_data::SEARCH_TYPES}}),
            json!([]),
            json!([["tv", "mcp", "search", "Apple", "--type", "stock"]]),
            json!([
                "Query words are joined with spaces before byte-length validation.",
                "Results are candidates, not automatic symbol selection.",
                "Coverage remains unconfirmed; receipt time is not market-data time."
            ]),
        ),
        "screener" => (
            mcp_data::Kind::Screener.contract(),
            json!({
                "market": {"max_bytes": 64, "pattern": "^[a-z_-]+$", "default": "america", "catalog": "provider-dependent"},
                "filters": {
                    "format": "JSON object",
                    "max_bytes": 16384,
                    "max_fields": 50,
                    "field": text_constraint(128),
                    "numeric_range": {"length": 2, "finite_or_null": true, "ordering": "min <= max when both present", "null": "unbounded"},
                    "string_fields": ["index", "sector", "industry", "analyst_rating"],
                    "string_value": text_constraint(256)
                },
                "sort_by": text_constraint(128),
                "sort_order": {"choices": ["asc", "desc"], "default": "desc"},
                "limit": {"minimum": 1, "maximum": 1000, "default": 100},
                "columns": {
                    "max_items": mcp_data::MAX_COLUMNS,
                    "unique": true,
                    "item": text_constraint(mcp_data::MAX_COLUMN_BYTES),
                    "default_when_empty": mcp_data::DEFAULT_COLUMNS
                },
                "symbol_types": {
                    "cli_option": "--types",
                    "when_present": {"min_items": 1, "max_items": 50, "unique": true, "item": text_constraint(256)}
                },
                "symbolset": {
                    "when_present": {"min_items": 1, "max_items": 50, "unique": true, "item": text_constraint(256)}
                },
                "filter_preset": {
                    "cli_option": "--preset",
                    "choices": [
                        "pullback_with_reversal", "breakout_with_volume", "oversold_healthy",
                        "earnings_runup", "relative_strength", "new_highs"
                    ]
                }
            }),
            json!([{
                "argument": "columns",
                "argv": ["tv", "mcp", "columns", "--search", "<field>"],
                "result_path": "data.columns[].name",
                "limit": "column discovery categories are not the screener's regional market argument"
            }]),
            json!([
                [
                    "tv",
                    "mcp",
                    "screener",
                    "--columns",
                    "close,volume",
                    "--limit",
                    "20"
                ],
                [
                    "tv",
                    "mcp",
                    "screener",
                    "--filters",
                    "{\"close\":[10,null],\"sector\":\"Technology\"}",
                    "--sort-by",
                    "volume",
                    "--preset",
                    "relative_strength"
                ]
            ]),
            json!([
                "One authenticated read with no offset, paging, automatic chunking or source fallback. limit is a returned-row cap, not a complete-population request.",
                "Empty success is valid. Compare client_observation.returned_count with provider_observation.total_count.value; limited/all_reported/unconfirmed describe reported counts, not exhaustive market coverage.",
                "Items retain provider row order and distinguish missing requested fields from explicit nulls. Duplicate or invalid symbol identities and inconsistent counts reject the response.",
                "Data time, delay and session remain unconfirmed. Receipt time is not market-data time; official source does not establish realtime access.",
                "Market syntax, field names and selection strings pass local shape checks, not provider capability validation. Nonblank text is not trimmed; selection uniqueness is exact-string uniqueness.",
                "Presets are provider selection rules, not independently verified technical indicators or trading recommendations. Their interaction with explicit filters is provider-owned.",
                "--types maps to symbol_types and --preset to filter_preset. Omitted type/symbolset lists are not sent. Use explicit symbols/fields according to provider semantics rather than assuming Desktop scanner equivalence."
            ]),
        ),
        "columns" => (
            mcp_data::Kind::Columns.contract(),
            json!({"market": {"choices": mcp_data::COLUMN_MARKETS},
                "group": text_constraint(mcp_data::MAX_TEXT_BYTES),
                "search": text_constraint(mcp_data::MAX_TEXT_BYTES)}),
            json!([{"argument": "group", "argv": ["tv", "mcp", "columns"],
                "result_path": "data.groups[].group"}]),
            json!([
                ["tv", "mcp", "columns"],
                ["tv", "mcp", "columns", "--search", "volume"]
            ]),
            json!([
                "Without group or search, returns groups and column names in overview mode.",
                "With group or search, returns column detail; these options may be combined.",
                "Market alone does not select detail mode. Coverage remains unconfirmed."
            ]),
        ),
        "symbol" | "symbols" => {
            let batch = *name == "symbols";
            let mut constraints = json!({"columns": {
                "max_items": mcp_data::MAX_COLUMNS, "unique": true,
                "item": text_constraint(mcp_data::MAX_COLUMN_BYTES),
                "default_when_empty": mcp_data::DEFAULT_COLUMNS
            }});
            constraints[if batch { "symbols" } else { "symbol" }] = if batch {
                json!({"min_items": 1, "max_items": mcp_data::MAX_SYMBOLS,
                    "unique": true, "item": symbol_constraint()})
            } else {
                symbol_constraint()
            };
            (
                if batch {
                    mcp_data::Kind::Symbols.contract()
                } else {
                    mcp_data::Kind::Symbol.contract()
                },
                constraints,
                json!([symbol_discovery(if batch { "symbols" } else { "symbol" }),
                    {"argument": "columns", "argv": ["tv", "mcp", "columns", "--search", "<field>"],
                        "result_path": "data.columns[].name"}]),
                if batch {
                    json!([[
                        "tv",
                        "mcp",
                        "symbols",
                        "NASDAQ:AAPL",
                        "NASDAQ:MSFT",
                        "--columns",
                        "close,volume"
                    ]])
                } else {
                    json!([[
                        "tv",
                        "mcp",
                        "symbol",
                        "NASDAQ:AAPL",
                        "--columns",
                        "close,volume"
                    ]])
                },
                if batch {
                    json!([
                        "Items retain input order and distinguish returned, missing and unreported.",
                        "No automatic chunking or retry; missing/unreported is not a deletion claim.",
                        "Missing fields and explicit null values remain distinct; neither means zero.",
                        "Data time, delay and session are unconfirmed."
                    ])
                } else {
                    json!([
                        "Provider identity may be unconfirmed; inspect identity_match and provider_observation.symbol.",
                        "missing_fields identifies absent keys; explicit null remains unknown, not zero.",
                        "Data time, delay and session are unconfirmed."
                    ])
                },
            )
        }
        "bars" => (
            mcp_bars::CONTRACT,
            json!({"symbol": symbol_constraint(),
                "timeframe": {"choices": mcp_bars::TIMEFRAMES},
                "count": {"minimum": 1, "maximum": mcp_bars::MAX_COUNT},
                "unsupported": ["from", "to"]}),
            json!([symbol_discovery("symbol")]),
            json!([[
                "tv",
                "mcp",
                "bars",
                "NASDAQ:AAPL",
                "--timeframe",
                "1D",
                "--count",
                "20"
            ]]),
            json!([
                "Recent bounded bars only; date ranges, backward pagination and automatic fallback are unsupported.",
                "1M is the CLI monthly timeframe and maps to provider interval M.",
                "Count satisfaction is separate from calendar coverage.",
                "Adjustment, delay, session and bar finality may be unknown; do not infer parity with tv bars."
            ]),
        ),
        _ => return None,
    };
    let mut result = read_metadata();
    result["output_contract"] = json!(contract);
    result["constraints"] = constraints;
    result["constraints"]["timeout"] = read_timeout();
    result["discovery"] = discovery;
    result["examples"] = examples;
    result["limits"] = interpretation;
    result["limits"].as_array_mut().unwrap().extend([
        json!("Credential refresh can update local authorization state."),
        json!("No implicit source fallback; transport success does not prove complete data."),
    ]);
    Some(result)
}

fn text_constraint(max: usize) -> Value {
    json!({
        "max_bytes": max,
        "encoding": "UTF-8",
        "nonblank": true,
        "control_characters": false
    })
}

pub(super) fn symbol_constraint() -> Value {
    json!({
        "format": "exchange-qualified-symbol",
        "max_bytes": mcp_bars::MAX_SYMBOL_BYTES,
        "pattern": "^[A-Za-z0-9_]+:[A-Za-z0-9_.!\\-]+$"})
}

pub(super) fn symbol_discovery(argument: &str) -> Value {
    json!({
        "argument": argument,
        "argv": ["tv", "mcp", "search", "<query>"],
        "result_path": "data.symbols[].symbol"})
}

fn read_timeout() -> Value {
    json!({
        "minimum": 1,
        "maximum": Operation::MAX_READ_TIMEOUT_SECONDS,
        "default": Operation::DEFAULT_READ_TIMEOUT_SECONDS,
        "unit": "seconds"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn screener_metadata_matches_request_validation() {
        let spec = describe(&["mcp", "screener"]).unwrap();
        let max = spec["constraints"]["limit"]["maximum"].as_u64().unwrap() as u32;
        for (limit, valid) in [(0, false), (max, true), (max + 1, false)] {
            assert_eq!(
                mcp_data::Request::screener(mcp_data::ScreenerOptions {
                    limit,
                    ..Default::default()
                })
                .is_ok(),
                valid
            );
        }
        for preset in spec["constraints"]["filter_preset"]["choices"]
            .as_array()
            .unwrap()
        {
            assert!(
                mcp_data::Request::screener(mcp_data::ScreenerOptions {
                    filter_preset: Some(preset.as_str().unwrap().into()),
                    ..Default::default()
                })
                .is_ok()
            );
        }
        for field in spec["constraints"]["filters"]["string_fields"]
            .as_array()
            .unwrap()
        {
            assert!(
                mcp_data::Request::screener(mcp_data::ScreenerOptions {
                    filters: json!({field.as_str().unwrap(): "Example"}),
                    ..Default::default()
                })
                .is_ok()
            );
        }
        assert!(
            mcp_data::Request::screener(mcp_data::ScreenerOptions {
                filters: json!({"close": [20, 10]}),
                ..Default::default()
            })
            .is_err()
        );
    }

    #[test]
    fn examples_parse_and_constraints_match_real_request_boundaries() {
        for name in ["search", "columns", "symbol", "symbols", "bars", "screener"] {
            let spec = describe(&["mcp", name]).unwrap();
            for example in spec["examples"].as_array().unwrap() {
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
        let search = describe(&["mcp", "search"]).unwrap();
        let max = search["constraints"]["query"]["max_bytes"]
            .as_u64()
            .unwrap() as usize;
        assert!(mcp_data::Request::search(&"x".repeat(max), None).is_ok());
        assert!(mcp_data::Request::search(&"x".repeat(max + 1), None).is_err());
        assert!(mcp_data::Request::search(&"é".repeat(max / 2 + 1), None).is_err());
        for kind in search["constraints"]["type"]["choices"].as_array().unwrap() {
            assert!(mcp_data::Request::search("Apple", kind.as_str()).is_ok());
        }
        let columns = describe(&["mcp", "columns"]).unwrap();
        for market in columns["constraints"]["market"]["choices"]
            .as_array()
            .unwrap()
        {
            assert!(mcp_data::Request::columns(market.as_str(), None, None).is_ok());
        }
        let bars = describe(&["mcp", "bars"]).unwrap();
        let max = bars["constraints"]["count"]["maximum"].as_u64().unwrap() as u32;
        for interval in bars["constraints"]["timeframe"]["choices"]
            .as_array()
            .unwrap()
        {
            assert!(
                mcp_bars::Request::new("NASDAQ:EXAMPLE", interval.as_str().unwrap(), max).is_ok()
            );
        }
        assert!(mcp_bars::Request::new("NASDAQ:EXAMPLE", "1D", max + 1).is_err());
        assert!(mcp_bars::Request::new("NASDAQ:EXAMPLE", "1D", 0).is_err());
    }

    #[test]
    fn column_defaults_and_batch_bounds_match_execution() {
        let spec = describe(&["mcp", "symbols"]).unwrap();
        let constraints = &spec["constraints"];
        let max = constraints["symbols"]["max_items"].as_u64().unwrap() as usize;
        let mut symbols = (0..max)
            .map(|n| format!("NASDAQ:EXAMPLE{n}"))
            .collect::<Vec<_>>();
        let request = mcp_data::Request::symbols(&symbols, &[]).unwrap();
        assert_eq!(
            request.arguments()["columns"],
            constraints["columns"]["default_when_empty"]
        );
        symbols.push("NASDAQ:EXTRA".into());
        assert!(mcp_data::Request::symbols(&symbols, &[]).is_err());
        assert!(mcp_data::Request::symbols(&["NASDAQ:X".into(), "NASDAQ:X".into()], &[]).is_err());
        let max = constraints["columns"]["max_items"].as_u64().unwrap() as usize;
        let mut columns = (0..max).map(|n| format!("field{n}")).collect::<Vec<_>>();
        assert!(mcp_data::Request::symbol("NASDAQ:X", &columns).is_ok());
        columns.push("extra".into());
        assert!(mcp_data::Request::symbol("NASDAQ:X", &columns).is_err());
        assert!(mcp_data::Request::symbol("NASDAQ:X", &["close".into(), "close".into()]).is_err());
    }
}
