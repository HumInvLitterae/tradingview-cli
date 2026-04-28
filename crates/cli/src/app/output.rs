use std::process::ExitCode;

use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, ErrorKind};

pub fn startup_error(message: impl Into<String>) -> ExitCode {
    let app_error = AppError::new(ErrorKind::Internal, message);
    let envelope = ErrorEnvelope::new("tv", ErrorBody::from(app_error));
    print_json_stderr(&envelope);
    ExitCode::from(1)
}
pub fn print_json_stdout<T: serde::Serialize>(value: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("JSON envelope serialization should not fail")
    );
}

pub fn print_json_stderr<T: serde::Serialize>(value: &T) {
    eprintln!(
        "{}",
        serde_json::to_string_pretty(value).expect("JSON envelope serialization should not fail")
    );
}

pub fn print_jsonl_stdout<T: serde::Serialize>(value: &T) {
    println!(
        "{}",
        serde_json::to_string(value).expect("JSON envelope serialization should not fail")
    );
}

pub fn print_jsonl_stderr<T: serde::Serialize>(value: &T) {
    eprintln!(
        "{}",
        serde_json::to_string(value).expect("JSON envelope serialization should not fail")
    );
}
