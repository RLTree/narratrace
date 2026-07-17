const PARENT_JSON_MAX_BYTES: u64 = 1024 * 1024;

fn read_json(path: &Path) -> Result<Value> {
    Ok(read_json_with_digest(path)?.0)
}

fn read_json_with_digest(path: &Path) -> Result<(Value, String)> {
    let bytes = read_bounded_bytes(
        path,
        "parent-operation JSON artifact",
        PARENT_JSON_MAX_BYTES,
    )?;
    let digest = sha256_digest(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

fn read_bounded_bytes(path: &Path, label: &str, max_bytes: u64) -> Result<Vec<u8>> {
    let mut file = open_regular_file(path)?;
    let metadata = file.metadata()?;
    if metadata.len() > max_bytes {
        bail!("{label} exceeds {max_bytes} byte limit");
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref().take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        bail!("{label} exceeds {max_bytes} byte limit");
    }
    Ok(bytes)
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn completed_transcript_segments_from_drain(post_commit_drain: &Value) -> Option<u64> {
    [
        post_commit_drain.get("completedSegments"),
        post_commit_drain.pointer("/captureStats/realtimeCompletedSegmentsObserved"),
    ]
    .into_iter()
    .flatten()
    .filter_map(Value::as_u64)
    .max()
}

fn parse_utc_timestamp_ms(value: &str) -> Result<i64> {
    parse_strict_utc_millis(value)
        .ok_or_else(|| anyhow::anyhow!("timestamp must be strict UTC RFC3339 with a real date"))
}

fn parse_strict_utc_millis(value: &str) -> Option<i64> {
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
    let year = timestamp_digits(&seconds[0..4])?;
    let month = timestamp_digits(&seconds[5..7])?;
    let day = timestamp_digits(&seconds[8..10])?;
    let hour = timestamp_digits(&seconds[11..13])?;
    let minute = timestamp_digits(&seconds[14..16])?;
    let second = timestamp_digits(&seconds[17..19])?;
    if !(1..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > timestamp_days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let millis = fraction.map(timestamp_fraction_millis).unwrap_or(0);
    timestamp_days_from_civil(year, month, day)?
        .checked_mul(86_400)?
        .checked_add(hour.checked_mul(3_600)?)?
        .checked_add(minute.checked_mul(60)?)?
        .checked_add(second)?
        .checked_mul(1_000)?
        .checked_add(millis)
}

fn timestamp_digits(value: &str) -> Option<i64> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())?
}

fn timestamp_fraction_millis(value: &str) -> i64 {
    let mut millis = 0_i64;
    for (index, byte) in value.bytes().take(3).enumerate() {
        millis += i64::from(byte - b'0') * [100, 10, 1][index];
    }
    millis
}

fn timestamp_days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn timestamp_days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
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
