use super::schema::{validate_backlog, validate_lane_registry, validate_manifest_shape};
use super::trusted_goal::TrustedGoalObservation;
use serde_json::{Value, json};

#[test]
fn schema_validators_accept_minimal_valid_lane_and_backlog() {
    let backlog = json!({
        "schema": "harness-ultragoal.verification-backlog.v1",
        "manifest_digest": "sha256:test",
        "generated_at": "2026-06-25T00:00:00Z",
        "rows": [complete_backlog_row()]
    });
    let registry = json!({
        "schema": "harness-ultragoal.lane-registry.v1",
        "goal_id": "goal",
        "canonical_authority": "GOAL_CONTRACT.md",
        "generated_at": "2026-06-25T00:00:00Z",
        "lanes": [complete_lane()]
    });

    validate_lane_registry(&registry).unwrap();
    validate_backlog(&backlog).unwrap();
}

#[test]
fn manifest_shape_rejects_missing_goal_binding_or_claim_id() {
    let mut value = minimal_manifest();
    value.as_object_mut().unwrap().remove("goal_binding");
    assert!(
        validate_manifest_shape(&value, &TrustedGoalObservation::for_test())
            .unwrap_err()
            .to_string()
            .contains("goal_binding")
    );

    let mut value = minimal_manifest();
    value["claims"][0].as_object_mut().unwrap().remove("id");
    assert!(
        validate_manifest_shape(&value, &TrustedGoalObservation::for_test())
            .unwrap_err()
            .to_string()
            .contains("claims rows must have string id")
    );
}

fn minimal_manifest() -> Value {
    let claim_ids = (1..=13)
        .map(|index| format!("CLAIM-{index:03}"))
        .collect::<Vec<_>>();
    json!({
        "schema": "harness-ultragoal.completion-manifest.v1",
        "required_claim_ids": claim_ids,
        "claims": claim_ids.iter().map(|id| json!({
            "id": id,
            "title": "claim",
            "claim_scope": "static",
            "claim_kind": "proof",
            "status": "withheld_claim",
            "claim_surface": "source",
            "claim_ceiling_effect": "withheld",
            "allowed_evidence_surfaces": [],
            "evidence": [],
            "backlog_row_id": "BACKLOG-001"
        })).collect::<Vec<_>>(),
        "contract_bundle_hash": "sha256:test",
        "required_claim_ids_hash": "sha256:test",
        "commit": "NO-GIT",
        "root": "/repo",
        "verification_backlog_path": "VERIFICATION_BACKLOG.json",
        "verification_backlog_digest": "sha256:test",
        "generated_at": "2026-06-24T00:00:00Z",
        "goal_binding": {
            "status": "bound",
            "goal_id": "test-goal",
            "objective": "test objective",
            "contract_path": "GOAL_CONTRACT.md",
            "checked_at": "2026-07-17T00:00:00Z"
        }
    })
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
