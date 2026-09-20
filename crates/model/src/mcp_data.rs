//! I/O-free requests and interpretation for official MCP symbol data.

mod batch;
mod screener;
pub use screener::ScreenerOptions;

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Search,
    Columns,
    Symbol,
    Symbols,
    Screener,
}

#[derive(Clone, Debug)]
pub struct Request {
    kind: Kind,
    arguments: Value,
}

impl Request {
    pub fn search(query: &str, type_filter: Option<&str>) -> Result<Self, AppError> {
        text(query, 256, "query")?;
        if type_filter.is_some_and(|v| {
            !matches!(
                v,
                "all" | "stock" | "etf" | "bond" | "forex" | "index" | "futures" | "crypto"
            )
        }) {
            return Err(invalid_request("type_filter"));
        }
        let mut arguments = json!({"query": query});
        if let Some(filter) = type_filter {
            arguments["type_filter"] = json!(filter);
        }
        Ok(Self {
            kind: Kind::Search,
            arguments,
        })
    }

    pub fn columns(
        market: Option<&str>,
        group: Option<&str>,
        search: Option<&str>,
    ) -> Result<Self, AppError> {
        if market.is_some_and(|v| !matches!(v, "all" | "stock" | "etf" | "crypto" | "bond")) {
            return Err(invalid_request("market"));
        }
        let mut arguments = json!({});
        for (name, value) in [("market", market), ("group", group), ("search", search)] {
            if let Some(value) = value {
                text(value, 256, name)?;
                arguments[name] = json!(value);
            }
        }
        Ok(Self {
            kind: Kind::Columns,
            arguments,
        })
    }

    pub fn symbol(symbol: &str, columns: &[String]) -> Result<Self, AppError> {
        crate::mcp_bars::validate_symbol(symbol)?;
        let arguments = json!({"symbol": symbol, "columns": column_arguments(columns)?});
        Ok(Self {
            kind: Kind::Symbol,
            arguments,
        })
    }

    pub fn symbols(symbols: &[String], columns: &[String]) -> Result<Self, AppError> {
        if symbols.is_empty() || symbols.len() > 50 {
            return Err(invalid_request("symbols_count"));
        }
        let mut unique = std::collections::HashSet::new();
        for symbol in symbols {
            crate::mcp_bars::validate_symbol(symbol)?;
            if !unique.insert(symbol) {
                return Err(invalid_request("duplicate_symbol"));
            }
        }
        Ok(Self {
            kind: Kind::Symbols,
            arguments: json!({"symbols": symbols, "columns": column_arguments(columns)?}),
        })
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn arguments(&self) -> Value {
        self.arguments.clone()
    }
}

fn column_arguments(columns: &[String]) -> Result<Value, AppError> {
    if columns.len() > 50 {
        return Err(invalid_request("columns"));
    }
    let mut unique = std::collections::HashSet::new();
    for column in columns {
        text(column, 128, "columns")?;
        if !unique.insert(column) {
            return Err(invalid_request("duplicate_column"));
        }
    }
    Ok(if columns.is_empty() {
        json!([
            "name",
            "description",
            "close",
            "change",
            "change_abs",
            "volume",
            "market_cap_basic",
            "price_earnings_ttm",
            "sector",
            "industry"
        ])
    } else {
        json!(columns)
    })
}

fn text(value: &str, max: usize, field: &str) -> Result<(), AppError> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(invalid_request(field));
    }
    Ok(())
}

fn invalid_request(reason: &str) -> AppError {
    AppError::new(ErrorKind::Validation, "Invalid MCP data request").with_details(json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": "invalid_request",
        "reason": reason,
        "tool_attempts": 0
    }))
}

pub fn normalize(request: &Request, value: Value, received_ms: u64) -> Result<Value, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid_response("unsupported_wrapper"))?;
    match object.get("success") {
        Some(Value::Bool(false)) => {
            return Err(AppError::new(
                ErrorKind::InternalApiUnavailable,
                "TradingView did not return a successful result",
            )
            .with_details(json!({
                "contract_version": "mcp_error.v1",
                "source": "tradingview_mcp",
                "code": "provider_error"
            })));
        }
        Some(Value::Bool(true)) | None => {}
        _ => return Err(invalid_response("invalid_success_flag")),
    }
    let contract = match request.kind {
        Kind::Search => "mcp_search.v1",
        Kind::Columns => "mcp_columns.v1",
        Kind::Symbol => "mcp_symbol.v1",
        Kind::Symbols => "mcp_symbols.v1",
        Kind::Screener => "mcp_screener.v1",
    };
    let mut data = json!({
        "contract_version": contract,
        "source": "tradingview_mcp",
        "source_category": "desktop_free_read",
        "requires_desktop": false,
        "request": request.arguments,
        "client_observation": {"received_at": crate::mcp_bars::received_at(received_ms)?},
        "transport": {"status": "succeeded", "tool_attempts": 1}
    });
    match request.kind {
        Kind::Search => normalize_search(&mut data, &value)?,
        Kind::Columns => normalize_columns(&mut data, &value)?,
        Kind::Symbol => normalize_symbol(&request.arguments, &mut data, &value)?,
        Kind::Symbols => batch::normalize(request, &mut data, &value)?,
        Kind::Screener => screener::normalize(request, &mut data, &value)?,
    }
    Ok(data)
}

