use super::ingestion;
use super::time::parse_utc_millis;
use crate::redaction::redact_text;
use crate::safe_path::regular_file_metadata;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::path::Path;

#[derive(Clone, Debug)]
pub(super) struct RnrEvent {
    pub(super) id: Option<i64>,
    pub(super) kind: String,
    pub(super) timestamp: Option<String>,
    pub(super) unix_ms: Option<i64>,
    pub(super) timestamp_parse_status: String,
    pub(super) app: Option<String>,
    pub(super) window: Option<String>,
    pub(super) ui_hint: Option<String>,
}

impl RnrEvent {
    pub(super) fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "kind": self.kind,
            "timestamp": self.timestamp,
            "unixMs": self.unix_ms,
            "timestampParseStatus": self.timestamp_parse_status,
            "app": self.app,
            "window": self.window,
            "uiHint": self.ui_hint,
        })
    }
}

pub(super) fn read_rnr_events(path: &Path) -> Result<Vec<RnrEvent>> {
    read_json_lines(path)?
        .into_iter()
        .map(|event| -> Result<RnrEvent> {
            let raw_timestamp = event
                .get("timestamp")
                .and_then(Value::as_str)
                .map(str::to_string);
            if let Some(value) = raw_timestamp.as_deref() {
                ingestion::enforce_text(value, "event timestamp", ingestion::EVENT_TEXT_MAX_BYTES)?;
            }
            let (timestamp, unix_ms, timestamp_parse_status) = match raw_timestamp.as_deref() {
                None => (None, None, "missing"),
                Some(value) => match parse_utc_millis(value) {
                    Some(unix_ms) => (Some(value.to_string()), Some(unix_ms), "valid"),
                    None => (None, None, "invalid"),
                },
            };
            Ok(RnrEvent {
                id: event.get("id").and_then(Value::as_i64),
                kind: safe_agent_field(
                    event
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    "event kind",
                )?,
                unix_ms,
                timestamp_parse_status: timestamp_parse_status.to_string(),
                timestamp,
                app: event
                    .pointer("/app/name")
                    .and_then(Value::as_str)
                    .map(|value| safe_agent_field(value, "event app"))
                    .transpose()?,
                window: event
                    .pointer("/window/title")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(|value| safe_agent_field(value, "event window"))
                    .transpose()?,
                ui_hint: ui_hint(&event)?,
            })
        })
        .collect()
}

fn ui_hint(event: &Value) -> Result<Option<String>> {
    let Some(value) = event
        .pointer("/selection/selectedText")
        .and_then(Value::as_str)
        .or_else(|| {
            event
                .pointer("/selection/target/value")
                .and_then(Value::as_str)
        })
        .or_else(|| event.pointer("/ax/mode").and_then(Value::as_str))
    else {
        return Ok(None);
    };
    ingestion::enforce_text(value, "event UI", ingestion::EVENT_TEXT_MAX_BYTES)?;
    Ok(Some(safe_agent_field(&truncate(value, 240), "event UI")?))
}

fn safe_agent_field(value: &str, label: &str) -> Result<String> {
    ingestion::enforce_text(value, label, ingestion::EVENT_TEXT_MAX_BYTES)?;
    let redacted = redact_text(value);
    let mut out = String::with_capacity(redacted.len());
    let mut needs_space = false;
    for ch in redacted.chars() {
        if ch.is_whitespace() {
            needs_space = !out.is_empty();
            continue;
        }
        if needs_space {
            out.push(' ');
            needs_space = false;
        }
        if matches!(
            ch,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | '#'
                | '|'
                | '!'
                | '('
                | ')'
                | '-'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    Ok(out)
}

fn truncate(value: &str, max: usize) -> String {
    let mut out = String::new();
    for ch in value.chars().take(max) {
        out.push(ch);
    }
    if value.chars().count() > max {
        out.push_str("...");
    }
    out
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>> {
    if regular_file_metadata(path).is_err() {
        return Ok(Vec::new());
    }
    let text =
        ingestion::read_bounded(path, "recording event artifact", ingestion::EVENT_MAX_BYTES)?;
    ingestion::enforce_rows(&text, "recording event artifact", ingestion::EVENT_MAX_ROWS)?;
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            if line.trim().is_empty() {
                bail!("recording event artifact line {} is empty", index + 1);
            }
            let value: Value = serde_json::from_str(line).with_context(|| {
                format!(
                    "recording event artifact line {} is malformed JSON",
                    index + 1
                )
            })?;
            if !value.is_object() {
                bail!(
                    "recording event artifact line {} must be a JSON object",
                    index + 1
                );
            }
            Ok(value)
        })
        .collect()
}
