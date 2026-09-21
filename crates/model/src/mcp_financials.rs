//! Official financial data requests and source-preserving interpretation.

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Snapshot,
    History,
    Forecasts,
    Earnings,
}

#[derive(Clone, Debug)]
pub struct Request {
    kind: Kind,
    arguments: Value,
}

impl Request {
    pub fn snapshot(symbol: &str, period: &str, metrics: &[String]) -> Result<Self, AppError> {
        crate::mcp_bars::validate_symbol(symbol)?;
        if !matches!(period, "fy" | "fq" | "ttm" | "fh" | "current") {
            return Err(invalid("period"));
        }
        if metrics.len() > 50 {
            return Err(invalid("metrics_count"));
        }
        let mut unique = std::collections::HashSet::new();
        for metric in metrics {
            if metric.is_empty()
                || metric.len() > 128
                || !metric
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_|.-".contains(&b))
                || !unique.insert(metric)
            {
                return Err(invalid("metrics"));
            }
        }
        let mut arguments = json!({"symbol": symbol, "period": period});
        if !metrics.is_empty() {
            arguments["metric"] = json!(metrics);
        }
        Ok(Self {
            kind: Kind::Snapshot,
            arguments,
        })
    }

    pub fn history(
        symbol: &str,
        period: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Self, AppError> {
        crate::mcp_bars::validate_symbol(symbol)?;
        if !matches!(period, "fy" | "fq") {
            return Err(invalid("period"));
        }
        let mut arguments = json!({"symbol": symbol, "period": period});
        dates(&mut arguments, from, to)?;
        Ok(Self {
            kind: Kind::History,
            arguments,
        })
    }

    pub fn forecasts(symbol: &str) -> Result<Self, AppError> {
        crate::mcp_bars::validate_symbol(symbol)?;
        Ok(Self {
            kind: Kind::Forecasts,
            arguments: json!({"symbol": symbol}),
        })
    }

    pub fn earnings(
        symbols: &[String],
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Self, AppError> {
        if symbols.is_empty() || symbols.len() > 50 {
            return Err(invalid("symbols_count"));
        }
        let mut unique = std::collections::HashSet::new();
        for symbol in symbols {
            crate::mcp_bars::validate_symbol(symbol)?;
            if !unique.insert(symbol) {
                return Err(invalid("duplicate_symbol"));
            }
        }
        let mut arguments = json!({"symbols": symbols});
        dates(&mut arguments, from, to)?;
        Ok(Self {
            kind: Kind::Earnings,
            arguments,
        })
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn arguments(&self) -> Value {
        self.arguments.clone()
    }
}

fn dates(arguments: &mut Value, from: Option<&str>, to: Option<&str>) -> Result<(), AppError> {
    for (key, date) in [("date_from", from), ("date_to", to)] {
        if let Some(date) = date {
            let bytes = date.as_bytes();
            if bytes.len() != 10
                || bytes[4] != b'-'
                || bytes[7] != b'-'
                || !bytes
                    .iter()
                    .enumerate()
                    .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
            {
                return Err(invalid("date"));
            }
            let year: u32 = date[..4].parse().map_err(|_| invalid("date"))?;
            let month: u32 = date[5..7].parse().map_err(|_| invalid("date"))?;
            let day: u32 = date[8..].parse().map_err(|_| invalid("date"))?;
            let max = match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 if year.is_multiple_of(4)
                    && (!year.is_multiple_of(100) || year.is_multiple_of(400)) =>
                {
                    29
                }
                2 => 28,
                _ => 0,
            };
            if year == 0 || day == 0 || day > max {
                return Err(invalid("date"));
            }
            arguments[key] = json!(date);
        }
    }
    if from.zip(to).is_some_and(|(from, to)| from > to) {
        return Err(invalid("date_order"));
    }
    Ok(())
}

fn invalid(reason: &str) -> AppError {
    AppError::new(ErrorKind::Validation, "Invalid MCP financial request").with_details(json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": "invalid_request",
        "reason": reason,
        "tool_attempts": 0
    }))
}