fn invalid_response(reason: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "TradingView MCP response is invalid",
    )
    .with_details(json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": "invalid_response",
        "reason": reason
    }))
}

fn optional_string(value: Option<&Value>) -> Result<Value, AppError> {
    match value {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(Value::String(value)) => Ok(json!(value)),
        _ => Err(invalid_response("invalid_text_field")),
    }
}

fn normalize_search(data: &mut Value, value: &Value) -> Result<(), AppError> {
    let value = value
        .get("data")
        .ok_or_else(|| invalid_response("missing_search_data"))?;
    let rows = value
        .get("symbols")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("missing_symbols"))?;
    validate_count(value.get("count"), rows.len())?;
    let mut symbols = Vec::new();
    for row in rows {
        let symbol = row
            .get("symbol")
            .and_then(Value::as_str)
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| invalid_response("missing_symbol"))?;
        symbols.push(json!({
            "symbol": symbol,
            "description": optional_string(row.get("description"))?,
            "type": optional_string(row.get("type"))?,
            "exchange": optional_string(row.get("exchange"))?,
            "logoid": optional_string(row.get("logoid"))?,
            "currency_logoid": optional_string(row.get("currency_logoid"))?
        }));
    }
    data["symbols"] = json!(symbols);
    data["client_observation"]["returned_count"] = json!(rows.len());
    data["client_observation"]["coverage"] = json!("unconfirmed");
    Ok(())
}

fn normalize_columns(data: &mut Value, value: &Value) -> Result<(), AppError> {
    let overview =
        data["request"].get("group").is_none() && data["request"].get("search").is_none();
    let reported_count = optional_count(value.get("count"))?;
    data["provider_observation"] = json!({
        "reported_count": {
            "value": reported_count,
            "evidence": if reported_count.is_null() { "unconfirmed" } else { "provider_response" }
        }
    });
    data["client_observation"]["coverage"] = json!("unconfirmed");
    if overview {
        if value.get("columns").is_some() {
            return Err(invalid_response("unexpected_catalog_mode"));
        }
        let rows = value
            .get("groups")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid_response("missing_groups"))?;
        let mut groups = Vec::new();
        let mut column_count = 0;
        for row in rows {
            let group = row
                .get("group")
                .and_then(Value::as_str)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| invalid_response("missing_group_name"))?;
            let columns = optional_strings(row.get("columns"))?;
            let names = columns
                .as_array()
                .ok_or_else(|| invalid_response("missing_group_columns"))?;
            column_count += names.len();
            groups.push(json!({
                "group": group,
                "columns": columns,
                "reported_count": optional_count(row.get("count"))?
            }));
        }
        data["mode"] = json!("overview");
        data["groups"] = json!(groups);
        data["columns"] = json!([]);
        data["client_observation"]["returned_group_count"] = json!(rows.len());
        data["client_observation"]["returned_column_count"] = json!(column_count);
        return Ok(());
    }
    if value.get("groups").is_some() {
        return Err(invalid_response("unexpected_catalog_mode"));
    }
    let rows = value
        .get("columns")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("missing_columns"))?;
    validate_count(value.get("count"), rows.len())?;
    let mut columns = Vec::new();
    for row in rows {
        let name = row
            .get("name")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| invalid_response("missing_column_name"))?;
        columns.push(json!({
            "name": name,
            "description": optional_string(row.get("description"))?,
            "group": optional_string(row.get("group"))?,
            "markets": optional_strings(row.get("markets"))?,
            "variants": optional_strings(row.get("variants"))?
        }));
    }
    data["mode"] = json!("details");
    data["groups"] = json!([]);
    data["columns"] = json!(columns);
    data["client_observation"]["returned_group_count"] = json!(0);
    data["client_observation"]["returned_column_count"] = json!(rows.len());
    data["client_observation"]["coverage"] = json!("unconfirmed");
    Ok(())
}

