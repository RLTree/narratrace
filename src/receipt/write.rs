mod artifacts;

use crate::config::{Args, required_session_dir};
use crate::parent_operation::evaluate_parent_operation;
use crate::private_fs::write_private;
use crate::review;
use crate::safe_path::regular_file_metadata;
use anyhow::Result;
use artifacts::{artifact, artifact_entries, optional_artifact};
use serde_json::{Value, json};
#[cfg(test)]
use std::fs;
use std::path::Path;

const MAX_RECEIPT_JSON_BYTES: u64 = 8 * 1024 * 1024;

pub fn write_receipt(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    let receipt_path = session_dir.join("dogfood-receipt.json");
    let manifest = read_json(&session_dir.join("manifest.json")).unwrap_or(Value::Null);
    let status = read_json(&session_dir.join("status.json")).unwrap_or(Value::Null);
    let temporal = read_json(&session_dir.join("temporal-context.json")).unwrap_or(Value::Null);
    let evidence =
        read_json(&session_dir.join("evidence-boundary-report.json")).unwrap_or(Value::Null);
    let inspection = read_json(&session_dir.join("packet-inspection.json")).unwrap_or(Value::Null);
    let review_contract =
        read_json(&session_dir.join("review-contract.json")).unwrap_or(Value::Null);
    let post_commit_drain =
        read_json(&session_dir.join("post-commit-drain.json")).unwrap_or(Value::Null);
    let stop_timeout = read_json(&session_dir.join("stop-timeout.json")).unwrap_or(Value::Null);
    let parent_operation =
        read_json(&session_dir.join("parent-operation-receipt.json")).unwrap_or(Value::Null);
    let final_alignment =
        read_json(&session_dir.join("final-transcript-alignment.json")).unwrap_or(Value::Null);
    let post_commit_completed_segments = post_commit_completed_segments(&post_commit_drain);
    let raw_local = [
        artifact(
            "transcript-timeline",
            session_dir.join("transcript.timeline.jsonl"),
        ),
        artifact(
            "transcript-events",
            session_dir.join("transcript.events.jsonl"),
        ),
        artifact("transcript-final", session_dir.join("transcript.final.txt")),
        artifact("transcript-live", session_dir.join("transcript.live.txt")),
        artifact("capture-clock", session_dir.join("capture-clock.json")),
        artifact("manifest", session_dir.join("manifest.json")),
        artifact("capture-status", session_dir.join("status.json")),
        artifact(
            "post-commit-drain",
            session_dir.join("post-commit-drain.json"),
        ),
        artifact("stop-timeout", session_dir.join("stop-timeout.json")),
        artifact(
            "parent-operation-receipt",
            session_dir.join("parent-operation-receipt.json"),
        ),
    ];
    let generated = generated_review_artifacts(&session_dir);
    let recording = [
        optional_artifact("recording-metadata", args.recording_metadata.as_deref()),
        optional_artifact("recording-events", args.recording_events.as_deref()),
    ];
    let recording_entries = artifact_entries(&recording, "external-record-replay-evidence")?;
    let raw_local_entries = artifact_entries(&raw_local, "raw-local-private")?;
    let generated_entries = artifact_entries(&generated, "generated-review-candidate")?;
    let mut blockers = blockers(
        args,
        &manifest,
        &status,
        &temporal,
        &evidence,
        &inspection,
        &post_commit_drain,
        &parent_operation,
        &final_alignment,
    );
    if !digest_bindings_complete(&recording_entries, &raw_local_entries, &generated_entries)
        || !current_run_values_unchanged(
            &session_dir,
            &manifest,
            &status,
            &temporal,
            &evidence,
            &inspection,
            &post_commit_drain,
            &parent_operation,
            &final_alignment,
        )
    {
        blockers.push("current-run proof digest binding is missing or changed during verification");
    }
    let status_text = if blockers == ["operator review of generated artifacts is still required"] {
        "requires-operator-review"
    } else {
        "blocked"
    };
    let mut payload = json!({
        "schema": "narrated-record-replay.dogfood-receipt.v1",
        "status": status_text,
        "runBinding": {
            "runId": args.receipt_run_id,
            "source": "trusted-current-receipt-invocation"
        },
        "evidenceTrust": {
            "status": if status_text == "requires-operator-review" {
                "verified-current-run"
            } else {
                "blocked-untrusted-or-incomplete"
            },
            "parentOperationVerified": parent_operation_receipt_matches_current_artifacts(args, &parent_operation)
        },
        "claimIds": ["CLAIM-008", "CLAIM-009", "CLAIM-010", "CLAIM-011", "CLAIM-012"],
        "session": {
            "path": session_dir.display().to_string(),
            "exists": session_dir.exists(),
            "goalProvided": manifest.get("goal").and_then(Value::as_str).is_some(),
            "helperState": status.get("state").and_then(Value::as_str),
            "model": status.get("model").and_then(Value::as_str)
        },
        "evidenceSummary": {
            "transcriptSegments": first_u64(&[
                temporal.pointer("/transcriptSegments"),
                evidence.pointer("/evidenceSurfaces/transcriptSegments"),
            ]),
            "recordReplayEvents": first_u64(&[
                temporal.pointer("/recordReplayEvents"),
                evidence.pointer("/evidenceSurfaces/recordReplayEvents"),
            ]),
            "alignedSegments": first_u64(&[
                temporal.pointer("/alignments"),
                evidence.pointer("/evidenceSurfaces/alignedSegments"),
            ]),
            "conflictWarnings": first_u64(&[
                temporal.pointer("/conflictDiagnostics/warnings"),
                evidence.pointer("/evidenceSurfaces/conflictWarnings"),
            ]),
            "redactionStatus": evidence.pointer("/evidenceSurfaces/redactionStatus").and_then(Value::as_str),
            "packetInspectionStatus": inspection.get("status").and_then(Value::as_str),
            "reviewStatus": review_contract.get("status").and_then(Value::as_str),
            "generatedArtifactLeakScanStatus": inspection.pointer("/privacyBoundary/generatedArtifactLeakScan/status").and_then(Value::as_str),
            "postCommitDrainCompletedSegments": post_commit_completed_segments,
            "postCommitDrainMessages": post_commit_drain.pointer("/messages").and_then(Value::as_u64),
            "postCommitDrainErrors": post_commit_drain.pointer("/errors").and_then(Value::as_array).map(|errors| errors.len() as u64),
            "parentOperationStatus": parent_operation.pointer("/status").and_then(Value::as_str),
            "parentOperationStartDeltaMs": parent_operation.pointer("/sameStartChecks/startDeltaMs").and_then(Value::as_i64),
            "finalTranscriptAlignmentStatus": final_alignment.get("status").and_then(Value::as_str),
            "finalTranscriptUnresolvedMismatches": final_alignment.get("unresolvedMismatches").and_then(Value::as_u64)
        },
        "parentOperation": {
            "status": parent_operation.pointer("/status").and_then(Value::as_str),
            "proofClass": parent_operation.pointer("/proofClass").and_then(Value::as_str),
            "startDeltaMs": parent_operation.pointer("/sameStartChecks/startDeltaMs").and_then(Value::as_i64),
            "withinAllowedStartDelta": parent_operation.pointer("/sameStartChecks/withinAllowedStartDelta").and_then(Value::as_bool),
            "recordReplayEventsPresent": parent_operation.pointer("/sameStartChecks/recordReplayEventsPresent").and_then(Value::as_bool),
            "microphoneStoppedCleanly": parent_operation.pointer("/sameStartChecks/microphoneStoppedCleanly").and_then(Value::as_bool),
            "postCommitDrainCompleted": parent_operation.pointer("/sameStartChecks/postCommitDrainCompleted").and_then(Value::as_bool),
            "startDeltaDisposition": if parent_operation.pointer("/sameStartChecks/withinAllowedStartDelta").and_then(Value::as_bool) == Some(false) {
                "diagnostic-warning; transcript/video alignment is judged from final transcript alignment and timestamp-window evidence"
            } else {
                "within-configured-threshold"
            }
        },
        "capture": {
            "helperState": status.get("state").and_then(Value::as_str),
            "audioInput": status.get("audioInput"),
            "postCommitDrain": {
                "present": post_commit_drain.is_object(),
                "completedSegments": post_commit_completed_segments,
                "messages": post_commit_drain.pointer("/messages").and_then(Value::as_u64),
                "errors": post_commit_drain.pointer("/errors").and_then(Value::as_array).map(|errors| errors.len())
            },
            "stopTimeout": {
                "present": stop_timeout.is_object(),
                "status": stop_timeout.get("status").and_then(Value::as_str),
                "waitedMs": stop_timeout.get("waitedMs").and_then(Value::as_u64)
            }
        },
        "reviewState": review_contract.get("reviewState").cloned().unwrap_or(Value::Null),
        "generatedArtifactLeakScan": inspection.pointer("/privacyBoundary/generatedArtifactLeakScan").cloned().unwrap_or(Value::Null),
        "rawLocalPrivateArtifacts": inspection.pointer("/privacyBoundary/rawLocalOnly").cloned().unwrap_or(Value::Null),
        "rawLocalSensitiveArtifacts": sensitive_raw_local_artifacts(&inspection),
        "artifactEvidence": {
            "recordReplay": recording_entries,
            "rawLocalPrivate": raw_local_entries,
            "generatedReviewCandidates": generated_entries
        },
        "privacyBoundary": {
            "rawTextCopiedIntoReceipt": false,
            "rawAudioCopiedIntoReceipt": false,
            "secretsCopiedIntoReceipt": false,
            "allowedToShareWithoutReview": false,
            "policy": "Receipt records paths, counts, byte sizes, line counts, and SHA-256 fingerprints only. Raw transcript and audio remain local-private."
        },
        "blockers": blockers,
        "claimCeiling": "dogfood receipt metadata only; live capability claims require the actual run artifacts plus operator inspection"
    });
    write_private(&receipt_path, serde_json::to_string_pretty(&payload)?)?;
    refresh_review_surface(args, &session_dir)?;
    let refreshed_generated = generated_review_artifacts(&session_dir);
    payload["artifactEvidence"]["generatedReviewCandidates"] = json!(artifact_entries(
        &refreshed_generated,
        "generated-review-candidate"
    )?);
    write_private(&receipt_path, serde_json::to_string_pretty(&payload)?)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "receiptPath": receipt_path,
            "sessionDir": session_dir,
            "status": status_text
        }))?
    );
    Ok(())
}
