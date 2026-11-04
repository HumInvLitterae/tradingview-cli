use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{CHART_API, js_string};

pub async fn drawing_remove(
    runtime: &mut impl RuntimeEvaluator,
    entity_id: &str,
) -> Result<Value, AppError> {
    let entity_id_literal = js_string(entity_id)?;
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    function getShapeByIdSafe(api, entityId) {{
                        try {{ return api.getShapeById(entityId); }} catch(e) {{ return null; }}
                    }}

                    var api = {CHART_API};
                    var entityId = {entity_id_literal};
                    var before = api.getAllShapes();
                    var beforeShape = getShapeByIdSafe(api, entityId);
                    if (!beforeShape) {{
                        return {{
                            error: "Shape not found: " + entityId,
                            available: before.map(function(shape) {{ return shape.id; }})
                        }};
                    }}

                    api.removeEntity(entityId);
                    var after = api.getAllShapes();
                    var afterShape = getShapeByIdSafe(api, entityId);
                    return {{
                        action: "remove",
                        entity_id: entityId,
                        removed: !afterShape,
                        before_count: before.length,
                        remaining_shapes: after.length
                    }};
                }})()
                "#
            ),
            false,
        )
        .await?;

    if let Some(message) = data.get("error").and_then(Value::as_str) {
        return Err(AppError::new(ErrorKind::Validation, message.to_string()).with_details(data));
    }
    if data.get("removed").and_then(Value::as_bool) != Some(true) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Drawing remove did not remove entity: {entity_id}"),
        )
        .with_details(data));
    }

    Ok(data)
}

