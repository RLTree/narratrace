use super::agent_text::AgentTranscriptEvidence;
use super::alignment_authority::verified_final_alignment_bytes;
use super::ingestion;
use super::time::now_unix_ms;
use crate::private_fs::append_private;
use crate::safe_path::regular_file_metadata;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const MAX_AGENT_ALIGNMENT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct TranscriptSegment {
    pub id: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub monotonic_offset_ms: Option<u64>,
    pub timing_source: String,
    pub text: String,
}

impl TranscriptSegment {
    pub fn to_json(&self) -> Value {
        let text = AgentTranscriptEvidence::from_untrusted(&self.text);
        json!({
            "id": self.id,
            "startMs": self.start_ms,
            "endMs": self.end_ms,
            "monotonicOffsetMs": self.monotonic_offset_ms,
            "timingSource": self.timing_source,
            "text": text.rendered(),
            "textBoundary": text.boundary(),
        })
    }
}

pub fn record_transcript_event_with_monotonic_offset(
    session_dir: &Path,
    event: &Value,
    kind: &str,
    text: &str,
    monotonic_offset_ms: Option<u64>,
) -> Result<()> {
    let observed_at_unix_ms = now_unix_ms();
    let audio_started_at_unix_ms =
        read_audio_started_at(session_dir).unwrap_or(observed_at_unix_ms);
    append_json_line(
        session_dir.join("transcript.timeline.jsonl"),
        &json!({
            "schema": "narrated-record-replay.transcript-event.v1",
            "kind": kind,
            "observedAtUnixMs": observed_at_unix_ms,
            "audioOffsetMs": observed_at_unix_ms.saturating_sub(audio_started_at_unix_ms),
            "monotonicOffsetMs": monotonic_offset_ms,
            "eventId": event.get("event_id").or_else(|| event.get("id")),
            "itemId": event.get("item_id"),
            "text": text,
        }),
    )
}

pub fn transcript_segments(session_dir: &Path) -> Vec<TranscriptSegment> {
    transcript_segments_checked(session_dir).unwrap_or_default()
}

pub fn raw_realtime_segments(session_dir: &Path) -> Vec<TranscriptSegment> {
    raw_realtime_segments_checked(session_dir).unwrap_or_default()
}

pub(super) fn transcript_segments_checked(session_dir: &Path) -> Result<Vec<TranscriptSegment>> {
    Ok(match final_aligned_segments(session_dir)? {
        Some(segments) => segments,
        None => raw_realtime_segments_checked(session_dir)?,
    })
}

pub(super) fn raw_realtime_segments_checked(session_dir: &Path) -> Result<Vec<TranscriptSegment>> {
    let events = read_json_lines(&session_dir.join("transcript.timeline.jsonl"))?;
    let mut segments = Vec::new();
    let mut previous_end = 0_u64;
    let mut deltas = String::new();
    let mut last_delta_offset = 0_u64;
    let mut last_delta_monotonic_offset = None;
    let mut last_delta_timing_source = "audio-wall-clock-offset".to_string();

    for event in events {
        let offset = event
            .get("monotonicOffsetMs")
            .and_then(Value::as_u64)
            .or_else(|| event.get("audioOffsetMs").and_then(Value::as_u64))
            .unwrap_or(0);
        let monotonic_offset_ms = event.get("monotonicOffsetMs").and_then(Value::as_u64);
        let timing_source = if monotonic_offset_ms.is_some() {
            "process-local-monotonic-offset"
        } else {
            "audio-wall-clock-offset"
        };
        let text = event.get("text").and_then(Value::as_str).unwrap_or("");
        ingestion::enforce_text(
            text,
            "transcript text",
            ingestion::TRANSCRIPT_TEXT_MAX_BYTES,
        )?;
        match event.get("kind").and_then(Value::as_str) {
            Some("completed") if !text.trim().is_empty() => {
                segments.push(TranscriptSegment {
                    id: segments.len() + 1,
                    start_ms: previous_end,
                    end_ms: offset.max(previous_end),
                    monotonic_offset_ms,
                    timing_source: timing_source.to_string(),
                    text: text.trim().to_string(),
                });
                previous_end = offset.max(previous_end);
                deltas.clear();
            }
            Some("delta") => {
                if deltas.len().saturating_add(text.len()) > ingestion::TRANSCRIPT_TEXT_MAX_BYTES {
                    anyhow::bail!(
                        "transcript text exceeds {} byte limit",
                        ingestion::TRANSCRIPT_TEXT_MAX_BYTES
                    );
                }
                deltas.push_str(text);
                last_delta_offset = offset;
                last_delta_monotonic_offset = monotonic_offset_ms;
                last_delta_timing_source = timing_source.to_string();
            }
            _ => {}
        }
    }

    if segments.is_empty() && !deltas.trim().is_empty() {
        segments.push(TranscriptSegment {
            id: 1,
            start_ms: 0,
            end_ms: last_delta_offset,
            monotonic_offset_ms: last_delta_monotonic_offset,
            timing_source: last_delta_timing_source,
            text: deltas.trim().to_string(),
        });
    }
    Ok(segments)
}

