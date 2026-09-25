//! Saved column edits with positional selection and storage-only readback.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [
        "screener",
        "columns",
        action @ ("add" | "remove" | "reorder"),
    ] = path
    else {
        return None;
    };
    let mut result = json!({
        "source": "ui_screener_dialog", "scope": "screen_storage_api", "output_contract": null,
        "requires": {"desktop": true, "authentication": null, "screener_open": true},
        "effects": {"account_mutation": null, "provider_request": true,
            "ui_mutation": false, "local_file_write": false},
        "constraints": {"dry_run": {"default": false},
            "active_screen": {"non_dry_run_required_substring": ["CLI-Test", "テスト"], "case_sensitive": true}},
        "variants": [
            {"when": {"present": ["dry-run"]}, "effects": {"account_mutation": false},
                "operation": "read storage and calculate proposed column sequence without saving"},
            {"when": {"absent": ["dry-run"]}, "effects": {"account_mutation": true},
                "operation": "save custom column set, then read storage back"}
        ],
        "discovery": [
            {"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.screener_targets[].target_cli_args"},
            {"argument": "column identity and order", "argv": ["tv", "screener", "columns", "config"], "result_path": "data.columns[]"}
        ],
        "limits": [
            "Requires an already open Screener, active title and usable storage init data. Does not open the panel or force a UI refresh. Storage access uses the Desktop session, not official MCP OAuth.",
            "Dry-run performs live storage reads and returns proposed after_columns; it is not offline and does not validate provider acceptance of a future write. Execution is code-restricted to test screen names containing CLI-Test or テスト.",
            "Writes a fetched screen document with default_custom_column_set replaced and active_column_set set to custom. This is a saved account-setting change, not merely a visible-column toggle; intervening edits are not reconciled by this adapter.",
            "Post-check compares column count and ordered id/params pairs. It does not verify displayed rows, names, refreshed UI or every screen field. Returned columns is the computed sequence; names come from earlier positional mapping and are not fresh ID-based verification.",
            "Storage title must match the active title. Missing storage fields can fall back to loaded init data. A write may succeed before readback fails; errors do not imply rollback and do not justify blindly repeating an add or other edit.",
            "Official MCP screening does not edit saved Desktop column sets. Keep account IDs/configuration private and verify the selected screen before a requested write."
        ]
    });
    let example = match *action {
        "add" => {
            result["constraints"]["id"] = json!({"trimmed": true, "nonblank": true, "meaning": "storage ID, not display name"});
            result["constraints"]["params_json"] = json!({"type": "JSON object", "default": {}});
            result["constraints"]["after_index"] = json!({"optional": true, "minimum": 0, "bound": "must address an existing saved column", "omitted": "append"});
            result["limits"].as_array_mut().unwrap().push(json!(
                "Inserts after the requested zero-based saved index, or appends. An empty set can only use append. No local column-ID/parameter catalog validation or duplicate ID/params rejection is performed; successful preview does not prove the column is supported."
            ));
            json!([
                "tv",
                "--target-id",
                "<target_id>",
                "screener",
                "columns",
                "add",
                "--id",
                "<storage_column_id>",
                "--params-json",
                "{}",
                "--dry-run"
            ])
        }
        "remove" => {
            result["constraints"]["selector"] = json!({"exactly_one": ["index", "name"], "index": "zero-based visible column position", "name": "trimmed nonblank case-insensitive substring; must match exactly one visible column", "blank_name": "treated as omitted"});
            result["limits"].as_array_mut().unwrap().push(json!(
                "Resolves the visible column then uses the same index in storage; duplicate/missing name matches or an absent saved index fail. Positional correspondence does not prove the displayed name belongs to that storage ID. No local guard prevents removing the last saved column."
            ));
            json!([
                "tv",
                "--target-id",
                "<target_id>",
                "screener",
                "columns",
                "remove",
                "--index",
                "1",
                "--dry-run"
            ])
        }
        _ => {
            result["constraints"]["indexes"] = json!({"fields": ["from-index", "to-index"], "minimum": 0, "bounds": "both must address existing saved columns", "distinct": true});
            result["limits"].as_array_mut().unwrap().push(json!(
                "Moves the source column to the final destination index after removing it; this is not an insert-after index. Equal indexes fail even in preview; the sequence is reindexed afterward."
            ));
            json!([
                "tv",
                "--target-id",
                "<target_id>",
                "screener",
                "columns",
                "reorder",
                "--from-index",
                "1",
                "--to-index",
                "0",
                "--dry-run"
            ])
        }
    };
    result["examples"] = json!([example]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::screener::{
        columns::ensure_test_screener_screen_for_column_mutation,
        validation::{
            validate_screener_column_add_request, validate_screener_column_reorder_request,
            validate_screener_column_selector,
        },
    };

    #[test]
    fn column_edit_examples_and_constraints_match_validation() {
        for action in ["add", "remove", "reorder"] {
            let spec = describe(&["screener", "columns", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
            assert!(
                ensure_test_screener_screen_for_column_mutation("CLI-Test-Example", action).is_ok()
            );
            assert!(ensure_test_screener_screen_for_column_mutation("ordinary", action).is_err());
        }
        assert!(validate_screener_column_selector(Some(0), Some("Price")).is_err());
        assert!(validate_screener_column_selector(Some(0), Some(" ")).is_ok());
        assert!(validate_screener_column_selector(None, Some(" ")).is_err());
        assert!(validate_screener_column_add_request("id", Some("[]"), None, true).is_err());
        assert!(validate_screener_column_add_request("id", Some("{}"), None, true).is_ok());
        assert!(validate_screener_column_reorder_request(1, 1).is_err());
    }
}
