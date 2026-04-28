use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::common::js_string,
    payload::{normalize_alert_delete_all_payload, normalize_alert_delete_payload},
};

pub async fn alert_delete(
    runtime: &mut impl RuntimeEvaluator,
    alert_id: &str,
) -> Result<Value, AppError> {
    let alert_id = alert_id.trim();
    if alert_id.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Alert ID must not be empty",
        ));
    }

    let alert_id_literal = js_string(alert_id)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                const requestedAlertId = {alert_id_literal};

                function normalizeAlert(alert) {{
                    return {{
                        alert_id: alert.alert_id || alert.id || null,
                        symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                        type: alert.type || null,
                        message: alert.message || alert.description || '',
                        active: alert.active !== false,
                        condition: alert.condition || null,
                        resolution: alert.resolution || alert.interval || null,
                        created: alert.created || alert.create_time || null,
                        last_fired: alert.last_fired || alert.last_fire_time || null,
                        expiration: alert.expiration || alert.expire_time || null
                    }};
                }}

                async function listAlerts() {{
                    const response = await fetch('https://pricealerts.tradingview.com/list_alerts', {{
                        credentials: 'include',
                        headers: {{ 'accept': 'application/json' }}
                    }});
                    if (!response.ok) {{
                        return {{
                            ok: false,
                            error: 'HTTP ' + response.status + ': ' + response.statusText,
                            alerts: []
                        }};
                    }}
                    const data = await response.json();
                    if (data.err) {{
                        return {{
                            ok: false,
                            error: data.errmsg || (data.err && data.err.code) || 'Alert list failed',
                            alerts: []
                        }};
                    }}
                    const rows = Array.isArray(data.r) ? data.r : [];
                    return {{ ok: true, alerts: rows.map(normalizeAlert) }};
                }}

                function findAlert(alerts) {{
                    return alerts.find(function(alert) {{
                        return String(alert.alert_id) === String(requestedAlertId);
                    }}) || null;
                }}

                try {{
                    const before = await listAlerts();
                    if (!before.ok) {{
                        return {{
                            error: before.error,
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api'
                        }};
                    }}

                    const matched = findAlert(before.alerts);
                    if (!matched) {{
                        return {{
                            error: 'Alert not found: ' + requestedAlertId,
                            error_kind: 'validation',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: false
                        }};
                    }}

                    function wireAlertId(id) {{
                        return /^\d+$/.test(String(id)) ? Number(id) : id;
                    }}

                    async function deleteAlerts(ids) {{
                        const response = await fetch('https://pricealerts.tradingview.com/delete_alerts', {{
                            method: 'POST',
                            credentials: 'include',
                            body: JSON.stringify({{ payload: {{ alert_ids: ids }} }})
                        }});
                        if (!response.ok) {{
                            return {{
                                ok: false,
                                http_error: 'HTTP ' + response.status + ': ' + response.statusText,
                                data: null
                            }};
                        }}
                        const data = await response.json();
                        return {{ ok: !data.err, http_error: null, data }};
                    }}

                    const deleteAttempts = [];
                    const firstAlertIdValue = wireAlertId(requestedAlertId);
                    deleteAttempts.push(typeof firstAlertIdValue);
                    let deleteResult = await deleteAlerts([firstAlertIdValue]);
                    if (!deleteResult.ok && deleteResult.data && deleteResult.data.err && deleteResult.data.err.code === 'invalid_request' && typeof firstAlertIdValue !== 'string') {{
                        deleteAttempts.push('string');
                        deleteResult = await deleteAlerts([String(requestedAlertId)]);
                    }}

                    if (deleteResult.http_error) {{
                        const deleteData = deleteResult.data;
                        return {{
                            error: deleteResult.http_error,
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: true,
                            matched_alert: matched,
                            delete_attempts: deleteAttempts,
                            delete_response: deleteData
                        }};
                    }}

                    const deleteData = deleteResult.data;
                    if (!deleteResult.ok) {{
                        return {{
                            error: deleteData.errmsg || (deleteData.err && deleteData.err.code) || 'Alert delete failed',
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: true,
                            matched_alert: matched,
                            delete_attempts: deleteAttempts,
                            delete_response: deleteData
                        }};
                    }}

                    const after = await listAlerts();
                    if (!after.ok) {{
                        return {{
                            error: after.error,
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: true,
                            matched_alert: matched,
                            delete_attempts: deleteAttempts,
                            delete_response: deleteData
                        }};
                    }}

                    const matchedAfter = findAlert(after.alerts);
                    return {{
                        alert_id: requestedAlertId,
                        deleted: !matchedAfter,
                        source: 'internal_api',
                        before_count: before.alerts.length,
                        after_count: after.alerts.length,
                        matched_before: true,
                        matched_after: !!matchedAfter,
                        matched_alert: matched,
                        delete_attempts: deleteAttempts,
                        delete_response: deleteData
                    }};
                }} catch (error) {{
                    return {{
                        error: error && error.message ? error.message : String(error),
                        error_kind: 'internal_api_unavailable',
                        alert_id: requestedAlertId,
                        source: 'internal_api'
                    }};
                }}
            }})()
            "#
            ),
            true,
        )
        .await?;

    normalize_alert_delete_payload(result)
}

