use serde_json::{Value, json};

use crate::cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

pub async fn data_depth(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let data = runtime
        .evaluate(
            r#"
            (function() {
                var domPanel =
                    document.querySelector('[class*="depth"]') ||
                    document.querySelector('[class*="orderBook"]') ||
                    document.querySelector('[class*="dom-"]') ||
                    document.querySelector('[class*="DOM"]') ||
                    document.querySelector('[data-name="dom"]');
                if (!domPanel) return { found: false, error: "DOM / Depth of Market panel not found." };

                function parseNumber(text) {
                    if (!text) return NaN;
                    return parseFloat(String(text).replace(/[^0-9.\-]/g, ""));
                }

                var bids = [];
                var asks = [];
                var rows = domPanel.querySelectorAll('[class*="row"], tr');
                for (var i = 0; i < rows.length; i++) {
                    var row = rows[i];
                    var priceEl = row.querySelector('[class*="price"]');
                    var sizeEl = row.querySelector('[class*="size"], [class*="volume"], [class*="qty"]');
                    if (!priceEl) continue;

                    var price = parseNumber(priceEl.textContent);
                    var size = sizeEl ? parseNumber(sizeEl.textContent) : 0;
                    if (isNaN(price)) continue;
                    if (isNaN(size)) size = 0;

                    var rowClass = row.className || "";
                    var rowHTML = row.innerHTML || "";
                    if (/bid|buy/i.test(rowClass) || /bid|buy/i.test(rowHTML)) {
                        bids.push({ price: price, size: size });
                    } else if (/ask|sell/i.test(rowClass) || /ask|sell/i.test(rowHTML)) {
                        asks.push({ price: price, size: size });
                    } else if (i < rows.length / 2) {
                        asks.push({ price: price, size: size });
                    } else {
                        bids.push({ price: price, size: size });
                    }
                }

                if (bids.length === 0 && asks.length === 0) {
                    var cells = domPanel.querySelectorAll('[class*="cell"], td');
                    var prices = [];
                    cells.forEach(function(cell) {
                        var value = parseNumber(cell.textContent);
                        if (!isNaN(value) && value > 0) prices.push(value);
                    });
                    if (prices.length > 0) {
                        return {
                            found: true,
                            raw_values: prices.slice(0, 50),
                            bids: [],
                            asks: [],
                            note: "Could not classify bid/ask levels."
                        };
                    }
                }

                bids.sort(function(a, b) { return b.price - a.price; });
                asks.sort(function(a, b) { return a.price - b.price; });

                var spread = null;
                if (asks.length > 0 && bids.length > 0) {
                    spread = +(asks[0].price - bids[0].price).toFixed(6);
                }

                return { found: true, bids: bids, asks: asks, spread: spread };
            })()
            "#,
            false,
        )
        .await?;

    if !data.get("found").and_then(Value::as_bool).unwrap_or(false) {
        let message = data
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("DOM / Depth of Market panel not found.");
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }

    let bids = data
        .get("bids")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let asks = data
        .get("asks")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let bid_levels = bids.as_array().map_or(0, Vec::len);
    let ask_levels = asks.as_array().map_or(0, Vec::len);
    let raw_values = data.get("raw_values").cloned();

    if bid_levels == 0 && ask_levels == 0 && raw_values.is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "DOM / Depth of Market panel did not expose readable depth levels.",
        ));
    }

    Ok(json!({
        "bid_levels": bid_levels,
        "ask_levels": ask_levels,
        "spread": data.get("spread").cloned().unwrap_or(Value::Null),
        "bids": bids,
        "asks": asks,
        "raw_values": raw_values,
        "note": data.get("note").cloned(),
    }))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use serde_json::json;

    use super::*;
    use crate::ops::test_support::FakeRuntime;

    #[tokio::test]
    async fn data_depth_maps_bid_and_ask_levels() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "found": true,
            "bids": [
                { "price": 100.25, "size": 10 },
                { "price": 100.0, "size": 5 }
            ],
            "asks": [
                { "price": 100.5, "size": 8 }
            ],
            "spread": 0.25
        })]));

        let data = data_depth(&mut runtime).await.unwrap();

        assert_eq!(data["bid_levels"], 2);
        assert_eq!(data["ask_levels"], 1);
        assert_eq!(data["spread"], 0.25);
        assert_eq!(data["bids"][0]["price"], 100.25);
        assert_eq!(data["asks"][0]["size"], 8);
        assert!(runtime.evaluated[0].0.contains("orderBook"));
        assert!(runtime.evaluated[0].0.contains("[data-name=\"dom\"]"));
    }

    #[tokio::test]
    async fn data_depth_allows_raw_values_when_levels_cannot_be_classified() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "found": true,
            "bids": [],
            "asks": [],
            "raw_values": [101.0, 100.5, 100.0],
            "note": "Could not classify bid/ask levels."
        })]));

        let data = data_depth(&mut runtime).await.unwrap();

        assert_eq!(data["bid_levels"], 0);
        assert_eq!(data["ask_levels"], 0);
        assert_eq!(data["raw_values"][1], 100.5);
        assert_eq!(data["note"], "Could not classify bid/ask levels.");
    }

    #[tokio::test]
    async fn data_depth_maps_missing_panel_to_internal_api_error() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "found": false,
            "error": "DOM / Depth of Market panel not found."
        })]));

        let error = data_depth(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "DOM / Depth of Market panel not found.");
    }

    #[tokio::test]
    async fn data_depth_rejects_empty_panel_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "found": true,
            "bids": [],
            "asks": []
        })]));

        let error = data_depth(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.message.contains("readable depth levels"));
    }
}
