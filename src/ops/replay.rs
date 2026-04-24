use serde_json::{Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

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
