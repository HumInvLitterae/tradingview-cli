mod cdp;
mod cli;
mod error;
mod ops;
mod output;
mod transport;

use std::process::ExitCode;

use cdp::CdpClient;
use clap::{Parser, error::ErrorKind as ClapErrorKind};
use cli::{
    AlertCommand, Cli, Command, DataCommand, DrawingCommand, IndicatorCommand, PaneCommand,
    WatchlistCommand,
};
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
        Command::Type { chart_type } => match chart_type {
            Some(chart_type) => {
                if chart_type.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Chart type must not be empty",
                    ));
                }
                ops::validate_chart_type(&chart_type)?;
                let mut runtime = connect_runtime().await?;
                ops::set_chart_type(&mut runtime, &chart_type).await
            }
            None => {
                let mut runtime = connect_runtime().await?;
                ops::current_chart_type(&mut runtime).await
            }
        },
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
        Command::Watchlist { command } => match command {
            WatchlistCommand::Get => {
                let mut runtime = connect_runtime().await?;
                ops::watchlist_get(&mut runtime).await
            }
            WatchlistCommand::Add { symbol } => {
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Symbol must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::watchlist_add(&mut runtime, &symbol).await
            }
            WatchlistCommand::Remove { symbol } => {
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Symbol must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::watchlist_remove(&mut runtime, &symbol).await
            }
        },
        Command::Alert { command } => match command {
            AlertCommand::List => {
                let mut runtime = connect_runtime().await?;
                ops::alert_list(&mut runtime).await
            }
            AlertCommand::Create {
                price,
                condition,
                message,
            } => {
                if !price.is_finite() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "price must be a finite number",
                    ));
                }
                ops::validate_alert_condition(&condition)?;
                let mut runtime = connect_runtime().await?;
                ops::alert_create(&mut runtime, price, &condition, message.as_deref()).await
            }
            AlertCommand::Delete { id } => {
                if id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Alert ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::alert_delete(&mut runtime, &id).await
            }
        },
        Command::Indicator { command } => match command {
            IndicatorCommand::Add { indicator, inputs } => {
                let indicator = indicator.join(" ");
                if indicator.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Indicator name required. Usage: tv indicator add \"Volume\"",
                    ));
                }
                let inputs = inputs
                    .as_deref()
                    .map(ops::parse_indicator_inputs)
                    .transpose()?;
                let mut runtime = connect_runtime().await?;
                ops::indicator_add(&mut runtime, &indicator, inputs.as_ref()).await
            }
            IndicatorCommand::Remove { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::indicator_remove(&mut runtime, &entity_id).await
            }
            IndicatorCommand::Toggle {
                entity_id,
                visible,
                hidden,
            } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                if visible && hidden {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Use either --visible or --hidden, not both",
                    ));
                }
                let target_visible = !hidden;
                let mut runtime = connect_runtime().await?;
                ops::indicator_toggle(&mut runtime, &entity_id, target_visible).await
            }
            IndicatorCommand::Set { entity_id, inputs } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let inputs = ops::parse_indicator_inputs(&inputs)?;
                let mut runtime = connect_runtime().await?;
                ops::indicator_set(&mut runtime, &entity_id, &inputs).await
            }
            IndicatorCommand::Get { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::data_indicator(&mut runtime, &entity_id).await
            }
        },
        Command::Draw { command } => match command {
            DrawingCommand::Shape {
                shape_type,
                price,
                time,
                price2,
                time2,
                text,
                overrides,
            } => {
                if shape_type.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Drawing shape type must not be empty",
                    ));
                }
                validate_finite(price, "price")?;
                validate_finite(time, "time")?;
                let point2 = match (price2, time2) {
                    (Some(price2), Some(time2)) => {
                        validate_finite(price2, "price2")?;
                        validate_finite(time2, "time2")?;
                        Some(ops::DrawingPoint {
                            time: time2,
                            price: price2,
                        })
                    }
                    (None, None) => None,
                    _ => {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "--price2 and --time2 must be provided together",
                        ));
                    }
                };
                let overrides = overrides
                    .as_deref()
                    .map(ops::parse_drawing_overrides)
                    .transpose()?;
                let request = ops::DrawingShapeRequest {
                    shape_type: shape_type.trim().to_string(),
                    point: ops::DrawingPoint { time, price },
                    point2,
                    text,
                    overrides,
                };
                let mut runtime = connect_runtime().await?;
                ops::drawing_shape(&mut runtime, request).await
            }
            DrawingCommand::List => {
                let mut runtime = connect_runtime().await?;
                ops::drawing_list(&mut runtime).await
            }
            DrawingCommand::Get { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::drawing_get(&mut runtime, &entity_id).await
            }
            DrawingCommand::Remove { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::drawing_remove(&mut runtime, &entity_id).await
            }
        },
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
                DataCommand::Depth => ops::data_depth(&mut runtime).await,
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
        Command::Pane { command } => match command {
            PaneCommand::List => {
                let mut runtime = connect_runtime().await?;
                ops::pane_list(&mut runtime).await
            }
            PaneCommand::Layout { layout } => {
                ops::validate_pane_layout(&layout)?;
                let mut runtime = connect_runtime().await?;
                ops::pane_layout(&mut runtime, &layout).await
            }
            PaneCommand::Focus { index } => {
                let mut runtime = connect_runtime().await?;
                ops::pane_focus(&mut runtime, index).await
            }
            PaneCommand::Symbol { index, symbol } => {
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Symbol must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::pane_symbol(&mut runtime, index, &symbol).await
            }
        },
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

fn validate_finite(value: f64, label: &str) -> Result<(), AppError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} must be a finite number"),
        ))
    }
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
