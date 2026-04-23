mod cdp;
mod cli;
mod error;
mod ops;
mod output;
mod transport;

use std::process::ExitCode;

use cdp::CdpClient;
use clap::{Parser, error::ErrorKind as ClapErrorKind};
use cli::{Cli, Command};
use error::{AppError, ErrorKind};
use output::{ErrorBody, ErrorEnvelope, SuccessEnvelope};
use serde_json::json;
use transport::TransportConfig;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err)
            if matches!(
                err.kind(),
                ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion
            ) =>
        {
            print!("{err}");
            return ExitCode::SUCCESS;
        }
        Err(err) => {
            let app_error = AppError::new(ErrorKind::Validation, err.to_string());
            let envelope = ErrorEnvelope::new("tv", ErrorBody::from(app_error));
            print_json_stderr(&envelope);
            return ExitCode::from(1);
        }
    };
    let command_name = cli.command.name();

    match dispatch(cli.command).await {
        Ok(data) => {
            let envelope = SuccessEnvelope::new(command_name, data);
            print_json_stdout(&envelope);
            ExitCode::SUCCESS
        }
        Err(err) => {
            let code = err.exit_code();
            let envelope = ErrorEnvelope::new(command_name, ErrorBody::from(err));
            print_json_stderr(&envelope);
            ExitCode::from(code)
        }
    }
}

async fn dispatch(command: Command) -> Result<serde_json::Value, AppError> {
    let config = TransportConfig::from_env()?;
    match command {
        Command::Status => ops::status(&config).await,
        Command::State => {
            let mut runtime = connect_runtime().await?;
            ops::state(&mut runtime).await
        }
        Command::Quote => {
            let mut runtime = connect_runtime().await?;
            ops::quote(&mut runtime).await
        }
        Command::Ohlcv { summary } => {
            if !summary {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Only --summary is supported in v1",
                ));
            }
            let mut runtime = connect_runtime().await?;
            ops::ohlcv_summary(&mut runtime).await
        }
        Command::Symbol { symbol } => {
            if symbol.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Symbol must not be empty",
                ));
            }
            let mut runtime = connect_runtime().await?;
            ops::set_symbol(&mut runtime, &symbol).await
        }
        Command::Timeframe { timeframe } => {
            if timeframe.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Timeframe must not be empty",
                ));
            }
            let mut runtime = connect_runtime().await?;
            ops::set_timeframe(&mut runtime, &timeframe).await
        }
        Command::Screenshot { region, output } => {
            if region != "full" {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Only --region full is supported in v1",
                )
                .with_details(json!({ "region": region })));
            }
            if output.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Output path must not be empty",
                ));
            }
            let mut runtime = connect_runtime().await?;
            ops::screenshot_full(&mut runtime, &output).await
        }
    }
}

async fn connect_runtime() -> Result<CdpClient, AppError> {
    let target = transport::discover_target(&TransportConfig::from_env()?).await?;
    CdpClient::connect(&target).await
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

fn print_json_stdout<T: serde::Serialize>(value: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("JSON envelope serialization should not fail")
    );
}

fn print_json_stderr<T: serde::Serialize>(value: &T) {
    eprintln!(
        "{}",
        serde_json::to_string_pretty(value).expect("JSON envelope serialization should not fail")
    );
}
