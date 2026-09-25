//! Saved-screen operations, test-name guards and bounded readback evidence.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [
        "screener",
        "screens",
        action @ ("switch" | "save" | "create" | "rename" | "save-as" | "delete"),
    ] = path
    else {
        return None;
    };
    let delete = *action == "delete";
    let mut result = json!({
        "source": "ui_screener_dialog", "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {"ui_mutation": !delete, "account_mutation": null,
            "provider_request": if delete { json!(true) } else { Value::Null }, "local_file_write": false},
        "constraints": {"dry_run": {"default": false}},
        "variants": [
            {"when": {"present": ["dry-run"]}, "effects": {"account_mutation": false},
                "operation": "resolve target and preview without final submission"},
            {"when": {"absent": ["dry-run"]}, "effects": {"account_mutation": if *action == "switch" { Value::Null } else { json!(true) }}}
        ],
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"],
            "result_path": "data.screener_targets[].target_cli_args"}],
        "limits": [
            "Operates the selected Desktop Screener, not an official MCP saved-screen API. Data-only MCP screening does not replace these saved-screen operations.",
            "Dry-run still connects and discovers the target. Menu operations can open panels, menus and name dialogs, focus/select input and then attempt cleanup; delete preview fetches storage. It is not an offline or UI-free validation.",
            "opened_for_mutation records panel opening, while restored_open_state is the initial open boolean, not cleanup success. Cleanup does not roll back screen selection or account edits; early errors can leave changed UI or uncertain writes. Inspect current state rather than blindly retrying.",
            "Account-local names, IDs and storage replies stay private. Example test names demonstrate code restrictions, not authorization to change an account."
        ]
    });
    if *action != "save" {
        result["constraints"]["name"] = json!({"trimmed": true, "nonblank": true});
    }
    if matches!(*action, "create" | "save-as" | "rename" | "delete") {
        result["constraints"]["name"]["non_dry_run_required_substring"] =
            json!(["CLI-Test", "テスト"]);
        result["constraints"]["name"]["substring_case_sensitive"] = json!(true);
    }
    let mut argv = vec![
        json!("tv"),
        json!("--target-id"),
        json!("<target_id>"),
        json!("screener"),
        json!("screens"),
        json!(action),
    ];
    if *action != "save" {
        argv.extend([json!("--name"), json!("CLI-Test-Example")]);
    }
    let detail = match *action {
        "switch" => {
            result["constraints"]["catalog"] = json!({"default": false});
            result["discovery"].as_array_mut().unwrap().push(json!({
                "argument": "name", "argv": ["tv", "screener", "screens", "list"],
                "result_path": "data.screens[].name", "catalog_mode": "add --catalog to discovery and switch"
            }));
            "Resolves one exact case-sensitive name in the menu or --catalog scope; missing/duplicate matches fail. No test-name restriction applies. An already-active title returns switched false. Otherwise readback waits for the title, not a full filter/column/data equivalence check, and leaves the selected screen active."
        }
        "save" => {
            "Saves the currently active screen without a --name selector or test-name restriction. Requires an enabled save menu action. Dry-run does not click it; actual save returns save_requested true and confirmation not_observable after a UI post-check. This does not prove durable persistence. Verify intended active screen before invoking."
        }
        "rename" => {
            result["constraints"]["to"] = result["constraints"]["name"].clone();
            result["constraints"]["to"]["different_from"] = json!("trimmed name");
            argv.extend([json!("--to"), json!("CLI-Test-Renamed")]);
            "Requires the active title to equal --name, including during dry-run. Both old and new names require the test substring for execution, and equal trimmed names fail even in preview. Preview opens the rename dialog without submitting; execution waits for the new title, not storage persistence."
        }
        "delete" => {
            result["constraints"]["confirm_delete"] = json!({"required_unless": "dry-run"});
            "Fetches saved screens using the Desktop session and resolves one exact name, not a title-menu item. Execution requires --confirm-delete and refuses an active target; preview may report that target without applying the active-target rejection. Deletes by storage ID and checks the name is absent from a fresh list. A readback failure can follow a successful deletion; do not infer rollback. No automatic switch to another screen is performed."
        }
        "create" => {
            "Opens the create-name dialog even in preview. Execution submits the new name and waits for that title to become active. created true reflects this UI readback, not an independent storage/uniqueness audit; previous screen selection is not restored."
        }
        _ => {
            "Opens the make-copy name dialog even in preview. Execution creates a copy of the active screen and waits for the requested title. It does not verify complete filter/column equivalence or independent persistence, and leaves the copy active."
        }
    };
    argv.push(json!("--dry-run"));
    result["examples"] = json!([argv]);
    result["limits"].as_array_mut().unwrap().push(json!(detail));
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::screener::validation::{
        validate_screener_screen_delete_request, validate_screener_screen_rename_request,
        validate_screener_screen_test_mutation_name,
    };

    #[test]
    fn examples_and_test_name_guards_match_dispatch_validation() {
        for action in ["switch", "save", "create", "rename", "save-as", "delete"] {
            let spec = describe(&["screener", "screens", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }
        for marker in ["CLI-Test", "テスト"] {
            assert!(validate_screener_screen_test_mutation_name(marker, false, "create").is_ok());
        }
        assert!(validate_screener_screen_test_mutation_name("ordinary", false, "create").is_err());
        assert!(validate_screener_screen_test_mutation_name("ordinary", true, "create").is_ok());
        assert!(validate_screener_screen_rename_request("old", "new", true).is_ok());
        assert!(validate_screener_screen_rename_request("old", "old", true).is_err());
        assert!(validate_screener_screen_rename_request("old", "CLI-Test-New", false).is_err());
        assert!(validate_screener_screen_delete_request("CLI-Test-Old", false, false).is_err());
        assert!(validate_screener_screen_delete_request("CLI-Test-Old", false, true).is_ok());
    }
}
