const PARENT_EVENT_MAX_BYTES: u64 = 32 * 1024 * 1024;
const PARENT_EVENT_MAX_ROWS: usize = 20_000;
const PARENT_EVENT_TEXT_MAX_BYTES: usize = 4 * 1024;

#[derive(Debug, Clone)]
struct ParentEventEvidence {
    count: u64,
    digest: String,
    first_timestamp_unix_ms: Option<i64>,
    last_timestamp_unix_ms: Option<i64>,
}

fn read_parent_event_evidence(
    path: &Path,
    recording_started_at_unix_ms: i64,
    recording_ended_at_unix_ms: Option<i64>,
) -> Result<ParentEventEvidence> {
    let bytes = read_bounded_bytes(
        path,
        "parent-operation Record & Replay events",
        PARENT_EVENT_MAX_BYTES,
    )?;
    let digest = sha256_digest(&bytes);
    let text = std::str::from_utf8(&bytes).context("Record & Replay events must be UTF-8 JSONL")?;
    let mut count = 0_u64;
    let mut first_timestamp_unix_ms = None;
    let mut last_timestamp_unix_ms = None;
    for (index, line) in text.lines().enumerate() {
        if index >= PARENT_EVENT_MAX_ROWS {
            bail!(
                "parent-operation Record & Replay events exceed {PARENT_EVENT_MAX_ROWS} row limit"
            );
        }
        let event: Value = serde_json::from_str(line).with_context(|| {
            format!("Record & Replay event line {} is malformed JSON", index + 1)
        })?;
        if !event.is_object() {
            bail!(
                "Record & Replay event line {} must be a JSON object",
                index + 1
            );
        }
        let kind = event
            .get("kind")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("Record & Replay event line {} missing kind", index + 1)
            })?;
        if kind.len() > PARENT_EVENT_TEXT_MAX_BYTES {
            bail!(
                "Record & Replay event line {} kind exceeds byte limit",
                index + 1
            );
        }
        let timestamp = event
            .get("timestamp")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("Record & Replay event line {} missing timestamp", index + 1)
            })?;
        let unix_ms = parse_utc_timestamp_ms(timestamp)
            .with_context(|| format!("Record & Replay event line {} timestamp", index + 1))?;
        if unix_ms < recording_started_at_unix_ms {
            bail!(
                "Record & Replay event line {} predates recording metadata",
                index + 1
            );
        }
        if recording_ended_at_unix_ms.is_some_and(|ended_at| unix_ms > ended_at) {
            bail!(
                "Record & Replay event line {} follows recording metadata end",
                index + 1
            );
        }
        first_timestamp_unix_ms.get_or_insert(unix_ms);
        last_timestamp_unix_ms = Some(unix_ms);
        count = count
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("event count overflow"))?;
    }
    Ok(ParentEventEvidence {
        count,
        digest,
        first_timestamp_unix_ms,
        last_timestamp_unix_ms,
    })
}
