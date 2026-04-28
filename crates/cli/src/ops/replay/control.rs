use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::payload::normalize_replay_action;
use super::validation::parse_replay_date_ms;

pub async fn replay_start(
    runtime: &mut impl RuntimeEvaluator,
    date: Option<&str>,
) -> Result<Value, AppError> {
    let (date_payload, date_js) = match date {
        Some(date) => {
            let timestamp_ms = parse_replay_date_ms(date)?;
            (json!(date), timestamp_ms.to_string())
        }
        None => (json!("(first available)"), "null".to_string()),
    };
    let data = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                function unwrap(value) {{
                    return value && typeof value === 'object' && typeof value.value === 'function'
                        ? value.value()
                        : value;
                }}
                function sleep(ms) {{
                    return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                }}

                try {{
                    var replay = window.TradingViewApi && window.TradingViewApi._replayApi;
                    if (!replay) {{
                        return {{
                            ok: false,
                            error_kind: 'internal_api_unavailable',
                            message: 'TradingView replay API is not available'
                        }};
                    }}
                    var required = ['isReplayAvailable', 'isReplayStarted', 'currentDate', 'showReplayToolbar', 'selectDate', 'selectFirstAvailableDate', 'stopReplay'];
                    for (var i = 0; i < required.length; i++) {{
                        if (typeof replay[required[i]] !== 'function') {{
                            return {{
                                ok: false,
                                error_kind: 'internal_api_unavailable',
                                message: 'TradingView replay API is missing method: ' + required[i],
                                missing_method: required[i]
                            }};
                        }}
                    }}

                    var available = unwrap(replay.isReplayAvailable());
                    if (!available) {{
                        return {{
                            ok: false,
                            error_kind: 'internal_api_unavailable',
                            message: 'Replay is not available for the current symbol/timeframe',
                            is_replay_available: available
                        }};
                    }}

                    replay.showReplayToolbar();
                    var requestedDate = {date_js};
                    if (requestedDate !== null) {{
                        var selected = replay.selectDate(requestedDate);
                        if (selected && typeof selected.then === 'function') {{
                            await selected;
                        }}
                    }} else {{
                        replay.selectFirstAvailableDate();
                    }}

                    var started = false;
                    var currentDate = null;
                    for (var poll = 0; poll < 30; poll++) {{
                        started = !!unwrap(replay.isReplayStarted());
                        currentDate = unwrap(replay.currentDate());
                        if (started && currentDate !== null) break;
                        await sleep(250);
                    }}

                    if (!started) {{
                        try {{ replay.stopReplay(); }} catch (ignored) {{}}
                        return {{
                            ok: false,
                            error_kind: 'internal_api_unavailable',
                            message: 'Replay failed to start. The selected date may not have data for this timeframe.',
                            replay_started: started,
                            current_date: currentDate
                        }};
                    }}

                    return {{
                        ok: true,
                        action: 'started',
                        replay_started: true,
                        date: {date_payload},
                        current_date: currentDate,
                        source: 'internal_api'
                    }};
                }} catch (error) {{
                    return {{
                        ok: false,
                        error_kind: 'internal_api_unavailable',
                        message: error && error.message ? error.message : String(error)
                    }};
                }}
            }})()
            "#
            ),
            true,
        )
        .await?;

    normalize_replay_action(data)
}

pub async fn replay_step(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let data = runtime
        .evaluate(
            r#"
            (async function() {
                function unwrap(value) {
                    return value && typeof value === 'object' && typeof value.value === 'function'
                        ? value.value()
                        : value;
                }
                function sleep(ms) {
                    return new Promise(function(resolve) { setTimeout(resolve, ms); });
                }

                try {
                    var replay = window.TradingViewApi && window.TradingViewApi._replayApi;
                    if (!replay) {
                        return {
                            ok: false,
                            error_kind: 'internal_api_unavailable',
                            message: 'TradingView replay API is not available'
                        };
                    }
                    var required = ['isReplayStarted', 'currentDate', 'doStep'];
                    for (var i = 0; i < required.length; i++) {
                        if (typeof replay[required[i]] !== 'function') {
                            return {
                                ok: false,
                                error_kind: 'internal_api_unavailable',
                                message: 'TradingView replay API is missing method: ' + required[i],
                                missing_method: required[i]
                            };
                        }
                    }

                    var started = !!unwrap(replay.isReplayStarted());
                    if (!started) {
                        return {
                            ok: false,
                            error_kind: 'validation',
                            message: 'Replay is not started. Use replay start first.'
                        };
                    }

                    var previousDate = unwrap(replay.currentDate());
                    replay.doStep();
                    var currentDate = previousDate;
                    for (var poll = 0; poll < 12; poll++) {
                        await sleep(250);
                        currentDate = unwrap(replay.currentDate());
                        if (currentDate !== previousDate) break;
                    }

                    return {
                        ok: true,
                        action: 'step',
                        previous_date: previousDate,
                        current_date: currentDate,
                        source: 'internal_api'
                    };
                } catch (error) {
                    return {
                        ok: false,
                        error_kind: 'internal_api_unavailable',
                        message: error && error.message ? error.message : String(error)
                    };
                }
            })()
            "#,
            true,
        )
        .await?;

    normalize_replay_action(data)
}

