use super::amendments::{amendment_hash, read_amendments};
use super::receipt::{local_validator_receipt, parse_strict_utc_seconds};
use super::schema_rows::{validate_goal_binding, validate_manifest_claim};
use super::source_identity::SourceIdentity;
use super::trusted_context::TrustedBundleContext;
use super::trusted_goal::TrustedGoalObservation;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn rejects_semantically_false_rehashed_amendment() {
    let root = unique_tmp("nrr-amendment-semantic-regression");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("AMENDMENTS.jsonl");
    let mut row = amendment_row();
    row["monotonicity"] = json!("weakens");
    row["derived_removed_or_weakened_claim_ids"] = json!(["CLAIM-001"]);
    row["derived_claim_delta_matches_declared"] = json!(false);
    row["approval"] = json!({});
    row["amendment_hash"] = json!(amendment_hash(&row).unwrap());
    fs::write(&path, serde_json::to_string(&row).unwrap()).unwrap();

    assert!(read_amendments(&path).is_err());
}

#[test]
fn accepts_semantically_consistent_amendment() {
    let root = unique_tmp("nrr-amendment-semantic-positive");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("AMENDMENTS.jsonl");
    let mut row = amendment_row();
    row["amendment_hash"] = json!(amendment_hash(&row).unwrap());
    fs::write(&path, serde_json::to_string(&row).unwrap()).unwrap();

    assert_eq!(read_amendments(&path).unwrap().len(), 1);
}

#[test]
fn rejects_unknown_and_evidence_free_proven_claims() {
    for status in ["proven_by_attacker", "proven_static"] {
        let claim = claim(status);
        assert!(validate_manifest_claim(&claim, 0).is_err(), "{status}");
    }
}

#[test]
fn accepts_proven_claim_with_bound_evidence() {
    let mut value = claim("proven_static");
    value["allowed_evidence_surfaces"] = json!(["static"]);
    value["evidence"] = json!([{
        "id": "EVIDENCE-001",
        "claim_id": "CLAIM-001",
        "kind": "static_check",
        "surface": "static",
        "path": "src/bundle.rs",
        "digest": empty_digest()
    }]);

    validate_manifest_claim(&value, 0).unwrap();
}

#[test]
fn receipt_does_not_copy_self_declared_source_identity() {
    let (root, context) = trusted_context();
    let receipt = local_validator_receipt(
        &root,
        root.parent().unwrap(),
        &context,
        1,
        Some("security-regression"),
        None,
    )
    .unwrap();

    assert_ne!(receipt["commit"], "attacker-chosen-nonexistent-revision");
}

#[test]
fn measured_source_identity_changes_with_source_content() {
    let (root, _) = trusted_context();
    let before = SourceIdentity::measure(&root).unwrap();
    fs::write(
        root.join("src/main.rs"),
        "fn main() { println!(\"changed\"); }\n",
    )
    .unwrap();
    let after = SourceIdentity::measure(&root).unwrap();

    assert_ne!(before.digest(), after.digest());
    assert!(
        after
            .validate_manifest(&json!({"commit": before.digest()}))
            .is_err()
    );
    after
        .validate_manifest(&json!({"commit": after.digest()}))
        .unwrap();
}

#[test]
fn rejects_untrusted_goal_binding_strings() {
    let (_, context) = trusted_context();
    let binding = json!({
        "status": "fabricated-complete",
        "goal_id": "goal-beta",
        "objective": "different objective",
        "contract_path": "GOAL_CONTRACT.md",
        "checked_at": "not-even-a-timestamp"
    });

    assert!(validate_goal_binding(&binding, &context.goal).is_err());
}

#[test]
fn accepts_exact_trusted_goal_binding() {
    let (root, context) = trusted_context();
    let binding = json!({
        "status": "bound",
        "goal_id": "goal-alpha",
        "objective": "secure the bundle",
        "contract_path": root.join("GOAL_CONTRACT.md"),
        "checked_at": "2026-07-17T00:00:00Z"
    });

    validate_goal_binding(&binding, &context.goal).unwrap();
}

fn claim(status: &str) -> serde_json::Value {
    json!({
        "id": "CLAIM-001",
        "title": "claim",
        "claim_scope": "required",
        "claim_kind": "security",
        "status": status,
        "claim_surface": "static",
        "claim_ceiling_effect": "satisfied",
        "allowed_evidence_surfaces": [],
        "evidence": []
    })
}

fn amendment_row() -> serde_json::Value {
    json!({
        "schema": "harness-ultragoal.contract-amendment.v1",
        "amendment_id": "AMEND-001",
        "previous_contract_hash": empty_digest(),
        "new_contract_hash": "sha256:43eff85bb010b81d85da463f7eb44336529bfb1b87527a6eebddd9c25da6d986",
        "previous_amendment_hash": empty_digest(),
        "amendment_hash": "pending",
        "change_class": "strengthens",
        "monotonicity": "preserves_or_strengthens",
        "affected_claim_ids": ["CLAIM-001"],
        "removed_or_weakened_claim_ids": [],
        "before_claim_ceiling": [],
        "after_claim_ceiling": ["CLAIM-001"],
        "derived_removed_or_weakened_claim_ids": [],
        "derived_claim_delta_matches_declared": true,
        "approval": {"required": false, "status": "not_required"},
        "backlog_updates": [],
        "created_at": "2026-07-17T00:00:00Z"
    })
}

fn empty_digest() -> &'static str {
    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos}"))
}

fn trusted_context() -> (PathBuf, TrustedBundleContext) {
    let root = unique_tmp("nrr-trusted-context");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='test'\nversion='0.0.0'\n",
    )
    .unwrap();
    fs::write(root.join("Cargo.lock"), "").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("GOAL_CONTRACT.md"), "contract\n").unwrap();
    let contract_path = root.join("GOAL_CONTRACT.md");
    let goal = TrustedGoalObservation::from_value_at(
        json!({
            "schema": "narrated-record-replay.trusted-goal-observation.v1",
            "observation_id": "observation-alpha",
            "goal_id": "goal-alpha",
            "objective": "secure the bundle",
            "status": "active",
            "contract_path": contract_path,
            "contract_sha256": format!("sha256:{:x}", Sha256::digest(b"contract\n")),
            "observed_at": "2026-07-17T00:00:00Z"
        }),
        &root,
        parse_strict_utc_seconds("2026-07-17T00:00:00Z").unwrap(),
    )
    .unwrap();
    let context = TrustedBundleContext::for_test(SourceIdentity::measure(&root).unwrap(), goal);
    (root, context)
}
