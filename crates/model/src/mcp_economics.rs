//! Official economic catalog, releases and calendar contracts.

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Symbols,
    Series,
    Calendar,
    Dividends,
}

#[derive(Clone, Debug)]
pub struct Request {
    kind: Kind,
    arguments: Value,
}

#[derive(Clone, Debug, Default)]
pub struct CalendarOptions {
    pub countries: Option<String>,
    pub currencies: Option<String>,
    pub category: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub min_importance: Option<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct DividendOptions {
    pub symbols: Vec<String>,
    pub market: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<u32>,
}

impl Request {
    pub fn symbols(
        country: Option<&str>,
        category: Option<&str>,
        search: Option<&str>,
    ) -> Result<Self, AppError> {
        let mut arguments = json!({});
        if let Some(country) = country {
            codes(country, 2, false)?;
            arguments["country"] = json!(country);
        }
        if let Some(category) = category {
            if !matches!(
                category,
                "gdp"
                    | "lbr"
                    | "prce"
                    | "hlth"
                    | "mny"
                    | "trd"
                    | "gov"
                    | "bsnss"
                    | "cnsm"
                    | "hse"
                    | "txs"
                    | "enrg"
                    | "clmt"
            ) {
                return Err(invalid("category"));
            }
            arguments["category"] = json!(category);
        }
        if let Some(search) = search {
            text(search, 256)?;
            arguments["search"] = json!(search);
        }
        Ok(Self {
            kind: Kind::Symbols,
            arguments,
        })
    }

    pub fn series(symbol: &str, from: Option<&str>, to: Option<&str>) -> Result<Self, AppError> {
        economic_symbol(symbol).map_err(|_| invalid("economic_symbol"))?;
        let mut arguments = json!({"symbol": symbol});
        window(&mut arguments, from, to, false)?;
        Ok(Self {
            kind: Kind::Series,
            arguments,
        })
    }

    pub fn calendar(options: CalendarOptions) -> Result<Self, AppError> {
        let countries = options.countries.as_deref().unwrap_or("US");
        codes(countries, 2, true)?;
        let importance = options.min_importance.unwrap_or(-1);
        if !(-1..=1).contains(&importance) {
            return Err(invalid("min_importance"));
        }
        let mut arguments = json!({"countries": countries, "min_importance": importance});
        if let Some(currencies) = options.currencies {
            codes(&currencies, 3, true)?;
            arguments["currencies"] = json!(currencies);
        }
        if let Some(category) = options.category {
            if !matches!(
                category.as_str(),
                "all"
                    | "gdp"
                    | "bonds"
                    | "business"
                    | "consumer"
                    | "goverment"
                    | "health"
                    | "housing"
                    | "labor"
                    | "money"
                    | "prices"
                    | "trade"
                    | "taxes"
            ) {
                return Err(invalid("category"));
            }
            arguments["category"] = json!(category);
        }
        window(
            &mut arguments,
            options.from.as_deref(),
            options.to.as_deref(),
            true,
        )?;
        Ok(Self {
            kind: Kind::Calendar,
            arguments,
        })
    }

