//! Opt-in connection proof, not the public bars contract or an arbitrary MCP proxy.

use crate::{
    Failure, Result,
    admission::{Admission, now_ms, write_private_json},
    auth::Auth,
    budget::Budget,
    credentials::Store,
    http::{Endpoints, Http},
};
use rmcp::model::CallToolResult;
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::Instant;

/// The caller explicitly chooses each effect; no startup discovery or login.
#[derive(Clone, Copy)]
pub enum ProofOperation {
    EconomicCommands,
    EconomicCodesShape,
    EconomicSeriesShape,
    EconomicOverviewShape,
    EconomicSymbolsShape,
    EconomicCalendarShape,
    DividendsShape,
    DividendScreenShape,
    ResearchCommands,
    StoryShape,
    DocumentShape,
    NewsShape,
    DocumentsShape,
    FinancialCommands,
    FinancialShape,
    HistoryShape,
    ForecastShape,
    EarningsShape,
    Discover,
    Login,
    Status,
    AuthorizeStore,
    ReadDaily,
    ReadAll,
    ReadWeekly,
    ReadMonthly,
    IntradayCommand,
    AccountLists,
    AccountCommands,
    AlertCatalog,
    AlertLifecycle,
    WatchlistCatalog,
    WatchlistLifecycle,
    Refresh,
    Search,
    Columns,
    ColumnsOverview,
    Symbol,
    Symbols,
    SymbolsCommand,
    Screener,
    ScreenerCommand,
    ScreenerEmptyCommand,
    Logout,
}

pub async fn run_proof(directory: &Path, operation: ProofOperation) -> Result<Value> {
    run_proof_with_worker(directory, operation, None).await
}

