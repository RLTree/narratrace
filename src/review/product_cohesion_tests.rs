use super::*;

#[test]
fn valid_replay_voice_plan_requires_all_contract_fields() {
    let valid = serde_json::json!({
        "schema": "narrated-record-replay.replay-voice-execution-plan.v1",
        "status": "dry-run-not-spoken",
        "cueCount": 0,
        "proofBoundary": {"speaksAudio": false},
        "cues": []
    });
    assert!(replay_voice_plan_valid(&valid));

    for bad in [
        serde_json::json!({"status": "dry-run-plan-generated", "cueCount": 0, "proofBoundary": {"speaksAudio": false}, "cues": []}),
        serde_json::json!({"schema": "narrated-record-replay.replay-voice-execution-plan.v1", "status": "", "cueCount": 0, "proofBoundary": {"speaksAudio": false}, "cues": []}),
        serde_json::json!({"schema": "narrated-record-replay.replay-voice-execution-plan.v1", "status": "dry-run-not-spoken", "proofBoundary": {"speaksAudio": false}, "cues": []}),
        serde_json::json!({"schema": "narrated-record-replay.replay-voice-execution-plan.v1", "status": "dry-run-not-spoken", "cueCount": 0, "cues": []}),
        serde_json::json!({"schema": "narrated-record-replay.replay-voice-execution-plan.v1", "status": "dry-run-not-spoken", "cueCount": 0, "proofBoundary": {"speaksAudio": false}}),
        serde_json::json!({"schema": "narrated-record-replay.replay-voice-execution-plan.v1", "status": "caller-chosen-success", "cueCount": 0, "proofBoundary": {"speaksAudio": false}, "cues": []}),
    ] {
        assert!(!replay_voice_plan_valid(&bad));
    }
}

#[test]
fn review_status_distinguishes_context_blocked_and_bounded_review() {
    assert_eq!(
        review_status(
            true,
            0,
            0,
            0,
            true,
            Some(false),
            true,
            true,
            false,
            true,
            true
        ),
        "requires-operator-review"
    );
    assert_eq!(
        review_status(
            true,
            1,
            0,
            0,
            true,
            Some(false),
            true,
            true,
            false,
            true,
            true
        ),
        "requires-operator-review"
    );
    assert_eq!(
        review_status(
            false,
            0,
            0,
            0,
            true,
            Some(false),
            true,
            true,
            false,
            true,
            true
        ),
        "blocked-missing-temporal-context"
    );
    assert_eq!(
        review_status(
            true,
            0,
            0,
            1,
            true,
            Some(false),
            true,
            true,
            false,
            true,
            true
        ),
        "blocked"
    );
    assert_eq!(
        review_status(
            true,
            0,
            0,
            0,
            true,
            Some(true),
            true,
            true,
            false,
            true,
            true
        ),
        "blocked"
    );
    assert_eq!(
        review_status(
            true,
            0,
            0,
            0,
            true,
            Some(false),
            true,
            true,
            true,
            true,
            true
        ),
        "blocked"
    );
}

#[test]
fn recovery_actions_surface_each_missing_or_risky_surface() {
    let actions = recovery_actions(
        false,
        2,
        false,
        false,
        Some(true),
        1,
        1,
        false,
        3,
        true,
        false,
    );

    assert!(actions.iter().any(|a| a.contains("temporal-context")));
    assert!(actions.iter().any(|a| a.contains("evidence-boundary")));
    assert!(actions.iter().any(|a| a.contains("replay-voice-preview")));
    assert!(actions.iter().any(|a| a.contains("no-audio dry-run")));
    assert!(actions.iter().any(|a| a.contains("conflict warnings")));
    assert!(actions.iter().any(|a| a.contains("out-of-window")));
    assert!(actions.iter().any(|a| a.contains("leak scan")));
    assert!(actions.iter().any(|a| a.contains("raw-local sensitive")));
    assert!(actions.iter().any(|a| a.contains("denser narration")));
    assert!(actions.iter().any(|a| a.contains("batch transcription")));
    assert!(actions.iter().any(|a| a.contains("dogfood artifacts")));
}

#[test]
fn recovery_actions_have_default_static_review_action() {
    let actions = recovery_actions(true, 0, true, true, Some(false), 0, 0, true, 0, false, true);

    assert_eq!(actions.len(), 1);
    assert!(actions[0].contains("static review artifact"));
}