    pub fn dividends(options: DividendOptions) -> Result<Self, AppError> {
        let mut arguments = json!({});
        if !options.symbols.is_empty() {
            if options.market.is_some()
                || options.from.is_some()
                || options.to.is_some()
                || options.limit.is_some()
            {
                return Err(invalid("mixed_dividend_modes"));
            }
            if options.symbols.len() > 50 {
                return Err(invalid("symbols_count"));
            }
            let mut seen = std::collections::HashSet::new();
            for symbol in &options.symbols {
                crate::mcp_bars::validate_symbol(symbol)?;
                if !seen.insert(symbol) {
                    return Err(invalid("duplicate_symbol"));
                }
            }
            arguments["symbols"] = json!(options.symbols);
        } else {
            let market = options.market.ok_or_else(|| invalid("dividend_mode"))?;
            if market.is_empty()
                || market.len() > 64
                || !market.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
            {
                return Err(invalid("market"));
            }
            let limit = options.limit.unwrap_or(50);
            if !(1..=200).contains(&limit) {
                return Err(invalid("limit"));
            }
            arguments["market"] = json!(market);
            arguments["limit"] = json!(limit);
            window(
                &mut arguments,
                options.from.as_deref(),
                options.to.as_deref(),
                false,
            )?;
        }
        Ok(Self {
            kind: Kind::Dividends,
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

fn window(
    arguments: &mut Value,
    from: Option<&str>,
    to: Option<&str>,
    allow_time: bool,
) -> Result<(), AppError> {
    for (field, date) in [("date_from", from), ("date_to", to)] {
        if let Some(date) = date {
            if !crate::mcp_dates::iso_date(date)
                && !(allow_time && crate::mcp_dates::utc_seconds(date))
            {
                return Err(invalid("date"));
            }
            arguments[field] = json!(date);
        }
    }
    if let Some((from, to)) = from.zip(to) {
        let reversed = if from.len() == to.len() {
            from > to
        } else {
            from[..10] > to[..10]
        };
        if reversed {
            return Err(invalid("date_order"));
        }
    }
    Ok(())
}

fn economic_symbol(value: &str) -> Result<(), AppError> {
    let suffix = value
        .strip_prefix("ECONOMICS:")
        .ok_or_else(|| invalid("economic_symbol"))?;
    if suffix.is_empty()
        || suffix.len() > 128
        || !suffix
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    {
        return Err(invalid("economic_symbol"));
    }
    Ok(())
}

fn codes(value: &str, width: usize, multiple: bool) -> Result<(), AppError> {
    let parts: Vec<_> = value.split(',').collect();
    let mut seen = std::collections::HashSet::new();
    if parts.len() > if multiple { 50 } else { 1 }
        || parts.iter().any(|p| {
            p.len() != width || !p.bytes().all(|b| b.is_ascii_uppercase()) || !seen.insert(*p)
        })
    {
        return Err(invalid("country_or_currency"));
    }
    Ok(())
}

fn text(value: &str, max: usize) -> Result<(), AppError> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(invalid("search"));
    }
    Ok(())
}

fn invalid(reason: &str) -> AppError {
    AppError::new(ErrorKind::Validation, "Invalid MCP economic request").with_details(json!({
        "contract_version": "mcp_error.v1", "source": "tradingview_mcp",
        "code": "invalid_request", "reason": reason, "tool_attempts": 0
    }))
}

pub fn normalize(request: &Request, value: Value, received_ms: u64) -> Result<Value, AppError> {
    if !value.is_object() {
        return Err(response_error("wrapper"));
    }
    if value.get("success") == Some(&json!(false)) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView did not return economic data",
        )
        .with_details(json!({
            "contract_version": "mcp_error.v1", "source": "tradingview_mcp",
            "code": "provider_error"
        })));
    }
    if value.get("success").is_some_and(|v| v != true)
        || value.get("error").is_some_and(|v| !v.is_null())
    {
        return Err(response_error("provider_status"));
    }
    let contract = match request.kind {
        Kind::Symbols => "mcp_economic_symbols.v1",
        Kind::Series => "mcp_economic_data.v1",
        Kind::Calendar => "mcp_economic_calendar.v1",
        Kind::Dividends => "mcp_dividends.v1",
    };
    let mut output = json!({
        "contract_version": contract, "source": "tradingview_mcp",
        "source_category": "desktop_free_read", "requires_desktop": false,
        "request": request.arguments,
        "client_observation": {
            "received_at": crate::mcp_bars::received_at(received_ms)?,
            "completeness": "unconfirmed"
        },
        "transport": {"status": "succeeded", "tool_attempts": 1}
    });
    match request.kind {
        Kind::Symbols => normalize_symbols(request, &value, &mut output)?,
        Kind::Series => normalize_series(request, &value, &mut output)?,
        Kind::Calendar => normalize_calendar(&value, &mut output)?,
        Kind::Dividends => normalize_dividends(request, &value, &mut output)?,
    }
    Ok(output)
}

