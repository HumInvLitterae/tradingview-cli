use std::time::{Duration, Instant};

use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, SuccessEnvelope};

use crate::{
    app::{
        output::{
            JsonlOutput, JsonlRunError, OutputDisposition, emit_jsonl_stderr, emit_jsonl_stdout,
        },
        runtime::connect_runtime,
    },
    cli::StreamCommand,
    ops,
};
use tradingview_cdp::TransportConfig;

pub async fn run_stream_command(
    command: StreamCommand,
    config: &TransportConfig,
) -> Result<(), JsonlRunError> {
    let request = stream_request_from_command(command)?;
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut stdout = JsonlOutput::new(stdout.lock());
    let mut stderr = JsonlOutput::new(stderr.lock());
    let mut runtime = connect_runtime(config).await?;
    let mut dedupe = ops::StreamDedupe::default();
    let interval = Duration::from_millis(request.interval_ms);
    let duration = request.duration_ms.map(Duration::from_millis);
    let heartbeat = request.heartbeat_ms.map(Duration::from_millis);
    let started_at = Instant::now();
    let mut last_output_at = started_at;
    let mut next_sample_at = started_at;
    let mut next_heartbeat_at = heartbeat.map(|heartbeat| started_at + heartbeat);
    let mut sample_count = 0_u64;
    let mut heartbeat_count = 0_u64;
    let mut last_sample_ts = None;

    let end_reason = loop {
        let now = Instant::now();
        if duration.is_some_and(|limit| now.duration_since(started_at) >= limit) {
            break ops::StreamEndReason::DurationElapsed;
        }

        if now >= next_sample_at {
            match ops::stream_sample(&mut runtime, &request).await {
                Ok(sample) => {
                    if dedupe.should_emit(&sample) {
                        sample_count += 1;
                        last_sample_ts = sample["_ts"].as_u64();
                        let envelope = SuccessEnvelope::new("stream", sample);
                        if emit_jsonl_stdout(&mut stdout, &envelope)?
                            == OutputDisposition::BrokenPipe
                        {
                            return Ok(());
                        }
                        last_output_at = Instant::now();
                        next_heartbeat_at = heartbeat.map(|heartbeat| last_output_at + heartbeat);
                        if request
                            .max_events
                            .is_some_and(|max_events| sample_count >= max_events)
                        {
                            break ops::StreamEndReason::MaxEventsReached;
                        }
                    }
                }
                Err(err) => {
                    let envelope = ErrorEnvelope::new("stream", ErrorBody::from(err));
                    emit_jsonl_stderr(&mut stderr, &envelope)?;
                }
            }
            next_sample_at = Instant::now() + interval;
        }

        if let Some(heartbeat_interval) = heartbeat
            && let Some(next_heartbeat) = next_heartbeat_at
            && Instant::now() >= next_heartbeat
            && last_output_at.elapsed() >= heartbeat_interval
        {
            let payload = ops::stream_heartbeat(
                &request,
                started_at.elapsed().as_millis() as u64,
                sample_count,
                last_sample_ts,
            )?;
            let envelope = SuccessEnvelope::new("stream", payload);
            if emit_jsonl_stdout(&mut stdout, &envelope)? == OutputDisposition::BrokenPipe {
                return Ok(());
            }
            heartbeat_count += 1;
            last_output_at = Instant::now();
            next_heartbeat_at = heartbeat.map(|heartbeat| last_output_at + heartbeat);
        }

        let mut sleep_until = next_sample_at;
        if let Some(next_heartbeat) = next_heartbeat_at {
            sleep_until = sleep_until.min(next_heartbeat);
        }
        if let Some(limit) = duration {
            sleep_until = sleep_until.min(started_at + limit);
        }
        let sleep_duration = sleep_until.saturating_duration_since(Instant::now());
        if sleep_duration.is_zero() {
            tokio::task::yield_now().await;
            continue;
        }
        tokio::time::sleep(sleep_duration).await;
    };
    let payload = ops::stream_summary(
        &request,
        started_at.elapsed().as_millis() as u64,
        sample_count,
        heartbeat_count,
        last_sample_ts,
        end_reason,
    )?;
    let envelope = SuccessEnvelope::new("stream", payload);
    let _ = emit_jsonl_stdout(&mut stdout, &envelope)?;
    Ok(())
}

fn stream_request_from_command(command: StreamCommand) -> Result<ops::StreamRequest, AppError> {
    match command {
        StreamCommand::Quote { options } => {
            stream_request_with_options(ops::StreamKind::Quote, options, None)
        }
        StreamCommand::Bars { options } => {
            stream_request_with_options(ops::StreamKind::Bars, options, None)
        }
        StreamCommand::Values { options } => {
            stream_request_with_options(ops::StreamKind::Values, options, None)
        }
        StreamCommand::Lines { filter, options } => {
            stream_request_with_options(ops::StreamKind::Lines, options, filter)
        }
        StreamCommand::Labels { filter, options } => {
            stream_request_with_options(ops::StreamKind::Labels, options, filter)
        }
        StreamCommand::Tables { filter, options } => {
            stream_request_with_options(ops::StreamKind::Tables, options, filter)
        }
        StreamCommand::All { options } => {
            stream_request_with_options(ops::StreamKind::All, options, None)
        }
    }
}

fn stream_request_with_options(
    kind: ops::StreamKind,
    options: crate::cli::StreamOptions,
    filter: Option<String>,
) -> Result<ops::StreamRequest, AppError> {
    ops::StreamRequest::with_controls(
        kind,
        options.interval,
        filter,
        options.duration_ms,
        options.max_events,
        options.heartbeat_ms,
    )
}
