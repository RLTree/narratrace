use super::build_temporal_context;
use crate::safe_path::normalize_system_temp;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn authorized_oversized_alignment_fails_before_json_work() {
    let root = unique_tmp("nrr-authorized-alignment-byte-bound");
    write_authorized_alignment(
        &root,
        json!([{"startMs": 0, "endMs": 1, "text": "ok"}]),
        Some("x".repeat(8 * 1024 * 1024)),
    );

    let error = build_temporal_context(&root, None, None).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("alignment artifact exceeds 8388608 byte limit")
    );
}

#[test]
fn authorized_alignment_rejects_excessive_segments() {
    let root = unique_tmp("nrr-authorized-alignment-segment-bound");
    let segments = vec![json!({"startMs": 0, "endMs": 1, "text": "ok"}); 10_001];
    write_authorized_alignment(&root, Value::Array(segments), None);

    let error = build_temporal_context(&root, None, None).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("alignment artifact exceeds 10000 segment limit")
    );
}

fn write_authorized_alignment(root: &Path, segments: Value, padding: Option<String>) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("manifest.json"), r#"{"session":"current"}"#).unwrap();
    fs::write(root.join("batch-transcript.json"), r#"{"text":"batch"}"#).unwrap();
    fs::write(
        root.join("cleaned-transcript.json"),
        r#"{"cleanedText":"ok"}"#,
    )
    .unwrap();
    fs::write(
        root.join("cleanup-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        r#"{"kind":"completed","monotonicOffsetMs":1,"text":"realtime"}"#,
    )
    .unwrap();
    fs::write(
        root.join("final-transcript.timeline.jsonl"),
        r#"{"startMs":0,"endMs":1,"text":"ok"}"#,
    )
    .unwrap();

    let session_identity = session_identity(root);
    let batch = file_sha256(&root.join("batch-transcript.json"));
    let cleaned = file_sha256(&root.join("cleaned-transcript.json"));
    let cleanup_receipt = file_sha256(&root.join("cleanup-receipt.json"));
    let realtime = file_sha256(&root.join("transcript.timeline.jsonl"));
    let final_timeline = file_sha256(&root.join("final-transcript.timeline.jsonl"));
    let binding = json!({
        "sessionIdentity": session_identity,
        "batchTranscriptSha256": batch,
        "cleanedTranscriptSha256": cleaned,
        "cleanupReceiptSha256": cleanup_receipt,
        "realtimeTimelineSha256": realtime,
        "cleanupValidationStatus": "verified-conservative-transform",
        "cleanupValidationPolicyVersion": "nrr-cleanup-transform-v1",
        "alignmentPolicyVersion": "nrr-final-alignment-v2",
    });
    let artifact = json!({
        "schema": "narrated-record-replay.final-transcript-alignment.v2",
        "wordAuthority": "verified-cleaned-batch-transcript",
        "sourceBinding": binding,
        "segments": segments,
        "padding": padding,
    });
    fs::write(
        root.join("final-transcript-alignment.json"),
        serde_json::to_vec(&artifact).unwrap(),
    )
    .unwrap();
    let final_alignment = file_sha256(&root.join("final-transcript-alignment.json"));
    let receipt = json!({
        "schema": "narrated-record-replay.final-transcript-alignment-receipt.v2",
        "status": "completed",
        "sessionIdentity": session_identity,
        "batchTranscriptSha256": batch,
        "cleanedTranscriptSha256": cleaned,
        "cleanupReceiptSha256": cleanup_receipt,
        "realtimeTimelineSha256": realtime,
        "finalAlignmentSha256": final_alignment,
        "finalTimelineSha256": final_timeline,
        "cleanupValidationStatus": "verified-conservative-transform",
        "cleanupValidationPolicyVersion": "nrr-cleanup-transform-v1",
        "alignmentPolicyVersion": "nrr-final-alignment-v2",
        "wordAuthority": "verified-cleaned-batch-transcript",
    });
    fs::write(
        root.join("final-transcript-alignment-receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
}

fn session_identity(root: &Path) -> String {
    let normalized = normalize_system_temp(root);
    let manifest = file_sha256(&normalized.join("manifest.json"));
    sha256(format!("session-binding-v2\0{}\0{manifest}", normalized.display()).as_bytes())
}

fn file_sha256(path: &Path) -> String {
    sha256(&fs::read(path).unwrap())
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
