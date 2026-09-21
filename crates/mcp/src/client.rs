//! Application service for the standalone `tv mcp` command group.

use crate::{
    Failure,
    admission::{Admission, now_ms},
    auth::Auth,
    budget::Budget,
    credentials::Store,
    http::{Endpoints, Http},
    transport,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::Instant;
use tradingview_core::{AppError, ErrorKind};
use tradingview_model::mcp_bars::{self, Request};

pub enum Operation {
    Login,
    Status,
    Logout,
    Economic(tradingview_model::mcp_economics::Request),
    Research(tradingview_model::mcp_research::Request),
    Financial(tradingview_model::mcp_financials::Request),
    Bars(Request),
    Data(tradingview_model::mcp_data::Request),
    Account(tradingview_model::mcp_account::Request),
    AlertMutation(tradingview_model::mcp_account::AlertMutation),
    WatchlistMutation(tradingview_model::mcp_account::WatchlistMutation),
}

/// Internal workspace service; no external stable Rust API is promised.
pub struct Client {
    directory: PathBuf,
    worker: PathBuf,
}

impl Client {
    pub fn current_user() -> Result<Self, AppError> {
        let directory = state_directory().map_err(|e| failure(e, "local_state", 0, None))?;
        let worker = std::env::current_exe()
            .map_err(|_| failure(Failure::StorageUnavailable, "credentials", 0, None))?;
        Ok(Self::with_paths(directory, worker))
    }

    /// Explicit paths for the development harness. Production CLI has no helper
    /// override, endpoint override, or automatic helper discovery.
    #[doc(hidden)]
    pub fn with_paths(directory: PathBuf, worker: PathBuf) -> Self {
        Self { directory, worker }
    }

    pub async fn run(&self, operation: Operation) -> Result<Value, AppError> {
        let deadline = Instant::now()
            + Duration::from_secs(if matches!(operation, Operation::Login) {
                300
            } else {
                30
            });
        let mut admission = Admission::acquire(&self.directory, deadline)
            .await
            .map_err(|e| failure(e, "local_admission", 0, None))?;
        if matches!(
            operation,
            Operation::Bars(_)
                | Operation::Data(_)
                | Operation::Financial(_)
                | Operation::Research(_)
                | Operation::Economic(_)
                | Operation::Account(_)
                | Operation::WatchlistMutation(_)
                | Operation::AlertMutation(_)
        ) {
            admission
                .check_cooldown()
                .map_err(|e| failure(e, "local_admission", 0, None))?;
        }
        let store = Store::native_worker(Endpoints::tradingview(), deadline, self.worker.clone())
            .map_err(|e| failure(e, "credentials", 0, None))?;
        if matches!(operation, Operation::Logout) {
            store
                .clear_record()
                .await
                .map_err(|e| failure(e, "credentials", 0, None))?;
            return Ok(json!({
                "operation": "logout",
                "local_credentials_removed": true,
                "remote_revocation": false
            }));
        }
        let record = match store.load_record().await {
            Err(Failure::StorageInteractionRequired) if matches!(operation, Operation::Login) => {
                eprintln!(
                    "The OS will ask to let tv access its dedicated credential. \
                     On macOS choose Always Allow for this executable to enable \
                     later noninteractive reads."
                );
                store
                    .authorize_access()
                    .await
                    .map_err(|e| failure(e, "credentials", 0, None))?;
                store
                    .load_record()
                    .await
                    .map_err(|e| failure(e, "credentials", 0, None))?
            }
            result => result.map_err(|e| failure(e, "credentials", 0, None))?,
        };
        if matches!(operation, Operation::Status) {
            let expires_at = record.as_ref().and_then(|r| {
                let value = serde_json::to_value(r).ok()?;
                r.token_received_at?
                    .checked_add(value.pointer("/token_response/expires_in")?.as_u64()?)
            });
            let now = now_ms().map_err(|e| failure(e, "local_state", 0, None))? / 1000;
            return Ok(json!({
                "operation": "status",
                "credentials_present": record.is_some(),
                "expires_at_unix_seconds": expires_at,
                "locally_expired": expires_at.map(|expiry| now >= expiry),
                "provider_acceptance": "unconfirmed",
                "next_action": if record.is_none() { Some("tv mcp login") } else { None }
            }));
        }
        if record.is_none() && !matches!(operation, Operation::Login) {
            return Err(failure(Failure::AuthRequired, "credentials", 0, None));
        }
        let budget = Arc::new(Mutex::new(
            Budget::open(&self.directory, true).map_err(|e| failure(e, "local_state", 0, None))?,
        ));
        let http = Http::new(Endpoints::tradingview(), deadline, budget.clone())
            .map_err(|e| failure(e, "discovery", 0, None))?;
        execute(
            operation,
            &mut admission,
            store,
            http,
            budget,
            record.is_some(),
        )
        .await
    }
}

pub(crate) async fn execute(
    operation: Operation,
    admission: &mut Admission,
    store: Store,
    http: Http,
    budget: Arc<Mutex<Budget>>,
    has_record: bool,
) -> Result<Value, AppError> {
    let before = budget
        .lock()
        .map_err(|_| failure(Failure::LocalState, "local_state", 0, None))?
        .counts()
        .tools;
    let mut stage = "discovery";
    let mut result: Result<Value, AppError> = async {
        let mut auth = Auth::discover(http.clone(), store, budget.clone())
            .await
            .map_err(AppError::from)?;
        stage = "authentication";
        if matches!(operation, Operation::Login) {
            let reusable = if has_record {
                match auth.restore().await {
                    Ok(()) => match auth.token().await {
                        Ok(_) => true,
                        Err(Failure::AuthRequired) => false,
                        Err(error) => return Err(error.into()),
                    },
                    Err(Failure::AuthRequired) => false,
                    Err(error) => return Err(error.into()),
                }
            } else {
                false
            };
            if !reusable {
                auth.browser_login().await.map_err(AppError::from)?;
            }
            return Ok(json!({
                "operation": "login",
                "credentials_saved": true,
                "reused_credentials": reusable,
                "provider_acceptance": "unconfirmed"
            }));
        }
        auth.restore().await.map_err(AppError::from)?;
        let token = auth.token().await.map_err(AppError::from)?;
        stage = "tool_response";
        if let Operation::WatchlistMutation(request) = &operation {
            stage = "watchlist_mutation";
            return crate::watchlist::change(request, &http, token, admission).await;
        }
        if let Operation::AlertMutation(request) = &operation {
            stage = "alert_mutation";
            return crate::alert::change(request, &http, token, admission).await;
        }
        let (tool, arguments) = match &operation {
            Operation::Bars(request) => (crate::tools::Tool::Bars, request.arguments()),
            Operation::Data(request) => (request.kind().into(), request.arguments()),
            Operation::Financial(request) => (request.kind().into(), request.arguments()),
            Operation::Research(request) => (request.kind().into(), request.arguments()),
            Operation::Economic(request) => (request.kind().into(), request.arguments()),
            Operation::Account(request) => (request.kind().into(), request.arguments()),
            _ => return Err(Failure::UnsupportedCapability.into()),
        };
        let outcome = transport::call(&http, token, tool, &[arguments], Some(admission))
            .await
            .and_then(|mut values| values.pop().ok_or(Failure::InvalidResponse)?)
            .and_then(transport::result_value);
        let value = match outcome {
            Err(Failure::AuthRequired) => {
                return Err(match auth.refresh().await {
                    Ok(()) => Failure::AuthRefreshedRetryRequired,
                    Err(error) => error,
                }
                .into());
            }
            result => result.map_err(AppError::from)?,
        };
        let received_ms = now_ms().map_err(AppError::from)?;
        match &operation {
            Operation::Bars(request) => mcp_bars::normalize(request, value, received_ms),
            Operation::Economic(request) => {
                tradingview_model::mcp_economics::normalize(request, value, received_ms)
            }
            Operation::Research(request) => {
                tradingview_model::mcp_research::normalize(request, value, received_ms)
            }
            Operation::Financial(request) => {
                tradingview_model::mcp_financials::normalize(request, value, received_ms)
            }
            Operation::Data(request) => {
                tradingview_model::mcp_data::normalize(request, value, received_ms)
            }
            Operation::Account(request) => {
                tradingview_model::mcp_account::normalize(request, value, received_ms)
            }
            _ => Err(Failure::UnsupportedCapability.into()),
        }
    }
    .await;
    if let Some(wait) = http.cooldown()
        && let Err(error) = admission.cooldown(wait)
    {
        result = Err(failure(error, "local_state", 0, None));
    }
    let attempts = budget
        .lock()
        .map_err(|_| failure(Failure::LocalState, "local_state", 0, None))?
        .counts()
        .tools
        .saturating_sub(before);
    result.map_err(|mut error| {
        let mut error = if let Some(code) = error
            .details
            .as_ref()
            .and_then(|v| v.get("failure"))
            .and_then(|v| serde_json::from_value::<Failure>(v.clone()).ok())
        {
            failure(code, stage, attempts, Some(&http))
        } else {
            let details = error.details.get_or_insert_with(|| json!({}));
            details["stage"] = json!(stage);
            details["tool_attempts"] = json!(attempts);
            error
        };
        let mutation = match &operation {
            Operation::WatchlistMutation(request) => Some((
                request.action_name(),
                "inspect the watchlist before considering another mutation",
            )),
            Operation::AlertMutation(request) => Some((
                request.action_name(),
                "inspect the alerts before considering another mutation",
            )),
            _ => None,
        };
        if let Some((action, next_action)) = mutation {
            let details = error.details.get_or_insert_with(|| json!({}));
            details["mutation"] = json!({
                "operation": action,
                "status": if attempts == 0 { "not_attempted" } else { "outcome_unknown" },
                "automatic_retry": false
            });
            details["next_action"] = json!(next_action);
        }
        error
    })
}

pub(crate) fn failure(error: Failure, stage: &str, attempts: u32, http: Option<&Http>) -> AppError {
    let mut result = AppError::from(error);
    let code = match error {
        Failure::Timeout if stage == "local_admission" => "local_busy",
        Failure::Timeout => "deadline_exceeded",
        Failure::Connection => "transport_error",
        Failure::SchemaChanged => "schema_changed",
        Failure::StorageUnavailable
        | Failure::CredentialWorkerSpawn
        | Failure::CredentialWorkerWrite
        | Failure::CredentialWorkerRead
        | Failure::CredentialWorkerExit
        | Failure::CredentialWorkerReply
        | Failure::CredentialRecordDecode
        | Failure::CredentialRead
        | Failure::CredentialWrite => "credential_store_unavailable",
        Failure::StorageInteractionRequired => "credential_store_interaction_required",
        Failure::StorageTooLarge => "credential_store_too_large",
        Failure::InvalidResponse | Failure::BindingMismatch | Failure::ResponseTooLarge => {
            "invalid_response"
        }
        Failure::LocalState | Failure::Clock | Failure::StateSecurity { .. } => {
            "local_state_unavailable"
        }
        Failure::AuthRequired => "auth_required",
        Failure::AccessDenied => "access_denied",
        Failure::RateLimited => "rate_limited",
        Failure::BudgetExhausted => "duplicate_dispatch",
        Failure::ProviderError => "provider_error",
        Failure::UnsupportedCapability => "unsupported_capability",
        Failure::AuthRefreshedRetryRequired => "auth_refreshed_retry_required",
    };
    if error == Failure::UnsupportedCapability {
        result.kind = ErrorKind::InternalApiUnavailable;
    }
    let stage = if matches!(
        error,
        Failure::SchemaChanged | Failure::UnsupportedCapability
    ) && stage == "tool_response"
    {
        "tool_schema"
    } else {
        stage
    };
    let mut details = json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": code,
        "stage": stage,
        "tool_attempts": attempts,
        "automatic_retry": false
    });
    if let Failure::StateSecurity {
        reason,
        win32_error,
    } = error
    {
        details["reason"] = json!(reason);
        if let Some(code) = win32_error {
            details["win32_error"] = json!(code);
        }
    }
    if let Some(reason) = error.credential_reason() {
        details["reason"] = json!(reason);
    }
    if matches!(
        error,
        Failure::AuthRequired | Failure::StorageInteractionRequired
    ) {
        details["next_action"] = json!("tv mcp login");
    }
    if error == Failure::AuthRefreshedRetryRequired {
        details["next_action"] = json!("repeat the same explicit tv mcp command");
    }
    if error == Failure::RateLimited {
        details["retry_after_seconds"] = Value::Null;
        details["retry_after_evidence"] = json!("unconfirmed");
        if let Some(http) = http {
            let d = http.diagnostics();
            details["retry_after_seconds"] =
                d.get("retry_after_seconds").cloned().unwrap_or(Value::Null);
            details["retry_after_evidence"] = d
                .get("retry_after_evidence")
                .cloned()
                .unwrap_or(json!("unconfirmed"));
            details["local_cooldown_seconds"] = json!(http.cooldown());
        }
    }
    result.details = Some(details);
    result
}

