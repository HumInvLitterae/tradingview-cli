use super::*;

const NO_INPUTS: &str = "indicator(\"Signals\")\nalertcondition(close > open, \"Long\")";
const WITH_INPUTS: &str =
    "indicator(\"Signals\")\nlength=input.int(20)\nalertcondition(close > open, \"Long\")";
const DYNAMIC_TITLE: &str = "indicator(\"Signals\")\nconditionTitle = \"Long\"\nalertcondition(close > open, conditionTitle)";

#[tokio::test]
async fn legacy_public_request_and_preview_remain_compatible() {
    let request = crate::ops::IndicatorAlertRequest {
        script: "Signals",
        source: NO_INPUTS,
        input_source: "stdin",
        condition_title: Some("Long"),
        alert_cond_id: None,
        symbol: None,
        resolution: None,
        message: None,
        dry_run: true,
    };
    let mut runtime = FakeRuntime::new([saved_script_fixture(), json!({"source": NO_INPUTS})]);
    let result = crate::ops::alert_create_indicator(&mut runtime, request)
        .await
        .unwrap();
    assert_eq!(runtime.evaluated.len(), 2);
    assert!(result.get("verification").is_none());
    assert_eq!(
        result["request"],
        json!({"symbol":null,"resolution":null,"message":null})
    );
    assert_eq!(result["condition"]["alert_cond_id"], "plot_0");
    assert_eq!(result["source"], "indicator_alert_dry_run");
}

#[tokio::test]
async fn verified_indicator_rejects_blank_entity_before_reading() {
    let mut runtime = FakeRuntime::new([]);
    let error = crate::ops::alert_create_indicator_verified(
        &mut runtime,
        source_check_request(NO_INPUTS, true),
        crate::ops::IndicatorStudySelection::Entity("  "),
    )
    .await
    .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(runtime.evaluated.is_empty());
}

#[tokio::test]
async fn verified_indicator_source_mismatch_prevents_compilation_and_creation() {
    for dry_run in [true, false] {
        let mut runtime =
            FakeRuntime::new([saved_script_fixture(), json!({"source": "different"})]);
        let error = alert_create_indicator_verified(
            &mut runtime,
            source_check_request(NO_INPUTS, dry_run),
            IndicatorStudySelection::Automatic,
        )
        .await
        .unwrap_err();
        assert_eq!(error.details.unwrap()["reason"], "source_mismatch");
        assert_eq!(runtime.evaluated.len(), 2);
        assert!(runtime.evaluated.iter().all(|(expression, _)| {
            !expression.contains("pine-facade/translate/")
                && !expression.contains("pricealerts.tradingview.com")
        }));
    }
}

#[tokio::test]
async fn verified_indicator_lost_evaluation_distinguishes_preview_from_creation() {
    for dry_run in [true, false] {
        for kind in [
            ErrorKind::Timeout,
            ErrorKind::Connection,
            ErrorKind::InternalApiUnavailable,
        ] {
            let mut runtime =
                FakeRuntime::new([saved_script_fixture(), json!({"source": NO_INPUTS})])
                    .with_evaluate_app_error_after_responses(
                        AppError::new(kind, "evaluation unavailable")
                            .with_details(json!({"failure_stage": "method_call"})),
                    );
            let error = alert_create_indicator_verified(
                &mut runtime,
                source_check_request(NO_INPUTS, dry_run),
                IndicatorStudySelection::Automatic,
            )
            .await
            .unwrap_err();
            assert_eq!(error.kind, kind);
            let details = error.details.unwrap();
            assert_eq!(details["failure_stage"], "method_call");
            if dry_run {
                assert_eq!(details["created"], false);
                assert!(details.get("creation_outcome").is_none());
            } else {
                assert_eq!(details.get("created"), Some(&Value::Null));
                assert_eq!(details["creation_outcome"], "unknown");
            }
        }
    }
}

#[tokio::test]
async fn verified_indicator_rejects_incomplete_or_inconsistent_runtime_results() {
    let valid = json!({
        "condition_source": "saved_compilation", "input_source": "none",
        "input_count": 0, "study": null
    });
    let mut invalid_summaries = vec![Value::Null, json!({})];
    for (key, value) in [
        ("condition_source", json!("local_parser")),
        ("input_source", json!("defaults")),
        ("input_count", json!(1)),
        ("input_count", json!("0")),
        ("study", json!({"entity_id":"chosen","selection":"unique"})),
        ("private_payload", json!("must-not-escape")),
    ] {
        let mut summary = valid.clone();
        summary[key] = value;
        invalid_summaries.push(summary);
    }
    for dry_run in [true, false] {
        for summary in &invalid_summaries {
            let response = json!({
                "created": !dry_run, "action": if dry_run {"dry_run"} else {"create_indicator"},
                "symbol": "EXCHANGE:EXAMPLE", "resolution": "1D", "verification": summary
            });
            let mut runtime = FakeRuntime::new([
                saved_script_fixture(),
                json!({"source": NO_INPUTS}),
                response,
            ]);
            let error = alert_create_indicator_verified(
                &mut runtime,
                source_check_request(NO_INPUTS, dry_run),
                IndicatorStudySelection::Automatic,
            )
            .await
            .unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert!(!format!("{error:?}").contains("must-not-escape"));
            let details = error.details.unwrap();
            assert_eq!(
                details["created"],
                if dry_run { json!(false) } else { Value::Null }
            );
            assert_eq!(details.get("creation_outcome").is_some(), !dry_run);
        }
    }
}