pub async fn alert_delete_all(
    runtime: &mut impl RuntimeEvaluator,
    dry_run: bool,
) -> Result<Value, AppError> {
    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                const dryRun = {dry_run};

                function normalizeAlert(alert) {{
                    return {{
                        alert_id: alert.alert_id || alert.id || null,
                        symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                        type: alert.type || null,
                        message: alert.message || alert.description || '',
                        active: alert.active !== false,
                        condition: alert.condition || null,
                        resolution: alert.resolution || alert.interval || null,
                        created: alert.created || alert.create_time || null,
                        last_fired: alert.last_fired || alert.last_fire_time || null,
                        expiration: alert.expiration || alert.expire_time || null
                    }};
                }}

                async function listAlerts() {{
                    const response = await fetch('https://pricealerts.tradingview.com/list_alerts', {{
                        credentials: 'include',
                        headers: {{ 'accept': 'application/json' }}
                    }});
                    if (!response.ok) {{
                        return {{
                            ok: false,
                            error: 'HTTP ' + response.status + ': ' + response.statusText,
                            alerts: []
                        }};
                    }}
                    const data = await response.json();
                    if (data.err) {{
                        return {{
                            ok: false,
                            error: data.errmsg || (data.err && data.err.code) || 'Alert list failed',
                            alerts: []
                        }};
                    }}
                    const rows = Array.isArray(data.r) ? data.r : [];
                    return {{ ok: true, alerts: rows.map(normalizeAlert) }};
                }}

                function alertIds(alerts) {{
                    return alerts
                        .map(function(alert) {{ return alert.alert_id; }})
                        .filter(function(id) {{ return id !== null && id !== undefined && String(id).trim() !== ''; }});
                }}

                function wireAlertId(id) {{
                        return /^\d+$/.test(String(id)) ? Number(id) : id;
                }}

                try {{
                    const before = await listAlerts();
                    if (!before.ok) {{
                        return {{
                            error: before.error,
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api'
                        }};
                    }}

                    const targetIds = alertIds(before.alerts);
                    if (targetIds.length !== before.alerts.length) {{
                        return {{
                            error: 'Alert list contained alerts without alert_id',
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts
                        }};
                    }}

                    if (dryRun) {{
                        return {{
                            action: 'dry_run',
                            dry_run: true,
                            deleted: false,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            after_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts
                        }};
                    }}

                    if (targetIds.length === 0) {{
                        return {{
                            action: 'noop',
                            dry_run: false,
                            deleted: false,
                            source: 'internal_api',
                            before_count: 0,
                            after_count: 0,
                            target_alert_ids: [],
                            target_alerts: []
                        }};
                    }}

                    const deleteResponse = await fetch('https://pricealerts.tradingview.com/delete_alerts', {{
                        method: 'POST',
                        credentials: 'include',
                        body: JSON.stringify({{ payload: {{ alert_ids: targetIds.map(wireAlertId) }} }})
                    }});
                    if (!deleteResponse.ok) {{
                        return {{
                            error: 'HTTP ' + deleteResponse.status + ': ' + deleteResponse.statusText,
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts
                        }};
                    }}

                    const deleteData = await deleteResponse.json();
                    if (deleteData.err) {{
                        return {{
                            error: deleteData.errmsg || (deleteData.err && deleteData.err.code) || 'Alert delete failed',
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts,
                            delete_response: deleteData
                        }};
                    }}

                    const after = await listAlerts();
                    if (!after.ok) {{
                        return {{
                            error: after.error,
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts,
                            delete_response: deleteData
                        }};
                    }}

                    const remainingTargetIds = new Set(alertIds(after.alerts).map(String));
                    const stillPresent = targetIds.filter(function(id) {{ return remainingTargetIds.has(String(id)); }});
                    return {{
                        action: 'delete_all',
                        dry_run: false,
                        deleted: stillPresent.length === 0,
                        source: 'internal_api',
                        before_count: before.alerts.length,
                        after_count: after.alerts.length,
                        target_alert_ids: targetIds,
                        target_alerts: before.alerts,
                        remaining_target_alert_ids: stillPresent,
                        delete_response: deleteData
                    }};
                }} catch (error) {{
                    return {{
                        error: error && error.message ? error.message : String(error),
                        error_kind: 'internal_api_unavailable',
                        source: 'internal_api'
                    }};
                }}
            }})()
            "#
            ),
            true,
        )
        .await?;

    normalize_alert_delete_all_payload(result)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn alert_delete_returns_practical_fields() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_id": "4546454367",
            "deleted": true,
            "source": "internal_api",
            "before_count": 1,
            "after_count": 0,
            "matched_before": true,
            "matched_after": false,
            "matched_alert": {
                "alert_id": "4546454367",
                "message": "smoke",
                "condition": {
                    "type": "alert_cond",
                    "series": [
                        {
                            "type": "study",
                            "pine_id": "USER;redacted;script"
                        }
                    ],
                    "inputs": {
                        "length": 21
                    }
                }
            },
            "delete_response": { "s": "ok" }
        })]));

        let data = alert_delete(&mut runtime, "4546454367").await.unwrap();

        assert_eq!(data["alert_id"], "4546454367");
        assert_eq!(data["deleted"], true);
        assert_eq!(data["source"], "internal_api");
        assert_eq!(data["before_count"], 1);
        assert_eq!(data["after_count"], 0);
        assert_eq!(data["matched_alert"]["message"], "smoke");
        assert_eq!(data["matched_alert"]["condition"]["type"], "alert_cond");
        assert_eq!(data["matched_alert"]["condition"]["has_study_series"], true);
        assert!(data["matched_alert"]["condition"].get("series").is_none());
        assert!(data["matched_alert"]["condition"].get("pine_id").is_none());
        assert!(data["matched_alert"]["condition"].get("inputs").is_none());
        assert!(runtime.evaluated[0].0.contains("delete_alerts"));
        assert!(runtime.evaluated[0].0.contains("alert_ids"));
        assert!(runtime.evaluated[0].0.contains("deleteAttempts"));
        assert!(!runtime.evaluated[0].0.contains("log_username"));
        assert!(!runtime.evaluated[0].0.contains("build_time"));
        assert!(runtime.evaluated[0].0.contains("\"4546454367\""));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_delete_rejects_empty_id_before_evaluating() {
        let mut runtime = FakeRuntime::new(VecDeque::new());

        let error = alert_delete(&mut runtime, " ").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn alert_delete_maps_missing_alert_to_validation() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "error": "Alert not found: missing",
            "error_kind": "validation",
            "alert_id": "missing",
            "source": "internal_api",
            "before_count": 3,
            "matched_before": false
        })]));

        let error = alert_delete(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["matched_before"], false);
    }

    #[tokio::test]
    async fn alert_delete_maps_failed_delete_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "error": "Alert delete failed",
            "error_kind": "internal_api_unavailable",
            "alert_id": "4546454367",
            "source": "internal_api",
            "matched_alert": {
                "alert_id": "4546454367",
                "condition": {
                    "type": "alert_cond",
                    "series": [
                        {
                            "type": "study",
                            "pine_id": "USER;redacted;script"
                        }
                    ],
                    "inputs": {
                        "length": 21
                    }
                }
            }
        })]));

        let error = alert_delete(&mut runtime, "4546454367").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert!(
            details["matched_alert"]["condition"]
                .get("series")
                .is_none()
        );
        assert!(
            details["matched_alert"]["condition"]
                .get("inputs")
                .is_none()
        );
        assert!(
            details["matched_alert"]["condition"]
                .get("pine_id")
                .is_none()
        );
    }

    #[tokio::test]
    async fn alert_delete_all_returns_dry_run_targets() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "dry_run",
            "dry_run": true,
            "deleted": false,
            "source": "internal_api",
            "before_count": 2,
            "after_count": 2,
            "target_alert_ids": ["1", "2"],
            "target_alerts": [
                { "alert_id": "1", "message": "one" },
                { "alert_id": "2", "message": "two" }
            ]
        })]));

        let data = alert_delete_all(&mut runtime, true).await.unwrap();

        assert_eq!(data["action"], "dry_run");
        assert_eq!(data["dry_run"], true);
        assert_eq!(data["deleted"], false);
        assert_eq!(data["before_count"], 2);
        assert_eq!(data["target_alert_ids"][0], "1");
        assert!(runtime.evaluated[0].0.contains("list_alerts"));
        assert!(runtime.evaluated[0].0.contains("delete_alerts"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_delete_all_returns_noop_when_empty() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "noop",
            "dry_run": false,
            "deleted": false,
            "source": "internal_api",
            "before_count": 0,
            "after_count": 0,
            "target_alert_ids": [],
            "target_alerts": []
        })]));

        let data = alert_delete_all(&mut runtime, false).await.unwrap();

        assert_eq!(data["action"], "noop");
        assert_eq!(data["deleted"], false);
        assert_eq!(data["before_count"], 0);
        assert_eq!(data["after_count"], 0);
    }

    #[tokio::test]
    async fn alert_delete_all_returns_success_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "delete_all",
            "dry_run": false,
            "deleted": true,
            "source": "internal_api",
            "before_count": 2,
            "after_count": 0,
            "target_alert_ids": ["1", "2"],
            "target_alerts": [
                { "alert_id": "1", "message": "one" },
                { "alert_id": "2", "message": "two" }
            ],
            "remaining_target_alert_ids": [],
            "delete_response": { "s": "ok" }
        })]));

        let data = alert_delete_all(&mut runtime, false).await.unwrap();

        assert_eq!(data["action"], "delete_all");
        assert_eq!(data["deleted"], true);
        assert_eq!(data["after_count"], 0);
        assert_eq!(
            data["remaining_target_alert_ids"].as_array().unwrap().len(),
            0
        );
        assert!(runtime.evaluated[0].0.contains("delete_alerts"));
        assert!(!runtime.evaluated[0].0.contains("log_username"));
        assert!(!runtime.evaluated[0].0.contains("build_time"));
    }

    #[tokio::test]
    async fn alert_delete_all_requires_target_absence_after_delete() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "delete_all",
            "dry_run": false,
            "deleted": false,
            "source": "internal_api",
            "before_count": 2,
            "after_count": 1,
            "target_alert_ids": ["1", "2"],
            "target_alerts": [
                { "alert_id": "1", "message": "one" },
                { "alert_id": "2", "message": "two" }
            ],
            "remaining_target_alert_ids": ["2"]
        })]));

        let error = alert_delete_all(&mut runtime, false).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Alert delete --all did not remove all target alerts"
        );
    }
}
