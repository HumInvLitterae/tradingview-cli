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
        let target_command = if path[0] == "mcp" { "mcp" } else { "values" };
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
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&json!([values_case, bars_case])).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
