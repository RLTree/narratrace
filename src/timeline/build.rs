#[cfg(test)]
mod agent_content_tests;
mod agent_text;
mod alignment;
mod alignment_authority;
#[cfg(test)]
mod authority_integration_tests;
#[cfg(test)]
mod authority_limits_tests;
#[cfg(test)]
mod authority_tests;
mod conflict;
mod ingestion;
mod record_replay;
mod time;
mod transcript;

#[cfg(test)]
mod record_replay_tests;

pub use transcript::{
    TranscriptSegment, raw_realtime_segments, record_transcript_event_with_monotonic_offset,
    transcript_segments,
};

use crate::private_fs::write_private;
use crate::redaction::render_untrusted_markdown;
use crate::safe_path::regular_file_metadata;
pub(crate) use agent_text::consume_transcript_segment_text;
use agent_text::transcript_content_boundary;
use alignment::{align_segments, alignment_diagnostics};
use anyhow::Result;
use conflict::conflict_diagnostics;
use record_replay::{RnrEvent, read_rnr_events};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
pub(crate) use time::parse_utc_millis;
use time::{format_offset, now_unix_ms};
use transcript::read_audio_started_at;

const ALIGNMENT_WINDOW_MS: i64 = 6_000;
const HIGH_CONFIDENCE_MS: i64 = 1_000;
const MEDIUM_CONFIDENCE_MS: i64 = 3_000;
const CLOCK_SKEW_WARNING_MS: i64 = ALIGNMENT_WINDOW_MS;

#[derive(Debug)]
pub struct TemporalArtifacts {
    pub context_path: PathBuf,
    pub notes_path: PathBuf,
    pub transcript_segment_count: usize,
    pub rnr_event_count: usize,
    pub alignment_count: usize,
    pub conflict_count: usize,
}

pub fn write_capture_clock(session_dir: &Path, delay: &str) -> Result<u64> {
    let started_at_unix_ms = now_unix_ms();
    write_private(
        session_dir.join("capture-clock.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.capture-clock.v1",
            "audioStartedAtUnixMs": started_at_unix_ms,
            "audioStartEvidence": "helper-process-clock-before-audio-read",
            "delay": delay,
        }))?,
    )?;
    Ok(started_at_unix_ms)
}

pub fn record_first_audio_chunk_clock(session_dir: &Path) -> Result<u64> {
    let path = session_dir.join("capture-clock.json");
    let mut value = regular_file_metadata(&path)
        .ok()
        .and_then(|_| read_json(&path).ok())
        .unwrap_or_else(|| {
            json!({
                "schema": "narrated-record-replay.capture-clock.v1"
            })
        });
    let first_audio_chunk_at_unix_ms = now_unix_ms();
    value["firstAudioChunkAtUnixMs"] = json!(first_audio_chunk_at_unix_ms);
    value["firstAudioChunkEvidence"] = json!("first-nonempty-ffmpeg-stdout-read");
    write_private(path, serde_json::to_string_pretty(&value)?)?;
    Ok(first_audio_chunk_at_unix_ms)
}

