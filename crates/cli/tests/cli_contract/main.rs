//! Deterministic contract tests for the `tv` binary.
//!
//! The suites share one test binary so a build links it once. Select one with
//! a module filter, for example `cargo test -p tradingview-cli --test
//! cli_contract quote::`.

mod support;

mod bars;
mod desktop;
mod diagnose;
mod general;
mod mcp;
mod offline;
mod output;
mod quote;
mod spec;