pub async fn drawing_clear(
    runtime: &mut impl RuntimeEvaluator,
    dry_run: bool,
) -> Result<Value, AppError> {
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    function shapeName(shape) {{
                        try {{
                            if (typeof shape.name === "function") return shape.name();
                            if (typeof shape.title === "function") return shape.title();
                            return shape.name || shape.title || null;
                        }} catch(e) {{
                            return null;
                        }}
                    }}
                    function shapeSummary(shape) {{
                        return {{
                            id: shape.id,
                            name: shapeName(shape)
                        }};
                    }}

                    var api = {CHART_API};
                    var before = api.getAllShapes();
                    var targets = before.map(shapeSummary);
                    var dryRun = {dry_run};
                    if (dryRun) {{
                        return {{
                            action: "dry_run",
                            dry_run: true,
                            before_count: before.length,
                            would_clear_count: before.length,
                            cleared_entities: targets,
                            source: "chart_api"
                        }};
                    }}

                    if (before.length === 0) {{
                        return {{
                            action: "noop",
                            cleared: false,
                            before_count: 0,
                            after_count: 0,
                            cleared_entities: [],
                            source: "chart_api"
                        }};
                    }}

                    api.removeAllShapes();
                    var after = api.getAllShapes();
                    return {{
                        action: "cleared",
                        cleared: after.length === 0,
                        before_count: before.length,
                        after_count: after.length,
                        cleared_entities: targets,
                        source: "chart_api"
                    }};
                }})()
                "#,
                dry_run = if dry_run { "true" } else { "false" },
            ),
            false,
        )
        .await?;

    if !dry_run && data.get("after_count").and_then(Value::as_u64) != Some(0) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Drawing clear did not remove all drawings",
        )
        .with_details(data));
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::{FakeRuntime, run_node_json};

    use super::*;
    use tradingview_core::ErrorKind;

    #[tokio::test]
    async fn drawing_remove_requires_post_delete_absence() {
        let mut runtime = FakeRuntime::new([
            json!({"action": "remove", "entity_id": "shape123", "removed": false}),
        ]);

        let err = drawing_remove(&mut runtime, "shape123").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    #[ignore = "run through `node scripts/check-js-contracts.mjs desktop`"]
    async fn javascript_desktop_drawing_remove_and_clear_touch_only_their_targets() {
        let mut expressions = Vec::new();
        let mut runtime = FakeRuntime::new([]);
        let _ = drawing_remove(&mut runtime, "shape'2").await;
        expressions.push(runtime.evaluated[0].0.clone());
        for dry_run in [true, false] {
            let mut runtime = FakeRuntime::new([]);
            let _ = drawing_clear(&mut runtime, dry_run).await;
            expressions.push(runtime.evaluated[0].0.clone());
        }
        let expressions = serde_json::to_string(&expressions).unwrap();
        let results = run_node_json(&format!(
            r#"
            const [remove, dryRun, clear] = {expressions};
            // A chart whose mutations are recorded; `stuck` ignores them.
            function chart(ids, stuck = false) {{
                const mutations = [];
                let shapes = ids.map(id => ({{ id, name: () => 'Line ' + id }}));
                const api = {{
                    getAllShapes: () => shapes.slice(),
                    getShapeById: id => {{
                        const shape = shapes.find(shape => shape.id === id);
                        if (!shape) throw new Error('missing');
                        return shape;
                    }},
                    removeEntity: id => {{
                        mutations.push(['removeEntity', id]);
                        if (!stuck) shapes = shapes.filter(shape => shape.id !== id);
                    }},
                    removeAllShapes: () => {{
                        mutations.push(['removeAllShapes']);
                        if (!stuck) shapes = [];
                    }}
                }};
                global.window = {{ TradingViewApi: {{ _activeChartWidgetWV: {{ value: () => api }} }} }};
                return mutations;
            }}
            const results = [];
            for (const [expression, ids, stuck] of [
                [remove, ['shape1', "shape'2", 'shape3'], false],
                [remove, ['shape1'], false],
                [remove, ["shape'2"], true],
                [dryRun, ['shape1', 'shape2'], false],
                [clear, ['shape1', 'shape2'], false],
                [clear, ['shape1'], true],
                [clear, [], false]
            ]) {{
                const mutations = chart(ids, stuck);
                results.push({{ result: eval(expression), mutations }});
            }}
            console.log(JSON.stringify(results));
            "#
        ));

        // Remove deletes only the requested ID and reads the chart again.
        assert_eq!(
            results[0]["mutations"],
            json!([["removeEntity", "shape'2"]])
        );
        assert_eq!(
            results[0]["result"],
            json!({"action": "remove", "entity_id": "shape'2", "removed": true,
                "before_count": 3, "remaining_shapes": 2})
        );
        assert_eq!(results[1]["mutations"], json!([]));
        assert_eq!(results[1]["result"]["available"], json!(["shape1"]));
        assert_eq!(results[2]["result"]["removed"], false);

        // A dry run lists the targets without removing anything.
        assert_eq!(results[3]["mutations"], json!([]));
        assert_eq!(
            results[3]["result"]["cleared_entities"],
            json!([{"id": "shape1", "name": "Line shape1"}, {"id": "shape2", "name": "Line shape2"}])
        );
        assert_eq!(results[3]["result"]["would_clear_count"], 2);

        // Clearing removes once and reports what the chart shows afterwards.
        assert_eq!(results[4]["mutations"], json!([["removeAllShapes"]]));
        assert_eq!(results[4]["result"]["after_count"], 0);
        assert_eq!(results[4]["result"]["cleared"], true);
        assert_eq!(results[5]["result"]["after_count"], 1);
        assert_eq!(results[5]["result"]["cleared"], false);
        assert_eq!(results[6]["mutations"], json!([]));
        assert_eq!(results[6]["result"]["action"], "noop");
    }

    #[tokio::test]
    async fn drawing_remove_returns_success_payload() {
        let payload = json!({"action": "remove", "entity_id": "shape123", "removed": true, "before_count": 2, "remaining_shapes": 1});
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_remove(&mut runtime, "shape123").await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("removeEntity"));
    }

    #[tokio::test]
    async fn drawing_clear_dry_run_returns_targets() {
        let payload = json!({
            "action": "dry_run", "dry_run": true, "before_count": 2, "would_clear_count": 2,
            "cleared_entities": [{"id": "shape1", "name": "Horizontal Line"}, {"id": "shape2", "name": "Trend Line"}],
            "source": "chart_api"
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_clear(&mut runtime, true).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("getAllShapes"));
        assert!(runtime.evaluated[0].0.contains("removeAllShapes"));
    }

    #[tokio::test]
    async fn drawing_clear_returns_success_payload() {
        let payload = json!({
            "action": "cleared", "cleared": true, "before_count": 2, "after_count": 0,
            "cleared_entities": [{"id": "shape1", "name": "Horizontal Line"}, {"id": "shape2", "name": null}],
            "source": "chart_api"
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_clear(&mut runtime, false).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("removeAllShapes"));
    }

    #[tokio::test]
    async fn drawing_clear_requires_empty_post_delete_state() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "cleared", "cleared": false, "before_count": 2, "after_count": 1,
            "cleared_entities": [{"id": "shape1", "name": "Horizontal Line"}, {"id": "shape2", "name": "Trend Line"}],
            "source": "chart_api"
        })]);

        let err = drawing_clear(&mut runtime, false).await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }
}
