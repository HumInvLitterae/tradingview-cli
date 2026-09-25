//! Visible Screener context and legacy watchlist reads.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if let Some(result) = discovery(path) {
        return Some(result);
    }
    let status = path == ["screener", "status"];
    let get = path == ["screener", "get"];
    let watchlist = path == ["watchlist", "get"];
    if !status
        && !get
        && !watchlist
        && !matches!(
            path,
            ["screener", "screens", "active"]
                | ["screener", "filters", "list"]
                | ["screener", "columns", "list"]
        )
    {
        return None;
    }
    let mut result = json!({
        "source": if watchlist { Value::Null } else { json!("ui_screener_dialog") },
        "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {
            "account_mutation": false, "local_file_write": false,
            "ui_mutation": if status || watchlist { json!(false) } else { Value::Null }
        },
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"],
            "result_path": if watchlist { "data.tabs[].id" } else { "data.screener_targets[].target_cli_args" }}],
        "limits": [],
        "examples": [["tv", "--target-id", "<target_id>"]]
    });
    let example = result["examples"][0].as_array_mut().unwrap();
    example.extend(path.iter().map(|part| json!(part)));
    if watchlist {
        result["limits"] = json!([
            "Reads the selected Desktop right-panel DOM, not an account list by ID. source is the extraction outcome: panel_closed, no_container, data_attributes, text_scan or empty.",
            "Closed/missing panels return an empty successful result; that does not establish an empty account watchlist. Rows are deduplicated by extracted symbol and can be limited by rendered DOM content.",
            "data_attributes projects heuristic numeric cell strings as last/change/change_percent. text_scan can return unqualified ticker-like text with null prices. Neither validates quote freshness, exchange identity or full list coverage.",
            "For account-list contents prefer explicit tv mcp watchlist list followed by get <ID> when available and authorized. MCP preserves list sections and order but does not reproduce visible quote cells or prove which Desktop list is selected. No automatic source fallback is performed."
        ]);
        return Some(result);
    }
    result["limits"] = json!([
        "Reads the current Desktop Screener UI, including localized display text, not scanner REST or official MCP data. Use the intended Screener target's target_cli_args from tab list; do not silently open or select a different target.",
        "For data-only screening, prefer tv mcp screener when its filters meet the request. It does not operate saved Desktop screens or duplicate their filter/column state; do not silently substitute sources.",
        "Rendered rows and visible labels are not complete provider results, storage IDs or validated numeric fields. Empty or missing UI content does not establish no matching symbols."
    ]);
    if status {
        result["limits"].as_array_mut().unwrap().push(json!(
            "Status does not open the panel. It reports open/button_found, titles and counts only, not row data. Missing counts default to zero; closed or unrecognized UI is not an empty-screen result."
        ));
    } else {
        result["variants"] = json!([
            {"when": "panel initially open", "effects": {"ui_mutation": false}},
            {"when": "panel initially closed", "effects": {"ui_mutation": true},
                "operation": "temporarily open for read, then attempt close"}
        ]);
        result["limits"].as_array_mut().unwrap().extend([
            json!("May open a closed Screener and send Escape to close it after reading. opened_for_read records that opening; restored_open_state is the INITIAL open boolean, not restoration success. false is expected after a successful closed-to-open-to-closed read."),
            json!("Returned open describes the captured state during reading, not necessarily the final UI. Read errors still attempt close after successful opening; opening/cleanup failures can leave changed UI. Inspect actual state after errors rather than assuming restoration.")
        ]);
    }
    if get {
        result["constraints"] =
            json!({"limit": {"default": 20, "minimum": 1, "clamped_maximum": 100}});
        result["limits"].as_array_mut().unwrap().push(json!(
            "Returns up to limit DOM table rows without scrolling or paging. visible_row_count counts DOM tbody rows, not total matches or strictly viewport-visible rows. cells retains display strings; field_values maps nonempty column labels to positions, so duplicate labels overwrite keys and missing headers can misalign mapping. text is truncated to 500 characters. Readiness polling does not guarantee final results."
        ));
    } else if path == ["screener", "screens", "active"] {
        result["limits"].as_array_mut().unwrap().push(json!(
            "Returns current screen_title/dialog_title text, not a stable saved-screen ID or the saved-screen catalog. Titles can be null."
        ));
    } else if path == ["screener", "filters", "list"] {
        result["limits"].as_array_mut().unwrap().push(json!(
            "Returns visible filter-pill metadata, not the full saved filter definition or proof of all matching criteria. Use filters actions for supported edits; reading labels does not authorize changing them."
        ));
    } else if path == ["screener", "columns", "list"] {
        result["limits"].as_array_mut().unwrap().push(json!(
            "Returns nonempty displayed column names and zero-based positional indexes. Names/indexes are not storage column IDs; inspect columns config before storage edits."
        ));
    }
    Some(result)
}

