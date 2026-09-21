//! Closed tool identities and request validation; never forward arbitrary tools.

use crate::{Failure, Result};
use serde_json::Value;
use tradingview_model::{mcp_account, mcp_bars, mcp_data};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tool {
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
    CreateWatchlist,
    UpdateWatchlist,
    AddWatchlist,
    RemoveWatchlist,
    DeleteWatchlist,
}

impl Tool {
    pub fn names(self) -> &'static [&'static str] {
        match self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn allowlist_rejects_mutations_and_unvalidated_arguments() {
        for name in [
            "create_alert",
            "mcp-watchlist-get-active-watchlist",
            "mcp-tv-stop-alerts",
            "mcp-tv-delete-alert",
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