pub fn normalize(request: &Request, value: Value, received_ms: u64) -> Result<Value, AppError> {
    let root = value.as_object().ok_or_else(|| response_error("wrapper"))?;
    match root.get("success") {
        Some(Value::Bool(false)) => {
            return Err(AppError::new(
                ErrorKind::InternalApiUnavailable,
                "TradingView did not return financial data",
            )
            .with_details(json!({
                "contract_version": "mcp_error.v1",
                "source": "tradingview_mcp",
                "code": "provider_error"
            })));
        }
        None | Some(Value::Bool(true)) => {}
        _ => return Err(response_error("success_flag")),
    }
    let content = if request.kind == Kind::History {
        &value
    } else {
        value
            .get("data")
            .filter(|v| v.is_object())
            .ok_or_else(|| response_error("data"))?
    };
    if value.get("error").is_some_and(|v| !v.is_null())
        || content.get("error").is_some_and(|v| !v.is_null())
    {
        return Err(response_error("provider_error_object"));
    }
    let mut metadata = json!({});
    for field in ["symbol", "name", "period", "currency", "unit", "from", "to"] {
        let inner = optional(content, field, false)?;
        let outer = optional(&value, field, false)?;
        if !inner.is_null() && !outer.is_null() && inner != outer {
            return Err(response_error("metadata_conflict"));
        }
        metadata[field] = if inner.is_null() { outer } else { inner };
    }
    for field in ["scale", "as_of"] {
        let inner = optional_scalar(content, field)?;
        let outer = optional_scalar(&value, field)?;
        if !inner.is_null() && !outer.is_null() && inner != outer {
            return Err(response_error("metadata_conflict"));
        }
        metadata[field] = if inner.is_null() { outer } else { inner };
    }
    // A ticker name alone is not a confirmed exchange-qualified identity.
    if request.kind != Kind::Earnings {
        if let Some(symbol) = metadata["symbol"].as_str()
            && Some(symbol) != request.arguments["symbol"].as_str()
        {
            return Err(response_error("symbol_mismatch"));
        }
        if let Some(period) = metadata["period"].as_str()
            && request.arguments.get("period").is_some()
            && Some(period) != request.arguments["period"].as_str()
        {
            return Err(response_error("period_mismatch"));
        }
    }
    let contract = match request.kind {
        Kind::Snapshot => "mcp_financials.v1",
        Kind::History => "mcp_financial_history.v1",
        Kind::Forecasts => "mcp_forecasts.v1",
        Kind::Earnings => "mcp_earnings.v1",
    };
    let mut output = json!({
        "contract_version": contract,
        "source": "tradingview_mcp",
        "source_category": "desktop_free_read",
        "requires_desktop": false,
        "request": request.arguments,
        "provider_metadata": metadata,
        "client_observation": {
            "received_at": crate::mcp_bars::received_at(received_ms)?,
            "completeness": "unconfirmed"
        },
        "transport": {"status": "succeeded", "tool_attempts": 1}
    });
    if request.kind != Kind::Earnings {
        output["client_observation"]["symbol_status"] = json!(if metadata["symbol"].is_string() {
            "matched"
        } else {
            "unconfirmed"
        });
    }
    match request.kind {
        Kind::Snapshot => {
            let fields = scalar_fields(content)?;
            output["client_observation"]["returned_field_count"] = json!(fields.len());
            output["client_observation"]["null_fields"] = json!(
                fields
                    .iter()
                    .filter(|(_, value)| value.is_null())
                    .map(|(name, _)| name)
                    .collect::<Vec<_>>()
            );
            // Aliases and period rewrites belong to the provider, not a guessed client map.
            output["client_observation"]["metric_selection_status"] = json!("unconfirmed");
            output["fields"] = json!(fields);
        }
        Kind::History => history(content, &mut output)?,
        Kind::Forecasts => forecasts(content, &mut output)?,
        Kind::Earnings => earnings(request, content, &mut output)?,
    }
    Ok(output)
}

