use serde_json::{Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

const VALID_AUTOPLAY_DELAYS: [u64; 9] = [100, 143, 200, 300, 1000, 2000, 3000, 5000, 10000];

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

pub fn validate_replay_date(date: &str) -> Result<(), AppError> {
    parse_replay_date_ms(date).map(|_| ())
}

pub fn validate_replay_autoplay_speed(speed: u64) -> Result<(), AppError> {
    if speed == 0 || VALID_AUTOPLAY_DELAYS.contains(&speed) {
        return Ok(());
    }

    Err(AppError::new(
        ErrorKind::Validation,
        format!(
            "Invalid replay autoplay delay: {speed}ms. Use 0 or one of: {}.",
            VALID_AUTOPLAY_DELAYS
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
    .with_details(json!({
        "speed": speed,
        "supported": VALID_AUTOPLAY_DELAYS,
    })))
}

pub async fn replay_autoplay(
    runtime: &mut impl RuntimeEvaluator,
    speed: Option<u64>,
) -> Result<Value, AppError> {
    if let Some(speed) = speed {
        validate_replay_autoplay_speed(speed)?;
    }

    let requested_delay = speed.filter(|speed| *speed > 0);
    let requested_delay_js = requested_delay
        .map(|speed| speed.to_string())
        .unwrap_or_else(|| "null".to_string());
    let change_delay_check = if requested_delay.is_some() {
        r#"
                    if (typeof replay.changeAutoplayDelay !== 'function') {
                        return {
                            ok: false,
                            error_kind: 'internal_api_unavailable',
                            message: 'TradingView replay API is missing method: changeAutoplayDelay',
                            missing_method: 'changeAutoplayDelay'
                        };
                    }
"#
    } else {
        ""
    };
    let change_delay_call = if requested_delay.is_some() {
        r#"
                    replay.changeAutoplayDelay(requestedDelay);
"#
    } else {
        ""
    };

    let data = runtime
        .evaluate(
            &format!(
                r#"
            (function() {{
                function unwrap(value) {{
                    return value && typeof value === 'object' && typeof value.value === 'function'
                        ? value.value()
                        : value;
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

                    var required = ['isReplayStarted', 'toggleAutoplay', 'isAutoplayStarted', 'autoplayDelay'];
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
{change_delay_check}
                    var started = !!unwrap(replay.isReplayStarted());
                    if (!started) {{
                        return {{
                            ok: false,
                            error_kind: 'validation',
                            message: 'Replay is not started. Use replay start first.'
                        }};
                    }}

                    var requestedDelay = {requested_delay_js};
{change_delay_call}
                    replay.toggleAutoplay();
                    return {{
                        ok: true,
                        action: 'autoplay',
                        autoplay_active: !!unwrap(replay.isAutoplayStarted()),
                        delay_ms: unwrap(replay.autoplayDelay()),
                        requested_delay_ms: requestedDelay,
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
            false,
        )
        .await?;

    normalize_replay_action(data)
}

pub fn validate_replay_trade_action(action: &str) -> Result<(), AppError> {
    match action {
        "buy" | "sell" | "close" => Ok(()),
        _ => Err(AppError::new(
            ErrorKind::Validation,
            "Invalid replay trade action. Use buy, sell, or close.",
        )
        .with_details(json!({
            "action": action,
            "supported": ["buy", "sell", "close"],
        }))),
    }
}

pub async fn replay_trade(
    runtime: &mut impl RuntimeEvaluator,
    action: &str,
) -> Result<Value, AppError> {
    validate_replay_trade_action(action)?;

    let method = match action {
        "buy" => "buy",
        "sell" => "sell",
        "close" => "closePosition",
        _ => unreachable!("replay trade action is validated above"),
    };
    let action_literal = json!(action).to_string();

    let data = runtime
        .evaluate(
            &format!(
                r#"
            (function() {{
                function unwrap(value) {{
                    return value && typeof value === 'object' && typeof value.value === 'function'
                        ? value.value()
                        : value;
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

                    var required = ['isReplayStarted', 'position', 'realizedPL', '{method}'];
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

                    var started = !!unwrap(replay.isReplayStarted());
                    if (!started) {{
                        return {{
                            ok: false,
                            error_kind: 'validation',
                            message: 'Replay is not started. Use replay start first.'
                        }};
                    }}

                    replay.{method}();
                    return {{
                        ok: true,
                        action: {action_literal},
                        position: unwrap(replay.position()),
                        realized_pnl: unwrap(replay.realizedPL()),
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
            false,
        )
        .await?;

    normalize_replay_action(data)
}

pub async fn replay_status(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
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

                    var required = [
                        'isReplayAvailable',
                        'isReplayStarted',
                        'isAutoplayStarted',
                        'replayMode',
                        'currentDate',
                        'autoplayDelay',
                        'position',
                        'realizedPL'
                    ];
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

                    return {
                        ok: true,
                        is_replay_available: unwrap(replay.isReplayAvailable()),
                        is_replay_started: unwrap(replay.isReplayStarted()),
                        is_autoplay_started: unwrap(replay.isAutoplayStarted()),
                        replay_mode: unwrap(replay.replayMode()),
                        current_date: unwrap(replay.currentDate()),
                        autoplay_delay: unwrap(replay.autoplayDelay()),
                        position: unwrap(replay.position()),
                        realized_pnl: unwrap(replay.realizedPL()),
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

    normalize_replay_status(data)
}

fn normalize_replay_action(data: Value) -> Result<Value, AppError> {
    if data.get("ok").and_then(Value::as_bool) == Some(false) {
        let message = data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("TradingView replay operation failed");
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message).with_details(data));
    }

    if data.get("action").is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView replay operation did not return an action",
        )
        .with_details(data));
    }

    let mut payload = data;
    if let Some(object) = payload.as_object_mut() {
        object.remove("ok");
    }
    Ok(payload)
}

fn normalize_replay_status(data: Value) -> Result<Value, AppError> {
    if data.get("ok").and_then(Value::as_bool) == Some(false) {
        let message = data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("TradingView replay API is not available");
        return Err(AppError::new(ErrorKind::InternalApiUnavailable, message).with_details(data));
    }

    if data.get("is_replay_available").is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView replay status did not return replay state",
        )
        .with_details(data));
    }

    Ok(json!({
        "is_replay_available": data.get("is_replay_available").cloned().unwrap_or(Value::Null),
        "is_replay_started": data.get("is_replay_started").cloned().unwrap_or(Value::Null),
        "is_autoplay_started": data.get("is_autoplay_started").cloned().unwrap_or(Value::Null),
        "replay_mode": data.get("replay_mode").cloned().unwrap_or(Value::Null),
        "current_date": data.get("current_date").cloned().unwrap_or(Value::Null),
        "autoplay_delay": data.get("autoplay_delay").cloned().unwrap_or(Value::Null),
        "position": data.get("position").cloned().unwrap_or(Value::Null),
        "realized_pnl": data.get("realized_pnl").cloned().unwrap_or(Value::Null),
        "source": data.get("source").cloned().unwrap_or_else(|| json!("internal_api")),
    }))
}

fn parse_replay_date_ms(date: &str) -> Result<i64, AppError> {
    let trimmed = date.trim();
    let parts = trimmed.split('-').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(invalid_replay_date(date));
    }

    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| invalid_replay_date(date))?;
    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| invalid_replay_date(date))?;
    let day = parts[2]
        .parse::<u32>()
        .map_err(|_| invalid_replay_date(date))?;
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return Err(invalid_replay_date(date));
    }

    let days = days_from_civil(year, month, day);
    Ok(days * 86_400_000)
}

fn invalid_replay_date(date: &str) -> AppError {
    AppError::new(
        ErrorKind::Validation,
        format!("Invalid replay date: {date}. Use YYYY-MM-DD."),
    )
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = (year - era * 400) as u32;
    let month = month as i32;
    let day = day as i32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era =
        year_of_era as i32 * 365 + year_of_era as i32 / 4 - year_of_era as i32 / 100 + day_of_year;
    (era * 146_097 + day_of_era - 719_468) as i64
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn replay_start_serializes_date_as_timestamp() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": true,
            "action": "started",
            "replay_started": true,
            "date": "2026-04-01",
            "current_date": 1775001600000i64,
            "source": "internal_api"
        })]);

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
        let mut runtime = FakeRuntime::new([json!({
            "ok": false,
            "error_kind": "internal_api_unavailable",
            "message": "Replay is not available for the current symbol/timeframe"
        })]);

        let error = replay_start(&mut runtime, None).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn replay_step_requires_started_replay() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": false,
            "error_kind": "validation",
            "message": "Replay is not started. Use replay start first."
        })]);

        let error = replay_step(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn replay_step_returns_previous_and_current_date() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": true,
            "action": "step",
            "previous_date": 1775001600000i64,
            "current_date": 1775088000000i64,
            "source": "internal_api"
        })]);

        let result = replay_step(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "step");
        assert_eq!(result["previous_date"], 1775001600000i64);
        assert_eq!(result["current_date"], 1775088000000i64);
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn replay_stop_returns_already_stopped() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": true,
            "action": "already_stopped",
            "replay_started": false,
            "source": "internal_api"
        })]);

        let result = replay_stop(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "already_stopped");
        assert_eq!(result["replay_started"], false);
        assert!(!runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn replay_stop_returns_stopped_action() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": true,
            "action": "replay_stopped",
            "replay_started": false,
            "source": "internal_api"
        })]);

        let result = replay_stop(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "replay_stopped");
    }

    #[tokio::test]
    async fn replay_autoplay_accepts_known_delays() {
        for speed in VALID_AUTOPLAY_DELAYS {
            let mut runtime = FakeRuntime::new([json!({
                "ok": true,
                "action": "autoplay",
                "autoplay_active": true,
                "delay_ms": speed,
                "requested_delay_ms": speed,
                "source": "internal_api"
            })]);

            let result = replay_autoplay(&mut runtime, Some(speed)).await.unwrap();

            assert_eq!(result["action"], "autoplay");
            assert_eq!(result["autoplay_active"], true);
            assert_eq!(result["delay_ms"], speed);
            assert_eq!(result["requested_delay_ms"], speed);
            assert!(runtime.evaluated[0].0.contains("changeAutoplayDelay"));
        }
    }

    #[tokio::test]
    async fn replay_autoplay_rejects_invalid_speed_before_evaluating() {
        for speed in [50, 99, 101, 500, 750, 1500, 9999, 20000] {
            let mut runtime = FakeRuntime::new([]);

            let error = replay_autoplay(&mut runtime, Some(speed))
                .await
                .unwrap_err();

            assert_eq!(error.kind, ErrorKind::Validation);
            assert!(runtime.evaluated.is_empty());
        }
    }

    #[tokio::test]
    async fn replay_autoplay_toggles_without_speed_change_when_omitted_or_zero() {
        for speed in [None, Some(0)] {
            let mut runtime = FakeRuntime::new([json!({
                "ok": true,
                "action": "autoplay",
                "autoplay_active": false,
                "delay_ms": 300,
                "requested_delay_ms": null,
                "source": "internal_api"
            })]);

            let result = replay_autoplay(&mut runtime, speed).await.unwrap();

            assert_eq!(result["action"], "autoplay");
            assert_eq!(result["requested_delay_ms"], Value::Null);
            assert!(runtime.evaluated[0].0.contains("var requestedDelay = null"));
            assert!(!runtime.evaluated[0].0.contains("changeAutoplayDelay"));
        }
    }

    #[tokio::test]
    async fn replay_autoplay_requires_started_replay() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": false,
            "error_kind": "validation",
            "message": "Replay is not started. Use replay start first."
        })]);

        let error = replay_autoplay(&mut runtime, Some(1000)).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn replay_autoplay_maps_missing_api_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": false,
            "error_kind": "internal_api_unavailable",
            "message": "TradingView replay API is not available"
        })]);

        let error = replay_autoplay(&mut runtime, None).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn replay_trade_accepts_supported_actions() {
        for (action, expected_method) in [
            ("buy", "replay.buy()"),
            ("sell", "replay.sell()"),
            ("close", "replay.closePosition()"),
        ] {
            let mut runtime = FakeRuntime::new([json!({
                "ok": true,
                "action": action,
                "position": {"side": action},
                "realized_pnl": 50.5,
                "source": "internal_api"
            })]);

            let result = replay_trade(&mut runtime, action).await.unwrap();

            assert_eq!(result["action"], action);
            assert_eq!(result["position"], json!({"side": action}));
            assert_eq!(result["realized_pnl"], 50.5);
            assert_eq!(result["source"], "internal_api");
            assert!(runtime.evaluated[0].0.contains(expected_method));
        }
    }

    #[tokio::test]
    async fn replay_trade_rejects_invalid_action_before_evaluating() {
        let mut runtime = FakeRuntime::new([]);

        let error = replay_trade(&mut runtime, "hold").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn replay_trade_requires_started_replay() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": false,
            "error_kind": "validation",
            "message": "Replay is not started. Use replay start first."
        })]);

        let error = replay_trade(&mut runtime, "buy").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn replay_trade_maps_missing_api_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": false,
            "error_kind": "internal_api_unavailable",
            "message": "TradingView replay API is not available"
        })]);

        let error = replay_trade(&mut runtime, "close").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn replay_status_returns_practical_old_cli_fields() {
        let payload = json!({
            "ok": true,
            "is_replay_available": true,
            "is_replay_started": true,
            "is_autoplay_started": false,
            "replay_mode": "ActiveChart",
            "current_date": 1713916800000i64,
            "autoplay_delay": 1000,
            "position": {"side": "long"},
            "realized_pnl": 12.5,
            "source": "internal_api"
        });
        let mut runtime = FakeRuntime::new([payload]);

        let result = replay_status(&mut runtime).await.unwrap();

        assert_eq!(result["is_replay_available"], true);
        assert_eq!(result["is_replay_started"], true);
        assert_eq!(result["is_autoplay_started"], false);
        assert_eq!(result["replay_mode"], "ActiveChart");
        assert_eq!(result["current_date"], 1713916800000i64);
        assert_eq!(result["autoplay_delay"], 1000);
        assert_eq!(result["position"], json!({"side": "long"}));
        assert_eq!(result["realized_pnl"], 12.5);
        assert_eq!(result["source"], "internal_api");
        assert!(runtime.evaluated[0].0.contains("_replayApi"));
        assert!(!runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn replay_status_maps_missing_api_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": false,
            "error_kind": "internal_api_unavailable",
            "message": "TradingView replay API is not available"
        })]);

        let error = replay_status(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.exit_code(), 3);
        assert_eq!(
            error.details.unwrap()["error_kind"],
            "internal_api_unavailable"
        );
    }

    #[tokio::test]
    async fn replay_status_rejects_unexpected_payload() {
        let mut runtime = FakeRuntime::new([json!({"ok": true})]);

        let error = replay_status(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
