use super::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn coverage_policy_rejects_bad_template_schema_and_output() {
    let root = unique_tmp("nrr-coverage-policy-bad-template");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("COVERAGE_RECEIPT.json"),
        r#"{"target_schema":"wrong","canonical_output":"elsewhere.json"}"#,
    )
    .unwrap();

    let error = validate_coverage_policy(&root).unwrap_err().to_string();

    assert!(error.contains("target_schema"));
}

#[test]
fn coverage_policy_rejects_malformed_receipt_shape() {
    let root = unique_tmp("nrr-coverage-policy-bad-receipt");
    let receipt_dir = root.join("validation_artifacts/coverage");
    fs::create_dir_all(&receipt_dir).unwrap();
    write_template(&root);
    fs::write(
        receipt_dir.join("coverage-receipt.json"),
        r#"{"schema":"harness-ultragoal.coverage-receipt.v1","claim_id":""}"#,
    )
    .unwrap();

    let error = validate_coverage_policy(&root).unwrap_err().to_string();

    assert!(error.contains("claim_id"));
}

#[test]
fn coverage_policy_rejects_plain_test_receipt_from_files() {
    let root = unique_tmp("nrr-coverage-policy-plain-test");
    let receipt_dir = root.join("validation_artifacts/coverage");
    fs::create_dir_all(&receipt_dir).unwrap();
    write_template(&root);
    fs::write(
        receipt_dir.join("coverage-receipt.json"),
        serde_json::json!({
            "schema": "harness-ultragoal.coverage-receipt.v1",
            "claim_id": "CLAIM-005",
            "command": "cargo test --manifest-path Cargo.toml",
            "tool": "cargo test",
            "target_paths": ["src"],
            "measured_dimensions": ["line"],
            "coverage": {
                "percent": 95.0,
                "floor_percent": 100.0,
                "policy": "blocked_or_withheld",
                "owner": "root",
                "reason": "plain tests are not coverage",
                "blocker_or_debt_id": "coverage-command"
            },
            "uncovered_records": [],
            "exclusions": [],
            "generated_at": "2026-01-01T00:00:00Z",
            "claim_ceiling": "withheld_or_blocked"
        })
        .to_string(),
    )
    .unwrap();

    let error = validate_coverage_policy(&root).unwrap_err().to_string();

    assert!(error.contains("plain cargo test"));
}

#[test]
fn check_coverage_policy_accepts_valid_skill_dir_argument() {
    let root = unique_tmp("nrr-coverage-policy-cli-valid");
    let receipt_dir = root.join("validation_artifacts/coverage");
    let receipt_path = root.join("custom-coverage-receipt.json");
    fs::create_dir_all(&receipt_dir).unwrap();
    write_template(&root);
    let mut receipt = serde_json::json!({
        "schema": "harness-ultragoal.coverage-receipt.v1",
        "claim_id": "CLAIM-005",
        "command": "cargo llvm-cov --json",
        "tool": "cargo-llvm-cov",
        "target_paths": ["src"],
        "measured_dimensions": ["line", "region"],
        "coverage": {
            "percent": 100.0,
            "floor_percent": 100.0,
            "policy": "100_percent_required"
        },
        "uncovered_records": [],
        "exclusions": [],
        "generated_at": "2026-01-01T00:00:00Z",
        "claim_ceiling": "supports_complete_claim"
    });
    receipt["provenance"] = super::super::provenance::test_provenance(
        &root,
        "cargo llvm-cov --json",
        "2026-01-01T00:00:00Z",
    );
    fs::write(&receipt_path, receipt.to_string()).unwrap();
    let root_arg = root.to_string_lossy().to_string();
    let receipt_arg = receipt_path.to_string_lossy().to_string();
    let args = crate::config::parse_args_from([
        "nrr",
        "check-coverage-policy",
        "--skill-dir",
        &root_arg,
        "--coverage-receipt",
        &receipt_arg,
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();

    check_coverage_policy(&args).unwrap();
}

#[test]
fn coverage_policy_rejects_exclusions_missing_required_fields() {
    let receipt = serde_json::json!({
        "exclusions": [{
            "path": "generated/client.rs",
            "counts_as_covered": false,
            "reviewed": true
        }]
    });

    let error = validate_exclusions(&receipt).unwrap_err().to_string();

    assert!(error.contains("coverage exclusion#/kind"));
}

#[test]
fn policy_allows_template_only_before_validation_artifacts_exist() {
    let root = unique_tmp("nrr-coverage-policy-template-only");
    fs::create_dir_all(&root).unwrap();
    write_template(&root);
    validate_coverage_policy(&root).unwrap();
}

fn write_template(root: &std::path::Path) {
    fs::write(
        root.join("COVERAGE_RECEIPT.json"),
        serde_json::json!({
            "target_schema": "harness-ultragoal.coverage-receipt.v1",
            "canonical_output": "validation_artifacts/coverage/coverage-receipt.json"
        })
        .to_string(),
    )
    .unwrap();
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
