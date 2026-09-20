use serde::{Deserialize, Serialize};
use tradingview_core::{AppError, ErrorKind};

/// Only closed, public-safe codes leave local coordination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Failure {
    LocalState,
    Clock,
    Timeout,
    RateLimited,
    InvalidResponse,
    SchemaChanged,
    ResponseTooLarge,
    AuthRequired,
    AccessDenied,
    StorageUnavailable,
    StorageInteractionRequired,
    StorageTooLarge,
    Connection,
    BudgetExhausted,
    BindingMismatch,
    ProviderError,
    UnsupportedCapability,
    AuthRefreshedRetryRequired,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AuthRequired => "authorization required",
            Self::AccessDenied => "access or scope requires user review",
            Self::StorageUnavailable => "credential store unavailable",
            Self::StorageInteractionRequired => {
                "credential store needs explicit user authorization"
            }
            Self::StorageTooLarge => "credential record exceeds native storage limit",
            Self::Connection => "connection failed",
            Self::BudgetExhausted => "proof budget exhausted",
            Self::BindingMismatch => "resource or issuer binding mismatch",
            Self::ProviderError => "provider returned an error",
            Self::UnsupportedCapability => "required capability is unavailable",
            Self::AuthRefreshedRetryRequired => "authorization refreshed; repeat the explicit read",
            Self::LocalState => "local coordination state unavailable",
            Self::Clock => "local clock is inconsistent with coordination state",
            Self::Timeout => "operation deadline exceeded",
            Self::SchemaChanged => "provider tool schema changed",
            Self::InvalidResponse => "response did not match the supported contract",
            Self::ResponseTooLarge => "response exceeded the byte limit",
            Self::RateLimited => "request limited; no automatic retry",
        })
    }
}

impl std::error::Error for Failure {}

impl From<Failure> for AppError {
    fn from(error: Failure) -> Self {
        let kind = match error {
            Failure::Timeout => ErrorKind::Timeout,
            Failure::Connection => ErrorKind::Connection,
            Failure::LocalState
            | Failure::Clock
            | Failure::StorageUnavailable
            | Failure::StorageInteractionRequired
            | Failure::StorageTooLarge => ErrorKind::Internal,
            Failure::BudgetExhausted | Failure::UnsupportedCapability => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        AppError::new(kind, error.to_string())
            .with_details(serde_json::json!({"failure": error, "automatic_retry": false}))
    }
}
