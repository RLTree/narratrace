use crate::safe_path::open_regular_file;
use sha2::{Digest, Sha256};
use std::io::Read;

const REVIEW_PROOF_MAX_BYTES: u64 = 64 * 1024 * 1024;

fn review_proofs_valid(session_dir: &Path, expected_run_id: Option<&str>) -> bool {
    let packet_inspection =
        read_json(&session_dir.join("packet-inspection.json")).unwrap_or(Value::Null);
    let dogfood_receipt =
        read_json(&session_dir.join("dogfood-receipt.json")).unwrap_or(Value::Null);
    let replay_plan =
        read_json(&session_dir.join("replay-voice-execution-plan.json")).unwrap_or(Value::Null);
    let transcript_quality = transcript_quality_state(session_dir);
    let final_alignment =
        read_json(&session_dir.join("final-transcript-alignment.json")).unwrap_or(Value::Null);
    let Some(run_id) = expected_run_id.filter(|id| !id.trim().is_empty()) else {
        return false;
    };
    dogfood_receipt.get("schema").and_then(Value::as_str)
        == Some("narrated-record-replay.dogfood-receipt.v1")
        && dogfood_receipt.get("status").and_then(Value::as_str) == Some("requires-operator-review")
        && dogfood_receipt
            .pointer("/runBinding/runId")
            .and_then(Value::as_str)
            == Some(run_id)
        && dogfood_receipt
            .pointer("/evidenceTrust/status")
            .and_then(Value::as_str)
            == Some("verified-current-run")
        && dogfood_receipt.pointer("/evidenceTrust/parentOperationVerified")
            == Some(&Value::Bool(true))
        && dogfood_receipt
            .get("blockers")
            .and_then(Value::as_array)
            .is_some_and(|blockers| {
                blockers.as_slice()
                    == [Value::String(
                        "operator review of generated artifacts is still required".to_string(),
                    )]
            })
        && strict_packet_inspection(&packet_inspection)
        && replay_voice_plan_valid(&replay_plan)
        && transcript_quality.is_complete()
        && final_alignment.get("schema").and_then(Value::as_str)
            == Some("narrated-record-replay.final-transcript-alignment.v1")
        && final_alignment.get("status").and_then(Value::as_str) == Some("aligned")
        && final_alignment
            .get("unresolvedMismatches")
            .and_then(Value::as_u64)
            == Some(0)
        && [
            ("temporal-context", "temporal-context.json"),
            ("packet-inspection", "packet-inspection.json"),
            (
                "replay-voice-execution-plan",
                "replay-voice-execution-plan.json",
            ),
            (
                "batch-transcription-receipt",
                "batch-transcription-receipt.json",
            ),
            ("cleanup-receipt", "cleanup-receipt.json"),
            (
                "final-transcript-alignment-receipt",
                "final-transcript-alignment-receipt.json",
            ),
            (
                "final-transcript-alignment",
                "final-transcript-alignment.json",
            ),
        ]
        .iter()
        .all(|(name, file)| bound_artifact_matches(session_dir, &dogfood_receipt, name, file))
}

fn strict_packet_inspection(value: &Value) -> bool {
    value.get("schema").and_then(Value::as_str)
        == Some("narrated-record-replay.packet-inspection.v1")
        && value.get("status").and_then(Value::as_str) == Some("requires-real-packet-review")
        && value.pointer("/privacyBoundary/allowedToShareWithoutReview")
            == Some(&Value::Bool(false))
        && value
            .pointer("/privacyBoundary/generatedArtifactLeakScan/status")
            .and_then(Value::as_str)
            .is_some_and(|status| {
                matches!(
                    status,
                    "no-obvious-sensitive-patterns-detected" | "expected-local-references-only"
                )
            })
        && value
            .pointer("/privacyBoundary/generatedArtifactLeakScan/findings")
            .and_then(Value::as_array)
            .is_some()
        && value
            .get("blockers")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
}

fn bound_artifact_matches(
    session_dir: &Path,
    dogfood_receipt: &Value,
    name: &str,
    file: &str,
) -> bool {
    let expected_path = session_dir.join(file);
    let Some(entry) = dogfood_receipt
        .pointer("/artifactEvidence/generatedReviewCandidates")
        .and_then(Value::as_array)
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry.get("name").and_then(Value::as_str) == Some(name))
        })
    else {
        return false;
    };
    entry.get("path").and_then(Value::as_str) == Some(expected_path.to_string_lossy().as_ref())
        && entry.get("exists").and_then(Value::as_bool) == Some(true)
        && entry.get("isRegularFile").and_then(Value::as_bool) == Some(true)
        && entry.get("contentFingerprint").and_then(Value::as_str)
            == artifact_digest(&expected_path).as_deref()
}

fn artifact_digest(path: &Path) -> Option<String> {
    let mut file = open_regular_file(path).ok()?;
    if file.metadata().ok()?.len() > REVIEW_PROOF_MAX_BYTES {
        return None;
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Some(format!("sha256:{:x}", hasher.finalize()))
}
