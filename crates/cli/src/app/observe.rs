use std::time::{Duration, Instant};

use tradingview_cdp::TransportConfig;
use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, SuccessEnvelope};

use crate::{
    app::{
        output::{print_jsonl_stderr, print_jsonl_stdout},
        runtime::connect_runtime,
    },
    cli::ObserveCommand,
    ops,
};

pub async fn run_observe_command(
    command: ObserveCommand,
    config: &TransportConfig,
) -> Result<(), AppError> {
    let request = observe_request_from_command(command)?;
    let readiness = ops::readiness(config).await?;
    let readiness = ops::observe_readiness_event(readiness)?;
    let envelope = SuccessEnvelope::new("observe", readiness);
    print_jsonl_stdout(&envelope);

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
                        let sample = ops::observe_chart_event(sample)?;
                        let envelope = SuccessEnvelope::new("observe", sample);
                        print_jsonl_stdout(&envelope);
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
                    let envelope = ErrorEnvelope::new("observe", ErrorBody::from(err));
                    print_jsonl_stderr(&envelope);
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
            let payload = ops::observe_chart_event(payload)?;
            let envelope = SuccessEnvelope::new("observe", payload);
            print_jsonl_stdout(&envelope);
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
    let payload = ops::observe_chart_event(payload)?;
    let envelope = SuccessEnvelope::new("observe", payload);
    print_jsonl_stdout(&envelope);
    Ok(())
}

fn observe_request_from_command(command: ObserveCommand) -> Result<ops::StreamRequest, AppError> {
    match command {
        ObserveCommand::Chart { options } => ops::StreamRequest::with_controls(
            ops::StreamKind::Bars,
            options.interval,
            None,
            options.duration_ms,
            options.max_events,
            options.heartbeat_ms,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::StreamOptions;
    use tradingview_core::ErrorKind;

    #[test]
    fn observe_chart_uses_bars_stream_request() {
        let request = observe_request_from_command(ObserveCommand::Chart {
            options: StreamOptions {
                interval: Some(250),
                duration_ms: Some(1000),
                max_events: Some(2),
                heartbeat_ms: Some(500),
            },
        })
        .unwrap();

        assert_eq!(request.kind, ops::StreamKind::Bars);
        assert_eq!(request.interval_ms, 250);
        assert_eq!(request.duration_ms, Some(1000));
        assert_eq!(request.max_events, Some(2));
        assert_eq!(request.heartbeat_ms, Some(500));
    }

    #[test]
    fn observe_chart_reuses_stream_validation() {
        let error = observe_request_from_command(ObserveCommand::Chart {
            options: StreamOptions {
                interval: Some(99),
                duration_ms: None,
                max_events: None,
                heartbeat_ms: None,
            },
        })
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["minimum_interval_ms"], 100);
    }
}
