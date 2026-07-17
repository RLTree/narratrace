use super::schema_rows::{
    validate_backlog_row, validate_goal_binding, validate_lane, validate_manifest_claim,
};
use super::trusted_goal::TrustedGoalObservation;
use serde_json::{Value, json};

#[test]
fn row_validators_report_missing_required_field_classes() {
    assert!(
        validate_lane(&json!({}), 0)
            .unwrap_err()
            .to_string()
            .contains("missing string id")
    );
    let mut lane = complete_lane();
    lane["covered_claim_ids"] = json!("nope");
    assert!(
        validate_lane(&lane, 1)
            .unwrap_err()
            .to_string()
            .contains("missing array covered_claim_ids")
    );
    let mut lane = complete_lane();
    lane["actor_binding"] = json!("nope");
    assert!(
        validate_lane(&lane, 2)
            .unwrap_err()
            .to_string()
            .contains("missing object actor_binding")
    );
    assert!(
        validate_backlog_row(&json!({}), 0)
            .unwrap_err()
            .to_string()
            .contains("missing string id")
    );
    let mut backlog = complete_backlog_row();
    backlog["attempts"] = json!("nope");
    assert!(
        validate_backlog_row(&backlog, 0)
            .unwrap_err()
            .to_string()
            .contains("missing array attempts")
    );
}

#[test]
fn manifest_claim_requires_evidence_and_backlog_for_unproven_status() {
    assert!(
        validate_goal_binding(&json!({}), &TrustedGoalObservation::for_test())
            .unwrap_err()
            .to_string()
            .contains("/goal_id")
    );
    let mut claim = json!({
        "id": "CLAIM-001",
        "title": "title",
        "claim_scope": "required",
        "claim_kind": "security",
        "status": "lane_owed",
        "claim_surface": "runtime_cli",
        "claim_ceiling_effect": "blocked",
        "allowed_evidence_surfaces": [],
        "evidence": []
    });
    assert!(
        validate_manifest_claim(&claim, 0)
            .unwrap_err()
            .to_string()
            .contains("backlog_row_id")
    );
    claim["backlog_row_id"] = json!("BACKLOG-001");
    claim["evidence"] = json!([{"id": "EVIDENCE-001"}]);
    assert!(
        validate_manifest_claim(&claim, 0)
            .unwrap_err()
            .to_string()
            .contains("missing string claim_id")
    );
}

fn complete_backlog_row() -> Value {
    json!({
        "id": "BACKLOG-001",
        "claim_id": "CLAIM-001",
        "class": "owed",
        "owner": "owner",
        "required_proof": "proof",
        "blocker_type": "blocked",
        "next_action": "act",
        "claim_ceiling_impact": "withheld",
        "attempts": [],
        "evidence": {}
    })
}

fn complete_lane() -> Value {
    let mut lane = json!({
        "actor_binding": {},
        "lane_size_evidence": {},
        "ready_receipt": {},
        "workspace_status": {},
        "teardown": {}
    });
    let object = lane.as_object_mut().unwrap();
    for key in STRING_LANE_KEYS {
        object.insert(key.to_string(), json!("value"));
    }
    for key in ARRAY_LANE_KEYS {
        object.insert(key.to_string(), json!([]));
    }
    lane
}

const STRING_LANE_KEYS: &[&str] = &[
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
];

const ARRAY_LANE_KEYS: &[&str] = &[
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
];
