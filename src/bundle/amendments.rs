use super::amendment_semantics::validate_amendment_semantics;
use super::util::{read_text, require_schema, required_string};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) fn read_amendments(path: &Path) -> Result<Vec<Value>> {
    let text = read_text(path)?;
    let rows: Vec<Value> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str::<Value>(line)
                .with_context(|| format!("AMENDMENTS.jsonl:{} is not valid JSON", index + 1))
        })
        .collect::<Result<_>>()?;
    if rows.is_empty() {
        bail!("AMENDMENTS.jsonl must contain at least one amendment row");
    }
    let mut previous_hash =
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
    let mut previous_contract_hash: Option<String> = None;
    let mut previous_claim_ceiling: Option<Value> = None;
    for (index, row) in rows.iter().enumerate() {
        require_schema(
            row,
            &format!("AMENDMENTS.jsonl:{}", index + 1),
            "harness-ultragoal.contract-amendment.v1",
        )?;
        for key in [
            "amendment_id",
            "previous_contract_hash",
            "new_contract_hash",
            "previous_amendment_hash",
            "amendment_hash",
            "change_class",
            "monotonicity",
            "affected_claim_ids",
            "removed_or_weakened_claim_ids",
            "before_claim_ceiling",
            "after_claim_ceiling",
            "derived_removed_or_weakened_claim_ids",
            "derived_claim_delta_matches_declared",
            "approval",
            "backlog_updates",
            "created_at",
        ] {
            if row.get(key).is_none() {
                bail!("AMENDMENTS.jsonl:{} missing {key}", index + 1);
            }
        }
        for key in [
            "affected_claim_ids",
            "removed_or_weakened_claim_ids",
            "before_claim_ceiling",
            "after_claim_ceiling",
            "derived_removed_or_weakened_claim_ids",
            "backlog_updates",
        ] {
            if row.get(key).and_then(Value::as_array).is_none() {
                bail!("AMENDMENTS.jsonl:{} {key} must be an array", index + 1);
            }
        }
        if let Some(expected) = &previous_claim_ceiling
            && row.get("before_claim_ceiling") != Some(expected)
        {
            bail!(
                "AMENDMENTS.jsonl:{} before_claim_ceiling does not continue prior claim state",
                index + 1
            );
        }
        validate_amendment_semantics(row, index, previous_contract_hash.as_deref())?;
        let declared_previous = row
            .get("previous_amendment_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("AMENDMENTS.jsonl:{} missing previous hash", index + 1)
            })?;
        if declared_previous != previous_hash {
            bail!(
                "AMENDMENTS.jsonl:{} previous_amendment_hash mismatch: expected {}, got {}",
                index + 1,
                previous_hash,
                declared_previous
            );
        }
        let actual_hash = amendment_hash(row)?;
        let declared_hash = row
            .get("amendment_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("AMENDMENTS.jsonl:{} missing amendment hash", index + 1)
            })?;
        if declared_hash != actual_hash {
            bail!(
                "AMENDMENTS.jsonl:{} amendment_hash mismatch: expected {}, got {}",
                index + 1,
                actual_hash,
                declared_hash
            );
        }
        previous_hash = actual_hash;
        previous_contract_hash =
            Some(required_string(row, "/new_contract_hash", "amendment")?.into());
        previous_claim_ceiling = row.get("after_claim_ceiling").cloned();
    }
    Ok(rows)
}

pub(super) fn amendment_hash(row: &Value) -> Result<String> {
    let mut value = row.clone();
    value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("amendment row must be an object"))?
        .remove("amendment_hash");
    let digest = Sha256::digest(serde_json::to_vec(&value)?);
    Ok(format!("sha256:{digest:x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn amendment_log_rejects_empty_and_malformed_rows() {
        let root = unique_tmp("nrr-amendments-empty");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("AMENDMENTS.jsonl");
        fs::write(&path, "\n").unwrap();
        assert!(
            read_amendments(&path)
                .unwrap_err()
                .to_string()
                .contains("at least one")
        );

        fs::write(&path, "{not-json}\n").unwrap();
        assert!(
            read_amendments(&path)
                .unwrap_err()
                .to_string()
                .contains("not valid JSON")
        );
    }

    #[test]
    fn amendment_log_rejects_missing_or_bad_shape_fields() {
        let root = unique_tmp("nrr-amendments-shape");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("AMENDMENTS.jsonl");
        let mut row = base_row();
        row.as_object_mut().unwrap().remove("approval");
        fs::write(&path, serde_json::to_string(&row).unwrap()).unwrap();
        assert!(
            read_amendments(&path)
                .unwrap_err()
                .to_string()
                .contains("missing approval")
        );

        let mut row = base_row();
        row["affected_claim_ids"] = json!("CLAIM-001");
        fs::write(&path, serde_json::to_string(&row).unwrap()).unwrap();
        assert!(
            read_amendments(&path)
                .unwrap_err()
                .to_string()
                .contains("must be an array")
        );

        let mut row = base_row();
        row["approval"] = json!("approved");
        fs::write(&path, serde_json::to_string(&row).unwrap()).unwrap();
        assert!(
            read_amendments(&path)
                .unwrap_err()
                .to_string()
                .contains("approval must be an object")
        );
    }

    #[test]
    fn amendment_log_validates_previous_and_current_hashes() {
        let root = unique_tmp("nrr-amendments-hash");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("AMENDMENTS.jsonl");
        let mut row = base_row();
        row["amendment_hash"] = json!(amendment_hash(&row).unwrap());
        fs::write(&path, serde_json::to_string(&row).unwrap()).unwrap();
        assert_eq!(read_amendments(&path).unwrap().len(), 1);

        let mut bad_previous = row.clone();
        bad_previous["previous_amendment_hash"] = json!("sha256:bad");
        bad_previous["amendment_hash"] = json!(amendment_hash(&bad_previous).unwrap());
        fs::write(&path, serde_json::to_string(&bad_previous).unwrap()).unwrap();
        assert!(
            read_amendments(&path)
                .unwrap_err()
                .to_string()
                .contains("previous_amendment_hash mismatch")
        );

        let mut bad_hash = row;
        bad_hash["amendment_hash"] = json!("sha256:bad");
        fs::write(&path, serde_json::to_string(&bad_hash).unwrap()).unwrap();
        assert!(
            read_amendments(&path)
                .unwrap_err()
                .to_string()
                .contains("amendment_hash mismatch")
        );
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
}
