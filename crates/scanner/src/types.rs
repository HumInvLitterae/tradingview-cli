use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScannerRow {
    pub symbol: String,
    pub values: Vec<Value>,
    pub field_values: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScannerHotlistResult {
    pub source: String,
    pub region: String,
    pub slug: String,
    pub limit: usize,
    pub count: usize,
    pub total_count: Value,
    pub fields: Vec<String>,
    pub symbols: Vec<ScannerRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScannerScanResult {
    pub source: String,
    pub market: String,
    pub limit: usize,
    pub count: usize,
    pub total_count: Value,
    pub columns: Vec<String>,
    pub sort: ScannerSort,
    pub filters: Vec<Value>,
    pub symbols: Vec<ScannerRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScannerSort {
    pub field: String,
    pub order: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScannerMetainfoResult {
    pub source: String,
    pub market: String,
    pub requested_fields: Vec<String>,
    pub field_count: usize,
    pub fields: Vec<ScannerFieldInfo>,
    pub missing_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_currency: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScannerFieldInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Value>,
}