/// Development-only override for an explicitly selected, already-authorized
/// immutable worker. Never discover or execute a helper from a state directory.
pub async fn run_proof_with_worker(
    directory: &Path,
    operation: ProofOperation,
    worker: Option<&Path>,
) -> Result<Value> {
    if matches!(operation, ProofOperation::EconomicCommands) {
        return verify_economic_commands(directory, worker).await;
    }
    if matches!(operation, ProofOperation::ResearchCommands) {
        return verify_research_commands(directory, worker).await;
    }
    if matches!(operation, ProofOperation::FinancialCommands) {
        return verify_financial_commands(directory, worker).await;
    }
    if matches!(operation, ProofOperation::AlertLifecycle) {
        return verify_alert_lifecycle(directory, worker).await;
    }
    if matches!(operation, ProofOperation::WatchlistLifecycle) {
        return verify_watchlist_lifecycle(directory, worker).await;
    }
    if matches!(operation, ProofOperation::AccountCommands) {
        return verify_account_commands(directory, worker).await;
    }
    if matches!(operation, ProofOperation::IntradayCommand) {
        return verify_intraday_command(directory, worker).await;
    }
    if matches!(
        operation,
        ProofOperation::SymbolsCommand
            | ProofOperation::ScreenerCommand
            | ProofOperation::ScreenerEmptyCommand
    ) {
        return verify_data_command(directory, worker, operation).await;
    }
    let deadline = Instant::now()
        + if matches!(
            operation,
            ProofOperation::Login | ProofOperation::AuthorizeStore
        ) {
            Duration::from_secs(300)
        } else if matches!(
            operation,
            ProofOperation::EconomicCodesShape
                | ProofOperation::EconomicSeriesShape
                | ProofOperation::EconomicOverviewShape
                | ProofOperation::EconomicSymbolsShape
                | ProofOperation::EconomicCalendarShape
                | ProofOperation::DividendsShape
                | ProofOperation::DividendScreenShape
        ) {
            // Schema investigation only; public commands retain their 30-second deadline.
            Duration::from_secs(180)
        } else {
            Duration::from_secs(30)
        };
    let mut admission = Admission::acquire(directory, deadline).await?;
    let endpoints = Endpoints::tradingview();
    let store = match worker {
        Some(worker) => Store::native_worker(endpoints.clone(), deadline, worker.to_owned())?,
        None => Store::native(endpoints.clone(), deadline)?,
    };
    if matches!(operation, ProofOperation::AuthorizeStore) {
        store.authorize_access().await?;
        return Ok(json!({
            "local_credential_access_authorized": true,
            "provider_requests": 0
        }));
    }
    if matches!(operation, ProofOperation::Status) {
        let record = store.load_record().await?;
        return Ok(json!({
            "local_credentials_present": record.is_some(),
            "provider_acceptance": "unconfirmed"
        }));
    }
    if matches!(operation, ProofOperation::Logout) {
        store.clear_record().await?;
        return Ok(json!({"local_credentials_removed": true, "remote_revocation": false}));
    }
    let budget = Arc::new(Mutex::new(Budget::open(
        directory,
        matches!(operation, ProofOperation::Login),
    )?));
    let http = Http::new(endpoints, deadline, budget.clone())?;
    let outcome = async {
        if matches!(operation, ProofOperation::Login) && store.load_record().await?.is_some() {
            return Err(Failure::AuthRequired);
        }
        let mut auth = Auth::discover(http.clone(), store.clone(), budget.clone()).await?;

        match operation {
            ProofOperation::EconomicCodesShape
            | ProofOperation::EconomicSeriesShape
            | ProofOperation::EconomicOverviewShape
            | ProofOperation::EconomicSymbolsShape
            | ProofOperation::EconomicCalendarShape
            | ProofOperation::DividendsShape
            | ProofOperation::DividendScreenShape => {
                inspect_economics(operation, &mut auth, &mut admission, &budget).await
            }
            ProofOperation::NewsShape | ProofOperation::DocumentsShape | ProofOperation::StoryShape | ProofOperation::DocumentShape => {
                inspect_research(operation, &mut auth, &mut admission, &budget).await
            }
            ProofOperation::FinancialShape | ProofOperation::HistoryShape | ProofOperation::ForecastShape | ProofOperation::EarningsShape => {
                use tradingview_model::mcp_financials::Request;
                auth.restore().await?;
                let token = auth.token().await?;
                let request = match operation {
                    ProofOperation::FinancialShape => Request::snapshot("NASDAQ:AAPL", "ttm", &[]),
                    ProofOperation::HistoryShape => Request::history("NASDAQ:AAPL", "fq", Some("2025-01-01"), Some("2026-09-21")),
                    ProofOperation::ForecastShape => Request::forecasts("NASDAQ:AAPL"),
                    _ => Request::earnings(&["NASDAQ:AAPL".into()], Some("2026-07-01"), Some("2026-12-31")),
                }.map_err(|_| Failure::UnsupportedCapability)?;
                let tool = crate::tools::Tool::from(request.kind());
                let mut responses = crate::transport::call(&http, token, tool, &[request.arguments()], Some(&mut admission)).await?;
                let value = crate::transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
                Ok(json!({"tool": tool.names()[0], "shape": response_shape(&value, 0)}))
            }
            ProofOperation::Discover => Ok(json!({
                "public_discovery": "validated",
                "registration": "not_attempted"
            })),
            ProofOperation::Login => {
                auth.browser_login().await?;
                let size = *store
                    .last_saved_bytes
                    .lock()
                    .map_err(|_| Failure::LocalState)?;
                Ok(json!({
                    "authorization_saved": true,
                    "record_bytes": size,
                    "windows_record_fits": size.map(crate::credentials::windows_record_fits),
                    "native_platform": std::env::consts::OS,
                    "other_platforms": "unverified"
                }))
            }
            ProofOperation::Refresh => {
                auth.restore().await?;
                auth.refresh().await?;
                Ok(json!({
                    "server_refresh_succeeded": true,
                    "rotated_record_saved": true,
                    "subsequent_read": "pending"
                }))
            }
            ProofOperation::Search
            | ProofOperation::Columns
            | ProofOperation::ColumnsOverview
            | ProofOperation::Symbol
            | ProofOperation::Symbols
            | ProofOperation::Screener => {
                use tradingview_model::mcp_data::Request;
                let request = match operation {
                    ProofOperation::Screener => Request::screener(tradingview_model::mcp_data::ScreenerOptions {
                        limit: 3,
                        columns: vec!["name".into(), "close".into(), "volume".into()],
                        filters: json!({"close": [1, null]}),
                        ..Default::default()
                    }),
                    ProofOperation::Search => Request::search("Apple", None),
                    ProofOperation::Columns => Request::columns(None, None, Some("volume")),
                    ProofOperation::ColumnsOverview => Request::columns(None, None, None),
                    ProofOperation::Symbols => Request::symbols(
                        &["NASDAQ:AAPL".into(), "NASDAQ:MSFT".into(), "NASDAQ:TVCLIINVALID".into()],
                        &["close".into(), "volume".into()],
                    ),
                    _ => Request::symbol(
                        "NASDAQ:AAPL",
                        &["close".into(), "volume".into(), "market_cap_basic".into()],
                    ),
                }
                .map_err(|_| Failure::UnsupportedCapability)?;
                auth.restore().await?;
                let token = auth.token().await?;
                let mut results = crate::transport::call(
                    &http,
                    token,
                    request.kind().into(),
                    &[request.arguments()],
                    Some(&mut admission),
                )
                .await?;
                let value = crate::transport::result_value(
                    results.pop().ok_or(Failure::InvalidResponse)??,
                )?;
                Ok(json!({
                    "response_shape": response_shape(&value, 0),
                    "batch_data_value_shapes": if matches!(operation, ProofOperation::Symbols) {
                        value.get("data").and_then(Value::as_object).map(|rows| {
                            rows.values().take(2).map(|row| response_shape(row, 0)).collect::<Vec<_>>()
                        })
                    } else { None },
                    "batch_data_keys_match_requested": if matches!(operation, ProofOperation::Symbols) {
                        value.get("data").and_then(Value::as_object).map(|rows| rows.keys().all(|name| {
                            request.arguments()["symbols"].as_array().is_some_and(|symbols| {
                                symbols.iter().any(|symbol| symbol.as_str() == Some(name))
                            })
                        }))
                    } else { None },
                    "reported_count": value.get("count").and_then(Value::as_u64),
                    "reported_missing_count": value.get("missing_count").and_then(Value::as_u64),
                    "provider_success": value.get("success").and_then(Value::as_bool),
                    "symbol_echo_matches": value.get("symbol").and_then(Value::as_str).map(|v| v == "NASDAQ:AAPL")
                }))
            }
            ProofOperation::AlertCatalog => {
                auth.restore().await?;
                crate::transport::inspect_alert_tools(&http, auth.token().await?).await
            }
            ProofOperation::WatchlistCatalog => {
                auth.restore().await?;
                crate::transport::inspect_watchlist_tools(&http, auth.token().await?).await
            }
            ProofOperation::AccountLists => {
                auth.restore().await?;
                let mut observations = Vec::new();
                for tool in [crate::tools::Tool::Watchlists, crate::tools::Tool::Alerts] {
                    let mut results = crate::transport::call(
                        &http,
                        auth.token().await?,
                        tool,
                        &[json!({})],
                        Some(&mut admission),
                    )
                    .await?;
                    let value = crate::transport::result_value(
                        results.pop().ok_or(Failure::InvalidResponse)??,
                    )?;
                    observations.push(json!({
                        "tool": tool.names()[0],
                        "response_shape": response_shape(&value, 0)
                    }));
                    let (rows_key, id_key, detail_tool) = if tool == crate::tools::Tool::Watchlists {
                        ("watchlists", "id", crate::tools::Tool::Watchlist)
                    } else {
                        ("alerts", "alert_id", crate::tools::Tool::AlertDetails)
                    };
                    if let Some(id) = value
                        .get(rows_key)
                        .and_then(Value::as_array)
                        .and_then(|rows| rows.first())
                        .and_then(|row| row.get(id_key))
                        .and_then(Value::as_u64)
                    {
                        let args = if detail_tool == crate::tools::Tool::Watchlist {
                            json!({"watchlist_id": id.to_string()})
                        } else {
                            json!({"alert_ids": [id]})
                        };
                        let mut results = crate::transport::call(
                            &http,
                            auth.token().await?,
                            detail_tool,
                            &[args],
                            Some(&mut admission),
                        )
                        .await?;
                        let detail = crate::transport::result_value(
                            results.pop().ok_or(Failure::InvalidResponse)??,
                        )?;
                        observations.push(json!({
                            "tool": detail_tool.names()[0],
                            "response_shape": response_shape(&detail, 0)
                        }));
                    }
                }
                Ok(json!({"lists": observations}))
            }
            ProofOperation::ReadAll => {
                auth.restore().await?;
                let token = auth.token().await?;
                read_intervals(&http, token, &["1D", "1W", "M"], Some(&mut admission)).await
            }
            ProofOperation::ReadDaily
            | ProofOperation::ReadWeekly
            | ProofOperation::ReadMonthly => {
                let interval = match operation {
                    ProofOperation::ReadWeekly => "1W",
                    ProofOperation::ReadMonthly => "M",
                    _ => "1D",
                };
                authenticated_read(&mut auth, &mut admission, interval).await
            }
            _ => Err(Failure::UnsupportedCapability),
        }
    }
    .await;
    if let Some(seconds) = http.cooldown() {
        admission.cooldown(seconds)?;
    }
    let counts = budget.lock().map_err(|_| Failure::LocalState)?.counts();
    let report = match outcome {
        Ok(data) => {
            json!({
                "success": data.get("failure").is_none(),
                "observation": data,
                "budget": counts
            })
        }
        Err(error) => {
            json!({
                "success": false,
                "error": error,
                "budget": counts,
                "http": http.diagnostics()
            })
        }
    };
    // Only closed, sanitized observations are persisted; no raw tool payloads.
    write_private_json(
        &directory.join(format!("observation-{}.json", now_ms()?)),
        &report,
    )?;
    Ok(report)
}

