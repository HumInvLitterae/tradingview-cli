use std::{
    io::{self, IsTerminal, Read},
    path::Path,
};

use tradingview_core::{AppError, ErrorKind};

pub fn read_pine_source(file: Option<&Path>) -> Result<(String, &'static str), AppError> {
    let (source, input_source) = if let Some(path) = file {
        let source = std::fs::read_to_string(path).map_err(|err| {
            AppError::new(
                ErrorKind::Validation,
                format!("Failed to read Pine source file: {err}"),
            )
        })?;
        (source, "file")
    } else {
        let mut stdin = io::stdin();
        if stdin.is_terminal() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "Pine source required via stdin or --file",
            ));
        }
        let mut source = String::new();
        stdin.read_to_string(&mut source).map_err(|err| {
            AppError::new(
                ErrorKind::Validation,
                format!("Failed to read Pine source from stdin: {err}"),
            )
        })?;
        (source, "stdin")
    };

    if source.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Pine source must not be empty",
        ));
    }

    Ok((source, input_source))
}
