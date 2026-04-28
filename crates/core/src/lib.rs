use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Validation,
    Connection,
    InternalApiUnavailable,
    Timeout,
    TargetAmbiguous,
    Internal,
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub details: Option<Value>,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn exit_code(&self) -> u8 {
        match self.kind {
            ErrorKind::Connection => 2,
            ErrorKind::InternalApiUnavailable => 3,
            ErrorKind::Timeout => 4,
            ErrorKind::Validation | ErrorKind::TargetAmbiguous | ErrorKind::Internal => 1,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SuccessEnvelope {
    pub success: bool,
    pub command: &'static str,
    pub data: Value,
}

impl SuccessEnvelope {
    pub fn new(command: &'static str, data: Value) -> Self {
        Self {
            success: true,
            command,
            data,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub success: bool,
    pub command: &'static str,
    pub error: ErrorBody,
}

impl ErrorEnvelope {
    pub fn new(command: &'static str, error: ErrorBody) -> Self {
        Self {
            success: false,
            command,
            error,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub kind: ErrorKind,
    pub message: String,
    pub details: Option<Value>,
}

impl From<AppError> for ErrorBody {
    fn from(error: AppError) -> Self {
        Self {
            kind: error.kind,
            message: error.message,
            details: error.details,
        }
    }
}