pub(crate) async fn authenticated_read(
    auth: &mut Auth,
    admission: &mut Admission,
    interval: &str,
) -> Result<Value> {
    auth.restore().await?;
    let token = auth.token().await?;
    admission.before_tool(auth.http.deadline).await?;
    let result = read(&auth.http, token, interval).await;
    if result == Err(Failure::AuthRequired) {
        // Refresh may prepare the next explicit invocation, never replay this one.
        return match auth.refresh().await {
            Ok(()) => Err(Failure::AuthRefreshedRetryRequired),
            Err(Failure::BudgetExhausted) => Err(Failure::AuthRequired),
            Err(error) => Err(error),
        };
    }
    result
}

pub(crate) async fn read(http: &Http, token: String, interval: &str) -> Result<Value> {
    let report = read_intervals(http, token, &[interval], None).await?;
    if let Some(error) = report.get("failure") {
        return Err(serde_json::from_value(error.clone()).map_err(|_| Failure::InvalidResponse)?);
    }
    report["observations"]
        .as_array()
        .and_then(|v| v.first())
        .cloned()
        .ok_or(Failure::InvalidResponse)
}

pub(crate) async fn read_intervals(
    http: &Http,
    token: String,
    intervals: &[&str],
    mut admission: Option<&mut Admission>,
) -> Result<Value> {
    if intervals.is_empty()
        || intervals.len() > 3
        || intervals.iter().any(|v| !matches!(*v, "1D" | "1W" | "M"))
    {
        return Err(Failure::UnsupportedCapability);
    }
    let requests = intervals
        .iter()
        .map(|interval| {
            tradingview_model::mcp_bars::Request::new(
                "NASDAQ:AAPL",
                if *interval == "M" { "1M" } else { interval },
                20,
            )
            .map_err(|_| Failure::UnsupportedCapability)
        })
        .collect::<Result<Vec<_>>>()?;
    let results = crate::transport::read(http, token, &requests, admission.take()).await?;
    let mut observations = Vec::new();
    for (interval, result) in intervals.iter().zip(results) {
        match result.and_then(|result| observe_result(result, interval)) {
            Ok(observation) => observations.push(observation),
            Err(error) => {
                return Ok(json!({
                    "all_reads_completed": false,
                    "observations": observations,
                    "failure": error,
                    "failed_interval": interval,
                    "automatic_retry": false
                }));
            }
        }
    }
    Ok(json!({"all_reads_completed": true, "observations": observations}))
}

fn observe_result(result: CallToolResult, interval: &str) -> Result<Value> {
    let structured = result.structured_content.is_some();
    let value = crate::transport::result_value(result)?;
    // Probe supported documented row candidates, but never release a guessed
    // normalized contract. Unknown wrappers are reported as unconfirmed shape.
    let rows = value
        .as_array()
        .or_else(|| value.get("bars").and_then(Value::as_array));
    let mut observation = json!({
        "symbol_requested": "NASDAQ:AAPL",
        "interval_requested": interval,
        "count_requested": 20,
        "received_at_unix_ms": now_ms()?,
        "structured_content": structured,
        "payload_kind": if value.is_array() {
            "array"
        } else if value.is_object() {
            "object"
        } else {
            "other"
        },
        "object_field_count": value.as_object().map(|v| v.len()),
        "documented_rows_observed": false,
        "identity": "unconfirmed",
        "delay": "unconfirmed",
        "adjustment": "unconfirmed",
        "session": "unconfirmed",
        "finality": "unconfirmed",
        "timestamp_semantics": "unconfirmed"
    });
    if let Some(object) = value.as_object() {
        let fields: Vec<_> = object
            .keys()
            .filter(|name| {
                name.len() <= 64 && name.bytes().all(|b| b.is_ascii_alphabetic() || b == b'_')
            })
            .collect();
        observation["response_field_names"] = json!(fields);
        observation["provider_symbol_echo_matches_request"] = value
            .get("symbol")
            .and_then(Value::as_str)
            .map(|symbol| json!(symbol == "NASDAQ:AAPL"))
            .unwrap_or(Value::Null);
        observation["provider_interval_echo_matches_request"] = value
            .get("interval")
            .and_then(Value::as_str)
            .map(|actual| json!(actual == interval))
            .unwrap_or(Value::Null);
    }
    if let Some(rows) = rows {
        let mut last = None;
        let mut first = None;
        let mut missing_volume = 0;
        if rows.len() > 20 {
            return Err(Failure::InvalidResponse);
        }
        for row in rows {
            let time = row
                .get("t")
                .and_then(Value::as_i64)
                .filter(|t| *t >= 0)
                .ok_or(Failure::InvalidResponse)?;
            if last.is_some_and(|t| time <= t) {
                return Err(Failure::InvalidResponse);
            }
            let number = |name| {
                row.get(name)
                    .and_then(Value::as_f64)
                    .filter(|v| v.is_finite())
                    .ok_or(Failure::InvalidResponse)
            };
            let (open, high, low, close) = (number("o")?, number("h")?, number("l")?, number("c")?);
            if high < open.max(close).max(low) || low > open.min(close) {
                return Err(Failure::InvalidResponse);
            }
            if row.get("v").is_none_or(Value::is_null) {
                missing_volume += 1;
            } else {
                number("v")?;
            }
            first.get_or_insert(time);
            last = Some(time);
        }
        observation["documented_rows_observed"] = json!(true);
        observation["bar_count"] = json!(rows.len());
        observation["first_time"] = json!(first);
        observation["last_time"] = json!(last);
        observation["missing_volume_count"] = json!(missing_volume);
        observation["count_status"] = json!(if rows.is_empty() {
            "empty"
        } else if rows.len() < 20 {
            "short"
        } else {
            "met"
        });
    }
    Ok(observation)
}

