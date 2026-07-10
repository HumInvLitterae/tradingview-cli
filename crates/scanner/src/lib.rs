//! Desktop-free TradingView scanner reads.
//!
//! This crate contains credential-free, read-only helpers for scanner hotlists,
//! table scans, and scanner field metadata. It does not connect to TradingView
//! Desktop, CDP, chart state, UI automation, or account mutation paths.
//!
//! Prefer the typed functions for Rust callers:
//!
//! - [`scanner_hotlist_typed`] for preset hotlist reads.
//! - [`scanner_scan_typed`] for scanner table reads.
//! - [`scanner_metainfo_typed`] for scanner field metadata discovery.
//!
//! The older JSON-returning functions remain public for CLI payload
//! compatibility. New Rust integration code should usually use the typed
//! functions and serialize explicitly only at its own boundary.
//!
//! ```no_run
//! # async fn example() -> Result<(), tradingview_core::AppError> {
//! let request = tradingview_scanner::ScannerScanRequest {
//!     market: "america".to_string(),
//!     exchanges: Vec::new(),
//!     columns: Some("name,close,premarket_close".to_string()),
//!     sort: None,
//!     asc: false,
//!     desc: false,
//!     limit: Some(3),
//!     min_price: None,
//!     max_price: None,
//!     min_volume: None,
//!     min_market_cap: None,
//!     sectors: Vec::new(),
//!     industries: Vec::new(),
//!     symbol_types: Vec::new(),
//!     subtypes: Vec::new(),
//!     min_change: None,
//!     max_change: None,
//!     min_relative_volume: None,
//!     max_pe: None,
//!     min_average_volume: None,
//!     min_performance_week: None,
//!     max_performance_week: None,
//!     min_performance_month: None,
//!     max_performance_month: None,
//!     min_performance_quarter: None,
//!     max_performance_quarter: None,
//!     min_rsi: None,
//!     max_rsi: None,
//!     min_recommendation: None,
//!     max_recommendation: None,
//! };
//!
//! let result = tradingview_scanner::scanner_scan_typed(request).await?;
//! println!("{} rows", result.count);
//! # Ok(())
//! # }
//! ```
//!
//! ```no_run
//! # async fn example() -> Result<(), tradingview_core::AppError> {
//! let request = tradingview_scanner::ScannerMetainfoRequest {
//!     market: "america".to_string(),
//!     fields: vec!["close".to_string(), "premarket_close".to_string()],
//! };
//!
//! let result = tradingview_scanner::scanner_metainfo_typed(request).await?;
//! println!("{} fields", result.field_count);
//! # Ok(())
//! # }
//! ```

mod common;
mod hotlist;
mod http;
mod metainfo;
mod scan;
mod types;

pub use hotlist::{scanner_hotlist, scanner_hotlist_typed};
pub use metainfo::{ScannerMetainfoRequest, scanner_metainfo, scanner_metainfo_typed};
pub use scan::{ScannerScanRequest, scanner_scan, scanner_scan_typed};
pub use types::{
    ScannerFieldInfo, ScannerHotlistResult, ScannerMetainfoResult, ScannerRow, ScannerScanResult,
    ScannerSort,
};
