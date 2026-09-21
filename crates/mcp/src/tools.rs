//! Closed tool identities and request validation; never forward arbitrary tools.

use crate::{Failure, Result};
use serde_json::Value;
use tradingview_model::{mcp_account, mcp_bars, mcp_data};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tool {
    Financials,
    FinancialHistory,
    Forecasts,
    Earnings,
    Bars,
    Search,
    Columns,
    Symbol,
    Symbols,
    Screener,
    Watchlists,
    Watchlist,
    Alerts,
    AlertDetails,
    CreateAlert,
    UpdateAlert,
    StopAlerts,
    RestartAlerts,
    DeleteAlerts,
    CreateWatchlist,
    UpdateWatchlist,
    AddWatchlist,
    RemoveWatchlist,
    DeleteWatchlist,
}

impl Tool {
    pub fn names(self) -> &'static [&'static str] {
        match self {
            Self::Financials => &["mcp-tv-get-financials", "get_financials"],
            Self::FinancialHistory => &["mcp-tv-get-financial-history", "get_financial_history"],
            Self::Forecasts => &["mcp-tv-get-forecasts", "get_forecasts"],
            Self::Earnings => &["mcp-tv-get-earnings-calendar", "get_earnings_calendar"],
            Self::CreateAlert => &["mcp-tv-create-alert", "create_alert"],
            Self::UpdateAlert => &["mcp-tv-update-alert", "update_alert"],
            Self::StopAlerts => &["mcp-tv-stop-alerts", "stop_alerts"],
            Self::RestartAlerts => &["mcp-tv-restart-alerts", "restart_alerts"],
            Self::DeleteAlerts => &["mcp-tv-delete-alert", "delete_alert"],
            Self::CreateWatchlist => &["mcp-watchlist-create-watchlist", "create_watchlist"],
            Self::UpdateWatchlist => &["mcp-watchlist-update-watchlist", "update_watchlist"],
            Self::AddWatchlist => &["mcp-watchlist-add-to-watchlist", "add_to_watchlist"],
            Self::RemoveWatchlist => &[
                "mcp-watchlist-remove-from-watchlist",
                "remove_from_watchlist",
            ],
            Self::DeleteWatchlist => &["mcp-watchlist-delete-watchlist", "delete_watchlist"],
            Self::Watchlists => &["mcp-watchlist-list-watchlists", "list_watchlists"],
            Self::Watchlist => &["mcp-watchlist-get-watchlist", "get_watchlist"],
            Self::Alerts => &["mcp-tv-list-alerts", "list_alerts"],
            Self::AlertDetails => &["mcp-tv-get-alerts", "get_alerts"],
            Self::Bars => &["mcp-tv-get-ohlcv", "get_ohlcv"],
            Self::Search => &["mcp-tv-search-symbols", "search_symbols"],
            Self::Columns => &["mcp-tv-get-screener-columns", "get_screener_columns"],
            Self::Symbol => &["mcp-tv-get-symbol-data", "get_symbol_data"],
            Self::Symbols => &["mcp-tv-get-symbol-data-batch", "get_symbol_data_batch"],
            Self::Screener => &["mcp-tv-run-screener", "run_screener"],
        }
    }

    pub fn fields(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Financials => &[
                ("symbol", "string"),
                ("period", "string"),
                ("metric", "array"),
            ],
            Self::FinancialHistory => &[
                ("symbol", "string"),
                ("period", "string"),
                ("date_from", "string"),
                ("date_to", "string"),
            ],
            Self::Forecasts => &[("symbol", "string")],
            Self::Earnings => &[
                ("symbols", "array"),
                ("date_from", "string"),
                ("date_to", "string"),
            ],
            Self::CreateAlert => &[
                ("symbol", "string"),
                ("price", "number"),
                ("condition", "string"),
                ("resolution", "string"),
                ("name", "string"),
                ("auto_deactivate", "boolean"),
                ("email", "boolean"),
                ("mobile_push", "boolean"),
                ("popup", "boolean"),
                ("monitor", "boolean"),
            ],
            Self::UpdateAlert => &[
                ("alert_id", "integer"),
                ("name", "string"),
                ("auto_deactivate", "boolean"),
                ("email", "boolean"),
                ("mobile_push", "boolean"),
                ("popup", "boolean"),
            ],
            Self::StopAlerts | Self::RestartAlerts | Self::DeleteAlerts => {
                &[("alert_ids", "array")]
            }
            Self::CreateWatchlist => &[("name", "string"), ("symbols", "array")],
            Self::UpdateWatchlist => &[
                ("watchlist_id", "string"),
                ("name", "string"),
                ("description", "string"),
            ],
            Self::AddWatchlist | Self::RemoveWatchlist => {
                &[("watchlist_id", "string"), ("symbols", "array")]
            }
            Self::DeleteWatchlist => &[("watchlist_id", "string")],
            Self::Watchlists => &[],
            Self::Watchlist => &[("watchlist_id", "string")],
            Self::Alerts => &[("symbol", "string"), ("active", "boolean")],
            Self::AlertDetails => &[("alert_ids", "array")],
            Self::Bars => &[
                ("symbol", "string"),
                ("interval", "string"),
                ("count", "integer"),
                ("summary", "boolean"),
            ],
            Self::Search => &[("query", "string"), ("type_filter", "string")],
            Self::Columns => &[
                ("market", "string"),
                ("group", "string"),
                ("search", "string"),
            ],
            Self::Symbol => &[("symbol", "string"), ("columns", "array")],
            Self::Symbols => &[("symbols", "array"), ("columns", "array")],
            Self::Screener => &[
                ("market", "string"),
                ("filters", "object"),
                ("sort_by", "string"),
                ("sort_order", "string"),
                ("limit", "integer"),
                ("columns", "array"),
                ("symbol_types", "array"),
                ("filter_preset", "string"),
                ("symbolset", "array"),
            ],
        }
    }

    pub fn from_name(name: &str) -> Result<Self> {
        [
            Self::Financials,
            Self::FinancialHistory,
            Self::Forecasts,
            Self::Earnings,
            Self::Bars,
            Self::Search,
            Self::Columns,
            Self::Symbol,
            Self::Symbols,
            Self::Screener,
            Self::Watchlists,
            Self::Watchlist,
            Self::Alerts,
            Self::AlertDetails,
            Self::CreateAlert,
            Self::UpdateAlert,
            Self::StopAlerts,
            Self::RestartAlerts,
            Self::DeleteAlerts,
            Self::CreateWatchlist,
            Self::UpdateWatchlist,
            Self::AddWatchlist,
            Self::RemoveWatchlist,
            Self::DeleteWatchlist,
        ]
        .into_iter()
        .find(|tool| tool.names().contains(&name))
        .ok_or(Failure::UnsupportedCapability)
    }

    pub fn validate_arguments(self, args: &Value) -> Result<()> {
        let object = args.as_object().ok_or(Failure::UnsupportedCapability)?;
        if object
            .keys()
            .any(|name| !self.fields().iter().any(|(field, _)| field == name))
        {
            return Err(Failure::UnsupportedCapability);
        }
        let string = |name| {
            args.get(name)
                .and_then(Value::as_str)
                .ok_or(Failure::UnsupportedCapability)
        };
        let optional = |name| match args.get(name) {
            None => Ok(None),
            Some(Value::String(value)) => Ok(Some(value.as_str())),
            _ => Err(Failure::UnsupportedCapability),
        };
        let validated = match self {
            Self::Financials => {
                let metrics = args
                    .get("metric")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!([]));
                let metrics: Vec<String> =
                    serde_json::from_value(metrics).map_err(|_| Failure::UnsupportedCapability)?;
                tradingview_model::mcp_financials::Request::snapshot(
                    string("symbol")?,
                    string("period")?,
                    &metrics,
                )
                .map(|r| r.arguments())
            }
            Self::FinancialHistory => tradingview_model::mcp_financials::Request::history(
                string("symbol")?,
                string("period")?,
                optional("date_from")?,
                optional("date_to")?,
            )
            .map(|r| r.arguments()),
            Self::Forecasts => {
                tradingview_model::mcp_financials::Request::forecasts(string("symbol")?)
                    .map(|r| r.arguments())
            }
            Self::Earnings => {
                let symbols: Vec<String> = serde_json::from_value(args["symbols"].clone())
                    .map_err(|_| Failure::UnsupportedCapability)?;
                tradingview_model::mcp_financials::Request::earnings(
                    &symbols,
                    optional("date_from")?,
                    optional("date_to")?,
                )
                .map(|r| r.arguments())
            }
            Self::CreateAlert
            | Self::UpdateAlert
            | Self::StopAlerts
            | Self::RestartAlerts
            | Self::DeleteAlerts => {
                use mcp_account::AlertAction;
                let action = match self {
                    Self::CreateAlert => AlertAction::Create,
                    Self::UpdateAlert => AlertAction::Update,
                    Self::StopAlerts => AlertAction::Stop,
                    Self::RestartAlerts => AlertAction::Restart,
                    Self::DeleteAlerts => AlertAction::Delete,
                    _ => unreachable!(),
                };
                mcp_account::AlertMutation::from_arguments(action, args).map(|v| v.arguments())
            }
            Self::CreateWatchlist | Self::AddWatchlist | Self::RemoveWatchlist => {
                let symbols: Vec<String> = serde_json::from_value(
                    args.get("symbols")
                        .cloned()
                        .ok_or(Failure::UnsupportedCapability)?,
                )
                .map_err(|_| Failure::UnsupportedCapability)?;
                if self == Self::CreateWatchlist {
                    mcp_account::WatchlistMutation::create(string("name")?, &symbols)
                        .map(|v| v.arguments())
                } else {
                    mcp_account::WatchlistMutation::symbols(
                        string("watchlist_id")?,
                        &symbols,
                        self == Self::RemoveWatchlist,
                    )
                    .map(|v| v.arguments())
                }
            }
            Self::UpdateWatchlist => mcp_account::WatchlistMutation::update(
                string("watchlist_id")?,
                optional("name")?,
                optional("description")?,
            )
            .map(|v| v.arguments()),
            Self::DeleteWatchlist => {
                mcp_account::WatchlistMutation::delete(string("watchlist_id")?)
                    .map(|v| v.arguments())
            }
            Self::Watchlists => Ok(mcp_account::Request::watchlists().arguments()),
            Self::Watchlist => {
                mcp_account::Request::watchlist(string("watchlist_id")?).map(|v| v.arguments())
            }
            Self::Alerts => {
                let active = match args.get("active") {
                    None => None,
                    Some(Value::Bool(value)) => Some(*value),
                    _ => return Err(Failure::UnsupportedCapability),
                };
                mcp_account::Request::alerts(optional("symbol")?, active).map(|v| v.arguments())
            }
            Self::AlertDetails => {
                let ids: Vec<u64> = serde_json::from_value(
                    args.get("alert_ids")
                        .cloned()
                        .ok_or(Failure::UnsupportedCapability)?,
                )
                .map_err(|_| Failure::UnsupportedCapability)?;
                mcp_account::Request::alert_details(&ids).map(|v| v.arguments())
            }
            Self::Screener => {
                let options = mcp_data::ScreenerOptions::from_arguments(args)
                    .map_err(|_| Failure::UnsupportedCapability)?;
                mcp_data::Request::screener(options).map(|v| v.arguments())
            }
            Self::Bars => {
                let interval = string("interval")?;
                let count = args
                    .get("count")
                    .and_then(Value::as_u64)
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or(Failure::UnsupportedCapability)?;
                if args.get("summary") != Some(&Value::Bool(false)) {
                    return Err(Failure::UnsupportedCapability);
                }
                mcp_bars::Request::new(
                    string("symbol")?,
                    if interval == "M" { "1M" } else { interval },
                    count,
                )
                .map(|v| v.arguments())
            }
            Self::Search => mcp_data::Request::search(string("query")?, optional("type_filter")?)
                .map(|v| v.arguments()),
            Self::Columns => mcp_data::Request::columns(
                optional("market")?,
                optional("group")?,
                optional("search")?,
            )
            .map(|v| v.arguments()),
            Self::Symbol | Self::Symbols => {
                let columns = match args.get("columns") {
                    Some(value) => serde_json::from_value::<Vec<String>>(value.clone())
                        .map_err(|_| Failure::UnsupportedCapability)?,
                    None => Vec::new(),
                };
                if self == Self::Symbols {
                    let symbols = serde_json::from_value::<Vec<String>>(
                        args.get("symbols")
                            .cloned()
                            .ok_or(Failure::UnsupportedCapability)?,
                    )
                    .map_err(|_| Failure::UnsupportedCapability)?;
                    mcp_data::Request::symbols(&symbols, &columns).map(|v| v.arguments())
                } else {
                    mcp_data::Request::symbol(string("symbol")?, &columns).map(|v| v.arguments())
                }
            }
        }
        .map_err(|_| Failure::UnsupportedCapability)?;
        if validated != *args {
            return Err(Failure::UnsupportedCapability);
        }
        Ok(())
    }
}

