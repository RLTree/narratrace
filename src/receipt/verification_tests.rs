use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn blocker_list_reports_missing_or_untrusted_proof_surfaces() {
    let args = Args {
        command: "receipt".to_string(),
        goal: None,
        root: std::env::temp_dir(),
        skill_dir: None,
        session_dir: Some(unique_tmp("nrr-receipt-blockers")),
        recording_metadata: None,
        recording_events: None,
        baseline_delay_evaluation: None,
        candidate_delay_evaluation: None,
        coverage_json: None,
        coverage_receipt: None,
        delay: "high".to_string(),
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
        receipt_run_id: None,
        receipt_generated_at: None,
        json: false,
    };

    let blockers = blockers(
        &args,
        &json!({"startCoordination":{"recordReplayAndMicrophoneSameOperation":false}}),
        &json!({"state":"recording"}),
        &json!({"transcriptSegments":0,"recordReplayEvents":0}),
        &json!({"evidenceSurfaces":{"audioClockPresent":false}}),
        &json!({"privacyBoundary":{"allowedToShareWithoutReview":true}}),
        &Value::Null,
        &Value::Null,
        &json!({"status":"disabled","unresolvedMismatches":1}),
    );

    assert!(blockers.contains(&"narration helper did not stop cleanly"));
    assert!(blockers.contains(&"Record & Replay metadata path is required for live capture proof"));
    assert!(blockers.contains(&"post-commit transcription drain receipt is missing"));
    assert!(blockers.contains(&"final transcript alignment has unresolved mismatches"));
}

#[test]
fn helper_counts_arrays_numbers_and_sensitive_raw_local_artifacts() {
    let value = json!({
        "count": 7,
        "items": [1, 2, 3],
        "privacyBoundary": {
            "rawLocalOnly": [
                {"path":"a.wav","containsSensitivePatterns":true},
                {"path":"b.json","containsSensitivePatterns":false}
            ]
        }
    });

    assert_eq!(first_u64(&[value.pointer("/items")]), Some(3));
    assert_eq!(first_u64(&[value.pointer("/count")]), Some(7));
    assert_eq!(
        max_u64(&[value.pointer("/items"), value.pointer("/count")]),
        Some(7)
    );
    assert_eq!(
        post_commit_completed_segments(
            &json!({"captureStats":{"realtimeCompletedSegmentsObserved":2}})
        ),
        Some(2)
    );
    assert_eq!(
        sensitive_raw_local_artifacts(&value)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(sensitive_raw_local_artifacts(&json!({})), Value::Null);
}

#[test]
fn artifact_nonempty_file_requires_regular_nonempty_file() {
    let root = unique_tmp("nrr-receipt-artifact-file");
    fs::create_dir_all(&root).unwrap();
    let empty = root.join("empty.json");
    let nonempty = root.join("nonempty.json");
    fs::write(&empty, "").unwrap();
    fs::write(&nonempty, "{}").unwrap();

    assert!(!artifact_is_nonempty_file(None));
    assert!(!artifact_is_nonempty_file(Some(empty.to_str().unwrap())));
    assert!(artifact_is_nonempty_file(Some(nonempty.to_str().unwrap())));
}

#[test]
fn blocker_list_reports_bad_existing_artifacts_and_review_failures() {
    let root = unique_tmp("nrr-receipt-blocker-branches");
    fs::create_dir_all(&root).unwrap();
    let empty_metadata = root.join("metadata.json");
    let empty_events = root.join("events.jsonl");
    fs::write(&empty_metadata, "").unwrap();
    fs::write(&empty_events, "").unwrap();
    let mut args = base_args(root.clone());
    args.recording_metadata = Some(empty_metadata.display().to_string());
    args.recording_events = Some(empty_events.display().to_string());

    let blockers = blockers(
        &args,
        &json!({"startCoordination":{"recordReplayAndMicrophoneSameOperation":true}}),
        &json!({"state":"stopped"}),
        &json!({"transcriptSegments":[{}],"recordReplayEvents":[{}]}),
        &json!({"evidenceSurfaces":{"audioClockPresent":true}}),
        &json!({
            "privacyBoundary": {
                "allowedToShareWithoutReview": false,
                "generatedArtifactLeakScan": {"status": "blocked"}
            }
        }),
        &json!({"completedSegments":0,"errors":2}),
        &Value::Null,
        &json!({"status":"aligned","unresolvedMismatches":0}),
    );

    assert!(blockers.contains(&"Record & Replay metadata path must exist as a non-empty file"));
    assert!(blockers.contains(&"Record & Replay events path must exist as a non-empty file"));
    assert!(blockers.contains(&"post-commit transcription drain did not complete any segments"));
    assert!(blockers.contains(&"post-commit transcription drain recorded errors"));
    assert!(blockers.contains(&"generated artifact leak scan is blocked"));
}

#[test]
fn helper_counts_missing_values_and_reads_regular_json_only() {
    let root = unique_tmp("nrr-receipt-helper-json");
    fs::create_dir_all(&root).unwrap();
    let good = root.join("good.json");
    fs::write(&good, r#"{"ok":true}"#).unwrap();

    assert_eq!(first_u64(&[None, Some(&Value::String("no".into()))]), None);
    assert_eq!(max_u64(&[None, Some(&Value::String("no".into()))]), None);
    assert_eq!(read_json(&good).unwrap()["ok"], true);
    assert!(read_json(&root.join("missing.json")).is_err());
}

fn base_args(session_dir: PathBuf) -> Args {
    Args {
        command: "receipt".to_string(),
        goal: None,
        root: std::env::temp_dir(),
        skill_dir: None,
        session_dir: Some(session_dir),
        recording_metadata: None,
        recording_events: None,
        baseline_delay_evaluation: None,
        candidate_delay_evaluation: None,
        coverage_json: None,
        coverage_receipt: None,
        delay: "high".to_string(),
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
        receipt_run_id: None,
        receipt_generated_at: None,
        json: false,
    }
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
