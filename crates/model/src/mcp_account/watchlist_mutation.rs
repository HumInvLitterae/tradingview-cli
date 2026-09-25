//! Explicit watchlist changes and evidence from subsequent account reads.

use super::{Request, invalid_request, invalid_response};
use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Create,
    Update,
    Add,
    Remove,
    Delete,
}

#[derive(Clone, Debug)]
pub struct WatchlistMutation {
    action: Action,
    arguments: Value,
}

impl WatchlistMutation {
    pub const MAX_NAME_CHARS: usize = 500;
    pub const MAX_DESCRIPTION_CHARS: usize = 4096;
    pub const MAX_SYMBOLS: usize = 100;
    pub const CONTRACT: &str = "mcp_watchlist_mutation.v1";

    pub fn create(name: &str, symbols: &[String]) -> Result<Self, AppError> {
        name_value(name)?;
        symbols_value(symbols, true)?;
        Ok(Self {
            action: Action::Create,
            arguments: json!({"name": name, "symbols": symbols}),
        })
    }

    pub fn update(
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Self, AppError> {
        let mut arguments = Request::watchlist(id)?.arguments();
        if name.is_none() && description.is_none() {
            return Err(invalid_request("empty_update"));
        }
        if let Some(name) = name {
            name_value(name)?;
            arguments["name"] = json!(name);
        }
        if let Some(description) = description {
            if description.chars().count() > Self::MAX_DESCRIPTION_CHARS
                || description.contains('\0')
            {
                return Err(invalid_request("description"));
            }
            arguments["description"] = json!(description);
        }
        Ok(Self {
            action: Action::Update,
            arguments,
        })
    }

    pub fn symbols(id: &str, symbols: &[String], remove: bool) -> Result<Self, AppError> {
        let mut arguments = Request::watchlist(id)?.arguments();
        symbols_value(symbols, false)?;
        arguments["symbols"] = json!(symbols);
        Ok(Self {
            action: if remove { Action::Remove } else { Action::Add },
            arguments,
        })
    }

    pub fn delete(id: &str) -> Result<Self, AppError> {
        Ok(Self {
            action: Action::Delete,
            arguments: Request::watchlist(id)?.arguments(),
        })
    }

    pub fn action(&self) -> Action {
        self.action
    }

    pub fn arguments(&self) -> Value {
        self.arguments.clone()
    }

    pub fn action_name(&self) -> &'static str {
        match self.action {
            Action::Create => "create",
            Action::Update => "update",
            Action::Add => "add",
            Action::Remove => "remove",
            Action::Delete => "delete",
        }
    }

