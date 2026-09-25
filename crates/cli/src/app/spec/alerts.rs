//! Legacy account alert routes and source-dependent confirmation.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["alert", action @ ("list" | "create" | "delete")] = path else {
        return None;
    };
    let mut result = json!({
        "source": if *action == "create" { Value::Null } else { json!("internal_api") },
        "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {"provider_request": true, "local_file_write": false,
            "ui_mutation": if *action == "create" { Value::Null } else { json!(false) },
            "account_mutation": if *action == "list" { json!(false) } else { Value::Null }},
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"}],
        "limits": [
            "Uses the selected Desktop session and private alert endpoints, not official MCP. Account listing/deletion is not limited to the current chart symbol.",
            "Legacy payloads can include alert messages and account-local details. Condition projections omit study internals and cannot reconstruct complex Pine alerts; keep results private.",
            "Prefer official MCP for supported explicit-symbol price alerts and explicit-ID account management. Compare notification, expiry, condition and readback semantics first; no silent MCP fallback is performed. Pine alertcondition workflows require separate assessment."
        ]
    });
    let examples = match *action {
        "list" => {
            result["limits"].as_array_mut().unwrap().extend([
                json!("HTTP/fetch errors may be returned inside data.error with an outer success envelope and an empty alerts list. Inspect error; empty data alone is not proof that the account has no alerts."),
                json!("Missing/malformed row collections can normalize to empty results. Missing active defaults to true; timestamps preserve provider values without canonical normalization. No completeness or pagination guarantee is supplied.")
            ]);
            json!([["tv", "--target-id", "<target_id>", "alert", "list"]])
        }
        "create" => {
            result["effects"]["account_mutation"] = json!(true);
            result["constraints"] = json!({
                "price": {"finite": true},
                "condition": {"default": "crossing", "choices": ["crossing", "greater_than", "less_than"],
                    "normalization": "trim, ASCII lowercase, hyphen to underscore",
                    "internal_api_mapping": {"crossing": "cross", "greater_than": "cross_up", "less_than": "cross_down"}},
                "message": {"optional": true, "blank_or_omitted": "(none)"},
                "dry_run": {"supported": false}
            });
            result["limits"].as_array_mut().unwrap().extend([
                json!("Symbol and resolution come from the active chart, with resolution fallback 1 and currency fallback USD. Internal API uses split adjustment, about 30-day expiration, on_first_fire and auto_deactivate; popup/mobile_push are true, email/SMS are false and webhook is null. These are not the MCP notification defaults."),
                json!("greater_than/less_than map to crossing-up/down conditions, not generalized persistent inequalities. The internal API verifies a new ID with matching symbol marker, message, condition type and price tolerance; it does not independently confirm every notification/expiration field."),
                json!("Only pre-create API failures marked fallback-allowed lead to DOM. Once the API creation request is attempted, request/post-check failures do not retry via DOM."),
                json!("DOM fallback opens an alert dialog, sets a candidate price field and optionally message, then clicks Create. It does not set the requested condition UI: returned condition is an echo, and created true records a click, not a persisted alert or condition verification. Inspect source before claiming success."),
                json!("The dialog is not guaranteed to be restored. Failed readback can follow account creation; inspect the account before retrying to avoid duplicates.")
            ]);
            json!([[
                "tv",
                "--target-id",
                "<target_id>",
                "alert",
                "create",
                "--price",
                "100",
                "--condition",
                "crossing"
            ]])
        }
        _ => {
            result["constraints"] = json!({
                "target": {"exactly_one": ["id", "all"]},
                "id": {"trimmed": true, "nonblank": true, "numeric_only_validation": false},
                "dry_run": {"supported_only_with": "all"}
            });
            result["discovery"].as_array_mut().unwrap().push(json!({
                "argument": "id", "argv": ["tv", "alert", "list"], "result_path": "data.alerts[].alert_id"
            }));
            result["variants"] = json!([
                {"when": {"present": ["all", "dry-run"]}, "effects": {"account_mutation": false}},
                {"when": {"absent": ["dry-run"]}, "effects": {"account_mutation": true}}
            ]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Single-ID deletion first requires that ID in the list; absence is a validation error. Digit-only IDs are converted to JavaScript Number on the wire, with no explicit safe-integer bound; do not assume the stricter MCP ID contract."),
                json!("--all snapshots every listed account alert at execution time, rejects rows without IDs and deletes that set in one request. It is not current-symbol-only or tied to a prior dry-run snapshot. Empty targets return noop; there is no separate confirm-all flag."),
                json!("Readback checks absence of targeted IDs, not that no new alerts were concurrently created. Partial deletion/post-check failures are errors and do not roll back removed alerts. No DOM or official-MCP fallback is used.")
            ]);
            json!([[
                "tv",
                "--target-id",
                "<target_id>",
                "alert",
                "delete",
                "--all",
                "--dry-run"
            ]])
        }
    };
    result["examples"] = examples;
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::alert::{
        alert_condition_type, normalize_alert_condition, validate_alert_condition,
    };

    #[test]
    fn legacy_alert_examples_and_condition_mapping_match_execution() {
        for action in ["list", "create", "delete"] {
            let spec = describe(&["alert", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }
        let spec = describe(&["alert", "create"]).unwrap();
        for choice in spec["constraints"]["condition"]["choices"]
            .as_array()
            .unwrap()
        {
            let choice = choice.as_str().unwrap();
            assert!(validate_alert_condition(choice).is_ok());
            assert_eq!(
                spec["constraints"]["condition"]["internal_api_mapping"][choice],
                alert_condition_type(choice)
            );
        }
        assert_eq!(
            normalize_alert_condition(" Greater-Than ").unwrap(),
            "greater_than"
        );
        assert!(validate_alert_condition("above").is_err());
    }
}
