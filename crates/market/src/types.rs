use serde::Serialize;
use serde_json::Value;
use tradingview_core::ErrorKind;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SymbolSearchResponse {
    pub query: String,
    pub source: String,
    pub count: usize,
    pub results: Vec<SymbolSearchResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SymbolSearchResult {
    pub symbol: String,
    pub description: String,
    pub exchange: String,
    #[serde(rename = "type")]
    pub symbol_type: String,
    pub full_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub full_name: String,
    pub exchange: String,
    pub description: Value,
    #[serde(rename = "type")]
    pub symbol_type: Value,
    pub pro_name: String,
    pub typespecs: Value,
    pub resolution: Value,
    pub chart_type: Value,
    pub source: String,
    pub non_mutating: bool,
    pub requested_symbol: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Quote {
    pub symbol: String,
    pub time: Value,
    pub last: Value,
    pub close: Value,
    pub open: Value,
    pub high: Value,
    pub low: Value,
    pub volume: Value,
    pub change: Value,
    pub description: Value,
    pub exchange: Value,
    #[serde(rename = "type")]
    pub symbol_type: Value,
    pub subtype: Value,
    pub extended_hours: ExtendedHoursQuote,
    pub update_mode: Value,
    pub delay_seconds: Value,
    pub source: String,
    pub non_mutating: bool,
    pub requested_symbol: String,
    pub original_symbol: Value,
    pub observed_symbol: String,
    pub switch_performed: bool,
    pub restored: bool,
    pub freshness_check: FreshnessCheck,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtendedHoursQuote {
    pub premarket: SessionQuote,
    pub postmarket: SessionQuote,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SessionQuote {
    pub open: Value,
    pub high: Value,
    pub low: Value,
    pub last: Value,
    pub close: Value,
    pub change_percent: Value,
    pub change_abs: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap_percent: Option<Value>,
    pub volume: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FreshnessCheck {
    pub kind: String,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BatchQuotes {
    pub source: String,
    pub requested_count: usize,
    pub resolved_count: usize,
    pub error_count: usize,
    pub items: Vec<BatchQuoteItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BatchQuoteItem {
    pub requested_symbol: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QuoteError>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuoteError {
    pub kind: ErrorKind,
    pub message: String,
    pub details: Option<Value>,
}
