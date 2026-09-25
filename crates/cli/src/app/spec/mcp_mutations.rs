//! Account mutation instructions describe effects, never authorize execution.

use serde_json::{Value, json};
use tradingview_model::mcp_account::{AlertMutation, Request, WatchlistMutation};

use super::mcp_reads::{symbol_constraint, symbol_discovery};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["mcp", family, action] = path else {
        return None;
    };
    let mut result = match *family {
        "watchlist" => watchlist(action)?,
        "alert" => alert(action)?,
        _ => return None,
    };
    result["source"] = json!("tradingview_mcp");
    result["requires"] = json!({"authentication": true, "desktop": false});
    result["effects"]["account_mutation"] = json!(true);
    result["effects"]["provider_request"] = json!(true);
    result["constraints"]["timeout"] = json!({"supported": false});
    result["execution"] = json!({
        "authorization": "The user must request the target and account effect; examples grant no authority.",
        "automatic_retry": false,
        "local_effects": "Authentication renewal and request admission can update local state.",
        "uncertain_outcome": "Do not repeat a mutation after outcome_unknown. Read the target first; an error does not prove no change occurred.",
        "uncertain_create": "If no target ID is known, inspect the list and resolve ownership; do not choose a same-named item or create another automatically."
    });
    result["error_contract"] = json!("mcp_error.v1");
    result["readback"]["automatic"] = json!(true);
    result["readback"]["when"] = json!(
        "After a usable mutation reply identifies the target; missing target or readback failure stays unconfirmed."
    );
    result["readback"]["meaning"] = json!(
        "response_received is not proof of persistence. Inspect readback.status and per-item results; unavailable or unconfirmed evidence is not matched."
    );
    Some(result)
}

fn watchlist(action: &str) -> Option<Value> {
    let mut result = json!({
        "output_contract": WatchlistMutation::CONTRACT,
        "constraints": {}, "effects": {},
        "discovery": [{"argument": "id", "argv": ["tv", "mcp", "watchlist", "list"],
            "result_path": "data.items[].id"}],
        "readback": {"argv": ["tv", "mcp", "watchlist", "get", "<target_id>"]}
    });
    if action != "create" {
        result["constraints"]["id"] = json!({
            "type": "decimal-string", "minimum": 0, "maximum": u64::MAX,
            "leading_zeros": false, "account_local": true
        });
    }
    let example = match action {
        "create" => {
            result["constraints"]["name"] = name(WatchlistMutation::MAX_NAME_CHARS);
            result["constraints"]["symbols"] = symbols(0);
            result["discovery"] = json!([symbol_discovery("symbols")]);
            result["effects"]["creates_list"] = json!(true);
            json!([
                "tv",
                "mcp",
                "watchlist",
                "create",
                "Example research",
                "--symbols",
                "NASDAQ:EXAMPLE"
            ])
        }
        "update" => {
            result["constraints"]["name"] = name(WatchlistMutation::MAX_NAME_CHARS);
            result["constraints"]["description"] = json!({
                "max_unicode_scalars": WatchlistMutation::MAX_DESCRIPTION_CHARS,
                "nul_allowed": false, "empty_clears": true
            });
            result["constraints"]["at_least_one"] = json!(["name", "description"]);
            result["effects"]["omitted_fields"] = json!("unchanged");
            json!([
                "tv",
                "mcp",
                "watchlist",
                "update",
                "12",
                "--name",
                "Example renamed"
            ])
        }
        "add" | "remove" => {
            result["constraints"]["symbols"] = symbols(1);
            result["discovery"]
                .as_array_mut()
                .unwrap()
                .push(symbol_discovery("symbols"));
            result["effects"]["symbol_operation"] = json!(if action == "add" {
                "Append in request order; existing members move to the end."
            } else {
                "Remove the requested members."
            });
            json!(["tv", "mcp", "watchlist", action, "12", "NASDAQ:EXAMPLE"])
        }
        "delete" => {
            result["effects"]["deletes_list"] = json!(true);
            result["readback"]["argv"] = json!(["tv", "mcp", "watchlist", "list"]);
            result["readback"]["absence"] =
                json!("not_reported is not proof of deletion; list completeness is unconfirmed.");
            json!(["tv", "mcp", "watchlist", "delete", "12"])
        }
        _ => return None,
    };
    result["effects"]["requests_active_selection_change"] = json!(false);
    result["examples"] = json!([example]);
    result["limits"] = json!([
        "IDs and symbols in examples are synthetic; replace them with explicitly selected account targets."
    ]);
    Some(result)
}

