//! Simple price-alert lifecycle; never reconstruct Pine conditions or webhook data.

use super::{Request, invalid_request, invalid_response};
use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Create,
    Update,
    Stop,
    Restart,
    Delete,
}

#[derive(Clone, Debug, Default)]
pub struct AlertSettings {
    pub name: Option<String>,
    pub auto_deactivate: Option<bool>,
    pub email: Option<bool>,
    pub mobile_push: Option<bool>,
    pub popup: Option<bool>,
}

impl AlertSettings {
    fn arguments(&self) -> Result<Value, AppError> {
        let mut value = json!({});
        if let Some(name) = &self.name {
            if name.trim().is_empty()
                || name.chars().count() > AlertMutation::MAX_NAME_CHARS
                || name.chars().any(char::is_control)
            {
                return Err(invalid_request("name"));
            }
            value["name"] = json!(name);
        }
        for (field, setting) in [
            ("auto_deactivate", self.auto_deactivate),
            ("email", self.email),
            ("mobile_push", self.mobile_push),
            ("popup", self.popup),
        ] {
            if let Some(setting) = setting {
                value[field] = json!(setting);
            }
        }
        Ok(value)
    }

    fn from_arguments(value: &Value) -> Result<Self, AppError> {
        let name = match value.get("name") {
            None => None,
            Some(Value::String(name)) => Some(name.clone()),
            _ => return Err(invalid_request("name")),
        };
        let boolean = |field| match value.get(field) {
            None => Ok(None),
            Some(Value::Bool(value)) => Ok(Some(*value)),
            _ => Err(invalid_request("notification_setting")),
        };
        Ok(Self {
            name,
            auto_deactivate: boolean("auto_deactivate")?,
            email: boolean("email")?,
            mobile_push: boolean("mobile_push")?,
            popup: boolean("popup")?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct AlertMutation {
    action: Action,
    arguments: Value,
}

impl AlertMutation {
    pub const MAX_NAME_CHARS: usize = 300;
    pub const CONDITIONS: &[&str] = &["cross", "cross_up", "cross_down", "greater", "less"];
    pub const RESOLUTIONS: &[&str] = &["1", "5", "15", "30", "60", "240", "1D", "1W", "1M"];
    pub const CONTRACT: &str = "mcp_alert_mutation.v1";

    pub fn create(
        symbol: &str,
        price: f64,
        condition: &str,
        resolution: &str,
        settings: AlertSettings,
    ) -> Result<Self, AppError> {
        crate::mcp_bars::validate_symbol(symbol)?;
        if !price.is_finite() {
            return Err(invalid_request("price"));
        }
        if !Self::CONDITIONS.contains(&condition) {
            return Err(invalid_request("condition"));
        }
        if !Self::RESOLUTIONS.contains(&resolution) {
            return Err(invalid_request("resolution"));
        }
        if settings.name.is_none() {
            return Err(invalid_request("name"));
        }

        let mut arguments = settings.arguments()?;
        for field in ["auto_deactivate", "email", "mobile_push", "popup"] {
            if arguments.get(field).is_none() {
                arguments[field] = json!(false);
            }
        }
        arguments["symbol"] = json!(symbol);
        arguments["price"] = json!(price);
        arguments["condition"] = json!(condition);
        arguments["resolution"] = json!(resolution);
        arguments["monitor"] = json!(false);
        Ok(Self {
            action: Action::Create,
            arguments,
        })
    }

    pub fn update(id: u64, settings: AlertSettings) -> Result<Self, AppError> {
        Request::alert_details(&[id])?;

        let mut arguments = settings.arguments()?;
        if arguments.as_object().unwrap().is_empty() {
            return Err(invalid_request("empty_update"));
        }
        arguments["alert_id"] = json!(id);
        Ok(Self {
            action: Action::Update,
            arguments,
        })
    }

    pub fn state(action: Action, ids: &[u64]) -> Result<Self, AppError> {
        if !matches!(action, Action::Stop | Action::Restart | Action::Delete) {
            return Err(invalid_request("operation"));
        }
        Ok(Self {
            action,
            arguments: Request::alert_details(ids)?.arguments(),
        })
    }

    pub fn from_arguments(action: Action, args: &Value) -> Result<Self, AppError> {
        let string = |field| {
            args.get(field)
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_request(field))
        };
        match action {
            Action::Create => {
                if args.get("monitor") != Some(&Value::Bool(false)) {
                    return Err(invalid_request("monitor"));
                }
                Self::create(
                    string("symbol")?,
                    args.get("price")
                        .and_then(Value::as_f64)
                        .ok_or_else(|| invalid_request("price"))?,
                    string("condition")?,
                    string("resolution")?,
                    AlertSettings::from_arguments(args)?,
                )
            }
            Action::Update => Self::update(
                args.get("alert_id")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| invalid_request("alert_id"))?,
                AlertSettings::from_arguments(args)?,
            ),
            _ => Self::state(
                action,
                &serde_json::from_value::<Vec<u64>>(args["alert_ids"].clone())
                    .map_err(|_| invalid_request("alert_ids"))?,
            ),
        }
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
            Action::Stop => "stop",
            Action::Restart => "restart",
            Action::Delete => "delete",
        }
    }

    pub fn targets_after_reply(&self, value: &Value) -> Result<Vec<u64>, AppError> {
        if !value.is_object() {
            return Err(invalid_response("mutation_reply"));
        }
        match value.get("success") {
            Some(Value::Bool(false)) => {
                return Err(AppError::new(
                    ErrorKind::InternalApiUnavailable,
                    "TradingView did not acknowledge the alert change",
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

        let mut expected = match self.action {
            Action::Create => Vec::new(),
            Action::Update => vec![self.arguments["alert_id"].as_u64().unwrap()],
            _ => serde_json::from_value(self.arguments["alert_ids"].clone()).unwrap(),
        };

        let mut echoes = Vec::new();
        for value in [value.get("alert_id"), value.pointer("/alert/alert_id")]
            .into_iter()
            .flatten()
        {
            echoes.push(value);
        }
        if let Some(ids) = value.get("alert_ids") {
            echoes.extend(
                ids.as_array()
                    .ok_or_else(|| invalid_response("mutation_ids"))?,
            );
        }
        if let Some(rows) = value.get("alerts") {
            for row in rows
                .as_array()
                .ok_or_else(|| invalid_response("mutation_alerts"))?
            {
                echoes.push(
                    row.get("alert_id")
                        .ok_or_else(|| invalid_response("mutation_id"))?,
                );
            }
        }

        for echo in echoes {
            let id = echo
                .as_u64()
                .ok_or_else(|| invalid_response("mutation_id"))?;
            Request::alert_details(&[id]).map_err(|_| invalid_response("mutation_id"))?;
            if self.action == Action::Create && expected.is_empty() {
                expected.push(id);
            } else if !expected.contains(&id) {
                return Err(invalid_response("mutation_id_mismatch"));
            }
        }
        Ok(expected)
    }

    pub fn readback(&self, ids: &[u64]) -> Result<Request, AppError> {
        if self.action == Action::Delete {
            Request::alerts(None, None)
        } else {
            Request::alert_details(ids)
        }
    }

    pub fn verified_readback(&self, ids: &[u64], data: Value, attempts: u32) -> Value {
        let mut items = Vec::new();
        for id in ids {
            let row = if self.action == Action::Delete {
                data["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|row| row["alert_id"] == *id)
            } else {
                data["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|item| item["requested_id"] == *id && item["status"] == "returned")
                    .map(|item| &item["alert"])
            };
            let status = if self.action == Action::Delete {
                if row.is_some() {
                    "still_present"
                } else {
                    "not_reported"
                }
            } else if let Some(row) = row {
                self.assess(row)
            } else {
                "unreported"
            };
            items.push(json!({
                "requested_id": id,
                "status": status,
                "alert": if self.action == Action::Delete { None } else { row }
            }));
        }

        let status = if items.iter().all(|item| item["status"] == "matched") {
            "matched"
        } else if self.action == Action::Delete
            && items.iter().all(|item| item["status"] == "not_reported")
        {
            "not_reported"
        } else if items
            .iter()
            .any(|item| item["status"] == "mismatch" || item["status"] == "still_present")
        {
            "mismatch"
        } else {
            "unconfirmed"
        };
        json!({
            "status": status,
            "tool_attempts": attempts,
            "items": items,
            "received_at": data["client_observation"]["received_at"],
            "completeness": data["client_observation"]["completeness"]
        })
    }

    fn assess(&self, row: &Value) -> &'static str {
        let mut expected = self.arguments.clone();
        expected["active"] = json!(self.action != Action::Stop);
        let mut unknown = false;
        for field in [
            "active",
            "name",
            "auto_deactivate",
            "email",
            "mobile_push",
            "popup",
            "symbol",
            "resolution",
        ] {
            if let Some(value) = expected.get(field) {
                if row[field].is_null() {
                    unknown = true;
                } else if row[field] != *value {
                    return "mismatch";
                }
            }
        }

        if self.action == Action::Create {
            let single = row["conditions"]
                .as_array()
                .filter(|rows| rows.len() == 1)
                .and_then(|rows| rows.first());
            let condition = row
                .get("condition_type")
                .filter(|value| !value.is_null())
                .or_else(|| single.and_then(|row| row.get("type")));
            let values: Vec<_> = single
                .and_then(|row| row["series"].as_array())
                .into_iter()
                .flatten()
                .filter(|entry| entry["type"] == "value")
                .filter_map(|entry| entry["value"].as_f64())
                .collect();
            let price = row["threshold"].as_f64().or_else(|| {
                if values.len() == 1 {
                    Some(values[0])
                } else {
                    None
                }
            });
            match condition.filter(|value| !value.is_null()) {
                Some(value) if *value != self.arguments["condition"] => return "mismatch",
                None => unknown = true,
                _ => {}
            }
            match price {
                Some(price) if Some(price) != self.arguments["price"].as_f64() => return "mismatch",
                None => unknown = true,
                _ => {}
            }
        }
        if unknown { "unconfirmed" } else { "matched" }
    }

    pub fn report(&self, ids: &[u64], readback: Value) -> Value {
        json!({
            "contract_version": Self::CONTRACT,
            "source": "tradingview_mcp",
            "source_category": "desktop_free_mutation",
            "requires_desktop": false,
            "operation": self.action_name(),
            "request": self.arguments,
            "target_ids": ids,
            "effects": {
                "reactivates": matches!(self.action, Action::Update | Action::Restart),
                "deletes_fire_history": self.action == Action::Delete
            },
            "mutation": {
                "status": "response_received",
                "tool_attempts": 1,
                "automatic_retry": false
            },
            "readback": readback
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create() -> AlertMutation {
        AlertMutation::create(
            "NASDAQ:EXAMPLE",
            100.0,
            "greater",
            "1D",
            AlertSettings {
                name: Some("Example".into()),
                ..Default::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn notification_defaults_false_and_updates_preserve_omission() {
        let request = create();
        for field in [
            "email",
            "mobile_push",
            "popup",
            "monitor",
            "auto_deactivate",
        ] {
            assert_eq!(request.arguments()[field], false);
        }
        let update = AlertMutation::update(
            12,
            AlertSettings {
                email: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(update.arguments(), json!({"alert_id": 12, "email": false}));
        assert_eq!(
            update.report(&[12], Value::Null)["effects"]["reactivates"],
            true
        );
        assert!(AlertMutation::update(12, AlertSettings::default()).is_err());
        assert!(AlertMutation::state(Action::Stop, &[12, 12]).is_err());
        assert!(AlertMutation::state(Action::Delete, &[0]).is_err());
        assert!(
            AlertMutation::create(
                "NASDAQ:EXAMPLE",
                f64::NAN,
                "cross",
                "1D",
                AlertSettings::default()
            )
            .is_err()
        );
    }

    #[test]
    fn mutation_echoes_never_guess_or_expand_targets() {
        assert_eq!(
            create()
                .targets_after_reply(&json!({"success": true}))
                .unwrap(),
            Vec::<u64>::new()
        );
        assert_eq!(
            create()
                .targets_after_reply(&json!({"alert_id": 12}))
                .unwrap(),
            [12]
        );
        assert!(
            create()
                .targets_after_reply(&json!({"alert_id": 12, "alert": {"alert_id": 13}}))
                .is_err()
        );
        let stop = AlertMutation::state(Action::Stop, &[12, 13]).unwrap();
        assert_eq!(
            stop.targets_after_reply(&json!({"alert_ids": [12]}))
                .unwrap(),
            [12, 13]
        );
        assert!(
            stop.targets_after_reply(&json!({"alert_ids": [14]}))
                .is_err()
        );
        assert!(
            stop.targets_after_reply(&json!({"success": false}))
                .is_err()
        );
    }

    #[test]
    fn batch_readback_distinguishes_contradiction_unknown_and_unreported() {
        let stop = AlertMutation::state(Action::Stop, &[12, 13, 14, 15]).unwrap();
        let read = stop.readback(&[12, 13, 14, 15]).unwrap();
        let data = super::super::normalize(&read, json!({"alerts": [
            {"alert_id": 14}, {"alert_id": 13, "active": true}, {"alert_id": 12, "active": false}
        ]}), 1000).unwrap();
        let report = stop.verified_readback(&[12, 13, 14, 15], data, 1);
        assert_eq!(report["status"], "mismatch");
        for (index, status) in ["matched", "mismatch", "unconfirmed", "unreported"]
            .iter()
            .enumerate()
        {
            assert_eq!(report["items"][index]["status"], *status);
        }
        let delete = AlertMutation::state(Action::Delete, &[12]).unwrap();
        let data = super::super::normalize(
            &delete.readback(&[12]).unwrap(),
            json!({"alerts": [{"alert_id": 13}]}),
            1000,
        )
        .unwrap();
        let report = delete.verified_readback(&[12], data, 1);
        assert_eq!(report["status"], "not_reported");
        assert_eq!(report["completeness"], "unconfirmed");
        assert!(!report.to_string().contains("13"));
    }

    #[test]
    fn create_requires_observed_conditions_and_notifications_for_matching() {
        let request = create();
        let mut row = json!({
            "symbol": "NASDAQ:EXAMPLE", "name": "Example", "active": true,
            "auto_deactivate": false, "email": false, "mobile_push": false, "popup": false,
            "resolution": "1D", "condition_type": "greater", "threshold": 100.0
        });
        assert_eq!(request.assess(&row), "matched");
        row["email"] = Value::Null;
        assert_eq!(request.assess(&row), "unconfirmed");
        row["email"] = json!(true);
        assert_eq!(request.assess(&row), "mismatch");
        row["email"] = json!(false);
        row["threshold"] = Value::Null;
        assert_eq!(request.assess(&row), "unconfirmed");
    }
}
