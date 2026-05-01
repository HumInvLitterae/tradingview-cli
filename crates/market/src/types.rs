use serde::Serialize;
use serde_json::Value;
use tradingview_core::ErrorKind;

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Normalized symbol search response from the Desktop-free symbol search API.
pub struct SymbolSearchResponse {
    /// Search text supplied by the caller.
    pub query: String,
    /// Public source marker used by the CLI JSON payload.
    pub source: String,
    /// Number of normalized results.
    pub count: usize,
    /// Ordered search candidates returned by TradingView.
    pub results: Vec<SymbolSearchResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// One symbol search candidate.
pub struct SymbolSearchResult {
    /// Exchange-local symbol, such as `AAPL`.
    pub symbol: String,
    /// Human-readable symbol description.
    pub description: String,
    /// Exchange code, such as `NASDAQ`.
    pub exchange: String,
    /// TradingView symbol type, serialized as `type` for CLI compatibility.
    #[serde(rename = "type")]
    pub symbol_type: String,
    /// Exchange-qualified symbol, such as `NASDAQ:AAPL`.
    pub full_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Desktop-free symbol metadata resolved from symbol search.
pub struct SymbolInfo {
    /// Exchange-local symbol.
    pub symbol: String,
    /// Exchange-qualified symbol.
    pub full_name: String,
    /// Exchange code.
    pub exchange: String,
    /// Description value as returned by TradingView normalization.
    pub description: Value,
    /// Symbol type, serialized as `type` for CLI compatibility.
    #[serde(rename = "type")]
    pub symbol_type: Value,
    /// TradingView-style pro name.
    pub pro_name: String,
    /// Placeholder for chart-backed metadata not available from this read.
    pub typespecs: Value,
    /// Placeholder for chart-backed metadata not available from this read.
    pub resolution: Value,
    /// Placeholder for chart-backed metadata not available from this read.
    pub chart_type: Value,
    /// Public source marker.
    pub source: String,
    /// True because this read does not mutate a chart.
    pub non_mutating: bool,
    /// Symbol text supplied by the caller.
    pub requested_symbol: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Scanner-backed fundamental fields for one resolved symbol.
pub struct Fundamentals {
    /// Public source marker.
    pub source: String,
    /// Symbol text supplied by the caller.
    pub requested_symbol: String,
    /// Exchange-qualified resolved symbol.
    pub symbol: String,
    /// Symbol observed in the scanner response.
    pub observed_symbol: String,
    /// Scanner market used for the read.
    pub market: String,
    /// Requested or default scanner field names.
    pub fields: Vec<String>,
    /// Object mapping field names to TradingView scanner values.
    pub field_values: Value,
    /// Fields whose value slot was missing from the scanner row.
    pub missing_fields: Vec<String>,
    /// True because scanner fundamentals reads do not mutate a chart.
    pub non_mutating: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Scanner-backed quote for one resolved symbol.
pub struct Quote {
    /// Exchange-qualified resolved symbol.
    pub symbol: String,
    /// TradingView quote timestamp when returned by the scanner feed.
    pub time: Value,
    /// Last price value, matching `close` for regular-session scanner reads.
    pub last: Value,
    /// Regular-session close or latest scanner price.
    pub close: Value,
    /// Regular-session open.
    pub open: Value,
    /// Regular-session high.
    pub high: Value,
    /// Regular-session low.
    pub low: Value,
    /// Regular-session volume.
    pub volume: Value,
    /// Regular-session percentage change.
    pub change: Value,
    /// Symbol description.
    pub description: Value,
    /// Exchange code.
    pub exchange: Value,
    /// Symbol type, serialized as `type` for CLI compatibility.
    #[serde(rename = "type")]
    pub symbol_type: Value,
    /// Symbol subtype when TradingView provides one.
    pub subtype: Value,
    /// Premarket and postmarket values returned by the scanner feed.
    pub extended_hours: ExtendedHoursQuote,
    /// TradingView feed update mode, such as delayed streaming when provided.
    pub update_mode: Value,
    /// Parsed delay in seconds when `update_mode` exposes one.
    pub delay_seconds: Value,
    /// Public source marker.
    pub source: String,
    /// True because scanner quote reads do not mutate a chart.
    pub non_mutating: bool,
    /// Symbol text supplied by the caller.
    pub requested_symbol: String,
    /// Chart-backed original symbol placeholder for CLI payload compatibility.
    pub original_symbol: Value,
    /// Symbol observed in the scanner response.
    pub observed_symbol: String,
    /// Always false for scanner-backed typed quotes.
    pub switch_performed: bool,
    /// Always true for scanner-backed typed quotes because no chart restore is needed.
    pub restored: bool,
    /// Structured freshness check result used by CLI payloads.
    pub freshness_check: FreshnessCheck,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Premarket and postmarket quote groups.
pub struct ExtendedHoursQuote {
    /// Premarket quote values.
    pub premarket: SessionQuote,
    /// Postmarket quote values.
    pub postmarket: SessionQuote,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// One extended-hours quote group.
pub struct SessionQuote {
    /// Session open.
    pub open: Value,
    /// Session high.
    pub high: Value,
    /// Session low.
    pub low: Value,
    /// Session last price.
    pub last: Value,
    /// Session close, matching `last` for scanner extended-hours reads.
    pub close: Value,
    /// Session percentage change.
    pub change_percent: Value,
    /// Session absolute change.
    pub change_abs: Value,
    /// Session gap percentage when this field exists for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap_percent: Option<Value>,
    /// Session volume.
    pub volume: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Result of a quote freshness check.
pub struct FreshnessCheck {
    /// Machine-readable check name.
    pub kind: String,
    /// Whether the check passed.
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Ordered batch quote result.
pub struct BatchQuotes {
    /// Public source marker.
    pub source: String,
    /// Number of requested symbols.
    pub requested_count: usize,
    /// Number of symbols resolved successfully.
    pub resolved_count: usize,
    /// Number of per-item errors.
    pub error_count: usize,
    /// Per-requested-symbol results in input order.
    pub items: Vec<BatchQuoteItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// One ordered batch quote item.
pub struct BatchQuoteItem {
    /// Symbol text supplied for this item.
    pub requested_symbol: String,
    /// True when `quote` is present.
    pub ok: bool,
    /// Successful quote payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quote>,
    /// Public-safe error payload for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QuoteError>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Public-safe per-item quote error.
pub struct QuoteError {
    /// Structured error kind.
    pub kind: ErrorKind,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error details.
    pub details: Option<Value>,
}
