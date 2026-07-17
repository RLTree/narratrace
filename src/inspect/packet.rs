mod artifacts;
mod usefulness;

use crate::config::{Args, required_session_dir};
use crate::private_fs::write_private;
use crate::review;
use crate::safe_path::{read_regular_text_bounded, regular_file_metadata};
use anyhow::Result;
use artifacts::{artifact_entries, artifact_spec, leak_scan, raw_local_entries};
use serde_json::{Value, json};
#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};

const REDACTED_REVIEW: &str = "generated-redacted-review-candidate";
const REVIEW: &str = "generated-review-candidate";
const RAW_LOCAL: &str = "raw-local-private";
const MAX_INSPECT_JSON_BYTES: u64 = 8 * 1024 * 1024;

pub fn inspect_packet(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    let temporal_path = session_dir.join("temporal-context.json");
    let evidence_path = session_dir.join("evidence-boundary-report.json");
    let review_contract_path = session_dir.join("review-contract.json");
    let packet_path = session_dir.join("skill-refinement-packet.md");
    let voice_path = session_dir.join("replay-voice-parameters.json");
    let replay_plan_path = session_dir.join("replay-voice-execution-plan.json");
    let notes_path = session_dir.join("timestamped-notes.md");
    let thought_path = session_dir.join("thought-process.md");
    let dogfood_receipt_path = session_dir.join("dogfood-receipt.json");
    let transcript_timeline_path = session_dir.join("transcript.timeline.jsonl");
    let transcript_events_path = session_dir.join("transcript.events.jsonl");
    let transcript_final_path = session_dir.join("transcript.final.txt");
    let transcript_live_path = session_dir.join("transcript.live.txt");
    let batch_transcript_path = session_dir.join("batch-transcript.json");
    let cleaned_transcript_path = session_dir.join("cleaned-transcript.json");
    let final_transcript_alignment_path = session_dir.join("final-transcript-alignment.json");
    let final_transcript_timeline_path = session_dir.join("final-transcript.timeline.jsonl");
    let audio_retention_manifest_path = session_dir.join("audio-retention.json");
    let audio_chunks_path = session_dir.join("audio-chunks.jsonl");
    let narration_sync_path = session_dir.join("narration.sync.jsonl");
    let retained_audio_path = session_dir.join("retained-audio.wav");
    let mut review_candidates = vec![
        artifact_spec("skill-refinement-packet", &packet_path, REDACTED_REVIEW),
        artifact_spec("timestamped-notes", &notes_path, REDACTED_REVIEW),
        artifact_spec("thought-process", &thought_path, REDACTED_REVIEW),
        artifact_spec("temporal-context", &temporal_path, REDACTED_REVIEW),
        artifact_spec("evidence-boundary-report", &evidence_path, REDACTED_REVIEW),
        artifact_spec("review-contract", &review_contract_path, REVIEW),
        artifact_spec("replay-voice-parameters", &voice_path, REVIEW),
    ];
    if regular_file_metadata(&replay_plan_path).is_ok() {
        review_candidates.push(artifact_spec(
            "replay-voice-execution-plan",
            &replay_plan_path,
            REVIEW,
        ));
    }
    let raw_local = [
        artifact_spec("transcript-timeline", &transcript_timeline_path, RAW_LOCAL),
        artifact_spec("transcript-events", &transcript_events_path, RAW_LOCAL),
        artifact_spec("transcript-final", &transcript_final_path, RAW_LOCAL),
        artifact_spec("transcript-live", &transcript_live_path, RAW_LOCAL),
        artifact_spec("batch-transcript", &batch_transcript_path, RAW_LOCAL),
        artifact_spec("cleaned-transcript", &cleaned_transcript_path, RAW_LOCAL),
        artifact_spec(
            "final-transcript-alignment",
            &final_transcript_alignment_path,
            RAW_LOCAL,
        ),
        artifact_spec(
            "final-transcript-timeline",
            &final_transcript_timeline_path,
            RAW_LOCAL,
        ),
        artifact_spec(
            "audio-retention-manifest",
            &audio_retention_manifest_path,
            RAW_LOCAL,
        ),
        artifact_spec("audio-chunks", &audio_chunks_path, RAW_LOCAL),
        artifact_spec("narration-sync", &narration_sync_path, RAW_LOCAL),
        artifact_spec("retained-audio", &retained_audio_path, RAW_LOCAL),
    ];
    let temporal = read_json(&temporal_path).unwrap_or(Value::Null);
    let evidence = read_json(&evidence_path).unwrap_or(Value::Null);
    let approved_paths = approved_record_replay_paths(&evidence);
    let initial_leak_scan = leak_scan(&review_candidates, &approved_paths);
    let initial_blocking_leak_count = blocking_leak_count(&initial_leak_scan);
    let review_contract = read_json(&review_contract_path).unwrap_or(Value::Null);
    let voice = read_json(&voice_path).unwrap_or(Value::Null);
    let replay_plan = read_json(&replay_plan_path).unwrap_or(Value::Null);
    let conflict_warnings = temporal
        .pointer("/conflictDiagnostics/warnings")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let required_review = evidence
        .get("requiredReview")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let unsupported_claims = evidence
        .get("unsupportedClaims")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let initial_blockers = blockers(
        &temporal_path,
        &evidence_path,
        &review_contract_path,
        &packet_path,
        &evidence,
        conflict_warnings,
        initial_blocking_leak_count,
    );
    let status = if initial_blockers.is_empty() {
        "requires-real-packet-review"
    } else {
        "requires-operator-review"
    };
    let inspection_path = session_dir.join("packet-inspection.json");
    let mut payload = json!({
        "schema": "narrated-record-replay.packet-inspection.v1",
        "status": status,
        "claimIds": ["CLAIM-010", "CLAIM-011", "CLAIM-012"],
        "artifactPresence": {
            "skillPacket": path_exists(&packet_path),
            "temporalContext": path_exists(&temporal_path),
            "evidenceBoundaryReport": path_exists(&evidence_path),
            "reviewContract": path_exists(&review_contract_path),
            "replayVoiceParameters": path_exists(&voice_path),
            "replayVoiceExecutionPlan": path_exists(&replay_plan_path)
        },
        "evidenceSummary": {
            "transcriptSegments": evidence.pointer("/evidenceSurfaces/transcriptSegments").and_then(Value::as_u64),
            "recordReplayEvents": evidence.pointer("/evidenceSurfaces/recordReplayEvents").and_then(Value::as_u64),
            "alignedSegments": evidence.pointer("/evidenceSurfaces/alignedSegments").and_then(Value::as_u64),
            "conflictWarnings": conflict_warnings,
            "redactionStatus": evidence.pointer("/evidenceSurfaces/redactionStatus").and_then(Value::as_str),
            "reviewStatus": review_contract.get("status").and_then(Value::as_str),
            "replayVoiceStatus": voice.get("status").and_then(Value::as_str),
            "replayVoicePreviewStatus": replay_plan.get("status").and_then(Value::as_str),
            "replayVoicePreviewCueCount": replay_plan.get("cueCount").and_then(Value::as_u64),
            "replayVoicePreviewSpeaksAudio": replay_plan.pointer("/proofBoundary/speaksAudio").and_then(Value::as_bool)
        },
        "privacyBoundary": {
            "shareableStatus": "operator-review-required",
            "allowedToShareWithoutReview": false,
            "policy": "Raw transcript inputs are local-private. Generated artifacts are review candidates only after redaction and operator inspection.",
            "distilledReviewCandidates": artifact_entries(&review_candidates),
            "rawLocalOnly": raw_local_entries(&raw_local),
            "generatedArtifactLeakScan": initial_leak_scan
        },
        "packetUsefulnessReview": usefulness::packet_usefulness_review(&packet_path, &notes_path, &thought_path, &temporal_path, &evidence_path, &temporal, &evidence),
        "blockers": initial_blockers,
        "requiredReview": required_review,
        "unsupportedClaims": unsupported_claims,
        "claimCeiling": "automated packet inspection only; real non-toy workflow review and raw-private leakage inspection still owed"
    });
    write_private(&inspection_path, serde_json::to_string_pretty(&payload)?)?;
    let review_artifact = review::write_review_artifact(
        &session_dir,
        &temporal_path,
        existing(&packet_path),
        existing(&voice_path),
        existing(&replay_plan_path),
        existing(&evidence_path),
        Some(&inspection_path),
        existing(&dogfood_receipt_path),
    )?;
    review_candidates.push(artifact_spec(
        "review-artifact",
        &review_artifact.html_path,
        REVIEW,
    ));
    let final_leak_scan = leak_scan(&review_candidates, &approved_paths);
    let final_blocking_leak_count = blocking_leak_count(&final_leak_scan);
    let refreshed_review_contract = read_json(&review_contract_path).unwrap_or(Value::Null);
    payload["artifactPresence"]["reviewContract"] = path_exists(&review_contract_path);
    payload["evidenceSummary"]["reviewStatus"] = refreshed_review_contract
        .get("status")
        .and_then(Value::as_str)
        .map_or(Value::Null, |text| json!(text));
    payload["privacyBoundary"]["distilledReviewCandidates"] =
        json!(artifact_entries(&review_candidates));
    payload["privacyBoundary"]["generatedArtifactLeakScan"] = final_leak_scan;
    payload["blockers"] = json!(blockers(
        &temporal_path,
        &evidence_path,
        &review_contract_path,
        &packet_path,
        &evidence,
        conflict_warnings,
        final_blocking_leak_count,
    ));
    write_private(&inspection_path, serde_json::to_string_pretty(&payload)?)?;
    let _ = review::write_review_artifact(
        &session_dir,
        &temporal_path,
        existing(&packet_path),
        existing(&voice_path),
        existing(&replay_plan_path),
        existing(&evidence_path),
        Some(&inspection_path),
        existing(&dogfood_receipt_path),
    )?;
    let refreshed_leak_scan = leak_scan(&review_candidates, &approved_paths);
    let refreshed_blocking_leak_count = blocking_leak_count(&refreshed_leak_scan);
    payload["privacyBoundary"]["generatedArtifactLeakScan"] = refreshed_leak_scan;
    payload["blockers"] = json!(blockers(
        &temporal_path,
        &evidence_path,
        &review_contract_path,
        &packet_path,
        &evidence,
        conflict_warnings,
        refreshed_blocking_leak_count,
    ));
    write_private(&inspection_path, serde_json::to_string_pretty(&payload)?)?;
    let review_artifact = review::write_review_artifact(
        &session_dir,
        &temporal_path,
        existing(&packet_path),
        existing(&voice_path),
        existing(&replay_plan_path),
        existing(&evidence_path),
        Some(&inspection_path),
        existing(&dogfood_receipt_path),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "inspectionPath": inspection_path,
            "reviewPath": review_artifact.html_path,
            "reviewContractPath": review_artifact.contract_path,
            "sessionDir": session_dir,
            "status": status
        }))?
    );
    Ok(())
}
