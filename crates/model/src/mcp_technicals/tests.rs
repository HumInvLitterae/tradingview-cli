use super::*;

fn response() -> Value {
    json!({"success": true, "data": {
        "symbol": "NASDAQ:EXAMPLE", "interval": "1D",
        "oscillators": {"rsi": 52.5, "momentum": 0},
        "moving_averages": {"ema10": 100},
        "summary": {"recommendation": "BUY", "value": 0, "ma": "STRONG_BUY", "other": "SELL"},
        "private_extra": "secret"
    }})
}

#[test]
fn request_timeframes_are_independent_of_bars() {
    for timeframe in TIMEFRAMES {
        assert_eq!(
            Request::new("NASDAQ:EXAMPLE", timeframe)
                .unwrap()
                .arguments(),
            json!({"symbol": "NASDAQ:EXAMPLE", "interval": timeframe})
        );
    }
    assert!(Request::new("EXAMPLE", "1D").is_err());
    for timeframe in ["M", "1mo", "2D", "", "1d"] {
        assert!(Request::new("NASDAQ:EXAMPLE", timeframe).is_err());
    }
    assert!(crate::mcp_bars::Request::new("NASDAQ:EXAMPLE", "2h", 20).is_err());
}

#[test]
fn snapshot_preserves_summary_zero_missing_and_unknown_conditions() {
    let request = Request::new("NASDAQ:EXAMPLE", "1D").unwrap();
    let output = normalize(&request, response(), 1700000000000).unwrap();
    assert_eq!(output["status"], "available");
    assert_eq!(output["summary"], response()["data"]["summary"]);
    let items = output["indicators"].as_array().unwrap();
    assert_eq!(items.len(), 23);
    assert_eq!(
        items.iter().find(|i| i["name"] == "momentum").unwrap()["value"],
        0
    );
    assert!(items.iter().find(|i| i["name"] == "ema30").unwrap()["value"].is_null());
    assert_eq!(
        output["provider_observation"]["symbol"]["evidence"],
        "provider_response"
    );
    assert_eq!(
        output["provider_observation"]["data_as_of"],
        json!({"value": null, "evidence": "unconfirmed"})
    );
    assert_eq!(
        output["client_observation"]["received_at"],
        "2023-11-14T22:13:20.000Z"
    );
    assert!(!output.to_string().contains("secret"));
    assert!(output.get("rating_value").is_none());
}

#[test]
fn empty_is_not_zero_or_provider_failure() {
    let request = Request::new("NASDAQ:EXAMPLE", "1D").unwrap();
    let mut value = json!({"success": true, "data": {
        "oscillators": {"rsi": null}, "moving_averages": {"ema10": null},
        "summary": {"ma": null}
    }});
    let output = normalize(&request, value.clone(), 0).unwrap();
    assert_eq!(output["status"], "empty");
    assert!(
        output["indicators"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["value"].is_null())
    );
    assert!(output["provider_observation"]["symbol"]["value"].is_null());
    value["data"]["summary"]["value"] = json!(0);
    assert_eq!(
        normalize(&request, value, 0).unwrap()["status"],
        "available"
    );
    let failure = normalize(
        &request,
        json!({"success": false, "error": "429 private-account"}),
        0,
    )
    .unwrap_err();
    assert_eq!(failure.details.as_ref().unwrap()["code"], "provider_error");
    assert!(!format!("{failure:?}").contains("private-account"));
}

#[test]
fn malformed_fields_and_contradictory_identity_fail_closed() {
    let request = Request::new("NASDAQ:EXAMPLE", "1D").unwrap();
    for (pointer, value) in [
        ("/success", json!("true")),
        ("/data", Value::Null),
        ("/data/symbol", json!("NASDAQ:OTHER")),
        ("/data/interval", json!("1W")),
        ("/data/oscillators", json!([])),
        ("/data/moving_averages", Value::Null),
        ("/data/summary", json!(false)),
        ("/data/oscillators/rsi", json!("52.5")),
        ("/data/summary/recommendation", json!(0)),
        ("/data/summary/value", json!({})),
    ] {
        let mut input = response();
        *input.pointer_mut(pointer).unwrap() = value;
        assert_eq!(
            normalize(&request, input, 0).unwrap_err().details.unwrap()["code"],
            "invalid_response"
        );
    }
    for key in ["oscillators", "moving_averages", "summary"] {
        let mut input = response();
        input["data"].as_object_mut().unwrap().remove(key);
        assert!(normalize(&request, input, 0).is_err());
    }
}