fn normalize_symbols(request: &Request, value: &Value, output: &mut Value) -> Result<(), AppError> {
    if request.arguments.as_object().unwrap().is_empty() {
        let categories = rows(value, "categories")?;
        let countries = rows(value, "countries")?;
        if countries.iter().any(|v| !v.is_string()) {
            return Err(response_error("countries"));
        }
        let mut items = Vec::new();
        for row in categories {
            items.push(json!({
                "id": required_text(row, "id")?, "name": required_text(row, "name")?,
                "count": uint(row, "count")?
            }));
        }
        output["mode"] = json!("overview");
        output["categories"] = json!(items);
        output["countries"] = json!(countries);
        output["indicators_count"] = uint(value, "indicators_count")?;
        output["ticker_format"] = optional(value, "ticker_format", Value::is_string)?;
        return Ok(());
    }
    if request.arguments.get("country").is_none() {
        let entries = rows(value, "indicators")?;
        check_count(value, entries.len())?;
        let mut items = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for row in entries {
            let code = required_text(row, "code")?;
            if !seen.insert(code) {
                return Err(response_error("duplicate_code"));
            }
            items.push(json!({
                "code": code,
                "name": optional(row, "name", Value::is_string)?,
                "category": optional(row, "category", Value::is_string)?
            }));
        }
        output["mode"] = json!("indicators");
        output["items"] = json!(items);
        output["client_observation"]["returned_count"] = json!(entries.len());
        return Ok(());
    }
    let entries = rows(value, "symbols")?;
    check_count(value, entries.len())?;
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in entries {
        let symbol = required_text(row, "symbol")?;
        if !seen.insert(symbol) {
            return Err(response_error("duplicate_symbol"));
        }
        if request.arguments.get("country").is_some() {
            economic_symbol(symbol).map_err(|_| response_error("economic_symbol"))?;
        }
        items.push(json!({
            "symbol": symbol, "description": optional(row, "description", Value::is_string)?,
            "category": optional(row, "category", Value::is_string)?
        }));
    }
    let country = match value.get("country") {
        None | Some(Value::Null) => Value::Null,
        Some(country) if country.is_object() => {
            let code = required_text(country, "code")?;
            if request.arguments.get("country").is_some_and(|v| v != code) {
                return Err(response_error("country_mismatch"));
            }
            json!({"code": code, "name": optional(country, "name", Value::is_string)?})
        }
        _ => return Err(response_error("country")),
    };
    output["mode"] = json!("symbols");
    output["country"] = country;
    output["items"] = json!(items);
    output["client_observation"]["returned_count"] = json!(entries.len());
    Ok(())
}

fn normalize_series(request: &Request, value: &Value, output: &mut Value) -> Result<(), AppError> {
    let symbol = required_text(value, "symbol")?;
    if request.arguments["symbol"] != symbol {
        return Err(response_error("symbol_mismatch"));
    }
    let entries = rows(value, "series")?;
    check_count(value, entries.len())?;
    let mut series = Vec::new();
    let mut dates = std::collections::BTreeSet::new();
    for row in entries {
        let date = required_text(row, "date")?;
        if !crate::mcp_dates::iso_date(date) || !dates.insert(date) {
            return Err(response_error("series_date"));
        }
        if row.get("value").is_none() {
            return Err(response_error("series_value"));
        }
        series.push(json!({"date": date, "value": optional(row, "value", Value::is_number)?}));
    }
    output["symbol"] = json!(symbol);
    output["series"] = json!(series);
    output["provider_metadata"] = json!({
        "description": optional(value, "description", Value::is_string)?,
        "unit": optional(value, "unit", Value::is_string)?,
        "scale": optional(value, "scale", Value::is_number)?,
        "notice": optional(value, "notice", Value::is_string)?
    });
    output["client_observation"]["returned_count"] = json!(entries.len());
    output["client_observation"]["actual_range"] =
        json!({"from": dates.first(), "to": dates.last()});
    output["client_observation"]["requested_window_coverage"] = json!("unconfirmed");
    Ok(())
}

