use super::manifest_backlink::validate_backlog_manifest_backlink;
use super::receipt::{validate_validator_receipt, validate_validator_receipt_allow_stale};
use super::trusted_context::TrustedBundleContext;
use super::util::{
    assert_digest, bundle_artifact_path, read_text, required_claim_ids, required_claim_ids_hash,
    required_string,
};
use anyhow::{Result, bail};
use serde_json::Value;
use std::path::Path;

pub(super) fn assert_contract_claim_ids(path: &Path) -> Result<()> {
    let contract = read_text(path)?;
    let missing: Vec<String> = required_claim_ids()
        .into_iter()
        .filter(|claim_id| !contract.contains(claim_id))
        .collect();
    if !missing.is_empty() {
        bail!(
            "GOAL_CONTRACT.md missing canonical claim IDs: {}",
            missing.join(", ")
        );
    }
    Ok(())
}

pub(super) fn validate_identity_chain(
    skill_dir: &Path,
    manifest: &Value,
    amendments: &[Value],
) -> Result<()> {
    assert_digest(
        "COMPLETION_MANIFEST.json#/contract_bundle_hash",
        &skill_dir.join("GOAL_CONTRACT.md"),
        required_string(
            manifest,
            "/contract_bundle_hash",
            "COMPLETION_MANIFEST.json",
        )?,
    )?;
    let required_hash = required_claim_ids_hash();
    let actual_required_hash = required_string(
        manifest,
        "/required_claim_ids_hash",
        "COMPLETION_MANIFEST.json",
    )?;
    if actual_required_hash != required_hash {
        bail!(
            "COMPLETION_MANIFEST.json#/required_claim_ids_hash mismatch: expected {required_hash}, got {actual_required_hash}"
        );
    }
    let mut previous_amendment_hash =
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
    for (index, row) in amendments.iter().enumerate() {
        let declared_previous = required_string(
            row,
            "/previous_amendment_hash",
            &format!("AMENDMENTS.jsonl:{}", index + 1),
        )?;
        if declared_previous != previous_amendment_hash {
            bail!(
                "AMENDMENTS.jsonl:{} previous_amendment_hash mismatch: expected {}, got {}",
                index + 1,
                previous_amendment_hash,
                declared_previous
            );
        }
        previous_amendment_hash = required_string(
            row,
            "/amendment_hash",
            &format!("AMENDMENTS.jsonl:{}", index + 1),
        )?
        .to_string();
    }
    Ok(())
}

pub(super) fn validate_repeated_digests(
    skill_dir: &Path,
    repo_root: &Path,
    manifest: &Value,
    amendments: &[Value],
    context: &TrustedBundleContext,
) -> Result<()> {
    validate_repeated_digests_with_receipt_policy(
        skill_dir,
        repo_root,
        manifest,
        amendments,
        Some(context),
    )
}

pub(super) fn validate_repeated_digests_for_refresh(
    skill_dir: &Path,
    repo_root: &Path,
    manifest: &Value,
    amendments: &[Value],
) -> Result<()> {
    validate_repeated_digests_with_receipt_policy(skill_dir, repo_root, manifest, amendments, None)
}

fn validate_repeated_digests_with_receipt_policy(
    skill_dir: &Path,
    repo_root: &Path,
    manifest: &Value,
    amendments: &[Value],
    context: Option<&TrustedBundleContext>,
) -> Result<()> {
    let backlog_path = bundle_artifact_path(
        repo_root,
        required_string(
            manifest,
            "/verification_backlog_path",
            "COMPLETION_MANIFEST.json",
        )?,
    )?;
    assert_digest(
        "COMPLETION_MANIFEST.json#/verification_backlog_digest",
        &backlog_path,
        required_string(
            manifest,
            "/verification_backlog_digest",
            "COMPLETION_MANIFEST.json",
        )?,
    )?;
    validate_backlog_manifest_backlink(&backlog_path, &skill_dir.join("COMPLETION_MANIFEST.json"))?;

    for (claim_index, claim) in manifest
        .get("claims")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .enumerate()
    {
        for (evidence_index, evidence) in claim
            .get("evidence")
            .and_then(Value::as_array)
            .unwrap_or(&Vec::new())
            .iter()
            .enumerate()
        {
            let path = bundle_artifact_path(
                repo_root,
                required_string(evidence, "/path", "COMPLETION_MANIFEST.json evidence")?,
            )?;
            assert_digest(
                &format!(
                    "COMPLETION_MANIFEST.json#/claims/{claim_index}/evidence/{evidence_index}/digest"
                ),
                &path,
                required_string(evidence, "/digest", "COMPLETION_MANIFEST.json evidence")?,
            )?;
        }
    }

    let receipts = manifest
        .get("validator_receipts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            anyhow::anyhow!("COMPLETION_MANIFEST.json#/validator_receipts must be an array")
        })?;
    for (receipt_index, receipt) in receipts.iter().enumerate() {
        let path = bundle_artifact_path(
            repo_root,
            required_string(
                receipt,
                "/path",
                "COMPLETION_MANIFEST.json validator receipt",
            )?,
        )?;
        assert_digest(
            &format!("COMPLETION_MANIFEST.json#/validator_receipts/{receipt_index}/digest"),
            &path,
            required_string(
                receipt,
                "/digest",
                "COMPLETION_MANIFEST.json validator receipt",
            )?,
        )?;
        let expected_commit = required_string(manifest, "/commit", "COMPLETION_MANIFEST.json")?;
        if let Some(context) = context {
            validate_validator_receipt(&path, skill_dir, repo_root, context)?;
        } else {
            validate_validator_receipt_allow_stale(&path, skill_dir, repo_root, expected_commit)?;
        }
    }

    let latest = amendments
        .last()
        .ok_or_else(|| anyhow::anyhow!("AMENDMENTS.jsonl must contain at least one row"))?;
    if context.is_some() {
        for (update_index, update) in latest
            .get("backlog_updates")
            .and_then(Value::as_array)
            .unwrap_or(&Vec::new())
            .iter()
            .enumerate()
        {
            let path = bundle_artifact_path(
                repo_root,
                required_string(update, "/path", "AMENDMENTS.jsonl backlog update")?,
            )?;
            assert_digest(
                &format!("AMENDMENTS.jsonl:last#/backlog_updates/{update_index}/digest"),
                &path,
                required_string(update, "/digest", "AMENDMENTS.jsonl backlog update")?,
            )?;
        }
    }
    assert_digest(
        "AMENDMENTS.jsonl:last#/new_contract_hash",
        &skill_dir.join("GOAL_CONTRACT.md"),
        required_string(latest, "/new_contract_hash", "AMENDMENTS.jsonl latest row")?,
    )?;
    Ok(())
}
