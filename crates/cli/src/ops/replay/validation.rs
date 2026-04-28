use serde_json::json;

use tradingview_core::{AppError, ErrorKind};

const VALID_AUTOPLAY_DELAYS: [u64; 9] = [100, 143, 200, 300, 1000, 2000, 3000, 5000, 10000];

pub fn validate_replay_date(date: &str) -> Result<(), AppError> {
    parse_replay_date_ms(date).map(|_| ())
}

pub fn validate_replay_autoplay_speed(speed: u64) -> Result<(), AppError> {
    if speed == 0 || VALID_AUTOPLAY_DELAYS.contains(&speed) {
        return Ok(());
    }

    Err(AppError::new(
        ErrorKind::Validation,
        format!(
            "Invalid replay autoplay delay: {speed}ms. Use 0 or one of: {}.",
            VALID_AUTOPLAY_DELAYS
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
    .with_details(json!({
        "speed": speed,
        "supported": VALID_AUTOPLAY_DELAYS,
    })))
}

pub fn validate_replay_trade_action(action: &str) -> Result<(), AppError> {
    match action {
        "buy" | "sell" | "close" => Ok(()),
        _ => Err(AppError::new(
            ErrorKind::Validation,
            "Invalid replay trade action. Use buy, sell, or close.",
        )
        .with_details(json!({
            "action": action,
            "supported": ["buy", "sell", "close"],
        }))),
    }
}

pub(super) fn parse_replay_date_ms(date: &str) -> Result<i64, AppError> {
    let trimmed = date.trim();
    let parts = trimmed.split('-').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(invalid_replay_date(date));
    }

    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| invalid_replay_date(date))?;
    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| invalid_replay_date(date))?;
    let day = parts[2]
        .parse::<u32>()
        .map_err(|_| invalid_replay_date(date))?;
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return Err(invalid_replay_date(date));
    }

    let days = days_from_civil(year, month, day);
    Ok(days * 86_400_000)
}

fn invalid_replay_date(date: &str) -> AppError {
    AppError::new(
        ErrorKind::Validation,
        format!("Invalid replay date: {date}. Use YYYY-MM-DD."),
    )
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = (year - era * 400) as u32;
    let month = month as i32;
    let day = day as i32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era =
        year_of_era as i32 * 365 + year_of_era as i32 / 4 - year_of_era as i32 / 100 + day_of_year;
    (era * 146_097 + day_of_era - 719_468) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_start_rejects_invalid_date_before_evaluating() {
        let error = validate_replay_date("2026-02-31").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[test]
    fn replay_autoplay_rejects_invalid_speed_before_evaluating() {
        let error = validate_replay_autoplay_speed(123).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("Invalid replay autoplay delay"));
    }

    #[test]
    fn replay_autoplay_accepts_known_delays() {
        for delay in [0, 100, 143, 200, 300, 1000, 2000, 3000, 5000, 10000] {
            assert!(validate_replay_autoplay_speed(delay).is_ok());
        }
    }

    #[test]
    fn replay_trade_rejects_invalid_action_before_evaluating() {
        let error = validate_replay_trade_action("hold").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[test]
    fn replay_trade_accepts_supported_actions() {
        for action in ["buy", "sell", "close"] {
            assert!(validate_replay_trade_action(action).is_ok());
        }
    }
}