// Shape only: never persist provider values, prices, descriptions or raw payloads.
fn response_shape(value: &Value, depth: usize) -> Value {
    if depth > 6 {
        return json!("nested");
    }
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .take(40)
                .filter(|(name, _)| {
                    name.len() <= 128
                        && name
                            .bytes()
                            .all(|b| b.is_ascii_alphabetic() || b"_-.|".contains(&b))
                })
                .map(|(name, value)| (name.clone(), response_shape(value, depth + 1)))
                .collect(),
        ),
        Value::Array(items) => {
            json!({
                "array_length": items.len(),
                "sample_shapes": items.iter().take(2)
                    .map(|item| response_shape(item, depth + 1))
                    .collect::<Vec<_>>()
            })
        }
        Value::Null => json!("null"),
        Value::Bool(_) => json!("boolean"),
        Value::Number(_) => json!("number"),
        Value::String(_) => json!("string"),
    }
}

// Use the public application service with a previously authorized immutable
// worker. This proves the new read path without replacing a trusted executable.
async fn verify_data_command(
    directory: &Path,
    worker: Option<&Path>,
    operation: ProofOperation,
) -> Result<Value> {
    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let symbols = vec![
        "NASDAQ:AAPL".into(),
        "NASDAQ:MSFT".into(),
        "NASDAQ:TVCLIINVALID".into(),
    ];
    use tradingview_model::mcp_data::{Request, ScreenerOptions};
    let request = match operation {
        ProofOperation::SymbolsCommand => Request::symbols(&symbols, &["close".into(), "volume".into()]),
        _ => Request::screener(ScreenerOptions {
            limit: 3,
            columns: vec!["name".into(), "close".into(), "volume".into()],
            filters: json!({
                "close": [
                    if matches!(operation, ProofOperation::ScreenerEmptyCommand) { 1e15 } else { 1.0 },
                    null
                ]
            }),
            ..Default::default()
        }),
    }.map_err(|_| Failure::UnsupportedCapability)?;
    let result = crate::Client::with_paths(directory.to_owned(), worker.to_owned())
        .run(crate::Operation::Data(request))
        .await;
    match result {
        Ok(data) => {
            let items = data["items"].as_array().ok_or(Failure::InvalidResponse)?;
            if !matches!(operation, ProofOperation::SymbolsCommand) {
                return Ok(json!({
                    "success": true,
                    "contract_version": data["contract_version"],
                    "source": data["source"],
                    "client_observation": data["client_observation"],
                    "total_count": data["provider_observation"]["total_count"],
                    "tool_attempts": data["transport"]["tool_attempts"],
                    "only_requested_fields": items.iter().all(|item| item["fields"].as_object()
                        .is_some_and(|fields| {
                            fields.len() == 3 && fields.keys().all(|key| {
                                ["name", "close", "volume"].contains(&key.as_str())
                            })
                        }))
                }));
            }
            Ok(json!({
                "success": true,
                "contract_version": data["contract_version"],
                "source": data["source"],
                "client_observation": data["client_observation"],
                "tool_attempts": data["transport"]["tool_attempts"],
                "input_order_preserved": items.len() == symbols.len()
                    && items.iter().zip(&symbols)
                        .all(|(item, symbol)| item["requested_symbol"] == *symbol),
                "statuses": items.iter().map(|item| &item["status"]).collect::<Vec<_>>(),
                "returned_fields_status": items.iter().filter(|item| item["status"] == "returned")
                    .map(|item| &item["client_observation"]["fields_status"]).collect::<Vec<_>>(),
                "missing_fields_are_null": items.iter().filter(|item| item["status"] != "returned")
                    .all(|item| item["fields"].is_null())
            }))
        }
        Err(error) => {
            let details = error.details.unwrap_or(Value::Null);
            Ok(json!({
                "success": false,
                "code": details["code"],
                "stage": details["stage"],
                "tool_attempts": details["tool_attempts"]
            }))
        }
    }
}

async fn verify_intraday_command(directory: &Path, worker: Option<&Path>) -> Result<Value> {
    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let mut observations = Vec::new();
    for timeframe in ["1m", "5m", "15m", "30m", "1h", "4h"] {
        let request = tradingview_model::mcp_bars::Request::new("NASDAQ:AAPL", timeframe, 20)
            .map_err(|_| Failure::UnsupportedCapability)?;
        let result = crate::Client::with_paths(directory.to_owned(), worker.to_owned())
            .run(crate::Operation::Bars(request))
            .await;
        let data = match result {
            Ok(data) => data,
            Err(error) => {
                let details = error.details.unwrap_or(Value::Null);
                return Ok(json!({
                    "success": false,
                    "timeframe": timeframe,
                    "code": details["code"],
                    "reason": details["reason"],
                    "tool_attempts": details["tool_attempts"],
                    "completed": observations
                }));
            }
        };
        observations.push(json!({
            "timeframe": timeframe,
            "contract_version": data["contract_version"],
            "bar_count": data["client_observation"]["bar_count"],
            "count_status": data["client_observation"]["count_status"],
            "identity_match": data["client_observation"]["identity_match"],
            "interval_match": data["client_observation"]["interval_match"],
            "calendar_coverage": data["client_observation"]["calendar_coverage"],
            "tool_attempts": data["transport"]["tool_attempts"]
        }));
    }
    Ok(json!({"success": true, "observations": observations}))
}