fn history(content: &Value, output: &mut Value) -> Result<(), AppError> {
    let labels = content["labels"]
        .as_array()
        .ok_or_else(|| response_error("labels"))?;
    if labels.iter().any(|label| !label.is_string()) {
        return Err(response_error("label_type"));
    }
    let columns = content["series"]
        .as_object()
        .ok_or_else(|| response_error("series"))?;
    if !labels.is_empty() && columns.is_empty() {
        return Err(response_error("missing_series"));
    }
    let mut series = serde_json::Map::new();
    for (metric, rows) in columns {
        field_name(metric)?;
        let rows = rows
            .as_array()
            .ok_or_else(|| response_error("series_rows"))?;
        if rows.len() != labels.len() {
            return Err(response_error("series_alignment"));
        }
        let points = rows
            .iter()
            .map(|point| {
                if point.is_null() {
                    return Ok(Value::Null);
                }
                if !point.is_object() {
                    return Err(response_error("series_point"));
                }
                Ok(json!({
                    "value": optional(point, "value", true)?,
                    "yoy_pct": optional(point, "yoy_pct", true)?
                }))
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        series.insert(metric.clone(), json!(points));
    }
    output["labels"] = json!(labels);
    output["series"] = json!(series);
    output["capex_latest"] = match content.get("capex_latest") {
        None | Some(Value::Null) => Value::Null,
        Some(value) if value.is_object() => json!({
            "last_fiscal_year": optional(value, "last_fiscal_year", true)?,
            "last_quarter": optional(value, "last_quarter", true)?,
            "ttm": optional(value, "ttm", true)?
        }),
        _ => return Err(response_error("capex_latest")),
    };
    output["client_observation"]["returned_period_count"] = json!(labels.len());
    output["client_observation"]["requested_window_coverage"] = json!("unconfirmed");
    Ok(())
}

fn forecasts(content: &Value, output: &mut Value) -> Result<(), AppError> {
    let mut forecast = json!({"price": optional(content, "price", true)?});
    for (group, fields) in [
        (
            "analyst_rating",
            &[
                "score",
                "buy",
                "hold",
                "overweight",
                "underweight",
                "sell",
                "total_analysts",
            ][..],
        ),
        (
            "price_targets",
            &["average", "high", "low", "num_estimates", "upside_pct"][..],
        ),
        (
            "estimates",
            &[
                "eps_next_quarter",
                "eps_next_year",
                "eps_ttm",
                "pe_ratio",
                "revenue_next_quarter",
                "revenue_next_year",
            ][..],
        ),
    ] {
        let value = content
            .get(group)
            .ok_or_else(|| response_error("forecast_group"))?;
        if value.is_null() {
            forecast[group] = Value::Null;
            continue;
        }
        if !value.is_object() {
            return Err(response_error("forecast_group"));
        }
        let mut projected = json!({});
        for field in fields {
            projected[field] = optional(value, field, true)?;
        }
        if group == "analyst_rating" {
            projected["recommendation"] = optional(value, "recommendation", false)?;
        }
        forecast[group] = projected;
    }
    output["forecast"] = forecast;
    Ok(())
}

fn earnings(request: &Request, content: &Value, output: &mut Value) -> Result<(), AppError> {
    let rows = content["earnings"]
        .as_array()
        .ok_or_else(|| response_error("earnings"))?;
    if let Some(count) = content.get("count")
        && count.as_u64() != Some(rows.len() as u64)
    {
        return Err(response_error("count_mismatch"));
    }
    let symbols: Vec<String> = serde_json::from_value(request.arguments["symbols"].clone())
        .map_err(|_| response_error("request_symbols"))?;
    let mut items = Vec::new();
    for row in rows {
        let symbol = row["symbol"]
            .as_str()
            .ok_or_else(|| response_error("earnings_symbol"))?;
        if !symbols.iter().any(|requested| requested == symbol) {
            return Err(response_error("unexpected_symbol"));
        }
        let mut item = json!({"symbol": symbol});
        for field in [
            "name",
            "description",
            "release_date",
            "release_next_date",
            "currency",
            "unit",
        ] {
            item[field] = optional(row, field, false)?;
        }
        for field in ["close", "eps_forecast_next_fq", "revenue_forecast_next_fq"] {
            item[field] = optional(row, field, true)?;
        }
        items.push(item);
    }
    let outcomes: Vec<_> = symbols
        .iter()
        .map(|symbol| {
            let indices: Vec<_> = items
                .iter()
                .enumerate()
                .filter(|(_, row)| row["symbol"] == *symbol)
                .map(|(i, _)| i)
                .collect();
            json!({
                "requested_symbol": symbol,
                "status": if indices.is_empty() { "unreported" } else { "returned" },
                "item_indices": indices
            })
        })
        .collect();
    output["client_observation"]["returned_count"] = json!(items.len());
    output["client_observation"]["requested_window_coverage"] = json!("unconfirmed");
    output["items"] = json!(items);
    output["symbol_results"] = json!(outcomes);
    Ok(())
}

fn scalar_fields(content: &Value) -> Result<serde_json::Map<String, Value>, AppError> {
    let fields = content
        .as_object()
        .ok_or_else(|| response_error("fields"))?;
    for (name, value) in fields {
        field_name(name)?;
        if !(value.is_null() || value.is_string() || value.is_number() || value.is_boolean()) {
            return Err(response_error("field_type"));
        }
    }
    Ok(fields.clone())
}

fn field_name(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_|.-".contains(&b))
    {
        return Err(response_error("field_name"));
    }
    Ok(())
}

fn optional(content: &Value, name: &str, number: bool) -> Result<Value, AppError> {
    match content.get(name) {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(value) if (number && value.is_number()) || (!number && value.is_string()) => {
            Ok(value.clone())
        }
        _ => Err(response_error("field_type")),
    }
}

fn optional_scalar(content: &Value, name: &str) -> Result<Value, AppError> {
    let value = content.get(name).cloned().unwrap_or(Value::Null);
    if value.is_null() || value.is_string() || value.is_number() {
        Ok(value)
    } else {
        Err(response_error("metadata_type"))
    }
}

fn response_error(reason: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "TradingView financial response is invalid",
    )
    .with_details(json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": "invalid_response",
        "reason": reason
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_validate_periods_dates_and_explicit_symbols() {
        for period in ["fy", "fq", "ttm", "fh", "current"] {
            assert!(Request::snapshot("NASDAQ:EXAMPLE", period, &[]).is_ok());
        }
        assert!(Request::history("NASDAQ:EXAMPLE", "ttm", None, None).is_err());
        for date in [
            "2025-02-29",
            "2026-04-31",
            "0000-01-01",
            "2026-13-01",
            "2026-1-01",
            "é026-01-01",
        ] {
            assert!(Request::history("NASDAQ:EXAMPLE", "fq", Some(date), None).is_err());
        }
        assert!(
            Request::history(
                "NASDAQ:EXAMPLE",
                "fq",
                Some("2024-02-29"),
                Some("2024-03-01")
            )
            .is_ok()
        );
        assert!(
            Request::history(
                "NASDAQ:EXAMPLE",
                "fy",
                Some("2026-02-01"),
                Some("2026-01-01")
            )
            .is_err()
        );
        assert!(
            Request::earnings(
                &["NASDAQ:EXAMPLE".into(), "NASDAQ:EXAMPLE".into()],
                None,
                None
            )
            .is_err()
        );
        assert!(Request::snapshot("NASDAQ:EXAMPLE", "fq", &["pe".into(), "pe".into()]).is_err());
    }

    #[test]
    fn snapshot_does_not_invent_currency_identity_or_metric_alias_evidence() {
        let request = Request::snapshot("NASDAQ:EXAMPLE", "fq", &["revenue".into()]).unwrap();
        let out = normalize(
            &request,
            json!({"success": true, "data": {
                "name": "EXAMPLE", "total_revenue_fq": null, "net_income_fq": 0
            }}),
            1000,
        )
        .unwrap();
        assert!(out["fields"]["total_revenue_fq"].is_null());
        assert_eq!(out["fields"]["net_income_fq"], 0);
        assert!(out["provider_metadata"]["currency"].is_null());
        assert!(out["provider_metadata"]["symbol"].is_null());
        assert_eq!(out["client_observation"]["symbol_status"], "unconfirmed");
        assert_eq!(
            out["client_observation"]["metric_selection_status"],
            "unconfirmed"
        );
        for bad in [
            json!({"data": {"total_revenue_fq": []}}),
            json!({"data": {"symbol": "NYSE:OTHER"}}),
            json!({"success": false}),
            json!({"data": {"error": "synthetic-private-message"}}),
        ] {
            let error = normalize(&request, bad, 1000).unwrap_err();
            assert!(!format!("{error:?}").contains("synthetic-private-message"));
        }
    }

    #[test]
    fn history_preserves_labels_yoy_and_alignment_without_date_inference() {
        let request = Request::history("NASDAQ:EXAMPLE", "fq", Some("2025-01-01"), None).unwrap();
        let value = json!({
            "symbol": "NASDAQ:EXAMPLE", "period": "fq",
            "labels": ["FY2025 Q2", "FY2025 Q1"],
            "series": {"revenue": [{"value": 0, "yoy_pct": null}, {"value": null, "yoy_pct": -20}]}
        });
        let out = normalize(&request, value.clone(), 1000).unwrap();
        assert_eq!(out["labels"], value["labels"]);
        assert_eq!(out["series"], value["series"]);
        assert_eq!(out["client_observation"]["returned_period_count"], 2);
        assert_eq!(
            out["client_observation"]["requested_window_coverage"],
            "unconfirmed"
        );
        let mut bad = value.clone();
        bad["series"]["revenue"].as_array_mut().unwrap().pop();
        assert!(normalize(&request, bad, 1000).is_err());
        let mut bad = value;
        bad["period"] = json!("fy");
        assert!(normalize(&request, bad, 1000).is_err());
    }

    #[test]
    fn forecasts_preserve_provider_opinion_and_null_estimates() {
        let request = Request::forecasts("NASDAQ:EXAMPLE").unwrap();
        let out = normalize(
            &request,
            json!({"data": {
                "symbol": "NASDAQ:EXAMPLE", "currency": "EUR", "price": 20,
                "analyst_rating": {"recommendation": "provider-opinion", "buy": 0},
                "price_targets": {"average": 30}, "estimates": {"eps_next_quarter": null}
            }}),
            1000,
        )
        .unwrap();
        assert_eq!(out["provider_metadata"]["currency"], "EUR");
        assert_eq!(
            out["forecast"]["analyst_rating"]["recommendation"],
            "provider-opinion"
        );
        assert!(out["forecast"]["estimates"]["eps_next_quarter"].is_null());
        assert!(normalize(&request, json!({"data": {"estimates": []}}), 1000).is_err());
    }

    #[test]
    fn earnings_preserve_multiple_events_and_unreported_symbols() {
        let request =
            Request::earnings(&["NASDAQ:EXAMPLE".into(), "NYSE:OTHER".into()], None, None).unwrap();
        let value = json!({"data": {"count": 2, "earnings": [
            {"symbol": "NASDAQ:EXAMPLE", "release_date": "2026-02-01", "eps_forecast_next_fq": null},
            {"symbol": "NASDAQ:EXAMPLE", "release_date": "2026-05-01", "eps_forecast_next_fq": 0}
        ]}});
        let out = normalize(&request, value.clone(), 1000).unwrap();
        assert_eq!(out["symbol_results"][0]["item_indices"], json!([0, 1]));
        assert_eq!(out["symbol_results"][1]["status"], "unreported");
        assert!(out["items"][0]["eps_forecast_next_fq"].is_null());
        assert_eq!(out["items"][1]["eps_forecast_next_fq"], 0);
        let mut bad = value;
        bad["data"]["count"] = json!(1);
        assert!(normalize(&request, bad, 1000).is_err());
        assert!(
            normalize(
                &request,
                json!({"data": {"earnings": [{"symbol": "NYSE:UNREQUESTED"}]}}),
                1000
            )
            .is_err()
        );
        let empty = normalize(
            &request,
            json!({"data": {"count": 0, "earnings": []}}),
            1000,
        )
        .unwrap();
        assert_eq!(empty["symbol_results"][0]["status"], "unreported");
        assert_eq!(empty["client_observation"]["completeness"], "unconfirmed");
    }
}