fn normalize_symbol(request: &Value, data: &mut Value, value: &Value) -> Result<(), AppError> {
    let symbol = optional_string(value.get("symbol"))?;
    if !symbol.is_null() && symbol != request["symbol"] {
        return Err(invalid_response("symbol_mismatch"));
    }
    let fields = value
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid_response("missing_symbol_data"))?;
    let (output, missing) = select_fields(request.get("columns"), fields)?;
    let unknown = json!({"value": null, "evidence": "unconfirmed"});
    data["fields"] = json!(output);
    data["provider_observation"] = json!({
        "symbol": {"value": symbol, "evidence": if symbol.is_null() { "unconfirmed" } else { "provider_response" }},
        "data_as_of": unknown,
        "delay_seconds": unknown,
        "session": unknown
    });
    data["client_observation"]["identity_match"] = json!(if symbol.is_null() {
        "unconfirmed"
    } else {
        "matched"
    });
    data["client_observation"]["fields_status"] = json!(if missing.is_empty() {
        "present"
    } else {
        "incomplete"
    });
    data["client_observation"]["missing_fields"] = json!(missing);
    Ok(())
}

fn select_fields(
    columns: Option<&Value>,
    fields: &serde_json::Map<String, Value>,
) -> Result<(serde_json::Map<String, Value>, Vec<Value>), AppError> {
    let requested: Vec<String> = match columns {
        Some(columns) => serde_json::from_value(columns.clone())
            .map_err(|_| invalid_response("invalid_columns"))?,
        None => fields.keys().cloned().collect(),
    };
    let mut output = serde_json::Map::new();
    let mut missing = Vec::new();
    for name in requested {
        let field = fields.get(&name);
        output.insert(name.clone(), field.cloned().unwrap_or(Value::Null));
        if field.is_none_or(Value::is_null) {
            missing.push(
                json!({"field": name, "reason": if field.is_none() { "absent" } else { "null" }}),
            );
        }
    }
    Ok((output, missing))
}

fn validate_count(count: Option<&Value>, length: usize) -> Result<(), AppError> {
    if count.is_some_and(|v| v.as_u64() != Some(length as u64)) {
        return Err(invalid_response("count_mismatch"));
    }
    Ok(())
}

fn optional_strings(value: Option<&Value>) -> Result<Value, AppError> {
    match value {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(Value::Array(values)) if values.iter().all(Value::is_string) => Ok(json!(values)),
        _ => Err(invalid_response("invalid_text_array")),
    }
}

