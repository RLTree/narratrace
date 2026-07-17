use super::*;
use crate::config::parse_args_from;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parent_receipt_requires_the_complete_current_binding() {
    let root = unique_tmp("nrr-parent-complete-binding");
    let session = root.join("session");
    let rnr = root.join("rnr");
    fs::create_dir_all(&session).unwrap();
    fs::create_dir_all(&rnr).unwrap();
    let metadata = rnr.join("session.json");
    let events = rnr.join("events.jsonl");
    fs::write(
        &metadata,
        r#"{"id":"current-rnr","startedAt":"2026-06-19T01:38:25Z"}"#,
    )
    .unwrap();
    fs::write(
        &events,
        r#"{"id":1,"kind":"click","timestamp":"2026-06-19T01:38:26Z"}"#,
    )
    .unwrap();
    fs::write(
        session.join("status.json"),
        r#"{"state":"stopped","audioInput":{"deviceName":"test"}}"#,
    )
    .unwrap();
    fs::write(
        session.join("capture-clock.json"),
        r#"{"audioStartedAtUnixMs":1781833105000}"#,
    )
    .unwrap();
    fs::write(
        session.join("post-commit-drain.json"),
        r#"{"completedSegments":1,"errors":[]}"#,
    )
    .unwrap();
    let args = parse_args_from([
        "nrr",
        "receipt",
        "--session-dir",
        session.to_str().unwrap(),
        "--recording-metadata",
        metadata.to_str().unwrap(),
        "--recording-events",
        events.to_str().unwrap(),
        "--receipt-run-id",
        "trusted-run",
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();
    let evaluation = evaluate_parent_operation(
        &session,
        metadata.to_str().unwrap(),
        events.to_str().unwrap(),
    )
    .unwrap();
    let binding = crate::parent_operation::ParentOperationBinding::from_evaluation(
        &evaluation,
        Some("trusted-run"),
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
    assert!(parent_operation_receipt_matches_current_artifacts(
        &args, &receipt
    ));

    for (pointer, replacement) in [
        ("/schema", json!("wrong-schema")),
        ("/status", json!("caller-success")),
        ("/proofClass", json!("self-authored")),
        ("/runBinding/runId", json!("other-run")),
        ("/runBinding/source", json!("self-authored")),
        ("/recordReplay/sessionId", json!("old-rnr")),
        ("/recordReplay/metadataDigest", json!("sha256:old")),
        ("/recordReplay/eventsDigest", json!("sha256:old")),
        ("/microphoneCapture/sessionDir", json!("/private/tmp/old")),
        ("/sameStartChecks/startDeltaMs", json!(1)),
    ] {
        let mut mutated = receipt.clone();
        *mutated.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            !parent_operation_receipt_matches_current_artifacts(&args, &mutated),
            "mutation at {pointer} was accepted"
        );
    }

    fs::write(
        &events,
        r#"{"id":2,"kind":"type","timestamp":"2026-06-19T01:38:27Z"}"#,
    )
    .unwrap();
    assert!(!parent_operation_receipt_matches_current_artifacts(
        &args, &receipt
    ));
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
