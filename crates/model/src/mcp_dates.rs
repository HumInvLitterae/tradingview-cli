//! Shared exact date syntax for MCP request boundaries.

pub(crate) fn iso_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = date[..4].parse().unwrap();
    let month: u32 = date[5..7].parse().unwrap();
    let day: u32 = date[8..].parse().unwrap();
    let max = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > max {
        return false;
    }
    true
}

pub(crate) fn utc_seconds(value: &str) -> bool {
    let b = value.as_bytes();
    if b.len() != 20
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
        || !b[..19]
            .iter()
            .enumerate()
            .all(|(i, b)| matches!(i, 4 | 7 | 10 | 13 | 16) || b.is_ascii_digit())
    {
        return false;
    }
    iso_date(&value[..10])
        && value[11..13].parse::<u32>().is_ok_and(|v| v < 24)
        && value[14..16].parse::<u32>().is_ok_and(|v| v < 60)
        && value[17..19].parse::<u32>().is_ok_and(|v| v < 60)
}

pub(crate) fn utc_seconds_to_unix(value: &str) -> Option<i64> {
    if !utc_seconds(value) {
        return None;
    }
    let year = value[..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<usize>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let leap_days = |year: i64| year / 4 - year / 100 + year / 400;
    let month_days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap = month > 2 && year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = (year - 1970) * 365 + leap_days(year - 1) - leap_days(1969)
        + month_days[month - 1]
        + i64::from(leap)
        + day
        - 1;
    Some(
        days * 86400
            + value[11..13].parse::<i64>().ok()? * 3600
            + value[14..16].parse::<i64>().ok()? * 60
            + value[17..19].parse::<i64>().ok()?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_timestamps_preserve_utc_seconds_and_validate_calendar() {
        for (text, expected) in [
            ("1970-01-01T00:00:00Z", 0),
            ("1969-12-31T23:59:59Z", -1),
            ("2000-02-29T12:34:56Z", 951827696),
            ("2024-03-01T00:00:00Z", 1709251200),
        ] {
            assert_eq!(utc_seconds_to_unix(text), Some(expected));
        }
        for text in [
            "1900-02-29T00:00:00Z",
            "2024-02-30T00:00:00Z",
            "2024-01-01T24:00:00Z",
            "2024-01-01T00:00:00",
            "2024-01-01T00:00:00.123Z",
        ] {
            assert_eq!(utc_seconds_to_unix(text), None);
        }
    }
}