pub async fn replay_stop(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let data = runtime
        .evaluate(
            r#"
            (function() {
                function unwrap(value) {
                    return value && typeof value === 'object' && typeof value.value === 'function'
                        ? value.value()
                        : value;
                }

                try {
                    var replay = window.TradingViewApi && window.TradingViewApi._replayApi;
                    if (!replay) {
                        return {
                            ok: false,
                            error_kind: 'internal_api_unavailable',
                            message: 'TradingView replay API is not available'
                        };
                    }
                    if (typeof replay.isReplayStarted !== 'function' || typeof replay.stopReplay !== 'function') {
                        return {
                            ok: false,
                            error_kind: 'internal_api_unavailable',
                            message: 'TradingView replay API is missing stop methods'
                        };
                    }

                    var started = !!unwrap(replay.isReplayStarted());
                    if (!started) {
                        return {
                            ok: true,
                            action: 'already_stopped',
                            replay_started: false,
                            source: 'internal_api'
                        };
                    }

                    replay.stopReplay();
                    return {
                        ok: true,
                        action: 'replay_stopped',
                        replay_started: false,
                        source: 'internal_api'
                    };
                } catch (error) {
                    return {
                        ok: false,
                        error_kind: 'internal_api_unavailable',
                        message: error && error.message ? error.message : String(error)
                    };
                }
            })()
            "#,
            false,
        )
        .await?;

    normalize_replay_action(data)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_core::ErrorKind;

    #[tokio::test]
    async fn replay_start_serializes_date_as_timestamp() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": true, "action": "started", "replay_started": true, "date": "2026-04-01", "current_date": 1775001600000i64, "source": "internal_api"}),
        ]);
        let result = replay_start(&mut runtime, Some("2026-04-01"))
            .await
            .unwrap();
        assert_eq!(result["action"], "started");
        assert_eq!(result["replay_started"], true);
        assert_eq!(result["date"], "2026-04-01");
        assert!(runtime.evaluated[0].0.contains("1775001600000"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn replay_start_rejects_invalid_date_before_evaluating() {
        let mut runtime = FakeRuntime::new([]);
        let error = replay_start(&mut runtime, Some("2026-02-31"))
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn replay_start_maps_unavailable_replay_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": false, "error_kind": "internal_api_unavailable", "message": "Replay is not available for the current symbol/timeframe"}),
        ]);
        let error = replay_start(&mut runtime, None).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn replay_step_requires_started_replay() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": false, "error_kind": "validation", "message": "Replay is not started. Use replay start first."}),
        ]);
        let error = replay_step(&mut runtime).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn replay_step_returns_previous_and_current_date() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": true, "action": "step", "previous_date": 1775001600000i64, "current_date": 1775088000000i64, "source": "internal_api"}),
        ]);
        let result = replay_step(&mut runtime).await.unwrap();
        assert_eq!(result["action"], "step");
        assert_eq!(result["previous_date"], 1775001600000i64);
        assert_eq!(result["current_date"], 1775088000000i64);
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn replay_stop_returns_already_stopped() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": true, "action": "already_stopped", "replay_started": false, "source": "internal_api"}),
        ]);
        let result = replay_stop(&mut runtime).await.unwrap();
        assert_eq!(result["action"], "already_stopped");
    }

    #[tokio::test]
    async fn replay_stop_returns_stopped_action() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": true, "action": "replay_stopped", "replay_started": false, "source": "internal_api"}),
        ]);
        let result = replay_stop(&mut runtime).await.unwrap();
        assert_eq!(result["action"], "replay_stopped");
    }
}