async fn verify_account_commands(directory: &Path, worker: Option<&Path>) -> Result<Value> {
    use tradingview_model::mcp_account::{Kind, Request};
    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let client = crate::Client::with_paths(directory.to_owned(), worker.to_owned());
    let mut observations = Vec::new();
    for request in [Request::watchlists(), Request::alerts(None, None).unwrap()] {
        let kind = request.kind();
        let data = client
            .run(crate::Operation::Account(request))
            .await
            .map_err(|_| Failure::InvalidResponse)?;
        observations.push(json!({
            "contract_version": data["contract_version"],
            "tool_attempts": data["transport"]["tool_attempts"],
            "nonempty": data["items"].as_array().is_some_and(|items| !items.is_empty())
        }));
        if let Some(first) = data["items"].as_array().and_then(|items| items.first()) {
            let request = if kind == Kind::Watchlists {
                Request::watchlist(first["id"].as_str().ok_or(Failure::InvalidResponse)?)
            } else {
                Request::alert_details(&[first["alert_id"]
                    .as_u64()
                    .ok_or(Failure::InvalidResponse)?])
            }
            .map_err(|_| Failure::InvalidResponse)?;
            let detail = client
                .run(crate::Operation::Account(request))
                .await
                .map_err(|_| Failure::InvalidResponse)?;
            observations.push(json!({
                "contract_version": detail["contract_version"],
                "identity_match": detail["client_observation"]["identity_match"],
                "ids_status": detail["client_observation"]["ids_status"],
                "tool_attempts": detail["transport"]["tool_attempts"]
            }));
        }
    }
    Ok(json!({"success": true, "observations": observations}))
}

/// Explicitly opted-in account writes. Never run as part of a read proof.
async fn verify_watchlist_lifecycle(directory: &Path, worker: Option<&Path>) -> Result<Value> {
    use tradingview_model::mcp_account::WatchlistMutation;
    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let client = crate::Client::with_paths(directory.to_owned(), worker.to_owned());
    let started = now_ms()?;
    let name = format!("tv-cli-mcp-verification-{started}");
    let record = directory.join(format!("watchlist-verification-{started}.json"));
    let mut state = json!({"name": name, "target_id": null, "phase": "create_pending"});
    write_private_json(&record, &state)?;
    let create = WatchlistMutation::create(&name, &["NASDAQ:AAPL".into()])
        .map_err(|_| Failure::InvalidResponse)?;
    let result = client
        .run(crate::Operation::WatchlistMutation(create))
        .await;
    let created = match result {
        Ok(data) => data,
        Err(error) => {
            let details = error.details.unwrap_or(Value::Null);
            state["phase"] = json!("create_unconfirmed");
            state["error_code"] = details["code"].clone();
            write_private_json(&record, &state)?;
            return Ok(json!({
                "success": false,
                "phase": "create_unconfirmed",
                "code": details["code"],
                "mutation_status": details["mutation"]["status"]
            }));
        }
    };
    let Some(id) = created["target_id"].as_str().map(str::to_owned) else {
        state["phase"] = json!("create_id_unreported");
        write_private_json(&record, &state)?;
        return Ok(json!({"success": false, "phase": "create_id_unreported"}));
    };
    state["target_id"] = json!(id);
    state["phase"] = json!("created");
    write_private_json(&record, &state)?;
    if created["readback"]["status"] != "matched" {
        state["phase"] = json!("create_readback_unconfirmed");
        write_private_json(&record, &state)?;
        return Ok(json!({"success": false, "phase": "create_readback_unconfirmed"}));
    }

    let mut observations = vec![json!({"operation": "create", "readback": "matched"})];
    let renamed = format!("{name}-renamed");
    for request in [
        WatchlistMutation::update(&id, Some(&renamed), None),
        WatchlistMutation::symbols(&id, &["NASDAQ:MSFT".into()], false),
        WatchlistMutation::symbols(&id, &["NASDAQ:AAPL".into()], false),
        WatchlistMutation::symbols(&id, &["NASDAQ:MSFT".into()], true),
        WatchlistMutation::delete(&id),
    ] {
        let request = request.map_err(|_| Failure::InvalidResponse)?;
        let action = request.action_name();
        state["phase"] = json!(format!("{action}_pending"));
        write_private_json(&record, &state)?;
        let result = client
            .run(crate::Operation::WatchlistMutation(request))
            .await;
        let data = match result {
            Ok(data) => data,
            Err(error) => {
                let details = error.details.unwrap_or(Value::Null);
                state["phase"] = json!(format!("{action}_unconfirmed"));
                state["error_code"] = details["code"].clone();
                write_private_json(&record, &state)?;
                return Ok(json!({
                    "success": false,
                    "phase": state["phase"],
                    "code": details["code"],
                    "mutation_status": details["mutation"]["status"],
                    "completed": observations
                }));
            }
        };
        let status = data["readback"]["status"].as_str().unwrap_or("unconfirmed");
        observations.push(json!({"operation": action, "readback": status}));
        let expected = if action == "delete" {
            "not_reported"
        } else {
            "matched"
        };
        if status != expected {
            state["phase"] = json!(format!("{action}_readback_unconfirmed"));
            write_private_json(&record, &state)?;
            return Ok(
                json!({"success": false, "phase": state["phase"], "completed": observations}),
            );
        }
    }
    state["phase"] = json!("delete_replied_and_not_reported");
    write_private_json(&record, &state)?;
    Ok(json!({
        "success": true,
        "observations": observations,
        "cleanup_observation": "delete_replied_and_not_reported"
    }))
}

