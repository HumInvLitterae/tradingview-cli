use std::process::ExitCode;

#[cfg(windows)]
const WINDOWS_CLI_STACK_SIZE: usize = 8 * 1024 * 1024;

#[cfg(windows)]
fn main() -> ExitCode {
    match std::thread::Builder::new()
        .name("tv-cli".to_string())
        .stack_size(WINDOWS_CLI_STACK_SIZE)
        .spawn(tradingview_cli::app::run_cli)
    {
        Ok(handle) => match handle.join() {
            Ok(code) => code,
            Err(_) => tradingview_cli::app::startup_error("tv CLI runtime thread panicked"),
        },
        Err(err) => tradingview_cli::app::startup_error(format!(
            "Failed to start tv CLI runtime thread: {err}"
        )),
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    tradingview_cli::app::run_cli()
}
