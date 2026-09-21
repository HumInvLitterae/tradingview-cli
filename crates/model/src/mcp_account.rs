//! I/O-free requests and response shaping for official MCP account reads.

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Watchlists,
    Watchlist,
    Alerts,
    AlertDetails,
}

#[derive(Clone, Debug)]
pub struct Request {
    kind: Kind,
    arguments: Value,
}

impl Request {
    pub fn watchlists() -> Self {
        Self {
            kind: Kind::Watchlists,
            arguments: json!({}),
        }
    }

    pub fn watchlist(id: &str) -> Result<Self, AppError> {
        if id.is_empty()
            || id.parse::<u64>().is_err()
            || (id.len() > 1 && id.starts_with('0'))
            || !id.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(invalid_request("watchlist_id"));
        }
        Ok(Self {
            kind: Kind::Watchlist,
            arguments: json!({"watchlist_id": id}),
        })
    }

    pub fn alerts(symbol: Option<&str>, active: Option<bool>) -> Result<Self, AppError> {
        let mut arguments = json!({});
        if let Some(symbol) = symbol {
            crate::mcp_bars::validate_symbol(symbol)?;
            arguments["symbol"] = json!(symbol);
        }
        if let Some(active) = active {
            arguments["active"] = json!(active);
        }
        Ok(Self {
            kind: Kind::Alerts,
            arguments,
        })
    }

    pub fn alert_details(ids: &[u64]) -> Result<Self, AppError> {
        let mut unique = std::collections::HashSet::new();
        if ids.is_empty()
            || ids.len() > 100
            || ids
                .iter()
                .any(|id| *id == 0 || *id > i64::MAX as u64 || !unique.insert(id))
        {
            return Err(invalid_request("alert_ids"));
        }
        Ok(Self {
            kind: Kind::AlertDetails,
            arguments: json!({"alert_ids": ids}),
        })
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn arguments(&self) -> Value {
        self.arguments.clone()
    }
}

fn invalid_request(reason: &str) -> AppError {
    AppError::new(ErrorKind::Validation, "Invalid MCP account request").with_details(json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": "invalid_request",
        "reason": reason,
        "tool_attempts": 0
    }))
}

fn invalid_response(reason: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "TradingView MCP account response is invalid",
    )
    .with_details(json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": "invalid_response",
        "reason": reason
    }))
}

pub fn normalize(request: &Request, value: Value, received_ms: u64) -> Result<Value, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid_response("wrapper"))?;
    for flag in ["success", "connected"] {
        match object.get(flag) {
            None | Some(Value::Bool(true)) => {}
            Some(Value::Bool(false)) => {
                return Err(AppError::new(
                    ErrorKind::InternalApiUnavailable,
                    "TradingView account read failed",
                )
                .with_details(json!({
                    "contract_version": "mcp_error.v1",
                    "source": "tradingview_mcp",
                    "code": "provider_error"
                })));
            }
            _ => return Err(invalid_response("status_flag")),
        }
    }
    let contract = match request.kind {
        Kind::Watchlists => "mcp_watchlists.v1",
        Kind::Watchlist => "mcp_watchlist.v1",
        Kind::Alerts => "mcp_alerts.v1",
        Kind::AlertDetails => "mcp_alert_details.v1",
    };
    let mut output = json!({
        "contract_version": contract,
        "source": "tradingview_mcp",
        "source_category": "desktop_free_read",
        "requires_desktop": false,
        "request": request.arguments,
        "client_observation": {
            "received_at": crate::mcp_bars::received_at(received_ms)?,
            "completeness": "unconfirmed"
        },
        "transport": {"status": "succeeded", "tool_attempts": 1}
    });

    if request.kind == Kind::Watchlist {
        let item = watchlist(
            value
                .get("watchlist")
                .ok_or_else(|| invalid_response("watchlist"))?,
        )?;
        let requested = request.arguments["watchlist_id"].as_str().unwrap();
        if item["id"].as_str() != Some(requested) {
            return Err(invalid_response("watchlist_id_mismatch"));
        }
        output["watchlist"] = item;
        output["client_observation"]["identity_match"] = json!("matched");
        return Ok(output);
    }

    let key = if request.kind == Kind::Watchlists {
        "watchlists"
    } else {
        "alerts"
    };
    let rows = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("missing_items"))?;
    let mut items = Vec::with_capacity(rows.len());
    let mut seen = std::collections::HashSet::new();
    for row in rows {
        let item = if request.kind == Kind::Watchlists {
            watchlist(row)?
        } else {
            alert(row)?
        };
        let id_key = if request.kind == Kind::Watchlists {
            "id"
        } else {
            "alert_id"
        };
        if !seen.insert(item[id_key].to_string()) {
            return Err(invalid_response("duplicate_id"));
        }
        if request.kind == Kind::Alerts {
            for field in ["symbol", "active"] {
                if let Some(expected) = request.arguments.get(field)
                    && !item[field].is_null()
                    && item[field] != *expected
                {
                    return Err(invalid_response("filter_mismatch"));
                }
            }
        }
        items.push(item);
    }

    output["client_observation"]["returned_count"] = json!(items.len());
    if request.kind == Kind::AlertDetails {
        let ids = request.arguments["alert_ids"].as_array().unwrap();
        if items.iter().any(|item| !ids.contains(&item["alert_id"])) {
            return Err(invalid_response("unexpected_alert_id"));
        }
        let returned = items.len();
        output["items"] = Value::Array(
            ids.iter()
                .map(|id| {
                    let item = items.iter().find(|item| item["alert_id"] == *id);
                    json!({
                        "requested_id": id,
                        "status": if item.is_some() { "returned" } else { "unreported" },
                        "alert": item
                    })
                })
                .collect(),
        );
        output["client_observation"]["unreported_count"] = json!(ids.len() - returned);
        output["client_observation"]["ids_status"] = json!(if returned == ids.len() {
            "all_returned"
        } else {
            "partial"
        });
    } else {
        output["items"] = json!(items);
        output["client_observation"]["order"] = json!("provider");
    }
    Ok(output)
}

