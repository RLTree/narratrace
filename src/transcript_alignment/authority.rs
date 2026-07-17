const FINAL_ALIGNMENT_SCHEMA: &str = "narrated-record-replay.final-transcript-alignment.v2";
const FINAL_ALIGNMENT_RECEIPT_SCHEMA: &str =
    "narrated-record-replay.final-transcript-alignment-receipt.v2";
const FINAL_ALIGNMENT_POLICY_VERSION: &str = "nrr-final-alignment-v2";

pub fn ensure_final_alignment(session_dir: &Path) -> Result<Option<PathBuf>> {
    let cleaned = match crate::transcript_cleanup::verified_cleaned_for_alignment(session_dir) {
        Ok(cleaned) => cleaned,
        Err(error) => {
            write_disabled(
                session_dir,
                &format!("unverified-cleaned-transcript: {error}"),
            )?;
            return Ok(None);
        }
    };
    let realtime_path = session_dir.join("transcript.timeline.jsonl");
    let (_, realtime_sha256) = match alignment_read_hash(&realtime_path) {
        Ok(value) => value,
        Err(error) => {
            write_disabled(
                session_dir,
                &format!("unverified-realtime-timeline: {error}"),
            )?;
            return Ok(None);
        }
    };
    let realtime_segments = timeline::raw_realtime_segments(session_dir);
    require_alignment_digest(&realtime_path, &realtime_sha256)?;
    let final_segments = align_cleaned_text(&cleaned.text, &realtime_segments)?;
    require_alignment_digest(&realtime_path, &realtime_sha256)?;
    let unresolved = final_segments
        .iter()
        .filter(|segment| segment.mismatch.is_some())
        .count();
    let artifact_path = session_dir.join("final-transcript-alignment.json");
    let source_binding = json!({
        "sessionIdentity": cleaned.session_identity,
        "batchTranscriptSha256": cleaned.batch_artifact_sha256,
        "cleanedTranscriptSha256": cleaned.artifact_sha256,
        "cleanupReceiptSha256": cleaned.receipt_sha256,
        "realtimeTimelineSha256": realtime_sha256,
        "cleanupValidationStatus": "verified-conservative-transform",
        "cleanupValidationPolicyVersion": cleaned.validation_policy_version,
        "alignmentPolicyVersion": FINAL_ALIGNMENT_POLICY_VERSION
    });
    let artifact = serde_json::to_string_pretty(&json!({
        "schema": FINAL_ALIGNMENT_SCHEMA,
        "wordAuthority": "verified-cleaned-batch-transcript",
        "timingAuthority": "realtime-transcript-segments",
        "alignmentPolicyVersion": FINAL_ALIGNMENT_POLICY_VERSION,
        "status": if unresolved == 0 { "aligned" } else { "aligned-with-review-warnings" },
        "unresolvedMismatches": unresolved,
        "segments": final_segments.iter().map(final_segment_json).collect::<Vec<_>>(),
        "sourceBinding": source_binding,
        "privacy": {
            "localPrivate": true,
            "rawSourcesCopiedIntoGeneratedPacketsByDefault": false,
            "humanFacingReviewPrefersAlignedFinal": true
        }
    }))?;
    write_private(&artifact_path, artifact.as_bytes())?;
    let final_alignment_sha256 = alignment_sha256(artifact.as_bytes());
    write_final_timeline(session_dir, &final_segments)?;
    let (_, final_timeline_sha256) =
        alignment_read_hash(&session_dir.join("final-transcript.timeline.jsonl"))?;
    require_alignment_sources(session_dir, &cleaned, &realtime_sha256)?;
    write_private(
        session_dir.join("final-transcript-alignment-receipt.json"),
        serde_json::to_string_pretty(&json!({
            "schema": FINAL_ALIGNMENT_RECEIPT_SCHEMA,
            "status": "completed",
            "sessionIdentity": cleaned.session_identity,
            "batchTranscriptSha256": cleaned.batch_artifact_sha256,
            "cleanedTranscriptSha256": cleaned.artifact_sha256,
            "cleanupReceiptSha256": cleaned.receipt_sha256,
            "realtimeTimelineSha256": realtime_sha256,
            "finalAlignmentSha256": final_alignment_sha256,
            "finalTimelineSha256": final_timeline_sha256,
            "cleanupValidationStatus": "verified-conservative-transform",
            "cleanupValidationPolicyVersion": cleaned.validation_policy_version,
            "alignmentPolicyVersion": FINAL_ALIGNMENT_POLICY_VERSION,
            "wordAuthority": "verified-cleaned-batch-transcript"
        }))?,
    )?;
    Ok(Some(artifact_path))
}

