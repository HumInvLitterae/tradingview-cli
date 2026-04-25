mod cdp;
mod cli;
mod error;
mod ops;
mod output;
mod transport;

use std::{
    io::{self, IsTerminal, Read},
    process::ExitCode,
    time::Duration,
};

use cdp::CdpClient;
use clap::{Parser, error::ErrorKind as ClapErrorKind};
use cli::{
    AlertCommand, Cli, Command, DataCommand, DrawingCommand, IndicatorCommand, LayoutCommand,
    PaneCommand, PineCommand, ReplayCommand, StreamCommand, TabCommand, UiCommand,
    WatchlistCommand,
};
use error::{AppError, ErrorKind};
use output::{ErrorBody, ErrorEnvelope, SuccessEnvelope};
use serde_json::json;
use transport::TransportConfig;

const UNSAFE_UI_EVAL_ENV: &str = "TV_ALLOW_UNSAFE_UI_EVAL";

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

    if let Command::Stream { command } = cli.command {
        return match run_stream_command(command).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                let code = err.exit_code();
                let envelope = ErrorEnvelope::new("stream", ErrorBody::from(err));
                print_json_stderr(&envelope);
                ExitCode::from(code)
            }
        };
    }

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
        Command::Launch {
            port,
            path,
            kill_existing,
        } => {
            let request = ops::LaunchRequest::new(&config, port, path, kill_existing)?;
            ops::launch(request).await
        }
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
        Command::Range { from, to } => match (from, to) {
            (Some(from), Some(to)) => {
                let mut runtime = connect_runtime().await?;
                ops::set_visible_range(&mut runtime, from, to).await
            }
            (None, None) => {
                let mut runtime = connect_runtime().await?;
                ops::visible_range(&mut runtime).await
            }
            _ => Err(AppError::new(
                ErrorKind::Validation,
                "Both --from and --to are required when setting range",
            )),
        },
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
            AlertCommand::Delete { id, all, dry_run } => {
                if id.is_some() == all {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Use exactly one of --id <ID> or --all",
                    ));
                }
                if all {
                    let mut runtime = connect_runtime().await?;
                    ops::alert_delete_all(&mut runtime, dry_run).await
                } else {
                    let id = id.unwrap_or_default();
                    if id.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Alert ID must not be empty",
                        ));
                    }
                    if dry_run {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "--dry-run is only supported with --all",
                        ));
                    }
                    let mut runtime = connect_runtime().await?;
                    ops::alert_delete(&mut runtime, &id).await
                }
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
            DrawingCommand::Position {
                direction,
                entry_price,
                stop_loss,
                take_profit,
                entry_time,
                account_size,
                risk,
                lot_size,
            } => {
                let direction = ops::PositionDirection::parse(&direction)?;
                let request = ops::DrawingPositionRequest {
                    direction,
                    entry_price,
                    stop_loss,
                    take_profit,
                    entry_time,
                    account_size,
                    risk,
                    lot_size,
                };
                ops::validate_position_request(&request)?;
                let mut runtime = connect_runtime().await?;
                ops::drawing_position(&mut runtime, request).await
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
            DrawingCommand::Clear { dry_run } => {
                let mut runtime = connect_runtime().await?;
                ops::drawing_clear(&mut runtime, dry_run).await
            }
        },
        Command::Pine { command } => match command {
            PineCommand::Get => {
                let mut runtime = connect_runtime().await?;
                ops::pine_get(&mut runtime).await
            }
            PineCommand::Set { file } => {
                let (source, input_source) = read_pine_source(file.as_deref())?;
                let mut runtime = connect_runtime().await?;
                ops::pine_set(&mut runtime, &source, input_source).await
            }
            PineCommand::Compile => {
                let mut runtime = connect_runtime().await?;
                ops::pine_compile(&mut runtime).await
            }
            PineCommand::RawCompile => {
                let mut runtime = connect_runtime().await?;
                ops::pine_raw_compile(&mut runtime).await
            }
            PineCommand::Save => {
                let mut runtime = connect_runtime().await?;
                ops::pine_save(&mut runtime).await
            }
            PineCommand::New { script_type } => {
                let script_type =
                    ops::validate_pine_script_type(script_type.as_deref().unwrap_or("indicator"))?;
                let mut runtime = connect_runtime().await?;
                ops::pine_new(&mut runtime, script_type).await
            }
            PineCommand::Open { name } => {
                let name = name.join(" ");
                if name.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Script name required. Usage: tv pine open \"My Script\"",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::pine_open(&mut runtime, &name).await
            }
            PineCommand::Analyze { file } => {
                let (source, input_source) = read_pine_source(file.as_deref())?;
                Ok(ops::pine_analyze(&source, input_source))
            }
            PineCommand::Check { file } => {
                let (source, input_source) = read_pine_source(file.as_deref())?;
                ops::pine_check(&source, input_source).await
            }
            PineCommand::Errors => {
                let mut runtime = connect_runtime().await?;
                ops::pine_errors(&mut runtime).await
            }
            PineCommand::Console => {
                let mut runtime = connect_runtime().await?;
                ops::pine_console(&mut runtime).await
            }
            PineCommand::List => {
                let mut runtime = connect_runtime().await?;
                ops::pine_list(&mut runtime).await
            }
        },
        Command::Data { command } => match command {
            DataCommand::Indicator { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::data_indicator(&mut runtime, &entity_id).await
            }
            DataCommand::Depth => {
                let mut runtime = connect_runtime().await?;
                ops::data_depth(&mut runtime).await
            }
            DataCommand::Strategy => {
                let mut runtime = connect_runtime().await?;
                ops::data_strategy(&mut runtime).await
            }
            DataCommand::Trades { max } => {
                let mut runtime = connect_runtime().await?;
                ops::data_trades(&mut runtime, max).await
            }
            DataCommand::Equity => {
                let mut runtime = connect_runtime().await?;
                ops::data_equity(&mut runtime).await
            }
            DataCommand::Lines { filter, verbose } => {
                let mut runtime = connect_runtime().await?;
                ops::data_lines(&mut runtime, filter.as_deref(), verbose).await
            }
            DataCommand::Labels {
                filter,
                max,
                verbose,
            } => {
                let mut runtime = connect_runtime().await?;
                ops::data_labels(&mut runtime, filter.as_deref(), max, verbose).await
            }
            DataCommand::Tables { filter } => {
                let mut runtime = connect_runtime().await?;
                ops::data_tables(&mut runtime, filter.as_deref()).await
            }
            DataCommand::Boxes { filter, verbose } => {
                let mut runtime = connect_runtime().await?;
                ops::data_boxes(&mut runtime, filter.as_deref(), verbose).await
            }
            DataCommand::Shapes {
                filter,
                count,
                verbose,
            } => {
                if count == Some(0) {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "--count must be greater than 0",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::data_shapes(&mut runtime, filter.as_deref(), count, verbose).await
            }
        },
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
        Command::Layout { command } => match command {
            LayoutCommand::List => {
                let mut runtime = connect_runtime().await?;
                ops::saved_layout_list(&mut runtime).await
            }
            LayoutCommand::Switch { target, dry_run } => {
                let target = target.join(" ");
                if target.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Layout target required. Usage: tv layout switch \"My Layout\"",
                    ));
                }
                let mut runtime = connect_runtime().await?;
                ops::saved_layout_switch(&mut runtime, &target, dry_run).await
            }
        },
        Command::Tab { command } => match command {
            TabCommand::List => ops::tab_list(&config).await,
            TabCommand::Switch { index } => ops::tab_switch(&config, index).await,
            TabCommand::New { from } => ops::tab_new(&config, from).await,
            TabCommand::Close { index } => ops::tab_close(&config, index).await,
        },
        Command::Replay { command } => match command {
            ReplayCommand::Start { date } => {
                if let Some(date) = date.as_deref() {
                    ops::validate_replay_date(date)?;
                }
                let mut runtime = connect_runtime().await?;
                ops::replay_start(&mut runtime, date.as_deref()).await
            }
            ReplayCommand::Step => {
                let mut runtime = connect_runtime().await?;
                ops::replay_step(&mut runtime).await
            }
            ReplayCommand::Stop => {
                let mut runtime = connect_runtime().await?;
                ops::replay_stop(&mut runtime).await
            }
            ReplayCommand::Status => {
                let mut runtime = connect_runtime().await?;
                ops::replay_status(&mut runtime).await
            }
            ReplayCommand::Autoplay { speed } => {
                if let Some(speed) = speed {
                    ops::validate_replay_autoplay_speed(speed)?;
                }
                let mut runtime = connect_runtime().await?;
                ops::replay_autoplay(&mut runtime, speed).await
            }
            ReplayCommand::Trade { action } => {
                ops::validate_replay_trade_action(&action)?;
                let mut runtime = connect_runtime().await?;
                ops::replay_trade(&mut runtime, &action).await
            }
        },
        Command::Stream { .. } => unreachable!("stream commands use a dedicated JSONL runner"),
        Command::Ui {
            command: UiCommand::Eval { expression },
        } => {
            let expression = expression.join(" ");
            if expression.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Expression required. Usage: tv ui eval \"1+1\"",
                ));
            }
            require_unsafe_ui_eval_enabled()?;
            let mut runtime = connect_runtime().await?;
            ops::ui_eval(&mut runtime, &expression).await
        }
        Command::Ui { command } => {
            let mut runtime = connect_runtime().await?;
            match command {
                UiCommand::Eval { .. } => unreachable!("ui eval is handled before CDP connection"),
                UiCommand::Click { by, value } => ops::ui_click(&mut runtime, &by, &value).await,
                UiCommand::Keyboard {
                    key,
                    ctrl,
                    shift,
                    alt,
                    meta,
                } => ops::ui_keyboard(&mut runtime, &key, ctrl, shift, alt, meta).await,
                UiCommand::Hover { by, value } => ops::ui_hover(&mut runtime, &by, &value).await,
                UiCommand::Scroll { direction, amount } => {
                    if let Some(amount) = amount {
                        validate_finite(amount, "amount")?;
                    }
                    ops::ui_scroll(&mut runtime, direction.as_deref().unwrap_or("down"), amount)
                        .await
                }
                UiCommand::Find { query, strategy } => {
                    let query = query.join(" ");
                    if query.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Query required. Usage: tv ui find \"Indicators\"",
                        ));
                    }
                    ops::ui_find(&mut runtime, &query, strategy.as_deref()).await
                }
                UiCommand::Type { text } => {
                    let text = text.join(" ");
                    if text.is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Text required. Usage: tv ui type \"hello\"",
                        ));
                    }
                    ops::ui_type(&mut runtime, &text).await
                }
                UiCommand::Panel { panel, action } => {
                    ops::ui_panel(&mut runtime, &panel, action.as_deref().unwrap_or("toggle")).await
                }
                UiCommand::Fullscreen => ops::ui_fullscreen(&mut runtime).await,
                UiCommand::Mouse {
                    x,
                    y,
                    right,
                    double,
                } => {
                    validate_finite(x, "x")?;
                    validate_finite(y, "y")?;
                    ops::ui_mouse(&mut runtime, x, y, right, double).await
                }
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