fn discovery(path: &[&str]) -> Option<Value> {
    let (scope, detail) = match path {
        ["screener", "screens", "list"] => (
            "screen_title_menu or screen_catalog",
            "Default mode inspects visible title-menu entries; --catalog opens the screen catalog instead. Neither enumerates all account storage. Rows have name/index, optional ID and owner/shared flags; active is display-derived and not a unique identity. Select exact names within the inspected scope and reject ambiguity before switching.",
        ),
        ["screener", "screens", "actions"] => (
            "screen_title_menu",
            "Opens the title menu and reports recognized action labels/kinds/enabled state. save_available means a save candidate was seen; save_enabled means at least one was enabled, not that saving succeeded or is authorized. Unknown/localized menu entries may be missed.",
        ),
        ["screener", "filters", "actions"] => (
            "visible filter controls",
            "Probes one candidate numeric filter's manual-range popover and returns candidate_filter/range_options. It is not a capability audit of every filter. add_supported is currently false because this probe does not verify the add catalog; the separate filters add command still exists. numeric_modify_supported reflects found range options, not arbitrary text/multi-option editing.",
        ),
        ["screener", "columns", "actions"] => (
            "visible column settings",
            "Opens column settings and collects recognized category labels/counts. header_menu_actions is currently empty, so remove_supported/reset_supported are false; these probe fields do not negate the separate storage-based remove command. No column-reset command is provided. Category counts are displayed observations, not a complete catalog of storage IDs.",
        ),
        ["screener", "columns", "config"] => (
            "screen_storage_api",
            "Requires an already open Screener and readable active title/init data. Fetches screen storage with Desktop session credentials; no MCP OAuth or fallback. A missing storage configuration or exact title mismatch fails. Title matching alone cannot distinguish two same-title screens.",
        ),
        _ => return None,
    };
    let config = path == ["screener", "columns", "config"];
    let mut example = vec![json!("tv"), json!("--target-id"), json!("<target_id>")];
    example.extend(path.iter().map(|part| json!(part)));
    let mut result = json!({
        "source": "ui_screener_dialog", "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {"account_mutation": false, "local_file_write": false,
            "ui_mutation": !config, "provider_request": if config { json!(true) } else { Value::Null }},
        "scope": scope,
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"],
            "result_path": "data.screener_targets[].target_cli_args"}],
        "examples": [example],
        "limits": [detail,
            "These are observations of the selected Desktop screen, not official MCP screener queries. MCP can serve data-only screening but cannot supply saved-screen menus or storage column definitions. Do not infer edit authority from discovery results.",
            "Indexes and localized display names can change. Keep account-local IDs, names and configuration private; reread target state before applying a separately requested edit."]
    });
    if config {
        result["limits"].as_array_mut().unwrap().push(json!(
            "Returns screen_id, active_column_set and columns with index/id/params. Storage data can fall back to loaded init data when fields are absent; it is not guaranteed to equal unsaved UI state. name is paired from the visible column at the same index, with name_source visible_column_index, not ID-based verification. Missing names remain null."
        ));
    } else {
        result["limits"].as_array_mut().unwrap().extend([
            json!("Can open a closed Screener and opens/closes menus, catalogs or popovers even if the panel was already open. No saved-screen/filter/column edits are dispatched. Exact preexisting popup/focus state is not restored."),
            json!("opened_for_read records panel opening; restored_open_state is the initial open boolean, not a cleanup-success flag. Successful paths attempt panel closure only when opened by this command. Early errors can bypass cleanup and leave UI changed; inspect actual state after failure.")
        ]);
    }
    if path == ["screener", "screens", "list"] {
        result["constraints"] = json!({"catalog": {"default": false, "effect": "open catalog instead of inspecting title-menu entries"}});
        let mut example = result["examples"][0].as_array().unwrap().clone();
        example.push(json!("--catalog"));
        result["examples"]
            .as_array_mut()
            .unwrap()
            .push(json!(example));
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::screener::validation::validate_screener_limit;

    #[test]
    fn visible_read_examples_and_limits_match_execution() {
        for path in [
            vec!["screener", "status"],
            vec!["screener", "get"],
            vec!["screener", "screens", "active"],
            vec!["screener", "filters", "list"],
            vec!["screener", "columns", "list"],
            vec!["watchlist", "get"],
            vec!["screener", "screens", "list"],
            vec!["screener", "screens", "actions"],
            vec!["screener", "filters", "actions"],
            vec!["screener", "columns", "actions"],
            vec!["screener", "columns", "config"],
        ] {
            let spec = describe(&path).unwrap();
            for example in spec["examples"].as_array().unwrap() {
                let argv = example.as_array().unwrap();
                assert!(
                    Cli::try_parse_from(argv.iter().map(|value| value.as_str().unwrap())).is_ok()
                );
            }
        }
        let spec = describe(&["screener", "get"]).unwrap();
        assert_eq!(
            spec["constraints"]["limit"]["default"],
            validate_screener_limit(None).unwrap()
        );
        assert!(validate_screener_limit(Some(0)).is_err());
        assert_eq!(
            spec["constraints"]["limit"]["clamped_maximum"],
            validate_screener_limit(Some(101)).unwrap()
        );
    }
}