fn require_alignment_sources(
    session_dir: &Path,
    cleaned: &crate::transcript_cleanup::VerifiedCleanedTranscript,
    realtime_sha256: &str,
) -> Result<()> {
    for (path, expected) in [
        (
            session_dir.join("batch-transcript.json"),
            cleaned.batch_artifact_sha256.as_str(),
        ),
        (
            session_dir.join("cleaned-transcript.json"),
            cleaned.artifact_sha256.as_str(),
        ),
        (
            session_dir.join("cleanup-receipt.json"),
            cleaned.receipt_sha256.as_str(),
        ),
        (
            session_dir.join("transcript.timeline.jsonl"),
            realtime_sha256,
        ),
    ] {
        require_alignment_digest(&path, expected)?;
    }
    Ok(())
}

fn require_alignment_digest(path: &Path, expected: &str) -> Result<()> {
    let (_, actual) = alignment_read_hash(path)?;
    if actual != expected {
        anyhow::bail!("alignment source changed while deriving final authority");
    }
    Ok(())
}

fn alignment_read_hash(path: &Path) -> Result<(Vec<u8>, String)> {
    let bytes = crate::safe_path::read_regular_file_bounded(path, MAX_FINAL_ALIGNMENT_BYTES)?;
    let digest = alignment_sha256(&bytes);
    Ok((bytes, digest))
}

fn alignment_sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn authoritative_alignment_bytes(session_dir: &Path) -> Option<Vec<u8>> {
    let cleaned = crate::transcript_cleanup::verified_cleaned_for_alignment(session_dir).ok()?;
    let (artifact, artifact_sha256) =
        alignment_read_hash(&session_dir.join("final-transcript-alignment.json")).ok()?;
    let (receipt_bytes, _) =
        alignment_read_hash(&session_dir.join("final-transcript-alignment-receipt.json")).ok()?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes).ok()?;
    let artifact_json: Value = serde_json::from_slice(&artifact).ok()?;
    if receipt.get("schema").and_then(Value::as_str) != Some(FINAL_ALIGNMENT_RECEIPT_SCHEMA)
        || receipt.get("status").and_then(Value::as_str) != Some("completed")
        || receipt.get("finalAlignmentSha256").and_then(Value::as_str)
            != Some(artifact_sha256.as_str())
        || receipt.get("sessionIdentity").and_then(Value::as_str)
            != Some(cleaned.session_identity.as_str())
        || receipt.get("batchTranscriptSha256").and_then(Value::as_str)
            != Some(cleaned.batch_artifact_sha256.as_str())
        || receipt
            .get("cleanedTranscriptSha256")
            .and_then(Value::as_str)
            != Some(cleaned.artifact_sha256.as_str())
        || receipt.get("cleanupReceiptSha256").and_then(Value::as_str)
            != Some(cleaned.receipt_sha256.as_str())
        || receipt
            .get("cleanupValidationStatus")
            .and_then(Value::as_str)
            != Some("verified-conservative-transform")
        || receipt
            .get("cleanupValidationPolicyVersion")
            .and_then(Value::as_str)
            != Some(cleaned.validation_policy_version.as_str())
        || receipt
            .get("alignmentPolicyVersion")
            .and_then(Value::as_str)
            != Some(FINAL_ALIGNMENT_POLICY_VERSION)
        || receipt.get("wordAuthority").and_then(Value::as_str)
            != Some("verified-cleaned-batch-transcript")
        || artifact_json.get("schema").and_then(Value::as_str) != Some(FINAL_ALIGNMENT_SCHEMA)
        || artifact_json.get("wordAuthority").and_then(Value::as_str)
            != Some("verified-cleaned-batch-transcript")
    {
        return None;
    }
    for (field, path) in [
        ("realtimeTimelineSha256", "transcript.timeline.jsonl"),
        ("finalTimelineSha256", "final-transcript.timeline.jsonl"),
    ] {
        let (_, actual) = alignment_read_hash(&session_dir.join(path)).ok()?;
        if receipt.get(field).and_then(Value::as_str) != Some(actual.as_str()) {
            return None;
        }
    }
    Some(artifact)
}