/// Opt-in disposable alert lifecycle. Recovery identity stays in private state.
async fn verify_alert_lifecycle(directory: &Path, worker: Option<&Path>) -> Result<Value> {
    use tradingview_model::mcp_account::{AlertAction, AlertMutation, AlertSettings};

    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let client = crate::Client::with_paths(directory.to_owned(), worker.to_owned());
    let started = now_ms()?;
    let name = format!("tv-cli-mcp-alert-verification-{started}");
    let record = directory.join(format!("alert-verification-{started}.json"));
    let mut state = json!({"name": name, "target_id": null, "phase": "create_pending"});
    write_private_json(&record, &state)?;
    let create = AlertMutation::create(
        "NASDAQ:AAPL",
        1_000_000_000.0,
        "greater",
        "1D",
        AlertSettings {
            name: Some(name.clone()),
            ..Default::default()
        },
    )
    .map_err(|_| Failure::InvalidResponse)?;
    let created = match client.run(crate::Operation::AlertMutation(create)).await {
        Ok(data) => data,
        Err(error) => {
            let details = error.details.unwrap_or(Value::Null);
            state["phase"] = json!("create_unconfirmed");
            state["error_code"] = details["code"].clone();
            write_private_json(&record, &state)?;
            return Ok(json!({
                "success": false,
                "phase": state["phase"],
                "code": details["code"],
                "mutation_status": details["mutation"]["status"]
            }));
        }
    };
    let Some(id) = created["target_ids"]
        .as_array()
        .filter(|ids| ids.len() == 1)
        .and_then(|ids| ids[0].as_u64())
    else {
        state["phase"] = json!("create_id_unreported");
        write_private_json(&record, &state)?;
        return Ok(json!({"success": false, "phase": state["phase"]}));
    };
    state["target_id"] = json!(id);
    state["phase"] = json!("created");
    write_private_json(&record, &state)?;
    let mut observations =
        vec![json!({"operation": "create", "readback": created["readback"]["status"]})];
    if created["readback"]["status"] != "matched" {
        state["phase"] = json!("create_readback_unconfirmed");
        write_private_json(&record, &state)?;
        return Ok(json!({
            "success": false,
            "phase": state["phase"],
            "completed": observations
        }));
    }

    for request in [
        AlertMutation::state(AlertAction::Stop, &[id]),
        AlertMutation::update(
            id,
            AlertSettings {
                name: Some(format!("{name}-renamed")),
                ..Default::default()
            },
        ),
        AlertMutation::state(AlertAction::Stop, &[id]),
        AlertMutation::state(AlertAction::Restart, &[id]),
        AlertMutation::state(AlertAction::Delete, &[id]),
    ] {
        let request = request.map_err(|_| Failure::InvalidResponse)?;
        let action = request.action_name();
        state["phase"] = json!(format!("{action}_pending"));
        write_private_json(&record, &state)?;
        let data = match client.run(crate::Operation::AlertMutation(request)).await {
            Ok(data) => data,
            Err(error) => {
                let details = error.details.unwrap_or(Value::Null);
                state["phase"] = json!(format!("{action}_unconfirmed"));
                state["error_code"] = details["code"].clone();
                write_private_json(&record, &state)?;
                return Ok(json!({
                    "success": false,
                    "phase": state["phase"],
                    "code": details["code"],
                    "mutation_status": details["mutation"]["status"],
                    "completed": observations
                }));
            }
        };
        let status = data["readback"]["status"].as_str().unwrap_or("unconfirmed");
        observations.push(json!({"operation": action, "readback": status}));
        let expected = if action == "delete" {
            "not_reported"
        } else {
            "matched"
        };
        if status != expected {
            state["phase"] = json!(format!("{action}_readback_unconfirmed"));
            write_private_json(&record, &state)?;
            return Ok(json!({
                "success": false,
                "phase": state["phase"],
                "completed": observations
            }));
        }
    }
    state["phase"] = json!("delete_replied_and_not_reported");
    write_private_json(&record, &state)?;
    Ok(json!({
        "success": true,
        "observations": observations,
        "cleanup_observation": state["phase"]
    }))
}

/// Same public service as CLI commands, with a fixed credential worker for development.
async fn verify_financial_commands(directory: &Path, worker: Option<&Path>) -> Result<Value> {
    use tradingview_model::mcp_financials::Request;

    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let client = crate::Client::with_paths(directory.to_owned(), worker.to_owned());
    let mut observations = Vec::new();
    for request in [
        Request::snapshot("NASDAQ:AAPL", "ttm", &[]),
        Request::snapshot("NASDAQ:AAPL", "fq", &["revenue".into(), "pe".into()]),
        Request::history("NASDAQ:AAPL", "fq", Some("2025-01-01"), Some("2026-09-21")),
        Request::history("NASDAQ:AAPL", "fy", Some("2022-01-01"), Some("2026-09-21")),
        Request::forecasts("NASDAQ:AAPL"),
        Request::earnings(
            &["NASDAQ:AAPL".into(), "NASDAQ:MSFT".into()],
            Some("2026-07-01"),
            Some("2026-12-31"),
        ),
        Request::earnings(
            &["NASDAQ:AAPL".into()],
            Some("2000-01-01"),
            Some("2000-01-02"),
        ),
    ] {
        let request = request.map_err(|_| Failure::UnsupportedCapability)?;
        let tool = crate::tools::Tool::from(request.kind());
        let data = match client.run(crate::Operation::Financial(request)).await {
            Ok(data) => data,
            Err(error) => {
                let details = error.details.unwrap_or(Value::Null);
                return Ok(json!({
                    "success": false,
                    "tool": tool.names()[0],
                    "code": details["code"],
                    "reason": details["reason"],
                    "completed": observations
                }));
            }
        };
        observations.push(json!({
            "tool": tool.names()[0],
            "contract": data["contract_version"],
            "symbol_status": data["client_observation"]["symbol_status"],
            "returned_field_count": data["client_observation"]["returned_field_count"],
            "returned_period_count": data["client_observation"]["returned_period_count"],
            "returned_event_count": data["client_observation"]["returned_count"],
            "currency_reported": data["provider_metadata"]["currency"].is_string(),
            "tool_attempts": data["transport"]["tool_attempts"]
        }));
    }
    Ok(json!({"success": true, "observations": observations}))
}

