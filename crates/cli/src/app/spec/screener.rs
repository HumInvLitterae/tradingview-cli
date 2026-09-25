//! Visible Screener context and legacy watchlist reads.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
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
        ] {
            let spec = describe(&path).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|value| value.as_str().unwrap())).is_ok());
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
