use super::agent_text::AgentTranscriptEvidence;
use super::record_replay::RnrEvent;
use super::transcript::TranscriptSegment;
use super::{ALIGNMENT_WINDOW_MS, CLOCK_SKEW_WARNING_MS, HIGH_CONFIDENCE_MS, MEDIUM_CONFIDENCE_MS};
use serde_json::{Value, json};

pub(super) fn align_segments(
    segments: &[TranscriptSegment],
    events: &[RnrEvent],
    audio_started_at_unix_ms: Option<u64>,
) -> Vec<Value> {
    let Some(audio_start) = audio_started_at_unix_ms else {
        return Vec::new();
    };
    segments
        .iter()
        .filter_map(|segment| {
            let transcript = AgentTranscriptEvidence::from_untrusted(&segment.text);
            let midpoint = audio_start as i64 + ((segment.start_ms + segment.end_ms) / 2) as i64;
            let nearby = events
                .iter()
                .filter_map(|event| {
                    let delta_ms = event.unix_ms? - midpoint;
                    if delta_ms.abs() <= ALIGNMENT_WINDOW_MS {
                        Some(json!({
                            "eventId": event.id,
                            "kind": event.kind,
                            "deltaMs": delta_ms,
                            "alignmentConfidence": alignment_confidence(delta_ms),
                            "app": event.app,
                            "window": event.window,
                            "uiHint": event.ui_hint,
                        }))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if nearby.is_empty() {
                None
            } else {
                Some(json!({
                    "transcriptSegmentId": segment.id,
                    "transcriptWindowMs": [segment.start_ms, segment.end_ms],
                    "transcriptText": transcript.rendered(),
                    "transcriptTextBoundary": transcript.boundary(),
                    "nearbyRecordReplayEvents": nearby,
                    "alignmentMethod": "timestamp-window",
                    "alignmentWindowMs": ALIGNMENT_WINDOW_MS,
                    "alignmentConfidence": segment_alignment_confidence(&nearby),
                    "clockAssumptions": [
                        "transcript offsets are relative to audioStartedAtUnixMs",
                        "Record & Replay event timestamps are UTC wall-clock timestamps",
                        "nearby events provide context and are not causal proof"
                    ],
                }))
            }
        })
        .collect()
}

pub(super) fn alignment_diagnostics(
    segments: &[TranscriptSegment],
    events: &[RnrEvent],
    audio_started_at_unix_ms: Option<u64>,
    rnr_started_at_unix_ms: Option<i64>,
) -> Value {
    let malformed_timestamps = events
        .iter()
        .filter(|event| event.timestamp_parse_status == "invalid")
        .map(|event| {
            json!({
                "eventId": event.id,
                "kind": event.kind,
                "reason": "unparseable-timestamp",
                "untrustedValueReflected": false,
            })
        })
        .collect::<Vec<_>>();
    let missing_timestamp_count = events
        .iter()
        .filter(|event| event.timestamp_parse_status == "missing")
        .count();
    let clock_skew_ms = audio_started_at_unix_ms.and_then(|audio_start| {
        rnr_started_at_unix_ms.map(|rnr_start| rnr_start - audio_start as i64)
    });
    let duplicate_segments = duplicate_transcript_segments(segments);
    let out_of_window_event_count = audio_started_at_unix_ms
        .map(|audio_start| {
            events
                .iter()
                .filter(|event| {
                    let Some(event_ms) = event.unix_ms else {
                        return false;
                    };
                    !segments.iter().any(|segment| {
                        let midpoint =
                            audio_start as i64 + ((segment.start_ms + segment.end_ms) / 2) as i64;
                        (event_ms - midpoint).abs() <= ALIGNMENT_WINDOW_MS
                    })
                })
                .count()
        })
        .unwrap_or(0);

    json!({
        "missingAudioClock": audio_started_at_unix_ms.is_none(),
        "missingRecordReplayStart": rnr_started_at_unix_ms.is_none(),
        "malformedRecordReplayTimestamps": malformed_timestamps,
        "recordReplayEventsWithoutTimestamp": missing_timestamp_count,
        "outOfWindowRecordReplayEvents": out_of_window_event_count,
        "recordReplayToAudioStartDeltaMs": clock_skew_ms,
        "clockSkewStatus": clock_skew_status(clock_skew_ms),
        "clockSkewWarningMs": CLOCK_SKEW_WARNING_MS,
        "duplicateTranscriptSegments": duplicate_segments.len(),
        "duplicateTranscriptSegmentDetails": duplicate_segments,
        "claimCeiling": if audio_started_at_unix_ms.is_some() {
            "wall-clock timestamp-window alignment; monotonic drift proof still owed"
        } else {
            "no alignment without audioStartedAtUnixMs"
        },
    })
}

fn alignment_confidence(delta_ms: i64) -> &'static str {
    let abs_delta = delta_ms.abs();
    if abs_delta <= HIGH_CONFIDENCE_MS {
        "high"
    } else if abs_delta <= MEDIUM_CONFIDENCE_MS {
        "medium"
    } else {
        "low"
    }
}

fn segment_alignment_confidence(nearby: &[Value]) -> &'static str {
    nearby
        .iter()
        .filter_map(|event| event.get("deltaMs").and_then(Value::as_i64))
        .map(alignment_confidence)
        .find(|confidence| *confidence == "high")
        .or_else(|| {
            nearby
                .iter()
                .filter_map(|event| event.get("deltaMs").and_then(Value::as_i64))
                .map(alignment_confidence)
                .find(|confidence| *confidence == "medium")
        })
        .unwrap_or("low")
}

fn clock_skew_status(clock_skew_ms: Option<i64>) -> &'static str {
    match clock_skew_ms {
        None => "missing-anchor",
        Some(delta) if delta.abs() <= CLOCK_SKEW_WARNING_MS => "within-window",
        Some(_) => "exceeds-window",
    }
}

fn duplicate_transcript_segments(segments: &[TranscriptSegment]) -> Vec<Value> {
    segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| {
            let duplicate_of = segments.iter().take(index).find(|previous| {
                normalized_text(&previous.text) == normalized_text(&segment.text)
            })?;
            Some(json!({
                "segmentId": segment.id,
                "duplicateOfSegmentId": duplicate_of.id,
                "text": AgentTranscriptEvidence::from_untrusted(&segment.text).rendered(),
                "textBoundary": AgentTranscriptEvidence::from_untrusted(&segment.text).boundary(),
                "transcriptWindowMs": [segment.start_ms, segment.end_ms],
            }))
        })
        .collect()
}

fn normalized_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
