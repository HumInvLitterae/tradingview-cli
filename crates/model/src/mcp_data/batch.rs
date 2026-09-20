//! Match batch results by explicit provider identity, never by response position.

use super::{Request, invalid_response, normalize_symbol, validate_count};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use tradingview_core::AppError;

pub(super) fn normalize(
    request: &Request,
    output: &mut Value,
    value: &Value,
) -> Result<(), AppError> {
    let data = value
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid_response("missing_batch_data"))?;
    validate_count(value.get("count"), data.len())?;
    let mut rows = Vec::new();
    for (symbol, fields) in data {
        let fields = fields
            .as_object()
            .ok_or_else(|| invalid_response("invalid_batch_fields"))?;
        rows.push((symbol.clone(), fields.clone()));
    }

    let mut missing = Vec::new();
    if let Some(entries) = value.get("missing") {
        for entry in entries
            .as_array()
            .ok_or_else(|| invalid_response("invalid_missing_list"))?
        {
            let symbol = entry
                .get("symbol")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_response("missing_symbol_identity"))?;
            if entry
                .get("reason")
                .is_some_and(|reason| !reason.is_string())
            {
                return Err(invalid_response("invalid_missing_reason"));
            }
            // The provider's freeform reason is not a classified failure cause.
            missing.push(symbol.to_owned());
        }
    }
    validate_count(value.get("missing_count"), missing.len())?;
    assemble(request, output, rows, missing)
}

