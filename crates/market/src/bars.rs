mod payload;
mod protocol;
mod transport;
mod types;
mod validation;

use serde_json::Value;
use serde_json::json;
use tokio::time::Instant;
use tradingview_core::{AppError, ErrorKind};

use self::{
    payload::{bars_payload, no_bars_error},
    transport::fetch_bars_ws,
    types::BarsSymbolResolution,
    validation::{
        validate_bars_range_request_with_resolution, validate_bars_request_with_resolution,
    },
};
use crate::{
    http::configured_client, search::search_symbols_typed_with_client, types::SymbolSearchResponse,
};

pub async fn bars_symbol(symbol: &str, timeframe: &str, count: usize) -> Result<Value, AppError> {
    let client = configured_client()?;
    let symbol_resolution = resolve_bars_symbol(&client, symbol).await?;
    let resolved_symbol = symbol_resolution.resolved_symbol.clone();
    let request = validate_bars_request_with_resolution(
        symbol,
        &resolved_symbol,
        symbol_resolution,
        timeframe,
        count,
    )?;
    bars_for_request(request).await
}

pub async fn bars_symbol_range(
    symbol: &str,
    timeframe: &str,
    from: &str,
    to: &str,
    count_cap: usize,
) -> Result<Value, AppError> {
    let client = configured_client()?;
    let symbol_resolution = resolve_bars_symbol(&client, symbol).await?;
    let resolved_symbol = symbol_resolution.resolved_symbol.clone();
    let request = validate_bars_range_request_with_resolution(
        symbol,
        &resolved_symbol,
        symbol_resolution,
        timeframe,
        from,
        to,
        count_cap,
    )?;
    bars_for_request(request).await
}

async fn bars_for_request(request: self::types::BarsRequest) -> Result<Value, AppError> {
    let started = Instant::now();
    let result = fetch_bars_ws(&request).await?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    if result.bars.is_empty() {
        return Err(no_bars_error(&request, &result, elapsed_ms));
    }

    Ok(bars_payload(&request, result, elapsed_ms))
}

async fn resolve_bars_symbol(
    client: &reqwest::Client,
    input_symbol: &str,
) -> Result<BarsSymbolResolution, AppError> {
    let input_symbol = input_symbol.trim();
    if input_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars symbol must not be empty",
        ));
    }

    if input_symbol.contains(':') {
        return Ok(BarsSymbolResolution::input_exchange_qualified(input_symbol));
    }

    let search = search_symbols_typed_with_client(client, input_symbol).await?;
    resolve_bars_symbol_from_search(input_symbol, &search)
}

fn resolve_bars_symbol_from_search(
    input_symbol: &str,
    search: &SymbolSearchResponse,
) -> Result<BarsSymbolResolution, AppError> {
    let exact_candidate = search.results.iter().find(|candidate| {
        candidate.symbol.eq_ignore_ascii_case(input_symbol) && candidate.full_name.contains(':')
    });

    let Some(candidate) = exact_candidate else {
        let candidates = search.results.iter().take(10).collect::<Vec<_>>();
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars symbol could not be resolved; use EXCHANGE:SYMBOL",
        )
        .with_details(json!({
            "requested_symbol": input_symbol,
            "expected_format": "EXCHANGE:SYMBOL",
            "resolution_source": "symbol_search_rest",
            "resolution_status": "unresolved",
            "candidate_count": search.count,
            "candidates": candidates,
            "next_action_hint": "Run `tv search <SYMBOL>` and retry `tv bars <EXCHANGE:SYMBOL> ...` with the intended exchange-qualified symbol.",
        })));
    };

    Ok(BarsSymbolResolution::symbol_search(
        input_symbol,
        &candidate.full_name,
        search.count,
    ))
}

#[cfg(test)]
mod symbol_resolution_tests {
    use tradingview_core::ErrorKind;

    use super::*;
    use crate::types::SymbolSearchResult;

    fn search(results: Vec<SymbolSearchResult>) -> SymbolSearchResponse {
        SymbolSearchResponse {
            query: "AAPL".to_string(),
            source: "rest_api".to_string(),
            source_category: "desktop_free_read".to_string(),
            requires_desktop: false,
            non_mutating: true,
            count: results.len(),
            results,
        }
    }

    fn candidate(symbol: &str, exchange: &str) -> SymbolSearchResult {
        SymbolSearchResult {
            symbol: symbol.to_string(),
            description: format!("{symbol} description"),
            exchange: exchange.to_string(),
            symbol_type: "stock".to_string(),
            full_name: format!("{exchange}:{symbol}"),
        }
    }

    #[test]
    fn resolves_first_exact_symbol_candidate() {
        let search = search(vec![
            candidate("AAPL", "NASDAQ"),
            candidate("AAPL", "TSX"),
            candidate("AAPL34", "BMFBOVESPA"),
        ]);

        let resolution = resolve_bars_symbol_from_search("AAPL", &search).unwrap();

        assert_eq!(resolution.input_symbol, "AAPL");
        assert_eq!(resolution.resolved_symbol, "NASDAQ:AAPL");
        assert_eq!(resolution.resolution_source, "symbol_search_rest");
        assert_eq!(resolution.resolution_status, "resolved");
        assert_eq!(resolution.candidate_count, 3);
    }

    #[test]
    fn unresolved_bare_symbol_returns_public_safe_guidance() {
        let search = search(vec![candidate("AAPL34", "BMFBOVESPA")]);

        let err = resolve_bars_symbol_from_search("AAPL", &search).unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        let details = err.details.as_ref().expect("details");
        assert_eq!(details["requested_symbol"], "AAPL");
        assert_eq!(details["expected_format"], "EXCHANGE:SYMBOL");
        assert_eq!(details["resolution_source"], "symbol_search_rest");
        assert_eq!(details["resolution_status"], "unresolved");
        assert_eq!(details["candidate_count"], 1);
        assert_eq!(details["candidates"][0]["full_name"], "BMFBOVESPA:AAPL34");
        assert!(
            details["next_action_hint"]
                .as_str()
                .unwrap()
                .contains("tv search")
        );
    }
}
