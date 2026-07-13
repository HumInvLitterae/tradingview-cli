use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::common::{CHART_API, CHART_WIDGET_COLLECTION, js_string};
use super::data::study_values::{identity_helper_js, normalize_study_value_rows};

const MIN_STREAM_INTERVAL_MS: u64 = 100;
const STREAM_CONTRACT_VERSION: &str = "stream.v1";
const STREAM_SOURCE: &str = "desktop_chart_stream";
const STREAM_SOURCE_CATEGORY: &str = "desktop_backed_read";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    Quote,
    Bars,
    Values,
    Lines,
    Labels,
    Tables,
    All,
}

impl StreamKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Quote => "quote",
            Self::Bars => "bars",
            Self::Values => "values",
            Self::Lines => "lines",
            Self::Labels => "labels",
            Self::Tables => "tables",
            Self::All => "all",
        }
    }

    pub fn default_interval_ms(self) -> u64 {
        match self {
            Self::Quote => 300,
            Self::Bars | Self::Values | Self::All => 500,
            Self::Lines | Self::Labels => 1000,
            Self::Tables => 2000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamEndReason {
    Completed,
    DurationElapsed,
    MaxEventsReached,
}

impl StreamEndReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::DurationElapsed => "duration_elapsed",
            Self::MaxEventsReached => "max_events_reached",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamRequest {
    pub kind: StreamKind,
    pub interval_ms: u64,
    pub filter: Option<String>,
    pub duration_ms: Option<u64>,
    pub max_events: Option<u64>,
    pub heartbeat_ms: Option<u64>,
}

impl StreamRequest {
    pub fn new(
        kind: StreamKind,
        interval_ms: Option<u64>,
        filter: Option<String>,
    ) -> Result<Self, AppError> {
        Self::with_controls(kind, interval_ms, filter, None, None, None)
    }

    pub fn with_controls(
        kind: StreamKind,
        interval_ms: Option<u64>,
        filter: Option<String>,
        duration_ms: Option<u64>,
        max_events: Option<u64>,
        heartbeat_ms: Option<u64>,
    ) -> Result<Self, AppError> {
        let interval_ms = interval_ms.unwrap_or_else(|| kind.default_interval_ms());
        validate_stream_interval(interval_ms)?;
        validate_positive_optional("duration_ms", duration_ms)?;
        validate_positive_optional("max_events", max_events)?;
        validate_positive_optional("heartbeat_ms", heartbeat_ms)?;
        if let Some(heartbeat_ms) = heartbeat_ms {
            validate_stream_interval(heartbeat_ms)?;
        }
        let filter = filter.and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        });
        Ok(Self {
            kind,
            interval_ms,
            filter,
            duration_ms,
            max_events,
            heartbeat_ms,
        })
    }
}

#[derive(Debug, Default)]
pub struct StreamDedupe {
    last_sample: Option<Value>,
}

impl StreamDedupe {
    pub fn should_emit(&mut self, sample: &Value) -> bool {
        let comparable = comparable_sample(sample);
        if self.last_sample.as_ref() == Some(&comparable) {
            return false;
        }
        self.last_sample = Some(comparable);
        true
    }
}

pub fn validate_stream_interval(interval_ms: u64) -> Result<(), AppError> {
    if interval_ms < MIN_STREAM_INTERVAL_MS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("Stream interval must be at least {MIN_STREAM_INTERVAL_MS}ms"),
        )
        .with_details(json!({
            "interval_ms": interval_ms,
            "minimum_interval_ms": MIN_STREAM_INTERVAL_MS,
        })));
    }
    Ok(())
}

fn validate_positive_optional(field: &str, value: Option<u64>) -> Result<(), AppError> {
    if value == Some(0) {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("{field} must be greater than zero"),
        )
        .with_details(json!({
            "field": field,
            "value": 0,
        })));
    }
    Ok(())
}

pub async fn stream_sample(
    runtime: &mut impl RuntimeEvaluator,
    request: &StreamRequest,
) -> Result<Value, AppError> {
    let expression = stream_expression(request)?;
    let mut sample = runtime.evaluate(&expression, false).await?;
    if sample.is_null() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stream sample was empty",
        ));
    }
    if request.kind == StreamKind::Values {
        normalize_study_value_rows(&mut sample);
    }
    add_stream_metadata(&mut sample, request.kind)?;
    Ok(sample)
}

