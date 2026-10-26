//! Legacy active-list mutation and partial bulk outcomes.

use serde_json::{Value, json};
use tradingview_model::watchlist::{MAX_WATCHLIST_BULK_DELAY_MS, MAX_WATCHLIST_BULK_SYMBOLS};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if path == ["watchlist", "move"] || path == ["watchlist", "section", "move"] {
        return Some(json!({
            "source": "desktop_watchlist_handler",
            "output_contract": "watchlist_reorder.v1",
            "requires": {"desktop": true, "authentication": true},
            "effects": {
                "account_mutation": true,
                "ui_mutation": true,
                "provider_request": true,
                "local_file_write": false
            },
            "constraints": {
                "custom_lists_only": true,
                "destination_count": 1,
                "duplicate_entries_allowed": false,
                "selected_rows_allowed": false,
                "sorted_view_allowed": false,
                "automatic_retry": false,
                "automatic_rollback": false,
                "operation_budget_ms": 8000,
                "concurrency": "non_atomic_full_replace"
            },
            "limits": [
                "Targets the displayed custom list and pins its ID. Exact duplicate symbols and duplicate headings anywhere in the list are rejected; collapse markers do not distinguish headings.",
                "Symbol moves require an exchange-qualified source in the display array and an expanded destination section. before/after use exact symbol anchors; to-section appends within the named section, including an empty section.",
                "Section moves carry the heading and all its members, preserving internal order and collapse state. Leading unsectioned symbols stay in place. before/after use exact heading names.",
                "A fresh GET, internal state and rendered-row order are checked before one existing reorder handler call. Hidden members must reconstruct to the expected complete array. Already-satisfied moves do not write.",
                "The provider uses full-array replacement without an atomic revision precondition. Preflight checks cannot prevent an edit between the final GET and POST from being overwritten. Use only when the list is not being edited elsewhere; even matched readback cannot prove no concurrent edit was lost.",
                "Success requires the complete saved array, internal display order and observable DOM rows to agree. Virtualization prevents all-row visual verification. Other-tab synchronization can occur but is not a success requirement.",
                "Errors retain write_attempted, changed and independent saved/display confirmations. No retry, UI reload, list switch, direct REST fallback or compensating replacement. A failed save can leave the UI reordered."
            ]
        }));
    }
    if path == ["watchlist", "inspect"]
        || matches!(
            path,
            ["watchlist", "section", "rename" | "collapse" | "expand"]
        )
    {
        let inspect = path == ["watchlist", "inspect"];
        return Some(json!({
            "source": if inspect { "watchlist_api" } else { "desktop_watchlist_handler" },
            "output_contract": if inspect { "watchlist_inspect.v1" } else { "watchlist_section_mutation.v1" },
            "requires": {"desktop": true, "authentication": true},
            "effects": {
                "account_mutation": !inspect,
                "ui_mutation": !inspect,
                "provider_request": true,
                "local_file_write": false
            },
            "constraints": {
                "custom_lists_only": true,
                "automatically_selects_list": false,
                "section_matching": "exact unique display name",
                "selected_rows_allowed": false,
                "sorted_view_allowed": false,
                "automatic_retry": false,
                "automatic_rollback": false,
                "operation_budget_ms": 8000
            },
            "limits": [
                "Inspect reads a saved custom list by canonical decimal ID without selecting it. symbols and entries can be null when unknown; empty arrays mean empty.",
                "Section commands pin the displayed custom list ID before invoking its existing handler once. The section must be rendered, unique, and consistent with saved state; no selected rows or sorted view are allowed.",
                "Names are preserved literally. Blank names, control characters, reserved section markers and destination collisions are rejected. No automatic case conversion or suffixing is performed by the CLI.",
                "Success requires independent saved-state and target DOM confirmation; handler completion is insufficient. No-op requires both states to match. Internal handler actions can synchronize to other tabs and restore local display after errors.",
                "Errors preserve write_attempted, changed, saved_confirmation and display_confirmation. A lost execution response leaves write_attempted and changed null. Do not repeat an uncertain write.",
                "No atomic server-side concurrency guarantee, UI reload, list switch, MCP fallback or additional compensating write. Other-tab synchronization is not a success requirement."
            ]
        }));
    }
    let ["watchlist", action @ ("add" | "add-bulk" | "remove")] = path else {
        return None;
    };
    let mut result = json!({
        "source": null,
        "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {
            "account_mutation": true,
            "ui_mutation": null,
            "provider_request": true,
            "local_file_write": false
        },
        "constraints": {
            "symbol": {
                "trimmed": true,
                "nonblank": true,
                "case_normalization": false,
                "qualified_symbol_validation": false
            },
            "dry_run": {"supported": false}
        },
        "discovery": [{
            "argument": "target-id",
            "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"
        }],
        "limits": [
            "Targets the active account list through the selected Desktop session, with no list-ID argument. The internal API chooses the first active custom list; DOM fallback acts on the visible panel. Do not assume the target is pinned across operations.",
            "Tries internal watchlist_api first. DOM fallback requires a recognized pre-dispatch list failure and api_fallback_allowed=true. Once a POST is attempted, transport/HTTP/readback failures stop without DOM replay.",
            "API readback requires the original list ID and readable symbols; another active list cannot confirm the mutation. Inspect source, target_list, matched fields and error phase. A failure does not establish rollback.",
            "DOM paths can open the panel and change focus/input state without restoring the initial panel state. Counts and membership come from rendered rows, not complete account-list coverage. Output source distinguishes watchlist_api, dom_input and dom_row.",
            "Prefer explicit tv mcp watchlist list/get and add/remove <ID> for supported account-list changes. MCP does not require Desktop and avoids selecting the active list implicitly, but its existing-member add moves that member to the end whereas this legacy add skips it. Choose deliberately; do not silently substitute.",
            "No official-MCP fallback occurs. Keep account-local list details and raw error payloads private; do not repeat uncertain changes automatically."
        ]
    });
    let example = if *action == "add-bulk" {
        result["constraints"]["symbols"] = json!({
            "minimum": 1,
            "maximum_unique": MAX_WATCHLIST_BULK_SYMBOLS,
            "deduplication": "case-sensitive exact equality after trimming; preserves first occurrence"
        });
        result["constraints"]["delay_ms"] =
            json!({"minimum": 0, "maximum": MAX_WATCHLIST_BULK_DELAY_MS, "default": 750});
        result["constraints"]["allow_partial"] = json!({"default": false});
        result["limits"].as_array_mut().unwrap().extend([
            json!("Processes every unique symbol sequentially even after item errors. Delay applies between unique attempts, including failures, not duplicates; it is not a total timeout. Each item resolves the active list again, so external selection changes can split a batch across lists."),
            json!("allow_partial changes only final error handling, not whether processing continues: without it any failure returns an error with the batch payload in details; with it the payload succeeds even with failed rows. Neither mode rolls back earlier additions."),
            json!("Inspect results statuses added/already_present/failed/skipped_duplicate and counts. requested_count includes duplicates; processed_count excludes them. Case variants remain distinct. Retrying the full batch can repeat uncertain failed writes; choose follow-up from per-item evidence.")
        ]);
        json!([
            "tv",
            "--target-id",
            "<target_id>",
            "watchlist",
            "add-bulk",
            "NASDAQ:AAPL",
            "NASDAQ:MSFT",
            "--delay-ms",
            "750"
        ])
    } else {
        let limit = if *action == "add" {
            "An exact existing member returns already_present without reordering. DOM add opens search, types the symbol, sends Enter/Escape and verifies an exact rendered data-symbol-full match. This does not validate exchange identity or full list ordering."
        } else {
            "An absent member is a validation error, not a successful no-op. DOM removal uses the exact rendered symbol row's remove control and checks absence; it does not click a confirmation dialog. Virtualized/missing rows can limit that evidence."
        };
        result["limits"].as_array_mut().unwrap().push(json!(limit));
        json!([
            "tv",
            "--target-id",
            "<target_id>",
            "watchlist",
            action,
            "NASDAQ:AAPL"
        ])
    };
    result["examples"] = json!([example]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::watchlist::validate_watchlist_add_bulk_request;

    #[test]
    fn legacy_watchlist_examples_and_unique_limits_match_validation() {
        for action in ["add", "add-bulk", "remove"] {
            let spec = describe(&["watchlist", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }
        let spec = describe(&["watchlist", "add-bulk"]).unwrap();
        let max = spec["constraints"]["symbols"]["maximum_unique"]
            .as_u64()
            .unwrap();
        let mut symbols = (0..max).map(|i| format!("NASDAQ:X{i}")).collect::<Vec<_>>();
        assert!(validate_watchlist_add_bulk_request(&symbols, 0).is_ok());
        symbols.push(" NASDAQ:X0 ".into());
        let delay = spec["constraints"]["delay_ms"]["maximum"].as_u64().unwrap();
        assert!(validate_watchlist_add_bulk_request(&symbols, delay).is_ok());
        assert!(validate_watchlist_add_bulk_request(&symbols, delay + 1).is_err());
        symbols.push("NASDAQ:EXTRA".into());
        assert!(validate_watchlist_add_bulk_request(&symbols, 0).is_err());
    }
}
