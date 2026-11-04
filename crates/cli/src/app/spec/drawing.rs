//! Drawing effects and geometry constraints; no chart operations occur here.

use serde_json::{Value, json};
use tradingview_model::drawing::PositionDirection;

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["draw", action] = path else {
        return None;
    };
    let mut result = json!({
        "source": "chart_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": !matches!(*action, "list" | "get"), "broker_order": false},
        "output_contract": null,
        "constraints": {},
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"}],
        "readback": {"argv": ["tv", "--target-id", "<target_id>", "draw", "list"]},
        "limits": [
            "Drawing IDs belong to the selected chart. Do not substitute a name or another target's ID.",
            "Creation failures do not guarantee unchanged state; inspect the selected chart before retrying.",
            "Position drawings are visual tools, not broker orders. No implicit source or target fallback is used."
        ]
    });
    let example = match *action {
        "shape" => {
            result["constraints"] = json!({
                "type": {"nonblank": true, "trimmed": true, "catalog": "runtime-dependent"},
                "coordinates": {"finite": true, "time_unit": "Unix seconds"},
                "required_together": [["price2", "time2"], ["price3", "time3"]],
                "overrides": {"type": "JSON object", "empty_allowed": true},
                "point3": {"requires_point2": true, "type": "parallel_channel",
                    "time3_equals_time": true, "nonblank_text_allowed": false}
            });
            result["variants"] = json!([
                {"when": {"absent": ["price2", "time2", "price3", "time3"]}, "operation": "single_point_shape"},
                {"when": {"present": ["price2", "time2"], "absent": ["price3", "time3"]}, "operation": "two_point_shape"},
                {"when": {"present": ["price3", "time3"]}, "operation": "verified_parallel_channel", "constraints": "point3 rules apply"}
            ]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("The generic shape type is not a closed local enum; provider support and point requirements depend on the drawing type."),
                json!("Three-point parallel_channel has dedicated identity/point verification; inspect verification and cleanup evidence after failure."),
                json!("Ordinary one/two-point creation observes new IDs and counts; those are not proof that all geometry/properties match. Read back the returned entity ID.")
            ]);
            json!([
                "tv",
                "draw",
                "shape",
                "--type",
                "horizontal_line",
                "--price",
                "100",
                "--time",
                "1704067200"
            ])
        }
        "position" => {
            result["constraints"] = json!({
                "direction": {"choices": [PositionDirection::Long.as_str(), PositionDirection::Short.as_str()],
                    "trimmed": true, "case": "ASCII lowercase"},
                "exactly_one": ["direction", "direction_flag"],
                "prices": {"finite": true, "long": "stop_loss < entry_price < take_profit",
                    "short": "take_profit < entry_price < stop_loss"},
                "entry-time": {"finite": true, "unit": "Unix seconds",
                    "omitted": "visible range end, otherwise current Unix time"},
                "account-size": positive(), "risk": positive(), "lot-size": positive()
            });
            result["limits"].as_array_mut().unwrap().extend([
                json!("direction_flag is the clap ID for --direction; choose it or the positional direction, never both."),
                json!("Stop/profit levels are rounded using the selected symbol's pricescale. Displayed geometry and risk/reward can differ from exact input differences."),
                json!("Optional account/risk/lot settings are forwarded only when supplied. Inspect the resulting drawing; no order or position is opened.")
            ]);
            json!([
                "tv",
                "draw",
                "position",
                "long",
                "--entry-price",
                "100",
                "--stop-loss",
                "90",
                "--take-profit",
                "120"
            ])
        }
        "list" => {
            result["limits"].as_array_mut().unwrap().push(json!("Returns shape IDs/names from the selected chart inventory, not a complete properties export."));
            json!(["tv", "draw", "list"])
        }
        "get" | "remove" => {
            result["constraints"]["entity_id"] =
                json!({"nonblank": true, "scope": "selected chart"});
            result["discovery"].as_array_mut().unwrap().push(json!({"argument": "entity_id",
                "argv": ["tv", "--target-id", "<target_id>", "draw", "list"], "result_path": "data.shapes[].id"}));
            let limit = if *action == "get" {
                "Points/properties/visibility may be absent or include read errors; absent values are not defaults."
            } else {
                "Removes the selected entity and checks that the same ID is absent; verification failure is an error."
            };
            result["limits"].as_array_mut().unwrap().push(json!(limit));
            json!(["tv", "draw", action, "<entity_id>"])
        }
        "clear" => {
            result["effects"]["chart_mutation"] = Value::Null;
            result["variants"] = json!([
                {"when": {"flag_true": ["dry_run"]}, "effects": {"chart_mutation": false}, "operation": "inspect_would_clear"},
                {"when": {"flag_false": ["dry_run"]}, "effects": {"chart_mutation": true, "scope": "all drawings on selected chart"}, "operation": "clear_all"}
            ]);
            result["limits"].as_array_mut().unwrap().push(json!("Default execution removes all chart drawings and verifies an empty inventory. dry-run only reports targets/counts and does not reserve a later deletion set; empty inventory is a no-op."));
            json!(["tv", "draw", "clear", "--dry-run"])
        }
        _ => return None,
    };
    result["examples"] = json!([example]);
    Some(result)
}

