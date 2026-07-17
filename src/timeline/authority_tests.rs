use super::transcript::transcript_segments_checked;
use crate::safe_path::normalize_system_temp;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn final_alignment_without_current_receipt_is_non_authoritative() {
    let root = stale_alignment_session("nrr-alignment-no-receipt");

    let selected = transcript_segments_checked(&root).unwrap();

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].text, "current realtime words");
    assert_eq!(selected[0].timing_source, "process-local-monotonic-offset");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn disabled_or_malformed_receipt_denies_final_alignment_authority() {
    for (name, receipt) in [
        (
            "disabled",
            r#"{"schema":"narrated-record-replay.final-transcript-alignment-receipt.v1","status":"disabled","reason":"missing-cleaned-transcript"}"#,
        ),
        ("malformed", "not json"),
        (
            "unknown",
            r#"{"schema":"narrated-record-replay.final-transcript-alignment-receipt.v1","status":"trust-me"}"#,
        ),
    ] {
        let root = stale_alignment_session(&format!("nrr-alignment-{name}-receipt"));
        fs::write(
            root.join("final-transcript-alignment-receipt.json"),
            receipt,
        )
        .unwrap();

        let selected = transcript_segments_checked(&root).unwrap();

        assert_eq!(selected[0].text, "current realtime words", "case {name}");
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn current_digest_bound_receipt_grants_final_alignment_authority() {
    let root = stale_alignment_session("nrr-alignment-current-receipt");
    write_verified_alignment_contract(&root);

    let selected = transcript_segments_checked(&root).unwrap();

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].text, "verified aligned words");
    assert_eq!(
        selected[0].timing_source,
        "aligned-cleaned-batch-text-with-realtime-window"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stale_session_or_digest_mismatch_revokes_alignment_authority() {
    for mutation in ["session", "cleaned", "alignment", "policy"] {
        let root = stale_alignment_session(&format!("nrr-alignment-mismatch-{mutation}"));
        write_verified_alignment_contract(&root);
        match mutation {
            "session" => fs::write(root.join("manifest.json"), r#"{"session":"next"}"#).unwrap(),
            "cleaned" => fs::write(
                root.join("cleaned-transcript.json"),
                r#"{"cleanedText":"changed"}"#,
            )
            .unwrap(),
            "alignment" => {
                let mut text =
                    fs::read_to_string(root.join("final-transcript-alignment.json")).unwrap();
                text.push('\n');
                fs::write(root.join("final-transcript-alignment.json"), text).unwrap();
            }
            "policy" => {
                let text = fs::read_to_string(root.join("final-transcript-alignment-receipt.json"))
                    .unwrap()
                    .replace("nrr-cleanup-transform-v1", "unknown-policy");
                fs::write(root.join("final-transcript-alignment-receipt.json"), text).unwrap();
            }
            _ => unreachable!(),
        }

        let selected = transcript_segments_checked(&root).unwrap();

        assert_eq!(
            selected[0].text, "current realtime words",
            "case {mutation}"
        );
        fs::remove_dir_all(root).unwrap();
    }
}

fn write_verified_alignment_contract(root: &PathBuf) {
    fs::write(root.join("manifest.json"), r#"{"session":"current"}"#).unwrap();
    fs::write(root.join("batch-transcript.json"), r#"{"text":"batch"}"#).unwrap();
    fs::write(
        root.join("cleaned-transcript.json"),
        r#"{"cleanedText":"verified aligned words"}"#,
    )
    .unwrap();
    fs::write(
        root.join("cleanup-receipt.json"),
        r#"{"status":"completed"}"#,
    )
    .unwrap();
    fs::write(
        root.join("final-transcript.timeline.jsonl"),
        r#"{"startMs":10,"endMs":20,"text":"verified aligned words"}"#,
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
        "wordAuthority": "verified-cleaned-batch-transcript"
    });
    let artifact = json!({
        "schema": "narrated-record-replay.final-transcript-alignment.v2",
        "wordAuthority": "verified-cleaned-batch-transcript",
        "sourceBinding": binding,
        "segments": [{"startMs": 10, "endMs": 20, "text": "verified aligned words"}]
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
        "wordAuthority": "verified-cleaned-batch-transcript"
    });
    fs::write(
        root.join("final-transcript-alignment-receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
}

fn session_identity(root: &PathBuf) -> String {
    let normalized = normalize_system_temp(root);
    let manifest = file_sha256(&normalized.join("manifest.json"));
    sha256(format!("session-binding-v2\0{}\0{manifest}", normalized.display()).as_bytes())
}

fn file_sha256(path: &PathBuf) -> String {
    sha256(&fs::read(path).unwrap())
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn stale_alignment_session(prefix: &str) -> PathBuf {
    let root = unique_tmp(prefix);
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("final-transcript-alignment.json"),
        r#"{"segments":[{"startMs":10,"endMs":20,"text":"stale aligned words"}]}"#,
    )
    .unwrap();
    fs::write(
        root.join("transcript.timeline.jsonl"),
        r#"{"kind":"completed","monotonicOffsetMs":30,"text":"current realtime words"}"#,
    )
    .unwrap();
    root
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
