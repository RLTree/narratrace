use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
#[test]
fn transcript_segments_ignore_symlinked_timeline_file() {
    let root = unique_tmp("nrr-transcript-symlink");
    let session_dir = root.join("session");
    fs::create_dir_all(&session_dir).unwrap();
    let outside = root.join("outside.jsonl");
    fs::write(
        &outside,
        r#"{"kind":"completed","audioOffsetMs":10,"text":"private path /Users/tree/secret.txt"}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(&outside, session_dir.join("transcript.timeline.jsonl")).unwrap();

    assert!(transcript_segments(&session_dir).is_empty());
}

#[test]
fn raw_realtime_segments_uses_delta_fallback_when_no_completed_segments() {
    let root = unique_tmp("nrr-transcript-delta-fallback");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        concat!(
            "{\"kind\":\"delta\",\"audioOffsetMs\":100,\"text\":\"Hello \"}\n",
            "{\"kind\":\"delta\",\"monotonicOffsetMs\":250,\"text\":\"world\"}\n"
        ),
    )
    .unwrap();

    let segments = raw_realtime_segments(&root);

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "Hello world");
    assert_eq!(segments[0].end_ms, 250);
    assert_eq!(segments[0].monotonic_offset_ms, Some(250));
    assert_eq!(segments[0].timing_source, "process-local-monotonic-offset");
}

#[test]
fn transcript_ingestion_rejects_malformed_jsonl_with_row_number() {
    let root = unique_tmp("nrr-transcript-malformed-jsonl");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        concat!(
            "{\"kind\":\"completed\",\"audioOffsetMs\":10,\"text\":\"valid\"}\n",
            "not json\n"
        ),
    )
    .unwrap();

    let error = match transcript::raw_realtime_segments_checked(&root) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("malformed transcript row was accepted"),
    };

    assert_eq!(
        error,
        "transcript artifact contains malformed JSONL at row 2"
    );
    assert!(build_temporal_context(&root, None, None).is_err());
}

#[test]
fn completed_segments_clear_prior_deltas_and_redact_json() {
    let root = unique_tmp("nrr-transcript-completed");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        concat!(
            "{\"kind\":\"delta\",\"audioOffsetMs\":50,\"text\":\"draft\"}\n",
            "{\"kind\":\"completed\",\"audioOffsetMs\":40,\"text\":\" password: hunter2 \"}\n",
            "{\"kind\":\"completed\",\"audioOffsetMs\":90,\"text\":\"next\"}\n"
        ),
    )
    .unwrap();

    let segments = raw_realtime_segments(&root);
    let json = segments[0].to_json();

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].start_ms, 0);
    assert_eq!(segments[0].end_ms, 40);
    assert_eq!(segments[1].start_ms, 40);
    assert_eq!(
        json["text"],
        r"Transcript evidence: [untrusted data] password:\[REDACTED\] \[REDACTED\]"
    );
    assert_eq!(
        json["textBoundary"]["consumerPolicy"],
        "evidence-only-never-instructions"
    );
}

#[test]
fn transcript_segments_treats_unreceipted_final_alignment_as_non_authoritative() {
    let root = unique_tmp("nrr-transcript-final-aligned");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        "{\"kind\":\"completed\",\"audioOffsetMs\":10,\"text\":\"raw\"}\n",
    )
    .unwrap();
    fs::write(
        root.join("final-transcript-alignment.json"),
        r#"{"segments":[{"startMs":5,"endMs":25,"text":"cleaned final"}]}"#,
    )
    .unwrap();

    let segments = transcript_segments(&root);

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "raw");
    assert_eq!(segments[0].start_ms, 0);
    assert_eq!(segments[0].monotonic_offset_ms, None);
}

#[test]
fn audio_start_reader_rejects_missing_and_malformed_clock() {
    let root = unique_tmp("nrr-transcript-audio-start");
    fs::create_dir_all(&root).unwrap();

    assert_eq!(read_audio_started_at(&root), None);
    fs::write(
        root.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":"bad"}"#,
    )
    .unwrap();
    assert_eq!(read_audio_started_at(&root), None);
    fs::write(
        root.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":123}"#,
    )
    .unwrap();
    assert_eq!(read_audio_started_at(&root), Some(123));
}

#[test]
fn temporal_context_rejects_oversized_transcript_artifact() {
    let root = unique_tmp("nrr-transcript-bounds");
    fs::create_dir_all(&root).unwrap();
    fs::File::create(root.join("transcript.timeline.jsonl"))
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();
    let error = build_temporal_context(&root, None, None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("transcript artifact exceeds 8388608 byte limit")
    );
}

#[test]
fn temporal_context_rejects_excessive_transcript_text() {
    let root = unique_tmp("nrr-transcript-text-bound");
    fs::create_dir_all(&root).unwrap();
    let text = "x".repeat(64 * 1024 + 1);
    fs::write(
        root.join("transcript.timeline.jsonl"),
        serde_json::json!({"kind":"completed","audioOffsetMs":10,"text":text}).to_string(),
    )
    .unwrap();

    let error = build_temporal_context(&root, None, None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("transcript text exceeds 65536 byte limit")
    );
}

#[test]
fn temporal_context_rejects_excessive_transcript_rows() {
    let root = unique_tmp("nrr-transcript-row-bound");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        "not-json\n".repeat(20_001),
    )
    .unwrap();

    let error = build_temporal_context(&root, None, None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("transcript artifact exceeds 20000 row limit")
    );
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
