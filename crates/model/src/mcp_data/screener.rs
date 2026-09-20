//! Official screener input validation and source-honest result shaping.

use super::{
    Kind, Request, column_arguments, invalid_request, invalid_response, optional_count,
    select_fields, text,
};
use serde_json::{Value, json};
use tradingview_core::AppError;

#[derive(Clone, Debug)]
pub struct ScreenerOptions {
    pub market: String,
    pub filters: Value,
    pub sort_by: String,
    pub sort_order: String,
    pub limit: u32,
    pub columns: Vec<String>,
    pub symbol_types: Option<Vec<String>>,
    pub filter_preset: Option<String>,
    pub symbolset: Option<Vec<String>>,
}

impl Default for ScreenerOptions {
    fn default() -> Self {
        Self {
            market: "america".into(),
            filters: json!({}),
            sort_by: "volume".into(),
            sort_order: "desc".into(),
            limit: 100,
            columns: Vec::new(),
            symbol_types: None,
            filter_preset: None,
            symbolset: None,
        }
    }
}

impl ScreenerOptions {
    pub fn parse_filters(raw: &str) -> Result<Value, AppError> {
        if raw.len() > 16384 {
            return Err(invalid_request("filters_size"));
        }
        serde_json::from_str(raw).map_err(|_| invalid_request("filters_json"))
    }

    pub fn from_arguments(args: &Value) -> Result<Self, AppError> {
        let string = |name| {
            args.get(name)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| invalid_request("screener"))
        };
        let strings = |name| {
            args.get(name)
                .map(|value| {
                    serde_json::from_value::<Vec<String>>(value.clone())
                        .map_err(|_| invalid_request("screener"))
                })
                .transpose()
        };
        Ok(Self {
            market: string("market")?,
            filters: args
                .get("filters")
                .cloned()
                .ok_or_else(|| invalid_request("filters"))?,
            sort_by: string("sort_by")?,
            sort_order: string("sort_order")?,
            limit: args
                .get("limit")
                .and_then(Value::as_u64)
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| invalid_request("limit"))?,
            columns: strings("columns")?.ok_or_else(|| invalid_request("columns"))?,
            symbol_types: strings("symbol_types")?,
            filter_preset: args
                .get("filter_preset")
                .map(|_| string("filter_preset"))
                .transpose()?,
            symbolset: strings("symbolset")?,
        })
    }
}

impl Request {
    pub fn screener(mut options: ScreenerOptions) -> Result<Self, AppError> {
        text(&options.market, 64, "market")?;
        if !options
            .market
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b"_-".contains(&b))
        {
            return Err(invalid_request("market"));
        }
        text(&options.sort_by, 128, "sort_by")?;
        if !matches!(options.sort_order.as_str(), "asc" | "desc")
            || !(1..=1000).contains(&options.limit)
        {
            return Err(invalid_request("sort_or_limit"));
        }
        let filters = options
            .filters
            .as_object()
            .ok_or_else(|| invalid_request("filters"))?;
        if filters.len() > 50 {
            return Err(invalid_request("filters_count"));
        }
        for (field, value) in filters {
            text(field, 128, "filter_field")?;
            if matches!(
                field.as_str(),
                "index" | "sector" | "industry" | "analyst_rating"
            ) && let Some(value) = value.as_str()
            {
                text(value, 256, "filter_value")?;
                continue;
            }
            let range = value
                .as_array()
                .filter(|v| v.len() == 2)
                .ok_or_else(|| invalid_request("filter_range"))?;
            if range
                .iter()
                .any(|v| !v.is_null() && v.as_f64().is_none_or(|n| !n.is_finite()))
            {
                return Err(invalid_request("filter_range"));
            }
            if let (Some(min), Some(max)) = (range[0].as_f64(), range[1].as_f64())
                && min > max
            {
                return Err(invalid_request("filter_range_order"));
            }
        }
        if options.filter_preset.as_deref().is_some_and(|v| {
            !matches!(
                v,
                "pullback_with_reversal"
                    | "breakout_with_volume"
                    | "oversold_healthy"
                    | "earnings_runup"
                    | "relative_strength"
                    | "new_highs"
            )
        }) {
            return Err(invalid_request("filter_preset"));
        }
        for values in [&options.symbol_types, &options.symbolset]
            .into_iter()
            .flatten()
        {
            if values.is_empty() || values.len() > 50 {
                return Err(invalid_request("selection_count"));
            }
            let mut unique = std::collections::HashSet::new();
            for value in values {
                text(value, 256, "selection_value")?;
                if !unique.insert(value) {
                    return Err(invalid_request("duplicate_selection"));
                }
            }
        }
        options.columns = serde_json::from_value(column_arguments(&options.columns)?)
            .map_err(|_| invalid_request("columns"))?;
        let mut arguments = json!({
            "market": options.market,
            "filters": options.filters,
            "sort_by": options.sort_by,
            "sort_order": options.sort_order,
            "limit": options.limit,
            "columns": options.columns
        });
        if let Some(types) = options.symbol_types {
            arguments["symbol_types"] = json!(types);
        }
        if let Some(preset) = options.filter_preset {
            arguments["filter_preset"] = json!(preset);
        }
        if let Some(symbolset) = options.symbolset {
            arguments["symbolset"] = json!(symbolset);
        }
        Ok(Self {
            kind: Kind::Screener,
            arguments,
        })
    }
}

