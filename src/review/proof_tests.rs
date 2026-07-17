use super::*;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn caller_authored_review_proofs_do_not_generate_trust() {
    let root = unique_tmp("nrr-forged-review-proofs");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("packet-inspection.json"), "not-json").unwrap();
    fs::write(
        root.join("dogfood-receipt.json"),
        r#"{"schema":"narrated-record-replay.dogfood-receipt.v1","status":"requires-operator-review","runBinding":{"runId":"run-1"},"evidenceTrust":{"status":"verified-current-run","parentOperationVerified":true}}"#,
    )
    .unwrap();
    fs::write(
        root.join("replay-voice-execution-plan.json"),
        r#"{"schema":"narrated-record-replay.replay-voice-execution-plan.v1","status":"caller-chosen-success","cueCount":0,"proofBoundary":{"speaksAudio":false},"cues":[]}"#,
    )
    .unwrap();
    for file in [
        "batch-transcription-receipt.json",
        "cleanup-receipt.json",
        "final-transcript-alignment-receipt.json",
    ] {
        fs::write(root.join(file), r#"{"status":"forged-success"}"#).unwrap();
    }

    assert!(!review_proofs_valid(&root, Some("run-1")));
}

#[test]
fn exact_current_run_review_proofs_are_accepted_and_mutation_is_rejected() {
    let root = unique_tmp("nrr-bound-review-proofs");
    fs::create_dir_all(&root).unwrap();
    write_valid_review_inputs(&root);
    write_bound_dogfood_receipt(&root, "run-1");

    assert!(review_proofs_valid(&root, Some("run-1")));
    assert!(!review_proofs_valid(&root, Some("other-run")));

    fs::write(
        root.join("packet-inspection.json"),
        r#"{"schema":"narrated-record-replay.packet-inspection.v1","status":"requires-real-packet-review","privacyBoundary":{"allowedToShareWithoutReview":false,"generatedArtifactLeakScan":{"status":"blocked","findings":[]}}}"#,
    )
    .unwrap();
    assert!(!review_proofs_valid(&root, Some("run-1")));
}

fn write_valid_review_inputs(root: &Path) {
    let fixtures = [
        (
            "temporal-context.json",
            r#"{"schema":"narrated-record-replay.temporal-context.v1"}"#,
        ),
        (
            "packet-inspection.json",
            r#"{"schema":"narrated-record-replay.packet-inspection.v1","status":"requires-real-packet-review","privacyBoundary":{"allowedToShareWithoutReview":false,"generatedArtifactLeakScan":{"status":"no-obvious-sensitive-patterns-detected","findings":[]}},"blockers":[]}"#,
        ),
        (
            "replay-voice-execution-plan.json",
            r#"{"schema":"narrated-record-replay.replay-voice-execution-plan.v1","status":"dry-run-not-spoken","cueCount":0,"proofBoundary":{"speaksAudio":false},"cues":[]}"#,
        ),
        (
            "batch-transcription-receipt.json",
            r#"{"schema":"narrated-record-replay.batch-transcription-receipt.v1","status":"completed"}"#,
        ),
        (
            "cleanup-receipt.json",
            r#"{"schema":"narrated-record-replay.cleanup-receipt.v1","status":"completed"}"#,
        ),
        (
            "final-transcript-alignment-receipt.json",
            r#"{"schema":"narrated-record-replay.final-transcript-alignment-receipt.v1","status":"completed"}"#,
        ),
        (
            "final-transcript-alignment.json",
            r#"{"schema":"narrated-record-replay.final-transcript-alignment.v1","status":"aligned","unresolvedMismatches":0}"#,
        ),
    ];
    for (file, body) in fixtures {
        fs::write(root.join(file), body).unwrap();
    }
}

fn write_bound_dogfood_receipt(root: &Path, run_id: &str) {
    let artifacts = [
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
    .map(|(name, file)| {
        let path = root.join(file);
        json!({
            "name": name,
            "path": path,
            "exists": true,
            "isRegularFile": true,
            "contentFingerprint": artifact_digest(&path).unwrap()
        })
    });
    let receipt = json!({
        "schema": "narrated-record-replay.dogfood-receipt.v1",
        "status": "requires-operator-review",
        "runBinding": {"runId": run_id},
        "evidenceTrust": {
            "status": "verified-current-run",
            "parentOperationVerified": true
        },
        "blockers": ["operator review of generated artifacts is still required"],
        "artifactEvidence": {"generatedReviewCandidates": artifacts}
    });
    fs::write(
        root.join("dogfood-receipt.json"),
        serde_json::to_string(&receipt).unwrap(),
    )
    .unwrap();
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
