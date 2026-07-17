use crate::config::Args;
use anyhow::Result;
use std::path::{Path, PathBuf};

mod amendment_semantics;
mod amendments;
mod authority;
mod claim_policy;
mod identity;
mod manifest_backlink;
mod receipt;
mod receipt_legacy;
mod refresh;
mod schema;
mod schema_rows;
mod source_identity;
mod trusted_context;
mod trusted_goal;
mod util;

use amendments::read_amendments;
use authority::validate_rust_only_authority;
use identity::{assert_contract_claim_ids, validate_identity_chain, validate_repeated_digests};
use receipt::local_validator_receipt;
use schema::{
    validate_backlog, validate_lane_registry, validate_manifest_shape, validate_red_fixtures,
};
use trusted_context::TrustedBundleContext;
use util::read_json;

const REQUIRED_CLAIM_COUNT: usize = 13;
const LOCAL_VALIDATOR_SCHEMA: &str = "narrated-record-replay.bundle-validation.v1";
const LOCAL_VALIDATOR_NAME: &str = "narrated-record-replay-rust-bundle-validator";
const LOCAL_VALIDATOR_VERSION: &str = "2026-07-17";
const LOCAL_VALIDATOR_RECEIPT_PATH: &str = ".codex/skills/narrated-record-replay/validation_artifacts/root-gate/current-rust-bundle-validation.json";
const LOCAL_VALIDATOR_CHECK_IDS: &[&str] = &[
    "required-paths-readable",
    "lane-registry-shape",
    "verification-backlog-shape",
    "completion-manifest-shape",
    "red-fixtures-shape",
    "amendment-log-shape",
    "amendment-semantics",
    "claim-evidence-policy",
    "required-claim-id-closure",
    "contract-hashes-current",
    "identity-chain",
    "trusted-source-identity",
    "trusted-live-goal-binding",
    "repo-relative-artifact-paths",
    "artifact-digests-current",
    "local-validator-receipts-current",
    "rust-only-current-authority",
];

pub fn validate_bundle(args: &Args) -> Result<()> {
    let skill_dir = args
        .skill_dir
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--skill-dir is required"))?;
    let repo_root = repo_root_for_skill_dir(skill_dir)?;
    let context = TrustedBundleContext::from_trusted_services(skill_dir)?;

    let lane_registry = read_json(&skill_dir.join("LANE_REGISTRY.json"))?;
    let backlog = read_json(&skill_dir.join("VERIFICATION_BACKLOG.json"))?;
    let manifest = read_json(&skill_dir.join("COMPLETION_MANIFEST.json"))?;
    let red_fixtures = read_json(&skill_dir.join("RED_FIXTURES.json"))?;
    let amendments = read_amendments(&skill_dir.join("AMENDMENTS.jsonl"))?;

    validate_lane_registry(&lane_registry)?;
    validate_backlog(&backlog)?;
    validate_manifest_shape(&manifest, &context.goal)?;
    claim_policy::validate_backlog_bindings(&manifest, &backlog)?;
    claim_policy::validate_manifest_claim_ceiling(&manifest, &amendments)?;
    context.source.validate_manifest(&manifest)?;
    validate_red_fixtures(&red_fixtures)?;
    assert_contract_claim_ids(&skill_dir.join("GOAL_CONTRACT.md"))?;
    validate_identity_chain(skill_dir, &manifest, &amendments)?;
    validate_repeated_digests(skill_dir, &repo_root, &manifest, &amendments, &context)?;
    validate_rust_only_authority(skill_dir)?;
    context.source.assert_unchanged(skill_dir)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&local_validator_receipt(
            skill_dir,
            &repo_root,
            &context,
            amendments.len(),
            args.receipt_run_id.as_deref(),
            args.receipt_generated_at.as_deref(),
        )?)?
    );
    Ok(())
}

pub fn refresh_bundle_receipt(args: &Args) -> Result<()> {
    refresh::refresh_bundle_receipt(args)
}

fn repo_root_for_skill_dir(skill_dir: &Path) -> Result<PathBuf> {
    let plugin_root = skill_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("--skill-dir must be inside a skills directory"))?;
    if plugin_root.join(".codex-plugin/plugin.json").is_file() {
        return Ok(plugin_root.to_path_buf());
    }
    skill_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            anyhow::anyhow!("--skill-dir must be inside a repo-local .codex/skills directory")
        })
}

#[cfg(test)]
mod amendments_extra_tests;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_args_from;

    #[test]
    fn validate_bundle_requires_trusted_goal_authority() {
        let skill_dir = env!("CARGO_MANIFEST_DIR");
        let args = parse_args_from([
            "nrr",
            "validate-bundle",
            "--skill-dir",
            skill_dir,
            "--receipt-run-id",
            "bundle-test",
            "--receipt-generated-at",
            "2026-06-24T23:50:00Z",
        ])
        .unwrap();

        let error = validate_bundle(&args).unwrap_err().to_string();
        assert!(
            error.contains("trusted goal-service attestation unavailable")
                || error.contains("environment goal observations are not accepted")
        );
    }

    #[test]
    fn validate_bundle_requires_skill_dir() {
        let args = parse_args_from(["nrr", "validate-bundle"]).unwrap();

        let error = validate_bundle(&args).unwrap_err().to_string();
        assert!(error.contains("--skill-dir is required"));
    }
}

#[cfg(test)]
mod identity_tests;

#[cfg(test)]
mod schema_util_tests;

#[cfg(test)]
mod schema_rows_tests;

#[cfg(test)]
mod schema_extra_tests;

#[cfg(test)]
mod security_fix_regression_tests;
