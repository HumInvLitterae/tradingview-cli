use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{http::map_http_error, normalize::split_exchange_symbol};

const FUNDAMENTALS_SCAN_URL: &str = "https://scanner.tradingview.com/america/scan";

pub(super) async fn fundamentals_symbol_via_scanner(
    client: &reqwest::Client,
    symbol: &str,
    fields: &[String],
) -> Result<Value, AppError> {
    let (exchange, name) = split_exchange_symbol(symbol);
    let mut filters = vec![json!({
        "left": "name",
        "operation": "equal",
        "right": name,
    })];
    if let Some(exchange) = exchange {
        filters.push(json!({
            "left": "exchange",
            "operation": "in_range",
            "right": [exchange],
        }));
    }

    let body = json!({
        "columns": fields,
        "filter": filters,
        "range": [0, 2],
    });
    let response = client
        .post(FUNDAMENTALS_SCAN_URL)
        .json(&body)
        .send()
        .await
        .map_err(|err| {
            map_http_error(err, ErrorKind::Connection, "Scanner fundamentals request")
        })?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("TradingView scanner fundamentals API returned {status}"),
        ));
    }

    response.json::<Value>().await.map_err(|err| {
        map_http_error(
            err,
            ErrorKind::InternalApiUnavailable,
            "Scanner fundamentals response",
        )
    })
}
