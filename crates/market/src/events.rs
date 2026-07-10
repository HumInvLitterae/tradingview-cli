use serde_json::{Map, Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{
    fundamentals::fundamentals_symbol_with_groups_typed_with_client,
    http::configured_client,
    types::{
        EventEntry, EventFieldReadback, EventSourceAvailability, Events, EventsCompare,
        EventsCompareItem, EventsCompareSummary, Fundamentals,
    },
};

const EVENTS_CONTRACT_VERSION: &str = "events.v1";
const EVENTS_COMPARE_CONTRACT_VERSION: &str = "events_compare.v1";
const EARNINGS: &str = "earnings";
const DIVIDENDS: &str = "dividends";
const ALL: &str = "all";
const MAX_EVENTS_COMPARE_SYMBOLS: usize = 25;

pub async fn events_symbol(symbol: &str, event_type: &str) -> Result<Value, AppError> {
    serde_json::to_value(events_symbol_typed(symbol, event_type).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

pub async fn events_compare_symbols(
    symbols: Vec<String>,
    event_type: &str,
) -> Result<Value, AppError> {
    serde_json::to_value(events_compare_symbols_typed(symbols, event_type).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads scanner-backed event-like fundamentals fields for one symbol.
///
/// This is an event-shaped view over scanner fundamentals fields. It does not
/// infer timezone/session semantics and does not read a standalone event
/// calendar source.
pub async fn events_symbol_typed(symbol: &str, event_type: &str) -> Result<Events, AppError> {
    let client = configured_client()?;
    events_symbol_typed_with_client(&client, symbol, event_type).await
}

async fn events_symbol_typed_with_client(
    client: &reqwest::Client,
    symbol: &str,
    event_type: &str,
) -> Result<Events, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(events_validation_error(
            "events symbol must not be empty",
            requested_symbol,
            event_type,
        ));
    }
    let requested_event_type = normalize_event_type(event_type)?;
    let groups = event_groups(requested_event_type);
    let fundamentals = fundamentals_symbol_with_groups_typed_with_client(
        client,
        requested_symbol,
        groups.clone(),
        Vec::new(),
    )
    .await
    .map_err(|err| add_event_error_details(err, requested_symbol, requested_event_type))?;

    Ok(events_from_fundamentals(fundamentals, requested_event_type))
}

/// Reads scanner-backed event-like fundamentals fields for several symbols.
///
/// This is a bounded, ordered view over `events.v1` payloads. It does not read
/// a standalone event calendar source and does not rank or recommend symbols.
pub async fn events_compare_symbols_typed(
    symbols: Vec<String>,
    event_type: &str,
) -> Result<EventsCompare, AppError> {
    let client = configured_client()?;
    let requested_event_type = normalize_event_type(event_type)?;
    let requested_symbols = normalize_compare_symbols(symbols)?;

    let mut items = Vec::with_capacity(requested_symbols.len());
    for (requested_index, requested_symbol) in requested_symbols.iter().enumerate() {
        match events_symbol_typed_with_client(&client, requested_symbol, requested_event_type).await
        {
            Ok(events) => items.push(EventsCompareItem {
                requested_index,
                requested_symbol: requested_symbol.clone(),
                status: "ok".to_string(),
                events: Some(events),
                failure_details: None,
            }),
            Err(err) => items.push(EventsCompareItem {
                requested_index,
                requested_symbol: requested_symbol.clone(),
                status: "error".to_string(),
                events: None,
                failure_details: Some(sanitized_error_details(&err)),
            }),
        }
    }

    Ok(events_compare_from_items(
        requested_symbols,
        requested_event_type,
        items,
    ))
}

fn normalize_compare_symbols(symbols: Vec<String>) -> Result<Vec<String>, AppError> {
    if symbols.len() < 2 {
        return Err(events_compare_validation_error(
            "events compare requires at least two symbols",
        ));
    }
    if symbols.len() > MAX_EVENTS_COMPARE_SYMBOLS {
        return Err(events_compare_validation_error(format!(
            "events compare accepts at most {MAX_EVENTS_COMPARE_SYMBOLS} symbols"
        )));
    }

    let requested_symbols = symbols
        .into_iter()
        .map(|symbol| symbol.trim().to_string())
        .collect::<Vec<_>>();
    if requested_symbols.iter().any(String::is_empty) {
        return Err(events_compare_validation_error(
            "events compare symbol must not be empty",
        ));
    }
    Ok(requested_symbols)
}

fn events_compare_from_items(
    requested_symbols: Vec<String>,
    requested_event_type: &str,
    items: Vec<EventsCompareItem>,
) -> EventsCompare {
    let ok_count = items.iter().filter(|item| item.status == "ok").count();
    let error_count = items.iter().filter(|item| item.status == "error").count();
    let total_event_count = items
        .iter()
        .filter_map(|item| item.events.as_ref())
        .map(|events| events.event_count)
        .sum::<usize>();
    let symbols_with_events_count = items
        .iter()
        .filter_map(|item| item.events.as_ref())
        .filter(|events| events.event_count > 0)
        .count();
    let symbols_without_events_count = items
        .iter()
        .filter_map(|item| item.events.as_ref())
        .filter(|events| events.event_count == 0)
        .count();

    EventsCompare {
        contract_version: EVENTS_COMPARE_CONTRACT_VERSION.to_string(),
        source: "scanner_fundamentals_rest".to_string(),
        source_category: "desktop_free_read".to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_symbols,
        requested_event_type: requested_event_type.to_string(),
        event_types: event_groups(requested_event_type),
        items,
        summary: EventsCompareSummary {
            requested_count: ok_count + error_count,
            ok_count,
            error_count,
            total_event_count,
            symbols_with_events_count,
            symbols_without_events_count,
        },
    }
}

fn normalize_event_type(event_type: &str) -> Result<&'static str, AppError> {
    match event_type.trim() {
        "" | ALL => Ok(ALL),
        EARNINGS => Ok(EARNINGS),
        DIVIDENDS => Ok(DIVIDENDS),
        other => Err(AppError::new(
            ErrorKind::Validation,
            format!("Unsupported events event type: {other}"),
        )
        .with_details(json!({
            "requested_event_type": other,
            "supported_event_types": [ALL, EARNINGS, DIVIDENDS],
            "source": "scanner_fundamentals_rest",
            "source_category": "desktop_free_read",
            "requires_desktop": false,
            "non_mutating": true,
            "next_action_hint": "Use --event-type all, --event-type earnings, or --event-type dividends.",
        }))),
    }
}

fn event_groups(event_type: &str) -> Vec<String> {
    match event_type {
        EARNINGS => vec![EARNINGS.to_string()],
        DIVIDENDS => vec![DIVIDENDS.to_string()],
        _ => vec![EARNINGS.to_string(), DIVIDENDS.to_string()],
    }
}

fn events_from_fundamentals(fundamentals: Fundamentals, requested_event_type: &str) -> Events {
    let mut events = Vec::new();
    if requested_event_type == ALL || requested_event_type == EARNINGS {
        push_earnings_events(&mut events, &fundamentals.field_values);
    }
    if requested_event_type == ALL || requested_event_type == DIVIDENDS {
        push_dividend_events(&mut events, &fundamentals.field_values);
    }

    let unavailable_fields = fundamentals
        .fields
        .iter()
        .filter(|field| value_for(&fundamentals.field_values, field).is_none())
        .cloned()
        .collect::<Vec<_>>();
    let unavailable_field_count = fundamentals.missing_fields.len() + unavailable_fields.len();
    let event_count = events.len();
    let status = if event_count > 0 {
        "events_present"
    } else {
        "no_events_returned"
    };

    Events {
        contract_version: EVENTS_CONTRACT_VERSION.to_string(),
        source: fundamentals.source,
        source_category: fundamentals.source_category,
        requires_desktop: fundamentals.requires_desktop,
        non_mutating: fundamentals.non_mutating,
        requested_symbol: fundamentals.requested_symbol,
        symbol: fundamentals.symbol,
        observed_symbol: fundamentals.observed_symbol,
        market: fundamentals.market,
        requested_event_type: requested_event_type.to_string(),
        event_types: event_groups(requested_event_type),
        event_count,
        events,
        field_readback: EventFieldReadback {
            requested_groups: fundamentals.requested_groups,
            requested_fields: fundamentals.fields,
            missing_fields: fundamentals.missing_fields,
            unavailable_fields,
        },
        source_availability: EventSourceAvailability {
            status: status.to_string(),
            event_count,
            unavailable_field_count,
        },
    }
}

fn push_earnings_events(events: &mut Vec<EventEntry>, values: &Value) {
    push_event_if_present(
        events,
        EventEntry {
            event_type: EARNINGS.to_string(),
            event_status: "next".to_string(),
            date: value_for(values, "earnings_release_next_date"),
            calendar_date: value_for(values, "earnings_release_next_calendar_date"),
            trading_date: value_for(values, "earnings_release_next_trading_date_fq"),
            time: value_for(values, "earnings_release_next_time"),
            publication_type: value_for(values, "earnings_publication_type_next_fq"),
            ex_date: None,
            payment_date: None,
            amount: None,
            frequency: None,
            dividend_yield: None,
            expected_annual_dividends: None,
            source_fields: source_fields(
                values,
                &[
                    "earnings_release_next_date",
                    "earnings_release_next_calendar_date",
                    "earnings_release_next_trading_date_fq",
                    "earnings_release_next_time",
                    "earnings_publication_type_next_fq",
                ],
            ),
        },
    );
    push_event_if_present(
        events,
        EventEntry {
            event_type: EARNINGS.to_string(),
            event_status: "latest".to_string(),
            date: value_for(values, "earnings_release_date"),
            calendar_date: value_for(values, "earnings_release_calendar_date"),
            trading_date: value_for(values, "earnings_release_trading_date_fq"),
            time: value_for(values, "earnings_release_time"),
            publication_type: value_for(values, "earnings_publication_type_fq"),
            ex_date: None,
            payment_date: None,
            amount: None,
            frequency: None,
            dividend_yield: None,
            expected_annual_dividends: None,
            source_fields: source_fields(
                values,
                &[
                    "earnings_release_date",
                    "earnings_release_calendar_date",
                    "earnings_release_trading_date_fq",
                    "earnings_release_time",
                    "earnings_publication_type_fq",
                ],
            ),
        },
    );
}

fn push_dividend_events(events: &mut Vec<EventEntry>, values: &Value) {
    push_event_if_present(
        events,
        EventEntry {
            event_type: DIVIDENDS.to_string(),
            event_status: "upcoming".to_string(),
            date: value_for(values, "next_dividend_date"),
            calendar_date: None,
            trading_date: None,
            time: None,
            publication_type: None,
            ex_date: value_for(values, "dividend_ex_date_upcoming"),
            payment_date: value_for(values, "dividend_payment_date_upcoming"),
            amount: value_for(values, "dividend_amount_upcoming"),
            frequency: value_for(values, "dividend_frequency_upcoming"),
            dividend_yield: value_for(values, "dividends_yield_current"),
            expected_annual_dividends: value_for(values, "expected_annual_dividends"),
            source_fields: source_fields(
                values,
                &[
                    "next_dividend_date",
                    "dividend_ex_date_upcoming",
                    "dividend_payment_date_upcoming",
                    "dividend_amount_upcoming",
                    "dividend_frequency_upcoming",
                    "dividends_yield_current",
                    "expected_annual_dividends",
                ],
            ),
        },
    );
    push_event_if_present(
        events,
        EventEntry {
            event_type: DIVIDENDS.to_string(),
            event_status: "recent".to_string(),
            date: None,
            calendar_date: None,
            trading_date: None,
            time: None,
            publication_type: None,
            ex_date: value_for(values, "dividend_ex_date_recent"),
            payment_date: value_for(values, "dividend_payment_date_recent"),
            amount: value_for(values, "dividend_amount_recent"),
            frequency: value_for(values, "dividend_frequency_recent"),
            dividend_yield: value_for(values, "dividend_yield_recent"),
            expected_annual_dividends: None,
            source_fields: source_fields(
                values,
                &[
                    "dividend_ex_date_recent",
                    "dividend_payment_date_recent",
                    "dividend_amount_recent",
                    "dividend_frequency_recent",
                    "dividend_yield_recent",
                ],
            ),
        },
    );
}

fn push_event_if_present(events: &mut Vec<EventEntry>, event: EventEntry) {
    if !event.source_fields.is_empty() {
        events.push(event);
    }
}

fn value_for(values: &Value, field: &str) -> Option<Value> {
    values.get(field).filter(|value| !value.is_null()).cloned()
}

fn source_fields(values: &Value, fields: &[&str]) -> Vec<String> {
    fields
        .iter()
        .filter(|field| value_for(values, field).is_some())
        .map(|field| (*field).to_string())
        .collect()
}

fn events_validation_error(message: &str, requested_symbol: &str, event_type: &str) -> AppError {
    AppError::new(ErrorKind::Validation, message).with_details(json!({
        "requested_symbol": requested_symbol,
        "requested_event_type": event_type,
        "source": "scanner_fundamentals_rest",
        "source_category": "desktop_free_read",
        "requires_desktop": false,
        "non_mutating": true,
        "next_action_hint": "Pass a symbol such as NASDAQ:AAPL and optionally --event-type earnings or --event-type dividends.",
    }))
}

fn events_compare_validation_error(message: impl Into<String>) -> AppError {
    AppError::new(ErrorKind::Validation, message.into()).with_details(json!({
        "minimum": 2,
        "maximum": MAX_EVENTS_COMPARE_SYMBOLS,
        "source": "scanner_fundamentals_rest",
        "source_category": "desktop_free_read",
        "requires_desktop": false,
        "non_mutating": true,
        "next_action_hint": "Pass 2 to 25 symbols, such as `tv events compare NASDAQ:AAPL NASDAQ:MSFT`.",
    }))
}

fn add_event_error_details(
    mut error: AppError,
    requested_symbol: &str,
    event_type: &str,
) -> AppError {
    if error.details.as_ref().and_then(Value::as_object).is_none() {
        error.details = Some(json!({}));
    }
    let details = error
        .details
        .as_mut()
        .and_then(Value::as_object_mut)
        .expect("event error details should be an object");
    details
        .entry("requested_symbol")
        .or_insert_with(|| json!(requested_symbol));
    details
        .entry("requested_event_type")
        .or_insert_with(|| json!(event_type));
    details
        .entry("source")
        .or_insert_with(|| json!("scanner_fundamentals_rest"));
    details
        .entry("source_category")
        .or_insert_with(|| json!("desktop_free_read"));
    details
        .entry("requires_desktop")
        .or_insert_with(|| json!(false));
    details.entry("non_mutating").or_insert_with(|| json!(true));
    details.entry("next_action_hint").or_insert_with(|| {
        json!("Use an exchange-qualified symbol such as NASDAQ:AAPL, or verify the symbol with `tv search <SYMBOL>` before retrying `tv events`.")
    });
    error
}

fn sanitized_error_details(err: &AppError) -> Value {
    let mut details = match err.details.as_ref() {
        Some(Value::Object(map)) => sanitize_map(map),
        Some(_) | None => Map::new(),
    };
    details.insert("kind".to_string(), json!(err.kind));
    details.insert("message".to_string(), json!(err.message));
    details
        .entry("source".to_string())
        .or_insert_with(|| json!("scanner_fundamentals_rest"));
    details
        .entry("source_category".to_string())
        .or_insert_with(|| json!("desktop_free_read"));
    details
        .entry("requires_desktop".to_string())
        .or_insert_with(|| json!(false));
    details.entry("non_mutating").or_insert_with(|| json!(true));
    Value::Object(details)
}

fn sanitize_map(map: &Map<String, Value>) -> Map<String, Value> {
    map.iter()
        .filter_map(|(key, value)| {
            if is_private_detail_key(key) {
                return None;
            }
            Some((key.clone(), sanitize_value(value)))
        })
        .collect()
}

fn sanitize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(sanitize_map(map)),
        Value::Array(values) => Value::Array(values.iter().map(sanitize_value).collect()),
        _ => value.clone(),
    }
}