fn alert(action: &str) -> Option<Value> {
    let mut result = json!({
        "output_contract": AlertMutation::CONTRACT,
        "constraints": {}, "effects": {},
        "discovery": [{"argument": "id", "argv": ["tv", "mcp", "alert", "list"],
            "result_path": "data.items[].alert_id"}],
        "readback": {"argv": ["tv", "mcp", "alert", "get", "<target_id>..."]}
    });
    let example = match action {
        "create" => {
            result["constraints"]["symbol"] = symbol_constraint();
            result["constraints"]["price"] = json!({"type": "f64", "finite": true});
            result["constraints"]["condition"] = json!({"choices": AlertMutation::CONDITIONS});
            result["constraints"]["resolution"] = json!({"choices": AlertMutation::RESOLUTIONS});
            result["discovery"] = json!([symbol_discovery("symbol")]);
            result["effects"]["creates_alert"] = json!(true);
            result["effects"]["expiration"] = json!("provider_default");
            result["effects"]["monitor"] = json!(false);
            json!([
                "tv",
                "mcp",
                "alert",
                "create",
                "NASDAQ:EXAMPLE",
                "--price",
                "100",
                "--name",
                "Example threshold"
            ])
        }
        "update" => {
            result["constraints"]["id"] = alert_id();
            result["constraints"]["at_least_one"] =
                json!(["name", "auto-deactivate", "email", "mobile-push", "popup"]);
            result["effects"]["omitted_fields"] = json!("unchanged");
            json!([
                "tv",
                "mcp",
                "alert",
                "update",
                "12",
                "--name",
                "Example renamed"
            ])
        }
        "stop" | "restart" | "delete" => {
            result["constraints"]["ids"] = json!({"min_items": 1,
                "max_items": Request::MAX_ALERT_IDS, "unique": true, "item": alert_id()});
            result["discovery"][0]["argument"] = json!("ids");
            json!(["tv", "mcp", "alert", action, "12", "13"])
        }
        _ => return None,
    };
    if matches!(action, "create" | "update") {
        result["constraints"]["name"] = name(AlertMutation::MAX_NAME_CHARS);
        for flag in ["auto-deactivate", "email", "mobile-push", "popup"] {
            result["constraints"][flag] = json!({"choices": [true, false],
                "omitted": if action == "create" { json!(false) } else { json!("unchanged") }});
        }
    }
    result["effects"]["reactivates"] = json!(matches!(action, "update" | "restart"));
    result["effects"]["deletes_fire_history"] = json!(action == "delete");
    result["effects"]["deletes_alert"] = json!(action == "delete");
    result["effects"]["stops_alert"] = json!(action == "stop");
    result["readback"]["absence"] =
        json!("Unreported IDs do not prove deletion; inspect per-target results.");
    result["examples"] = json!([example]);
    result["limits"] = json!([
        "Updating even just the name reactivates an inactive alert; obtain authority for that effect.",
        "Restart preserves existing conditions and notification settings; stop preserves settings and fire history.",
        "Only simple price-alert creation is supported; do not reconstruct Pine conditions or webhook settings.",
        "Example IDs are synthetic and must be replaced with selected account-local IDs."
    ]);
    Some(result)
}

fn name(max: usize) -> Value {
    json!({"max_unicode_scalars": max, "nonblank": true, "control_characters": false})
}

fn symbols(min: usize) -> Value {
    json!({"min_items": min, "max_items": WatchlistMutation::MAX_SYMBOLS,
        "unique": true, "item": symbol_constraint()})
}

