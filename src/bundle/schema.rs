use super::schema_rows::{
    validate_backlog_row, validate_goal_binding, validate_lane, validate_manifest_claim,
};
use super::trusted_goal::TrustedGoalObservation;
use super::util::{require_array, require_schema, required_claim_ids, required_string};
use anyhow::{Result, bail};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn validate_lane_registry(value: &Value) -> Result<()> {
    require_schema(
        value,
        "LANE_REGISTRY.json",
        "harness-ultragoal.lane-registry.v1",
    )?;
    require_array(value, "/lanes", "LANE_REGISTRY.json")?;
    required_string(value, "/goal_id", "LANE_REGISTRY.json")?;
    required_string(value, "/canonical_authority", "LANE_REGISTRY.json")?;
    required_string(value, "/generated_at", "LANE_REGISTRY.json")?;
    for (index, lane) in require_array(value, "/lanes", "LANE_REGISTRY.json")?
        .iter()
        .enumerate()
    {
        validate_lane(lane, index)?;
    }
    Ok(())
}

pub(super) fn validate_backlog(value: &Value) -> Result<()> {
    require_schema(
        value,
        "VERIFICATION_BACKLOG.json",
        "harness-ultragoal.verification-backlog.v1",
    )?;
    require_array(value, "/rows", "VERIFICATION_BACKLOG.json")?;
    required_string(value, "/manifest_digest", "VERIFICATION_BACKLOG.json")?;
    required_string(value, "/generated_at", "VERIFICATION_BACKLOG.json")?;
    for (index, row) in require_array(value, "/rows", "VERIFICATION_BACKLOG.json")?
        .iter()
        .enumerate()
    {
        validate_backlog_row(row, index)?;
    }
    Ok(())
}

pub(super) fn validate_manifest_shape(
    value: &Value,
    trusted_goal: &TrustedGoalObservation,
) -> Result<()> {
    require_schema(
        value,
        "COMPLETION_MANIFEST.json",
        "harness-ultragoal.completion-manifest.v1",
    )?;
    require_array(value, "/required_claim_ids", "COMPLETION_MANIFEST.json")?;
    require_array(value, "/claims", "COMPLETION_MANIFEST.json")?;
    for pointer in [
        "/contract_bundle_hash",
        "/required_claim_ids_hash",
        "/commit",
        "/root",
        "/verification_backlog_path",
        "/verification_backlog_digest",
        "/generated_at",
    ] {
        required_string(value, pointer, "COMPLETION_MANIFEST.json")?;
    }
    validate_goal_binding(
        value
            .get("goal_binding")
            .ok_or_else(|| anyhow::anyhow!("COMPLETION_MANIFEST.json#/goal_binding is required"))?,
        trusted_goal,
    )?;
    let required: BTreeSet<String> = value
        .pointer("/required_claim_ids")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|claim| {
            claim
                .as_str()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "COMPLETION_MANIFEST.json#/required_claim_ids must contain strings"
                    )
                })
                .map(str::to_string)
        })
        .collect::<Result<_>>()?;
    let expected: BTreeSet<String> = required_claim_ids().into_iter().collect();
    if required != expected {
        bail!("COMPLETION_MANIFEST.json required_claim_ids do not match CLAIM-001..CLAIM-013");
    }
    let claim_rows: BTreeSet<String> = value
        .pointer("/claims")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|claim| {
            claim
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    anyhow::anyhow!("COMPLETION_MANIFEST.json#/claims rows must have string id")
                })
                .map(str::to_string)
        })
        .collect::<Result<_>>()?;
    if claim_rows != expected {
        bail!("COMPLETION_MANIFEST.json#/claims ids must exactly match required_claim_ids");
    }
    for (index, claim) in require_array(value, "/claims", "COMPLETION_MANIFEST.json")?
        .iter()
        .enumerate()
    {
        validate_manifest_claim(claim, index)?;
    }
    Ok(())
}

pub(super) fn validate_red_fixtures(value: &Value) -> Result<()> {
    if !value.is_array() {
        bail!("RED_FIXTURES.json must be an array");
    }
    Ok(())
}
