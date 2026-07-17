use super::util::{require_array, required_claim_ids, required_string};
use anyhow::{Result, bail};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn validate_amendment_semantics(
    row: &Value,
    index: usize,
    previous_contract_hash: Option<&str>,
) -> Result<()> {
    let context = format!("AMENDMENTS.jsonl:{}", index + 1);
    let previous = required_string(row, "/previous_contract_hash", &context)?;
    let next = required_string(row, "/new_contract_hash", &context)?;
    validate_digest(previous, &context)?;
    validate_digest(next, &context)?;
    if index == 0
        && (previous != "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            || !claim_set(row, "/before_claim_ceiling", &context)?.is_empty())
    {
        bail!("{context} genesis amendment must start from the empty contract state");
    }
    if let Some(expected) = previous_contract_hash
        && previous != expected
    {
        bail!("{context} previous_contract_hash does not continue prior contract state");
    }
    if row
        .get("derived_claim_delta_matches_declared")
        .and_then(Value::as_bool)
        != Some(true)
    {
        bail!("{context} derived_claim_delta_matches_declared must be true");
    }
    if required_string(row, "/monotonicity", &context)? != "preserves_or_strengthens" {
        bail!("{context} weakening amendments require unavailable trusted approval authority");
    }

    let before = claim_set(row, "/before_claim_ceiling", &context)?;
    let after = claim_set(row, "/after_claim_ceiling", &context)?;
    let declared = claim_set(row, "/removed_or_weakened_claim_ids", &context)?;
    let derived = claim_set(row, "/derived_removed_or_weakened_claim_ids", &context)?;
    let actual_removed = before.difference(&after).cloned().collect::<BTreeSet<_>>();
    if !actual_removed.is_empty() || declared != actual_removed || derived != actual_removed {
        bail!("{context} declared and derived claim deltas do not match the contract transition");
    }

    match required_string(row, "/change_class", &context)? {
        "strengthens" if after.is_superset(&before) && after != before && next != previous => {}
        "clarifies" if after == before => {}
        "evidence_refresh" if after == before && next == previous => {}
        "strengthens" => bail!("{context} strengthens must add at least one claim"),
        "clarifies" | "evidence_refresh" => {
            bail!("{context} non-semantic amendment must preserve the claim ceiling")
        }
        other => bail!("{context} unsupported change_class: {other}"),
    }
    let approval = row
        .get("approval")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("{context} approval must be an object"))?;
    if approval.get("required").and_then(Value::as_bool) != Some(false)
        || approval.get("status").and_then(Value::as_str) != Some("not_required")
    {
        bail!("{context} non-weakening approval must be explicitly not_required");
    }
    Ok(())
}

fn claim_set(row: &Value, pointer: &str, context: &str) -> Result<BTreeSet<String>> {
    let canonical = required_claim_ids().into_iter().collect::<BTreeSet<_>>();
    let values = require_array(row, pointer, context)?;
    let mut out = BTreeSet::new();
    for value in values {
        let claim = value
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("{context}#{pointer} must contain strings"))?;
        if !canonical.contains(claim) {
            bail!("{context}#{pointer} contains unknown claim id {claim}");
        }
        if !out.insert(claim.to_string()) {
            bail!("{context}#{pointer} contains duplicate claim id {claim}");
        }
    }
    Ok(out)
}

fn validate_digest(value: &str, context: &str) -> Result<()> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        bail!("{context} contract digest must use sha256");
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{context} contract digest must contain 64 hexadecimal characters");
    }
    Ok(())
}
