use super::amendments::read_amendments;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn amendment_log_rejects_bad_schema_and_non_boolean_delta_flag() {
    let root = unique_tmp("nrr-amendments-extra-shape");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("AMENDMENTS.jsonl");

    let mut bad_schema = base_row();
    bad_schema["schema"] = json!("wrong.schema");
    write_hashed_row(&path, &mut bad_schema);
    assert!(
        read_amendments(&path)
            .unwrap_err()
            .to_string()
            .contains("schema")
    );

    let mut bad_bool = base_row();
    bad_bool["derived_claim_delta_matches_declared"] = json!("true");
    write_hashed_row(&path, &mut bad_bool);
    assert!(
        read_amendments(&path)
            .unwrap_err()
            .to_string()
            .contains("derived_claim_delta_matches_declared must be true")
    );
}

#[test]
fn amendment_log_accepts_two_row_hash_chain() {
    let root = unique_tmp("nrr-amendments-extra-chain");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("AMENDMENTS.jsonl");
    let mut first = base_row();
    write_hash(&mut first);
    let mut second = base_row();
    second["amendment_id"] = json!("AMEND-002");
    second["previous_contract_hash"] = first["new_contract_hash"].clone();
    second["new_contract_hash"] = first["new_contract_hash"].clone();
    second["previous_amendment_hash"] = first["amendment_hash"].clone();
    second["change_class"] = json!("clarifies");
    second["before_claim_ceiling"] = first["after_claim_ceiling"].clone();
    second["after_claim_ceiling"] = first["after_claim_ceiling"].clone();
    write_hash(&mut second);
    fs::write(
        &path,
        format!(
            "{}\n{}\n",
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        ),
    )
    .unwrap();

    assert_eq!(read_amendments(&path).unwrap().len(), 2);
}

fn write_hashed_row(path: &std::path::Path, row: &mut Value) {
    write_hash(row);
    fs::write(path, serde_json::to_string(row).unwrap()).unwrap();
}

fn write_hash(row: &mut Value) {
    row["amendment_hash"] = json!(hash_without_declared(row));
}

fn hash_without_declared(row: &Value) -> String {
    let mut value = row.clone();
    value.as_object_mut().unwrap().remove("amendment_hash");
    format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&value).unwrap())
    )
}

fn base_row() -> Value {
    json!({
        "schema": "harness-ultragoal.contract-amendment.v1",
        "amendment_id": "AMEND-001",
        "previous_contract_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "new_contract_hash": "sha256:43eff85bb010b81d85da463f7eb44336529bfb1b87527a6eebddd9c25da6d986",
        "previous_amendment_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "amendment_hash": "sha256:placeholder",
        "change_class": "strengthens",
        "monotonicity": "preserves_or_strengthens",
        "affected_claim_ids": ["CLAIM-005"],
        "removed_or_weakened_claim_ids": [],
        "before_claim_ceiling": [],
        "after_claim_ceiling": ["CLAIM-005"],
        "derived_removed_or_weakened_claim_ids": [],
        "derived_claim_delta_matches_declared": true,
        "approval": {"required": false, "status": "not_required"},
        "backlog_updates": [],
        "created_at": "2026-06-24T00:00:00Z"
    })
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
