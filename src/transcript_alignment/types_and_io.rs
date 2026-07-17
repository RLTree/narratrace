use crate::private_fs::write_private;
use crate::redaction::redact_text;
use crate::timeline;
use anyhow::Result;
use serde_json::{Value, json};
#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
const MAX_CLEANED_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_FINAL_ALIGNMENT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct FinalSegment {
    pub id: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub confidence: f64,
    pub source_realtime_ids: Vec<usize>,
    pub mismatch: Option<String>,
}

#[derive(Debug, Clone)]
struct CleanedUtterance {
    marker_label: Option<String>,
    text: String,
    token_start: usize,
    token_end: usize,
}

#[derive(Debug, Clone)]
struct RealtimeToken {
    text: String,
    segment_index: usize,
}

const MARKER_LABELS: [&str; 26] = [
    "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india", "juliet",
    "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra", "tango",
    "uniform", "victor", "whiskey", "xray", "yankee", "zulu",
];

#[allow(dead_code)]
pub fn final_segments(session_dir: &Path) -> Option<Vec<FinalSegment>> {
    let value: Value = serde_json::from_slice(&authoritative_alignment_bytes(session_dir)?).ok()?;
    let segments = value.get("segments")?.as_array()?;
    Some(
        segments
            .iter()
            .enumerate()
            .filter_map(|(index, segment)| {
                Some(FinalSegment {
                    id: index + 1,
                    start_ms: segment.get("startMs")?.as_u64()?,
                    end_ms: segment.get("endMs")?.as_u64()?,
                    text: segment.get("text")?.as_str()?.to_string(),
                    confidence: segment
                        .get("confidence")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                    source_realtime_ids: segment
                        .get("sourceRealtimeSegmentIds")
                        .and_then(Value::as_array)
                        .map(|ids| {
                            ids.iter()
                                .filter_map(Value::as_u64)
                                .map(|id| id as usize)
                                .collect()
                        })
                        .unwrap_or_default(),
                    mismatch: segment
                        .get("mismatch")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                })
            })
            .collect(),
    )
}

pub fn align_cleaned_text(
    cleaned_text: &str,
    realtime_segments: &[timeline::TranscriptSegment],
) -> Result<Vec<FinalSegment>> {
    enforce_text_limits(cleaned_text, realtime_segments)?;
    if realtime_segments.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(token_segments) = align_cleaned_utterances(cleaned_text, realtime_segments)? {
        return Ok(token_segments);
    }
    let total_realtime_words: usize = realtime_segments
        .iter()
        .map(|segment| semantic_word_count(&segment.text))
        .sum::<usize>()
        .max(1);
    let cleaned_words = split_words(cleaned_text);
    let mut cursor = 0_usize;
    let mut out = Vec::new();
    for segment in realtime_segments {
        let realtime_words = semantic_word_count(&segment.text).max(1);
        let remaining_segments = realtime_segments.len().saturating_sub(segment.id);
        let mut take = ((cleaned_words.len() * realtime_words) + (total_realtime_words / 2))
            / total_realtime_words;
        if remaining_segments == 0 {
            take = cleaned_words.len().saturating_sub(cursor);
        } else {
            take = choose_take(
                &cleaned_words,
                cursor,
                take.max(1),
                remaining_segments,
                &segment.text,
            );
        }
        take = take.max(1).min(cleaned_words.len().saturating_sub(cursor));
        let text = cleaned_words[cursor..cursor + take].join(" ");
        cursor += take;
        let confidence = phrase_similarity(&segment.text, &text);
        let threshold = if realtime_words <= 3 { 0.25 } else { 0.35 };
        let mismatch = if confidence < threshold {
            Some("low-token-similarity".to_string())
        } else {
            None
        };
        out.push(FinalSegment {
            id: out.len() + 1,
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            text,
            confidence,
            source_realtime_ids: vec![segment.id],
            mismatch,
        });
    }
    if cursor < cleaned_words.len() {
        let last = realtime_segments.last().unwrap();
        let text = cleaned_words[cursor..].join(" ");
        out.push(FinalSegment {
            id: out.len() + 1,
            start_ms: last.end_ms,
            end_ms: last.end_ms,
            text,
            confidence: 0.0,
            source_realtime_ids: vec![last.id],
            mismatch: Some("cleaned-trailing-words-without-realtime-window".to_string()),
        });
    }
    Ok(out)
}