fn normalize_calendar(value: &Value, output: &mut Value) -> Result<(), AppError> {
    let status = required_text(value, "status")?;
    if status != "ok" {
        return Err(response_error("calendar_status"));
    }
    // A nonempty result is still not evidence of a complete requested window.
    let entries = rows(value, "result")?;
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in entries {
        let id = required_text(row, "id")?;
        if !seen.insert(id) {
            return Err(response_error("duplicate_event"));
        }
        let mut item = json!({"id": id, "date": required_text(row, "date")?});
        for field in [
            "title",
            "country",
            "currency",
            "category",
            "indicator",
            "ticker",
            "period",
            "referenceDate",
            "scale",
            "source",
            "source_url",
            "comment",
        ] {
            item[field] = optional(row, field, Value::is_string)?;
        }
        for field in [
            "actual",
            "actualRaw",
            "forecast",
            "forecastRaw",
            "previous",
            "previousRaw",
            "importance",
        ] {
            item[field] = optional(row, field, Value::is_number)?;
        }
        items.push(item);
    }
    output["provider_status"] = json!(status);
    output["items"] = json!(items);
    output["client_observation"]["returned_count"] = json!(entries.len());
    output["client_observation"]["requested_window_coverage"] = json!("unconfirmed");
    Ok(())
}

fn normalize_dividends(
    request: &Request,
    value: &Value,
    output: &mut Value,
) -> Result<(), AppError> {
    let entries = rows(value, "data")?;
    check_count(value, entries.len())?;
    if request.arguments["limit"]
        .as_u64()
        .is_some_and(|n| entries.len() as u64 > n)
    {
        return Err(response_error("limit_mismatch"));
    }
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in entries {
        let symbol = required_text(row, "symbol")?;
        crate::mcp_bars::validate_symbol(symbol).map_err(|_| response_error("symbol"))?;
        if !seen.insert(symbol) {
            return Err(response_error("duplicate_symbol"));
        }
        if request.arguments["symbols"]
            .as_array()
            .is_some_and(|symbols| !symbols.iter().any(|v| v == symbol))
        {
            return Err(response_error("unexpected_symbol"));
        }
        let mut item = json!({"symbol": symbol});
        for field in [
            "name",
            "description",
            "currency",
            "dividend_ex_date_recent",
            "dividend_ex_date_upcoming",
            "dividend_payment_date_recent",
            "dividend_payment_date_upcoming",
        ] {
            item[field] = optional(row, field, Value::is_string)?;
        }
        for field in [
            "dividend_amount",
            "dividend_amount_recent",
            "dividend_amount_upcoming",
            "dividends_yield",
        ] {
            item[field] = optional(row, field, Value::is_number)?;
        }
        items.push(item);
    }
    if let Some(symbols) = request.arguments["symbols"].as_array() {
        output["mode"] = json!("symbols");
        let outcomes: Vec<_> = symbols
            .iter()
            .map(|symbol| {
                let status = if seen.contains(symbol.as_str().unwrap()) {
                    "returned"
                } else {
                    "unreported"
                };
                json!({"requested_symbol": symbol, "status": status})
            })
            .collect();
        output["outcomes"] = json!(outcomes);
    } else {
        output["mode"] = json!("market");
        output["client_observation"]["requested_window_coverage"] = json!("unconfirmed");
    }
    output["items"] = json!(items);
    output["client_observation"]["returned_count"] = json!(entries.len());
    Ok(())
}

fn rows<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>, AppError> {
    let rows = value[field]
        .as_array()
        .ok_or_else(|| response_error("rows"))?;
    if field != "countries" && rows.iter().any(|v| !v.is_object()) {
        return Err(response_error("row_type"));
    }
    Ok(rows)
}

fn required_text<'a>(value: &'a Value, field: &str) -> Result<&'a str, AppError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| response_error("required_text"))
}

fn optional(value: &Value, field: &str, valid: fn(&Value) -> bool) -> Result<Value, AppError> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(value) if valid(value) => Ok(value.clone()),
        _ => Err(response_error("field_type")),
    }
}

fn uint(value: &Value, field: &str) -> Result<Value, AppError> {
    optional(value, field, |v| v.as_u64().is_some())
}

fn check_count(value: &Value, actual: usize) -> Result<(), AppError> {
    if value
        .get("count")
        .is_some_and(|v| v.as_u64() != Some(actual as u64))
    {
        return Err(response_error("count_mismatch"));
    }
    Ok(())
}

