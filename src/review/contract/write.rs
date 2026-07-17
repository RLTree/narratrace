use crate::private_fs::write_private;
use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::review::TranscriptQualityState;

pub fn write_review_contract(
    session_dir: &Path,
    context_path: &Path,
    packet_path: Option<&Path>,
    voice_path: Option<&Path>,
    replay_plan_path: Option<&Path>,
    evidence_report_path: Option<&Path>,
    packet_inspection_path: Option<&Path>,
    dogfood_receipt_path: Option<&Path>,
    replay_plan: &Value,
    packet_inspection: &Value,
    dogfood_receipt: &Value,
    context: &Value,
    diagnostics: &Value,
    alignment_count: usize,
    conflict_count: usize,
    voice_binding_count: usize,
    voice_execution_status: &str,
    voice_proof_obligation_count: usize,
    replay_plan_status: &str,
    replay_plan_cue_count: usize,
    replay_plan_speaks_audio: Option<bool>,
    leak_finding_count: usize,
    raw_local_sensitive_artifact_count: usize,
    raw_local_sensitive_categories: &[String],
    out_of_window_rnr_event_count: usize,
    narration_density_status: &str,
    transcript_word_count: u64,
    transcript_char_count: u64,
    final_alignment_status: &str,
    final_alignment_word_authority: &str,
    final_alignment_unresolved_mismatches: u64,
    transcript_quality: &TranscriptQualityState,
    review_proofs_valid: bool,
) -> Result<PathBuf> {
    let contract_path = session_dir.join("review-contract.json");
    let context_exists = context_path.exists();
    let packet_exists = packet_path.is_some_and(Path::exists);
    let voice_exists = voice_path.is_some_and(Path::exists);
    let replay_plan_exists = replay_plan_path.is_some_and(Path::exists);
    let evidence_report_exists = evidence_report_path.is_some_and(Path::exists);
    let packet_inspection_exists = packet_inspection_path.is_some_and(Path::exists);
    let dogfood_receipt_exists = dogfood_receipt_path.is_some_and(Path::exists);
    let replay_plan_valid = replay_voice_plan_valid(replay_plan);
    let narration_density_sparse = narration_density_status == "too-sparse-for-non-toy-replay";
    let transcript_quality_complete = transcript_quality.is_complete();
    let status = review_status(
        context_exists,
        conflict_count,
        leak_finding_count,
        raw_local_sensitive_artifact_count,
        dogfood_receipt_exists,
        replay_plan_speaks_audio,
        replay_plan_exists,
        replay_plan_valid,
        narration_density_sparse,
        transcript_quality_complete,
        review_proofs_valid,
    );
    write_private(
        &contract_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "narrated-record-replay.review-contract.v1",
            "claimIds": ["CLAIM-012", "CLAIM-013"],
            "status": status,
            "claimCeiling": "static local review contract only; browser/UI runtime proof and product-cohesion review still owed",
            "artifactPresence": {
                "sessionDir": session_dir.display().to_string(),
                "temporalContext": {
                    "path": context_path.display().to_string(),
                    "exists": context_exists
                },
                "skillPacket": {
                    "path": packet_path.map(|path| path.display().to_string()),
                    "exists": packet_exists
                },
                "replayVoiceParameters": {
                    "path": voice_path.map(|path| path.display().to_string()),
                    "exists": voice_exists
                },
                "replayVoiceExecutionPlan": {
                    "path": replay_plan_path.map(|path| path.display().to_string()),
                    "exists": replay_plan_exists
                },
                "evidenceBoundaryReport": {
                    "path": evidence_report_path.map(|path| path.display().to_string()),
                    "exists": evidence_report_exists
                },
                "packetInspection": {
                    "path": packet_inspection_path.map(|path| path.display().to_string()),
                    "exists": packet_inspection_exists
                },
                "dogfoodReceipt": {
                    "path": dogfood_receipt_path.map(|path| path.display().to_string()),
                    "exists": dogfood_receipt_exists
                }
            },
            "reviewState": {
                "alignments": alignment_count,
                "conflictWarnings": conflict_count,
                "voiceSegmentBindings": voice_binding_count,
                "replayVoiceExecutionStatus": voice_execution_status,
                "replayVoiceProofObligations": voice_proof_obligation_count,
                "replayVoicePreviewStatus": replay_plan_status,
                "replayVoicePreviewCueCount": replay_plan_cue_count,
                "replayVoicePreviewSpeaksAudio": replay_plan_speaks_audio,
                "replayVoicePreviewPlanValid": replay_plan_valid,
                "replayVoicePreviewClaimCeiling": replay_plan.get("claimCeiling").and_then(Value::as_str).unwrap_or("not-generated"),
                "redactionStatus": context.pointer("/redactionPolicy/status").and_then(Value::as_str).unwrap_or("unknown"),
                "alignmentClaimCeiling": diagnostics.get("claimCeiling").and_then(Value::as_str).unwrap_or("unknown"),
                "packetInspectionStatus": packet_inspection.get("status").and_then(Value::as_str).unwrap_or("not-generated"),
                "dogfoodReceiptStatus": dogfood_receipt.get("status").and_then(Value::as_str).unwrap_or("not-generated"),
                "generatedArtifactLeakScanStatus": packet_inspection.pointer("/privacyBoundary/generatedArtifactLeakScan/status").and_then(Value::as_str).unwrap_or("not-generated"),
                "generatedArtifactLeakFindings": leak_finding_count,
                "rawLocalSensitiveArtifacts": raw_local_sensitive_artifact_count,
                "rawLocalSensitiveCategories": raw_local_sensitive_categories,
                "outOfWindowRecordReplayEvents": out_of_window_rnr_event_count,
                "narrationDensityStatus": narration_density_status,
                "transcriptWordCount": transcript_word_count,
                "transcriptCharCount": transcript_char_count,
                "finalTranscriptAlignmentStatus": final_alignment_status,
                "finalTranscriptWordAuthority": final_alignment_word_authority,
                "finalTranscriptUnresolvedMismatches": final_alignment_unresolved_mismatches,
                "transcriptQualityPipeline": {
                    "batchTranscriptionReceipt": {
                        "status": transcript_quality.batch.status,
                        "reason": transcript_quality.batch.reason
                    },
                    "cleanupReceipt": {
                        "status": transcript_quality.cleanup.status,
                        "reason": transcript_quality.cleanup.reason
                    },
                    "finalAlignmentReceipt": {
                        "status": transcript_quality.final_receipt.status,
                        "reason": transcript_quality.final_receipt.reason
                    }
                }
            },
            "productCohesionReview": product_cohesion_review(
                context_exists,
                packet_exists,
                evidence_report_exists,
                replay_plan_exists,
                replay_plan_speaks_audio,
                replay_plan_valid,
                packet_inspection_exists,
                dogfood_receipt_exists,
                conflict_count,
                leak_finding_count,
                raw_local_sensitive_artifact_count,
                narration_density_status,
                transcript_quality_complete,
                review_proofs_valid,
            ),
            "recoveryActions": recovery_actions(context_exists, conflict_count, evidence_report_exists, replay_plan_exists, replay_plan_speaks_audio, leak_finding_count, raw_local_sensitive_artifact_count, dogfood_receipt_exists, out_of_window_rnr_event_count, narration_density_sparse, transcript_quality_complete),
            "unsupportedClaims": [
                "This contract does not prove browser rendering.",
                "This contract does not prove review UI product cohesion.",
                "This contract does not prove live capture or packet usefulness.",
                "This contract does not prove dogfood receipt operator approval.",
                "This contract does not prove replay voice audio playback.",
                "This contract does not prove replay voice live demonstration."
            ]
        }))?,
    )?;
    Ok(contract_path)
}
