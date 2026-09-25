//! Argument-dependent Desktop effects. Spec lookup itself remains offline.

use super::super::dispatch::MAX_CHART_COMPARE_SYMBOLS;
use crate::ops::{CHART_COMPARE_CONTRACT_VERSION, CHART_TYPES};
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
        ["status"] => {
            result["source"] = json!("desktop_status");
            result["examples"] = json!([
                ["tv", "status"],
                ["tv", "--target-id", "<target_id>", "status"]
            ]);
            result["readback"] = json!({"argv": ["tv", "--target-id", "<target_id>", "readiness"], "meaning": "Check chart API and recent-bar readiness after resolving the intended target."});
            result["limits"].as_array_mut().unwrap().extend([
                json!("Fetches CDP targets and, when selected, connects to query basic chart metadata. It does not launch Desktop, activate a tab, read bars or test official MCP authorization."),
                json!("Without an explicit target, no chart candidate or ambiguous candidates returns outer success with connected/cdp_connected false and data.error. The target list was reachable in these cases: false does not by itself prove CDP is offline. Inspect desktop_readiness.target_selection and candidates."),
                json!("Target-list, explicit-target selection or WebSocket connection failures are errors. After connection, chart evaluation errors can be suppressed, leaving api_available false and unknown/null chart fields; a JavaScript chart exception can appear as api_error."),
                json!("connected true and api_available true do not establish readable bars, freshness, entitlement or a future command's success. The desktop_readiness object is a target summary, not the dedicated readiness command's ready result."),
                json!("Reuse returned target_cli_args for the intended target. next_action_hint does not authorize launching, switching or mutating anything; keep target URLs/titles private.")
            ]);
        }
        ["ui-state"] => {
            result["source"] = Value::Null;
            result["examples"] = json!([["tv", "--target-id", "<target_id>", "ui-state"]]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Observes current DOM plus chart/Replay API state without opening panels, clicking controls or changing Replay. No versioned success contract or source field is supplied by this operation."),
                json!("Panel open flags are heuristics: bottom height/right width/widgetbar width over 50 pixels, Pine editor element presence, and Strategy Tester offsetParent presence. Hidden or changed DOM can be misclassified; dimensions are not operation readiness."),
                json!("Button discovery skips hidden, narrow and long-label entries; deduplication strips non-ASCII characters and truncates keys, so localized or repeated labels can collapse. Output labels are truncated and region names use fixed coordinate thresholds, not semantic panel membership."),
                json!("Returned x/y are rounded viewport top-left coordinates, not verified click centers or stable selectors. disabled reflects the element property only. key_buttons uses mostly English text patterns, and later matches can overwrite earlier ones; presence/absence is not proof that an action is supported or safe."),
                json!("Chart and Replay exceptions are embedded in their sections while the overall read can succeed. Replay available/started observations do not change playback. No recent-bar check, cross-section atomicity or permission to act follows from this snapshot.")
            ]);
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
            result["source"] = json!("desktop_target_list");
            result["discovery"] = json!([]);
            result["examples"] = json!([["tv", "tab", "list"]]);
            result["limits"] = json!([
                "Listing does not activate a target. Use the selected row's ID for --target-id; an index used by tab switch is a different argument."
            ]);
        }
        ["launch"] => {
            result["source"] = json!("desktop_launcher");
            result["requires"] = json!({"desktop_installed": null, "desktop_running": false, "authentication": null});
            result["effects"] = json!({"may_start_process": true, "may_terminate_process": null});
            result["discovery"] = json!([]);
            result["constraints"] = json!({
                "port": {"minimum": 1, "maximum": u16::MAX, "default_source": "CDP transport configuration"},
                "path": {"nonempty": true, "existing_file": true}
            });
            result["variants"] = json!([
                {"when": {"runtime": "CDP endpoint already responds"}, "operation": "reuse", "effects": {"starts_process": false, "terminates_process": false}},
                {"when": {"runtime": "No reusable CDP response; startup proceeds", "flag_true": ["kill_existing"]}, "operation": "terminate_then_launch"},
                {"when": {"runtime": "No reusable CDP response; startup proceeds", "flag_false": ["kill_existing"]}, "operation": "launch_without_termination"}
            ]);
            result["readback"] = json!({"argv": ["tv", "readiness"]});
            result["examples"] = json!([["tv", "launch"], ["tv", "launch", "--port", "9222"]]);
            result["limits"] = json!([
                "kill-existing may end the current Desktop session; require the user's authority for that effect.",
                "A responding CDP endpoint is reused even when kill-existing was supplied.",
                "Inspect CDP readiness and warnings; starting a process does not prove chart readiness.",
                "Starting a new process requires an installed app. Explicit path selects a local executable; normal macOS launch uses the system app launcher."
            ]);
        }
        ["tab", action @ ("switch" | "new" | "close")] => {
            result["source"] = json!("desktop_tab_operation");
            result["effects"] = json!({"desktop_mutation": true,
                "activates_source_tab": *action == "new", "activates_target": *action == "switch",
                "creates_tab": *action == "new", "closes_tab": *action == "close"});
            let argument = if *action == "new" { "from" } else { "index" };
            let result_path = if *action == "close" {
                "data.app_tabs[].index"
            } else {
                "data.tabs[].index"
            };
            result["discovery"] = json!([{"argument": argument,
                "argv": ["tv", "tab", "list"], "result_path": result_path}]);
            result["constraints"] = json!({argument: {"minimum": 0, "maximum": null, "runtime_bound": "current selected list length"}});
            result["readback"] = json!({"argv": ["tv", "tab", "list"]});
            result["examples"] = if *action == "new" {
                json!([["tv", "tab", "new", "--from", "0"]])
            } else {
                json!([["tv", "tab", action, "0"]])
            };
            result["limits"] = json!([
                "Indices are transient: refresh tab list and select the intended row before changing tabs.",
                "switch/new use chart-tab indices; close uses app-tab indices, which may differ.",
                "new without from is allowed only when exactly one chart tab exists; it activates the source before creating a tab.",
                "close refuses to close the last app tab. new/close also require an app-window target and usable UI.",
                "Inspect readback before retrying after an error; an error does not guarantee no UI change. --target-id does not replace an index."
            ]);
        }
        ["chart", "compare"] => {
            result["output_contract"] = json!(CHART_COMPARE_CONTRACT_VERSION);
            result["effects"] = json!({"chart_mutation": true, "temporary_symbol_changes": true, "restoration_guaranteed": false});
            result["constraints"] = json!({"symbols": {"min_items": 2, "max_items": MAX_CHART_COMPARE_SYMBOLS,
                "item": {"nonblank": true, "trimmed": true}, "unique_required": false}});
            result["readback"] = json!({"argv": ["tv", "--target-id", "<target_id>", "symbol"]});
            result["examples"] = json!([[
                "tv",
                "--target-id",
                "<target_id>",
                "chart",
                "compare",
                "NASDAQ:EXAMPLE",
                "NYSE:OTHER"
            ]]);
            result["limits"] = json!([
                "Requests each symbol on the selected chart and attempts to restore the original symbol after each read.",
                "Stops on an item error. Inspect per-item status/restored, summary and final chart context; a success envelope can contain partial results.",
                "Restoration can fail or remain unknown. Inspect the target before another mutation; do not treat comparison as a non-mutating read.",
                "No scanner, quote-data, Replay or historical-bar fallback is used. Use tv compare for Desktop-free scanner comparison."
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
            vec!["status"],
            vec!["ui-state"],
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
    fn lifecycle_examples_parse_and_comparison_limits_match_dispatch() {
        use super::super::super::dispatch::validate_chart_compare_symbols;
        for path in [
            vec!["launch"],
            vec!["tab", "switch"],
            vec!["tab", "new"],
            vec!["tab", "close"],
            vec!["chart", "compare"],
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
        let spec = describe(&["chart", "compare"]).unwrap();
        let max = spec["constraints"]["symbols"]["max_items"]
            .as_u64()
            .unwrap() as usize;
        assert!(validate_chart_compare_symbols(&vec!["NASDAQ:X".into(); max]).is_ok());
        assert!(validate_chart_compare_symbols(&vec!["NASDAQ:X".into(); max + 1]).is_err());
        assert!(validate_chart_compare_symbols(&["NASDAQ:X".into()]).is_err());
        assert!(validate_chart_compare_symbols(&["NASDAQ:X".into(), " ".into()]).is_err());
        assert_eq!(
            describe(&["tab", "close"]).unwrap()["discovery"][0]["result_path"],
            "data.app_tabs[].index"
        );
        assert_eq!(
            describe(&["tab", "switch"]).unwrap()["discovery"][0]["result_path"],
            "data.tabs[].index"
        );
        assert_eq!(spec["effects"]["restoration_guaranteed"], false);
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
