use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
/// One normalized scanner row.
pub struct ScannerRow {
    /// Exchange-qualified symbol when TradingView provides one.
    pub symbol: String,
    /// Raw row values in the same order as the requested columns.
    pub values: Vec<Value>,
    /// Object mapping column names to row values.
    pub field_values: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Result for a scanner preset hotlist read.
pub struct ScannerHotlistResult {
    /// Public source marker.
    pub source: String,
    /// Hotlist region marker used by TradingView presets.
    pub region: String,
    /// Hotlist slug requested by the caller.
    pub slug: String,
    /// Limit applied to the returned rows.
    pub limit: usize,
    /// Number of rows returned.
    pub count: usize,
    /// Total count reported by TradingView, when available.
    pub total_count: Value,
    /// Field names returned by the hotlist endpoint.
    pub fields: Vec<String>,
    /// Normalized rows.
    pub symbols: Vec<ScannerRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Result for a scanner table read.
pub struct ScannerScanResult {
    /// Public source marker.
    pub source: String,
    /// Scanner market, such as `america`.
    pub market: String,
    /// Limit applied to the returned rows.
    pub limit: usize,
    /// Number of rows returned.
    pub count: usize,
    /// Total count reported by TradingView, when available.
    pub total_count: Value,
    /// Requested or default columns.
    pub columns: Vec<String>,
    /// Sort field and direction sent to the scanner endpoint.
    pub sort: ScannerSort,
    /// Normalized filter payload sent to the scanner endpoint.
    pub filters: Vec<Value>,
    /// Normalized rows.
    pub symbols: Vec<ScannerRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Scanner sort descriptor.
pub struct ScannerSort {
    /// Sort field name.
    pub field: String,
    /// Sort direction, usually `asc` or `desc`.
    pub order: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Result for scanner field metadata discovery.
pub struct ScannerMetainfoResult {
    /// Public source marker.
    pub source: String,
    /// Scanner market, such as `america`.
    pub market: String,
    /// Field names requested by the caller.
    pub requested_fields: Vec<String>,
    /// Number of matched fields returned in `fields`.
    pub field_count: usize,
    /// Matched field metadata.
    pub fields: Vec<ScannerFieldInfo>,
    /// Requested field names that were not found.
    pub missing_fields: Vec<String>,
    /// Financial currency metadata when TradingView returns it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_currency: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Public-safe scanner field metadata.
pub struct ScannerFieldInfo {
    /// Field name used in scanner requests.
    pub name: String,
    /// TradingView field type, serialized as `type` for CLI compatibility.
    #[serde(rename = "type")]
    pub field_type: Value,
    /// Human-readable label when available and not redundant with `name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Range metadata when TradingView returns it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Value>,
}