async fn run_stream_command(command: StreamCommand) -> Result<(), AppError> {
    let request = stream_request_from_command(command)?;
    let mut runtime = connect_runtime().await?;
    let mut dedupe = ops::StreamDedupe::default();
    let interval = Duration::from_millis(request.interval_ms);

    loop {
        match ops::stream_sample(&mut runtime, &request).await {
            Ok(sample) => {
                if dedupe.should_emit(&sample) {
                    let envelope = SuccessEnvelope::new("stream", sample);
                    print_jsonl_stdout(&envelope);
                }
            }
            Err(err) => {
                let envelope = ErrorEnvelope::new("stream", ErrorBody::from(err));
                print_jsonl_stderr(&envelope);
            }
        }

        tokio::time::sleep(interval).await;
    }
}

fn stream_request_from_command(command: StreamCommand) -> Result<ops::StreamRequest, AppError> {
    match command {
        StreamCommand::Quote { interval } => {
            ops::StreamRequest::new(ops::StreamKind::Quote, interval, None)
        }
        StreamCommand::Bars { interval } => {
            ops::StreamRequest::new(ops::StreamKind::Bars, interval, None)
        }
        StreamCommand::Values { interval } => {
            ops::StreamRequest::new(ops::StreamKind::Values, interval, None)
        }
        StreamCommand::Lines { filter, interval } => {
            ops::StreamRequest::new(ops::StreamKind::Lines, interval, filter)
        }
        StreamCommand::Labels { filter, interval } => {
            ops::StreamRequest::new(ops::StreamKind::Labels, interval, filter)
        }
        StreamCommand::Tables { filter, interval } => {
            ops::StreamRequest::new(ops::StreamKind::Tables, interval, filter)
        }
        StreamCommand::All { interval } => {
            ops::StreamRequest::new(ops::StreamKind::All, interval, None)
        }
    }
}

