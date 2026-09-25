//! Quote routing and scanner field discovery, without provider access.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let mut result = json!({
        "source": "scanner_scan_rest",
        "requires": {"desktop": false, "authentication": false},
        "effects": {"chart_mutation": false, "account_mutation": false},
        "output_contract": null,
        "constraints": {},
        "limits": [
            "Inspect time, update_mode, delay_seconds and missing values; a successful quote does not guarantee realtime data.",
            "Scanner, chart main-series and Desktop quote-data are distinct sources, not interchangeable session prices. No official MCP routing is implied."
        ]
    });
    match path {
        ["quote"] => {
            result["source"] = Value::Null;
            result["requires"] = json!({"desktop": null, "authentication": null});
            result["effects"]["chart_mutation"] = Value::Null;
            result["constraints"]["symbol"] = json!({"nonblank_when_present": true});
            result["variants"] = json!([
                {
                    "when": {"absent": ["symbol"], "source": [null, "chart", "auto"]},
                    "source": "chart_api",
                    "requires": {"desktop": true},
                    "effects": {"chart_mutation": false}
                },
                {
                    "when": {"absent": ["symbol"], "source": ["scanner", "quote-data"]},
                    "operation": "validation_error"
                },
                {
                    "when": {"present": ["symbol"], "source": [null, "scanner"]},
                    "source": "scanner_scan_rest",
                    "requires": {"desktop": false, "authentication": false},
                    "effects": {"chart_mutation": false}
                },
                {
                    "when": {"present": ["symbol"], "source": ["chart"]},
                    "source": "chart_api",
                    "requires": {"desktop": true},
                    "effects": {"chart_mutation": true, "may_switch_symbol": true, "restoration_guaranteed": false}
                },
                {
                    "when": {"present": ["symbol"], "source": ["quote-data"]},
                    "source": "desktop_quote_data_ws",
                    "requires": {"desktop": true},
                    "effects": {"chart_mutation": false, "observes_network_events": true},
                    "output_contract": "quote_data.v1"
                },
                {
                    "when": {"present": ["symbol"], "source": ["auto"]},
                    "operation": "try_desktop_connection_then_chart_read",
                    "fallback": "scanner only if Desktop connection fails; never after chart read begins",
                    "effects": {"chart_mutation": null}
                }
            ]);
            result["discovery"] = json!([{
                "argument": "target-id",
                "argv": ["tv", "tab", "list"],
                "result_path": "data.tabs[].id",
                "applies_to": "Desktop routes"
            }]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Omitting symbol with auto still requires Desktop and does not use scanner fallback. Explicit scanner or quote-data requires a symbol."),
                json!("Chart symbol comparison/restoration uses bare symbols, so matching tickers across exchanges are not strict exchange-identity proof. Inspect observed symbol, switch_performed, restored and freshness_check; chart-source errors do not guarantee unchanged state."),
                json!("Chart quotes read the main-series last bar and session_boundary, not scanner extended_hours. Read the selected target's symbol after a requested chart-symbol read if restoration matters."),
                json!("quote-data enables CDP network observation and waits for matching quote-data events; it does not change the chart symbol or guarantee a matching event. Inspect rtc versus regular lp evidence; auto never chooses this source.")
            ]);
            result["examples"] = json!([
                ["tv", "quote", "NASDAQ:EXAMPLE"],
                ["tv", "--target-id", "<target_id>", "quote"],
                [
                    "tv",
                    "--target-id",
                    "<target_id>",
                    "quote",
                    "NASDAQ:EXAMPLE",
                    "--source",
                    "chart"
                ]
            ]);
        }
        ["quotes"] => {
            result["constraints"]["symbols"] = json!({
                "min_items": 1,
                "max_items": 25,
                "item": {"nonblank": true, "trimmed": true},
                "deduplicated": false
            });
            result["limits"].as_array_mut().unwrap().extend([
                json!("Preserves input order and requested_index, including duplicate requests. Each item has ok plus quote or error; one successful item is enough for envelope success."),
                json!("When every item fails, the error details retain the ordered batch. Inspect resolved_count/error_count and each item; no automatic chart or MCP fallback occurs."),
                json!("Scanner extended_hours is additive and can contain unknown fields. Do not replace missing extended-hours prices with the regular-session quote.")
            ]);
            result["examples"] = json!([["tv", "quotes", "NASDAQ:EXAMPLE", "NYSE:OTHER"]]);
        }
        ["scanner", "metainfo"] => {
            result["source"] = json!("scanner_metainfo_rest");
            result["constraints"] = json!({
                "market": {
                    "choices": ["america"],
                    "default": "america",
                    "trimmed": true,
                    "case_sensitive": true
                },
                "fields": {
                    "item_nonblank": true,
                    "trimmed": true,
                    "deduplicated": true,
                    "empty": "request all available fields"
                }
            });
            result["limits"] = json!([
                "Reads provider field metadata, not market prices or account state; Desktop and MCP credentials are not used.",
                "Repeat --field to request exact field names. Inspect missing_fields and optional kind/title metadata; envelope success does not mean every requested field was returned.",
                "Field metadata is runtime provider information, not proof of complete scan results or that an arbitrary field is supported by every CLI scan mode."
            ]);
            result["examples"] = json!([[
                "tv", "scanner", "metainfo", "--field", "close", "--field", "volume"
            ]]);
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
    fn quote_and_metadata_examples_parse() {
        for path in [vec!["quote"], vec!["quotes"], vec!["scanner", "metainfo"]] {
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
    }
}