    /// A successful tool reply is not proof that the requested state persisted.
    pub fn target_after_reply(&self, value: &Value) -> Result<Option<String>, AppError> {
        if !value.is_object() {
            return Err(invalid_response("mutation_reply"));
        }
        match value.get("success") {
            Some(Value::Bool(false)) => {
                return Err(AppError::new(
                    ErrorKind::InternalApiUnavailable,
                    "TradingView did not acknowledge the watchlist change",
                )
                .with_details(json!({
                    "contract_version": "mcp_error.v1",
                    "source": "tradingview_mcp",
                    "code": "provider_error"
                })));
            }
            None | Some(Value::Bool(true)) => {}
            _ => return Err(invalid_response("mutation_not_acknowledged")),
        }
        let mut target = self
            .arguments
            .get("watchlist_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        for echo in [value.get("id"), value.pointer("/watchlist/id")]
            .into_iter()
            .flatten()
        {
            let id = echo
                .as_u64()
                .ok_or_else(|| invalid_response("mutation_id"))?
                .to_string();
            if target.as_ref().is_some_and(|target| *target != id) {
                return Err(invalid_response("mutation_id_mismatch"));
            }
            target = Some(id);
        }
        Ok(target)
    }

    pub fn readback(&self, id: &str) -> Result<Request, AppError> {
        if self.action == Action::Delete {
            Ok(Request::watchlists())
        } else {
            Request::watchlist(id)
        }
    }

    fn assess_readback(&self, id: &str, data: &Value) -> &'static str {
        if self.action == Action::Delete {
            return if data["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["id"] == id)
            {
                "still_present"
            } else {
                // List completeness is unconfirmed; absence is not a deletion receipt.
                "not_reported"
            };
        }
        let row = &data["watchlist"];
        let mut unconfirmed = false;
        for field in ["name", "description"] {
            if let Some(expected) = self.arguments.get(field) {
                if row[field].is_null() {
                    unconfirmed = true;
                } else if row[field] != *expected {
                    return "mismatch";
                }
            }
        }
        if let Some(expected) = self.arguments.get("symbols") {
            let Some(actual) = row["symbols"].as_array() else {
                return "unconfirmed";
            };
            let expected = expected.as_array().unwrap();
            let matches = match self.action {
                Action::Create => actual == expected,
                // The documented add operation moves existing members to the end.
                Action::Add => {
                    actual.ends_with(expected)
                        && expected.iter().all(|symbol| {
                            actual.iter().filter(|value| *value == symbol).count() == 1
                        })
                }
                Action::Remove => expected.iter().all(|symbol| !actual.contains(symbol)),
                _ => true,
            };
            if !matches {
                return "mismatch";
            }
        }
        if unconfirmed {
            "unconfirmed"
        } else {
            "matched"
        }
    }

    pub fn verified_readback(&self, id: &str, data: Value, attempts: u32) -> Value {
        let status = self.assess_readback(id, &data);
        let observation = if self.action == Action::Delete {
            json!({
                "kind": "list_membership",
                "target_reported": status == "still_present",
                "received_at": data["client_observation"]["received_at"],
                "completeness": data["client_observation"]["completeness"]
            })
        } else {
            data
        };
        json!({"status": status, "tool_attempts": attempts, "observation": observation})
    }

    pub fn report(&self, target: Option<&str>, readback: Value) -> Value {
        json!({
            "contract_version": Self::CONTRACT,
            "source": "tradingview_mcp",
            "source_category": "desktop_free_mutation",
            "requires_desktop": false,
            "operation": self.action_name(),
            "request": self.arguments,
            "target_id": target,
            "mutation": {
                "status": "response_received",
                "tool_attempts": 1,
                "automatic_retry": false
            },
            "readback": readback
        })
    }
}

fn name_value(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty()
        || name.chars().count() > WatchlistMutation::MAX_NAME_CHARS
        || name.chars().any(char::is_control)
    {
        return Err(invalid_request("name"));
    }
    Ok(())
}

fn symbols_value(symbols: &[String], allow_empty: bool) -> Result<(), AppError> {
    if (!allow_empty && symbols.is_empty()) || symbols.len() > WatchlistMutation::MAX_SYMBOLS {
        return Err(invalid_request("symbols_count"));
    }
    let mut unique = std::collections::HashSet::new();
    for symbol in symbols {
        crate::mcp_bars::validate_symbol(symbol)?;
        if !unique.insert(symbol) {
            return Err(invalid_request("duplicate_symbol"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(row: Value) -> Value {
        super::super::normalize(
            &Request::watchlist("12").unwrap(),
            json!({"watchlist": row}),
            0,
        )
        .unwrap()
    }

    #[test]
    fn validation_keeps_omission_distinct_from_clearing() {
        assert!(WatchlistMutation::create(" ", &[]).is_err());
        assert!(WatchlistMutation::create(&"名".repeat(501), &[]).is_err());
        assert!(WatchlistMutation::update("12", None, None).is_err());
        assert!(WatchlistMutation::update("12", Some(""), None).is_err());
        let update = WatchlistMutation::update("12", None, Some("")).unwrap();
        assert_eq!(
            update.arguments(),
            json!({"watchlist_id": "12", "description": ""})
        );
        for symbols in [
            vec![],
            vec!["EXAMPLE".into()],
            vec!["NASDAQ:EXAMPLE".into(); 2],
        ] {
            assert!(WatchlistMutation::symbols("12", &symbols, false).is_err());
        }
        assert!(WatchlistMutation::delete("name").is_err());
    }

    #[test]
    fn receipts_never_invent_or_retarget_an_id() {
        let create = WatchlistMutation::create("Example", &[]).unwrap();
        assert_eq!(
            create
                .target_after_reply(&json!({"success": true}))
                .unwrap(),
            None
        );
        assert_eq!(
            create
                .target_after_reply(&json!({"watchlist": {"id": 12}}))
                .unwrap()
                .as_deref(),
            Some("12")
        );
        let delete = WatchlistMutation::delete("12").unwrap();
        assert_eq!(
            delete
                .target_after_reply(&json!({"success": false}))
                .unwrap_err()
                .details
                .unwrap()["code"],
            "provider_error"
        );
        for reply in [
            json!({"id": 10}),
            json!({"id": 12, "watchlist": {"id": 10}}),
            json!({"success": false}),
            json!({"success": "true"}),
            json!("ok"),
        ] {
            assert!(delete.target_after_reply(&reply).is_err());
        }
    }

    #[test]
    fn verification_preserves_unknowns_order_and_absence_limits() {
        let update = WatchlistMutation::update("12", Some("After"), Some("")).unwrap();
        assert_eq!(
            update.assess_readback("12", &snapshot(json!({"id": 12, "name": "After"}))),
            "unconfirmed"
        );
        assert_eq!(
            update.assess_readback("12", &snapshot(json!({"id": 12, "name": "Before"}))),
            "mismatch"
        );
        assert_eq!(
            update.assess_readback(
                "12",
                &snapshot(json!({"id": 12, "name": "After", "description": ""}))
            ),
            "matched"
        );
        let add = WatchlistMutation::symbols("12", &["NASDAQ:EXAMPLE".into()], false).unwrap();
        assert_eq!(
            add.assess_readback(
                "12",
                &snapshot(json!({"id": 12, "symbols": ["NASDAQ:EXAMPLE", "NYSE:OTHER"]}))
            ),
            "mismatch"
        );
        assert_eq!(
            add.assess_readback(
                "12",
                &snapshot(json!({"id": 12, "symbols": ["NYSE:OTHER", "NASDAQ:EXAMPLE"]}))
            ),
            "matched"
        );
        assert_eq!(
            add.assess_readback("12", &snapshot(json!({"id": 12, "symbols": null}))),
            "unconfirmed"
        );
        let remove = WatchlistMutation::symbols("12", &["NASDAQ:EXAMPLE".into()], true).unwrap();
        assert_eq!(
            remove.assess_readback("12", &snapshot(json!({"id": 12, "symbols": []}))),
            "matched"
        );
        let delete = WatchlistMutation::delete("12").unwrap();
        assert_eq!(
            delete.assess_readback("12", &json!({"items": []})),
            "not_reported"
        );
        assert_eq!(
            delete.assess_readback("12", &json!({"items": [{"id": "12"}]})),
            "still_present"
        );
    }
}
