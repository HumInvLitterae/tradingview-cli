use std::process::ExitCode;

use clap::{Parser, error::ErrorKind as ClapErrorKind};
use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, ErrorKind, SuccessEnvelope};

use crate::{
    app::{
        dispatch::dispatch,
        output::{print_json_stderr, print_json_stdout, startup_error},
        stream::run_stream_command,
    },
    cli::{Cli, Command},
    transport::TransportConfig,
};

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
            let envelope = ErrorEnvelope::new("tv", ErrorBody::from(app_error));
            print_json_stderr(&envelope);
            return ExitCode::from(1);
        }
    };

    let config = match TransportConfig::from_env_with_target_id(cli.target_id.as_deref()) {
        Ok(config) => config,
        Err(err) => {
            let code = err.exit_code();
            let envelope = ErrorEnvelope::new("tv", ErrorBody::from(err));
            print_json_stderr(&envelope);
            return ExitCode::from(code);
        }
    };

    if let Command::Stream { command } = cli.command {
        return match run_stream_command(command, &config).await {
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

    match dispatch(cli.command, &config).await {
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

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
