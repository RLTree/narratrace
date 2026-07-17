use crate::config::{Args, required_session_dir};
use crate::private_fs::write_private;
use crate::safe_path::open_regular_file;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_START_DELTA_MS: i64 = 5_000;
const TIMESTAMP_PROXIMITY_VERIFIED: &str = "timestamp-proximity-verified";

#[derive(Debug, Clone)]
pub struct ParentOperationEvaluation {
    pub status_text: String,
    pub session_dir: PathBuf,
    pub metadata_path: String,
    pub events_path: String,
    pub rnr_session_id: Option<String>,
    pub rnr_started_at: String,
    pub rnr_started_at_unix_ms: i64,
    pub rnr_ended_at: Option<String>,
    pub audio_started_at_unix_ms: i64,
    pub audio_started_at_source: String,
    pub capture_clock_audio_started_at_unix_ms: Option<i64>,
    pub first_audio_chunk_at_unix_ms: Option<i64>,
    pub audio_input: Value,
    pub microphone_state: Option<String>,
    pub event_count: u64,
    pub metadata_digest: String,
    pub events_digest: String,
    pub first_event_at_unix_ms: Option<i64>,
    pub last_event_at_unix_ms: Option<i64>,
    pub start_delta_ms: i64,
    pub within_allowed_start_delta: bool,
    pub microphone_stopped_cleanly: bool,
    pub post_commit_drain_completed: bool,
    pub post_commit_drain_completed_segments: Option<u64>,
    pub post_commit_drain_errors: Option<usize>,
}

pub fn write_parent_operation_receipt(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    let metadata_path = args
        .recording_metadata
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--recording-metadata is required"))?;
    let events_path = args
        .recording_events
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--recording-events is required"))?;
    let evaluation = evaluate_parent_operation(&session_dir, metadata_path, events_path)?;
    let binding =
        ParentOperationBinding::from_evaluation(&evaluation, args.receipt_run_id.as_deref());
    let receipt_path = session_dir.join("parent-operation-receipt.json");
    write_private(
        &receipt_path,
        serde_json::to_string_pretty(&json!({
            "schema": PARENT_RECEIPT_SCHEMA,
            "status": evaluation.status_text,
            "proofClass": PARENT_PROOF_CLASS,
            "runBinding": {
                "runId": binding.run_id,
                "source": if args.receipt_run_id.is_some() {
                    "trusted-current-invocation"
                } else {
                    "missing-trusted-current-invocation"
                }
            },
            "coordinationMode": "parent-orchestrated-parallel-tool-call",
            "claimCeiling": "timestamp-proximity parent-operation receipt; durable same-start proof still depends on current thread tool-call provenance or a future app/tool operation id",
            "durableParentOperationId": null,
            "recordReplay": binding.record_replay,
            "microphoneCapture": binding.microphone_capture,
            "sameStartChecks": binding.same_start_checks,
            "rawPayloadCopied": false,
            "parentToolCallEvidence": {
                "source": "codex-thread-visible parallel operation",
                "operationIdAvailable": false,
                "limitation": "Codex tool API does not expose a reusable parent operation id to the skill CLI; this receipt binds same-run metadata paths and timestamp proximity, and the thread transcript remains the parent tool-call provenance."
            }
        }))?,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "parentOperationReceiptPath": receipt_path,
            "sessionDir": session_dir,
            "status": evaluation.status_text,
            "startDeltaMs": evaluation.start_delta_ms
        }))?
    );
    Ok(())
}