fn final_aligned_segments(session_dir: &Path) -> Result<Option<Vec<TranscriptSegment>>> {
    let Some(bytes) = verified_final_alignment_bytes(session_dir) else {
        return Ok(None);
    };
    if bytes.len() as u64 > MAX_AGENT_ALIGNMENT_BYTES {
        anyhow::bail!(
            "alignment artifact exceeds {} byte limit",
            MAX_AGENT_ALIGNMENT_BYTES
        );
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    let Some(segments) = value.get("segments").and_then(Value::as_array) else {
        return Ok(None);
    };
    if segments.len() > ingestion::MAX_ALIGNMENT_SEGMENTS {
        anyhow::bail!(
            "alignment artifact exceeds {} segment limit",
            ingestion::MAX_ALIGNMENT_SEGMENTS
        );
    }
    let parsed = segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| {
            let start_ms = segment.get("startMs")?.as_u64()?;
            let end_ms = segment.get("endMs")?.as_u64()?;
            let text = segment.get("text")?.as_str()?;
            Some((index, start_ms, end_ms, text))
        })
        .map(
            |(index, start_ms, end_ms, text)| -> Result<TranscriptSegment> {
                ingestion::enforce_text(
                    text,
                    "alignment transcript text",
                    ingestion::TRANSCRIPT_TEXT_MAX_BYTES,
                )?;
                Ok(TranscriptSegment {
                    id: index + 1,
                    start_ms,
                    end_ms,
                    monotonic_offset_ms: Some(end_ms),
                    timing_source: "aligned-cleaned-batch-text-with-realtime-window".to_string(),
                    text: text.to_string(),
                })
            },
        )
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(parsed))
}

pub fn read_audio_started_at(session_dir: &Path) -> Option<u64> {
    read_json(session_dir.join("capture-clock.json"))
        .ok()?
        .get("audioStartedAtUnixMs")?
        .as_u64()
}

fn read_json(path: PathBuf) -> Result<Value> {
    let text = ingestion::read_bounded(
        &path,
        "capture clock artifact",
        ingestion::METADATA_MAX_BYTES,
    )?;
    Ok(serde_json::from_str(&text)?)
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>> {
    if regular_file_metadata(path).is_err() {
        return Ok(Vec::new());
    }
    let text =
        ingestion::read_bounded(path, "transcript artifact", ingestion::TRANSCRIPT_MAX_BYTES)?;
    ingestion::enforce_rows(&text, "transcript artifact", ingestion::TRANSCRIPT_MAX_ROWS)?;
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str::<Value>(line).map_err(|_| {
                anyhow::anyhow!(
                    "transcript artifact contains malformed JSONL at row {}",
                    index + 1
                )
            })
        })
        .collect()
}

fn append_json_line(path: impl AsRef<Path>, value: &Value) -> Result<()> {
    append_private(path, &format!("{}\n", serde_json::to_string(value)?))
}
