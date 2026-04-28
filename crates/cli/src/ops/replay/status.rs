use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::payload::normalize_replay_status;

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_core::ErrorKind;

    #[tokio::test]
    async fn replay_status_returns_practical_old_cli_fields() {
        let mut runtime = FakeRuntime::new([json!({
            "ok": true,
            "is_replay_available": true,
            "is_replay_started": true,
            "is_autoplay_started": false,
            "replay_mode": "replay",
            "current_date": 1775001600000i64,
            "autoplay_delay": 1000,
            "position": 0,
            "realized_pnl": 12.5,
            "source": "internal_api"
        })]);

        let result = replay_status(&mut runtime).await.unwrap();

        assert_eq!(result["is_replay_available"], true);
        assert_eq!(result["is_replay_started"], true);
        assert_eq!(result["is_autoplay_started"], false);
        assert_eq!(result["current_date"], 1775001600000i64);
        assert_eq!(result["realized_pnl"], 12.5);
    }

    #[tokio::test]
    async fn replay_status_rejects_unexpected_payload() {
        let mut runtime = FakeRuntime::new([json!({"ok": true})]);
        let error = replay_status(&mut runtime).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn replay_status_maps_missing_api_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": false, "message": "TradingView replay API is not available"}),
        ]);
        let error = replay_status(&mut runtime).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
