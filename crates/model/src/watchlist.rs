use std::collections::HashSet;

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

pub const MAX_WATCHLIST_BULK_SYMBOLS: usize = 50;
pub const MAX_WATCHLIST_BULK_DELAY_MS: u64 = 10_000;

#[derive(Debug)]
pub struct WatchlistBulkAccumulator {
    requested_count: usize,
    delay_ms: u64,
    allow_partial: bool,
    seen: HashSet<String>,
    results: Vec<Value>,
    processed_count: usize,
    added_count: usize,
    already_present_count: usize,
    failed_count: usize,
    skipped_duplicate_count: usize,
}

impl WatchlistBulkAccumulator {
    pub fn new(requested_count: usize, delay_ms: u64, allow_partial: bool) -> Self {
        Self {
            requested_count,
            delay_ms,
            allow_partial,
            seen: HashSet::new(),
            results: Vec::new(),
            processed_count: 0,
            added_count: 0,
            already_present_count: 0,
            failed_count: 0,
            skipped_duplicate_count: 0,
        }
    }

    pub fn mark_seen_or_duplicate(&mut self, input_index: usize, symbol: &str) -> bool {
        if self.seen.insert(symbol.to_string()) {
            true
        } else {
            self.skipped_duplicate_count += 1;
            self.results.push(json!({
                "input_index": input_index,
                "symbol": symbol,
                "status": "skipped_duplicate",
            }));
            false
        }
    }

    pub fn record_success(&mut self, input_index: usize, symbol: &str, data: Value) {
        self.processed_count += 1;
        let action = data
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("added");
        let status = if action == "already_present" {
            self.already_present_count += 1;
            "already_present"
        } else {
            self.added_count += 1;
            "added"
        };
        self.results.push(json!({
            "input_index": input_index,
            "symbol": symbol,
            "status": status,
            "data": data,
        }));
    }

    pub fn record_failure(&mut self, input_index: usize, symbol: &str, error: AppError) {
        self.processed_count += 1;
        self.failed_count += 1;
        self.results.push(json!({
            "input_index": input_index,
            "symbol": symbol,
            "status": "failed",
            "error": {
                "kind": error.kind,
                "message": error.message,
                "details": error.details,
            },
        }));
    }

    pub fn processed_count(&self) -> usize {
        self.processed_count
    }

    pub fn failed_count(&self) -> usize {
        self.failed_count
    }

    pub fn payload(self) -> Value {
        json!({
            "action": "bulk_add",
            "requested_count": self.requested_count,
            "processed_count": self.processed_count,
            "added_count": self.added_count,
            "already_present_count": self.already_present_count,
            "failed_count": self.failed_count,
            "skipped_duplicate_count": self.skipped_duplicate_count,
            "delay_ms": self.delay_ms,
            "allow_partial": self.allow_partial,
            "results": self.results,
        })
    }
}

pub fn normalize_watchlist_symbol(symbol: &str) -> Result<String, AppError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Symbol must not be empty",
        ));
    }
    Ok(symbol.to_string())
}

pub fn unique_watchlist_symbol_count(symbols: &[String]) -> Result<usize, AppError> {
    symbols
        .iter()
        .map(|symbol| normalize_watchlist_symbol(symbol))
        .collect::<Result<HashSet<_>, _>>()
        .map(|symbols| symbols.len())
}

pub fn validate_watchlist_add_bulk_request(
    symbols: &[String],
    delay_ms: u64,
) -> Result<(), AppError> {
    if symbols.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "At least one symbol is required",
        ));
    }
    if delay_ms > MAX_WATCHLIST_BULK_DELAY_MS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("--delay-ms must be at most {MAX_WATCHLIST_BULK_DELAY_MS}"),
        ));
    }

    let unique_count = unique_watchlist_symbol_count(symbols)?;
    if unique_count > MAX_WATCHLIST_BULK_SYMBOLS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("At most {MAX_WATCHLIST_BULK_SYMBOLS} unique symbols can be added at once"),
        ));
    }

    Ok(())
}

pub fn normalize_watchlist_api_payload(data: Value) -> Result<Value, AppError> {
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if data.get("source").and_then(Value::as_str) != Some("watchlist_api") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Watchlist API response shape was not recognized",
        )
        .with_details(json!({
            "phase": "unrecognized_response",
            "api_fallback_allowed": true,
            "source": data.get("source").cloned().unwrap_or(Value::Null),
        })));
    }

    Ok(data)
}