fn add_stream_metadata(sample: &mut Value, kind: StreamKind) -> Result<(), AppError> {
    let Some(object) = sample.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stream sample was not an object",
        )
        .with_details(sample.clone()));
    };

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
        .as_millis() as u64;
    object.insert("_stream".to_string(), json!(kind.label()));
    object.insert("_ts".to_string(), json!(ts));
    object.insert("_event".to_string(), json!("sample"));
    object.insert(
        "contract_version".to_string(),
        json!(STREAM_CONTRACT_VERSION),
    );
    object.insert("source".to_string(), json!(STREAM_SOURCE));
    object.insert("source_category".to_string(), json!(STREAM_SOURCE_CATEGORY));
    object.insert("requires_desktop".to_string(), json!(true));
    object.insert("non_mutating".to_string(), json!(true));
    Ok(())
}

pub fn stream_heartbeat(
    request: &StreamRequest,
    elapsed_ms: u64,
    sample_count: u64,
    last_sample_ts: Option<u64>,
) -> Result<Value, AppError> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
        .as_millis() as u64;
    Ok(json!({
        "_stream": request.kind.label(),
        "_event": "heartbeat",
        "_ts": ts,
        "contract_version": STREAM_CONTRACT_VERSION,
        "source": STREAM_SOURCE,
        "source_category": STREAM_SOURCE_CATEGORY,
        "requires_desktop": true,
        "non_mutating": true,
        "elapsed_ms": elapsed_ms,
        "sample_count": sample_count,
        "last_sample_ts": last_sample_ts,
    }))
}

pub fn stream_summary(
    request: &StreamRequest,
    elapsed_ms: u64,
    sample_count: u64,
    heartbeat_count: u64,
    last_sample_ts: Option<u64>,
    end_reason: StreamEndReason,
) -> Result<Value, AppError> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
        .as_millis() as u64;
    Ok(json!({
        "_stream": request.kind.label(),
        "_event": "summary",
        "_ts": ts,
        "contract_version": STREAM_CONTRACT_VERSION,
        "source": STREAM_SOURCE,
        "source_category": STREAM_SOURCE_CATEGORY,
        "requires_desktop": true,
        "non_mutating": true,
        "elapsed_ms": elapsed_ms,
        "sample_count": sample_count,
        "heartbeat_count": heartbeat_count,
        "last_sample_ts": last_sample_ts,
        "duration_ms": request.duration_ms,
        "max_events": request.max_events,
        "end_reason": end_reason.label(),
    }))
}

fn comparable_sample(sample: &Value) -> Value {
    let mut comparable = sample.clone();
    if let Some(object) = comparable.as_object_mut() {
        object.remove("_ts");
        object.remove("_event");
    }
    comparable
}

fn stream_expression(request: &StreamRequest) -> Result<String, AppError> {
    match request.kind {
        StreamKind::Quote => Ok(quote_expression()),
        StreamKind::Bars => Ok(bars_expression()),
        StreamKind::Values => Ok(values_expression()),
        StreamKind::Lines => filtered_study_expression("lines", request.filter.as_deref()),
        StreamKind::Labels => filtered_study_expression("labels", request.filter.as_deref()),
        StreamKind::Tables => filtered_study_expression("tables", request.filter.as_deref()),
        StreamKind::All => Ok(all_panes_expression()),
    }
}

fn quote_expression() -> String {
    format!(
        r#"
        (function() {{
            var chart = {CHART_API};
            var bars = chart._chartWidget.model().mainSeries().bars();
            var last = bars.lastIndex();
            var v = bars.valueAt(last);
            if (!v) return null;
            return {{
                symbol: chart.symbol(),
                time: v[0],
                open: v[1],
                high: v[2],
                low: v[3],
                close: v[4],
                volume: v[5] || 0
            }};
        }})()
        "#
    )
}