fn optional(row: &Value, field: &str, kind: &str) -> Result<Value, AppError> {
    let value = row.get(field).unwrap_or(&Value::Null);
    let valid = value.is_null()
        || match kind {
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            "number" => value.as_f64().is_some_and(f64::is_finite),
            _ => false,
        };
    if !valid {
        return Err(invalid_response("field_type"));
    }
    Ok(value.clone())
}

fn watchlist(row: &Value) -> Result<Value, AppError> {
    let id = row
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_response("watchlist_id"))?;
    let mut item = json!({"id": id.to_string()});
    for field in ["name", "description", "type", "created", "modified"] {
        item[field] = optional(row, field, "string")?;
    }
    for field in ["active", "shared"] {
        item[field] = optional(row, field, "boolean")?;
    }
    item["symbols"] = match row.get("symbols") {
        None | Some(Value::Null) => Value::Null,
        Some(Value::Array(symbols)) if symbols.iter().all(Value::is_string) => json!(symbols),
        _ => return Err(invalid_response("watchlist_symbols")),
    };
    Ok(item)
}

fn alert(row: &Value) -> Result<Value, AppError> {
    let id = row
        .get("alert_id")
        .and_then(Value::as_u64)
        .filter(|id| *id > 0 && *id <= i64::MAX as u64)
        .ok_or_else(|| invalid_response("alert_id"))?;
    let mut item = json!({"alert_id": id});
    for field in [
        "name",
        "symbol",
        "type",
        "condition_type",
        "resolution",
        "create_time",
        "expiration",
        "last_fire_time",
        "last_fire_bar_time",
        "last_stop_reason",
    ] {
        item[field] = optional(row, field, "string")?;
    }
    for field in ["active", "auto_deactivate", "has_webhook"] {
        item[field] = optional(row, field, "boolean")?;
    }
    item["threshold"] = optional(row, "threshold", "number")?;
    // Return the supported condition projection, never arbitrary account payloads.
    item["conditions"] = match row.get("conditions") {
        None | Some(Value::Null) => Value::Null,
        Some(Value::Array(conditions)) => {
            let mut projected = Vec::new();
            for condition in conditions {
                if !condition.is_object() {
                    return Err(invalid_response("condition"));
                }
                let mut result = json!({});
                for field in ["type", "frequency", "resolution"] {
                    result[field] = optional(condition, field, "string")?;
                }
                result["cross_interval"] = optional(condition, "cross_interval", "boolean")?;
                result["series"] = match condition.get("series") {
                    None | Some(Value::Null) => Value::Null,
                    Some(Value::Array(series)) => {
                        let mut values = Vec::new();
                        for entry in series {
                            if !entry.is_object() {
                                return Err(invalid_response("condition_series"));
                            }
                            values.push(json!({
                                "type": optional(entry, "type", "string")?,
                                "value": optional(entry, "value", "number")?
                            }));
                        }
                        json!(values)
                    }
                    _ => return Err(invalid_response("condition_series")),
                };
                projected.push(result);
            }
            json!(projected)
        }
        _ => return Err(invalid_response("conditions")),
    };
    item["condition_completeness"] = json!("unconfirmed");
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_reject_ambiguous_ids_and_preserve_false_filters() {
        for id in ["", "01", "-1", "name", "1/2", "18446744073709551616"] {
            assert!(Request::watchlist(id).is_err());
        }
        for ids in [vec![], vec![0], vec![1, 1], vec![u64::MAX], vec![1; 101]] {
            assert!(Request::alert_details(&ids).is_err());
        }
        assert_eq!(
            Request::alerts(None, Some(false)).unwrap().arguments(),
            json!({"active": false})
        );
        assert!(Request::alerts(Some("EXAMPLE"), None).is_err());
    }

    #[test]
    fn watchlists_keep_provider_order_sections_and_unknown_fields() {
        let data = normalize(
            &Request::watchlists(),
            json!({
                "watchlists": [
                    {"id": 12, "name": "Example", "symbols": ["###Section", "NASDAQ:EXAMPLE"]},
                    {"id": 10, "symbols": null}
                ]
            }),
            0,
        )
        .unwrap();
        assert_eq!(data["items"][0]["id"], "12");
        assert_eq!(data["items"][0]["symbols"][0], "###Section");
        assert!(data["items"][1]["symbols"].is_null());
        assert!(data["items"][0]["active"].is_null());
        assert_eq!(data["client_observation"]["completeness"], "unconfirmed");
        let empty = normalize(&Request::watchlists(), json!({"watchlists": []}), 0).unwrap();
        assert_eq!(empty["client_observation"]["returned_count"], 0);
        let request = Request::watchlist("12").unwrap();
        assert!(normalize(&request, json!({"watchlist": {"id": 10}}), 0).is_err());
        assert!(normalize(&request, json!({"watchlist": null}), 0).is_err());
    }

    #[test]
    fn alert_details_preserve_requested_order_and_unreported_ids() {
        let request = Request::alert_details(&[12, 10, 11]).unwrap();
        let data = normalize(
            &request,
            json!({
                "success": true,
                "alerts": [
                    {
                        "alert_id": 10,
                        "active": false,
                        "message": "private",
                        "webhook_url": "private"
                    },
                    {
                        "alert_id": 12,
                        "threshold": 0,
                        "conditions": [{
                            "type": "greater",
                            "series": [
                                {"type": "price"},
                                {"type": "value", "value": 0}
                            ]
                        }]
                    }
                ]
            }),
            0,
        )
        .unwrap();
        assert_eq!(data["items"][0]["alert"]["alert_id"], 12);
        assert_eq!(data["items"][1]["alert"]["active"], false);
        assert_eq!(data["items"][2]["status"], "unreported");
        assert!(data["items"][2]["alert"].is_null());
        assert_eq!(data["client_observation"]["ids_status"], "partial");
        assert_eq!(
            data["items"][0]["alert"]["conditions"][0]["series"][1]["value"],
            0
        );
        assert!(!data.to_string().contains("private"));
        assert!(normalize(&request, json!({"alerts": [{"alert_id": 99}]}), 0).is_err());
    }

    #[test]
    fn malformed_or_failed_account_reads_do_not_become_empty_success() {
        for value in [
            json!({}),
            json!({"watchlists": null}),
            json!({"watchlists": [{}]}),
            json!({"watchlists": [{"id": 1}, {"id": 1}]}),
            json!({"watchlists": [{"id": 1, "symbols": [0]}]}),
            json!({"watchlists": [], "success": false}),
            json!({"watchlists": [], "connected": false}),
            json!({"watchlists": [], "connected": "yes"}),
        ] {
            assert!(normalize(&Request::watchlists(), value, 0).is_err());
        }
        let request = Request::alerts(Some("NASDAQ:EXAMPLE"), Some(false)).unwrap();
        for row in [
            json!({"alert_id": 1, "symbol": "NASDAQ:OTHER"}),
            json!({"alert_id": 1, "active": true}),
            json!({"alert_id": 1, "active": "false"}),
            json!({"alert_id": -1}),
            json!({"alert_id": 1, "conditions": "invalid"}),
            json!({"alert_id": 1, "conditions": [null]}),
            json!({"alert_id": 1, "conditions": [{"series": [{"value": "invalid"}]}]}),
        ] {
            assert!(normalize(&request, json!({"alerts": [row]}), 0).is_err());
        }
        let empty = normalize(&request, json!({"alerts": []}), 0).unwrap();
        assert_eq!(empty["items"], json!([]));
    }
}