fn optional_count(value: Option<&Value>) -> Result<Value, AppError> {
    match value {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(value) if value.as_u64().is_some() => Ok(value.clone()),
        _ => Err(invalid_response("invalid_count")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_reject_invalid_fields_before_io() {
        for query in ["", "  ", "Example\nCorp"] {
            assert!(Request::search(query, None).is_err());
        }
        assert!(Request::search("Example", Some("unknown")).is_err());
        assert!(Request::columns(Some("america"), None, None).is_err());
        assert!(Request::columns(None, Some(""), None).is_err());
        assert!(Request::symbol("EXAMPLE", &[]).is_err());
        assert!(Request::symbol("NASDAQ:EXAMPLE", &["close".into(), "close".into()]).is_err());
        assert!(Request::symbol("NASDAQ:EXAMPLE", &["".into()]).is_err());
    }

    #[test]
    fn batch_requests_bound_distinct_symbols_and_share_column_defaults() {
        assert!(Request::symbols(&[], &[]).is_err());
        assert!(
            Request::symbols(&["NASDAQ:EXAMPLE".into(), "NASDAQ:EXAMPLE".into()], &[]).is_err()
        );
        let symbols: Vec<_> = (0..50)
            .map(|index| format!("NASDAQ:EXAMPLE{index}"))
            .collect();
        let request = Request::symbols(&symbols, &[]).unwrap();
        assert_eq!(request.arguments()["symbols"], json!(symbols));
        assert_eq!(
            request.arguments()["columns"],
            Request::symbol("NASDAQ:EXAMPLE", &[]).unwrap().arguments()["columns"]
        );
        let mut oversized = symbols;
        oversized.push("NASDAQ:EXAMPLE50".into());
        assert!(Request::symbols(&oversized, &[]).is_err());
    }

    #[test]
    fn search_preserves_candidates_without_choosing_or_inventing_coverage() {
        let request = Request::search("Example", None).unwrap();
        let response = json!({
            "success": true,
            "data": {
                "count": 2,
                "symbols": [
                    {"symbol": "NASDAQ:EXAMPLE", "exchange": "NASDAQ"},
                    {"symbol": "NYSE:EXAMPLE", "exchange": "NYSE"}
                ]
            }
        });
        let data = normalize(&request, response, 0).unwrap();
        assert_eq!(data["symbols"].as_array().unwrap().len(), 2);
        assert!(data["symbols"][0]["description"].is_null());
        assert_eq!(data["client_observation"]["coverage"], "unconfirmed");
        assert_eq!(
            data["client_observation"]["received_at"],
            "1970-01-01T00:00:00.000Z"
        );
        let empty = normalize(&request, json!({"data": {"count": 0, "symbols": []}}), 0).unwrap();
        assert_eq!(empty["client_observation"]["returned_count"], 0);
        assert!(normalize(&request, json!({"data": {"count": 1, "symbols": []}}), 0).is_err());
        assert!(
            normalize(
                &request,
                json!({"data": {"symbols": [{"symbol": null}]}}),
                0
            )
            .is_err()
        );
        assert!(
            normalize(
                &request,
                json!({"success": false, "data": {"symbols": []}}),
                0
            )
            .is_err()
        );
    }

    #[test]
    fn catalog_distinguishes_overview_from_details_and_keeps_variants() {
        let overview = Request::columns(None, None, None).unwrap();
        let data = normalize(
            &overview,
            json!({"success": true, "count": 2, "groups": [
                {"group": "market", "count": 2, "columns": ["close", "volume"]}
            ]}),
            0,
        )
        .unwrap();
        assert_eq!(data["mode"], "overview");
        assert_eq!(data["client_observation"]["returned_group_count"], 1);
        assert_eq!(data["client_observation"]["returned_column_count"], 2);
        let details = Request::columns(None, None, Some("volume")).unwrap();
        let data = normalize(
            &details,
            json!({"count": 1, "columns": [
                {"name": "volume", "markets": ["stock"], "variants": ["volume|5"]}
            ]}),
            0,
        )
        .unwrap();
        assert_eq!(data["columns"][0]["variants"][0], "volume|5");
        assert!(normalize(&overview, json!({"columns": []}), 0).is_err());
        assert!(normalize(&details, json!({"groups": []}), 0).is_err());
        assert!(
            normalize(
                &details,
                json!({"columns": [{"name": "volume", "variants": [1]}]}),
                0
            )
            .is_err()
        );
    }

    #[test]
    fn symbol_preserves_zero_null_absence_and_unknown_identity() {
        let request = Request::symbol(
            "NASDAQ:EXAMPLE",
            &["close".into(), "volume".into(), "market_cap_basic".into()],
        )
        .unwrap();
        let data = normalize(
            &request,
            json!({"success": true, "data": {"close": 0, "volume": null, "unexpected": "ignored"}}),
            0,
        )
        .unwrap();
        assert_eq!(data["fields"]["close"], 0);
        assert!(data["fields"]["volume"].is_null());
        assert!(data["fields"].get("unexpected").is_none());
        assert_eq!(
            data["client_observation"]["missing_fields"],
            json!([
                {"field": "volume", "reason": "null"},
                {"field": "market_cap_basic", "reason": "absent"}
            ])
        );
        assert_eq!(data["client_observation"]["identity_match"], "unconfirmed");
        assert!(data["provider_observation"]["symbol"]["value"].is_null());
        assert!(data["provider_observation"]["data_as_of"]["value"].is_null());
        assert!(normalize(&request, json!({"symbol": "NYSE:EXAMPLE", "data": {}}), 0).is_err());
        assert!(normalize(&request, json!({"data": []}), 0).is_err());
        assert!(normalize(&request, json!({"success": "true", "data": {}}), 0).is_err());
    }

    #[test]
    fn symbol_defaults_are_explicit_and_do_not_disguise_missing_fields() {
        let request = Request::symbol("NASDAQ:EXAMPLE", &[]).unwrap();
        assert_eq!(request.arguments()["columns"].as_array().unwrap().len(), 10);
        let data = normalize(&request, json!({"data": {}}), 0).unwrap();
        assert_eq!(data["client_observation"]["fields_status"], "incomplete");
        assert_eq!(
            data["client_observation"]["missing_fields"]
                .as_array()
                .unwrap()
                .len(),
            10
        );
    }
}
