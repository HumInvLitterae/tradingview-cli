//! Validate candidate argv using clap and shared pure request preparation only.

use std::ffi::OsString;

use clap::{CommandFactory, FromArgMatches, error::ErrorKind as ClapErrorKind};
use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};
use tradingview_mcp::Operation;

use crate::{
    cli::{Cli, Command, DataCommand, McpCommand},
    ops,
};

const CONTRACT: &str = "cli_validate.v1";

pub(super) fn check(args: &[OsString]) -> Result<Value, AppError> {
    let matches = Cli::command()
        .try_get_matches_from(std::iter::once(OsString::from("tv")).chain(args.iter().cloned()))
        .map_err(|error| {
            if matches!(
                error.kind(),
                ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion
            ) {
                failure(
                    None,
                    "non_executable_request",
                    None,
                    "unsupported",
                    "not_checked",
                    "not_checked",
                )
            } else {
                syntax_error()
            }
        })?;
    let mut path = Vec::new();
    let mut current = &matches;
    while let Some((name, child)) = current.subcommand() {
        path.push(name.to_owned());
        current = child;
    }
    let cli = Cli::from_arg_matches(&matches).map_err(|_| syntax_error())?;
    if cli.version {
        return Err(failure(
            Some(&path),
            "non_executable_request",
            None,
            "unsupported",
            "passed",
            "not_checked",
        ));
    }
    let Some(command) = cli.command else {
        return Err(syntax_error());
    };
    let result = match command {
        Command::Values
        | Command::Ohlcv { .. }
        | Command::Data {
            command: DataCommand::Lines { .. } | DataCommand::Boxes { .. },
        } => Ok(()),
        Command::Mcp {
            command:
                McpCommand::Bars {
                    symbol,
                    timeframe,
                    count,
                    from,
                    to,
                },
            timeout,
        } => ops::validate_mcp_target(cli.target_id.as_deref())
            .and_then(|()| {
                ops::prepare_mcp_bars(&symbol, &timeframe, count, from.as_deref(), to.as_deref())
            })
            .and_then(|request| {
                Operation::Bars(request)
                    .timeout_duration(timeout)
                    .map(|_| ())
            }),
        Command::Spec { .. } | Command::Schema { .. } | Command::Validate { .. } => {
            return Err(failure(
                Some(&path),
                "non_executable_request",
                None,
                "unsupported",
                "passed",
                "not_checked",
            ));
        }
        _ => {
            return Err(failure(
                Some(&path),
                "unsupported_command",
                None,
                "unsupported",
                "passed",
                "not_checked",
            ));
        }
    };
    result.map_err(|error| local_error(&path, error))?;
    Ok(json!({
        "contract_version": CONTRACT, "command": path, "status": "valid",
        "checks": {"syntax": "passed", "local_constraints": "passed", "runtime": "not_checked"}
    }))
}

fn local_error(path: &[String], error: AppError) -> AppError {
    let details = error.details.as_ref();
    let reason = details
        .and_then(|d| d.get("reason"))
        .and_then(Value::as_str);
    let field = match reason {
        Some("symbol") => Some("symbol"),
        Some("count") => Some("count"),
        Some("timeframe") => Some("timeframe"),
        Some("date_range") => Some("date_range"),
        Some("desktop_target") => Some("target-id"),
        Some("timeout_seconds") => Some("timeout"),
        _ => None,
    };
    let code = if details.and_then(|d| d.get("code")).and_then(Value::as_str)
        == Some("unsupported_capability")
    {
        "unsupported_capability"
    } else {
        "invalid_request"
    };
    failure(Some(path), code, field, "invalid", "passed", "failed")
}

pub(super) fn syntax_error() -> AppError {
    failure(
        None,
        "invalid_syntax",
        None,
        "invalid",
        "failed",
        "not_checked",
    )
}

pub(super) fn outer_target_error() -> AppError {
    failure(
        None,
        "unsupported_target",
        Some("target-id"),
        "invalid",
        "not_checked",
        "not_checked",
    )
}

fn failure(
    command: Option<&[String]>,
    code: &str,
    field: Option<&str>,
    status: &str,
    syntax: &str,
    constraints: &str,
) -> AppError {
    let message = match code {
        "unsupported_command" => "Offline validation unavailable for this command",
        "non_executable_request" => "Not an executable request",
        _ => "Invalid command input",
    };
    AppError::new(ErrorKind::Validation, message).with_details(json!({
        "contract_version": CONTRACT, "command": command, "status": status,
        "code": code, "field": field,
        "checks": {"syntax": syntax, "local_constraints": constraints, "runtime": "not_checked"}
    }))
}

#[cfg(test)]
mod tests;