fn bars_expression() -> String {
    format!(
        r#"
        (function() {{
            var chart = {CHART_API};
            var model = chart._chartWidget.model();
            var bars = model.mainSeries().bars();
            var last = bars.lastIndex();
            var v = bars.valueAt(last);
            if (!v) return null;
            return {{
                symbol: chart.symbol(),
                resolution: chart.resolution(),
                bar_time: v[0],
                open: v[1],
                high: v[2],
                low: v[3],
                close: v[4],
                volume: v[5] || 0,
                bar_index: last
            }};
        }})()
        "#
    )
}

fn values_expression() -> String {
    format!(
        r#"
        (function() {{
            {identity_helper}
            var chart = {CHART_API};
            var studies = chart.getAllStudies();
            var results = [];
            for (var i = 0; i < studies.length; i++) {{
                try {{
                    var study = chart.getStudyById(studies[i].id);
                    if (!study || (typeof study.isVisible === 'function' && !study.isVisible())) continue;
                    var src = study._study || study;
                    var data = src._lastBarValues || src._data;
                    if (!data) continue;
                    var vals = {{}};
                    if (typeof data === 'object') {{
                        for (var k in data) {{
                            if (typeof data[k] === 'number' && !isNaN(data[k])) vals[k] = data[k];
                        }}
                    }}
                    if (Object.keys(vals).length > 0) {{
                        var meta = {{}};
                        try {{ if (src && typeof src.metaInfo === 'function') meta = src.metaInfo() || {{}}; }} catch(e) {{}}
                        var identity = {{ entity_id: null, short_name: null, study_kind: 'unknown', inputs: null, visible: null }};
                        try {{ identity = tvStudyValueIdentity(src, study, meta, studies[i].id); }} catch(e) {{}}
                        results.push(Object.assign({{ name: studies[i].name, values: vals }}, identity));
                    }}
                }} catch(e) {{}}
            }}
            return {{ symbol: chart.symbol(), study_count: results.length, studies: results }};
        }})()
        "#,
        identity_helper = identity_helper_js(),
    )
}

fn filtered_study_expression(kind: &str, filter: Option<&str>) -> Result<String, AppError> {
    let filter = js_string(filter.unwrap_or(""))?;
    let body = match kind {
        "lines" => {
            r#"
            var pc = graphics._primitivesCollection;
            if (!pc || !pc.dwglines) continue;
            var linesMap = pc.dwglines.get('lines');
            if (!linesMap) continue;
            var data = linesMap.get(false);
            if (!data || !data._primitivesDataById) continue;
            var levels = [];
            var seen = {};
            data._primitivesDataById.forEach(function(line) {
                var p1 = line.points && line.points[0] ? line.points[0].price : null;
                var p2 = line.points && line.points[1] ? line.points[1].price : null;
                var price = (p1 !== null && p1 === p2) ? p1 : (p1 || p2);
                if (price !== null && !seen[price]) { seen[price] = true; levels.push(price); }
            });
            levels.sort(function(a, b) { return b - a; });
            if (levels.length > 0) results.push({ study: studyInfo.name, levels: levels });
            "#
        }
        "labels" => {
            r#"
            var pc = graphics._primitivesCollection;
            if (!pc || !pc.dwglabels) continue;
            var labelsMap = pc.dwglabels.get('labels');
            if (!labelsMap) continue;
            var data = labelsMap.get(false);
            if (!data || !data._primitivesDataById) continue;
            var labels = [];
            data._primitivesDataById.forEach(function(label) {
                var text = label.text || label.t || '';
                var price = label.points && label.points[0] ? label.points[0].price : (label.y || null);
                if (text) labels.push({ text: text, price: price });
            });
            if (labels.length > 0) results.push({ study: studyInfo.name, labels: labels.slice(0, 50) });
            "#
        }
        "tables" => {
            r#"
            var pc = graphics._primitivesCollection;
            if (!pc) continue;
            var tableMap = null;
            if (pc.ownFirstValue && typeof pc.ownFirstValue === 'function') tableMap = pc.ownFirstValue();
            if (!tableMap && pc.dwgtablecells) tableMap = pc.dwgtablecells;
            if (!tableMap) continue;
            var tables = [];
            if (typeof tableMap.forEach === 'function') {
                tableMap.forEach(function(table) {
                    if (!table || !table.data) return;
                    var rows = [];
                    for (var r = 0; r < table.data.length; r++) {
                        var row = [];
                        for (var c = 0; c < table.data[r].length; c++) row.push(table.data[r][c].text || '');
                        rows.push(row);
                    }
                    tables.push({ rows: rows });
                });
            }
            if (tables.length > 0) results.push({ study: studyInfo.name, tables: tables });
            "#
        }
        _ => unreachable!("stream kind should be validated"),
    };

    Ok(format!(
        r#"
        (function() {{
            var filter = {filter}.toLowerCase();
            var chart = {CHART_API};
            var studies = chart.getAllStudies();
            var results = [];
            for (var i = 0; i < studies.length; i++) {{
                var studyInfo = studies[i] || {{}};
                var name = studyInfo.name || "";
                if (filter && name.toLowerCase().indexOf(filter) === -1) continue;
                try {{
                    var study = chart.getStudyById(studyInfo.id);
                    if (!study) continue;
                    var src = study._study || study;
                    var graphics = src._graphics || (src._source && src._source._graphics);
                    if (!graphics) continue;
                    {body}
                }} catch(e) {{}}
            }}
            return {{ symbol: chart.symbol(), study_count: results.length, studies: results }};
        }})()
        "#
    ))
}

