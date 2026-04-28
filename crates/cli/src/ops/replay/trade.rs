use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::payload::normalize_replay_action;
use super::validation::validate_replay_trade_action;

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_core::ErrorKind;

    #[tokio::test]
    async fn replay_trade_rejects_invalid_action_before_evaluating() {
        let mut runtime = FakeRuntime::new([]);
        let error = replay_trade(&mut runtime, "hold").await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn replay_trade_requires_started_replay() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": false, "error_kind": "validation", "message": "Replay is not started. Use replay start first."}),
        ]);
        let error = replay_trade(&mut runtime, "buy").await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn replay_trade_maps_missing_api_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([
            json!({"ok": false, "error_kind": "internal_api_unavailable", "message": "TradingView replay API is missing method: buy"}),
        ]);
        let error = replay_trade(&mut runtime, "buy").await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn replay_trade_accepts_supported_actions() {
        for (action, method) in [("buy", "buy"), ("sell", "sell"), ("close", "closePosition")] {
            let mut runtime = FakeRuntime::new([
                json!({"ok": true, "action": action, "position": 1, "realized_pnl": 2.5, "source": "internal_api"}),
            ]);
            let result = replay_trade(&mut runtime, action).await.unwrap();
            assert_eq!(result["action"], action);
            assert!(runtime.evaluated[0].0.contains(method));
        }
    }
}
