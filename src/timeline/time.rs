use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn format_offset(ms: u64) -> String {
    let total_seconds = ms / 1_000;
    format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60)
}

pub fn parse_utc_millis(value: &str) -> Option<i64> {
    let body = value.strip_suffix('Z')?;
    let (seconds, fraction) = match body.split_once('.') {
        Some((seconds, fraction)) => {
            if fraction.is_empty()
                || fraction.len() > 9
                || !fraction.bytes().all(|byte| byte.is_ascii_digit())
            {
                return None;
            }
            (seconds, Some(fraction))
        }
        None => (body, None),
    };
    if seconds.len() != 19 {
        return None;
    }
    let bytes = seconds.as_bytes();
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }

    let year = digits(&seconds[0..4])?;
    let month = digits(&seconds[5..7])?;
    let day = digits(&seconds[8..10])?;
    let hour = digits(&seconds[11..13])?;
    let minute = digits(&seconds[14..16])?;
    let second = digits(&seconds[17..19])?;
    if !(1..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    let millis = fraction.map(fraction_millis).unwrap_or(0);
    days_from_civil(year, month, day)?
        .checked_mul(86_400)?
        .checked_add(hour.checked_mul(3_600)?)?
        .checked_add(minute.checked_mul(60)?)?
        .checked_add(second)?
        .checked_mul(1_000)?
        .checked_add(millis)
}

fn digits(value: &str) -> Option<i64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn fraction_millis(value: &str) -> i64 {
    let mut millis = 0_i64;
    for (index, byte) in value.bytes().take(3).enumerate() {
        millis += i64::from(byte - b'0') * [100, 10, 1][index];
    }
    millis
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
    let year = year.checked_sub(i64::from(month <= 2))?;
    let era = if year >= 0 {
        year
    } else {
        year.checked_sub(399)?
    } / 400;
    let yoe = year.checked_sub(era.checked_mul(400)?)?;
    let month_prime = month.checked_add(if month > 2 { -3 } else { 9 })?;
    let doy = 153_i64
        .checked_mul(month_prime)?
        .checked_add(2)?
        .checked_div(5)?
        .checked_add(day)?
        .checked_sub(1)?;
    let doe = yoe
        .checked_mul(365)?
        .checked_add(yoe / 4)?
        .checked_sub(yoe / 100)?
        .checked_add(doy)?;
    era.checked_mul(146_097)?
        .checked_add(doe)?
        .checked_sub(719_468)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_utc_parser_accepts_real_dates_and_fractional_seconds() {
        assert_eq!(
            parse_utc_millis("2026-06-20T01:32:03.456789Z"),
            Some(1_781_919_123_456)
        );
        assert!(parse_utc_millis("2024-02-29T23:59:59Z").is_some());
    }

    #[test]
    fn strict_utc_parser_rejects_impossible_or_surplus_components() {
        for value in [
            "2026-02-29T01:32:03Z",
            "2026-02-31T01:32:03Z",
            "2026-13-01T01:32:03Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T23:60:00Z",
            "2026-01-01T23:59:60Z",
            "2026-01-01T23:59:59:999Z",
            "9223372036854775807-01-01T00:00:00Z",
        ] {
            assert_eq!(parse_utc_millis(value), None, "{value}");
        }
    }
}
