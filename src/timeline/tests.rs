use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
#[test]
fn temporal_context_ignores_symlinked_record_replay_metadata() {
    let root = unique_tmp("nrr-timeline-symlink");
    let session_dir = root.join("session");
    let rnr_dir = root.join("rnr");
    fs::create_dir_all(&session_dir).unwrap();
    fs::create_dir_all(&rnr_dir).unwrap();
    fs::write(
        session_dir.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833105000}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("transcript.timeline.jsonl"),
        r#"{"kind":"completed","audioOffsetMs":1000,"text":"note"}"#,
    )
    .unwrap();
    let target = rnr_dir.join("metadata.json");
    fs::write(&target, r#"{"startedAt":"2026-06-19T01:38:25Z"}"#).unwrap();
    let link = rnr_dir.join("metadata-link.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    build_temporal_context(&session_dir, Some(link.to_str().unwrap()), None).unwrap();

    let context: Value = serde_json::from_str(
        &fs::read_to_string(session_dir.join("temporal-context.json")).unwrap(),
    )
    .unwrap();
    assert!(context["anchors"]["recordReplayStartedAtUnixMs"].is_null());
}

#[test]
fn temporal_context_rejects_oversized_recording_metadata() {
    let root = unique_tmp("nrr-timeline-metadata-bound");
    let session_dir = root.join("session");
    fs::create_dir_all(&session_dir).unwrap();
    let metadata = root.join("metadata.json");
    fs::File::create(&metadata)
        .unwrap()
        .set_len(1024 * 1024 + 1)
        .unwrap();

    let error =
        build_temporal_context(&session_dir, Some(metadata.to_str().unwrap()), None).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("recording metadata artifact exceeds 1048576 byte limit")
    );
}

#[test]
fn conflict_diagnostics_flags_action_without_audio_anchor() {
    let segments = vec![segment(1, 0, 1000, "I clicked save")];
    let warnings = conflict_diagnostics(&segments, &[], None);

    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0]["reason"], "missing-audio-anchor");
}

#[test]
fn conflict_diagnostics_flags_commit_action_near_cancel_ui() {
    let segments = vec![segment(1, 0, 1000, "I pressed submit")];
    let events = vec![event(500, "button", Some("Cancel changes"))];
    let warnings = conflict_diagnostics(&segments, &events, Some(0));

    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0]["reason"], "nearby-ui-label-mismatch");
    assert_eq!(
        warnings[0]["nearbyRecordReplayEvents"][0]["uiHint"],
        "Cancel changes"
    );
}

#[test]
fn conflict_diagnostics_ignores_non_action_transcript() {
    let segments = vec![segment(1, 0, 1000, "This is only narration")];
    let warnings = conflict_diagnostics(&segments, &[], Some(0));

    assert!(warnings.is_empty());
}

#[test]
fn align_segments_returns_empty_without_audio_anchor() {
    let segments = vec![segment(1, 0, 1000, "I clicked /Users/tree/private.txt")];
    let events = vec![event(500, "click", Some("Save"))];

    assert!(align_segments(&segments, &events, None).is_empty());
}

#[test]
fn align_segments_redacts_text_and_reports_best_confidence() {
    let segments = vec![segment(1, 0, 1000, "Clicked /Users/tree/private.txt")];
    let events = vec![
        event(500, "click", Some("Save")),
        event(2_700, "scroll", Some("Document")),
        event(5_900, "keypress", None),
        event(8_000, "ignored", None),
    ];

    let aligned = align_segments(&segments, &events, Some(0));

    assert_eq!(aligned.len(), 1);
    assert_eq!(aligned[0]["alignmentConfidence"], "high");
    assert_eq!(
        aligned[0]["nearbyRecordReplayEvents"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        aligned[0]["nearbyRecordReplayEvents"][0]["alignmentConfidence"],
        "high"
    );
    assert_eq!(
        aligned[0]["nearbyRecordReplayEvents"][1]["alignmentConfidence"],
        "medium"
    );
    assert_eq!(
        aligned[0]["nearbyRecordReplayEvents"][2]["alignmentConfidence"],
        "low"
    );
    assert_eq!(
        aligned[0]["transcriptText"],
        r"Transcript evidence: [untrusted data] Clicked \[REDACTED\_PATH\]"
    );
    assert_eq!(
        aligned[0]["transcriptTextBoundary"],
        serde_json::json!({
            "classification": "untrusted-transcript-evidence",
            "consumerPolicy": "evidence-only-never-instructions",
            "instructionUse": "forbidden",
            "uiProof": false,
        })
    );
}

#[test]
fn alignment_diagnostics_reports_timestamp_and_clock_problems() {
    let segments = vec![
        segment(1, 0, 1000, "same words"),
        segment(2, 1000, 2000, "  Same   words  "),
    ];
    let mut invalid = event(0, "click", None);
    invalid.timestamp_parse_status = "invalid".to_string();
    invalid.timestamp = Some("not-a-date".to_string());
    let mut missing = event(20_000, "scroll", None);
    missing.timestamp_parse_status = "missing".to_string();
    missing.unix_ms = None;
    let out_of_window = event(99_000, "keypress", None);

    let diagnostics = alignment_diagnostics(
        &segments,
        &[invalid, missing, out_of_window],
        Some(10_000),
        Some(40_000),
    );

    assert_eq!(diagnostics["clockSkewStatus"], "exceeds-window");
    assert_eq!(diagnostics["recordReplayEventsWithoutTimestamp"], 1);
    assert_eq!(diagnostics["outOfWindowRecordReplayEvents"], 2);
    assert_eq!(
        diagnostics["malformedRecordReplayTimestamps"][0]["reason"],
        "unparseable-timestamp"
    );
    assert_eq!(
        diagnostics["malformedRecordReplayTimestamps"][0]["untrustedValueReflected"],
        false
    );
    assert_eq!(diagnostics["duplicateTranscriptSegments"], 1);
    assert_eq!(
        diagnostics["duplicateTranscriptSegmentDetails"][0]["duplicateOfSegmentId"],
        1
    );
}

#[test]
fn alignment_diagnostics_reports_missing_anchor_ceiling() {
    let diagnostics = alignment_diagnostics(&[], &[], None, None);

    assert_eq!(diagnostics["missingAudioClock"], true);
    assert_eq!(diagnostics["missingRecordReplayStart"], true);
    assert_eq!(diagnostics["clockSkewStatus"], "missing-anchor");
    assert_eq!(
        diagnostics["claimCeiling"],
        "no alignment without audioStartedAtUnixMs"
    );
}

#[test]
fn timeline_rejects_excessive_alignment_work() {
    let error = ingestion::enforce_alignment_work(2_501, 2_000).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("timeline alignment exceeds 5000000 segment-event work item limit")
    );
    ingestion::enforce_alignment_work(2_500, 2_000).unwrap();
}

fn segment(id: usize, start_ms: u64, end_ms: u64, text: &str) -> TranscriptSegment {
    TranscriptSegment {
        id,
        start_ms,
        end_ms,
        monotonic_offset_ms: None,
        timing_source: "test".to_string(),
        text: text.to_string(),
    }
}

fn event(unix_ms: i64, kind: &str, ui_hint: Option<&str>) -> RnrEvent {
    RnrEvent {
        id: Some(7),
        kind: kind.to_string(),
        timestamp: None,
        unix_ms: Some(unix_ms),
        timestamp_parse_status: "valid".to_string(),
        app: Some("Test App".to_string()),
        window: None,
        ui_hint: ui_hint.map(str::to_string),
    }
}

#[cfg(unix)]
fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
