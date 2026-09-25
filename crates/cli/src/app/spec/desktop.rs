//! Argument-dependent Desktop effects. Spec lookup itself remains offline.

use crate::ops::CHART_TYPES;
use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let mut result = json!({
        "source": "chart_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": false},
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"}],
        "limits": ["Use an explicit target when multiple charts are open; never silently select another target.",
            "No official MCP or legacy market-data fallback is implied.",
            "No MCP OAuth is required. Desktop session and data access depend on the running application."],
        "output_contract": null
    });
    match path {
        ["symbol"] | ["timeframe"] | ["type"] => {
            let argument = match path[0] {
                "symbol" => "symbol",
                "timeframe" => "timeframe",
                _ => "chart_type",
            };
            result["effects"]["chart_mutation"] = Value::Null;
            result["variants"] = json!([
                {"when": {"absent": [argument]}, "effects": {"chart_mutation": false}, "operation": "read"},
                {"when": {"present": [argument]}, "effects": {"chart_mutation": true}, "operation": "set"}
            ]);
            result["constraints"] = json!({argument: {"nonblank_when_present": true}});
            if path[0] == "type" {
                result["constraints"][argument]["names"] = json!(CHART_TYPES);
                result["constraints"][argument]["numeric_codes"] =
                    json!({"minimum": 0, "maximum": CHART_TYPES.len() - 1});
                result["constraints"][argument]["name_matching"] = json!(
                    "case-insensitive; only ASCII letters and digits participate in name matching"
                );
            }
            result["examples"] = json!([
                ["tv", path[0]],
                [
                    "tv",
                    "--target-id",
                    "<target_id>",
                    path[0],
                    match path[0] {
                        "symbol" => "NASDAQ:EXAMPLE",
                        "timeframe" => "D",
                        _ => "Candles",
                    }
                ]
            ]);
            result["readback"] = json!({"argv": ["tv", "--target-id", "<target_id>", path[0]],
                "meaning": "Read the selected chart again if current state must be confirmed; a set request is not permission to change another target."});
        }
        ["range"] => {
            result["effects"]["chart_mutation"] = Value::Null;
            result["variants"] = json!([
                {"when": {"absent": ["from", "to"]}, "operation": "read", "effects": {"chart_mutation": false}},
                {"when": {"present": ["from", "to"]}, "operation": "set", "effects": {"chart_mutation": true, "may_request_older_history": true}},
                {"when": {"exactly_one_present": ["from", "to"]}, "operation": "validation_error", "effects": {"chart_mutation": false}}
            ]);
            result["constraints"] = json!({"from": {"finite": true, "unit": "Unix seconds"},
                "to": {"finite": true, "unit": "Unix seconds"}, "required_together": ["from", "to"], "ordering": "from < to"});
            result["examples"] = json!([
                ["tv", "range"],
                [
                    "tv",
                    "--target-id",
                    "<target_id>",
                    "range",
                    "--from",
                    "1704067200",
                    "--to",
                    "1704153600"
                ]
            ]);
            result["limits"].as_array_mut().unwrap().push(json!("A set can request older chart history and change the viewport only when matching loaded bars exist; inspect coverage, clamp and stop diagnostics."));
        }
        ["info"] => {
            result["source"] = Value::Null;
            result["requires"]["desktop"] = Value::Null;
            result["requires"]["authentication"] = Value::Null;
            result["variants"] = json!([
                {"when": {"absent": ["symbol"]}, "source": "chart_api", "requires": {"desktop": true, "authentication": null}},
                {"when": {"present": ["symbol"]}, "source": "symbol_search_rest", "requires": {"desktop": false, "authentication": false}}
            ]);
            result["constraints"] = json!({"symbol": {"nonblank_when_present": true}});
            result["examples"] = json!([["tv", "info"], ["tv", "info", "NASDAQ:EXAMPLE"]]);
            result["limits"].as_array_mut().unwrap().push(json!("An explicit symbol uses the credential-free HTTP endpoint and does not switch the chart. This is not official MCP."));
        }
        ["state"] => {
            result["examples"] = json!([["tv", "--target-id", "<target_id>", "state"]]);
            result["limits"].as_array_mut().unwrap().push(json!("Chart API readiness does not prove that bar values can be read; inspect chart_readiness."));
        }
        ["readiness"] => {
            result["source"] = json!("desktop_readiness");
            result["examples"] = json!([["tv", "readiness"]]);
            result["limits"].as_array_mut().unwrap().push(json!("A successful envelope may contain ready=false. This checks readiness without switching symbols, activating tabs or capturing screenshots."));
        }
        ["tab", "list"] => {
            result["source"] = json!("cdp_targets");
            result["discovery"] = json!([]);
            result["examples"] = json!([["tv", "tab", "list"]]);
            result["limits"] = json!([
                "Listing does not activate a target. Use the selected row's ID for --target-id; an index used by tab switch is a different argument."
            ]);
        }
        _ => return None,
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, ops};
    use clap::Parser;

    #[test]
    fn examples_parse_and_chart_types_share_execution_validation() {
        for path in [
            vec!["symbol"],
            vec!["timeframe"],
            vec!["type"],
            vec!["range"],
            vec!["info"],
            vec!["state"],
            vec!["readiness"],
            vec!["tab", "list"],
        ] {
            for example in describe(&path).unwrap()["examples"].as_array().unwrap() {
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
        let types = describe(&["type"]).unwrap();
        for value in types["constraints"]["chart_type"]["names"]
            .as_array()
            .unwrap()
        {
            assert!(ops::validate_chart_type(value.as_str().unwrap()).is_ok());
        }
        assert!(ops::validate_chart_type(&CHART_TYPES.len().to_string()).is_err());
        assert!(ops::validate_visible_range_request(1.0, 2.0).is_ok());
        assert!(ops::validate_visible_range_request(2.0, 1.0).is_err());
        assert!(ops::validate_visible_range_request(f64::NAN, 2.0).is_err());
    }

    #[test]
    fn conditional_paths_never_claim_unconditional_read_or_desktop_access() {
        for name in ["symbol", "timeframe", "type", "range"] {
            let spec = describe(&[name]).unwrap();
            assert!(spec["effects"]["chart_mutation"].is_null());
            assert_eq!(spec["variants"][0]["effects"]["chart_mutation"], false);
            assert_eq!(spec["variants"][1]["effects"]["chart_mutation"], true);
        }
        let info = describe(&["info"]).unwrap();
        assert!(info["requires"]["desktop"].is_null());
        assert_eq!(info["variants"][1]["requires"]["desktop"], false);
        assert!(describe(&["ui", "eval"]).is_none());
    }
}