async fn verify_research_commands(directory: &Path, worker: Option<&Path>) -> Result<Value> {
    use tradingview_model::mcp_research::{DocumentOptions, Request};

    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let client = crate::Client::with_paths(directory.to_owned(), worker.to_owned());
    let mut reports = Vec::new();
    let result: Result<()> = async {
        let news = research_read(
            &client,
            Request::news("NASDAQ:AAPL", "en", 2, 0),
            &mut reports,
        )
        .await?;
        if let Some(id) = news.pointer("/items/0/id").and_then(Value::as_str) {
            research_read(
                &client,
                Request::story(id, "en", "non_pro", None),
                &mut reports,
            )
            .await?;
        } else {
            return Err(Failure::InvalidResponse);
        }
        if let Some(next) = news
            .pointer("/pagination/next_offset")
            .and_then(Value::as_u64)
        {
            research_read(
                &client,
                Request::news(
                    "NASDAQ:AAPL",
                    "en",
                    2,
                    u32::try_from(next).map_err(|_| Failure::InvalidResponse)?,
                ),
                &mut reports,
            )
            .await?;
        }
        research_read(
            &client,
            Request::news("NASDAQ:AAPL", "en", 2, 200),
            &mut reports,
        )
        .await?;
        let documents = research_read(
            &client,
            Request::documents(
                "NASDAQ:AAPL",
                DocumentOptions {
                    limit: Some(2),
                    ..Default::default()
                },
            ),
            &mut reports,
        )
        .await?;
        if let Some(id) = documents
            .pointer("/items/0/views/0/id")
            .and_then(Value::as_str)
        {
            research_read(&client, Request::document(id), &mut reports).await?;
        } else {
            return Err(Failure::InvalidResponse);
        }
        research_read(
            &client,
            Request::documents(
                "NASDAQ:AAPL",
                DocumentOptions {
                    category: Some("annual_reports".into()),
                    start_date: Some("2025-01-01T00:00:00Z".into()),
                    end_date: Some("2026-09-21T23:59:59Z".into()),
                    limit: Some(2),
                    ..Default::default()
                },
            ),
            &mut reports,
        )
        .await?;
        Ok(())
    }
    .await;
    Ok(json!({"success": result.is_ok(), "failure": result.err(), "observations": reports}))
}

async fn research_read(
    client: &crate::Client,
    request: std::result::Result<
        tradingview_model::mcp_research::Request,
        tradingview_core::AppError,
    >,
    reports: &mut Vec<Value>,
) -> Result<Value> {
    let request = request.map_err(|_| Failure::UnsupportedCapability)?;
    let tool = crate::tools::Tool::from(request.kind());
    let data = match client.run(crate::Operation::Research(request)).await {
        Ok(data) => data,
        Err(error) => {
            let details = error.details.unwrap_or(Value::Null);
            reports.push(json!({
                "tool": tool.names()[0], "code": details["code"],
                "reason": details["reason"], "tool_attempts": details["tool_attempts"]
            }));
            return Err(Failure::InvalidResponse);
        }
    };
    reports.push(json!({
        "tool": tool.names()[0],
        "contract": data["contract_version"],
        "returned_count": data["client_observation"]["returned_count"],
        "content_status": data["client_observation"]["content_status"],
        "id_echo_matches": data["client_observation"]["id_echo_matches"],
        "has_more": data["pagination"]["has_more"],
        "tool_attempts": data["transport"]["tool_attempts"]
    }));
    Ok(data)
}

async fn inspect_research(
    operation: ProofOperation,
    auth: &mut Auth,
    admission: &mut Admission,
    budget: &Arc<Mutex<Budget>>,
) -> Result<Value> {
    use tradingview_model::mcp_research::{DocumentOptions, Request};
    auth.restore().await?;
    let http = auth.http.clone();
    let request = if matches!(
        operation,
        ProofOperation::NewsShape | ProofOperation::StoryShape
    ) {
        Request::news("NASDAQ:AAPL", "en", 2, 0)
    } else {
        Request::documents(
            "NASDAQ:AAPL",
            DocumentOptions {
                limit: Some(2),
                ..Default::default()
            },
        )
    }
    .map_err(|_| Failure::UnsupportedCapability)?;
    let tool = crate::tools::Tool::from(request.kind());
    let mut responses = crate::transport::call(
        &http,
        auth.token().await?,
        tool,
        &[request.arguments()],
        Some(admission),
    )
    .await?;
    let value = crate::transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
    if matches!(
        operation,
        ProofOperation::StoryShape | ProofOperation::DocumentShape
    ) {
        let request = if matches!(operation, ProofOperation::StoryShape) {
            Request::story(
                value
                    .pointer("/data/headlines/0/id")
                    .and_then(Value::as_str)
                    .ok_or(Failure::InvalidResponse)?,
                "en",
                "non_pro",
                None,
            )
        } else {
            Request::document(
                value
                    .pointer("/items/0/views/0/id")
                    .and_then(Value::as_str)
                    .ok_or(Failure::InvalidResponse)?,
            )
        };
        let request = match request {
            Ok(request) => request,
            Err(error) => {
                return Ok(json!({
                    "failure": "unsupported_capability",
                    "request_reason": error.details.and_then(|v| v.get("reason").cloned()),
                    "id_has_urn_prefix": value.pointer("/data/headlines/0/id").and_then(Value::as_str).map(|v| v.starts_with("urn:")),
                    "id_has_whitespace": value.pointer("/data/headlines/0/id").and_then(Value::as_str).map(|v| v.chars().any(char::is_whitespace))
                }));
            }
        };
        let tool = crate::tools::Tool::from(request.kind());
        // The list and detail are distinct operations in this development proof.
        let detail_http = Http::new(
            Endpoints::tradingview(),
            Instant::now() + Duration::from_secs(30),
            budget.clone(),
        )?;
        let responses = crate::transport::call(
            &detail_http,
            auth.token().await?,
            tool,
            &[request.arguments()],
            Some(admission),
        )
        .await;
        if let Some(wait) = detail_http.cooldown() {
            admission.cooldown(wait)?;
        }
        let mut responses = responses?;
        let value =
            crate::transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
        Ok(json!({"tool": tool.names()[0], "shape": response_shape(&value, 0)}))
    } else {
        Ok(json!({"tool": tool.names()[0], "shape": response_shape(&value, 0)}))
    }
}

