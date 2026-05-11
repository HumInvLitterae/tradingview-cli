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
    /// Command source category used by the CLI source taxonomy.
    pub source_category: String,
    /// False because this read does not require TradingView Desktop.
    pub requires_desktop: bool,
    /// True because this read does not mutate a chart.
    pub non_mutating: bool,
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
    /// Command source category used by the CLI source taxonomy.
    pub source_category: String,
    /// False because this read does not require TradingView Desktop.
    pub requires_desktop: bool,
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
    /// Command source category used by the CLI source taxonomy.
    pub source_category: String,
    /// False because this read does not require TradingView Desktop.
    pub requires_desktop: bool,
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
    /// Requested field groups, when the caller used group expansion.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requested_groups: Vec<String>,
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
    /// Command source category used by the CLI source taxonomy.
    pub source_category: String,
    /// False because this read does not require TradingView Desktop.
    pub requires_desktop: bool,
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
    /// Command source category used by the CLI source taxonomy.
    pub source_category: String,
    /// False because this read does not require TradingView Desktop.
    pub requires_desktop: bool,
    /// True because scanner batch quote reads do not mutate a chart.
    pub non_mutating: bool,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Desktop-free evidence packet for one requested symbol.
pub struct Snapshot {
    /// Command-local payload contract marker for downstream schema guards.
    pub contract_version: String,
    /// Public source marker.
    pub source: String,
    /// Command source category used by the CLI source taxonomy.
    pub source_category: String,
    /// False because this read does not require TradingView Desktop.
    pub requires_desktop: bool,
    /// True because snapshot reads do not mutate a chart.
    pub non_mutating: bool,
    /// Symbol text supplied by the caller.
    pub requested_symbol: String,
    /// Best resolved exchange-qualified symbol from the successful sections.
    pub symbol: Value,
    /// Best observed symbol from the successful sections.
    pub observed_symbol: Value,
    /// Machine-readable readback summary derived from `sections`.
    pub summary: SnapshotSummary,
    /// Per-source evidence sections.
    pub sections: SnapshotSections,
    /// Public-safe section error summaries.
    pub errors: Vec<SnapshotSectionError>,
    /// Machine-readable missing evidence and follow-up readback.
    pub missing_evidence: Vec<SnapshotMissingEvidence>,
    /// Machine-readable available follow-up surfaces.
    pub follow_up_hints: Vec<SnapshotFollowUpHint>,
    /// Suggested follow-up commands when Desktop-backed or visual evidence is needed.
    pub next_action_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Additive readback summary for a one-symbol snapshot packet.
pub struct SnapshotSummary {
    /// Evidence coverage status; this is not a ranking or recommendation.
    pub coverage_status: String,
    /// True when the quote section succeeded.
    pub quote_ok: bool,
    /// True when the info section succeeded.
    pub info_ok: bool,
    /// True when the fundamentals section succeeded.
    pub fundamentals_ok: bool,
    /// Number of section errors.
    pub error_count: usize,
    /// Total known missing field count.
    pub missing_total_count: usize,
    /// Section and field-category coverage readback.
    pub field_coverage: SnapshotFieldCoverage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Section and field-category coverage readback for a snapshot packet.
pub struct SnapshotFieldCoverage {
    /// True when the quote section succeeded.
    pub quote_ok: bool,
    /// Number of known missing quote fields.
    pub quote_missing_count: usize,
    /// True when the info section succeeded.
    pub info_ok: bool,
    /// Number of known missing symbol-info fields.
    pub info_missing_count: usize,
    /// True when the fundamentals section succeeded.
    pub fundamentals_ok: bool,
    /// Number of known missing fundamentals fields.
    pub fundamentals_missing_count: usize,
    /// Total known missing field count.
    pub total_missing_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Per-source evidence sections in a snapshot payload.
pub struct SnapshotSections {
    /// Scanner-backed quote evidence.
    pub quote: SnapshotSection,
    /// Desktop-free symbol metadata evidence.
    pub info: SnapshotSection,
    /// Scanner-backed fundamentals evidence.
    pub fundamentals: SnapshotSection,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// One snapshot evidence section.
pub struct SnapshotSection {
    /// True when `data` is present.
    pub ok: bool,
    /// Successful section payload, preserving the existing command shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Public-safe section error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SnapshotSectionError>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Public-safe per-section snapshot error.
pub struct SnapshotSectionError {
    /// Section name, such as `quote`, `info`, or `fundamentals`.
    pub section: String,
    /// Structured error kind.
    pub kind: ErrorKind,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error details.
    pub details: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Machine-readable follow-up surface for a snapshot packet.
pub struct SnapshotFollowUpHint {
    /// Stable hint kind.
    pub kind: String,
    /// Executor-readable command string.
    pub command: String,
    /// Stable reason explaining what the command can add.
    pub reason: String,
    /// True when the follow-up requires TradingView Desktop.
    pub requires_desktop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Missing evidence entry for one snapshot section.
pub struct SnapshotMissingEvidence {
    /// Section with missing evidence, such as `quote`, `info`, or `fundamentals`.
    pub section: String,
    /// Known missing fields when available.
    pub missing_fields: Vec<String>,
    /// Stable reason for the missing evidence.
    pub missing_reason: String,
    /// Stable follow-up kind that can help collect more evidence.
    pub suggested_follow_up: String,
    /// True when the suggested follow-up requires TradingView Desktop.
    pub requires_desktop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Desktop-free comparison packet for several requested symbols.
pub struct Compare {
    /// Command-local payload contract marker for downstream schema guards.
    pub contract_version: String,
    /// Public source marker.
    pub source: String,
    /// Command source category used by the CLI source taxonomy.
    pub source_category: String,
    /// False because this read does not require TradingView Desktop.
    pub requires_desktop: bool,
    /// True because compare reads do not mutate a chart.
    pub non_mutating: bool,
    /// Number of requested symbols after validation.
    pub requested_count: usize,
    /// Number of items with at least one successful evidence section.
    pub resolved_count: usize,
    /// Number of items with no successful evidence sections.
    pub error_count: usize,
    /// Machine-readable readback summary derived from `items`.
    pub summary: CompareSummary,
    /// Ordered per-symbol comparison items.
    pub items: Vec<CompareItem>,
    /// Public-safe symbol/section error summaries.
    pub errors: Vec<CompareItemError>,
    /// Suggested follow-up commands when Desktop-backed or visual evidence is needed.
    pub next_action_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Additive readback summary for a comparison packet.
pub struct CompareSummary {
    /// Number of requested symbols after validation.
    pub requested_count: usize,
    /// Number of items with at least one successful evidence section.
    pub resolved_count: usize,
    /// Number of items with no successful evidence sections.
    pub error_count: usize,
    /// Evidence coverage status; this is not a ranking or recommendation.
    pub coverage_status: String,
    /// Number of items with a successful quote section.
    pub quote_ok_count: usize,
    /// Number of items with a successful info section.
    pub info_ok_count: usize,
    /// Number of items with a successful fundamentals section.
    pub fundamentals_ok_count: usize,
    /// Total missing field count across all items.
    pub missing_total_count: usize,
    /// Section and field-category coverage readback.
    pub field_coverage: CompareFieldCoverage,
    /// Movement evidence coverage readback for downstream session posture tools.
    pub movement_coverage: CompareMovementCoverage,
    /// Ordered symbol resolution readback for downstream tools.
    pub resolved_symbols: Vec<CompareResolvedSymbol>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Regular-session movement coverage readback for a comparison packet.
pub struct CompareMovementCoverage {
    /// Number of items with a normalized regular-session percent change.
    pub regular_change_percent_available_count: usize,
    /// Number of items missing normalized regular-session percent change.
    pub regular_change_percent_missing_count: usize,
    /// Number of items with a normalized regular-session absolute change.
    pub regular_change_abs_available_count: usize,
    /// Number of items missing normalized regular-session absolute change.
    pub regular_change_abs_missing_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Section and field-category coverage readback for a comparison packet.
pub struct CompareFieldCoverage {
    /// Number of items with a successful quote section.
    pub quote_ok_count: usize,
    /// Number of missing quote fields across all items.
    pub quote_missing_count: usize,
    /// Number of items with a successful info section.
    pub info_ok_count: usize,
    /// Number of missing symbol-info fields across all items.
    pub info_missing_count: usize,
    /// Number of items with a successful fundamentals section.
    pub fundamentals_ok_count: usize,
    /// Number of missing fundamentals fields across all items.
    pub fundamentals_missing_count: usize,
    /// Number of missing earnings-group fundamentals fields.
    pub earnings_missing_count: usize,
    /// Number of missing dividends-group fundamentals fields.
    pub dividends_missing_count: usize,
    /// Total missing field count across all items.
    pub total_missing_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Symbol-level readback summary for one comparison item.
pub struct CompareResolvedSymbol {
    /// Zero-based position in the validated input symbol list.
    pub requested_index: usize,
    /// Symbol text supplied by the caller.
    pub requested_symbol: String,
    /// True when at least one evidence section succeeded.
    pub ok: bool,
    /// Best resolved exchange-qualified symbol from successful sections.
    pub symbol: Value,
    /// Best observed symbol from successful sections.
    pub observed_symbol: Value,
    /// True when the quote section succeeded.
    pub quote_ok: bool,
    /// True when the info section succeeded.
    pub info_ok: bool,
    /// True when the fundamentals section succeeded.
    pub fundamentals_ok: bool,
    /// Missing field count for this item.
    pub missing_total_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// One requested symbol inside a comparison packet.
pub struct CompareItem {
    /// Zero-based position in the validated input symbol list.
    pub requested_index: usize,
    /// Symbol text supplied by the caller.
    pub requested_symbol: String,
    /// Best resolved exchange-qualified symbol from successful sections.
    pub symbol: Value,
    /// Best observed symbol from successful sections.
    pub observed_symbol: Value,
    /// True when at least one evidence section succeeded.
    pub ok: bool,
    /// Per-source evidence sections.
    pub sections: SnapshotSections,
    /// Public-safe section error summaries for this item.
    pub errors: Vec<SnapshotSectionError>,
    /// Missing-value summary for successful sections.
    pub missing_summary: CompareMissingSummary,
    /// Regular-session movement readback derived from the quote section.
    pub movement: CompareMovement,
    /// Machine-readable missing evidence and follow-up readback for this item.
    pub missing_evidence: Vec<CompareMissingEvidence>,
    /// Machine-readable available follow-up surfaces for this item.
    pub follow_up_hints: Vec<CompareFollowUpHint>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Regular-session movement readback for one comparison item.
pub struct CompareMovement {
    /// Scanner quote regular-session percentage change.
    pub regular_change_percent: Value,
    /// Normalized regular-session absolute change when available.
    pub regular_change_abs: Value,
    /// Scanner quote last price.
    pub regular_last: Value,
    /// Scanner quote close price.
    pub regular_close: Value,
    /// Evidence section used to build this readback.
    pub source_section: String,
    /// Primary raw evidence path for regular-session percentage change.
    pub source_path: String,
    /// True when `regular_change_percent` is available.
    pub available: bool,
    /// Missing reason when `available` is false.
    pub missing_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Machine-readable follow-up surface for one comparison item.
pub struct CompareFollowUpHint {
    /// Stable hint kind.
    pub kind: String,
    /// Executor-readable command string.
    pub command: String,
    /// Stable reason explaining what the command can add.
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Missing-value summary for one comparison item.
pub struct CompareMissingSummary {
    /// Missing scanner quote fields when known.
    pub quote: Vec<String>,
    /// Missing symbol-info fields when known.
    pub info: Vec<String>,
    /// Missing scanner fundamentals fields when known.
    pub fundamentals: Vec<String>,
    /// Total missing field count across sections.
    pub total_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Missing evidence entry for one comparison item section.
pub struct CompareMissingEvidence {
    /// Section with missing evidence, such as `quote`, `info`, or `fundamentals`.
    pub section: String,
    /// Known missing fields when available.
    pub missing_fields: Vec<String>,
    /// Stable reason for the missing evidence.
    pub missing_reason: String,
    /// Stable follow-up kind that can help collect more evidence.
    pub suggested_follow_up: String,
    /// True when the suggested follow-up requires TradingView Desktop.
    pub requires_desktop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Public-safe top-level compare error summary.
pub struct CompareItemError {
    /// Requested symbol associated with the section failure.
    pub requested_symbol: String,
    /// Section name, such as `quote`, `info`, or `fundamentals`.
    pub section: String,
    /// Structured error kind.
    pub kind: ErrorKind,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error details.
    pub details: Option<Value>,
}
