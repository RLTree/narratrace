use super::amendments::{amendment_hash, read_amendments};
use super::authority::validate_rust_only_authority;
use super::identity::{
    assert_contract_claim_ids, validate_identity_chain, validate_repeated_digests,
    validate_repeated_digests_for_refresh,
};
use super::manifest_backlink::normalized_manifest_value_digest;
use super::receipt::local_validator_receipt;
use super::schema::{
    validate_backlog, validate_lane_registry, validate_manifest_shape, validate_red_fixtures,
};
use super::trusted_context::TrustedBundleContext;
use super::util::{read_json, required_claim_ids, required_string};
use super::{LOCAL_VALIDATOR_RECEIPT_PATH, repo_root_for_skill_dir};
use crate::config::Args;
use crate::private_fs::{write_atomic_preserving_mode, write_private};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(super) fn refresh_bundle_receipt(args: &Args) -> Result<()> {
    let skill_dir = args
        .skill_dir
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--skill-dir is required"))?;
    let repo_root = repo_root_for_skill_dir(skill_dir)?;
    let context = TrustedBundleContext::from_trusted_services(skill_dir)?;

    let lane_registry = read_json(&skill_dir.join("LANE_REGISTRY.json"))?;
    let mut backlog = read_json(&skill_dir.join("VERIFICATION_BACKLOG.json"))?;
    let mut manifest = read_json(&skill_dir.join("COMPLETION_MANIFEST.json"))?;
    let red_fixtures = read_json(&skill_dir.join("RED_FIXTURES.json"))?;
    let mut amendments = read_amendments(&skill_dir.join("AMENDMENTS.jsonl"))?;

    validate_lane_registry(&lane_registry)?;
    validate_backlog(&backlog)?;
    validate_manifest_shape(&manifest, &context.goal)?;
    super::claim_policy::validate_backlog_bindings(&manifest, &backlog)?;
    super::claim_policy::validate_manifest_claim_ceiling(&manifest, &amendments)?;
    validate_red_fixtures(&red_fixtures)?;
    assert_contract_claim_ids(&skill_dir.join("GOAL_CONTRACT.md"))?;
    validate_identity_chain(skill_dir, &manifest, &amendments)?;
    validate_repeated_digests_for_refresh(skill_dir, &repo_root, &manifest, &amendments)?;
    validate_rust_only_authority(skill_dir)?;
    context.source.assert_unchanged(skill_dir)?;

    manifest["commit"] = json!(context.source.digest());

    let receipt = local_validator_receipt(
        skill_dir,
        &repo_root,
        &context,
        amendments.len() + 1,
        args.receipt_run_id.as_deref(),
        None,
    )?;
    let generated_at = required_string(&receipt, "/generated_at", "local validator receipt")?;
    let receipt_bytes = pretty_json_bytes(&receipt)?;
    let receipt_digest = digest_bytes(&receipt_bytes);

    manifest["validator_receipts"] = json!([{
        "path": LOCAL_VALIDATOR_RECEIPT_PATH,
        "digest": receipt_digest,
    }]);
    manifest["generated_at"] = json!(generated_at);

    let mut normalized_manifest = manifest.clone();
    let manifest_digest = normalized_manifest_value_digest(&mut normalized_manifest)?;
    backlog["manifest_digest"] = json!(manifest_digest);
    backlog["generated_at"] = json!(generated_at);
    let backlog_bytes = compact_json_bytes(&backlog)?;
    let backlog_digest = digest_bytes(&backlog_bytes);
    manifest["verification_backlog_digest"] = json!(backlog_digest);
    let manifest_bytes = compact_json_bytes(&manifest)?;
    append_refresh_amendment(&mut amendments, &backlog_digest, generated_at)?;
    let amendment_bytes = json_lines_bytes(&amendments)?;

    let receipt_path =
        skill_dir.join("validation_artifacts/root-gate/current-rust-bundle-validation.json");
    write_private(&receipt_path, &receipt_bytes)?;
    write_atomic_preserving_mode(skill_dir.join("VERIFICATION_BACKLOG.json"), &backlog_bytes)?;
    write_atomic_preserving_mode(skill_dir.join("COMPLETION_MANIFEST.json"), &manifest_bytes)?;
    write_atomic_preserving_mode(skill_dir.join("AMENDMENTS.jsonl"), &amendment_bytes)?;

    let refreshed_manifest = read_json(&skill_dir.join("COMPLETION_MANIFEST.json"))?;
    let refreshed_amendments = read_amendments(&skill_dir.join("AMENDMENTS.jsonl"))?;
    validate_repeated_digests(
        skill_dir,
        &repo_root,
        &refreshed_manifest,
        &refreshed_amendments,
        &context,
    )?;
    context.source.assert_unchanged(skill_dir)?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}

fn append_refresh_amendment(
    amendments: &mut Vec<Value>,
    backlog_digest: &str,
    generated_at: &str,
) -> Result<()> {
    let latest = amendments
        .last()
        .ok_or_else(|| anyhow::anyhow!("AMENDMENTS.jsonl must contain at least one row"))?;
    let contract_hash = required_string(latest, "/new_contract_hash", "latest amendment")?;
    let previous_hash = required_string(latest, "/amendment_hash", "latest amendment")?;
    let before_ceiling = latest["after_claim_ceiling"].clone();
    let mut row = json!({
        "schema": "harness-ultragoal.contract-amendment.v1",
        "amendment_id": format!("AMEND-{:03}", amendments.len() + 1),
        "previous_contract_hash": contract_hash,
        "new_contract_hash": contract_hash,
        "previous_amendment_hash": previous_hash,
        "change_class": "evidence_refresh",
        "monotonicity": "preserves_or_strengthens",
        "affected_claim_ids": required_claim_ids(),
        "removed_or_weakened_claim_ids": [],
        "before_claim_ceiling": before_ceiling,
        "after_claim_ceiling": before_ceiling,
        "derived_removed_or_weakened_claim_ids": [],
        "derived_claim_delta_matches_declared": true,
        "approval": {"required": false, "status": "not_required"},
        "backlog_updates": [{
            "path": ".codex/skills/narrated-record-replay/VERIFICATION_BACKLOG.json",
            "digest": backlog_digest,
        }],
        "created_at": generated_at,
        "amendment_hash": "pending",
    });
    row["amendment_hash"] = json!(amendment_hash(&row)?);
    amendments.push(row);
    Ok(())
}

fn json_lines_bytes(rows: &[Value]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for row in rows {
        bytes.extend_from_slice(serde_json::to_string(row)?.as_bytes());
        bytes.push(b'\n');
    }
    if bytes.len() > 8 * 1024 * 1024 {
        bail!("refreshed amendment log exceeds 8 MiB limit");
    }
    Ok(bytes)
}

fn pretty_json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    if bytes.len() > 8 * 1024 * 1024 {
        bail!("refreshed bundle artifact exceeds 8 MiB limit");
    }
    Ok(bytes)
}

fn compact_json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    if bytes.len() > 8 * 1024 * 1024 {
        bail!("refreshed bundle artifact exceeds 8 MiB limit");
    }
    Ok(bytes)
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
