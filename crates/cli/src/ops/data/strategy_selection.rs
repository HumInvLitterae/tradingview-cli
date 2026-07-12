use serde::Deserialize;
use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::super::common::CHART_API;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StrategyReadKind {
    Metrics,
    Trades,
    Equity,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct StrategyCapabilities {
    performance: bool,
    trades: bool,
    equity: bool,
}

impl StrategyCapabilities {
    fn any(&self) -> bool {
        self.performance || self.trades || self.equity
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct StrategyCandidate {
    entity_id: Option<String>,
    detection_signals: Vec<String>,
    visible: Option<bool>,
    capabilities: StrategyCapabilities,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct StrategySelection {
    pub(super) entity_id: Option<String>,
    pub(super) context: Value,
    inspection_failed: bool,
}

impl StrategySelection {
    pub(super) fn availability_status(&self) -> Option<&str> {
        self.context
            .get("availability_status")
            .and_then(Value::as_str)
    }

    pub(super) fn is_available(&self) -> bool {
        self.availability_status() == Some("available") && self.entity_id.is_some()
    }
}

#[derive(Debug, Deserialize)]
struct StrategyInspection {
    candidates: Vec<StrategyCandidate>,
    inspection_error: bool,
}

pub(super) async fn inspect_and_select_strategy(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<StrategySelection, AppError> {
    let raw = runtime
        .evaluate(&strategy_candidates_expression(), false)
        .await?;
    let Ok(inspection) = serde_json::from_value::<StrategyInspection>(raw) else {
        return Ok(inspection_failure());
    };
    if inspection.inspection_error {
        return Ok(inspection_failure());
    }
    Ok(select_strategy(&inspection.candidates))
}

pub(super) fn unavailable_payload(kind: StrategyReadKind, selection: StrategySelection) -> Value {
    let status = selection.availability_status().unwrap_or("unknown");
    let message = if selection.inspection_failed {
        "Strategy data is unavailable because selected-chart inspection failed."
    } else {
        match status {
            "ambiguous" => {
                "Multiple strategy candidates are equally plausible. Remove or hide extra strategies before reading strategy data."
            }
            "strategy_hidden" => "The selected strategy is hidden and its report is unavailable.",
            "report_not_ready" => "The selected strategy report is not ready.",
            "not_found" if kind != StrategyReadKind::Metrics => "No strategy found on chart.",
            _ => "No strategy found on chart. Add a strategy indicator first.",
        }
    };
    let next_action = if selection.inspection_failed {
        "Confirm selected-chart readiness, then rerun the command."
    } else {
        match status {
            "ambiguous" => {
                "Leave only one report-bearing strategy on the selected chart, then rerun the command."
            }
            "strategy_hidden" => {
                "Explicitly make the intended strategy visible, then rerun the command."
            }
            "report_not_ready" => {
                "Wait for the selected strategy to finish calculating, then rerun the command."
            }
            _ => "Add a strategy to the selected chart, then rerun the command.",
        }
    };

    let mut payload = match kind {
        StrategyReadKind::Metrics => json!({
            "metric_count": 0,
            "source": "internal_api",
            "metrics": {},
        }),
        StrategyReadKind::Trades => json!({
            "trade_count": 0,
            "source": "internal_api",
            "trades": [],
        }),
        StrategyReadKind::Equity => json!({
            "data_points": 0,
            "source": "internal_api",
            "data": [],
        }),
    };
    let object = payload
        .as_object_mut()
        .expect("strategy payload is an object");
    object.insert("error".into(), Value::String(message.into()));
    object.insert("next_action_hint".into(), Value::String(next_action.into()));
    object.insert("strategy_context".into(), selection.context);
    payload
}

pub(super) fn attach_strategy_context(mut payload: Value, selection: StrategySelection) -> Value {
    if let Some(object) = payload.as_object_mut() {
        object.insert("strategy_context".into(), selection.context);
    }
    payload
}

fn select_strategy(candidates: &[StrategyCandidate]) -> StrategySelection {
    if candidates.is_empty() {
        return StrategySelection {
            entity_id: None,
            context: context(candidates, None, None, "not_found"),
            inspection_failed: false,
        };
    }

    let (selected, reason) = if candidates.len() == 1 {
        (Some(&candidates[0]), Some("only_candidate"))
    } else {
        let report_candidates: Vec<_> = candidates
            .iter()
            .filter(|candidate| candidate.capabilities.any())
            .collect();
        if report_candidates.len() == 1 {
            (Some(report_candidates[0]), Some("only_report_available"))
        } else {
            (None, None)
        }
    };

    let Some(selected) = selected else {
        return StrategySelection {
            entity_id: None,
            context: context(candidates, None, None, "ambiguous"),
            inspection_failed: false,
        };
    };
    let status = if selected.visible == Some(false) {
        "strategy_hidden"
    } else if selected.entity_id.is_none() || !selected.capabilities.any() {
        "report_not_ready"
    } else {
        "available"
    };
    StrategySelection {
        entity_id: selected.entity_id.clone(),
        context: context(candidates, Some(selected), reason, status),
        inspection_failed: false,
    }
}

fn inspection_failure() -> StrategySelection {
    StrategySelection {
        entity_id: None,
        context: json!({
            "candidate_count": null,
            "selected_entity_id": null,
            "selected_title": null,
            "detection_signals": [],
            "selection_reason": null,
            "visible": null,
            "report_available": false,
            "panel_status": "unknown",
            "availability_status": "unknown",
        }),
        inspection_failed: true,
    }
}

fn context(
    candidates: &[StrategyCandidate],
    selected: Option<&StrategyCandidate>,
    reason: Option<&str>,
    status: &str,
) -> Value {
    json!({
        "candidate_count": candidates.len(),
        "selected_entity_id": selected.and_then(|candidate| candidate.entity_id.clone()),
        "selected_title": null,
        "detection_signals": selected
            .map(|candidate| candidate.detection_signals.clone())
            .unwrap_or_default(),
        "selection_reason": reason,
        "visible": selected.and_then(|candidate| candidate.visible),
        "report_available": selected.is_some_and(|candidate| candidate.capabilities.any()),
        "panel_status": "unknown",
        "availability_status": status,
    })
}

fn strategy_candidates_expression() -> String {
    STRATEGY_CANDIDATES_EXPRESSION.replace("{CHART_API}", CHART_API)
}

const STRATEGY_CANDIDATES_EXPRESSION: &str = concat!(
    "(function() {\n",
    "  function unwrap(value) {\n",
    "    try { return value && typeof value.value === 'function' ? value.value() : value; } catch(e) { return null; }\n",
    "  }\n",
    "  function sourceValue(source, key) {\n",
    "    try { var value = source && source[key]; return typeof value === 'function' ? unwrap(value.call(source)) : unwrap(value); } catch(e) { return null; }\n",
    "  }\n",
    "  function hasArray(value) { return Array.isArray(unwrap(value)); }\n",
    "  try {\n",
    "    var chart = {CHART_API}._chartWidget;\n",
    "    var sources = chart.model().model().dataSources();\n",
    "    var candidates = [];\n",
    "    for (var i = 0; i < sources.length; i++) {\n",
    "    var source = sources[i];\n",
    "    try {\n",
    "      var meta = source.metaInfo ? (source.metaInfo() || {}) : {};\n",
    "      var id = unwrap(meta.id);\n",
    "      var signals = [];\n",
    "      if (id && /^StrategyScript/.test(String(id))) signals.push('strategy_script_id');\n",
    "      if (unwrap(meta.isTVScriptStrategy) === true) signals.push('tv_script_strategy');\n",
    "      if (unwrap(meta.is_strategy) === true) signals.push('strategy_flag');\n",
    "      if (signals.length === 0) continue;\n",
    "      var report = unwrap(source._reportData) || sourceValue(source, 'reportData');\n",
    "      var performance = Boolean(report || sourceValue(source, 'performance'));\n",
    "      var trades = Boolean((report && Array.isArray(report.trades)) || hasArray(sourceValue(source, 'ordersData')) || hasArray(source._orders) || hasArray(sourceValue(source, 'tradesData')));\n",
    "      var bars = sourceValue(source, 'bars');\n",
    "      var equity = Boolean((report && Array.isArray(report.buyHold)) || hasArray(sourceValue(source, 'equityData')) || (bars && typeof bars.firstIndex === 'function' && typeof bars.lastIndex === 'function') || performance);\n",
    "      var properties = source.properties ? source.properties() : null;\n",
    "      var visible = properties && properties.visible !== undefined ? Boolean(unwrap(properties.visible)) : null;\n",
    "      if (visible === null && source.isVisible !== undefined) visible = Boolean(sourceValue(source, 'isVisible'));\n",
    "      candidates.push({ entity_id: sourceValue(source, 'id') || sourceValue(source, 'entityId'), detection_signals: signals, visible: visible, capabilities: { performance: performance, trades: trades, equity: equity } });\n",
    "    } catch(e) {}\n",
    "    }\n",
    "    return { candidates: candidates, inspection_error: false };\n",
    "  } catch(e) {\n",
    "    return { candidates: [], inspection_error: true };\n",
    "  }\n",
    "})()"
);

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        id: Option<&str>,
        visible: Option<bool>,
        performance: bool,
        trades: bool,
        equity: bool,
    ) -> StrategyCandidate {
        StrategyCandidate {
            entity_id: id.map(str::to_owned),
            detection_signals: vec!["tv_script_strategy".into()],
            visible,
            capabilities: StrategyCapabilities {
                performance,
                trades,
                equity,
            },
        }
    }

    #[test]
    fn no_candidate_is_not_found() {
        let selection = select_strategy(&[]);
        assert_eq!(selection.availability_status(), Some("not_found"));
        assert_eq!(selection.context["candidate_count"], 0);
    }

    #[test]
    fn one_candidate_accepts_every_preserved_reader_capability() {
        for capabilities in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let selection = select_strategy(&[candidate(
                Some("study-1"),
                Some(true),
                capabilities.0,
                capabilities.1,
                capabilities.2,
            )]);
            assert!(selection.is_available());
            assert_eq!(selection.context["selection_reason"], "only_candidate");
            assert_eq!(selection.context["selected_title"], Value::Null);
        }
    }

    #[test]
    fn one_report_candidate_wins_independently_of_order() {
        for candidates in [
            vec![
                candidate(Some("unready"), Some(true), false, false, false),
                candidate(Some("ready"), Some(true), false, true, false),
            ],
            vec![
                candidate(Some("ready"), Some(true), false, true, false),
                candidate(Some("unready"), Some(true), false, false, false),
            ],
        ] {
            let selection = select_strategy(&candidates);
            assert_eq!(selection.entity_id.as_deref(), Some("ready"));
            assert_eq!(
                selection.context["selection_reason"],
                "only_report_available"
            );
        }
    }

    #[test]
    fn equal_report_candidates_are_ambiguous() {
        let selection = select_strategy(&[
            candidate(Some("first"), Some(true), true, false, false),
            candidate(Some("second"), Some(true), false, true, false),
        ]);
        assert_eq!(selection.availability_status(), Some("ambiguous"));
        assert_eq!(selection.entity_id, None);
    }

    #[test]
    fn hidden_and_unready_states_are_explicit() {
        let hidden = select_strategy(&[candidate(Some("hidden"), Some(false), true, false, false)]);
        assert_eq!(hidden.availability_status(), Some("strategy_hidden"));

        let unready =
            select_strategy(&[candidate(Some("unready"), Some(true), false, false, false)]);
        assert_eq!(unready.availability_status(), Some("report_not_ready"));
    }

    #[test]
    fn inspection_requires_explicit_strategy_metadata_and_covers_old_shapes() {
        let expression = strategy_candidates_expression();
        assert!(expression.contains("isTVScriptStrategy"));
        assert!(expression.contains("is_strategy"));
        assert!(!expression.contains("is_price_study"));
        assert!(expression.contains("return { candidates: [], inspection_error: true }"));
        for path in [
            "source._reportData",
            "sourceValue(source, 'reportData')",
            "sourceValue(source, 'performance')",
            "sourceValue(source, 'ordersData')",
            "source._orders",
            "sourceValue(source, 'tradesData')",
            "sourceValue(source, 'equityData')",
            "sourceValue(source, 'bars')",
        ] {
            assert!(expression.contains(path));
        }
    }
}