pub fn watchlist_api_error_allows_fallback(error: &AppError) -> bool {
    error
        .details
        .as_ref()
        .and_then(|details| details.get("api_fallback_allowed"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn normalize_watchlist_remove_payload(data: Value) -> Result<Value, AppError> {
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if !data
        .get("removed")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Watchlist remove did not remove the requested symbol",
        )
        .with_details(data));
    }

    Ok(json!({
        "symbol": data.get("symbol").cloned().unwrap_or(Value::Null),
        "requested_symbol": data.get("requested_symbol").cloned().unwrap_or(Value::Null),
        "action": data.get("action").cloned().unwrap_or_else(|| json!("removed")),
        "removed": true,
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("dom_row"),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "matched_before": data
            .get("matched_before")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        "matched_after": data
            .get("matched_after")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "remove_method": data.get("remove_method").cloned().unwrap_or(Value::Null),
        "click_method": data.get("click_method").cloned().unwrap_or(Value::Null),
        "confirmation_clicked": data
            .get("confirmation_clicked")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalize_watchlist_symbol_trims_and_rejects_empty_values() {
        assert_eq!(
            normalize_watchlist_symbol(" NASDAQ:AAPL ").unwrap(),
            "NASDAQ:AAPL"
        );
        assert_eq!(
            normalize_watchlist_symbol(" ").unwrap_err().kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn watchlist_add_bulk_validates_inputs_before_connecting() {
        assert_eq!(
            validate_watchlist_add_bulk_request(&[], 0)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_watchlist_add_bulk_request(&[" ".to_string()], 0)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_watchlist_add_bulk_request(
                &["NASDAQ:AAPL".to_string()],
                MAX_WATCHLIST_BULK_DELAY_MS + 1,
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
        let too_many = (0..=MAX_WATCHLIST_BULK_SYMBOLS)
            .map(|index| format!("NASDAQ:TEST{index}"))
            .collect::<Vec<_>>();
        assert_eq!(
            validate_watchlist_add_bulk_request(&too_many, 0)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn watchlist_bulk_accumulator_preserves_payload_shape() {
        let mut accumulator = WatchlistBulkAccumulator::new(3, 0, true);
        assert!(accumulator.mark_seen_or_duplicate(0, "NASDAQ:AAPL"));
        accumulator.record_success(0, "NASDAQ:AAPL", json!({"action": "added"}));
        assert!(!accumulator.mark_seen_or_duplicate(1, "NASDAQ:AAPL"));
        assert!(accumulator.mark_seen_or_duplicate(2, "NASDAQ:MSFT"));
        accumulator.record_success(2, "NASDAQ:MSFT", json!({"action": "already_present"}));

        let payload = accumulator.payload();

        assert_eq!(payload["action"], "bulk_add");
        assert_eq!(payload["requested_count"], 3);
        assert_eq!(payload["processed_count"], 2);
        assert_eq!(payload["added_count"], 1);
        assert_eq!(payload["already_present_count"], 1);
        assert_eq!(payload["failed_count"], 0);
        assert_eq!(payload["skipped_duplicate_count"], 1);
        assert_eq!(payload["results"][0]["status"], "added");
        assert_eq!(payload["results"][1]["status"], "skipped_duplicate");
        assert_eq!(payload["results"][2]["status"], "already_present");
    }

    #[test]
    fn watchlist_api_payload_normalization_maps_error_kind_and_fallback_flag() {
        let error = normalize_watchlist_api_payload(json!({
            "error": "Watchlist API unavailable",
            "error_kind": "internal_api_unavailable",
            "api_fallback_allowed": true,
            "source": "watchlist_api"
        }))
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(watchlist_api_error_allows_fallback(&error));

        let validation = normalize_watchlist_api_payload(json!({
            "error": "Watchlist symbol not found",
            "error_kind": "validation",
            "api_fallback_allowed": false,
            "source": "watchlist_api"
        }))
        .unwrap_err();

        assert_eq!(validation.kind, ErrorKind::Validation);
        assert!(!watchlist_api_error_allows_fallback(&validation));
    }

    #[test]
    fn watchlist_api_payload_normalization_requires_source_marker() {
        let error = normalize_watchlist_api_payload(json!({"source": "dom_row"})).unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.details.unwrap()["api_fallback_allowed"], true);
    }

    #[test]
    fn watchlist_remove_payload_normalization_preserves_public_fields() {
        let payload = normalize_watchlist_remove_payload(json!({
            "symbol": "NASDAQ:AAPL",
            "requested_symbol": "NASDAQ:AAPL",
            "action": "removed",
            "removed": true,
            "source": "watchlist_api",
            "before_count": 2,
            "after_count": 1,
            "matched_before": true,
            "matched_after": false,
            "remove_method": "api",
            "click_method": null,
            "confirmation_clicked": false
        }))
        .unwrap();

        assert_eq!(payload["symbol"], "NASDAQ:AAPL");
        assert_eq!(payload["removed"], true);
        assert_eq!(payload["source"], "watchlist_api");
        assert_eq!(payload["remove_method"], "api");
    }

    #[test]
    fn watchlist_remove_payload_normalization_rejects_unverified_remove() {
        let error = normalize_watchlist_remove_payload(json!({
            "symbol": "NASDAQ:AAPL",
            "removed": false,
            "matched_after": true
        }))
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.details.unwrap()["matched_after"], true);
    }
}