fn is_private_detail_key(key: &str) -> bool {
    matches!(
        key,
        "raw"
            | "raw_payload"
            | "raw_payloads"
            | "raw_response"
            | "target_id"
            | "session_id"
            | "cookie"
            | "authorization"
            | "credential"
            | "credentials"
            | "local_path"
            | "absolute_path"
            | "account_local_metadata"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::types::Fundamentals;

    fn fundamentals(field_values: Value, missing_fields: Vec<&str>) -> Fundamentals {
        let fields = field_values
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        Fundamentals {
            source: "scanner_fundamentals_rest".to_string(),
            source_category: "desktop_free_read".to_string(),
            requires_desktop: false,
            requested_symbol: "AAPL".to_string(),
            symbol: "NASDAQ:AAPL".to_string(),
            observed_symbol: "NASDAQ:AAPL".to_string(),
            market: "america".to_string(),
            fields,
            requested_groups: vec![EARNINGS.to_string(), DIVIDENDS.to_string()],
            field_values,
            missing_fields: missing_fields.into_iter().map(str::to_string).collect(),
            non_mutating: true,
        }
    }

    #[test]
    fn event_groups_match_requested_filter() {
        assert_eq!(event_groups(ALL), vec!["earnings", "dividends"]);
        assert_eq!(event_groups(EARNINGS), vec!["earnings"]);
        assert_eq!(event_groups(DIVIDENDS), vec!["dividends"]);
    }

    #[test]
    fn events_from_fundamentals_shapes_earnings_and_dividends() {
        let payload = fundamentals(
            json!({
                "earnings_release_next_date": 1777852800,
                "earnings_release_next_calendar_date": "2026-05-03",
                "earnings_release_next_trading_date_fq": "2026-05-04",
                "earnings_release_next_time": 1,
                "earnings_publication_type_next_fq": "estimated",
                "earnings_release_date": 1746316800,
                "earnings_release_calendar_date": "2025-05-03",
                "earnings_release_trading_date_fq": "2025-05-05",
                "earnings_release_time": 2,
                "earnings_publication_type_fq": "confirmed",
                "next_dividend_date": "2026-02-10",
                "dividend_ex_date_upcoming": "2026-02-07",
                "dividend_payment_date_upcoming": "2026-02-14",
                "dividend_amount_upcoming": 0.25,
                "dividend_frequency_upcoming": "quarterly",
                "dividends_yield_current": 0.45,
                "expected_annual_dividends": 1.0,
                "dividend_ex_date_recent": "2025-11-07",
                "dividend_payment_date_recent": "2025-11-14",
                "dividend_amount_recent": 0.24,
                "dividend_frequency_recent": "quarterly",
                "dividend_yield_recent": 0.44
            }),
            vec![],
        );

        let events = events_from_fundamentals(payload, ALL);

        assert_eq!(events.contract_version, "events.v1");
        assert_eq!(events.source, "scanner_fundamentals_rest");
        assert_eq!(events.source_category, "desktop_free_read");
        assert!(!events.requires_desktop);
        assert!(events.non_mutating);
        assert_eq!(events.requested_symbol, "AAPL");
        assert_eq!(events.symbol, "NASDAQ:AAPL");
        assert_eq!(events.event_count, 4);
        assert_eq!(events.source_availability.status, "events_present");
        assert_eq!(events.events[0].event_type, "earnings");
        assert_eq!(events.events[0].event_status, "next");
        assert_eq!(events.events[0].date, Some(json!(1777852800)));
        assert_eq!(events.events[0].publication_type, Some(json!("estimated")));
        assert_eq!(events.events[2].event_type, "dividends");
        assert_eq!(events.events[2].event_status, "upcoming");
        assert_eq!(events.events[2].ex_date, Some(json!("2026-02-07")));
        assert_eq!(events.events[2].amount, Some(json!(0.25)));
        assert_eq!(events.events[2].dividend_yield, Some(json!(0.45)));
    }

    #[test]
    fn events_filter_to_single_event_type() {
        let payload = fundamentals(
            json!({
                "earnings_release_next_date": 1777852800,
                "dividend_ex_date_upcoming": "2026-02-07"
            }),
            vec![],
        );

        let events = events_from_fundamentals(payload, EARNINGS);

        assert_eq!(events.requested_event_type, "earnings");
        assert_eq!(events.event_types, vec!["earnings"]);
        assert_eq!(events.event_count, 1);
        assert_eq!(events.events[0].event_type, "earnings");
    }

    #[test]
    fn events_empty_when_fields_are_unavailable() {
        let payload = fundamentals(
            json!({
                "earnings_release_next_date": null,
                "dividend_ex_date_upcoming": null
            }),
            vec!["earnings_release_next_time"],
        );

        let events = events_from_fundamentals(payload, ALL);

        assert_eq!(events.event_count, 0);
        assert!(events.events.is_empty());
        assert_eq!(events.source_availability.status, "no_events_returned");
        assert!(
            events
                .field_readback
                .unavailable_fields
                .contains(&"earnings_release_next_date".to_string())
        );
        assert!(
            events
                .field_readback
                .missing_fields
                .contains(&"earnings_release_next_time".to_string())
        );
    }

    #[test]
    fn events_compare_from_items_summarizes_ordered_results() {
        let events_with_entries = events_from_fundamentals(
            fundamentals(
                json!({
                    "earnings_release_next_date": 1777852800,
                    "dividend_ex_date_upcoming": "2026-02-07"
                }),
                vec![],
            ),
            ALL,
        );
        let events_without_entries = events_from_fundamentals(
            fundamentals(
                json!({
                    "earnings_release_next_date": null,
                    "dividend_ex_date_upcoming": null
                }),
                vec![],
            ),
            ALL,
        );
        let items = vec![
            EventsCompareItem {
                requested_index: 0,
                requested_symbol: "NASDAQ:AAPL".to_string(),
                status: "ok".to_string(),
                events: Some(events_with_entries),
                failure_details: None,
            },
            EventsCompareItem {
                requested_index: 1,
                requested_symbol: "NASDAQ:MSFT".to_string(),
                status: "ok".to_string(),
                events: Some(events_without_entries),
                failure_details: None,
            },
        ];

        let compare = events_compare_from_items(
            vec!["NASDAQ:AAPL".to_string(), "NASDAQ:MSFT".to_string()],
            ALL,
            items,
        );

        assert_eq!(compare.contract_version, "events_compare.v1");
        assert_eq!(compare.source, "scanner_fundamentals_rest");
        assert_eq!(compare.source_category, "desktop_free_read");
        assert_eq!(
            compare.requested_symbols,
            vec!["NASDAQ:AAPL", "NASDAQ:MSFT"]
        );
        assert_eq!(compare.summary.requested_count, 2);
        assert_eq!(compare.summary.ok_count, 2);
        assert_eq!(compare.summary.error_count, 0);
        assert_eq!(compare.summary.total_event_count, 2);
        assert_eq!(compare.summary.symbols_with_events_count, 1);
        assert_eq!(compare.summary.symbols_without_events_count, 1);
        assert_eq!(compare.items[0].requested_index, 0);
        assert_eq!(compare.items[1].requested_index, 1);
    }

    #[test]
    fn events_compare_validation_is_public_safe() {
        let too_many = normalize_compare_symbols((0..26).map(|idx| format!("SYM{idx}")).collect())
            .unwrap_err();
        assert_eq!(too_many.kind, ErrorKind::Validation);
        let details = too_many.details.unwrap();
        assert_eq!(details["maximum"], 25);
        assert_eq!(details["source"], "scanner_fundamentals_rest");
        assert_eq!(details["requires_desktop"], false);
        assert_eq!(details["non_mutating"], true);

        let blank = normalize_compare_symbols(vec!["NASDAQ:AAPL".to_string(), " ".to_string()])
            .unwrap_err();
        assert_eq!(blank.kind, ErrorKind::Validation);
    }

    #[test]
    fn events_compare_failure_details_are_sanitized() {
        let err = AppError::new(ErrorKind::InternalApiUnavailable, "failed").with_details(json!({
            "raw": {"hidden": true},
            "raw_response": "secret",
            "nested": {"session_id": "abc", "safe": true},
        }));

        let details = sanitized_error_details(&err);

        assert!(details.get("raw").is_none());
        assert!(details.get("raw_response").is_none());
        assert!(details["nested"].get("session_id").is_none());
        assert_eq!(details["nested"]["safe"], true);
        assert_eq!(details["source"], "scanner_fundamentals_rest");
        assert_eq!(details["source_category"], "desktop_free_read");
    }

    #[test]
    fn event_type_validation_is_public_safe() {
        let error = normalize_event_type("calendar").unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        let details = error.details.unwrap();
        assert_eq!(details["source"], "scanner_fundamentals_rest");
        assert_eq!(details["requires_desktop"], false);
        assert_eq!(details["non_mutating"], true);
        assert!(
            details["supported_event_types"]
                .as_array()
                .unwrap()
                .contains(&json!("earnings"))
        );
    }
}