pub fn build_temporal_context(
    session_dir: &Path,
    recording_metadata: Option<&str>,
    recording_events: Option<&str>,
) -> Result<TemporalArtifacts> {
    let transcript_segments = transcript::transcript_segments_checked(session_dir)?;
    let rnr_events = recording_events
        .and_then(existing_path)
        .map(read_rnr_events)
        .transpose()?
        .unwrap_or_default();
    let metadata = read_recording_metadata(recording_metadata)?;
    let audio_started_at_unix_ms = read_audio_started_at(session_dir);
    let monotonic_policy = monotonic_clock_policy(&transcript_segments);
    let rnr_started_at_unix_ms = metadata
        .get("startedAt")
        .and_then(Value::as_str)
        .and_then(parse_utc_millis);
    ingestion::enforce_alignment_work(transcript_segments.len(), rnr_events.len())?;
    let alignments = align_segments(&transcript_segments, &rnr_events, audio_started_at_unix_ms);
    let diagnostics = alignment_diagnostics(
        &transcript_segments,
        &rnr_events,
        audio_started_at_unix_ms,
        rnr_started_at_unix_ms,
    );
    let conflicts =
        conflict_diagnostics(&transcript_segments, &rnr_events, audio_started_at_unix_ms);
    let conflict_count = conflicts.len();

    let context_path = session_dir.join("temporal-context.json");
    let notes_path = session_dir.join("timestamped-notes.md");
    write_private(
        &context_path,
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.temporal-context.v1",
            "anchors": {
                "audioStartedAtUnixMs": audio_started_at_unix_ms,
                "recordReplayStartedAtUnixMs": rnr_started_at_unix_ms,
                "recordReplayMetadata": recording_metadata,
                "recordReplayEvents": recording_events,
            },
            "alignmentPolicy": {
                "method": "audio-start-plus-transcript-offsets-to-record-replay-utc-events",
                "windowMs": ALIGNMENT_WINDOW_MS,
                "confidenceThresholdsMs": {
                    "high": HIGH_CONFIDENCE_MS,
                    "medium": MEDIUM_CONFIDENCE_MS,
                    "low": ALIGNMENT_WINDOW_MS
                },
                "clockAssumptions": [
                    "transcript offsets are relative to audioStartedAtUnixMs",
                    "Record & Replay event timestamps are parseable UTC instants",
                    "timestamp-window alignment is context, not causal proof"
                ],
                "monotonicClock": {
                    "status": monotonic_policy.status,
                    "transcriptOffsetsWithMonotonicClock": monotonic_policy.segment_count,
                    "unit": "milliseconds",
                    "scope": "process-local narration capture offsets",
                    "claimCeiling": monotonic_policy.claim_ceiling
                },
                "clockSkewWarningMs": CLOCK_SKEW_WARNING_MS,
                "diagnosticContract": {
                    "missingAudioClock": "alignment cannot be produced without audioStartedAtUnixMs",
                    "clockSkewStatus": "compares Record & Replay startedAt to audioStartedAtUnixMs when both are available",
                    "duplicateTranscriptSegments": "repeated normalized transcript text is diagnostic only and may be legitimate narration"
                }
            },
            "redactionPolicy": {
                "status": "applied-to-generated-context",
                "scope": [
                    "transcript segment text in temporal-context.json",
                    "timestamped-notes.md",
                    "thought-process.md",
                    "skill-refinement-packet.md"
                ],
                "rawLocalInputs": [
                    "transcript.timeline.jsonl",
                    "transcript.final.txt",
                    "transcript.live.txt"
                ],
                "claimCeiling": "pattern redaction only; full privacy review and negative fixture corpus still owed"
            },
            "conflictPolicy": {
                "status": "heuristic",
                "rule": "transcript action claims without nearby Record & Replay UI events, or with obvious nearby UI label disagreement, are warnings and not observed action evidence",
                "claimCeiling": "detects missing nearby UI support and conservative commit-versus-cancel label mismatch only; real workflow usefulness inspection still owed"
            },
            "transcriptContentBoundary": transcript_content_boundary(),
            "capabilities": [
                "pair spoken reasoning with nearby Record & Replay UI events",
                "create timestamped skill-refinement packets",
                "support later keyframe or localImage references by shared unix-ms anchors"
            ],
            "transcriptSegments": transcript_segments.iter().map(TranscriptSegment::to_json).collect::<Vec<_>>(),
            "recordReplayEvents": rnr_events.iter().map(RnrEvent::to_json).collect::<Vec<_>>(),
            "alignmentDiagnostics": diagnostics,
            "conflictDiagnostics": {
                "transcriptActionClaimsWithoutNearbyUiEvidence": conflict_count,
                "transcriptActionClaimsNeedingReview": conflict_count,
                "warnings": conflicts,
            },
            "alignments": alignments,
        }))?,
    )?;
    write_private(
        &notes_path,
        timestamped_notes_markdown(&transcript_segments, &rnr_events, audio_started_at_unix_ms),
    )?;

    Ok(TemporalArtifacts {
        context_path,
        notes_path,
        transcript_segment_count: transcript_segments.len(),
        rnr_event_count: rnr_events.len(),
        alignment_count: alignments.len(),
        conflict_count,
    })
}
