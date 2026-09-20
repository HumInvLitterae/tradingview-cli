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
    Discover,
    Login,
    Status,
    AuthorizeStore,
    ReadDaily,
    ReadAll,
    ReadWeekly,
    ReadMonthly,
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
