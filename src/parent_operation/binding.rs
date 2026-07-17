pub const PARENT_RECEIPT_SCHEMA: &str = "narrated-record-replay.parent-operation-receipt.v1";
pub const PARENT_PROOF_CLASS: &str = "timestamp-proximity";

#[derive(Debug, Clone, PartialEq)]
pub struct ParentOperationBinding {
    pub run_id: Option<String>,
    pub record_replay: Value,
    pub microphone_capture: Value,
    pub same_start_checks: Value,
}

impl ParentOperationBinding {
    pub fn from_evaluation(evaluation: &ParentOperationEvaluation, run_id: Option<&str>) -> Self {
        Self {
            run_id: run_id.map(str::to_string),
            record_replay: json!({
                "sessionId": evaluation.rnr_session_id,
                "startedAt": evaluation.rnr_started_at,
                "startedAtUnixMs": evaluation.rnr_started_at_unix_ms,
                "endedAt": evaluation.rnr_ended_at,
                "metadataPath": evaluation.metadata_path,
                "eventsPath": evaluation.events_path,
                "eventCount": evaluation.event_count,
                "metadataDigest": evaluation.metadata_digest,
                "eventsDigest": evaluation.events_digest,
                "firstEventAtUnixMs": evaluation.first_event_at_unix_ms,
                "lastEventAtUnixMs": evaluation.last_event_at_unix_ms
            }),
            microphone_capture: json!({
                "sessionDir": evaluation.session_dir,
                "state": evaluation.microphone_state,
                "audioStartedAtUnixMs": evaluation.audio_started_at_unix_ms,
                "audioStartedAtSource": evaluation.audio_started_at_source,
                "captureClockAudioStartedAtUnixMs": evaluation.capture_clock_audio_started_at_unix_ms,
                "firstAudioChunkAtUnixMs": evaluation.first_audio_chunk_at_unix_ms,
                "audioInput": evaluation.audio_input,
                "postCommitDrainCompletedSegments": evaluation.post_commit_drain_completed_segments,
                "postCommitDrainErrors": evaluation.post_commit_drain_errors
            }),
            same_start_checks: json!({
                "maxAllowedStartDeltaMs": MAX_START_DELTA_MS,
                "startDeltaMs": evaluation.start_delta_ms,
                "withinAllowedStartDelta": evaluation.within_allowed_start_delta,
                "recordReplayEventsPresent": evaluation.event_count > 0,
                "microphoneStoppedCleanly": evaluation.microphone_stopped_cleanly,
                "postCommitDrainCompleted": evaluation.post_commit_drain_completed
            }),
        }
    }

    pub fn from_receipt(receipt: &Value) -> Option<Self> {
        if receipt.get("schema").and_then(Value::as_str) != Some(PARENT_RECEIPT_SCHEMA)
            || receipt.get("proofClass").and_then(Value::as_str) != Some(PARENT_PROOF_CLASS)
            || receipt
                .pointer("/runBinding/source")
                .and_then(Value::as_str)
                != Some("trusted-current-invocation")
        {
            return None;
        }
        let status = receipt.get("status").and_then(Value::as_str)?;
        if !matches!(status, TIMESTAMP_PROXIMITY_VERIFIED | "blocked") {
            return None;
        }
        Some(Self {
            run_id: receipt
                .pointer("/runBinding/runId")
                .and_then(Value::as_str)
                .map(str::to_string),
            record_replay: receipt.get("recordReplay")?.clone(),
            microphone_capture: receipt.get("microphoneCapture")?.clone(),
            same_start_checks: receipt.get("sameStartChecks")?.clone(),
        })
    }
}
