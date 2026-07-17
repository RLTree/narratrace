use super::{CaptureQuota, EventKind, MAX_REALTIME_MESSAGE_BYTES, handle_event};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn completed_and_error_events_write_expected_artifacts() {
    let session_dir = unique_tmp("nrr-completed-error-event");
    fs::create_dir_all(&session_dir).unwrap();
    let completed = json!({
        "type": "conversation.item.input_audio_transcription.completed",
        "transcript": "done"
    });
    let error = json!({"type": "error", "message": "failed"});
    let mut quota = CaptureQuota::default();

    assert_eq!(
        handle_event(&session_dir, &completed.to_string(), 77, &mut quota).unwrap(),
        EventKind::Completed
    );
    assert_eq!(
        fs::read_to_string(session_dir.join("transcript.final.txt")).unwrap(),
        "done\n"
    );
    assert_eq!(
        handle_event(&session_dir, &error.to_string(), 88, &mut quota).unwrap(),
        EventKind::Error
    );
    let status: Value =
        serde_json::from_str(&fs::read_to_string(session_dir.join("status.json")).unwrap())
            .unwrap();
    assert_eq!(status["state"], "failed");
    assert_eq!(
        status["error"],
        "realtime service returned an error event; payload omitted"
    );
    assert!(
        !fs::read_to_string(session_dir.join("transcript.events.jsonl"))
            .unwrap()
            .contains("failed")
    );
}

#[test]
fn malformed_or_incomplete_events_do_not_create_segments() {
    let session_dir = unique_tmp("nrr-incomplete-event");
    fs::create_dir_all(&session_dir).unwrap();
    let delta_without_text = json!({
        "type": "conversation.item.input_audio_transcription.delta"
    });
    let completed_without_text = json!({
        "type": "conversation.item.input_audio_transcription.completed"
    });
    let mut quota = CaptureQuota::default();

    assert!(handle_event(&session_dir, "{bad", 1, &mut quota).is_err());
    assert_eq!(
        handle_event(&session_dir, &delta_without_text.to_string(), 2, &mut quota).unwrap(),
        EventKind::Other
    );
    assert_eq!(
        handle_event(
            &session_dir,
            &completed_without_text.to_string(),
            3,
            &mut quota
        )
        .unwrap(),
        EventKind::Completed
    );
}

#[test]
fn oversized_realtime_event_is_rejected_before_any_persistence() {
    let session_dir = unique_tmp("nrr-oversized-event");
    fs::create_dir_all(&session_dir).unwrap();
    let oversized = json!({
        "type": "conversation.item.input_audio_transcription.delta",
        "delta": "x".repeat(MAX_REALTIME_MESSAGE_BYTES)
    });

    let error = handle_event(
        &session_dir,
        &oversized.to_string(),
        1,
        &mut CaptureQuota::default(),
    )
    .unwrap_err();

    assert!(error.to_string().contains("realtime event exceeds"));
    assert!(!session_dir.join("transcript.events.jsonl").exists());
    assert!(!session_dir.join("transcript.live.txt").exists());
    assert!(!session_dir.join("transcript.timeline.jsonl").exists());
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
