use crate::safe_path::{normalize_system_temp, read_regular_file_bounded};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

const MAX_BOUND_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SESSION_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const RECEIPT_SCHEMA: &str = "narrated-record-replay.final-transcript-alignment-receipt.v2";
const ARTIFACT_SCHEMA: &str = "narrated-record-replay.final-transcript-alignment.v2";
const VERIFIED_STATUS: &str = "verified-conservative-transform";
const VERIFIED_AUTHORITY: &str = "verified-cleaned-batch-transcript";
const CLEANUP_POLICY_VERSION: &str = "nrr-cleanup-transform-v1";
const ALIGNMENT_POLICY_VERSION: &str = "nrr-final-alignment-v2";

pub(super) fn verified_final_alignment_bytes(session_dir: &Path) -> Option<Vec<u8>> {
    let (alignment_bytes, final_alignment_sha256) =
        read_and_hash(&session_dir.join("final-transcript-alignment.json"))?;
    let (receipt_bytes, _) =
        read_and_hash(&session_dir.join("final-transcript-alignment-receipt.json"))?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes).ok()?;
    let artifact: Value = serde_json::from_slice(&alignment_bytes).ok()?;

    if receipt.get("schema").and_then(Value::as_str) != Some(RECEIPT_SCHEMA)
        || receipt.get("status").and_then(Value::as_str) != Some("completed")
        || receipt
            .get("cleanupValidationStatus")
            .and_then(Value::as_str)
            != Some(VERIFIED_STATUS)
        || receipt.get("wordAuthority").and_then(Value::as_str) != Some(VERIFIED_AUTHORITY)
        || receipt
            .get("cleanupValidationPolicyVersion")
            .and_then(Value::as_str)
            != Some(CLEANUP_POLICY_VERSION)
        || receipt
            .get("alignmentPolicyVersion")
            .and_then(Value::as_str)
            != Some(ALIGNMENT_POLICY_VERSION)
        || receipt.get("finalAlignmentSha256").and_then(Value::as_str)
            != Some(final_alignment_sha256.as_str())
        || artifact.get("schema").and_then(Value::as_str) != Some(ARTIFACT_SCHEMA)
        || artifact.get("wordAuthority").and_then(Value::as_str) != Some(VERIFIED_AUTHORITY)
    {
        return None;
    }

    let session_identity = current_session_identity(session_dir)?;
    if receipt.get("sessionIdentity").and_then(Value::as_str) != Some(session_identity.as_str()) {
        return None;
    }

    for (field, filename) in [
        ("batchTranscriptSha256", "batch-transcript.json"),
        ("cleanedTranscriptSha256", "cleaned-transcript.json"),
        ("cleanupReceiptSha256", "cleanup-receipt.json"),
        ("realtimeTimelineSha256", "transcript.timeline.jsonl"),
        ("finalTimelineSha256", "final-transcript.timeline.jsonl"),
    ] {
        let (_, actual) = read_and_hash(&session_dir.join(filename))?;
        if receipt.get(field).and_then(Value::as_str) != Some(actual.as_str()) {
            return None;
        }
    }

    let binding = artifact.get("sourceBinding")?;
    for field in [
        "sessionIdentity",
        "batchTranscriptSha256",
        "cleanedTranscriptSha256",
        "cleanupReceiptSha256",
        "realtimeTimelineSha256",
        "cleanupValidationStatus",
        "cleanupValidationPolicyVersion",
        "alignmentPolicyVersion",
    ] {
        let receipt_value = receipt.get(field).and_then(Value::as_str)?;
        if binding.get(field).and_then(Value::as_str) != Some(receipt_value) {
            return None;
        }
    }
    Some(alignment_bytes)
}

fn current_session_identity(session_dir: &Path) -> Option<String> {
    let normalized = normalize_system_temp(session_dir);
    let manifest = normalized.join("manifest.json");
    let manifest_sha256 = if std::fs::symlink_metadata(&manifest).is_ok() {
        let bytes = read_regular_file_bounded(&manifest, MAX_SESSION_MANIFEST_BYTES).ok()?;
        sha256(&bytes)
    } else {
        "manifest-absent".to_string()
    };
    Some(sha256(
        format!(
            "session-binding-v2\0{}\0{manifest_sha256}",
            normalized.display()
        )
        .as_bytes(),
    ))
}

fn read_and_hash(path: &Path) -> Option<(Vec<u8>, String)> {
    let bytes = read_regular_file_bounded(path, MAX_BOUND_ARTIFACT_BYTES).ok()?;
    let digest = sha256(&bytes);
    Some((bytes, digest))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
