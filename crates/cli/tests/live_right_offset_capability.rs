use std::time::Duration;

use serde_json::Value;
use tradingview_cdp::{CdpClient, RuntimeEvaluator, TransportConfig, discover_target};

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_RIGHT_OFFSET_PROBE=1"]
async fn selected_chart_right_offset_read_only_capability_probe() {
    if std::env::var("TV_LIVE_RIGHT_OFFSET_PROBE").ok().as_deref() != Some("1") {
        panic!(
            "right-offset capability probe is gated; set TV_LIVE_RIGHT_OFFSET_PROBE=1 and run with --ignored"
        );
    }

    let target_id = std::env::var("TV_LIVE_RIGHT_OFFSET_TARGET_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .expect("TV_LIVE_RIGHT_OFFSET_TARGET_ID must select the intended chart target");
    let config = TransportConfig::from_env_with_target_id(Some(&target_id))
        .unwrap_or_else(|_| panic!("right-offset probe transport configuration was invalid"));
    let target = discover_target(&config).await.unwrap_or_else(|_| {
        panic!("right-offset probe could not resolve the selected chart target")
    });
    let mut runtime = CdpClient::connect(&target).await.unwrap_or_else(|_| {
        panic!("right-offset probe could not connect to the selected chart target")
    });

    let result = tokio::time::timeout(
        PROBE_TIMEOUT,
        runtime.evaluate(RIGHT_OFFSET_READ_ONLY_PROBE, false),
    )
    .await
    .unwrap_or_else(|_| panic!("right-offset read-only capability probe timed out"))
    .unwrap_or_else(|_| panic!("right-offset read-only capability evaluation failed"));

    assert_public_safe_capability(&result);
    println!(
        "right-offset read-only capability: model_resolved={} time_scale_resolved={} setter_callable={} getter_callable={} getter_number={} getter_finite={} getter_integer={} visible_range_readable={} current_value={}",
        bool_field(&result, "model_resolved"),
        bool_field(&result, "time_scale_resolved"),
        bool_field(&result, "setter_callable"),
        bool_field(&result, "getter_callable"),
        bool_field(&result, "getter_number"),
        bool_field(&result, "getter_finite"),
        bool_field(&result, "getter_integer"),
        bool_field(&result, "visible_range_readable"),
        result
            .get("right_offset")
            .and_then(Value::as_f64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unavailable>".to_string())
    );
}

fn assert_public_safe_capability(result: &Value) {
    let object = result
        .as_object()
        .expect("right-offset probe result should be an object");
    let allowed = [
        "model_resolved",
        "time_scale_resolved",
        "setter_callable",
        "getter_callable",
        "getter_number",
        "getter_finite",
        "getter_integer",
        "visible_range_readable",
        "right_offset",
    ];
    assert!(object.keys().all(|key| allowed.contains(&key.as_str())));
    for key in &allowed[..8] {
        assert!(object.get(*key).is_some_and(Value::is_boolean));
    }
    if bool_field(result, "getter_finite") {
        assert!(result.get("right_offset").is_some_and(Value::is_number));
    } else {
        assert!(result.get("right_offset").is_some_and(Value::is_null));
    }
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

const RIGHT_OFFSET_READ_ONLY_PROBE: &str = r#"
(() => {
    const result = {
        model_resolved: false,
        time_scale_resolved: false,
        setter_callable: false,
        getter_callable: false,
        getter_number: false,
        getter_finite: false,
        getter_integer: false,
        visible_range_readable: false,
        right_offset: null,
    };
    try {
        const chart = window.TradingViewApi?.activeChart?.();
        const model = chart?._chartWidget?.model?.();
        result.model_resolved = !!model;
        if (!model) return result;
        const ts = model.timeScale?.();
        result.time_scale_resolved = !!ts;
        if (!ts) return result;
        result.setter_callable = typeof ts.setRightOffset === "function";
        result.getter_callable = typeof ts.rightOffset === "function";
        if (result.getter_callable) {
            try {
                const value = ts.rightOffset();
                result.getter_number = typeof value === "number";
                result.getter_finite = Number.isFinite(value);
                result.getter_integer = Number.isInteger(value);
                if (result.getter_finite) {
                    result.right_offset = value;
                }
            } catch (_) {}
        }
        try {
            const visible = chart.getVisibleRange();
            result.visible_range_readable = !!visible
                && Number.isFinite(visible.from)
                && Number.isFinite(visible.to);
        } catch (_) {}
    } catch (_) {}
    return result;
})()
"#;