pub fn evaluate_parent_operation(
    session_dir: &Path,
    metadata_path: &str,
    events_path: &str,
) -> Result<ParentOperationEvaluation> {
    let (metadata, metadata_digest) = read_json_with_digest(Path::new(metadata_path))
        .context("failed to read Record & Replay metadata")?;
    let status =
        read_json(&session_dir.join("status.json")).context("failed to read capture status")?;
    let capture_clock = read_json(&session_dir.join("capture-clock.json"))
        .context("failed to read capture clock")?;
    let post_commit_drain =
        read_json(&session_dir.join("post-commit-drain.json")).unwrap_or_else(|_| Value::Null);
    let rnr_started_at = metadata
        .get("startedAt")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Record & Replay metadata missing startedAt"))?
        .to_string();
    let rnr_started_at_unix_ms = parse_utc_timestamp_ms(&rnr_started_at)?;
    let rnr_session_id = metadata
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Record & Replay metadata missing id"))?
        .to_string();
    let rnr_ended_at = metadata
        .get("endedAt")
        .and_then(Value::as_str)
        .map(str::to_string);
    let rnr_ended_at_unix_ms = rnr_ended_at
        .as_deref()
        .map(parse_utc_timestamp_ms)
        .transpose()?;
    if rnr_ended_at_unix_ms.is_some_and(|ended_at| ended_at < rnr_started_at_unix_ms) {
        bail!("Record & Replay metadata endedAt predates startedAt");
    }
    let capture_clock_audio_started_at_unix_ms = capture_clock
        .get("audioStartedAtUnixMs")
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow::anyhow!("capture clock missing audioStartedAtUnixMs"))?;
    let first_audio_chunk_at_unix_ms = capture_clock
        .get("firstAudioChunkAtUnixMs")
        .and_then(Value::as_i64);
    let (audio_started_at_unix_ms, audio_started_at_source) =
        if let Some(first_audio_chunk_at_unix_ms) = first_audio_chunk_at_unix_ms {
            (
                first_audio_chunk_at_unix_ms,
                "first-nonempty-ffmpeg-stdout-read".to_string(),
            )
        } else {
            (
                capture_clock_audio_started_at_unix_ms,
                "legacy-helper-process-clock-before-audio-read".to_string(),
            )
        };
    let start_delta_ms = audio_started_at_unix_ms
        .checked_sub(rnr_started_at_unix_ms)
        .ok_or_else(|| anyhow::anyhow!("audio and Record & Replay start delta overflow"))?;
    let event_evidence = read_parent_event_evidence(
        Path::new(events_path),
        rnr_started_at_unix_ms,
        rnr_ended_at_unix_ms,
    )?;
    let microphone_stopped_cleanly = status.get("state").and_then(Value::as_str) == Some("stopped");
    let post_commit_drain_completed_segments =
        completed_transcript_segments_from_drain(&post_commit_drain);
    let post_commit_drain_completed = post_commit_drain_completed_segments.unwrap_or(0) > 0;
    let within_allowed_start_delta = start_delta_ms
        .checked_abs()
        .is_some_and(|delta| delta <= MAX_START_DELTA_MS);
    let status_text = if microphone_stopped_cleanly
        && event_evidence.count > 0
        && within_allowed_start_delta
        && post_commit_drain_completed
    {
        TIMESTAMP_PROXIMITY_VERIFIED
    } else {
        "blocked"
    };
    Ok(ParentOperationEvaluation {
        status_text: status_text.to_string(),
        session_dir: session_dir.to_path_buf(),
        metadata_path: metadata_path.to_string(),
        events_path: events_path.to_string(),
        rnr_session_id: Some(rnr_session_id),
        rnr_started_at,
        rnr_started_at_unix_ms,
        rnr_ended_at,
        audio_started_at_unix_ms,
        audio_started_at_source,
        capture_clock_audio_started_at_unix_ms: Some(capture_clock_audio_started_at_unix_ms),
        first_audio_chunk_at_unix_ms,
        audio_input: status.get("audioInput").cloned().unwrap_or(Value::Null),
        microphone_state: status
            .get("state")
            .and_then(Value::as_str)
            .map(str::to_string),
        event_count: event_evidence.count,
        metadata_digest,
        events_digest: event_evidence.digest,
        first_event_at_unix_ms: event_evidence.first_timestamp_unix_ms,
        last_event_at_unix_ms: event_evidence.last_timestamp_unix_ms,
        start_delta_ms,
        within_allowed_start_delta,
        microphone_stopped_cleanly,
        post_commit_drain_completed,
        post_commit_drain_completed_segments,
        post_commit_drain_errors: post_commit_drain
            .get("errors")
            .and_then(Value::as_array)
            .map(Vec::len),
    })
}