fn response_error(reason: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "TradingView economic response is invalid",
    )
    .with_details(json!({
        "contract_version": "mcp_error.v1", "source": "tradingview_mcp",
        "code": "invalid_response", "reason": reason
    }))
}

#[cfg(test)]
mod request_tests {
    use super::*;

    #[test]
    fn series_preserves_values_and_bounds_without_claiming_coverage() {
        let request = Request::series("ECONOMICS:USEXAMPLE", Some("2025-01-01"), None).unwrap();
        let value = json!({
            "success": true, "symbol": "ECONOMICS:USEXAMPLE", "count": 2,
            "series": [
                {"date": "2026-02-01", "value": null},
                {"date": "2026-01-01", "value": 0}
            ],
            "unit": "%", "scale": 100
        });
        let data = normalize(&request, value.clone(), 1000).unwrap();
        assert!(data["series"][0]["value"].is_null());
        assert_eq!(data["series"][1]["value"], 0);
        assert_eq!(data["provider_metadata"]["scale"], 100);
        assert_eq!(
            data["client_observation"]["actual_range"]["from"],
            "2026-01-01"
        );
        assert_eq!(
            data["client_observation"]["requested_window_coverage"],
            "unconfirmed"
        );
        for (field, bad) in [
            ("symbol", json!("ECONOMICS:OTHER")),
            ("count", json!(3)),
            ("series", json!([{"date":"invalid", "value": 1}])),
        ] {
            let mut invalid = value.clone();
            invalid[field] = bad;
            assert!(normalize(&request, invalid, 1000).is_err());
        }
        let empty = normalize(
            &request,
            json!({"symbol": "ECONOMICS:USEXAMPLE", "series": [], "count": 0}),
            1000,
        )
        .unwrap();
        assert!(empty["client_observation"]["actual_range"]["from"].is_null());
    }

    #[test]
    fn catalog_distinguishes_overview_from_qualified_symbols() {
        let overview = Request::symbols(None, None, None).unwrap();
        let value = json!({
            "categories": [{"id": "prce", "name": "Prices", "count": 1}],
            "countries": ["US"], "indicators_count": 1
        });
        let data = normalize(&overview, value, 1000).unwrap();
        assert_eq!(data["mode"], "overview");
        let request = Request::symbols(Some("US"), None, None).unwrap();
        let value = json!({
            "country": {"code": "US", "name": "United States"},
            "count": 1, "symbols": [{"symbol": "ECONOMICS:USEXAMPLE"}]
        });
        assert_eq!(
            normalize(&request, value.clone(), 1000).unwrap()["items"][0]["symbol"],
            "ECONOMICS:USEXAMPLE"
        );
        let mut wrong = value;
        wrong["country"]["code"] = json!("JP");
        assert!(normalize(&request, wrong, 1000).is_err());
        assert!(
            normalize(
                &overview,
                json!({"categories": [null], "countries": []}),
                1000
            )
            .is_err()
        );
    }

    #[test]
    fn indicator_codes_remain_codes_and_unknown_calendar_status_fails_closed() {
        let request = Request::symbols(None, Some("prce"), None).unwrap();
        let value = json!({"count": 1, "indicators": [{
            "code": "EXAMPLE", "name": "Synthetic", "category": "prce"
        }]});
        let data = normalize(&request, value, 1000).unwrap();
        assert_eq!(data["mode"], "indicators");
        assert_eq!(data["items"][0]["code"], "EXAMPLE");
        assert!(data["items"][0].get("symbol").is_none());
        let calendar = Request::calendar(CalendarOptions::default()).unwrap();
        assert!(normalize(&calendar, json!({"status": "error", "result": []}), 1000).is_err());
        assert_eq!(
            normalize(&calendar, json!({"status": "ok", "result": []}), 1000).unwrap()["items"],
            json!([])
        );
    }

