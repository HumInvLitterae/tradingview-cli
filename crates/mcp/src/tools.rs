//! Closed tool identities and request validation; never forward arbitrary tools.

use crate::{Failure, Result};
use serde_json::Value;
use tradingview_model::{mcp_bars, mcp_data};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tool {
    Bars,
    Search,
    Columns,
    Symbol,
}

impl Tool {
    pub fn names(self) -> &'static [&'static str] {
        match self {
            Self::Bars => &["mcp-tv-get-ohlcv", "get_ohlcv"],
            Self::Search => &["mcp-tv-search-symbols", "search_symbols"],
            Self::Columns => &["mcp-tv-get-screener-columns", "get_screener_columns"],
            Self::Symbol => &["mcp-tv-get-symbol-data", "get_symbol_data"],
        }
    }

    pub fn fields(self) -> &'static [(&'static str, &'static str)] {
        match self {
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
        }
    }

    pub fn from_name(name: &str) -> Result<Self> {
        [Self::Bars, Self::Search, Self::Columns, Self::Symbol]
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
            Self::Symbol => {
                let columns = match args.get("columns") {
                    Some(value) => serde_json::from_value::<Vec<String>>(value.clone())
                        .map_err(|_| Failure::UnsupportedCapability)?,
                    None => Vec::new(),
                };
                mcp_data::Request::symbol(string("symbol")?, &columns).map(|v| v.arguments())
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
