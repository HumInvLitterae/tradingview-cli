//! Chart-local indicator identity, input validation and mutation evidence.

use crate::ops::MAX_SAFE_INTEGER;
use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let action = match path {
        ["indicator", action] => *action,
        ["data", "indicator"] => "get",
        _ => return None,
    };
    let mut result = json!({
        "source": "chart_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": action != "get"},
        "output_contract": null,
        "constraints": {"entity_id": {"nonblank": true, "scope": "selected chart"}},
        "discovery": [
            {"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"},
            {"argument": "entity_id", "argv": ["tv", "--target-id", "<target_id>", "state"], "result_path": "data.studies[].id"}
        ],
        "readback": {"argv": ["tv", "--target-id", "<target_id>", "indicator", "get", "<entity_id>"]},
        "limits": [
            "Use an entity ID from the selected chart, not an indicator name or an ID from another target.",
            "No official MCP, legacy createStudy or indicator-dialog fallback is implied.",
            "After a failed change inspect current chart state before retrying; an error does not prove no mutation occurred."
        ]
    });
    let example = match action {
        "add" => {
            result["constraints"] = json!({
                "indicator": {"nonblank": true, "tokens_joined_with": " ", "trimmed": true,
                    "matching": "case-sensitive exact metainfo description; exactly one candidate"},
                "inputs": {"type": "JSON object", "nonempty_when_supplied": true,
                    "value_types": ["null", "boolean", "number", "string"],
                    "integer_minimum": -MAX_SAFE_INTEGER, "integer_maximum": MAX_SAFE_INTEGER,
                    "keys": "must exist in resolved metainfo inputs"}
            });
            result["discovery"].as_array_mut().unwrap().truncate(1);
            result["catalog_discovery"] = Value::Null;
            result["readback"] = json!({"argv": ["tv", "--target-id", "<target_id>", "indicator", "get", "<returned_entity_id>"],
                "entity_id_path": "data.entity_id",
                "verification": "Awaits insertion; requires one new row with the exact name and verifies requested scalar inputs on that row."});
            result["limits"].as_array_mut().unwrap().extend([
                json!("No dedicated metainfo search command is provided. Use a known exact name; zero or multiple matches fail before insertion."),
                json!("Failure may attempt bounded cleanup of newly inserted candidates. Inspect mutation_performed and cleanup evidence; rollback is not guaranteed.")
            ]);
            json!(["tv", "indicator", "add", "Volume"])
        }
        "get" => {
            result["limits"].as_array_mut().unwrap().push(json!("Same read as data indicator. Visibility/inputs may be unknown; long string inputs are omitted, so this is not a complete study export."));
            json!(["tv", "indicator", "get", "<entity_id>"])
        }
        "remove" => {
            result["effects"]["removes_study"] = json!(true);
            result["readback"] = json!({"argv": ["tv", "--target-id", "<target_id>", "state"],
                "verification": "Checks that the same entity ID is absent after removal; failed verification is an error."});
            json!(["tv", "indicator", "remove", "<entity_id>"])
        }
        "toggle" => {
            result["constraints"]["conflicts"] = json!([["visible", "hidden"]]);
            result["effects"]["toggles_current_state"] = json!(false);
            result["variants"] = json!([
                {"when": {"flag_true": ["hidden"], "flag_false": ["visible"]}, "effects": {"requested_visible": false}},
                {"when": {"flag_false": ["hidden"]}, "effects": {"requested_visible": true}},
                {"when": {"flag_true": ["visible", "hidden"]}, "operation": "validation_error"}
            ]);
            result["limits"].as_array_mut().unwrap().push(json!("Despite the name, omission shows the indicator. hidden hides it; visible explicitly shows it. Reads actual visibility after setting it."));
            json!(["tv", "indicator", "toggle", "<entity_id>", "--hidden"])
        }
        "set" => {
            result["constraints"]["inputs"] = json!({"type": "JSON object", "nonempty": true,
                "keys": "input IDs of the selected study; at least one key must match"});
            result["discovery"]
                .as_array_mut()
                .unwrap()
                .push(json!({"argument": "inputs keys",
                "argv": ["tv", "--target-id", "<target_id>", "indicator", "get", "<entity_id>"],
                "result_path": "data.inputs[].id"}));
            result["effects"]["omitted_inputs"] = json!("unchanged");
            result["limits"].as_array_mut().unwrap().push(json!("Unlike add, local validation does not restrict values to scalars. Matched keys are set even if other keys are unmatched; inspect updated_inputs/unmatched_inputs and read back actual values. Returned requested values are not verification."));
            json!([
                "tv",
                "indicator",
                "set",
                "<entity_id>",
                "--inputs",
                "{\"<observed_input_id>\":20}"
            ])
        }
        _ => return None,
    };
    result["examples"] = json!([example]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, ops};
    use clap::Parser;

    #[test]
    fn examples_parse_and_add_set_validation_remain_distinct() {
        for action in ["add", "get", "remove", "toggle", "set"] {
            for example in describe(&["indicator", action]).unwrap()["examples"]
                .as_array()
                .unwrap()
            {
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
        let spec = describe(&["indicator", "add"]).unwrap();
        let max = spec["constraints"]["inputs"]["integer_maximum"]
            .as_i64()
            .unwrap();
        assert!(ops::parse_indicator_add_inputs(&json!({"length": max}).to_string()).is_ok());
        assert!(ops::parse_indicator_add_inputs(&json!({"length": max + 1}).to_string()).is_err());
        assert!(ops::parse_indicator_add_inputs("{\"x\":[1]}").is_err());
        assert!(ops::parse_indicator_inputs("{\"x\":[1]}").is_ok());
        assert!(ops::parse_indicator_inputs("{}").is_err());
        let toggle = describe(&["indicator", "toggle"]).unwrap();
        assert_eq!(toggle["effects"]["toggles_current_state"], false);
        assert_eq!(toggle["variants"][1]["effects"]["requested_visible"], true);
        assert_eq!(
            describe(&["data", "indicator"]).unwrap()["effects"]["chart_mutation"],
            false
        );
    }
}
