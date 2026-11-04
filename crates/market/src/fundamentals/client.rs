use serde_json::{Value, json};
use tradingview_core::AppError;

use crate::{http::send_json, normalize::split_exchange_symbol};

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
    send_json(
        client.post(FUNDAMENTALS_SCAN_URL).json(&body),
        "Scanner fundamentals",
        "TradingView scanner fundamentals API",
    )
    .await
}
