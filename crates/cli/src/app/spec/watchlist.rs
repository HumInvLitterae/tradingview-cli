//! Legacy active-list mutation and partial bulk outcomes.

use serde_json::{Value, json};
use tradingview_model::watchlist::{MAX_WATCHLIST_BULK_DELAY_MS, MAX_WATCHLIST_BULK_SYMBOLS};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["watchlist", action @ ("add" | "add-bulk" | "remove")] = path else {
        return None;
    };
    let mut result = json!({
        "source": null, "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {"account_mutation": true, "ui_mutation": null,
            "provider_request": true, "local_file_write": false},
        "constraints": {"symbol": {"trimmed": true, "nonblank": true, "case_normalization": false,
            "qualified_symbol_validation": false}, "dry_run": {"supported": false}},
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"}],
        "limits": [
            "Targets the active account list through the selected Desktop session, with no list-ID argument. The internal API chooses the first active custom list; DOM fallback acts on the visible panel. Do not assume the target is pinned across operations.",
            "Tries internal watchlist_api first, then DOM only when api_fallback_allowed is reported. Fallback includes selected mutation transport/HTTP failures, so an uncertain write can precede DOM action. Post-check failures disable fallback. This is not a no-replay guarantee.",
            "API readback prefers the original list ID but falls back to an active list when it is absent; membership alone is not strict same-list identity proof. Inspect source, target_list, matched fields and error phase. A failure does not establish rollback.",
            "DOM paths can open the panel and change focus/input state without restoring the initial panel state. Counts and membership come from rendered rows, not complete account-list coverage. Output source distinguishes watchlist_api, dom_input and dom_row.",
            "Prefer explicit tv mcp watchlist list/get and add/remove <ID> for supported account-list changes. MCP does not require Desktop and avoids selecting the active list implicitly, but its existing-member add moves that member to the end whereas this legacy add skips it. Choose deliberately; do not silently substitute.",
            "No official-MCP fallback occurs. Keep account-local list details and raw error payloads private; do not repeat uncertain changes automatically."
        ]
    });
    let example = if *action == "add-bulk" {
        result["constraints"]["symbols"] = json!({"minimum": 1, "maximum_unique": MAX_WATCHLIST_BULK_SYMBOLS,
            "deduplication": "case-sensitive exact equality after trimming; preserves first occurrence"});
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
