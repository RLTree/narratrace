use super::authority::validate_rust_only_authority;
use super::identity::{
    assert_contract_claim_ids, validate_identity_chain, validate_repeated_digests,
};
use super::source_identity::SourceIdentity;
use super::trusted_context::TrustedBundleContext;
use super::trusted_goal::TrustedGoalObservation;
use super::util::{required_claim_ids, required_claim_ids_hash};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn contract_claim_ids_report_missing_canonical_ids() {
    let root = unique_tmp("nrr-bundle-claim-ids");
    fs::create_dir_all(&root).unwrap();
    let contract = root.join("GOAL_CONTRACT.md");
    fs::write(&contract, "one incomplete contract").unwrap();

    let error = assert_contract_claim_ids(&contract)
        .unwrap_err()
        .to_string();

    assert!(error.contains("GOAL_CONTRACT.md missing canonical claim IDs"));
}

#[test]
fn contract_claim_ids_accept_complete_contract() {
    let root = unique_tmp("nrr-bundle-claim-ids-complete");
    fs::create_dir_all(&root).unwrap();
    let contract = root.join("GOAL_CONTRACT.md");
    fs::write(&contract, required_claim_ids().join("\n")).unwrap();

    assert_contract_claim_ids(&contract).unwrap();
}

#[test]
fn identity_chain_rejects_required_claim_hash_drift() {
    let root = unique_tmp("nrr-bundle-identity-hash");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("GOAL_CONTRACT.md"), "contract").unwrap();
    let manifest = json!({
        "contract_bundle_hash": digest_for("contract"),
        "required_claim_ids_hash": "sha256:not-current"
    });

    let error = validate_identity_chain(&root, &manifest, &[])
        .unwrap_err()
        .to_string();

    assert!(error.contains("required_claim_ids_hash mismatch"));
    assert!(error.contains(&required_claim_ids_hash()));
}

#[test]
fn identity_chain_reports_missing_required_manifest_and_amendment_fields() {
    let root = unique_tmp("nrr-bundle-identity-missing-fields");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("GOAL_CONTRACT.md"), "contract").unwrap();

    let error = validate_identity_chain(&root, &json!({}), &[])
        .unwrap_err()
        .to_string();
    assert!(error.contains("contract_bundle_hash"));

    let manifest = json!({
        "contract_bundle_hash": digest_for("contract"),
        "required_claim_ids_hash": required_claim_ids_hash()
    });
    let amendments = vec![json!({
        "previous_amendment_hash": empty_digest()
    })];
    let error = validate_identity_chain(&root, &manifest, &amendments)
        .unwrap_err()
        .to_string();
    assert!(error.contains("amendment_hash"));
}

#[test]
fn identity_chain_rejects_amendment_hash_break() {
    let root = unique_tmp("nrr-bundle-amendment-chain");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("GOAL_CONTRACT.md"), "contract").unwrap();
    let manifest = json!({
        "contract_bundle_hash": digest_for("contract"),
        "required_claim_ids_hash": required_claim_ids_hash()
    });
    let amendments = vec![json!({
        "previous_amendment_hash": "sha256:wrong",
        "amendment_hash": "sha256:next"
    })];

    let error = validate_identity_chain(&root, &manifest, &amendments)
        .unwrap_err()
        .to_string();

    assert!(error.contains("previous_amendment_hash mismatch"));
}

#[test]
fn identity_chain_accepts_current_hashes_and_amendment_chain() {
    let root = unique_tmp("nrr-bundle-identity-success");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("GOAL_CONTRACT.md"), "contract").unwrap();
    let manifest = json!({
        "contract_bundle_hash": digest_for("contract"),
        "required_claim_ids_hash": required_claim_ids_hash()
    });
    let amendments = vec![
        json!({
            "previous_amendment_hash": empty_digest(),
            "amendment_hash": digest_for("first")
        }),
        json!({
            "previous_amendment_hash": digest_for("first"),
            "amendment_hash": digest_for("second")
        }),
    ];

    validate_identity_chain(&root, &manifest, &amendments).unwrap();
}

#[test]
fn rust_only_authority_rejects_retired_python_tokens() {
    let root = unique_tmp("nrr-bundle-rust-only");
    for path in [
        "AGENT_STANDARDS.md",
        "ARCHITECTURE.md",
        "COMPLETION_MANIFEST.json",
        "REFERENCES.md",
        "SKILL.md",
        "VALIDATION.md",
        "VERIFICATION_BACKLOG.json",
        "scripts/check",
    ] {
        let full = root.join(path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, "current Rust authority").unwrap();
    }
    fs::write(root.join("VALIDATION.md"), "run python3 old-validator").unwrap();

    let error = validate_rust_only_authority(&root).unwrap_err().to_string();

    assert!(error.contains("VALIDATION.md contains retired Python authority token"));
}

#[test]
fn rust_only_authority_accepts_current_authority_files() {
    let root = unique_tmp("nrr-bundle-rust-only-success");
    write_current_authority_files(&root);

    validate_rust_only_authority(&root).unwrap();
}

#[test]
fn repeated_digest_validation_requires_receipts_and_amendments() {
    let root = unique_tmp("nrr-bundle-repeated-digests");
    fs::create_dir_all(&root).unwrap();
    let manifest = json!({
        "verification_backlog_path": ".codex/skills/narrated-record-replay/VERIFICATION_BACKLOG.json",
        "verification_backlog_digest": digest_for("{}"),
        "validator_receipts": []
    });

    let context = TrustedBundleContext::for_test(
        SourceIdentity::for_test("sha256:test"),
        TrustedGoalObservation::for_test(),
    );
    let error = validate_repeated_digests(&root, &root, &manifest, &[], &context)
        .unwrap_err()
        .to_string();

    assert!(error.contains("VERIFICATION_BACKLOG.json"));
}

fn write_current_authority_files(root: &std::path::Path) {
    for path in [
        "AGENT_STANDARDS.md",
        "ARCHITECTURE.md",
        "COMPLETION_MANIFEST.json",
        "REFERENCES.md",
        "SKILL.md",
        "VALIDATION.md",
        "VERIFICATION_BACKLOG.json",
        "scripts/check",
    ] {
        let full = root.join(path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, "current Rust authority").unwrap();
    }
}

fn digest_for(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(text.as_bytes()))
}

fn empty_digest() -> String {
    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