impl From<mcp_account::AlertAction> for Tool {
    fn from(action: mcp_account::AlertAction) -> Self {
        use mcp_account::AlertAction;
        match action {
            AlertAction::Create => Self::CreateAlert,
            AlertAction::Update => Self::UpdateAlert,
            AlertAction::Stop => Self::StopAlerts,
            AlertAction::Restart => Self::RestartAlerts,
            AlertAction::Delete => Self::DeleteAlerts,
        }
    }
}

impl From<mcp_data::Kind> for Tool {
    fn from(kind: mcp_data::Kind) -> Self {
        match kind {
            mcp_data::Kind::Search => Self::Search,
            mcp_data::Kind::Columns => Self::Columns,
            mcp_data::Kind::Symbol => Self::Symbol,
            mcp_data::Kind::Symbols => Self::Symbols,
            mcp_data::Kind::Screener => Self::Screener,
        }
    }
}

impl From<mcp_account::Kind> for Tool {
    fn from(kind: mcp_account::Kind) -> Self {
        match kind {
            mcp_account::Kind::Watchlists => Self::Watchlists,
            mcp_account::Kind::Watchlist => Self::Watchlist,
            mcp_account::Kind::Alerts => Self::Alerts,
            mcp_account::Kind::AlertDetails => Self::AlertDetails,
        }
    }
}

