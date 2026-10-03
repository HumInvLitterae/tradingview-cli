use std::{
    io::Write,
    process::{Command, Stdio},
};

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorEnvelope, ErrorKind, SuccessEnvelope};

use super::describe;

#[test]
fn catalog_resolves_public_paths_and_distinguishes_unavailable_schemas() {
    let catalog = describe(&[]).unwrap();
    for path in catalog["commands"].as_array().unwrap() {
        let path: Vec<String> = serde_json::from_value(path.clone()).unwrap();
        let detail = describe(&path).unwrap();
        assert_eq!(detail["command"], json!(path));
        assert!(detail["schema"]["oneOf"].is_array());
        assert_eq!(detail["coverage"], "documented_fields");
    }
    for (path, code) in [
        (vec!["data", "equity"], "unsupported_command"),
        (vec!["missing-command"], "unknown_command"),
        (vec!["help"], "unknown_command"),
    ] {
        let path = path.into_iter().map(str::to_owned).collect::<Vec<_>>();
        assert_eq!(describe(&path).unwrap_err().details.unwrap()["code"], code);
    }
}

fn fixture_case(path: &[&str], samples: Vec<Value>, invalid_pointers: &[&str]) -> Value {
    let path = path
        .iter()
        .map(|part| (*part).to_owned())
        .collect::<Vec<_>>();
    let mut invalid = Vec::new();
    let mut valid = samples;
    let mut extra = valid[0].clone();
    extra["additional_field"] = json!(true);
    extra["data"]["additional_field"] = json!({});
    valid.push(extra);
    for pointer in invalid_pointers {
        let mut wrong_type = valid[0].clone();
        let field = wrong_type.pointer_mut(pointer).unwrap();
        *field = if field.is_string() {
            json!(false)
        } else {
            json!("wrong type")
        };
        invalid.push(wrong_type);
        let (parent, name) = pointer.rsplit_once('/').unwrap();
        let mut missing = valid[0].clone();
        missing
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(name);
        invalid.push(missing);
    }
    for kind in [
        ErrorKind::Validation,
        ErrorKind::Connection,
        ErrorKind::InternalApiUnavailable,
        ErrorKind::Timeout,
        ErrorKind::TargetAmbiguous,
        ErrorKind::Internal,
    ] {
        let target_command = match path[0].as_str() {
            "mcp" => "mcp",
            "values" => "values",
            "ohlcv" => "ohlcv",
            "data" => "data",
            _ => unreachable!("fixture command needs a static envelope name"),
        };
        for command in [target_command, "tv"] {
            let error = AppError::new(kind, "Synthetic error");
            valid.push(serde_json::to_value(ErrorEnvelope::new(command, error.into())).unwrap());
        }
    }
    json!({"schema": describe(&path).unwrap()["schema"], "valid": valid, "invalid": invalid})
}