#[tokio::test]
async fn verified_indicator_rejects_wrong_selected_entity_and_invalid_preview() {
    for dry_run in [true, false] {
        let valid = json!({
            "action": if dry_run {"dry_run"} else {"create_indicator"},
            "created": !dry_run, "symbol": "EXCHANGE:EXAMPLE", "resolution": "1D",
            "input_metadata": {"input_count":0,"study_matched":true},
            "verification": {
                "condition_source":"saved_compilation","input_source":"none","input_count":0,
                "study":{"entity_id":"chosen","selection":"explicit"}
            }
        });
        let mut cases = vec![
            ("/symbol", Value::Null),
            ("/resolution", json!("")),
            ("/created", json!(dry_run)),
            ("/verification/study/entity_id", json!("other")),
            ("/verification/study/selection", json!("unique")),
        ];
        if dry_run {
            cases.push(("/action", json!("create_indicator")));
        } else {
            cases.push(("/input_metadata/input_count", json!(1)));
            cases.push(("/input_metadata/study_matched", json!(false)));
        }
        for (path, value) in cases {
            let mut response = valid.clone();
            *response.pointer_mut(path).unwrap() = value;
            let mut runtime = FakeRuntime::new([
                saved_script_fixture(),
                json!({"source": NO_INPUTS}),
                response,
            ]);
            let error = alert_create_indicator_verified(
                &mut runtime,
                source_check_request(NO_INPUTS, dry_run),
                IndicatorStudySelection::Entity("chosen"),
            )
            .await
            .expect_err(path);
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(
                error.details.unwrap().get("creation_outcome").is_some(),
                !dry_run
            );
        }
    }
}

#[tokio::test]
#[ignore = "run through scripts/check-account-js-contract.py with pinned Node.js"]
async fn javascript_account_indicator_verification() {
    let mut expressions = serde_json::Map::new();
    for inputs in [false, true] {
        let source = if inputs { WITH_INPUTS } else { NO_INPUTS };
        for dry_run in [false, true] {
            for explicit in [false, true] {
                let selection = if explicit {
                    IndicatorStudySelection::Entity(" chosen ")
                } else {
                    IndicatorStudySelection::Automatic
                };
                let mut runtime =
                    FakeRuntime::new([saved_script_fixture(), json!({"source": source})]);
                let _ = alert_create_indicator_verified(
                    &mut runtime,
                    source_check_request(source, dry_run),
                    selection,
                )
                .await;
                expressions.insert(
                    format!("{inputs}:{dry_run}:{explicit}"),
                    json!(runtime.evaluated[2].0),
                );
            }
        }
    }
    for dry_run in [false, true] {
        let mut request = source_check_request(DYNAMIC_TITLE, dry_run);
        request.condition_title = None;
        request.alert_cond_id = Some("plot_0");
        let mut runtime =
            FakeRuntime::new([saved_script_fixture(), json!({"source": DYNAMIC_TITLE})]);
        let _ = alert_create_indicator_verified(
            &mut runtime,
            request,
            IndicatorStudySelection::Automatic,
        )
        .await;
        expressions.insert(format!("idOnly:{dry_run}"), json!(runtime.evaluated[2].0));
    }
    let script = format!(
        "const expressions = {};\n{}",
        Value::Object(expressions),
        include_str!("verification_contract.js")
    );
    let output = std::process::Command::new("node")
        .args(["-e", &script])
        .output()
        .expect("Node.js is required for the account JavaScript contract");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    println!(
        "{} verified indicator JavaScript scenarios passed",
        rows.len()
    );
    for row in rows {
        let source = if row["id_only"] == true {
            DYNAMIC_TITLE
        } else if row["inputs"] == true {
            WITH_INPUTS
        } else {
            NO_INPUTS
        };
        let dry_run = row["dry_run"].as_bool().unwrap();
        let selection = if row["explicit"] == true {
            IndicatorStudySelection::Entity(" chosen ")
        } else {
            IndicatorStudySelection::Automatic
        };
        let mut runtime = FakeRuntime::new([
            saved_script_fixture(),
            json!({"source": source}),
            row["result"].clone(),
        ]);
        let mut request = source_check_request(source, dry_run);
        if row["id_only"] == true {
            request.condition_title = None;
            request.alert_cond_id = Some("plot_0");
        }
        let result = alert_create_indicator_verified(&mut runtime, request, selection).await;
        if row["success"] == true {
            let data = result.unwrap_or_else(|error| panic!("{}: {error:?}", row["name"]));
            assert_eq!(data["verification"], row["result"]["verification"]);
            assert_eq!(data["dry_run"], dry_run);
            if dry_run {
                assert_eq!(data["request"]["symbol"], row["result"]["symbol"]);
                assert_eq!(data["request"]["resolution"], row["result"]["resolution"]);
            }
        } else {
            let error = result.expect_err("a failed JavaScript preflight cannot become a success");
            let details = error.details.unwrap();
            assert_eq!(details["phase"], row["result"]["phase"]);
            assert_eq!(details["created"], row["result"]["created"]);
            assert_eq!(
                details.get("creation_outcome"),
                row["result"].get("creation_outcome")
            );
        }
    }
}
