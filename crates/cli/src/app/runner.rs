use std::process::ExitCode;

use clap::{Parser, error::ErrorKind as ClapErrorKind};
use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, ErrorKind, SuccessEnvelope};

use crate::{
    app::{
        dispatch::dispatch,
        observe::run_observe_command,
        output::{
            JsonlRunError, OutputDisposition, OutputFailure, print_json_stderr, print_json_stdout,
            startup_error,
        },
        replay_log::run_replay_log_command,
        stream::run_stream_command,
        watch::run_watch_command,
    },
    cli::{Cli, Command, ReplayCommand},
};
use tradingview_cdp::TransportConfig;

pub fn run_cli() -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => return startup_error(format!("Failed to start async runtime: {err}")),
    };

    runtime.block_on(async_main())
}

async fn async_main() -> ExitCode {
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
            return terminal_error("tv", app_error);
        }
    };

    let config = match TransportConfig::from_env_with_target_id(cli.target_id.as_deref()) {
        Ok(config) => config,
        Err(err) => {
            return terminal_error("tv", err);
        }
    };

    let command = cli.command;
    match command {
        Command::Stream { command } => {
            jsonl_exit_code("stream", run_stream_command(command, &config).await)
        }
        Command::Observe { command } => {
            jsonl_exit_code("observe", run_observe_command(command, &config).await)
        }
        Command::Watch { command } => jsonl_exit_code("watch", run_watch_command(command).await),
        Command::Replay {
            command:
                ReplayCommand::Log {
                    steps,
                    attach_ohlcv_summary,
                    ohlcv_count,
                },
        } => jsonl_exit_code(
            "replay",
            run_replay_log_command(steps, attach_ohlcv_summary, ohlcv_count, &config).await,
        ),
        command => {
            let command_name = command.name();
            match dispatch(command, &config).await {
                Ok(data) => {
                    let envelope = SuccessEnvelope::new(command_name, data);
                    match print_json_stdout(&envelope) {
                        Ok(OutputDisposition::Written | OutputDisposition::BrokenPipe) => {
                            ExitCode::SUCCESS
                        }
                        Err(error) => stdout_failure(command_name, error),
                    }
                }
                Err(err) => terminal_error(command_name, err),
            }
        }
    }
}

fn jsonl_exit_code(command: &'static str, result: Result<(), JsonlRunError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(JsonlRunError::Application(error)) => terminal_error(command, error),
        Err(JsonlRunError::Stdout(error)) => stdout_failure(command, error),
        Err(JsonlRunError::Stderr) => ExitCode::from(1),
    }
}

fn terminal_error(command: &'static str, error: AppError) -> ExitCode {
    let code = error.exit_code();
    let envelope = ErrorEnvelope::new(command, ErrorBody::from(error));
    let _ = print_json_stderr(&envelope);
    ExitCode::from(code)
}

fn stdout_failure(command: &'static str, error: OutputFailure) -> ExitCode {
    let error = error.into_app_error("stdout");
    let envelope = ErrorEnvelope::new(command, ErrorBody::from(error));
    let _ = print_json_stderr(&envelope);
    ExitCode::from(1)
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
