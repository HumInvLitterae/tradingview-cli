//! Account discovery reads and partial-result interpretation.

use serde_json::{Value, json};
use tradingview_model::mcp_account::Request;

use super::mcp_reads::{read_metadata, read_timeout, symbol_constraint};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let [
        "mcp",
        family @ ("watchlist" | "alert"),
        action @ ("list" | "get"),
    ] = path
    else {
        return None;
    };
    let mut result = read_metadata();
    result["constraints"] = json!({"timeout": read_timeout()});
    result["limits"] = json!([
        "Completeness remains unconfirmed, including empty successful results. No pagination control or automatic traversal is provided.",
        "These reads do not activate a watchlist, create a default list or change alerts. Credential refresh can update local authorization state; no Desktop fallback is used.",
        "received_at is client receipt time, not account modification time. Optional fields retain null; unknown does not mean false or empty. Account-local IDs and returned account data stay private."
    ]);
    let (contract, example) = match (*family, *action) {
        ("watchlist", "list") => (
            "mcp_watchlists.v1",
            json!(["tv", "mcp", "watchlist", "list"]),
        ),
        ("watchlist", "get") => {
            result["constraints"]["id"] = json!({
                "type": "decimal string", "minimum": "0", "maximum": u64::MAX.to_string(),
                "leading_zeros": false, "whitespace": false, "account_local": true
            });
            result["discovery"] = json!([{
                "argument": "id", "argv": ["tv", "mcp", "watchlist", "list"],
                "result_path": "data.items[].id"
            }]);
            result["limits"].as_array_mut().unwrap().push(json!(
                "The result is data.watchlist, not data.items. Its ID must match the requested string; a mismatched or missing watchlist fails instead of becoming an empty list."
            ));
            (
                "mcp_watchlist.v1",
                json!(["tv", "mcp", "watchlist", "get", "12"]),
            )
        }
        ("alert", "list") => {
            result["constraints"]["symbol"] = symbol_constraint();
            result["constraints"]["symbol"]["optional"] = json!(true);
            result["constraints"]["active"] = json!({
                "optional": true, "choices": [true, false],
                "omitted": "no active-state filter", "syntax": "--active true|false"
            });
            result["limits"].as_array_mut().unwrap().push(json!(
                "List rows preserve provider order. Known symbol/active values conflicting with requested filters fail; null values remain unknown and do not prove a filter match. --active false is an explicit inactive filter, not omission."
            ));
            (
                "mcp_alerts.v1",
                json!([
                    "tv",
                    "mcp",
                    "alert",
                    "list",
                    "--symbol",
                    "NASDAQ:AAPL",
                    "--active",
                    "false"
                ]),
            )
        }
        _ => {
            result["constraints"]["ids"] = json!({
                "min_items": 1, "max_items": Request::MAX_ALERT_IDS, "unique": true,
                "item": {"type": "u64", "minimum": 1, "maximum": Request::MAX_ALERT_ID, "account_local": true}
            });
            result["discovery"] = json!([{
                "argument": "ids", "argv": ["tv", "mcp", "alert", "list"],
                "result_path": "data.items[].alert_id"
            }]);
            result["limits"].as_array_mut().unwrap().push(json!(
                "Detail items preserve requested ID order with requested_id, status and nested alert. Missing IDs have status unreported and alert null, not deleted/nonexistent. Inspect ids_status and unreported_count; returned_count counts returned alerts, not placeholder rows. Unexpected or duplicate returned IDs fail."
            ));
            (
                "mcp_alert_details.v1",
                json!(["tv", "mcp", "alert", "get", "12", "13"]),
            )
        }
    };
    result["output_contract"] = json!(contract);
    result["examples"] = json!([example]);
    let interpretation = if *family == "watchlist" {
        "Watchlist IDs are decimal strings. List rows and symbols retain provider order, including section labels; symbols null is not an empty list and section labels are not tradable symbols. Read results neither select nor activate the list."
    } else {
        "Alert IDs are positive integers. Conditions are a limited projection with condition_completeness unconfirmed, insufficient to recreate arbitrary Pine alerts. Notification messages and webhook URLs are excluded; has_webhook and notification flags do not prove delivery. Time fields remain provider strings, not normalized firing-history timestamps."
    };
    result["limits"]
        .as_array_mut()
        .unwrap()
        .push(json!(interpretation));
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn account_read_examples_and_id_limits_match_requests() {
        for family in ["watchlist", "alert"] {
            for action in ["list", "get"] {
                let spec = describe(&["mcp", family, action]).unwrap();
                let argv = spec["examples"][0].as_array().unwrap();
                assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
            }
        }
        let spec = describe(&["mcp", "watchlist", "get"]).unwrap();
        assert!(Request::watchlist(spec["constraints"]["id"]["maximum"].as_str().unwrap()).is_ok());
        assert!(Request::watchlist("0").is_ok());
        assert!(Request::watchlist("012").is_err());
        let spec = describe(&["mcp", "alert", "get"]).unwrap();
        let max = spec["constraints"]["ids"]["item"]["maximum"]
            .as_u64()
            .unwrap();
        assert!(Request::alert_details(&[max]).is_ok());
        assert!(Request::alert_details(&[max + 1]).is_err());
        let count = spec["constraints"]["ids"]["max_items"].as_u64().unwrap();
        assert!(Request::alert_details(&(1..=count).collect::<Vec<_>>()).is_ok());
        assert!(Request::alert_details(&(1..=count + 1).collect::<Vec<_>>()).is_err());
        assert_eq!(
            Request::alerts(None, Some(false)).unwrap().arguments()["active"],
            false
        );
    }
}
