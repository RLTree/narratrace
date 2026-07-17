use super::schema::{
    validate_backlog, validate_lane_registry, validate_manifest_shape, validate_red_fixtures,
};
use super::trusted_goal::TrustedGoalObservation;
use super::util::{bundle_artifact_path, read_json, read_text, require_array, required_string};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn schema_validators_reject_missing_required_shapes() {
    assert!(
        validate_lane_registry(&json!({"schema":"wrong"}))
            .unwrap_err()
            .to_string()
            .contains("LANE_REGISTRY.json must declare schema")
    );
    assert!(
        validate_backlog(&json!({
            "schema": "harness-ultragoal.verification-backlog.v1",
            "rows": []
        }))
        .unwrap_err()
        .to_string()
        .contains("manifest_digest")
    );
    assert!(
        validate_manifest_shape(
            &json!({
                "schema": "harness-ultragoal.completion-manifest.v1",
                "required_claim_ids": ["CLAIM-001"],
                "claims": []
            }),
            &TrustedGoalObservation::for_test(),
        )
        .unwrap_err()
        .to_string()
        .contains("contract_bundle_hash")
    );
    assert!(validate_red_fixtures(&json!({"not":"array"})).is_err());
}

#[test]
fn manifest_shape_rejects_non_string_claim_ids_and_claim_mismatch() {
    let mut value = minimal_manifest();
    value["required_claim_ids"] = json!(["CLAIM-001", 2]);

    assert!(
        validate_manifest_shape(&value, &TrustedGoalObservation::for_test())
            .unwrap_err()
            .to_string()
            .contains("required_claim_ids must contain strings")
    );

    let mut value = minimal_manifest();
    value["claims"] = json!([]);
    assert!(
        validate_manifest_shape(&value, &TrustedGoalObservation::for_test())
            .unwrap_err()
            .to_string()
            .contains("claims ids must exactly match")
    );
}

#[test]
fn util_helpers_report_missing_arrays_strings_and_invalid_json() {
    let value = json!({"name": 7, "items": "nope"});

    assert!(
        require_array(&value, "/items", "test.json")
            .unwrap_err()
            .to_string()
            .contains("must be an array")
    );
    assert!(
        required_string(&value, "/name", "test.json")
            .unwrap_err()
            .to_string()
            .contains("must be a string")
    );

    let root = unique_tmp("nrr-bundle-util-json");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("bad.json"), "{").unwrap();
    assert!(read_json(&root.join("bad.json")).is_err());
}

#[test]
fn bundle_artifact_path_maps_source_skill_paths_inside_plugin_package() {
    let root = unique_tmp("nrr-bundle-plugin-root");
    fs::create_dir_all(root.join(".codex-plugin")).unwrap();
    fs::write(root.join(".codex-plugin/plugin.json"), "{}").unwrap();

    let path = bundle_artifact_path(
        &root,
        ".codex/skills/narrated-record-replay/GOAL_CONTRACT.md",
    )
    .unwrap();

    assert_eq!(
        path,
        root.join("skills/narrated-record-replay/GOAL_CONTRACT.md")
    );
    assert!(bundle_artifact_path(&root, ".").is_err());
}

#[test]
fn read_text_rejects_non_utf8_regular_file() {
    let root = unique_tmp("nrr-bundle-non-utf8");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("bytes.bin");
    fs::write(&path, [0xff, 0xfe]).unwrap();

    assert!(
        read_text(&path)
            .unwrap_err()
            .to_string()
            .contains("as utf-8")
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

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    Path::new("/private/tmp").join(format!("{prefix}-{nanos}"))
}
