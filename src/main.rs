mod cdp;
mod cli;
mod error;
mod ops;
mod output;
mod transport;

use std::process::ExitCode;

use cdp::CdpClient;
use clap::{Parser, error::ErrorKind as ClapErrorKind};
use cli::{Cli, Command, DataCommand, PaneCommand, WatchlistCommand};
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
        Command::Info => {
            let mut runtime = connect_runtime().await?;
            ops::symbol_info(&mut runtime).await
        }
        Command::Search { query } => {
            let query = query.join(" ");
            if query.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Query required. Usage: tv search AAPL",
                ));
            }
            ops::symbol_search(&query).await
        }
        Command::Quote => {
            let mut runtime = connect_runtime().await?;
            ops::quote(&mut runtime).await
        }
        Command::Values => {
            let mut runtime = connect_runtime().await?;
            ops::study_values(&mut runtime).await
        }
        Command::Discover => {
            let mut runtime = connect_runtime().await?;
            ops::discover(&mut runtime).await
        }
        Command::UiState => {
            let mut runtime = connect_runtime().await?;
            ops::ui_state(&mut runtime).await
        }
        Command::Ohlcv { summary, count } => {
            let mut runtime = connect_runtime().await?;
            if summary {
                ops::ohlcv_summary(&mut runtime, count).await
            } else {
                ops::ohlcv_bars(&mut runtime, count).await
            }
        }
        Command::Symbol { symbol } => {
            let mut runtime = connect_runtime().await?;
            match symbol {
                Some(symbol) => {
                    if symbol.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Symbol must not be empty",
                        ));
                    }
                    ops::set_symbol(&mut runtime, &symbol).await
                }
                None => ops::current_symbol(&mut runtime).await,
            }
        }
        Command::Timeframe { timeframe } => {
            let mut runtime = connect_runtime().await?;
            match timeframe {
                Some(timeframe) => {
                    if timeframe.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Timeframe must not be empty",
                        ));
                    }
                    ops::set_timeframe(&mut runtime, &timeframe).await
                }
                None => ops::current_timeframe(&mut runtime).await,
            }
        }
        Command::Range { from, to } => {
            let mut runtime = connect_runtime().await?;
            match (from, to) {
                (Some(from), Some(to)) => ops::set_visible_range(&mut runtime, from, to).await,
                (None, None) => ops::visible_range(&mut runtime).await,
                _ => Err(AppError::new(
                    ErrorKind::Validation,
                    "Both --from and --to are required when setting range",
                )),
            }
        }
        Command::Scroll { date } => {
            if date.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Date must not be empty",
                ));
            }
            let mut runtime = connect_runtime().await?;
            ops::scroll_to_date(&mut runtime, &date).await
        }
        Command::Watchlist { command } => {
            let mut runtime = connect_runtime().await?;
            match command {
                WatchlistCommand::Get => ops::watchlist_get(&mut runtime).await,
            }
        }
        Command::Data { command } => {
            let mut runtime = connect_runtime().await?;
            match command {
                DataCommand::Indicator { entity_id } => {
                    if entity_id.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Entity ID must not be empty",
                        ));
                    }
                    ops::data_indicator(&mut runtime, &entity_id).await
                }
                DataCommand::Strategy => ops::data_strategy(&mut runtime).await,
                DataCommand::Trades { max } => ops::data_trades(&mut runtime, max).await,
                DataCommand::Equity => ops::data_equity(&mut runtime).await,
                DataCommand::Lines { filter, verbose } => {
                    ops::data_lines(&mut runtime, filter.as_deref(), verbose).await
                }
                DataCommand::Labels {
                    filter,
                    max,
                    verbose,
                } => ops::data_labels(&mut runtime, filter.as_deref(), max, verbose).await,
                DataCommand::Tables { filter } => {
                    ops::data_tables(&mut runtime, filter.as_deref()).await
                }
                DataCommand::Boxes { filter, verbose } => {
                    ops::data_boxes(&mut runtime, filter.as_deref(), verbose).await
                }
            }
        }
        Command::Pane { command } => {
            let mut runtime = connect_runtime().await?;
            match command {
                PaneCommand::List => ops::pane_list(&mut runtime).await,
            }
        }
        Command::Screenshot { region, output } => {
            if !matches!(region.as_str(), "full" | "chart") {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Only --region full and --region chart are supported",
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
            match region.as_str() {
                "full" => ops::screenshot_full(&mut runtime, &output).await,
                "chart" => ops::screenshot_chart(&mut runtime, &output).await,
                _ => unreachable!("screenshot region should be validated"),
            }
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