impl From<mcp_account::WatchlistAction> for Tool {
    fn from(action: mcp_account::WatchlistAction) -> Self {
        use mcp_account::WatchlistAction;
        match action {
            WatchlistAction::Create => Self::CreateWatchlist,
            WatchlistAction::Update => Self::UpdateWatchlist,
            WatchlistAction::Add => Self::AddWatchlist,
            WatchlistAction::Remove => Self::RemoveWatchlist,
            WatchlistAction::Delete => Self::DeleteWatchlist,
        }
    }
}

impl From<tradingview_model::mcp_financials::Kind> for Tool {
    fn from(kind: tradingview_model::mcp_financials::Kind) -> Self {
        use tradingview_model::mcp_financials::Kind;
        match kind {
            Kind::Snapshot => Self::Financials,
            Kind::History => Self::FinancialHistory,
            Kind::Forecasts => Self::Forecasts,
            Kind::Earnings => Self::Earnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn allowlist_rejects_unsupported_tools_and_unvalidated_arguments() {
        for name in [
            "mcp-watchlist-get-active-watchlist",
            "mcp-tv-get-alerts-log",
            "other-search_symbols",
        ] {
            assert_eq!(Tool::from_name(name), Err(Failure::UnsupportedCapability));
        }
        for (tool, args) in [
            (Tool::Search, json!({"query": "Example", "extra": true})),
            (Tool::Columns, json!({"search": null})),
            (
                Tool::Symbol,
                json!({"symbol": "NASDAQ:EXAMPLE", "columns": ["volume", "volume"]}),
            ),
            (
                Tool::Bars,
                json!({"symbol": "NASDAQ:EXAMPLE", "interval": "1D", "count": 1, "summary": true}),
            ),
        ] {
            assert_eq!(
                tool.validate_arguments(&args),
                Err(Failure::UnsupportedCapability)
            );
        }
    }

    #[test]
    fn alert_mutations_reject_unimplemented_fields_and_implicit_defaults() {
        let request = mcp_account::AlertMutation::create(
            "NASDAQ:EXAMPLE",
            100.0,
            "greater",
            "1D",
            mcp_account::AlertSettings {
                name: Some("Example".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let valid = request.arguments();
        Tool::CreateAlert.validate_arguments(&valid).unwrap();
        for (field, value) in [
            ("monitor", json!(true)),
            ("webhook", json!("https://example.invalid")),
            ("price", Value::Null),
        ] {
            let mut args = valid.clone();
            args[field] = value;
            assert_eq!(
                Tool::CreateAlert.validate_arguments(&args),
                Err(Failure::UnsupportedCapability)
            );
        }
        let mut implicit = valid.clone();
        implicit.as_object_mut().unwrap().remove("popup");
        assert_eq!(
            Tool::CreateAlert.validate_arguments(&implicit),
            Err(Failure::UnsupportedCapability)
        );
        assert_eq!(
            Tool::UpdateAlert.validate_arguments(&json!({"alert_id": 12, "price": 100})),
            Err(Failure::UnsupportedCapability)
        );
        for tool in [Tool::StopAlerts, Tool::RestartAlerts, Tool::DeleteAlerts] {
            assert_eq!(
                tool.validate_arguments(&json!({"alert_ids": [12.5]})),
                Err(Failure::UnsupportedCapability)
            );
        }
    }

    #[test]
    fn documented_and_observed_names_share_validated_requests() {
        for request in [
            mcp_data::Request::search("Example", Some("stock")).unwrap(),
            mcp_data::Request::symbols(
                &["NASDAQ:EXAMPLE".into(), "NYSE:OTHER".into()],
                &["close".into()],
            )
            .unwrap(),
            mcp_data::Request::columns(None, None, Some("volume")).unwrap(),
            mcp_data::Request::symbol("NASDAQ:EXAMPLE", &["close".into()]).unwrap(),
        ] {
            let tool = Tool::from(request.kind());
            for name in tool.names() {
                assert_eq!(Tool::from_name(name).unwrap(), tool);
                tool.validate_arguments(&request.arguments()).unwrap();
            }
        }
    }
}
