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
