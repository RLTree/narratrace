use super::trusted_context::TrustedBundleContext;
use super::util::{read_json, require_array, require_schema, required_claim_ids, required_string};
use super::{
    LOCAL_VALIDATOR_CHECK_IDS, LOCAL_VALIDATOR_NAME, LOCAL_VALIDATOR_SCHEMA,
    LOCAL_VALIDATOR_VERSION,
};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

const MAX_RECEIPT_AGE_MS: i64 = 24 * 60 * 60 * 1_000;
const MAX_FUTURE_SKEW_MS: i64 = 5 * 60 * 1_000;

pub(super) fn validate_validator_receipt(
    path: &Path,
    expected_skill_dir: &Path,
    expected_repo_root: &Path,
    context: &TrustedBundleContext,
) -> Result<()> {
    validate_validator_receipt_at_with_policy(
        path,
        expected_skill_dir,
        expected_repo_root,
        context,
        system_unix_ms()?,
    )
}

pub(super) fn validate_validator_receipt_allow_stale(
    path: &Path,
    expected_skill_dir: &Path,
    expected_repo_root: &Path,
    expected_commit: &str,
) -> Result<()> {
    super::receipt_legacy::validate_legacy_receipt(
        path,
        expected_skill_dir,
        expected_repo_root,
        expected_commit,
    )
}

#[cfg(test)]
fn validate_validator_receipt_at(
    path: &Path,
    expected_skill_dir: &Path,
    expected_repo_root: &Path,
    context: &TrustedBundleContext,
    now_unix_ms: i64,
) -> Result<()> {
    validate_validator_receipt_at_with_policy(
        path,
        expected_skill_dir,
        expected_repo_root,
        context,
        now_unix_ms,
    )
}

fn validate_validator_receipt_at_with_policy(
    path: &Path,
    expected_skill_dir: &Path,
    expected_repo_root: &Path,
    context: &TrustedBundleContext,
    now_unix_ms: i64,
) -> Result<()> {
    let receipt = read_json(path)?;
    require_schema(
        &receipt,
        &path.display().to_string(),
        LOCAL_VALIDATOR_SCHEMA,
    )?;
    let expected_skill_dir = expected_skill_dir.to_string_lossy().into_owned();
    let expected_repo_root = expected_repo_root.to_string_lossy().into_owned();
    for (pointer, expected) in [
        ("/validator", LOCAL_VALIDATOR_NAME),
        ("/version", LOCAL_VALIDATOR_VERSION),
        ("/status", "passed"),
        ("/target/skill_dir", expected_skill_dir.as_str()),
        ("/target/repo_root", expected_repo_root.as_str()),
        ("/root", expected_skill_dir.as_str()),
        ("/commit", context.source.digest()),
        ("/sourceIdentity/digest", context.source.digest()),
        ("/sourceIdentity/kind", "directory_snapshot"),
        (
            "/sourceIdentity/algorithm",
            "narrated-record-replay-source-v1",
        ),
        ("/goalBinding/goalId", context.goal.goal_id()),
        ("/goalBinding/observationId", context.goal.observation_id()),
        ("/goalBinding/observedAt", context.goal.observed_at()),
        ("/goalBinding/bindingDigest", context.goal.binding_digest()),
    ] {
        let actual = required_string(&receipt, pointer, "local validator receipt")?;
        if actual != expected {
            bail!(
                "local validator receipt {} expected {pointer}={expected}, got {actual}",
                path.display()
            );
        }
    }
    if required_string(&receipt, "/run_id", "local validator receipt")?
        .trim()
        .is_empty()
    {
        bail!("local validator receipt run_id must not be empty");
    }
    validate_receipt_freshness(&receipt, now_unix_ms)?;
    validate_check_rows(&receipt)?;
    Ok(())
}

pub(super) fn validate_receipt_timestamp_shape(receipt: &Value) -> Result<()> {
    let generated_at = required_string(receipt, "/generated_at", "local validator receipt")?;
    parse_strict_utc_seconds(generated_at).ok_or_else(|| {
        anyhow::anyhow!("local validator receipt generated_at must be strict UTC RFC3339 seconds")
    })?;
    Ok(())
}