    #[test]
    fn calendar_preserves_actual_forecast_previous_and_raw_values() {
        let request = Request::calendar(CalendarOptions::default()).unwrap();
        let value = json!({"status": "ok", "result": [{
            "id": "synthetic-event", "date": "2026-01-01T12:00:00Z",
            "actual": 0, "actualRaw": 0, "forecast": null,
            "previous": 5, "scale": "million"
        }]});
        let data = normalize(&request, value.clone(), 1000).unwrap();
        assert_eq!(data["items"][0]["actual"], 0);
        assert!(data["items"][0]["forecast"].is_null());
        assert_eq!(data["items"][0]["previous"], 5);
        assert_eq!(data["items"][0]["scale"], "million");
        let mut wrong = value;
        wrong["result"][0]["actual"] = json!("unknown");
        assert!(normalize(&request, wrong, 1000).is_err());
        assert!(normalize(&request, json!({"status": "ok"}), 1000).is_err());
    }

    #[test]
    fn dividends_do_not_hide_missing_symbols_or_fill_unknown_amounts() {
        let request = Request::dividends(DividendOptions {
            symbols: vec!["NYSE:OTHER".into(), "NASDAQ:EXAMPLE".into()],
            ..Default::default()
        })
        .unwrap();
        let value = json!({"success": true, "count": 1, "data": [{
            "symbol": "NASDAQ:EXAMPLE", "dividend_amount_recent": 0,
            "dividend_amount_upcoming": null
        }]});
        let data = normalize(&request, value.clone(), 1000).unwrap();
        assert_eq!(data["outcomes"][0]["status"], "unreported");
        assert_eq!(data["outcomes"][1]["status"], "returned");
        assert!(data["items"][0]["dividend_amount_upcoming"].is_null());
        assert_eq!(data["items"][0]["dividend_amount_recent"], 0);
        let mut wrong = value;
        wrong["data"][0]["symbol"] = json!("NYSE:UNREQUESTED");
        assert!(normalize(&request, wrong, 1000).is_err());
        let error = normalize(
            &request,
            json!({"success": false, "error": "synthetic-secret"}),
            1000,
        )
        .unwrap_err();
        assert!(!format!("{error:?}").contains("synthetic-secret"));
    }

    #[test]
    fn rejects_ambiguous_modes_and_preserves_explicit_dates() {
        assert!(Request::dividends(DividendOptions::default()).is_err());
        assert!(
            Request::dividends(DividendOptions {
                symbols: vec!["NASDAQ:EXAMPLE".into()],
                market: Some("america".into()),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            Request::dividends(DividendOptions {
                symbols: vec!["NASDAQ:EXAMPLE".into()],
                from: Some("2026-01-01".into()),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            Request::dividends(DividendOptions {
                market: Some("america".into()),
                limit: Some(201),
                ..Default::default()
            })
            .is_err()
        );
        assert!(Request::series("NASDAQ:EXAMPLE", None, None).is_err());
        assert!(Request::series("ECONOMICS:USIRYY", Some("2025-02-29"), None).is_err());
        assert!(
            Request::series("ECONOMICS:USIRYY", Some("2026-02-01"), Some("2026-01-01")).is_err()
        );
        let request = Request::calendar(CalendarOptions {
            from: Some("2026-01-01".into()),
            to: Some("2026-01-01T12:00:00Z".into()),
            min_importance: Some(0),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(request.arguments()["date_from"], "2026-01-01");
        assert_eq!(request.arguments()["date_to"], "2026-01-01T12:00:00Z");
        assert_eq!(request.arguments()["min_importance"], 0);
    }

    #[test]
    fn country_currency_and_category_inputs_are_closed() {
        assert!(Request::symbols(Some("US,JP"), None, None).is_err());
        assert!(Request::symbols(Some("US"), Some("prices"), None).is_err());
        assert!(Request::symbols(Some("US"), Some("prce"), Some("inflation")).is_ok());
        assert!(
            Request::calendar(CalendarOptions {
                countries: Some("US,US".into()),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            Request::calendar(CalendarOptions {
                currencies: Some("usd".into()),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            Request::calendar(CalendarOptions {
                min_importance: Some(2),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            Request::calendar(CalendarOptions {
                category: Some("goverment".into()),
                ..Default::default()
            })
            .is_ok()
        );
    }
}