fn state_directory() -> Result<PathBuf, Failure> {
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or(Failure::LocalState)?;
    #[cfg(target_os = "macos")]
    let root = std::env::var_os("HOME")
        .map(|p| PathBuf::from(p).join("Library/Application Support"))
        .ok_or(Failure::LocalState)?;
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let root = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/state")))
        .ok_or(Failure::LocalState)?;
    if !root.is_absolute() {
        return Err(Failure::LocalState);
    }
    Ok(root.join("tradingview-cli/mcp"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_security_diagnostics_preserve_codes_without_private_context() {
        use crate::error::StateSecurityReason;

        for win32_error in [None, Some(5)] {
            let cause = Failure::StateSecurity {
                reason: StateSecurityReason::SecurityQueryFailed,
                win32_error,
            };
            let encoded = serde_json::to_value(cause).unwrap();
            assert_eq!(serde_json::from_value::<Failure>(encoded).unwrap(), cause);
            let error = failure(cause, "local_admission", 0, None);
            let details = error.details.unwrap();
            assert_eq!(details["code"], "local_state_unavailable");
            assert_eq!(details["reason"], "state_security_query_failed");
            assert_eq!(details["stage"], "local_admission");
            assert_eq!(details["tool_attempts"], 0);
            assert_eq!(
                details.get("win32_error"),
                win32_error.map(|v| json!(v)).as_ref()
            );
            assert!(details.get("path").is_none());
            assert!(details.get("sid").is_none());
            assert!(details.get("stderr").is_none());
        }
    }

    #[test]
    fn credential_reasons_preserve_the_existing_public_error_contract() {
        for (cause, reason) in [
            (
                Failure::CredentialWorkerSpawn,
                "credential_worker_spawn_failed",
            ),
            (
                Failure::CredentialWorkerWrite,
                "credential_worker_input_failed",
            ),
            (
                Failure::CredentialWorkerRead,
                "credential_worker_output_failed",
            ),
            (
                Failure::CredentialWorkerExit,
                "credential_worker_exit_failed",
            ),
            (
                Failure::CredentialWorkerReply,
                "credential_worker_reply_invalid",
            ),
            (Failure::CredentialRecordDecode, "credential_record_invalid"),
            (Failure::CredentialRead, "credential_store_read_failed"),
            (Failure::CredentialWrite, "credential_store_write_failed"),
        ] {
            let error = failure(cause, "authentication", 0, None);
            assert_eq!(error.kind, ErrorKind::Internal);
            assert_eq!(error.exit_code(), 1);
            assert_eq!(error.message, "credential store unavailable");
            let details = error.details.unwrap();
            assert_eq!(details["contract_version"], "mcp_error.v1");
            assert_eq!(details["code"], "credential_store_unavailable");
            assert_eq!(details["stage"], "authentication");
            assert_eq!(details["tool_attempts"], 0);
            assert_eq!(details["automatic_retry"], false);
            assert_eq!(details["reason"], reason);
        }
    }

    #[test]
    fn downstream_credential_error_fixtures_match_the_public_envelope() {
        use tradingview_core::{ErrorBody, ErrorEnvelope};

        for (cause, fixture) in [
            (
                Failure::CredentialWorkerReply,
                include_str!("../tests/fixtures/credential-worker-reply-error.json"),
            ),
            (
                Failure::CredentialRead,
                include_str!("../tests/fixtures/credential-store-read-error.json"),
            ),
        ] {
            let error = failure(cause, "authentication", 0, None);
            let envelope = ErrorEnvelope::new("mcp", ErrorBody::from(error));
            assert_eq!(
                serde_json::to_value(envelope).unwrap(),
                serde_json::from_str::<Value>(fixture).unwrap()
            );
        }
    }
}
