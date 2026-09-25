//! Filter UI presets and conditional saved-storage edits.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [
        "screener",
        "filters",
        action @ ("add" | "modify" | "remove" | "clear"),
    ] = path
    else {
        return None;
    };
    let mut result = json!({
        "source": "ui_screener_dialog", "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {"ui_mutation": true, "account_mutation": null,
            "provider_request": null, "local_file_write": false},
        "constraints": {"dry_run": {"default": false}},
        "variants": [
            {"when": {"present": ["dry-run"]}, "effects": {"account_mutation": false}, "operation": "live target discovery without applying the requested filter change"},
            {"when": {"absent": ["dry-run"]}, "effects": {"account_mutation": null}, "operation": "UI edit or saved-storage write according to operation and current state"}
        ],
        "discovery": [
            {"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.screener_targets[].target_cli_args"},
            {"argument": "index/text", "argv": ["tv", "screener", "filters", "list"], "result_path": "data.filters[]"}
        ],
        "limits": [
            "Dry-run is live: it can open the panel, search or inspect popovers, and the storage range path can fetch account configuration. Preview does not establish that a later edit is permitted or supported.",
            "opened_for_mutation describes panel opening; restored_open_state is the initial open boolean, not cleanup success. Cleanup does not undo edits, restore every popup/focus state or guarantee completion after errors.",
            "UI add/modify paths have no test-name guard. Storage remove/clear/range-modify execution requires a screen title containing case-sensitive CLI-Test or テスト. Do not infer identical permissions or persistence from the shared command family.",
            "Storage edits replace filters in a fetched screen document using the Desktop session, then compare ordered raw filter definitions with numeric equivalence. No automatic merge of intervening edits or rollback is provided. A failure after writing can leave the change applied.",
            "Official MCP can perform a suitable independent data query, not edit these saved/visible Desktop filters. No automatic provider fallback occurs; keep account configuration and IDs private."
        ]
    });
    if matches!(*action, "modify" | "remove") {
        result["constraints"]["selector"] = json!({
            "exactly_one": ["index", "text"], "index": "zero-based",
            "text": "trimmed case-insensitive substring matching exactly one visible pill",
            "blank_text": "treated as omitted"
        });
    }
    let args = match *action {
        "add" => {
            result["constraints"]["name"] = json!({"trimmed": true, "nonblank": true});
            result["constraints"]["range"] = json!({"at_least_one": ["min", "max"], "finite": true, "when_both": "max > min", "meaning": "visible preset label matching, not arbitrary numeric input"});
            result["limits"].as_array_mut().unwrap().push(json!(
                "Searches the add catalog and chooses a matching candidate/preset. Preview resolves only the candidate, not whether the requested range option exists. Execution clicks the range and verifies a newly matching visible pill; it does not independently verify saved storage. Single bounds use recognized greater/less/above/below labels, not a normalized inclusive-bound contract."
            ));
            vec!["--name", "Change", "--min", "3"]
        }
        "modify" => {
            result["constraints"]["mode"] = json!({"option": "trimmed nonblank; conflicts with min/max", "otherwise": "numeric preset"});
            result["constraints"]["range"] = json!({
                "finite": true, "min_only_choices": [3,5,10,15,20,30,40,50,60,70,80,90],
                "bounded": {"min": 0, "max_choices": [3,5,10,20,30]},
                "max_only": false, "preset_tolerance": "absolute difference < 0.000001"
            });
            result["limits"].as_array_mut().unwrap().extend([
                json!("UI numeric modification selects a supported visible percent preset, not arbitrary bounds. Preview normally resolves the pill without opening/verifying the range preset. Already-matching text can return modified false; success checks the pill text, not persistence or result-row correctness."),
                json!("Option mode resolves one normalized case-insensitive exact option first, otherwise a unique bidirectional substring match. Preview opens the option popover. Execution can clear other selected options before choosing the target; this is not an additive multi-select API. Text readback is not complete filter-definition verification."),
                json!("When both button_found and open are false, numeric modification with --index first tries current-screen storage. Only Condition filters with above/between operations are supported; the same public preset validation still applies, with exact zero required for a stored bounded range. Unsupported/unavailable preflight may continue to UI, but a failed post-write check does not fall back."),
                json!("The storage range path returns scope screen_storage_api and requests a page reload after readback, with visible_refresh.confirmed false. A requested reload is not observed filter/row readiness.")
            ]);
            vec!["--index", "0", "--min", "0", "--max", "5"]
        }
        "remove" => {
            result["limits"].as_array_mut().unwrap().push(json!(
                "Preview resolves the visible pill only. Execution requires the test-name guard, equal visible/storage filter counts and a valid saved index; alignment checks count, not identity. It removes the saved entry at the visible index and verifies storage before optional refresh."
            ));
            vec!["--index", "0"]
        }
        _ => {
            result["constraints"]["confirm_clear"] = json!({"required_unless": "dry-run"});
            result["limits"].as_array_mut().unwrap().push(json!(
                "Execution requires --confirm-clear and the test-name guard, checks visible/storage counts and writes an empty saved filters list. Preview reports visible targets without testing the storage guard/alignment or writing. This clears saved filters, not just temporary UI pills."
            ));
            vec![]
        }
    };
    if matches!(*action, "remove" | "clear") {
        result["variants"][1]["effects"]["account_mutation"] = json!(true);
        result["variants"][1]["effects"]["provider_request"] = json!(true);
        result["limits"].as_array_mut().unwrap().push(json!(
            "After storage verification, full-page targets can reload and poll up to 20 times at 500 ms for filter count. visible_refresh.confirmed verifies count only; false or skipped refresh can accompany successful storage edits. Dialog targets skip reload. It is not proof of filter content, row coverage or freshness."
        ));
    }
    let mut example = vec![
        "tv",
        "--target-id",
        "<target_id>",
        "screener",
        "filters",
        action,
    ];
    example.extend(args);
    example.push("--dry-run");
    result["examples"] = json!([example]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::screener::validation::{
        validate_screener_filter_add_request, validate_screener_filter_clear,
        validate_screener_filter_modify_request,
    };

    #[test]
    fn filter_examples_and_presets_match_request_validation() {
        for action in ["add", "modify", "remove", "clear"] {
            let spec = describe(&["screener", "filters", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }
        let spec = describe(&["screener", "filters", "modify"]).unwrap();
        for min in spec["constraints"]["range"]["min_only_choices"]
            .as_array()
            .unwrap()
        {
            assert!(
                validate_screener_filter_modify_request(
                    Some(0),
                    None,
                    min.as_f64(),
                    None,
                    None,
                    true
                )
                .is_ok()
            );
        }
        for max in spec["constraints"]["range"]["bounded"]["max_choices"]
            .as_array()
            .unwrap()
        {
            assert!(
                validate_screener_filter_modify_request(
                    Some(0),
                    None,
                    Some(0.0),
                    max.as_f64(),
                    None,
                    true
                )
                .is_ok()
            );
        }
        assert!(
            validate_screener_filter_modify_request(Some(0), None, None, Some(5.0), None, true)
                .is_err()
        );
        assert!(validate_screener_filter_add_request("Change", None, Some(5.0), true).is_ok());
        assert!(validate_screener_filter_clear(false, false).is_err());
        assert!(validate_screener_filter_clear(true, false).is_ok());
    }
}