fn require_unsafe_ui_eval_enabled() -> Result<(), AppError> {
    if unsafe_ui_eval_enabled_from(std::env::var_os(UNSAFE_UI_EVAL_ENV).as_deref()) {
        Ok(())
    } else {
        Err(unsafe_ui_eval_disabled_error())
    }
}

fn unsafe_ui_eval_enabled_from(value: Option<&std::ffi::OsStr>) -> bool {
    value == Some(std::ffi::OsStr::new("1"))
}

fn unsafe_ui_eval_disabled_error() -> AppError {
    AppError::new(
        ErrorKind::Validation,
        format!(
            "tv ui eval is disabled by default because it runs arbitrary JavaScript in the authenticated TradingView page context. Set {UNSAFE_UI_EVAL_ENV}=1 to enable this unsafe compatibility command explicitly."
        ),
    )
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

fn read_pine_source(file: Option<&std::path::Path>) -> Result<(String, &'static str), AppError> {
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

fn print_jsonl_stdout<T: serde::Serialize>(value: &T) {
    println!(
        "{}",
        serde_json::to_string(value).expect("JSON envelope serialization should not fail")
    );
}

fn print_jsonl_stderr<T: serde::Serialize>(value: &T) {
    eprintln!(
        "{}",
        serde_json::to_string(value).expect("JSON envelope serialization should not fail")
    );
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn unsafe_ui_eval_gate_only_accepts_one() {
        assert!(unsafe_ui_eval_enabled_from(Some(OsStr::new("1"))));

        for value in [
            None,
            Some(OsStr::new("")),
            Some(OsStr::new("0")),
            Some(OsStr::new("true")),
        ] {
            assert!(!unsafe_ui_eval_enabled_from(value));
        }
    }

    #[test]
    fn unsafe_ui_eval_disabled_error_names_env_gate() {
        let error = unsafe_ui_eval_disabled_error();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("tv ui eval is disabled by default"));
        assert!(error.message.contains(UNSAFE_UI_EVAL_ENV));
    }
}