fn assemble(
    request: &Request,
    output: &mut Value,
    rows: Vec<(String, Map<String, Value>)>,
    missing: Vec<String>,
) -> Result<(), AppError> {
    let requested: Vec<String> = serde_json::from_value(request.arguments["symbols"].clone())
        .map_err(|_| invalid_response("invalid_batch_request"))?;
    let requested_set: HashSet<_> = requested.iter().collect();
    let mut by_symbol = HashMap::new();
    for (symbol, fields) in rows {
        if !requested_set.contains(&symbol) || by_symbol.insert(symbol, fields).is_some() {
            return Err(invalid_response("unexpected_or_duplicate_symbol"));
        }
    }

    let mut missing_set = HashSet::new();
    for symbol in missing {
        if !requested_set.contains(&symbol)
            || by_symbol.contains_key(&symbol)
            || !missing_set.insert(symbol)
        {
            return Err(invalid_response("conflicting_missing_symbols"));
        }
    }

    let returned_count = by_symbol.len();
    let missing_count = missing_set.len();
    let mut items = Vec::new();
    for (index, symbol) in requested.iter().enumerate() {
        let mut item = json!({
            "requested_index": index,
            "requested_symbol": symbol
        });
        if let Some(fields) = by_symbol.remove(symbol) {
            item["status"] = json!("returned");
            normalize_symbol(
                &json!({"symbol": symbol, "columns": request.arguments["columns"]}),
                &mut item,
                &json!({"symbol": symbol, "data": fields}),
            )?;
        } else {
            item["status"] = json!(if missing_set.contains(symbol) {
                "missing"
            } else {
                "unreported"
            });
            item["fields"] = Value::Null;
        }
        items.push(item);
    }

    output["items"] = json!(items);
    output["client_observation"]["requested_count"] = json!(requested.len());
    output["client_observation"]["returned_count"] = json!(returned_count);
    output["client_observation"]["missing_count"] = json!(missing_count);
    output["client_observation"]["unreported_count"] =
        json!(requested.len() - returned_count - missing_count);
    output["client_observation"]["symbols_status"] = json!(if returned_count == requested.len() {
        "all_returned"
    } else if returned_count == 0 {
        "none_returned"
    } else {
        "partial"
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_decoder_checks_shape_counts_and_explicit_missing_entries() {
        let request = Request::symbols(
            &["NASDAQ:EXAMPLE".into(), "NYSE:MISSING".into()],
            &["close".into()],
        )
        .unwrap();
        let wire = json!({
            "success": true,
            "data": {"NASDAQ:EXAMPLE": {"close": 0}},
            "count": 1,
            "missing": [{"symbol": "NYSE:MISSING", "reason": "synthetic unavailable"}],
            "missing_count": 1
        });
        let result = super::super::normalize(&request, wire.clone(), 0).unwrap();
        assert_eq!(result["contract_version"], "mcp_symbols.v1");
        assert_eq!(
            result["items"][0]["client_observation"]["identity_match"],
            "matched"
        );
        assert_eq!(result["items"][1]["status"], "missing");
        for (pointer, replacement) in [
            ("/data", json!([])),
            ("/data/NASDAQ:EXAMPLE", Value::Null),
            ("/count", json!(2)),
            ("/missing", Value::Null),
            ("/missing/0/symbol", json!("NASDAQ:EXAMPLE")),
            ("/missing/0/reason", json!({})),
            ("/missing_count", json!(0)),
            ("/success", json!(false)),
        ] {
            let mut invalid = wire.clone();
            *invalid.pointer_mut(pointer).unwrap() = replacement;
            assert!(
                super::super::normalize(&request, invalid, 0).is_err(),
                "{pointer}"
            );
        }
    }

    #[test]
    fn preserves_request_order_partiality_and_field_absence() {
        let request = Request::symbols(
            &[
                "NASDAQ:FIRST".into(),
                "NASDAQ:SECOND".into(),
                "NASDAQ:MISSING".into(),
                "NASDAQ:UNREPORTED".into(),
            ],
            &["close".into(), "volume".into()],
        )
        .unwrap();
        let mut output = json!({});
        assemble(
            &request,
            &mut output,
            vec![
                (
                    "NASDAQ:SECOND".into(),
                    json!({"close": 0, "volume": null})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                (
                    "NASDAQ:FIRST".into(),
                    json!({"close": 1}).as_object().unwrap().clone(),
                ),
            ],
            vec!["NASDAQ:MISSING".into()],
        )
        .unwrap();
        let items = output["items"].as_array().unwrap();
        assert_eq!(items[0]["requested_symbol"], "NASDAQ:FIRST");
        assert_eq!(items[1]["requested_index"], 1);
        assert_eq!(items[1]["fields"]["close"], 0);
        assert_eq!(
            items[0]["client_observation"]["missing_fields"][0]["reason"],
            "absent"
        );
        assert_eq!(
            items[1]["client_observation"]["missing_fields"][0]["reason"],
            "null"
        );
        assert_eq!(items[2]["status"], "missing");
        assert_eq!(items[3]["status"], "unreported");
        assert!(items[2]["fields"].is_null());
        assert_eq!(output["client_observation"]["symbols_status"], "partial");
        assert_eq!(output["client_observation"]["unreported_count"], 1);
    }

    #[test]
    fn rejects_unrequested_duplicate_and_contradictory_identity() {
        let request = Request::symbols(&["NASDAQ:EXAMPLE".into()], &["close".into()]).unwrap();
        let row = ("NASDAQ:EXAMPLE".into(), Map::new());
        for (rows, missing) in [
            (vec![("NYSE:OTHER".into(), Map::new())], vec![]),
            (vec![row.clone(), row.clone()], vec![]),
            (vec![row], vec!["NASDAQ:EXAMPLE".into()]),
            (
                vec![],
                vec!["NASDAQ:EXAMPLE".into(), "NASDAQ:EXAMPLE".into()],
            ),
        ] {
            assert!(assemble(&request, &mut json!({}), rows, missing).is_err());
        }
    }

    #[test]
    fn none_returned_does_not_turn_silence_into_provider_missing() {
        let request = Request::symbols(&["NASDAQ:EXAMPLE".into()], &["close".into()]).unwrap();
        for missing in [vec![], vec!["NASDAQ:EXAMPLE".into()]] {
            let expected = if missing.is_empty() {
                "unreported"
            } else {
                "missing"
            };
            let mut output = json!({});
            assemble(&request, &mut output, vec![], missing).unwrap();
            assert_eq!(output["items"][0]["status"], expected);
            assert!(output["items"][0]["fields"].is_null());
            assert_eq!(output["client_observation"]["returned_count"], 0);
            assert_eq!(
                output["client_observation"]["symbols_status"],
                "none_returned"
            );
        }
    }
}
