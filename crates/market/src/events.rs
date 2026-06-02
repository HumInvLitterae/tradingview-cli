use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{
    fundamentals_symbol_with_groups_typed,
    types::{EventEntry, EventFieldReadback, EventSourceAvailability, Events, Fundamentals},
};

const EVENTS_CONTRACT_VERSION: &str = "events.v1";
const EARNINGS: &str = "earnings";
const DIVIDENDS: &str = "dividends";
const ALL: &str = "all";

pub async fn events_symbol(symbol: &str, event_type: &str) -> Result<Value, AppError> {
    serde_json::to_value(events_symbol_typed(symbol, event_type).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads scanner-backed event-like fundamentals fields for one symbol.
///
/// This is an event-shaped view over scanner fundamentals fields. It does not
/// infer timezone/session semantics and does not read a standalone event
/// calendar source.
pub async fn events_symbol_typed(symbol: &str, event_type: &str) -> Result<Events, AppError> {
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
    let fundamentals =
        fundamentals_symbol_with_groups_typed(requested_symbol, groups.clone(), Vec::new())
            .await
            .map_err(|err| add_event_error_details(err, requested_symbol, requested_event_type))?;

    Ok(events_from_fundamentals(fundamentals, requested_event_type))
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