fn validate_receipt_freshness(receipt: &Value, now_unix_ms: i64) -> Result<()> {
    let generated_at = required_string(receipt, "/generated_at", "local validator receipt")?;
    let generated_at_ms = parse_strict_utc_seconds(generated_at).ok_or_else(|| {
        anyhow::anyhow!("local validator receipt generated_at must be strict UTC RFC3339 seconds")
    })?;
    let age_ms = now_unix_ms
        .checked_sub(generated_at_ms)
        .ok_or_else(|| anyhow::anyhow!("local validator receipt age overflow"))?;
    if age_ms < -MAX_FUTURE_SKEW_MS {
        bail!("local validator receipt generated_at exceeds allowed future skew");
    }
    if age_ms > MAX_RECEIPT_AGE_MS {
        bail!("local validator receipt is stale");
    }
    Ok(())
}

pub(super) fn validate_check_rows(receipt: &Value) -> Result<()> {
    let required_check_ids =
        require_array(receipt, "/required_check_ids", "local validator receipt")?;
    let declared: BTreeSet<String> = required_check_ids
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("local validator receipt check ids must be strings"))
                .map(str::to_string)
        })
        .collect::<Result<_>>()?;
    let expected: BTreeSet<String> = LOCAL_VALIDATOR_CHECK_IDS
        .iter()
        .map(|value| value.to_string())
        .collect();
    if declared != expected {
        bail!("local validator receipt required_check_ids mismatch");
    }
    let checks = require_array(receipt, "/checks", "local validator receipt")?;
    let mut seen = BTreeSet::new();
    for check in checks {
        let id = required_string(check, "/id", "local validator receipt check")?;
        let status = required_string(check, "/status", "local validator receipt check")?;
        if status != "passed" {
            bail!("local validator receipt check {id} is not passed");
        }
        seen.insert(id.to_string());
    }
    if seen != expected {
        bail!("local validator receipt checks do not match required_check_ids");
    }
    Ok(())
}

pub(super) fn local_validator_receipt(
    skill_dir: &Path,
    repo_root: &Path,
    context: &TrustedBundleContext,
    amendment_rows: usize,
    receipt_run_id: Option<&str>,
    _receipt_generated_at: Option<&str>,
) -> Result<Value> {
    let checks = LOCAL_VALIDATOR_CHECK_IDS
        .iter()
        .map(|id| json!({"id": id, "status": "passed"}))
        .collect::<Vec<_>>();
    let generated_at = trusted_utc_now()?;
    let run_id = receipt_run_id.unwrap_or("local-rust-bundle-validation");
    if run_id.trim().is_empty() {
        bail!("local validator receipt run_id must not be empty");
    }
    Ok(json!({
        "schema": LOCAL_VALIDATOR_SCHEMA,
        "validator": LOCAL_VALIDATOR_NAME,
        "version": LOCAL_VALIDATOR_VERSION,
        "run_id": run_id,
        "status": "passed",
        "target": {"skill_dir": skill_dir, "repo_root": repo_root},
        "commit": context.source.digest(),
        "sourceIdentity": {
            "kind": "directory_snapshot",
            "algorithm": "narrated-record-replay-source-v1",
            "digest": context.source.digest()
        },
        "goalBinding": {
            "goalId": context.goal.goal_id(),
            "observationId": context.goal.observation_id(),
            "observedAt": context.goal.observed_at(),
            "bindingDigest": context.goal.binding_digest()
        },
        "root": skill_dir,
        "pythonRequired": false,
        "amendmentRows": amendment_rows,
        "claimIds": required_claim_ids(),
        "required_check_ids": LOCAL_VALIDATOR_CHECK_IDS,
        "check_set_digest": local_check_set_digest(),
        "checks": checks,
        "generated_artifacts": [{
            "path": super::LOCAL_VALIDATOR_RECEIPT_PATH,
            "kind": "local-rust-bundle-validation-receipt"
        }],
        "claimCeiling": "local Rust bundle validation only; not a full ultragoal-audit receipt, review-team sign-off, live capability proof, or product-cohesion proof",
        "generated_at": generated_at
    }))
}

fn local_check_set_digest() -> String {
    let joined = LOCAL_VALIDATOR_CHECK_IDS.join("\n") + "\n";
    format!("sha256:{:x}", Sha256::digest(joined.as_bytes()))
}

include!("receipt_time.rs");

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod tests;
