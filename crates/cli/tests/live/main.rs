//! Opt-in live smokes and capability probes against TradingView Desktop or the
//! provider, plus the fixture tests for their own helpers.
//!
//! The suites share one test binary so a build links it once. Live tests stay
//! ignored; select one with a module filter, for example `cargo test -p
//! tradingview-cli --test live bars:: -- --ignored`.

mod after_hours_network_source;
mod after_hours_panel_source;
mod after_hours_widget_store_source;
mod after_hours_ws_correlation;
mod bars;
mod chart_quote;
mod chart_quote_concurrency;
mod compare;
mod consecutive_invocation_resilience;
mod indicator_search_reassessment;
mod observe_chart;
mod quote_data_source;
mod quote_session_extended_hours;
mod right_offset_capability;
mod snapshot;
mod support;
mod three_point_drawing_capability;