fn alert_id() -> Value {
    json!({"type": "u64", "minimum": 1, "maximum": Request::MAX_ALERT_ID, "account_local": true})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::mcp_account::{AlertAction, AlertSettings};

    #[test]
    fn all_mutation_examples_parse_and_describe_timeout_rejection() {
        for (family, actions) in [
            ("watchlist", ["create", "update", "add", "remove", "delete"]),
            ("alert", ["create", "update", "stop", "restart", "delete"]),
        ] {
            for action in actions {
                let spec = describe(&["mcp", family, action]).unwrap();
                assert_eq!(spec["effects"]["account_mutation"], true);
                assert_eq!(spec["constraints"]["timeout"]["supported"], false);
                assert_eq!(spec["execution"]["automatic_retry"], false);
                for example in spec["examples"].as_array().unwrap() {
                    let argv = example
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap());
                    assert!(Cli::try_parse_from(argv).is_ok());
                }
            }
        }
        assert!(describe(&["mcp", "alert", "list"]).is_none());
    }

    #[test]
    fn watchlist_limits_use_characters_and_preserve_omission() {
        let create = describe(&["mcp", "watchlist", "create"]).unwrap();
        let max = create["constraints"]["name"]["max_unicode_scalars"]
            .as_u64()
            .unwrap() as usize;
        assert!(WatchlistMutation::create(&"é".repeat(max), &[]).is_ok());
        assert!(WatchlistMutation::create(&"é".repeat(max + 1), &[]).is_err());
        let update = describe(&["mcp", "watchlist", "update"]).unwrap();
        let max = update["constraints"]["description"]["max_unicode_scalars"]
            .as_u64()
            .unwrap() as usize;
        assert!(WatchlistMutation::update("12", None, Some(&"é".repeat(max))).is_ok());
        assert!(WatchlistMutation::update("12", None, Some(&"é".repeat(max + 1))).is_err());
        assert!(WatchlistMutation::update("12", None, None).is_err());
        assert_eq!(
            WatchlistMutation::update("12", None, Some(""))
                .unwrap()
                .arguments()["description"],
            ""
        );
        assert!(WatchlistMutation::delete("012").is_err());
        assert!(WatchlistMutation::delete("0").is_ok());
        let max = create["constraints"]["symbols"]["max_items"]
            .as_u64()
            .unwrap() as usize;
        let mut symbols = (0..max).map(|i| format!("NASDAQ:X{i}")).collect::<Vec<_>>();
        assert!(WatchlistMutation::create("Example", &symbols).is_ok());
        symbols.push("NASDAQ:EXTRA".into());
        assert!(WatchlistMutation::create("Example", &symbols).is_err());
        assert!(WatchlistMutation::symbols("12", &[], false).is_err());
    }

    #[test]
    fn alert_metadata_matches_defaults_reactivation_and_id_bounds() {
        let create = describe(&["mcp", "alert", "create"]).unwrap();
        for condition in create["constraints"]["condition"]["choices"]
            .as_array()
            .unwrap()
        {
            for resolution in create["constraints"]["resolution"]["choices"]
                .as_array()
                .unwrap()
            {
                let request = AlertMutation::create(
                    "NASDAQ:EXAMPLE",
                    100.0,
                    condition.as_str().unwrap(),
                    resolution.as_str().unwrap(),
                    AlertSettings {
                        name: Some("Example".into()),
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(
                    request.arguments()["email"],
                    create["constraints"]["email"]["omitted"]
                );
                assert_eq!(request.arguments()["monitor"], false);
            }
        }
        let update = AlertMutation::update(
            12,
            AlertSettings {
                name: Some("Renamed".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let spec = describe(&["mcp", "alert", "update"]).unwrap();
        assert_eq!(
            update.report(&[12], Value::Null)["effects"]["reactivates"],
            spec["effects"]["reactivates"]
        );
        assert!(update.arguments().get("email").is_none());
        assert!(AlertMutation::update(12, AlertSettings::default()).is_err());
        let max = spec["constraints"]["id"]["maximum"].as_u64().unwrap();
        assert!(AlertMutation::state(AlertAction::Stop, &[max]).is_ok());
        assert!(AlertMutation::state(AlertAction::Stop, &[max + 1]).is_err());
        assert!(AlertMutation::state(AlertAction::Stop, &[0]).is_err());
        assert!(AlertMutation::state(AlertAction::Stop, &[12, 12]).is_err());
    }
}
