use super::claim_policy::validate_claim;
use super::trusted_goal::TrustedGoalObservation;
use anyhow::{Result, bail};
use serde_json::Value;

pub(super) fn validate_lane(value: &Value, index: usize) -> Result<()> {
    for key in [
        "id",
        "status",
        "lane_type",
        "lane_objective",
        "macro_lane_rationale",
        "owner",
        "thread_or_session_id",
        "branch",
        "workspace",
        "execplan",
        "base_commit",
        "current_commit",
        "target_branch",
        "target_head_at_launch",
        "target_head_at_validation",
        "merge_base_at_validation",
        "last_heartbeat",
        "artifact_root",
    ] {
        if value.get(key).and_then(Value::as_str).is_none() {
            bail!("LANE_REGISTRY.json#/lanes/{index} missing string {key}");
        }
    }
    for key in [
        "covered_claim_ids",
        "owned_paths",
        "forbidden_paths",
        "state_roots",
        "scratch_roots",
        "tool_cache_roots",
        "browser_profile_roots",
        "port_allocations",
        "claim_ids",
        "dependencies",
        "acceptance_scope",
    ] {
        if value.get(key).and_then(Value::as_array).is_none() {
            bail!("LANE_REGISTRY.json#/lanes/{index} missing array {key}");
        }
    }
    for key in [
        "actor_binding",
        "lane_size_evidence",
        "ready_receipt",
        "workspace_status",
        "teardown",
    ] {
        if value.get(key).and_then(Value::as_object).is_none() {
            bail!("LANE_REGISTRY.json#/lanes/{index} missing object {key}");
        }
    }
    Ok(())
}

pub(super) fn validate_backlog_row(value: &Value, index: usize) -> Result<()> {
    for key in [
        "id",
        "claim_id",
        "class",
        "owner",
        "required_proof",
        "blocker_type",
        "next_action",
        "claim_ceiling_impact",
    ] {
        if value.get(key).and_then(Value::as_str).is_none() {
            bail!("VERIFICATION_BACKLOG.json#/rows/{index} missing string {key}");
        }
    }
    if value.get("attempts").and_then(Value::as_array).is_none() {
        bail!("VERIFICATION_BACKLOG.json#/rows/{index} missing array attempts");
    }
    if value.get("evidence").and_then(Value::as_object).is_none() {
        bail!("VERIFICATION_BACKLOG.json#/rows/{index} missing object evidence");
    }
    Ok(())
}

pub(super) fn validate_goal_binding(value: &Value, trusted: &TrustedGoalObservation) -> Result<()> {
    trusted.validate_binding(value)
}

pub(super) fn validate_manifest_claim(value: &Value, index: usize) -> Result<()> {
    validate_claim(value, index)
}
