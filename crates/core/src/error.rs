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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kinds_keep_stable_exit_codes() {
        for (kind, expected) in [
            (ErrorKind::Validation, 1),
            (ErrorKind::Connection, 2),
            (ErrorKind::InternalApiUnavailable, 3),
            (ErrorKind::Timeout, 4),
            (ErrorKind::TargetAmbiguous, 1),
            (ErrorKind::Internal, 1),
        ] {
            assert_eq!(AppError::new(kind, "test").exit_code(), expected);
        }
    }
}