fn all_panes_expression() -> String {
    format!(
        r#"
        (function() {{
            var cwc = {CHART_WIDGET_COLLECTION};
            var all = cwc.getAll();
            var layoutType = cwc._layoutType;
            if (typeof layoutType === 'object' && layoutType && typeof layoutType.value === 'function') layoutType = layoutType.value();
            var count = cwc.inlineChartsCount;
            if (typeof count === 'object' && count && typeof count.value === 'function') count = count.value();
            var panes = [];
            for (var i = 0; i < Math.min(all.length, count || all.length); i++) {{
                try {{
                    var chart = all[i];
                    var model = chart.model();
                    var mainSeries = model.mainSeries();
                    var bars = mainSeries.bars();
                    var last = bars.lastIndex();
                    var v = bars.valueAt(last);
                    if (!v) {{
                        panes.push({{ index: i, symbol: mainSeries.symbol(), error: 'no bars' }});
                        continue;
                    }}
                    panes.push({{
                        index: i,
                        symbol: mainSeries.symbol(),
                        resolution: mainSeries.interval(),
                        time: v[0],
                        open: v[1],
                        high: v[2],
                        low: v[3],
                        close: v[4],
                        volume: v[5] || 0
                    }});
                }} catch(e) {{
                    panes.push({{ index: i, error: e.message }});
                }}
            }}
            return {{ layout: layoutType, pane_count: panes.length, panes: panes }};
        }})()
        "#
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ops::test_support::FakeRuntime;

    #[test]
    fn stream_request_uses_defaults_and_trims_filter() {
        let request =
            StreamRequest::new(StreamKind::Lines, None, Some("  Profiler  ".to_string())).unwrap();

        assert_eq!(request.interval_ms, 1000);
        assert_eq!(request.filter.as_deref(), Some("Profiler"));
        assert_eq!(request.duration_ms, None);
        assert_eq!(request.max_events, None);
        assert_eq!(request.heartbeat_ms, None);
    }

    #[test]
    fn stream_interval_rejects_too_small_values() {
        let error = StreamRequest::new(StreamKind::Quote, Some(99), None).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["minimum_interval_ms"], 100);
    }

    #[test]
    fn stream_controls_reject_zero_values_before_connecting() {
        let error =
            StreamRequest::with_controls(StreamKind::Quote, None, None, Some(0), None, None)
                .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["field"], "duration_ms");

        let error =
            StreamRequest::with_controls(StreamKind::Quote, None, None, None, Some(0), None)
                .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["field"], "max_events");

        let error =
            StreamRequest::with_controls(StreamKind::Quote, None, None, None, None, Some(0))
                .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["field"], "heartbeat_ms");
    }

    #[test]
    fn stream_dedupe_emits_first_and_changed_samples_only_ignoring_metadata() {
        let mut dedupe = StreamDedupe::default();
        let first = json!({"symbol": "NASDAQ:AAPL", "close": 1, "_ts": 1, "_event": "sample"});
        let same = json!({"symbol": "NASDAQ:AAPL", "close": 1, "_ts": 2, "_event": "sample"});
        let changed = json!({"symbol": "NASDAQ:AAPL", "close": 2, "_ts": 3, "_event": "sample"});

        assert!(dedupe.should_emit(&first));
        assert!(!dedupe.should_emit(&same));
        assert!(dedupe.should_emit(&changed));
    }

    #[test]
    fn stream_dedupe_treats_study_identity_changes_as_evidence() {
        let mut dedupe = StreamDedupe::default();
        let first = json!({
            "studies": [{"name": "EMA", "values": {"MA": 1}, "entity_id": "first", "inputs": {"length": 9}, "visible": true}],
            "_ts": 1,
            "_event": "sample"
        });
        let timestamp_only = json!({
            "studies": [{"name": "EMA", "values": {"MA": 1}, "entity_id": "first", "inputs": {"length": 9}, "visible": true}],
            "_ts": 2,
            "_event": "sample"
        });
        let changed_input = json!({
            "studies": [{"name": "EMA", "values": {"MA": 1}, "entity_id": "first", "inputs": {"length": 20}, "visible": true}],
            "_ts": 3,
            "_event": "sample"
        });
        let changed_entity = json!({
            "studies": [{"name": "EMA", "values": {"MA": 1}, "entity_id": "second", "inputs": {"length": 20}, "visible": true}],
            "_ts": 4,
            "_event": "sample"
        });
        let changed_visibility = json!({
            "studies": [{"name": "EMA", "values": {"MA": 1}, "entity_id": "second", "inputs": {"length": 20}, "visible": false}],
            "_ts": 5,
            "_event": "sample"
        });

        assert!(dedupe.should_emit(&first));
        assert!(!dedupe.should_emit(&timestamp_only));
        assert!(dedupe.should_emit(&changed_input));
        assert!(dedupe.should_emit(&changed_entity));
        assert!(dedupe.should_emit(&changed_visibility));
    }

    #[tokio::test]
    async fn stream_sample_adds_metadata_and_uses_quote_expression() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "time": 1,
            "open": 2,
            "high": 3,
            "low": 1,
            "close": 2.5,
            "volume": 100,
        })]);
        let request = StreamRequest::new(StreamKind::Quote, None, None).unwrap();

        let sample = stream_sample(&mut runtime, &request).await.unwrap();

        assert_eq!(sample["symbol"], "NASDAQ:AAPL");
        assert_eq!(sample["_stream"], "quote");
        assert_eq!(sample["_event"], "sample");
        assert_eq!(sample["contract_version"], "stream.v1");
        assert_eq!(sample["source"], "desktop_chart_stream");
        assert_eq!(sample["source_category"], "desktop_backed_read");
        assert_eq!(sample["requires_desktop"], true);
        assert_eq!(sample["non_mutating"], true);
        assert!(sample["_ts"].as_u64().unwrap() > 0);
        assert!(runtime.evaluated[0].0.contains("chart.symbol()"));
    }

    #[tokio::test]
    async fn stream_values_adds_shared_identity_contract() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "study_count": 2,
            "studies": [
                {"name": "EMA", "values": {"MA": 1}, "entity_id": "first", "short_name": "EMA", "study_kind": "indicator", "inputs": {"length": 9}, "visible": true},
                {"name": "EMA", "values": {"MA": 2}, "entity_id": "second", "short_name": "EMA", "study_kind": "indicator", "inputs": {"length": 20}, "visible": true}
            ]
        })]);
        let request = StreamRequest::new(StreamKind::Values, None, None).unwrap();

        let sample = stream_sample(&mut runtime, &request).await.unwrap();

        assert_eq!(sample["study_count"], 2);
        assert_eq!(sample["studies"][0]["name"], "EMA");
        assert_eq!(sample["studies"][0]["values"], json!({"MA": 1}));
        assert_eq!(sample["studies"][0]["entity_id"], "first");
        assert_eq!(sample["studies"][0]["inputs"]["length"], 9);
        assert_eq!(sample["studies"][1]["entity_id"], "second");
        assert_eq!(sample["studies"][1]["inputs"]["length"], 20);
        assert_eq!(sample["_stream"], "values");
        assert!(runtime.evaluated[0].0.contains("tvStudyValueIdentity"));
        assert!(runtime.evaluated[0].0.contains("_lastBarValues"));
        assert!(runtime.evaluated[0].0.contains("!study.isVisible()"));
    }

    #[test]
    fn stream_heartbeat_returns_observation_metadata() {
        let request =
            StreamRequest::with_controls(StreamKind::Quote, None, None, None, None, Some(1000))
                .unwrap();

        let heartbeat = stream_heartbeat(&request, 2500, 3, Some(123)).unwrap();

        assert_eq!(heartbeat["_stream"], "quote");
        assert_eq!(heartbeat["_event"], "heartbeat");
        assert_eq!(heartbeat["contract_version"], "stream.v1");
        assert_eq!(heartbeat["source"], "desktop_chart_stream");
        assert_eq!(heartbeat["source_category"], "desktop_backed_read");
        assert_eq!(heartbeat["requires_desktop"], true);
        assert_eq!(heartbeat["non_mutating"], true);
        assert_eq!(heartbeat["elapsed_ms"], 2500);
        assert_eq!(heartbeat["sample_count"], 3);
        assert_eq!(heartbeat["last_sample_ts"], 123);
        assert!(heartbeat["_ts"].as_u64().unwrap() > 0);
    }

    #[test]
    fn stream_summary_returns_final_observation_metadata() {
        let request = StreamRequest::with_controls(
            StreamKind::Bars,
            Some(500),
            None,
            Some(10_000),
            Some(5),
            Some(1000),
        )
        .unwrap();

        let summary = stream_summary(
            &request,
            10_001,
            5,
            2,
            Some(456),
            StreamEndReason::MaxEventsReached,
        )
        .unwrap();

        assert_eq!(summary["_stream"], "bars");
        assert_eq!(summary["_event"], "summary");
        assert_eq!(summary["contract_version"], "stream.v1");
        assert_eq!(summary["source"], "desktop_chart_stream");
        assert_eq!(summary["source_category"], "desktop_backed_read");
        assert_eq!(summary["requires_desktop"], true);
        assert_eq!(summary["non_mutating"], true);
        assert_eq!(summary["elapsed_ms"], 10_001);
        assert_eq!(summary["sample_count"], 5);
        assert_eq!(summary["heartbeat_count"], 2);
        assert_eq!(summary["last_sample_ts"], 456);
        assert_eq!(summary["duration_ms"], 10_000);
        assert_eq!(summary["max_events"], 5);
        assert_eq!(summary["end_reason"], "max_events_reached");
        assert!(summary["_ts"].as_u64().unwrap() > 0);
    }

    #[test]
    fn stream_summary_reports_duration_elapsed_and_no_samples() {
        let request =
            StreamRequest::with_controls(StreamKind::Quote, None, None, Some(1000), None, None)
                .unwrap();

        let summary =
            stream_summary(&request, 1000, 0, 0, None, StreamEndReason::DurationElapsed).unwrap();

        assert_eq!(summary["_event"], "summary");
        assert_eq!(summary["sample_count"], 0);
        assert_eq!(summary["heartbeat_count"], 0);
        assert!(summary["last_sample_ts"].is_null());
        assert_eq!(summary["duration_ms"], 1000);
        assert!(summary["max_events"].is_null());
        assert_eq!(summary["end_reason"], "duration_elapsed");
    }

    #[tokio::test]
    async fn stream_sample_serializes_filter_as_js_string() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "study_count": 0,
            "studies": [],
        })]);
        let request = StreamRequest::new(
            StreamKind::Labels,
            Some(1000),
            Some("x'; alert(1); //".to_string()),
        )
        .unwrap();

        let sample = stream_sample(&mut runtime, &request).await.unwrap();

        assert_eq!(sample["_stream"], "labels");
        assert!(runtime.evaluated[0].0.contains(r#""x'; alert(1); //""#));
    }

    #[tokio::test]
    async fn stream_sample_rejects_non_object_payload() {
        let mut runtime = FakeRuntime::new([json!(["not", "object"])]);
        let request = StreamRequest::new(StreamKind::Bars, None, None).unwrap();

        let error = stream_sample(&mut runtime, &request).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Stream sample was not an object");
    }
}
