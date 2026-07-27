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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BarsFailureStage {
    SymbolSearch,
    RequestPrepare,
    WebSocketConnect,
    SessionSetup,
    SeriesSetup,
    ResponseWait,
    Protocol,
    HeartbeatSend,
    Pagination,
    SourceResult,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "reviewed fail-closed vocabulary; normal production paths use a specific stage"
        )
    )]
    SourceUnknown,
}

impl BarsFailureStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::SymbolSearch => "symbol_search",
            Self::RequestPrepare => "request_prepare",
            Self::WebSocketConnect => "websocket_connect",
            Self::SessionSetup => "session_setup",
            Self::SeriesSetup => "series_setup",
            Self::ResponseWait => "response_wait",
            Self::Protocol => "protocol",
            Self::HeartbeatSend => "heartbeat_send",
            Self::Pagination => "pagination",
            Self::SourceResult => "source_result",
            Self::SourceUnknown => "source_unknown",
        }
    }
}

pub(super) fn with_source_failure_stage(mut error: AppError, stage: BarsFailureStage) -> AppError {
    let mut details = match error.details.take() {
        Some(Value::Object(details)) => details,
        Some(_) => {
            let mut details = serde_json::Map::new();
            details.insert("previous_details_omitted".to_string(), Value::Bool(true));
            details
        }
        None => serde_json::Map::new(),
    };
    details.insert(
        "source_failure_stage".to_string(),
        Value::String(stage.as_str().to_string()),
    );
    error.details = Some(Value::Object(details));
    error
}

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
        return Err(with_source_failure_stage(
            no_bars_error(&request, &result, elapsed_ms),
            BarsFailureStage::SourceResult,
        ));
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

    let search = search_symbols_typed_with_client(client, input_symbol)
        .await
        .map_err(|error| with_source_failure_stage(error, BarsFailureStage::SymbolSearch))?;
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
    fn source_failure_stages_use_the_closed_public_vocabulary() {
        for (stage, expected) in [
            (BarsFailureStage::SymbolSearch, "symbol_search"),
            (BarsFailureStage::RequestPrepare, "request_prepare"),
            (BarsFailureStage::WebSocketConnect, "websocket_connect"),
            (BarsFailureStage::SessionSetup, "session_setup"),
            (BarsFailureStage::SeriesSetup, "series_setup"),
            (BarsFailureStage::ResponseWait, "response_wait"),
            (BarsFailureStage::Protocol, "protocol"),
            (BarsFailureStage::HeartbeatSend, "heartbeat_send"),
            (BarsFailureStage::Pagination, "pagination"),
            (BarsFailureStage::SourceResult, "source_result"),
            (BarsFailureStage::SourceUnknown, "source_unknown"),
        ] {
            let error = with_source_failure_stage(
                AppError::new(ErrorKind::Connection, "fixed message"),
                stage,
            );
            assert_eq!(
                error.details.unwrap()["source_failure_stage"],
                expected,
                "{stage:?}"
            );
        }
    }

    #[test]
    fn source_failure_stage_preserves_object_details_and_omits_non_objects() {
        let error = with_source_failure_stage(
            AppError::new(ErrorKind::Timeout, "fixed message")
                .with_details(json!({"existing": "safe"})),
            BarsFailureStage::ResponseWait,
        );
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.message, "fixed message");
        assert_eq!(error.exit_code(), 4);
        assert_eq!(error.details.as_ref().unwrap()["existing"], "safe");
        assert_eq!(
            error.details.as_ref().unwrap()["source_failure_stage"],
            "response_wait"
        );

        let error = with_source_failure_stage(
            AppError::new(ErrorKind::Connection, "fixed message")
                .with_details(json!("private transport value")),
            BarsFailureStage::SourceUnknown,
        );
        let details = error.details.unwrap();
        assert_eq!(details["previous_details_omitted"], true);
        assert_eq!(details["source_failure_stage"], "source_unknown");
        assert!(!details.to_string().contains("private transport value"));
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
