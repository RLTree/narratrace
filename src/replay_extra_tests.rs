use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn preview_replay_voice_rejects_missing_or_empty_bindings() {
    let root = unique_tmp("nrr-replay-voice-empty");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("replay-voice-parameters.json"),
        r#"{"segmentBindings":[]}"#,
    )
    .unwrap();
    let args = preview_args(&root);

    let error = preview_replay_voice(&args).unwrap_err().to_string();

    assert!(error.contains("no segmentBindings"));
}

#[test]
fn preview_replay_voice_requires_segment_bindings_array() {
    let root = unique_tmp("nrr-replay-voice-missing");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("replay-voice-parameters.json"), "{}").unwrap();
    let args = preview_args(&root);

    let error = preview_replay_voice(&args).unwrap_err().to_string();

    assert!(error.contains("missing segmentBindings"));
}

#[test]
fn replay_parameters_reject_oversize_and_excessive_bindings() {
    let root = unique_tmp("nrr-replay-extra-bounds");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("replay-voice-parameters.json");
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_REPLAY_PARAMETERS_BYTES + 1)
        .unwrap();
    assert!(
        read_json(&path)
            .unwrap_err()
            .to_string()
            .contains("byte limit")
    );

    let bindings = vec![serde_json::json!({}); MAX_REPLAY_BINDINGS + 1];
    fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({
            "segmentBindings": bindings
        }))
        .unwrap(),
    )
    .unwrap();
    let args = crate::config::parse_args_from([
        "nrr",
        "replay-voice-preview",
        "--session-dir",
        root.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap();
    assert!(
        preview_replay_voice(&args)
            .unwrap_err()
            .to_string()
            .contains("segment binding limit")
    );
}

#[test]
fn voice_cue_accepts_direct_timeline_fields_and_default_tones() {
    let cue = voice_cue(&json!({
        "startMs": 10,
        "endMs": 110,
        "voice": { "style": "calm", "pace": "slow", "emphasis": "low" }
    }))
    .unwrap();

    assert_eq!(cue["plannedDurationMs"], 125);
    assert_eq!(cue["previewInstruction"]["tone"], "steady and low-variance");
    assert_eq!(cue["previewInstruction"]["paceMultiplier"], 0.8);
    assert_eq!(cue["previewInstruction"]["emphasisGain"], 0.85);
}

#[test]
fn voice_cue_reports_missing_required_fields() {
    let error = voice_cue(&json!({
        "timelineBinding": { "startMs": 10, "endMs": 20 },
        "voice": { "style": "focused", "pace": "normal" }
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("missing emphasis"));
}

fn preview_args(root: &std::path::Path) -> crate::config::Args {
    crate::config::parse_args_from([
        "nrr",
        "replay-voice-preview",
        "--session-dir",
        root.to_str().unwrap(),
        "--i-consent-to-custom-runtime-paths",
    ])
    .unwrap()
}

fn unique_tmp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from("/private/tmp").join(format!("{prefix}-{nanos}"))
}