fn positive() -> Value {
    json!({"finite": true, "exclusive_minimum": 0, "optional": true})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::drawing::{
        DrawingPoint, DrawingPositionRequest, DrawingShapeRequest, parse_drawing_overrides,
        validate_position_request, validate_shape_request,
    };

    #[test]
    fn examples_parse_and_clear_never_claims_unconditional_read() {
        for action in ["shape", "position", "list", "get", "remove", "clear"] {
            for example in describe(&["draw", action]).unwrap()["examples"]
                .as_array()
                .unwrap()
            {
                assert!(
                    Cli::try_parse_from(
                        example
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_str().unwrap())
                    )
                    .is_ok()
                );
            }
        }

        let clear = describe(&["draw", "clear"]).unwrap();
        assert!(clear["effects"]["chart_mutation"].is_null());
        assert_eq!(clear["variants"][0]["effects"]["chart_mutation"], false);
        assert_eq!(clear["variants"][1]["effects"]["chart_mutation"], true);
    }

    #[test]
    fn described_geometry_matches_model_validation() {
        // Each check reads the published constraint and confirms the model
        // validator enforces it, so the two cannot drift apart silently.
        let shape_spec = describe(&["draw", "shape"]).unwrap();
        let point3 = &shape_spec["constraints"]["point3"];
        let mut shape = DrawingShapeRequest {
            shape_type: point3["type"].as_str().unwrap().into(),
            point: DrawingPoint {
                time: 1.0,
                price: 100.0,
            },
            point2: Some(DrawingPoint {
                time: 2.0,
                price: 110.0,
            }),
            point3: Some(DrawingPoint {
                time: 1.0,
                price: 120.0,
            }),
            text: None,
            overrides: None,
        };
        assert!(validate_shape_request(&shape).is_ok());
        assert_eq!(point3["time3_equals_time"], true);
        shape.point3.as_mut().unwrap().time = 3.0;
        assert!(validate_shape_request(&shape).is_err());
        shape.point3.as_mut().unwrap().time = 1.0;
        assert_eq!(point3["nonblank_text_allowed"], false);
        shape.text = Some("unsupported".into());
        assert!(validate_shape_request(&shape).is_err());
        shape.text = None;
        assert_eq!(point3["requires_point2"], true);
        shape.point2 = None;
        assert!(validate_shape_request(&shape).is_err());
        shape.point2 = Some(DrawingPoint {
            time: 2.0,
            price: 110.0,
        });
        shape.shape_type = "trend_line".into();
        assert!(validate_shape_request(&shape).is_err());
        assert_eq!(shape_spec["constraints"]["coordinates"]["finite"], true);
        shape.shape_type = point3["type"].as_str().unwrap().into();
        shape.point.price = f64::NAN;
        assert!(validate_shape_request(&shape).is_err());
        assert_eq!(
            shape_spec["constraints"]["overrides"],
            json!({"type": "JSON object", "empty_allowed": true})
        );
        assert!(parse_drawing_overrides("{}").is_ok());
        assert!(parse_drawing_overrides("[]").is_err());

        let position_spec = describe(&["draw", "position"]).unwrap();
        let constraints = &position_spec["constraints"];
        let long = PositionDirection::Long.as_str();
        let short = PositionDirection::Short.as_str();
        assert_eq!(constraints["direction"]["choices"], json!([long, short]));
        assert_eq!(
            constraints["prices"]["long"],
            "stop_loss < entry_price < take_profit"
        );
        assert_eq!(
            constraints["prices"]["short"],
            "take_profit < entry_price < stop_loss"
        );
        let valid = |direction| DrawingPositionRequest {
            direction,
            entry_price: 100.0,
            stop_loss: if direction == PositionDirection::Long {
                90.0
            } else {
                110.0
            },
            take_profit: if direction == PositionDirection::Long {
                120.0
            } else {
                80.0
            },
            entry_time: None,
            account_size: None,
            risk: None,
            lot_size: None,
        };
        for direction in [PositionDirection::Long, PositionDirection::Short] {
            let request = valid(direction);
            assert!(validate_position_request(&request).is_ok());
            let mut swapped = request.clone();
            std::mem::swap(&mut swapped.stop_loss, &mut swapped.take_profit);
            assert!(validate_position_request(&swapped).is_err());
        }

        type Setter = fn(&mut DrawingPositionRequest, f64);
        let optional: [(&str, Setter); 3] = [
            ("account-size", |r, v| r.account_size = Some(v)),
            ("risk", |r, v| r.risk = Some(v)),
            ("lot-size", |r, v| r.lot_size = Some(v)),
        ];
        for (key, set) in optional {
            assert_eq!(constraints[key]["finite"], true, "{key}");
            let minimum = constraints[key]["exclusive_minimum"].as_f64().unwrap();
            for (value, accepted) in [
                (minimum, false),
                (minimum + 0.01, true),
                (f64::INFINITY, false),
            ] {
                let mut request = valid(PositionDirection::Long);
                set(&mut request, value);
                assert_eq!(
                    validate_position_request(&request).is_ok(),
                    accepted,
                    "{key} = {value}"
                );
            }
        }
    }
}
