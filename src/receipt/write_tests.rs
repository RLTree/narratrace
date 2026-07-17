use super::*;
use crate::config::parse_args_from;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn write_receipt_records_metadata_without_raw_private_content() {
    let root = unique_tmp("nrr-write-receipt");
    let session_dir = root.join("session");
    let rnr_dir = root.join("rnr");
    fs::create_dir_all(&session_dir).unwrap();
    fs::create_dir_all(&rnr_dir).unwrap();
    let metadata_path = rnr_dir.join("session.json");
    let events_path = rnr_dir.join("events.jsonl");
    fs::write(
        &metadata_path,
        r#"{"id":"rnr-1","startedAt":"2026-06-19T01:38:25Z"}"#,
    )
    .unwrap();
    fs::write(
        &events_path,
        r#"{"id":1,"kind":"click","timestamp":"2026-06-19T01:38:26Z"}"#,
    )
    .unwrap();
    write_fixture_session(&session_dir);
    let args = parse_args_from([
        "nrr",
        "receipt",
        "--session-dir",
        session_dir.to_str().unwrap(),
        "--recording-metadata",
        metadata_path.to_str().unwrap(),
        "--recording-events",
        events_path.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
        "--receipt-run-id",
        "trusted-dogfood-run",
    ])
    .unwrap();
    write_current_parent_receipt(&args);

    write_receipt(&args).unwrap();

    let receipt = fs::read_to_string(session_dir.join("dogfood-receipt.json")).unwrap();
    assert!(receipt.contains("narrated-record-replay.dogfood-receipt.v1"));
    assert!(receipt.contains("\"rawTextCopiedIntoReceipt\": false"));
    assert!(receipt.contains("\"status\": \"blocked\""));
    assert!(receipt.contains("\"status\": \"blocked-untrusted-or-incomplete\""));
    assert!(receipt.contains("\"runId\": \"trusted-dogfood-run\""));
    assert!(receipt.contains("trusted external current-run execution attestation is unavailable"));
    assert!(session_dir.join("review-contract.json").is_file());
    assert!(session_dir.join("review-artifact.html").is_file());
}

#[test]
fn caller_authored_success_json_remains_blocked_without_trusted_run_binding() {
    let root = unique_tmp("nrr-untrusted-write-receipt");
    let session_dir = root.join("session");
    let rnr_dir = root.join("rnr");
    fs::create_dir_all(&session_dir).unwrap();
    fs::create_dir_all(&rnr_dir).unwrap();
    let metadata_path = rnr_dir.join("session.json");
    let events_path = rnr_dir.join("events.jsonl");
    fs::write(
        &metadata_path,
        r#"{"id":"rnr-1","startedAt":"2026-06-19T01:38:25Z"}"#,
    )
    .unwrap();
    fs::write(
        &events_path,
        r#"{"id":1,"kind":"click","timestamp":"2026-06-19T01:38:26Z"}"#,
    )
    .unwrap();
    write_fixture_session(&session_dir);
    fs::write(
        session_dir.join("parent-operation-receipt.json"),
        r#"{"status":"timestamp-proximity-verified","sameStartChecks":{"startDeltaMs":0,"withinAllowedStartDelta":true,"recordReplayEventsPresent":true,"microphoneStoppedCleanly":true,"postCommitDrainCompleted":true}}"#,
    )
    .unwrap();
    let args = parse_args_from([
        "nrr",
        "receipt",
        "--session-dir",
        session_dir.to_str().unwrap(),
        "--recording-metadata",
        metadata_path.to_str().unwrap(),
        "--recording-events",
        events_path.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    write_receipt(&args).unwrap();

    let receipt: Value = serde_json::from_str(
        &fs::read_to_string(session_dir.join("dogfood-receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["status"], "blocked");
    assert_eq!(
        receipt["evidenceTrust"]["status"],
        "blocked-untrusted-or-incomplete"
    );
    assert!(
        receipt["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "trusted current-run receipt id is missing")
    );
}

fn write_fixture_session(session_dir: &Path) {
    fs::write(
        session_dir.join("manifest.json"),
        r#"{"schema":"narrated-record-replay.v1","goal":"fixture","startCoordination":{"recordReplayAndMicrophoneSameOperation":true}}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("status.json"),
        r#"{"state":"stopped","model":"gpt-realtime-whisper","audioInput":{"name":"MacBook Microphone"}}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833105000}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("post-commit-drain.json"),
        r#"{"schema":"narrated-record-replay.post-commit-drain.v1","completedSegments":1,"messages":3,"errors":[]}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("temporal-context.json"),
        r#"{"schema":"narrated-record-replay.temporal-context.v1","transcriptSegments":[{"id":"t1"}],"recordReplayEvents":[{"id":"e1"}],"alignments":[{"id":"a1"}],"conflictDiagnostics":{"warnings":[]},"alignmentDiagnostics":{"claimCeiling":"fixture","outOfWindowRecordReplayEvents":0},"redactionPolicy":{"status":"fixture-redacted"}}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("evidence-boundary-report.json"),
        r#"{"schema":"narrated-record-replay.evidence-boundary-report.v1","requiredReview":[],"unsupportedClaims":[],"evidenceSurfaces":{"transcriptSegments":1,"recordReplayEvents":1,"alignedSegments":1,"audioClockPresent":true,"redactionStatus":"fixture-redacted"}}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("packet-inspection.json"),
        r#"{"schema":"narrated-record-replay.packet-inspection.v1","status":"requires-real-packet-review","privacyBoundary":{"allowedToShareWithoutReview":false,"generatedArtifactLeakScan":{"status":"no-obvious-sensitive-patterns-detected","findings":[]},"rawLocalOnly":[]}}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("review-contract.json"),
        r#"{"status":"requires-operator-review","reviewState":{}}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("final-transcript-alignment.json"),
        r#"{"schema":"narrated-record-replay.final-transcript-alignment.v1","status":"aligned","unresolvedMismatches":0,"wordAuthority":"cleaned"}"#,
    )
    .unwrap();
    fs::write(
        session_dir.join("skill-refinement-packet.md"),
        "# Fixture Packet\n\nReviewed synthetic packet.",
    )
    .unwrap();
}

fn write_current_parent_receipt(args: &Args) {
    let session_dir = required_session_dir(args).unwrap();
    let evaluation = evaluate_parent_operation(
        &session_dir,
        args.recording_metadata.as_deref().unwrap(),
        args.recording_events.as_deref().unwrap(),
    )
    .unwrap();
    let binding = crate::parent_operation::ParentOperationBinding::from_evaluation(
        &evaluation,
        args.receipt_run_id.as_deref(),
    );
    let receipt = json!({
        "schema": crate::parent_operation::PARENT_RECEIPT_SCHEMA,
        "status": evaluation.status_text,
        "proofClass": crate::parent_operation::PARENT_PROOF_CLASS,
        "runBinding": {"runId": binding.run_id, "source": "trusted-current-invocation"},
        "recordReplay": binding.record_replay,
        "microphoneCapture": binding.microphone_capture,
        "sameStartChecks": binding.same_start_checks
    });
    fs::write(
        session_dir.join("parent-operation-receipt.json"),
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
