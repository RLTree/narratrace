use super::agent_text::AgentTranscriptEvidence;
use super::transcript::TranscriptSegment;
use super::{RnrEvent, nearby_events};
use crate::redaction::redact_text;
use serde_json::{Value, json};

pub(super) fn conflict_diagnostics(
    segments: &[TranscriptSegment],
    events: &[RnrEvent],
    audio_started_at_unix_ms: Option<u64>,
) -> Vec<Value> {
    segments
        .iter()
        .filter(|segment| transcript_mentions_action(&segment.text))
        .filter_map(|segment| warning_for_segment(segment, events, audio_started_at_unix_ms))
        .collect()
}

fn warning_for_segment(
    segment: &TranscriptSegment,
    events: &[RnrEvent],
    audio_started_at_unix_ms: Option<u64>,
) -> Option<Value> {
    let Some(audio_start) = audio_started_at_unix_ms else {
        return Some(warning(segment, "missing-audio-anchor", Vec::new()));
    };
    let midpoint = audio_start as i64 + ((segment.start_ms + segment.end_ms) / 2) as i64;
    let nearby = nearby_events(events, midpoint);
    if nearby.is_empty() {
        return Some(warning(
            segment,
            "no-nearby-record-replay-event",
            Vec::new(),
        ));
    }
    semantic_mismatch_reason(&segment.text, &nearby)
        .map(|reason| warning(segment, reason, nearby_event_summaries(&nearby)))
}

fn warning(segment: &TranscriptSegment, reason: &str, nearby_events: Vec<Value>) -> Value {
    let transcript = AgentTranscriptEvidence::from_untrusted(&segment.text);
    json!({
        "segmentId": segment.id,
        "reason": reason,
        "severity": "needs-ui-evidence",
        "transcriptWindowMs": [segment.start_ms, segment.end_ms],
        "transcriptText": transcript.rendered(),
        "transcriptTextBoundary": transcript.boundary(),
        "nearbyRecordReplayEvents": nearby_events,
        "instruction": "Do not treat this transcript action claim as observed UI evidence without operator review of Record & Replay support.",
    })
}

fn semantic_mismatch_reason(transcript_text: &str, nearby: &[&RnrEvent]) -> Option<&'static str> {
    let transcript_tokens = tokens(transcript_text);
    let claims_commit_action = transcript_tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "save" | "saved" | "submit" | "submitted" | "confirm" | "confirmed"
        )
    });
    if !claims_commit_action {
        return None;
    }
    nearby
        .iter()
        .any(has_cancel_or_destructive_label)
        .then_some("nearby-ui-label-mismatch")
}

fn has_cancel_or_destructive_label(event: &&RnrEvent) -> bool {
    event
        .ui_hint
        .iter()
        .chain(event.window.iter())
        .flat_map(|value| tokens(value))
        .any(|token| {
            matches!(
                token.as_str(),
                "cancel" | "delete" | "discard" | "close" | "back" | "remove" | "reject"
            )
        })
}

fn transcript_mentions_action(value: &str) -> bool {
    tokens(value).iter().any(|word| {
        matches!(
            word.as_str(),
            "click"
                | "clicked"
                | "type"
                | "typed"
                | "select"
                | "selected"
                | "open"
                | "opened"
                | "save"
                | "saved"
                | "submit"
                | "submitted"
                | "press"
                | "pressed"
                | "drag"
                | "dragged"
                | "drop"
                | "dropped"
        )
    })
}

fn nearby_event_summaries(events: &[&RnrEvent]) -> Vec<Value> {
    events
        .iter()
        .take(4)
        .map(|event| {
            json!({
                "eventId": event.id,
                "kind": event.kind,
                "app": event.app,
                "window": event.window.as_deref().map(redact_text),
                "uiHint": event.ui_hint.as_deref().map(redact_text),
            })
        })
        .collect()
}

fn tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}