pub(super) fn normalize(
    request: &Request,
    output: &mut Value,
    value: &Value,
) -> Result<(), AppError> {
    let data = value
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid_response("missing_screener_data"))?;
    let rows = data
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("missing_screener_rows"))?;
    let limit = request.arguments["limit"]
        .as_u64()
        .ok_or_else(|| invalid_response("invalid_limit"))?;
    let total = optional_count(data.get("totalCount"))?;
    if rows.len() as u64 > limit
        || total
            .as_u64()
            .is_some_and(|total| total < rows.len() as u64)
    {
        return Err(invalid_response("screener_count_mismatch"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut items = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let row = row
            .as_object()
            .ok_or_else(|| invalid_response("invalid_screener_row"))?;
        let symbol = row
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid_response("missing_symbol_identity"))?;
        crate::mcp_bars::validate_symbol(symbol)
            .map_err(|_| invalid_response("invalid_symbol_identity"))?;
        if !seen.insert(symbol) {
            return Err(invalid_response("duplicate_symbol"));
        }
        let (fields, missing) = select_fields(request.arguments.get("columns"), row)?;
        items.push(json!({
            "index": index,
            "symbol": symbol,
            "fields": fields,
            "client_observation": {
                "fields_status": if missing.is_empty() { "present" } else { "incomplete" },
                "missing_fields": missing
            }
        }));
    }
    let unknown = json!({"value": null, "evidence": "unconfirmed"});
    output["items"] = json!(items);
    output["provider_observation"] = json!({
        "total_count": {"value": total, "evidence": if total.is_null() { "unconfirmed" } else { "provider_response" }},
        "data_as_of": unknown,
        "delay_seconds": unknown,
        "session": unknown
    });
    output["client_observation"]["returned_count"] = json!(rows.len());
    output["client_observation"]["coverage_status"] = json!(match total.as_u64() {
        Some(total) if total > rows.len() as u64 => "limited",
        Some(_) => "all_reported",
        None => "unconfirmed",
    });
    output["client_observation"]["order"] = json!("provider_response");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_ranges_limits_and_presets_before_io() {
        for filters in [
            json!([]),
            json!({"close": [2, 1]}),
            json!({"close": [1]}),
            json!({"close": ["bad", null]}),
        ] {
            assert!(
                Request::screener(ScreenerOptions {
                    filters,
                    ..Default::default()
                })
                .is_err()
            );
        }
        for limit in [0, 1001] {
            assert!(
                Request::screener(ScreenerOptions {
                    limit,
                    ..Default::default()
                })
                .is_err()
            );
        }
        assert!(
            Request::screener(ScreenerOptions {
                filter_preset: Some("unknown".into()),
                ..Default::default()
            })
            .is_err()
        );
        assert!(ScreenerOptions::parse_filters("{\"close\":[NaN,null]}").is_err());
    }

    #[test]
    fn accepted_options_round_trip_without_rewriting_the_query() {
        let request = Request::screener(ScreenerOptions {
            market: "america".into(),
            filters: json!({"close": [10, null], "sector": "Technology"}),
            sort_order: "asc".into(),
            symbol_types: Some(vec!["stock".into()]),
            filter_preset: Some("new_highs".into()),
            symbolset: Some(vec!["SYML:SP;SPX".into()]),
            ..Default::default()
        })
        .unwrap();
        let parsed = ScreenerOptions::from_arguments(&request.arguments()).unwrap();
        assert_eq!(
            Request::screener(parsed).unwrap().arguments(),
            request.arguments()
        );
        assert_eq!(request.arguments()["symbolset"][0], "SYML:SP;SPX");
    }

    #[test]
    fn preserves_provider_order_nulls_and_partial_coverage() {
        let request = Request::screener(ScreenerOptions {
            limit: 2,
            columns: vec!["close".into(), "volume".into()],
            ..Default::default()
        })
        .unwrap();
        let value = json!({"success": true, "data": {
            "rows": [
                {"symbol": "NYSE:SECOND", "close": 0, "volume": null, "extra": "ignored"},
                {"symbol": "NASDAQ:FIRST", "close": 1}
            ],
            "totalCount": 100
        }});
        let result = super::super::normalize(&request, value, 0).unwrap();
        assert_eq!(result["items"][0]["symbol"], "NYSE:SECOND");
        assert_eq!(result["items"][0]["fields"]["close"], 0);
        assert_eq!(result["items"][0]["fields"].as_object().unwrap().len(), 2);
        assert_eq!(
            result["items"][0]["client_observation"]["missing_fields"][0]["reason"],
            "null"
        );
        assert_eq!(
            result["items"][1]["client_observation"]["missing_fields"][0]["reason"],
            "absent"
        );
        assert_eq!(result["client_observation"]["coverage_status"], "limited");
        assert!(result["provider_observation"]["data_as_of"]["value"].is_null());
    }

    #[test]
    fn empty_and_unconfirmed_totals_are_not_fabricated_completeness() {
        let request = Request::screener(Default::default()).unwrap();
        for (data, expected) in [
            (json!({"rows": [], "totalCount": 0}), "all_reported"),
            (json!({"rows": []}), "unconfirmed"),
            (json!({"rows": [], "totalCount": 5}), "limited"),
        ] {
            let result = super::super::normalize(&request, json!({"data": data}), 0).unwrap();
            assert_eq!(result["client_observation"]["coverage_status"], expected);
            assert_eq!(result["client_observation"]["returned_count"], 0);
        }
    }

    #[test]
    fn rejects_malformed_duplicate_and_excess_rows() {
        let request = Request::screener(ScreenerOptions {
            limit: 1,
            ..Default::default()
        })
        .unwrap();
        for data in [
            json!({"rows": [{"symbol": "NASDAQ:FIRST"}], "totalCount": 0}),
            json!({"rows": [{"symbol": "NASDAQ:FIRST"}, {"symbol": "NASDAQ:FIRST"}]}),
            json!({"rows": [{"symbol": "FIRST"}]}),
            json!({"rows": [null]}),
            json!({"rows": [], "totalCount": "1"}),
        ] {
            assert!(super::super::normalize(&request, json!({"data": data}), 0).is_err());
        }
        let request = Request::screener(Default::default()).unwrap();
        assert!(
            super::super::normalize(
                &request,
                json!({"data": {
                    "rows": [{"symbol": "NASDAQ:FIRST"}, {"symbol": "NASDAQ:FIRST"}]
                }}),
                0
            )
            .is_err()
        );
    }
}
