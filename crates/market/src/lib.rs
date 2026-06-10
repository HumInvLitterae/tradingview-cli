//! Desktop-free TradingView market reads.
//!
//! This crate contains credential-free, read-only helpers for symbol search,
//! symbol metadata, single-symbol fundamentals, single-symbol quotes, ordered
//! batch quotes, multi-symbol comparison packets, and browserless historical
//! bars. It does not connect to
//! TradingView Desktop, CDP, chart state, UI automation, or account mutation
//! paths.
//!
//! Prefer the typed functions for Rust callers:
//!
//! - [`search_symbols_typed`] for symbol search candidates.
//! - [`symbol_info_typed`] for Desktop-free symbol metadata.
//! - [`fundamentals_symbol_typed`] for scanner-backed fundamental fields.
//! - [`events_symbol_typed`] for event-shaped earnings/dividends readback.
//! - [`quote_symbol_typed`] for one scanner-backed quote.
//! - [`quote_symbols_typed`] for ordered batch quotes.
//! - [`compare_symbols_typed`] for ordered multi-symbol comparison packets.
//! - [`bars_symbol`] for CLI-compatible browserless recent historical bars.
//! - [`bars_symbol_range`] for CLI-compatible browserless daily range bars.
//!
//! The older JSON-returning functions remain public for CLI payload
//! compatibility. New Rust integration code should usually use the typed
//! functions and serialize explicitly only at its own boundary.
//!
//! ```no_run
//! # async fn example() -> Result<(), tradingview_core::AppError> {
//! let quote = tradingview_market::quote_symbol_typed("NYSE:IONQ").await?;
//! println!("{} {:?}", quote.symbol, quote.last);
//! # Ok(())
//! # }
//! ```
//!
//! ```no_run
//! # async fn example() -> Result<(), tradingview_core::AppError> {
//! let batch = tradingview_market::quote_symbols_typed(vec![
//!     "AAPL".to_string(),
//!     "MSFT".to_string(),
//!     "NYSE:IONQ".to_string(),
//! ])
//! .await?;
//!
//! for item in batch.items {
//!     if let Some(quote) = item.quote {
//!         println!("{} -> {}", item.requested_symbol, quote.symbol);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod bars;
mod compare;
mod events;
mod fundamentals;
mod info;
mod normalize;
mod quote;
mod search;
mod snapshot;
mod types;

pub use bars::{bars_symbol, bars_symbol_range};
pub use compare::{compare_symbols, compare_symbols_typed};
pub use events::{
    events_compare_symbols, events_compare_symbols_typed, events_symbol, events_symbol_typed,
};
pub use fundamentals::{
    fundamentals_symbol, fundamentals_symbol_typed, fundamentals_symbol_with_groups,
    fundamentals_symbol_with_groups_typed, validate_fundamentals_selection,
};
pub use info::{symbol_info, symbol_info_typed};
pub use quote::{quote_symbol, quote_symbol_typed, quote_symbols, quote_symbols_typed};
pub use search::{search_symbols_typed, symbol_search};
pub use snapshot::{snapshot_symbol, snapshot_symbol_typed};
pub use types::{
    BatchQuoteItem, BatchQuotes, Compare, CompareFieldCoverage, CompareFollowUpHint, CompareItem,
    CompareItemError, CompareMissingEvidence, CompareMissingSummary, CompareResolvedSymbol,
    CompareSummary, EventEntry, EventFieldReadback, EventSourceAvailability, Events,
    ExtendedHoursQuote, FreshnessCheck, Fundamentals, Quote, QuoteError, SessionQuote, Snapshot,
    SnapshotSection, SnapshotSectionError, SnapshotSections, SymbolInfo, SymbolSearchResponse,
    SymbolSearchResult,
};
