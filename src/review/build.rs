mod contract;
mod html;
mod inspection;

use crate::config::{Args, required_session_dir};
use crate::private_fs::write_private;
use crate::safe_path::regular_file_metadata;
use anyhow::Result;
use contract::{recovery_actions, replay_voice_plan_valid, review_status, write_review_contract};
use html::ReviewHtmlInput;
use inspection::{
    blocking_leak_finding_count, inspection_status, leak_categories, leak_finding_count,
    leak_scan_status, raw_local_sensitive_artifact_count, raw_local_sensitive_categories,
    read_packet_inspection,
};
use serde_json::Value;
#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};

const MAX_REVIEW_JSON_BYTES: u64 = 8 * 1024 * 1024;

pub struct ReviewArtifact {
    pub html_path: PathBuf,
    pub contract_path: PathBuf,
}

pub fn write_review_artifact(
    session_dir: &Path,
    context_path: &Path,
    packet_path: Option<&Path>,
    voice_path: Option<&Path>,
    replay_plan_path: Option<&Path>,
    evidence_report_path: Option<&Path>,
    packet_inspection_path: Option<&Path>,
    dogfood_receipt_path: Option<&Path>,
) -> Result<ReviewArtifact> {
    write_review_artifact_for_run(
        session_dir,
        context_path,
        packet_path,
        voice_path,
        replay_plan_path,
        evidence_report_path,
        packet_inspection_path,
        dogfood_receipt_path,
        None,
    )
}

pub(crate) fn write_review_artifact_for_receipt(
    session_dir: &Path,
    context_path: &Path,
    packet_path: Option<&Path>,
    voice_path: Option<&Path>,
    replay_plan_path: Option<&Path>,
    evidence_report_path: Option<&Path>,
    packet_inspection_path: Option<&Path>,
    dogfood_receipt_path: Option<&Path>,
    expected_run_id: Option<&str>,
) -> Result<ReviewArtifact> {
    write_review_artifact_for_run(
        session_dir,
        context_path,
        packet_path,
        voice_path,
        replay_plan_path,
        evidence_report_path,
        packet_inspection_path,
        dogfood_receipt_path,
        expected_run_id,
    )
}

pub fn make_review(args: &Args) -> Result<()> {
    let session_dir = required_session_dir(args)?;
    let context_path = session_dir.join("temporal-context.json");
    let packet_path = session_dir.join("skill-refinement-packet.md");
    let voice_path = session_dir.join("replay-voice-parameters.json");
    let replay_plan_path = session_dir.join("replay-voice-execution-plan.json");
    let evidence_report_path = session_dir.join("evidence-boundary-report.json");
    let packet_inspection_path = session_dir.join("packet-inspection.json");
    let dogfood_receipt_path = session_dir.join("dogfood-receipt.json");
    let review_artifact = write_review_artifact_for_run(
        &session_dir,
        &context_path,
        if regular_file_exists(&packet_path) {
            Some(&packet_path)
        } else {
            None
        },
        if regular_file_exists(&voice_path) {
            Some(&voice_path)
        } else {
            None
        },
        if regular_file_exists(&replay_plan_path) {
            Some(&replay_plan_path)
        } else {
            None
        },
        if regular_file_exists(&evidence_report_path) {
            Some(&evidence_report_path)
        } else {
            None
        },
        if regular_file_exists(&packet_inspection_path) {
            Some(&packet_inspection_path)
        } else {
            None
        },
        if regular_file_exists(&dogfood_receipt_path) {
            Some(&dogfood_receipt_path)
        } else {
            None
        },
        args.receipt_run_id.as_deref(),
    )?;
    println!(
        "{}",
        serde_json::json!({ "reviewPath": review_artifact.html_path, "reviewContractPath": review_artifact.contract_path, "sessionDir": session_dir })
    );
    Ok(())
}
