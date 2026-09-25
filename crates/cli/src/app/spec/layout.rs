//! Pane and saved-layout metadata, without connecting to Desktop.

use serde_json::{Value, json};

use crate::ops::supported_pane_layouts;

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [family @ ("pane" | "layout"), action] = path else {
        return None;
    };
    let mut result = json!({
        "source": "internal_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": *action != "list"},
        "output_contract": null,
        "discovery": [{
            "argument": "target-id",
            "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"
        }],
        "constraints": {},
        "limits": [
            "Select an explicit Desktop target. Pane indices and saved-layout IDs are not tab IDs.",
            "Desktop session and account access are runtime conditions; no MCP OAuth or provider fallback is used.",
            "After a mutation error, inspect current state before retrying. Restoration is not guaranteed."
        ]
    });
    let example = match (*family, *action) {
        ("pane", "list") => {
            result["limits"].as_array_mut().unwrap().push(json!(
                "active_index can be unknown and individual panes can contain read errors; missing values are not defaults."
            ));
            json!(["tv", "pane", "list"])
        }
        ("pane", "layout") => {
            result["constraints"]["layout"] = json!({
                "supported": supported_pane_layouts(),
                "normalization": "remove ASCII whitespace and lowercase",
                "aliases": {
                    "single": "s",
                    "1": "s",
                    "1x1": "s",
                    "2x1": "2h",
                    "1x2": "2v",
                    "2x2": "4",
                    "grid": "4",
                    "quad": "4",
                    "3x1": "3h",
                    "1x3": "3v"
                }
            });
            result["limits"].as_array_mut().unwrap().push(json!(
                "Changes the pane arrangement, not the saved-layout selection. Compare observed_layout with the requested layout; success does not assert equality."
            ));
            json!(["tv", "pane", "layout", "2x2"])
        }
        ("pane", "focus" | "symbol") => {
            result["constraints"]["index"] = json!({
                "minimum": 0,
                "runtime_bound": "less than the current pane count"
            });
            result["discovery"].as_array_mut().unwrap().push(json!({
                "argument": "index",
                "argv": ["tv", "--target-id", "<target_id>", "pane", "list"],
                "result_path": "data.panes[].index"
            }));
            result["effects"]["changes_active_pane"] = json!(true);
            result["limits"].as_array_mut().unwrap().push(json!(
                "Focus attempts a pane click. focused_index reports the requested index, not verified active-pane identity; inspect pane list afterward."
            ));
            if *action == "symbol" {
                result["constraints"]["symbol"] = json!({"nonblank": true});
                result["limits"].as_array_mut().unwrap().push(json!(
                    "Focus occurs before setting the active chart symbol. Returned symbol/requested_symbol echo the request; read pane list to confirm the intended pane and symbol. The previous focus is not restored."
                ));
                json!(["tv", "pane", "symbol", "0", "NASDAQ:EXAMPLE"])
            } else {
                json!(["tv", "pane", "focus", "0"])
            }
        }
        ("layout", "list") => {
            result["limits"].as_array_mut().unwrap().push(json!(
                "Inspect data.error even when the envelope succeeds. An empty normalized inventory alone does not prove the account has no saved layouts."
            ));
            json!(["tv", "layout", "list"])
        }
        ("layout", "switch") => {
            result["effects"]["chart_mutation"] = Value::Null;
            result["constraints"]["target"] = json!({
                "nonblank": true,
                "normalization": "join positional words with spaces, then trim",
                "matching": "exact ID first, otherwise case-insensitive exact name",
                "unique_match_required": true
            });
            result["discovery"].as_array_mut().unwrap().push(json!({
                "argument": "target",
                "argv": ["tv", "--target-id", "<target_id>", "layout", "list"],
                "result_path": "data.layouts[].id"
            }));
            result["variants"] = json!([
                {
                    "when": {"flag_true": ["dry_run"]},
                    "operation": "resolve_saved_layout",
                    "effects": {"chart_mutation": false}
                },
                {
                    "when": {"flag_false": ["dry_run"]},
                    "operation": "load_saved_layout",
                    "effects": {"chart_mutation": true, "may_navigate": true}
                }
            ]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("dry-run resolves against the live saved-layout list and requires Desktop; it does not load a layout or reserve a future match."),
                json!("switched records a load request or scheduled navigation, not confirmed completion. Inspect navigation_expected and unsaved_dialog_observed; the command does not dismiss unsaved-change dialogs."),
                json!("After navigation, refresh tab list, confirm the intended target and inspect state/pane list before continuing. Saved layout list is not active-layout readback.")
            ]);
            json!(["tv", "layout", "switch", "Example Layout", "--dry-run"])
        }
        _ => return None,
    };
    if *family == "pane" {
        result["readback"] = json!({
            "argv": ["tv", "--target-id", "<target_id>", "pane", "list"]
        });
    }
    result["examples"] = json!([example]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, ops::validate_pane_layout};
    use clap::Parser;

    #[test]
    fn examples_and_layout_choices_match_execution_parser() {
        for path in [
            ["pane", "list"],
            ["pane", "layout"],
            ["pane", "focus"],
            ["pane", "symbol"],
            ["layout", "list"],
            ["layout", "switch"],
        ] {
            let detail = describe(&path).unwrap();
            let argv = detail["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }

        let detail = describe(&["pane", "layout"]).unwrap();
        let layout = &detail["constraints"]["layout"];
        for choice in layout["supported"].as_array().unwrap() {
            assert!(validate_pane_layout(choice["layout"].as_str().unwrap()).is_ok());
        }
        for alias in layout["aliases"].as_object().unwrap().keys() {
            assert!(validate_pane_layout(alias).is_ok());
        }
        assert!(validate_pane_layout("unsupported").is_err());
    }
}
