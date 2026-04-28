use std::time::Duration;

use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, SuccessEnvelope};

use crate::{
    app::{
        output::{print_jsonl_stderr, print_jsonl_stdout},
        runtime::connect_runtime,
    },
    cli::StreamCommand,
    ops,
};
use tradingview_cdp::TransportConfig;

pub async fn run_stream_command(
    command: StreamCommand,
    config: &TransportConfig,
) -> Result<(), AppError> {
    let request = stream_request_from_command(command)?;
    let mut runtime = connect_runtime(config).await?;
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