#[tokio::test]
#[ignore = "run scripts/check-output-schemas.py with pinned python-jsonschema"]
async fn output_schemas_match_production_fixtures() {
    let mut runtime = crate::ops::test_support::FakeRuntime::new([json!({
        "study_count": 2,
        "studies": [
            {"name": "Same name", "values": {"Value": "0"}, "visible": false,
             "inputs": {"length": 20, "enabled": true, "choice": [null, "test"]}},
            {"name": "Same name", "values": {"Unknown": {"shape": "unconfirmed"}},
             "study_kind": "invalid", "entity_id": {"invalid": true}}
        ]
    })]);
    let values = crate::ops::study_values(&mut runtime).await.unwrap();
    let values = serde_json::to_value(SuccessEnvelope::new("values", values)).unwrap();
    let mut empty_runtime =
        crate::ops::test_support::FakeRuntime::new([json!({"study_count": 0, "studies": []})]);
    let empty = crate::ops::study_values(&mut empty_runtime).await.unwrap();
    let values_case = fixture_case(
        &["values"],
        vec![
            values,
            serde_json::to_value(SuccessEnvelope::new("values", empty)).unwrap(),
        ],
        &[
            "/data/study_count",
            "/data/studies",
            "/data/studies/0/name",
            "/data/studies/0/study_kind",
            "/data/studies/0/visible",
            "/data/studies/0/inputs",
        ],
    );

    let request = tradingview_model::mcp_bars::Request::new("NASDAQ:EXAMPLE", "1D", 2).unwrap();
    let mut samples = Vec::new();
    for (index, rows) in [
        json!([{ "t": 1, "o": 1, "h": 2, "l": 0, "c": 1, "v": null },
               { "t": 2, "o": 1, "h": 2, "l": 0, "c": 1, "v": 0 }]),
        json!([{ "t": 1, "o": 1, "h": 2, "l": 0, "c": 1 }]),
        json!([]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut response = json!({"bars": rows});
        if index == 0 {
            response["symbol"] = json!("NASDAQ:EXAMPLE");
            response["interval"] = json!("1D");
        }
        let data = tradingview_model::mcp_bars::normalize(&request, response, 0).unwrap();
        samples.push(serde_json::to_value(SuccessEnvelope::new("mcp", data)).unwrap());
    }
    let error = tradingview_model::mcp_bars::normalize(
        &request,
        json!({"success": false, "error": "synthetic 429"}),
        0,
    )
    .unwrap_err();
    samples.push(serde_json::to_value(ErrorEnvelope::new("mcp", error.into())).unwrap());
    let bars_case = fixture_case(
        &["mcp", "bars"],
        samples,
        &[
            "/data/bars",
            "/data/request/count",
            "/data/bars/0/volume",
            "/data/client_observation/bar_count",
            "/data/requires_desktop",
        ],
    );
    let mut probe = crate::ops::test_support::FakeRuntime::new([json!({})]);
    crate::ops::ohlcv_bars(&mut probe, Some(2)).await.unwrap();
    let expression = serde_json::to_string(&probe.evaluated[0].0).unwrap();
    let script = format!(
        r#"
        const expression = {expression};
        let rows = [];
        const bars = {{
            firstIndex: () => 0,
            lastIndex: () => rows.length - 1,
            size: () => rows.length,
            valueAt: i => rows[i]
        }};
        const chart = {{
            symbol: () => 'NASDAQ:EXAMPLE',
            resolution: () => 'D',
            getVisibleRange: () => ({{ from: 1, to: 2 }}),
            getVisibleBarsRange: () => ({{ from: 0, to: 1 }}),
            _chartWidget: {{
                model: () => ({{ mainSeries: () => ({{ bars: () => bars }}) }})
            }}
        }};
        global.window = {{
            TradingViewApi: {{ _activeChartWidgetWV: {{ value: () => chart }} }}
        }};
        function read(input) {{
            rows = input;
            return eval(expression);
        }}
        console.log(JSON.stringify([
            read([[1, 1, 3, 0, 2, 10], [2, 2, 4, 1, 3, 0]]),
            read([[1, undefined, 3, 0, 2, 10], [2, 2, NaN, 1, 3]]),
            read([[1, 0, 3, 0, 2, 0]]),
            read([[1, Infinity, 3, 0, 2, '0']]),
            read([new Float64Array([1, 1, 3, 0, 2, 0])]),
            read([[NaN, 1, 3, 0, 2, 10]]),
            read([{{ time: 1 }}]),
            read([])
        ]));
        "#
    );
    let output = Command::new("node").args(["-e", &script]).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let raw: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(raw[0]["bars"][1]["volume"], 0);
    assert!(raw[1]["bars"][0]["open"].is_null());
    assert!(raw[1]["bars"][1]["high"].is_null());
    assert!(raw[1]["bars"][1]["volume"].is_null());
    assert!(raw[3]["bars"][0]["open"].is_null());
    assert!(raw[3]["bars"][0]["volume"].is_null());
    assert_eq!(raw[4]["bars"][0]["volume"], 0);
    for malformed in &raw[5..] {
        let mut runtime = crate::ops::test_support::FakeRuntime::new([malformed.clone()]);
        assert!(crate::ops::ohlcv_bars(&mut runtime, Some(2)).await.is_err());
    }
    let mut ohlcv_samples = Vec::new();
    for (index, data) in raw[..5].iter().enumerate() {
        let mut runtime = crate::ops::test_support::FakeRuntime::new([data.clone()]);
        let summary = crate::ops::ohlcv_summary(&mut runtime, Some(2))
            .await
            .unwrap();
        if index == 0 {
            assert_eq!(summary["volume"], 10.0);
            assert_eq!(summary["avg_volume"], 5.0);
        } else if index == 1 {
            for field in [
                "open",
                "high",
                "range",
                "volume",
                "avg_volume",
                "change",
                "change_pct",
            ] {
                assert!(summary[field].is_null(), "{field}");
            }
            assert_eq!(summary["close"], 3.0);
            assert_eq!(summary["low"], 0.0);
        } else if index == 2 {
            assert_eq!(summary["open"], 0.0);
            assert_eq!(summary["volume"], 0.0);
            assert_eq!(summary["change"], 2.0);
            assert!(summary["change_pct"].is_null());
        }
        ohlcv_samples
            .push(serde_json::to_value(SuccessEnvelope::new("ohlcv", data.clone())).unwrap());
        ohlcv_samples.push(serde_json::to_value(SuccessEnvelope::new("ohlcv", summary)).unwrap());
    }
    let summary_case = fixture_case(
        &["ohlcv"],
        ohlcv_samples[1..].iter().step_by(2).cloned().collect(),
        &[
            "/data/open",
            "/data/volume",
            "/data/change_pct",
            "/data/period/from",
            "/data/last_5_bars/0/volume",
        ],
    );
    let raw_case = fixture_case(
        &["ohlcv"],
        ohlcv_samples.iter().step_by(2).cloned().collect(),
        &[
            "/data/bars/0/time",
            "/data/bars/0/volume",
            "/data/bar_count",
            "/data/source",
        ],
    );
    let graphics_raw = json!([{
        "name": "Example", "count": 3,
        "items": [
            {"id": "synthetic-1", "raw": {"y1": 10.001, "y2": 10.002, "x1": 1, "x2": 2}},
            {"id": 2, "raw": {"y1": 0, "y2": 0, "x1": null}},
            {"id": null, "raw": {"y1": null, "y2": 20, "x1": "unknown"}}
        ]
    }]);
    let mut graphics_cases = Vec::new();
    for kind in ["lines", "boxes"] {
        let mut samples = Vec::new();
        for raw in [
            graphics_raw.clone(),
            json!([]),
            json!({"unavailable": true}),
        ] {
            for verbose in [true, false] {
                let mut runtime = crate::ops::test_support::FakeRuntime::new([raw.clone()]);
                let output = if kind == "lines" {
                    crate::ops::data_lines(&mut runtime, Some("Example"), verbose).await
                } else {
                    crate::ops::data_boxes(&mut runtime, Some("Example"), verbose).await
                }
                .unwrap();
                if !raw.is_array() || raw.as_array().unwrap().is_empty() {
                    assert_eq!(output["study_count"], 0);
                } else if kind == "lines" {
                    assert_eq!(
                        output["studies"][0]["horizontal_levels"],
                        json!([10.0, 0.0])
                    );
                    if verbose {
                        assert!(output["studies"][0]["all_lines"][2]["y1"].is_null());
                        assert_eq!(output["studies"][0]["all_lines"][2]["x1"], "unknown");
                    }
                } else {
                    assert_eq!(
                        output["studies"][0]["zones"],
                        json!([
                            {"high": 10.0, "low": 10.0}, {"high": 0.0, "low": 0.0}
                        ])
                    );
                }
                samples.push(serde_json::to_value(SuccessEnvelope::new("data", output)).unwrap());
            }
        }
        let field = if kind == "lines" {
            "/data/studies/0/horizontal_levels"
        } else {
            "/data/studies/0/zones"
        };
        let verbose_field = if kind == "lines" {
            "/data/studies/0/all_lines/0/horizontal"
        } else {
            "/data/studies/0/all_boxes/0/high"
        };
        let count_field = format!("/data/studies/0/total_{kind}");
        let case = fixture_case(
            &["data", kind],
            samples,
            &["/data/study_count", &count_field, field, verbose_field],
        );
        graphics_cases.push(case);
    }

    let labels_raw = json!([{
        "name": "Example", "count": 3,
        "items": [
            {"id": false, "raw": {"t": "", "y": 0, "x": "unknown", "yl": {"mode": "unknown"}}},
            {"id": "note", "raw": {"t": "Text only", "y": null}},
            {"id": null, "raw": {"t": "", "y": null}}
        ]
    }]);
    let mut label_samples = Vec::new();
    for (raw, limit) in [
        (labels_raw.clone(), None),
        (labels_raw.clone(), Some(0)),
        (labels_raw, Some(1)),
        (json!([]), None),
        (json!({"unavailable": true}), None),
        (json!([{"items": []}]), None),
    ] {
        for verbose in [true, false] {
            let mut runtime = crate::ops::test_support::FakeRuntime::new([raw.clone()]);
            let output = crate::ops::data_labels(&mut runtime, Some("Example"), limit, verbose)
                .await
                .unwrap();
            if raw[0]["name"] == "Example" {
                let study = &output["studies"][0];
                assert_eq!(study["total_labels"], 3);
                assert_eq!(study["available_labels"], 2);
                assert_eq!(study["limit"], limit.unwrap_or(500));
                if limit == Some(0) {
                    assert_eq!(study["showing"], 0);
                    assert_eq!(study["truncated"], true);
                    assert_eq!(study["labels"], json!([]));
                } else if limit == Some(1) {
                    assert_eq!(study["showing"], 1);
                    assert_eq!(study["labels"][0]["text"], "Text only");
                    assert!(study["labels"][0]["price"].is_null());
                } else {
                    assert_eq!(study["showing"], 2);
                    assert_eq!(study["truncated"], false);
                    assert_eq!(study["labels"][0]["price"], 0.0);
                    assert!(study["labels"][1]["price"].is_null());
                    if verbose {
                        assert_eq!(study["labels"][0]["id"], false);
                        assert_eq!(study["labels"][0]["x"], "unknown");
                        assert_eq!(study["labels"][0]["yloc"], json!({"mode": "unknown"}));
                    } else {
                        assert!(study["labels"][0].get("id").is_none());
                    }
                }
            } else if !raw.is_array() || raw.as_array().unwrap().is_empty() {
                assert_eq!(output["study_count"], 0);
            } else {
                assert!(output["studies"][0]["name"].is_null());
                assert!(output["studies"][0]["total_labels"].is_null());
            }
            label_samples.push(serde_json::to_value(SuccessEnvelope::new("data", output)).unwrap());
        }
    }
    graphics_cases.push(fixture_case(
        &["data", "labels"],
        label_samples,
        &[
            "/data/study_count",
            "/data/studies/0/total_labels",
            "/data/studies/0/available_labels",
            "/data/studies/0/limit",
            "/data/studies/0/showing",
            "/data/studies/0/truncated",
            "/data/studies/0/labels/0/text",
            "/data/studies/0/labels/0/price",
        ],
    ));

    let tables_raw = json!([{
        "name": "Example", "count": 5,
        "items": [
            {"raw": {"tid": 1, "row": 0, "col": 1, "t": "Risk | note"}},
            {"raw": {"tid": 1, "row": 0, "col": 0, "t": "old"}},
            {"raw": {"tid": 1, "row": 0, "col": 0, "t": "replacement"}},
            {"raw": {"tid": 1, "row": 1, "col": 0, "t": ""}},
            {"raw": {"t": "0"}}
        ]
    }]);
    let mut table_samples = Vec::new();
    for raw in [
        tables_raw,
        json!([]),
        json!({"unavailable": true}),
        json!([{"items": []}]),
    ] {
        let mut runtime = crate::ops::test_support::FakeRuntime::new([raw.clone()]);
        let output = crate::ops::data_tables(&mut runtime, Some("Example"))
            .await
            .unwrap();
        if raw[0]["name"] == "Example" {
            assert_eq!(
                output["studies"][0]["tables"],
                json!([{"rows": ["0"]}, {"rows": ["replacement | Risk | note"]}])
            );
        } else if !raw.is_array() || raw.as_array().unwrap().is_empty() {
            assert_eq!(output["study_count"], 0);
        } else {
            assert!(output["studies"][0]["name"].is_null());
            assert_eq!(output["studies"][0]["tables"], json!([]));
        }
        table_samples.push(serde_json::to_value(SuccessEnvelope::new("data", output)).unwrap());
    }
    let mut tables_case = fixture_case(
        &["data", "tables"],
        table_samples,
        &["/data/study_count", "/data/studies/0/tables/0/rows"],
    );
    let mut numeric_row = tables_case["valid"][0].clone();
    numeric_row["data"]["studies"][0]["tables"][0]["rows"][0] = json!(0);
    tables_case["invalid"]
        .as_array_mut()
        .unwrap()
        .push(numeric_row);
    graphics_cases.push(tables_case);

    let python =
        std::env::var_os("TV_SCHEMA_TEST_PYTHON").expect("schema gate sets Python interpreter");
    let mut child = Command::new(python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scripts/check-output-schemas.py"
        ))
        .arg("--fixtures")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut cases = vec![values_case, bars_case, raw_case, summary_case];
    cases.extend(graphics_cases);
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&cases).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
