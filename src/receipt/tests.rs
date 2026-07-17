use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parent_operation_verification_recomputes_instead_of_trusting_receipt_booleans() {
    let root = unique_tmp("nrr-forged-parent");
    let session_dir = root.join("session");
    let rnr_dir = root.join("rnr");
    fs::create_dir_all(&session_dir).unwrap();
    fs::create_dir_all(&rnr_dir).unwrap();
    let metadata_path = rnr_dir.join("session.json");
    let events_path = rnr_dir.join("events.jsonl");
    fs::write(
        &metadata_path,
        r#"{"id":"rnr-1","startedAt":"2026-06-19T01:38:25Z"}"#,
    )
    .unwrap();
    fs::write(
        &events_path,
        r#"{"id":1,"kind":"click","timestamp":"2026-06-19T01:38:29Z"}"#,
    )
    .unwrap();
    fs::write(session_dir.join("status.json"), r#"{"state":"stopped"}"#).unwrap();
    fs::write(
        session_dir.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833115000}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("post-commit-drain.json"),
        r#"{"completedSegments":0,"errors":[]}"#,
    )
    .unwrap();
    let forged_receipt = json!({
        "status": "timestamp-proximity-verified",
        "sameStartChecks": {
            "startDeltaMs": 500,
            "withinAllowedStartDelta": true,
            "recordReplayEventsPresent": true,
            "microphoneStoppedCleanly": true,
            "postCommitDrainCompleted": true
        }
    });
    let args = test_args(
        session_dir,
        metadata_path.to_string_lossy().to_string(),
        events_path.to_string_lossy().to_string(),
    );

    assert!(!parent_operation_receipt_matches_current_artifacts(
        &args,
        &forged_receipt
    ));
}

#[test]
fn blocked_parent_receipt_still_counts_when_artifacts_match() {
    let root = unique_tmp("nrr-blocked-parent-proximity");
    let session_dir = root.join("session");
    let rnr_dir = root.join("rnr");
    fs::create_dir_all(&session_dir).unwrap();
    fs::create_dir_all(&rnr_dir).unwrap();
    let metadata_path = rnr_dir.join("session.json");
    let events_path = rnr_dir.join("events.jsonl");
    fs::write(
        &metadata_path,
        r#"{"id":"rnr-1","startedAt":"2026-06-19T01:38:25Z"}"#,
    )
    .unwrap();
    fs::write(
        &events_path,
        r#"{"id":1,"kind":"click","timestamp":"2026-06-19T01:38:29Z"}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("status.json"),
        r#"{"state":"stop-requested"}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833106000}"#,
    )
    .unwrap();
    let args = test_args(
        session_dir,
        metadata_path.to_string_lossy().to_string(),
        events_path.to_string_lossy().to_string(),
    );
    let receipt = current_parent_receipt(&args);

    assert!(parent_operation_receipt_matches_current_artifacts(
        &args, &receipt
    ));
}

#[test]
fn large_start_delta_is_diagnostic_when_parent_artifacts_match() {
    let root = unique_tmp("nrr-large-start-delta");
    let session_dir = root.join("session");
    let rnr_dir = root.join("rnr");
    fs::create_dir_all(&session_dir).unwrap();
    fs::create_dir_all(&rnr_dir).unwrap();
    let metadata_path = rnr_dir.join("session.json");
    let events_path = rnr_dir.join("events.jsonl");
    fs::write(
        &metadata_path,
        r#"{"id":"rnr-1","startedAt":"2026-06-19T01:38:25Z"}"#,
    )
    .unwrap();
    fs::write(
        &events_path,
        r#"{"id":1,"kind":"click","timestamp":"2026-06-19T01:38:29Z"}"#,
    )
    .unwrap();
    fs::write(session_dir.join("status.json"), r#"{"state":"stopped"}"#).unwrap();
    fs::write(
        session_dir.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833125000}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("post-commit-drain.json"),
        r#"{"completedSegments":1,"errors":[]}"#,
    )
    .unwrap();
    let args = test_args(
        session_dir,
        metadata_path.to_string_lossy().to_string(),
        events_path.to_string_lossy().to_string(),
    );
    let receipt = current_parent_receipt(&args);

    assert!(parent_operation_receipt_matches_current_artifacts(
        &args, &receipt
    ));
}

#[test]
fn completed_segment_fallback_uses_largest_counter() {
    let value = json!({
        "completedSegments": 0,
        "captureStats": {
            "realtimeCompletedSegmentsObserved": 15
        }
    });

    assert_eq!(
        max_u64(&[
            value.pointer("/completedSegments"),
            value.pointer("/captureStats/realtimeCompletedSegmentsObserved"),
        ]),
        Some(15)
    );
}

#[cfg(unix)]
#[test]
fn receipt_json_reads_reject_symlinked_status_file() {
    let root = unique_tmp("nrr-receipt-status-symlink");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("target.json"),
        r#"{"state":"stopped","model":"SYMLINKED_PRIVATE_SENTINEL"}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(root.join("target.json"), root.join("status.json")).unwrap();

    assert!(read_json(&root.join("status.json")).is_err());
}

fn test_args(session_dir: PathBuf, metadata_path: String, events_path: String) -> Args {
    Args {
        command: "receipt".to_string(),
        goal: None,
        root: std::env::temp_dir(),
        skill_dir: None,
        session_dir: Some(session_dir),
        recording_metadata: Some(metadata_path),
        recording_events: Some(events_path),
        baseline_delay_evaluation: None,
        candidate_delay_evaluation: None,
        coverage_json: None,
        coverage_receipt: None,
        delay: "low".to_string(),
        input: "auto".to_string(),
        max_seconds: None,
        record_replay_status: None,
        microphone_capture_consent: false,
        openai_postprocessing_consent: false,
        custom_runtime_path_consent: false,
        custom_audio_filter_consent: false,
        batch_transcription_enabled: true,
        cleanup_enabled: true,
        batch_transcription_model: "gpt-4o-transcribe".to_string(),
        cleanup_model: "gpt-5.4-mini".to_string(),
        audio_retention_mode: "private-wav".to_string(),
        audio_retention_path: None,
        audio_filter: crate::config::DEFAULT_AUDIO_FILTER.to_string(),
        cleanup_dictionary_source: None,
        replay_voice_style: "neutral".to_string(),
        replay_voice_pace: "normal".to_string(),
        replay_voice_emphasis: "balanced".to_string(),
        receipt_run_id: Some("trusted-test-run".to_string()),
        receipt_generated_at: None,
        json: false,
    }
}

fn current_parent_receipt(args: &Args) -> Value {
    let session_dir = required_session_dir(args).unwrap();
    let evaluation = evaluate_parent_operation(
        &session_dir,
        args.recording_metadata.as_deref().unwrap(),
        args.recording_events.as_deref().unwrap(),
    )
    .unwrap();
    let binding = crate::parent_operation::ParentOperationBinding::from_evaluation(
        &evaluation,
        args.receipt_run_id.as_deref(),
    );
    json!({
        "schema": crate::parent_operation::PARENT_RECEIPT_SCHEMA,
        "status": evaluation.status_text,
        "proofClass": crate::parent_operation::PARENT_PROOF_CLASS,
        "runBinding": {"runId": binding.run_id, "source": "trusted-current-invocation"},
        "recordReplay": binding.record_replay,
        "microphoneCapture": binding.microphone_capture,
        "sameStartChecks": binding.same_start_checks
    })
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
