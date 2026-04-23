use serde::Serialize;
use serde_json::Value;

use crate::error::{AppError, ErrorKind};

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
