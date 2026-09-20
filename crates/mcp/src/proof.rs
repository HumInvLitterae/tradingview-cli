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