async fn inspect_economics(
    operation: ProofOperation,
    auth: &mut Auth,
    admission: &mut Admission,
    budget: &Arc<Mutex<Budget>>,
) -> Result<Value> {
    use tradingview_model::mcp_economics::{CalendarOptions, DividendOptions, Request};
    auth.restore().await?;
    let request = match operation {
        ProofOperation::EconomicCodesShape => {
            Request::symbols(None, Some("prce"), Some("inflation"))
        }
        ProofOperation::EconomicOverviewShape => Request::symbols(None, None, None),
        ProofOperation::EconomicSymbolsShape | ProofOperation::EconomicSeriesShape => {
            Request::symbols(Some("US"), None, Some("inflation"))
        }
        ProofOperation::EconomicCalendarShape => Request::calendar(CalendarOptions {
            from: Some("2026-09-17".into()),
            to: Some("2026-09-18".into()),
            min_importance: Some(1),
            ..Default::default()
        }),
        ProofOperation::DividendsShape => Request::dividends(DividendOptions {
            symbols: vec!["NASDAQ:AAPL".into(), "NASDAQ:MSFT".into()],
            ..Default::default()
        }),
        _ => Request::dividends(DividendOptions {
            market: Some("america".into()),
            from: Some("2026-09-21".into()),
            to: Some("2026-09-25".into()),
            limit: Some(2),
            ..Default::default()
        }),
    }
    .map_err(|_| Failure::UnsupportedCapability)?;
    let tool = crate::tools::Tool::from(request.kind());
    let mut responses = crate::transport::call(
        &auth.http,
        auth.token().await?,
        tool,
        &[request.arguments()],
        Some(admission),
    )
    .await?;
    let value = crate::transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
    if matches!(operation, ProofOperation::EconomicSeriesShape) {
        let symbol = value["symbols"]
            .as_array()
            .ok_or(Failure::InvalidResponse)?
            .iter()
            .find_map(|row| row["symbol"].as_str().filter(|s| *s == "ECONOMICS:USIRYY"))
            .ok_or(Failure::UnsupportedCapability)?;
        let request = Request::series(symbol, Some("2025-01-01"), Some("2026-09-21"))
            .map_err(|_| Failure::UnsupportedCapability)?;
        let http = Http::new(
            Endpoints::tradingview(),
            Instant::now() + Duration::from_secs(180),
            budget.clone(),
        )?;
        let tool = crate::tools::Tool::from(request.kind());
        let response = crate::transport::call(
            &http,
            auth.token().await?,
            tool,
            &[request.arguments()],
            Some(admission),
        )
        .await;
        if let Some(wait) = http.cooldown() {
            admission.cooldown(wait)?;
        }
        let mut responses = response?;
        let value =
            crate::transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
        Ok(json!({
            "tool": tool.names()[0], "catalog_symbol_confirmed": true,
            "shape": response_shape(&value, 0),
            "normalization": economic_normalization(&request, &value),
            "status_is_ok": value["status"] == "ok",
            "status_is_success": value["status"] == "success"
        }))
    } else {
        Ok(json!({
            "tool": tool.names()[0], "shape": response_shape(&value, 0),
            "normalization": economic_normalization(&request, &value),
            "status_is_ok": value["status"] == "ok",
            "status_is_success": value["status"] == "success"
        }))
    }
}

async fn verify_economic_commands(directory: &Path, worker: Option<&Path>) -> Result<Value> {
    use tradingview_model::mcp_economics::{CalendarOptions, DividendOptions, Request};

    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let client = crate::Client::with_paths(directory.to_owned(), worker.to_owned());
    let mut reports = Vec::new();
    let mut complete = true;
    for request in [
        Request::symbols(None, None, None),
        Request::symbols(None, Some("prce"), Some("inflation")),
        Request::symbols(Some("US"), None, Some("inflation")),
        Request::calendar(CalendarOptions {
            from: Some("2026-09-17".into()),
            to: Some("2026-09-18".into()),
            min_importance: Some(1),
            ..Default::default()
        }),
        Request::dividends(DividendOptions {
            symbols: vec!["NASDAQ:AAPL".into(), "NASDAQ:MSFT".into()],
            ..Default::default()
        }),
        Request::dividends(DividendOptions {
            market: Some("america".into()),
            from: Some("2026-09-21".into()),
            to: Some("2026-09-25".into()),
            limit: Some(2),
            ..Default::default()
        }),
    ] {
        let request = request.map_err(|_| Failure::UnsupportedCapability)?;
        let data = economic_read(&client, request, &mut reports).await;
        complete &= data.is_some();
        if let Some(data) = data.filter(|data| data["mode"] == "symbols") {
            let symbol = data["items"].as_array().and_then(|items| {
                items
                    .iter()
                    .find_map(|item| item["symbol"].as_str().filter(|s| *s == "ECONOMICS:USIRYY"))
            });
            if let Some(symbol) = symbol {
                let request = Request::series(symbol, Some("2025-01-01"), Some("2026-09-21"))
                    .map_err(|_| Failure::UnsupportedCapability)?;
                complete &= economic_read(&client, request, &mut reports)
                    .await
                    .is_some();
            } else {
                complete = false;
            }
        }
    }
    Ok(json!({"success": complete, "observations": reports}))
}

async fn economic_read(
    client: &crate::Client,
    request: tradingview_model::mcp_economics::Request,
    reports: &mut Vec<Value>,
) -> Option<Value> {
    let tool = crate::tools::Tool::from(request.kind());
    match client.run(crate::Operation::Economic(request)).await {
        Ok(data) => {
            reports.push(json!({
                "tool": tool.names()[0],
                "contract": data["contract_version"],
                "mode": data["mode"],
                "returned_count": data["client_observation"]["returned_count"],
                "tool_attempts": data["transport"]["tool_attempts"]
            }));
            Some(data)
        }
        Err(error) => {
            let details = error.details.unwrap_or(Value::Null);
            reports.push(json!({
                "tool": tool.names()[0], "code": details["code"],
                "reason": details["reason"], "stage": details["stage"], "tool_attempts": details["tool_attempts"]
            }));
            None
        }
    }
}

fn economic_normalization(
    request: &tradingview_model::mcp_economics::Request,
    value: &Value,
) -> Value {
    match tradingview_model::mcp_economics::normalize(request, value.clone(), 1000) {
        Ok(data) => json!({"success": true, "contract": data["contract_version"]}),
        Err(error) => {
            let details = error.details.unwrap_or(Value::Null);
            let message = value["error"]
                .as_str()
                .unwrap_or_default()
                .to_ascii_lowercase();
            let indications: Vec<_> = [
                "429",
                "403",
                "401",
                "500",
                "502",
                "503",
                "timeout",
                "rate limit",
                "quota",
                "permission",
            ]
            .into_iter()
            .filter(|needle| message.contains(needle))
            .collect();
            json!({
                "success": false, "code": details["code"], "reason": details["reason"],
                "provider_error_indications": indications
            })
        }
    }
}
