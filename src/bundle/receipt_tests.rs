use super::super::util::required_claim_ids;
use super::*;
use crate::bundle::source_identity::SourceIdentity;
use crate::bundle::trusted_context::TrustedBundleContext;
use crate::bundle::trusted_goal::TrustedGoalObservation;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SKILL_DIR: &str = "/private/tmp/nrr-receipt-repo/.codex/skills/narrated-record-replay";
const REPO_ROOT: &str = "/private/tmp/nrr-receipt-repo";

#[test]
fn required_claim_ids_are_canonical() {
    assert_eq!(required_claim_ids().first().unwrap(), "CLAIM-001");
    assert_eq!(required_claim_ids().last().unwrap(), "CLAIM-013");
    assert_eq!(required_claim_ids().len(), 13);
}

#[test]
fn rejects_status_only_validator_receipt() {
    let path = write_receipt(
        "status-only",
        json!({
            "schema": LOCAL_VALIDATOR_SCHEMA,
            "status": "passed"
        }),
    );
    let now = parse_strict_utc_seconds("2026-07-16T00:00:00Z").unwrap();
    let context = test_context(now);

    assert!(validate_at(&path, now, &context).is_err());
}

#[test]
fn validator_receipt_binds_target_root_and_commit() {
    let now = parse_strict_utc_seconds("2026-07-16T00:00:00Z").unwrap();
    let context = test_context(now);
    let mut receipt = valid_receipt("2026-07-16T00:00:00Z", &context);
    for (pointer, forged) in [
        ("/target/skill_dir", "/private/tmp/other/skill"),
        ("/target/repo_root", "/private/tmp/other"),
        ("/root", "/private/tmp/other/skill"),
        ("/commit", "other-commit"),
    ] {
        *receipt.pointer_mut(pointer).unwrap() = json!(forged);
        let path = write_receipt("wrong-context", receipt.clone());
        assert!(validate_at(&path, now, &context).is_err(), "{pointer}");
        receipt = valid_receipt("2026-07-16T00:00:00Z", &context);
    }
}

#[test]
fn validator_receipt_rejects_malformed_stale_and_future_times() {
    let now = parse_strict_utc_seconds("2026-07-16T00:00:00Z").unwrap();
    let context = test_context(now);
    for (generated_at, message) in [
        ("not-a-time", "strict UTC RFC3339"),
        ("2026-02-31T00:00:00Z", "strict UTC RFC3339"),
        ("2026-07-14T00:00:00Z", "stale"),
        ("2026-07-16T00:06:00Z", "future skew"),
    ] {
        let path = write_receipt("bad-time", valid_receipt(generated_at, &context));
        let error = validate_at(&path, now, &context).unwrap_err().to_string();
        assert!(error.contains(message), "{generated_at}: {error}");
    }
}

#[test]
fn local_receipt_uses_trusted_current_time_not_caller_text() {
    let before = system_unix_ms().unwrap();
    let context = test_context(parse_strict_utc_seconds("2026-07-16T00:00:00Z").unwrap());
    let receipt = local_validator_receipt(
        Path::new(SKILL_DIR),
        Path::new(REPO_ROOT),
        &context,
        1,
        Some("test-run"),
        Some("not-a-time"),
    )
    .unwrap();
    let after = system_unix_ms().unwrap();
    let generated = parse_strict_utc_seconds(receipt["generated_at"].as_str().unwrap()).unwrap();

    assert_ne!(receipt["generated_at"], "not-a-time");
    assert!(generated >= before - 1_000 && generated <= after);
    assert_eq!(receipt["target"]["skill_dir"], SKILL_DIR);
    assert_eq!(receipt["commit"], context.source.digest());

    assert!(
        local_validator_receipt(
            Path::new(SKILL_DIR),
            Path::new(REPO_ROOT),
            &context,
            1,
            Some(" "),
            None,
        )
        .is_err()
    );
}

fn validate_at(path: &Path, now: i64, context: &TrustedBundleContext) -> Result<()> {
    validate_validator_receipt_at(
        path,
        Path::new(SKILL_DIR),
        Path::new(REPO_ROOT),
        context,
        now,
    )
}

fn valid_receipt(generated_at: &str, context: &TrustedBundleContext) -> Value {
    let checks = LOCAL_VALIDATOR_CHECK_IDS
        .iter()
        .map(|id| json!({"id": id, "status": "passed"}))
        .collect::<Vec<_>>();
    json!({
        "schema": LOCAL_VALIDATOR_SCHEMA,
        "validator": LOCAL_VALIDATOR_NAME,
        "version": LOCAL_VALIDATOR_VERSION,
        "status": "passed",
        "run_id": "test-run",
        "target": {"skill_dir": SKILL_DIR, "repo_root": REPO_ROOT},
        "root": SKILL_DIR,
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
        "generated_at": generated_at,
        "required_check_ids": LOCAL_VALIDATOR_CHECK_IDS,
        "checks": checks
    })
}

fn test_context(now: i64) -> TrustedBundleContext {
    let root = unique_tmp("receipt-context");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='receipt-test'\nversion='0.0.0'\n",
    )
    .unwrap();
    fs::write(root.join("Cargo.lock"), "").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("GOAL_CONTRACT.md"), "test contract\n").unwrap();
    let contract_digest = format!("sha256:{:x}", Sha256::digest(b"test contract\n"));
    let contract_path = root.join("GOAL_CONTRACT.md");
    let goal = TrustedGoalObservation::from_value_at(
        json!({
            "schema": "narrated-record-replay.trusted-goal-observation.v1",
            "observation_id": "observation-1",
            "goal_id": "goal-1",
            "objective": "test objective",
            "status": "active",
            "contract_path": contract_path,
            "contract_sha256": contract_digest,
            "observed_at": "2026-07-16T00:00:00Z"
        }),
        &root,
        now,
    )
    .unwrap();
    TrustedBundleContext::for_test(SourceIdentity::measure(&root).unwrap(), goal)
}

fn write_receipt(prefix: &str, receipt: Value) -> PathBuf {
    let root = unique_tmp(prefix);
    fs::create_dir_all(&root).unwrap();
    let path = root.join("receipt.json");
    fs::write(&path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    path
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
