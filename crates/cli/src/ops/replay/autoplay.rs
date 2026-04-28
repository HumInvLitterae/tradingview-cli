use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::payload::normalize_replay_action;
use super::validation::validate_replay_autoplay_speed;

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_core::ErrorKind;

    #[tokio::test]
    async fn replay_autoplay_rejects_invalid_speed_before_evaluating() {
        let mut runtime = FakeRuntime::new([]);
        let error = replay_autoplay(&mut runtime, Some(123)).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn replay_autoplay_requires_started_replay() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": false, "error_kind": "validation", "message": "Replay is not started. Use replay start first."}),
        ]);
        let error = replay_autoplay(&mut runtime, Some(1000)).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn replay_autoplay_maps_missing_api_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": false, "error_kind": "internal_api_unavailable", "message": "TradingView replay API is missing method: toggleAutoplay"}),
        ]);
        let error = replay_autoplay(&mut runtime, None).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn replay_autoplay_toggles_without_speed_change_when_omitted_or_zero() {
        for speed in [None, Some(0)] {
            let mut runtime = FakeRuntime::new([
                json!({"ok": true, "action": "autoplay", "autoplay_active": true, "delay_ms": 1000, "requested_delay_ms": null, "source": "internal_api"}),
            ]);
            let result = replay_autoplay(&mut runtime, speed).await.unwrap();
            assert_eq!(result["action"], "autoplay");
            assert!(runtime.evaluated[0].0.contains("requestedDelay = null"));
            assert!(
                !runtime.evaluated[0]
                    .0
                    .contains("changeAutoplayDelay(requestedDelay)")
            );
        }
    }

    #[tokio::test]
    async fn replay_autoplay_accepts_known_delays() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": true, "action": "autoplay", "autoplay_active": true, "delay_ms": 2000, "requested_delay_ms": 2000, "source": "internal_api"}),
        ]);
        let result = replay_autoplay(&mut runtime, Some(2000)).await.unwrap();
        assert_eq!(result["requested_delay_ms"], 2000);
        assert!(
            runtime.evaluated[0]
                .0
                .contains("changeAutoplayDelay(requestedDelay)")
        );
    }
}
