use crate::config::{Args, required_session_dir};
use crate::private_fs::write_private;
use crate::safe_path::regular_file_metadata;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
#[cfg(test)]
use std::fs;
use std::path::Path;

const SCRIPTED_MARKERS_ALPHA_QUEBEC: [&str; 17] = [
    "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india", "juliet",
    "kilo", "lima", "mike", "november", "oscar", "papa", "quebec",
];
const MAX_DELAY_JSON_BYTES: u64 = 8 * 1024 * 1024;
const MAX_DELAY_JSONL_BYTES: u64 = 16 * 1024 * 1024;
const MAX_DELAY_JSONL_ROWS: usize = 100_000;

pub fn write_delay_evaluation(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    let status = read_json(&session_dir.join("status.json")).unwrap_or(Value::Null);
    let capture_clock = read_json(&session_dir.join("capture-clock.json")).unwrap_or(Value::Null);
    let final_alignment =
        read_json(&session_dir.join("final-transcript-alignment.json")).unwrap_or(Value::Null);
    let transcript_events = read_json_lines(&session_dir.join("transcript.timeline.jsonl"))?;
    let metadata = args
        .recording_metadata
        .as_deref()
        .map(Path::new)
        .and_then(|path| read_json(path).ok());
    let event_count = args
        .recording_events
        .as_deref()
        .map(Path::new)
        .and_then(|path| line_count(path).ok());
    let audio_started_at_unix_ms = capture_clock
        .get("audioStartedAtUnixMs")
        .and_then(Value::as_i64);
    let first_audio_chunk_at_unix_ms = capture_clock
        .get("firstAudioChunkAtUnixMs")
        .and_then(Value::as_i64);
    let record_replay_started_at_unix_ms = metadata
        .as_ref()
        .and_then(|value| value.get("startedAt"))
        .and_then(Value::as_str)
        .and_then(|value| parse_utc_timestamp_ms(value).ok());
    let first_delta = first_event_latency(&transcript_events, "delta");
    let first_completed = first_event_latency(&transcript_events, "completed");
    let marker_recall = scripted_marker_recall(&final_alignment);
    let output_path = session_dir.join("delay-evaluation.json");
    write_private(
        &output_path,
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.delay-evaluation.v1",
            "status": "measured",
            "sessionDir": session_dir.display().to_string(),
            "realtimeDelay": status.get("delay").and_then(Value::as_str),
            "model": status.get("model").and_then(Value::as_str),
            "audioInput": status.get("audioInput").cloned().unwrap_or(Value::Null),
            "latencyMetrics": {
                "recordReplayStartToFirstAudioChunkMs": delta(first_audio_chunk_at_unix_ms, record_replay_started_at_unix_ms),
                "recordReplayStartToCaptureClockAudioStartMs": delta(audio_started_at_unix_ms, record_replay_started_at_unix_ms),
                "firstRealtimeDeltaLatencyMs": first_delta.latency_ms,
                "firstRealtimeDeltaLatencySource": first_delta.source,
                "firstCompletedRealtimeSegmentLatencyMs": first_completed.latency_ms,
                "firstCompletedRealtimeSegmentLatencySource": first_completed.source
            },
            "alignmentMetrics": {
                "finalAlignmentStatus": final_alignment.get("status").and_then(Value::as_str),
                "finalAlignedUtteranceCount": final_alignment.get("segments").and_then(Value::as_array).map(|segments| segments.len()),
                "unresolvedMismatches": final_alignment.get("unresolvedMismatches").and_then(Value::as_u64),
                "scriptedMarkerRecall": marker_recall
            },
            "recordReplay": {
                "metadataPath": args.recording_metadata,
                "eventsPath": args.recording_events,
                "eventCount": event_count,
                "startedAtUnixMs": record_replay_started_at_unix_ms
            },
            "comparisonPolicy": {
                "primaryMetric": "final aligned transcript quality and transcript/action window usefulness",
                "markerRecallUse": "diagnostic for scripted tests only; normal workflows must not rely on markers",
                "delayPolicy": "keep high as default until low matches final alignment quality with lower latency"
            },
            "privacy": {
                "rawTranscriptCopied": false,
                "rawAudioCopied": false,
                "localProvenanceMetadataIncluded": true,
                "artifactPolicy": "Contains timing/count metrics plus local provenance metadata such as session directory, artifact paths, and audio input metadata; does not copy raw transcript text or audio."
            }
        }))?,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "delayEvaluationPath": output_path,
            "sessionDir": session_dir,
            "status": "measured"
        }))?
    );
    Ok(())
}

pub fn write_delay_comparison(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    let baseline_path = args
        .baseline_delay_evaluation
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--baseline-delay-evaluation is required"))?;
    let candidate_path = args
        .candidate_delay_evaluation
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--candidate-delay-evaluation is required"))?;
    let baseline = read_json(baseline_path).context("failed to read baseline delay evaluation")?;
    let candidate =
        read_json(candidate_path).context("failed to read candidate delay evaluation")?;
    let summary = compare_delay_evaluations(&baseline, &candidate);
    let output_path = session_dir.join("delay-comparison.json");
    write_private(
        &output_path,
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.delay-comparison.v1",
            "status": summary.status,
            "baseline": evaluation_summary(&baseline, baseline_path),
            "candidate": evaluation_summary(&candidate, candidate_path),
            "deltas": summary.deltas,
            "decision": {
                "defaultDelayChangeAllowed": false,
                "recommendation": summary.recommendation,
                "reason": summary.reason,
                "operatorUsefulnessReviewRequired": true
            },
            "comparisonPolicy": {
                "primaryMetric": "final aligned transcript quality and transcript/action window usefulness",
                "latencyMetricUse": "optimization only after alignment quality is equivalent or better",
                "markerRecallUse": "diagnostic for scripted tests only; normal workflows must not rely on markers",
                "delayPolicy": "keep high as default unless a low-delay run matches final alignment quality and operator-reviewed action-window usefulness"
            },
            "privacy": {
                "rawTranscriptCopied": false,
                "rawAudioCopied": false,
                "localProvenanceMetadataIncluded": true,
                "artifactPolicy": "Contains timing/count comparisons plus local provenance paths; does not copy raw transcript text or audio."
            }
        }))?,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "delayComparisonPath": output_path,
            "sessionDir": session_dir,
            "status": summary.status,
            "recommendation": summary.recommendation
        }))?
    );
    Ok(())
}
